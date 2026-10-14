//! Synchronous paid-diary kit restoration from FloorOneScript login callbacks.
use crate::{
    inventory::{Catalog, Insert},
    simulation::PlayerData,
    tower_diary::Progress,
};
use std::collections::BTreeSet;
#[derive(Clone, Debug)]
pub struct Item {
    pub id: i32,
    pub count: i32,
    /// Source ownsNamed or ownsType union, already resolved from packed types.
    pub owned: BTreeSet<i32>,
    pub waiting: &'static str,
    pub restored: &'static str,
}
impl Item {
    fn owned(&self, p: &PlayerData) -> bool {
        [&p.inventory, &p.equipment, &p.bank]
            .into_iter()
            .any(|inv| {
                inv.slots()
                    .iter()
                    .flatten()
                    .any(|i| self.owned.contains(&i.id))
            })
    }
    fn restore(&self, p: &mut PlayerData, catalog: &Catalog, out: &mut Vec<&'static str>) {
        if self.owned(p) {
            return;
        }
        if p.inventory.free_space() == 0 {
            out.push(self.waiting);
            return;
        }
        let _ = p
            .inventory
            .insert(catalog, Insert::new(self.id, self.count));
        out.push(self.restored);
    }
}
#[derive(Clone, Debug)]
pub struct Config {
    pub axe: Item,
    pub needle: Item,
    pub thread: Item,
    pub pickaxe: Item,
    pub tinderbox: Item,
    pub scimitar: Item,
    pub net: Item,
    pub knife: Item,
    pub hammer: Item,
}
impl Config {
    /// Called after staff restoration and before food withdrawal. Unknown diary
    /// must be handled by the caller, never silently interpreted as unpaid.
    pub fn before_food(
        &self,
        p: &mut PlayerData,
        catalog: &Catalog,
        diary: Progress,
    ) -> Vec<&'static str> {
        let d = diary.done;
        let mut messages = Vec::new();
        if d[0] {
            self.axe.restore(p, catalog, &mut messages);
        }
        if d[1] {
            self.craft(p, catalog, &mut messages);
        }
        if d[2] {
            self.pickaxe.restore(p, catalog, &mut messages);
        }
        if d[3] {
            self.tinderbox.restore(p, catalog, &mut messages);
            self.scimitar.restore(p, catalog, &mut messages);
            // noteScimitarIfHeld repeats the tinderbox check, including a
            // second waiting message when the first call ran out of space.
            self.tinderbox.restore(p, catalog, &mut messages);
        }
        if d[4] {
            self.net.restore(p, catalog, &mut messages);
        }
        if d[5] && diary.bounty != 6 {
            self.knife.restore(p, catalog, &mut messages);
            self.knife.restore(p, catalog, &mut messages); // noteShrimpIfHeld
        }
        messages
    }
    fn craft(&self, p: &mut PlayerData, catalog: &Catalog, messages: &mut Vec<&'static str>) {
        let mut restored = false;
        for item in [&self.needle, &self.thread] {
            if item.owned(p) {
                continue;
            }
            if p.inventory.free_space() == 0 {
                messages.push(item.waiting);
                return;
            }
            let _ = p
                .inventory
                .insert(catalog, Insert::new(item.id, item.count));
            restored = true;
        }
        if restored {
            messages.push("You still have the needle and thread.");
        }
    }
    /// Source order is food, talisman, hammer. Invoke after talisman processing.
    pub fn after_talisman(
        &self,
        p: &mut PlayerData,
        catalog: &Catalog,
        diary: Progress,
    ) -> Vec<&'static str> {
        let mut messages = Vec::new();
        if diary.done[6] {
            self.hammer.restore(p, catalog, &mut messages);
        }
        messages
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::{Inventory, StackType},
        stats::Stats,
        CoordGrid,
    };
    fn setup(size: usize) -> (Config, PlayerData, Catalog) {
        fn item(id: i32) -> Item {
            Item {
                id,
                count: 1,
                owned: [id].into(),
                waiting: "waiting",
                restored: "restored",
            }
        }
        let c = Config {
            axe: item(10),
            needle: item(11),
            thread: Item {
                count: 4,
                ..item(12)
            },
            pickaxe: item(13),
            tinderbox: item(14),
            scimitar: item(15),
            net: item(16),
            knife: item(17),
            hammer: item(18),
        };
        let mut p = PlayerData::new(
            1,
            "Restore".into(),
            CoordGrid::new(10424, 120, 0).unwrap(),
            Stats::default(),
        );
        p.inventory = Inventory::new(size, StackType::Normal, false);
        let mut cat = Catalog::default();
        cat.stackable.insert(12);
        (c, p, cat)
    }
    #[test]
    fn unpaid_rows_do_not_grant_preexisting_item_credit_or_kit() {
        let (c, mut p, cat) = setup(28);
        assert!(c.before_food(&mut p, &cat, Progress::default()).is_empty());
        assert!(p.inventory.slots().iter().all(Option::is_none));
    }
    #[test]
    fn craft_pair_can_leave_only_needle_and_waiting_message() {
        let (c, mut p, cat) = setup(1);
        let mut diary = Progress::default();
        diary.done[1] = true;
        assert_eq!(c.before_food(&mut p, &cat, diary), ["waiting"]);
        assert_eq!(p.inventory.get(0).unwrap().id, 11);
    }
    #[test]
    fn full_pack_preserves_repeated_source_calls_and_knife_bounty_gate() {
        let (c, mut p, cat) = setup(0);
        let mut diary = Progress::default();
        diary.done[3] = true;
        diary.done[5] = true;
        assert_eq!(c.before_food(&mut p, &cat, diary), ["waiting"; 5]);
        diary.bounty = 6;
        assert_eq!(c.before_food(&mut p, &cat, diary), ["waiting"; 3]);
    }
    #[test]
    fn full_paid_kit_follows_source_order_and_repeats_do_not_duplicate() {
        let (c, mut p, cat) = setup(28);
        let diary = Progress {
            done: [true; 11],
            ..Default::default()
        };
        c.before_food(&mut p, &cat, diary);
        c.after_talisman(&mut p, &cat, diary);
        let items: Vec<_> = p
            .inventory
            .slots()
            .iter()
            .flatten()
            .map(|i| (i.id, i.count))
            .collect();
        assert_eq!(
            items,
            [
                (10, 1),
                (11, 1),
                (12, 4),
                (13, 1),
                (14, 1),
                (15, 1),
                (16, 1),
                (17, 1),
                (18, 1)
            ]
        );
        assert!(c.before_food(&mut p, &cat, diary).is_empty());
        assert!(c.after_talisman(&mut p, &cat, diary).is_empty());
    }
}

#[derive(Clone, Debug)]
pub struct HandlerConfig {
    pub timer: u16,
    pub agility: u8,
    pub mind_runes: BTreeSet<i32>,
    pub origin: crate::CoordGrid,
    pub skills: [u8; 3],
    pub bones: u16,
    pub kit: crate::tower_kit::Config,
    pub leather: BTreeSet<i32>,
    pub staff: crate::tower_staff::Config,
    pub rewards: Config,
    pub food: crate::tower_food::Config,
}
