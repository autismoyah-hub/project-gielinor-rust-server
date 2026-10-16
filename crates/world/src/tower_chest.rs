//! Tower bounty rewards commit as one inventory transaction. A failed claim
//! leaves both inventory and diary payment unchanged, so retries cannot mint items.
use crate::{
    inventory::{Catalog, Insert, Inventory},
    tower_diary::{Marks, Sync},
};
use std::collections::BTreeSet;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Reward {
    Axe,
    Needle,
    Thread,
    Pickaxe,
    Tinderbox,
    Net,
    Knife,
    Hammer,
    Potion,
    Arrow,
    Coins,
    Beer,
}
impl Reward {
    pub fn name(self) -> &'static str {
        match self {
            Self::Axe => "Bronze axe",
            Self::Needle => "Needle",
            Self::Thread => "Thread",
            Self::Pickaxe => "Bronze pickaxe",
            Self::Tinderbox => "Tinderbox",
            Self::Net => "Small fishing net",
            Self::Knife => "Knife",
            Self::Hammer => "Hammer",
            Self::Potion => "Attack potion(3)",
            Self::Arrow => "Bronze arrow",
            Self::Coins => "Coins",
            Self::Beer => "Beer",
        }
    }
}
pub fn items(row: i32) -> &'static [(Reward, i32)] {
    use Reward::*;
    match row {
        1 => &[(Axe, 1)],
        2 => &[(Needle, 1), (Thread, 4)],
        3 => &[(Pickaxe, 1)],
        4 => &[(Tinderbox, 1)],
        5 => &[(Net, 1)],
        6 => &[(Knife, 1)],
        7 => &[(Hammer, 1)],
        8 => &[(Potion, 1)],
        9 => &[(Arrow, 15)],
        10 => &[(Coins, 8)],
        11 => &[(Coins, 50), (Beer, 1)],
        _ => &[],
    }
}
pub fn waiting(row: i32) -> &'static str {
    match row {
        1 => "Your bronze axe is waiting. Drop something.",
        2 => "Your needle is waiting. Drop something.",
        3 => "Your bronze pickaxe is waiting. Drop something.",
        4 => "Your tinderbox is waiting. Drop something.",
        5 => "Your small fishing net is waiting. Drop something.",
        6 => "Your knife is waiting. Drop something.",
        7 => "Your hammer is waiting. Drop something.",
        8 => "Your attack potion is waiting. Drop something.",
        9 => "Your bronze arrows are waiting. Drop something.",
        10 | 11 => "Your coins are waiting. Drop something.",
        _ => "The chest is empty.",
    }
}
pub struct HandlerConfig {
    pub location: u32,
    pub origin: crate::CoordGrid,
    pub skills: [u8; 3],
    pub bones: u16,
    pub inventory: Config,
}
/// Kotlin's case-insensitive name comparison against these ASCII reward names.
/// Single-character upper/lower mappings cover dotted/dotless I, long S and
/// Kelvin sign; multi-character uppercase expansion is not a Java Char mapping.
pub fn name_matches(name: &str, target: &str) -> bool {
    assert!(target.is_ascii());
    fn folded(c: char) -> char {
        let mut upper = c.to_uppercase();
        let first = upper.next().unwrap();
        let upper = if upper.next().is_none() { first } else { c };
        upper.to_lowercase().next().unwrap()
    }
    name.chars().count() == target.len()
        && name
            .chars()
            .zip(target.chars())
            .all(|(a, b)| folded(a) == folded(b))
}
pub struct Config {
    /// Canonical grant IDs, in Reward order.
    pub ids: [i32; 12],
    /// Non-certificate name aliases for Thread, Bronze arrow, Coins. Original
    /// ownsNamed scans inventory, equipment AND bank during its space precheck.
    pub merge_aliases: [BTreeSet<i32>; 3],
}
#[derive(Debug)]
pub struct Outcome {
    /// None means neither inventory grants nor diary payment committed.
    pub sync: Option<Sync>,
    /// Sent before the optional diary refresh/synchronization effects.
    pub messages: Vec<String>,
}
impl Config {
    pub fn claim(
        &self,
        inventory: &mut Inventory,
        worn: &Inventory,
        bank: &Inventory,
        catalog: &Catalog,
        marks: Marks,
    ) -> Outcome {
        // Kotlin reads the stored bounty without resynchronizing first.
        let row = marks.bounty;
        let mut out = Outcome {
            sync: None,
            messages: Vec::new(),
        };
        if row == 0 {
            out.messages.push("The chest is empty.".into());
            return out;
        }
        let stacks = items(row);
        let missing = stacks
            .iter()
            .filter(|(reward, _)| {
                let aliases = match reward {
                    Reward::Thread => Some(&self.merge_aliases[0]),
                    Reward::Arrow => Some(&self.merge_aliases[1]),
                    Reward::Coins => Some(&self.merge_aliases[2]),
                    _ => None,
                };
                !aliases.is_some_and(|aliases| {
                    inventory
                        .slots()
                        .iter()
                        .chain(worn.slots())
                        .chain(bank.slots())
                        .flatten()
                        .any(|item| aliases.contains(&item.id))
                })
            })
            .count();
        if missing > inventory.free_space() {
            out.messages.push(waiting(row).into());
            return out;
        }
        let mut staged = inventory.clone();
        for &(reward, count) in stacks {
            if staged
                .insert(catalog, Insert::new(self.ids[reward as usize], count))
                .is_err()
            {
                out.messages.clear();
                out.messages.push(waiting(row).into());
                return out;
            }
            let name = reward.name();
            out.messages.push(if count == 1 {
                format!("The chest yields {name}.")
            } else {
                format!("The chest yields {count} {name}.")
            });
        }
        *inventory = staged;
        out.sync = Some(marks.pay().session_sync());
        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    fn inv(size: usize) -> Inventory {
        Inventory::new(size, StackType::Normal, false)
    }
    fn config() -> (Config, Catalog) {
        let mut catalog = Catalog::default();
        catalog.stackable.extend([2, 9, 10]);
        (
            Config {
                ids: std::array::from_fn(|i| i as i32),
                merge_aliases: [[2].into(), [9].into(), [10].into()],
            },
            catalog,
        )
    }
    #[test]
    fn row_one_grants_axe_and_marks_paid_only_once() {
        let (config, catalog) = config();
        let mut inventory = inv(2);
        let marks = Marks {
            counts: [5, 0, 0, 0, 0, 0],
            bounty: 1,
            schema11: true,
            ..Default::default()
        };
        let outcome = config.claim(&mut inventory, &inv(0), &inv(0), &catalog, marks);
        assert_eq!(outcome.messages, ["The chest yields Bronze axe."]);
        let next = outcome.sync.unwrap().marks;
        assert!(next.done[0]);
        assert_eq!(next.bounty, 0);
        assert_eq!(inventory.get(0).unwrap().id, 0);
        assert_eq!(
            config
                .claim(&mut inventory, &inv(0), &inv(0), &catalog, next)
                .messages,
            ["The chest is empty."]
        );
        assert_eq!(inventory.free_space(), 1);
    }
    #[test]
    fn bank_coin_precheck_cannot_mint_partial_rewards_on_retries() {
        let (config, catalog) = config();
        let mut inventory = inv(1);
        let mut bank = inv(1);
        bank.restore_slot(
            0,
            Some(Item {
                id: 10,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        let mut marks = Marks {
            bounty: 11,
            schema11: true,
            pending: [false, true],
            ..Default::default()
        };
        marks.done[..10].fill(true);
        let before = inventory.clone();
        for _ in 0..3 {
            let outcome = config.claim(&mut inventory, &inv(0), &bank, &catalog, marks);
            assert!(outcome.sync.is_none());
            assert_eq!(outcome.messages, ["Your coins are waiting. Drop something."]);
            assert_eq!(inventory, before);
        }
        assert_eq!(bank.get(0).unwrap().count, 1);
    }
    #[test]
    fn room_precheck_rejects_both_stacks_before_inserting_needle() {
        let (config, catalog) = config();
        let mut inventory = inv(1);
        let outcome = config.claim(
            &mut inventory,
            &inv(0),
            &inv(0),
            &catalog,
            Marks {
                bounty: 2,
                ..Default::default()
            },
        );
        assert!(outcome.sync.is_none());
        assert_eq!(inventory.free_space(), 1);
        assert_eq!(
            outcome.messages,
            ["Your needle is waiting. Drop something."]
        );
    }
}
