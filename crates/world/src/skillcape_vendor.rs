//! Fixed guild cape sales reuse Inventory transactions and the shared dialogue engine.
use crate::{
    inventory::{Catalog, Delete, Insert, Inventory, StackType},
    stats::Stats,
    CoordGrid,
};
pub const PRICE: i32 = 99_000;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Crafting,
    Cooking,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub kind: Kind,
    pub npc: u16,
    pub skill: u8,
    pub coins: i32,
    pub capes: [i32; 2],
    pub hood: i32,
    pub active_stats: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Context {
    pub config: Config,
    pub origin: CoordGrid,
    pub last_movement: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    About,
    Offer,
    Owned,
    Quote,
    Purchase,
    HoodOffer,
    Hood,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Policy,
    Members,
    Level,
    Money,
    Space,
    OwnCape,
    OwnHood,
    NeedCape,
}
pub struct Prepared {
    pub inventory: Inventory,
    pub output: i32,
}
impl Error {
    pub fn message(self) -> &'static str {
        match self {
            Self::Members => "Membership is required to obtain a skillcape or replacement hood.",
            Self::Level => "You need a base level of 99 in this skill to obtain its cape or hood.",
            Self::Money => "You need 99,000 coins in your inventory.",
            Self::Space => "Please make enough room in your inventory for the cape and hood.",
            Self::OwnCape => "You already have a Crafting cape with you.",
            Self::OwnHood => "You already have a matching hood with you.",
            Self::NeedCape => "Bring your cape with you to request a replacement hood.",
            Self::Policy => "Your items are not suitable for this exchange right now.",
        }
    }
}
impl Config {
    fn validate(
        self,
        inv: &Inventory,
        worn: &Inventory,
        items: &Catalog,
        stats: &Stats,
        member: bool,
    ) -> Result<(), Error> {
        if self.skill >= 24
            || self.npc == 0
            || self.active_stats & !0x00ff_ffff != 0
            || self.active_stats & (1u32 << self.skill) == 0
            || [self.coins, self.capes[0], self.capes[1], self.hood]
                .iter()
                .any(|&i| i < 0)
            || [self.coins, self.capes[0], self.capes[1], self.hood]
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != 4
            || inv.slots().len() != 28
            || inv.stack != StackType::Normal
            || inv.placeholders
            || worn.stack != StackType::Normal
            || worn.placeholders
        {
            return Err(Error::Policy);
        }
        if !member {
            return Err(Error::Members);
        }
        if (stats.base(self.skill) as u8) < 99 {
            return Err(Error::Level);
        }
        if !self
            .capes
            .into_iter()
            .chain([self.hood])
            .all(|i| items.ordinary_nonstackable(i))
            || !items.stackable.contains(&self.coins)
            || items.dummy.contains(&self.coins)
            || items
                .notes
                .get(&self.coins)
                .is_some_and(|t| t.template != 0)
            || items
                .placeholders
                .get(&self.coins)
                .is_some_and(|t| t.template != 0)
            || items
                .transforms
                .get(&self.coins)
                .is_some_and(|t| t.template != 0)
        {
            return Err(Error::Policy);
        }
        for item in inv
            .slots()
            .iter()
            .chain(worn.slots())
            .flatten()
            .filter(|i| self.capes.contains(&i.id) || i.id == self.hood)
        {
            if item.count != 1 || item.vars != 0 {
                return Err(Error::Policy);
            }
        }
        for (slot, item) in worn
            .slots()
            .iter()
            .enumerate()
            .filter_map(|(s, i)| i.map(|i| (s, i)))
        {
            if self.capes.contains(&item.id) && slot != 1 || item.id == self.hood && slot != 0 {
                return Err(Error::Policy);
            }
        }
        Ok(())
    }
    pub fn owns_cape(self, inv: &Inventory, worn: &Inventory) -> bool {
        inv.slots()
            .iter()
            .flatten()
            .chain(worn.get(1).iter())
            .any(|i| self.capes.contains(&i.id) && i.count == 1 && i.vars == 0)
    }
    fn owns_hood(self, inv: &Inventory, worn: &Inventory) -> bool {
        inv.slots()
            .iter()
            .flatten()
            .chain(worn.get(0).iter())
            .any(|i| i.id == self.hood && i.count == 1 && i.vars == 0)
    }
    pub fn prepare_purchase(
        self,
        inv: &Inventory,
        worn: &Inventory,
        items: &Catalog,
        stats: &Stats,
        member: bool,
    ) -> Result<Prepared, Error> {
        self.validate(inv, worn, items, stats, member)?;
        if self.kind == Kind::Crafting && self.owns_cape(inv, worn) {
            return Err(Error::OwnCape);
        }
        let mut coins = inv
            .slots()
            .iter()
            .enumerate()
            .filter_map(|(s, i)| i.filter(|i| i.id == self.coins).map(|i| (s, i)));
        let (slot, stock) = coins.next().ok_or(Error::Money)?;
        if coins.next().is_some() || stock.count <= 0 || stock.vars != 0 {
            return Err(Error::Policy);
        }
        if stock.count < PRICE {
            return Err(Error::Money);
        }
        // Source requires two free slots before the coin debit, even if exact payment frees one.
        if inv.free_space() < 2 {
            return Err(Error::Space);
        }
        let trimmed = (0u8..24).any(|s| {
            s != self.skill && self.active_stats & (1u32 << s) != 0 && (stats.base(s) as u8) >= 99
        });
        let output = self.capes[usize::from(trimmed)];
        let mut next = inv.clone();
        let mut debit = Delete::new(self.coins, PRICE);
        debit.strict_slot = Some(slot);
        if next
            .delete(items, debit)
            .map_err(|_| Error::Money)?
            .completed
            != PRICE
        {
            return Err(Error::Money);
        }
        for id in [output, self.hood] {
            if next
                .insert(items, Insert::new(id, 1))
                .map_err(|_| Error::Space)?
                .completed
                != 1
            {
                return Err(Error::Space);
            }
        }
        Ok(Prepared {
            inventory: next,
            output,
        })
    }
    pub fn prepare_hood(
        self,
        inv: &Inventory,
        worn: &Inventory,
        items: &Catalog,
        stats: &Stats,
        member: bool,
    ) -> Result<Prepared, Error> {
        self.validate(inv, worn, items, stats, member)?;
        if !self.owns_cape(inv, worn) {
            return Err(Error::NeedCape);
        }
        if self.kind == Kind::Crafting && self.owns_hood(inv, worn) {
            return Err(Error::OwnHood);
        }
        if inv.free_space() < 1 {
            return Err(Error::Space);
        }
        let mut next = inv.clone();
        if next
            .insert(items, Insert::new(self.hood, 1))
            .map_err(|_| Error::Space)?
            .completed
            != 1
        {
            return Err(Error::Space);
        }
        Ok(Prepared {
            inventory: next,
            output: self.hood,
        })
    }
}
use crate::dialogue::{Conversation, Effect, Expression, Page, Speaker};
const ABOUT: &[&str] = &["How can I earn a skillcape?", "Goodbye."];
const OFFER: &[&str] = &["No thanks.", "Yes, please."];
const OWNED: &[&str] = &["Skillcape", "Hood"];
const BUY: &[&str] = &["No thanks.", "Buy the cape and hood for 99,000 coins."];
fn npc(text: &'static str) -> Page {
    Page::Chat {
        speaker: Speaker::Npc,
        expression: Expression::Neutral,
        text,
    }
}
fn player(text: &'static str) -> Page {
    Page::Chat {
        speaker: Speaker::Player,
        expression: Expression::Quiz,
        text,
    }
}
fn conversation(ctx: Context, stage: Stage, pages: impl IntoIterator<Item = Page>) -> Conversation {
    Conversation {
        pages: pages.into_iter().collect(),
        after: Effect::SkillcapeVendor { ctx, stage },
    }
}
pub fn message(text: &'static str) -> Conversation {
    Conversation {
        pages: [npc(text)].into(),
        after: Effect::None,
    }
}
pub fn closed() -> Conversation {
    Conversation {
        pages: Default::default(),
        after: Effect::None,
    }
}
pub fn quote(ctx: Context) -> Conversation {
    conversation(ctx,Stage::Quote,[npc("A cape with its matching hood costs 99,000 coins. You need two free inventory slots."),Page::Choice(BUY)])
}
pub fn owned(ctx: Context) -> Conversation {
    conversation(
        ctx,
        Stage::Owned,
        [
            npc("Would you like to ask about your skillcape or its hood?"),
            Page::Choice(OWNED),
        ],
    )
}
pub fn purchase_ack(ctx: Context) -> Conversation {
    conversation(
        ctx,
        Stage::Purchase,
        [player("Sure, I would like to buy the cape and hood.")],
    )
}
pub fn hood_question(ctx: Context) -> Conversation {
    conversation(
        ctx,
        Stage::HoodOffer,
        [player("May I have another hood for my cape, please?")],
    )
}
pub fn hood_ack(ctx: Context) -> Conversation {
    conversation(
        ctx,
        Stage::Hood,
        [npc(
            "I can provide a matching replacement hood free of charge.",
        )],
    )
}
pub fn begin(
    ctx: Context,
    inv: &Inventory,
    worn: &Inventory,
    stats: &Stats,
    member: bool,
) -> Conversation {
    if !member {
        return message(Error::Members.message());
    }
    if (stats.base(ctx.config.skill) as u8) < 99 {
        return conversation(
            ctx,
            Stage::About,
            [
                npc("Welcome to the guild. A skillcape recognizes mastery of a skill."),
                Page::Choice(ABOUT),
            ],
        );
    }
    match ctx.config.kind {
        Kind::Crafting => {
            if ctx.config.owns_cape(inv, worn) {
                owned(ctx)
            } else {
                quote(ctx)
            }
        }
        Kind::Cooking => conversation(
            ctx,
            Stage::Offer,
            [
                npc("Welcome, master chef. Would you like to ask about a Cooking cape?"),
                Page::Choice(OFFER),
            ],
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::Item;
    fn fixture(kind: Kind) -> (Config, Inventory, Inventory, Catalog, Stats) {
        let (skill, capes, hood, npc) = if kind == Kind::Crafting {
            (12, [9780, 9781], 9782, 5810)
        } else {
            (7, [9801, 9802], 9803, 2658)
        };
        let c = Config {
            kind,
            npc,
            skill,
            coins: 995,
            capes,
            hood,
            active_stats: (1 << skill) | 1,
        };
        let mut inv = Inventory::new(28, StackType::Normal, false);
        inv.restore_slot(
            0,
            Some(Item {
                id: 995,
                count: 100_000,
                vars: 0,
            }),
        )
        .unwrap();
        for slot in 1..26 {
            inv.restore_slot(
                slot,
                Some(Item {
                    id: 590,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let worn = Inventory::new(15, StackType::Normal, false);
        let mut items = Catalog::default();
        items.stackable.insert(995);
        let mut stats = Stats::default();
        stats.set_base(skill, 99);
        stats.set_current(skill, 99);
        stats
            .set_xp(skill, crate::skills::xp_from_level(99).unwrap())
            .unwrap();
        (c, inv, worn, items, stats)
    }
    #[test]
    fn vendor_dialogue_purchase_stages_both_outputs_fee_and_target_active_base99_trim_only() {
        for kind in [Kind::Crafting, Kind::Cooking] {
            for trimmed in [false, true] {
                let (c, inv, worn, items, mut stats) = fixture(kind);
                if trimmed {
                    stats.set_base(0, 99);
                } else {
                    stats.set_base(0, 98);
                    stats.set_current(0, 100);
                    stats.set_base(40, 99);
                }
                let before = (inv.clone(), worn.clone(), stats.clone());
                let p = c
                    .prepare_purchase(&inv, &worn, &items, &stats, true)
                    .unwrap();
                assert_eq!(p.output, c.capes[usize::from(trimmed)]);
                assert_eq!(p.inventory.get(0).unwrap().count, 1000);
                assert_eq!(p.inventory.get(26).unwrap().id, p.output);
                assert_eq!(p.inventory.get(27).unwrap().id, c.hood);
                assert_eq!(p.inventory.free_space(), 0);
                assert_eq!((inv, worn, stats), before);
            }
        }
    }
    #[test]
    fn vendor_dialogue_purchase_money_space_membership_metadata_and_late_catalog_fail_atomically() {
        for kind in [Kind::Crafting, Kind::Cooking] {
            for case in 0..10 {
                let (mut c, mut inv, mut worn, mut items, mut stats) = fixture(kind);
                let mut member = true;
                match case {
                    0 => member = false,
                    1 => {
                        stats.set_base(c.skill, 98);
                        stats.set_current(c.skill, 100);
                    }
                    2 => {
                        inv.restore_slot(
                            0,
                            Some(Item {
                                id: 995,
                                count: 98_999,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    }
                    3 => {
                        inv.restore_slot(
                            0,
                            Some(Item {
                                id: 995,
                                count: PRICE,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                        inv.restore_slot(
                            26,
                            Some(Item {
                                id: 590,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    }
                    4 => {
                        inv.restore_slot(
                            0,
                            Some(Item {
                                id: 995,
                                count: 100_000,
                                vars: 9,
                            }),
                        )
                        .unwrap();
                    }
                    5 => {
                        inv.restore_slot(
                            26,
                            Some(Item {
                                id: 995,
                                count: 100_000,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    }
                    6 => {
                        items.dummy.insert(c.hood);
                    }
                    7 => {
                        items.stackable.insert(c.capes[0]);
                    }
                    8 => c.active_stats = 0,
                    9 => {
                        worn.restore_slot(
                            4,
                            Some(Item {
                                id: c.capes[0],
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    }
                    _ => unreachable!(),
                }
                let before = (inv.clone(), worn.clone(), stats.clone());
                assert!(
                    c.prepare_purchase(&inv, &worn, &items, &stats, member)
                        .is_err(),
                    "{kind:?}/{case}"
                );
                assert_eq!((inv, worn, stats), before);
            }
        }
    }
    #[test]
    fn vendor_dialogue_owned_cape_and_hood_source_policies_keep_bank_out_of_scope() {
        for kind in [Kind::Crafting, Kind::Cooking] {
            let (c, mut inv, mut worn, items, stats) = fixture(kind);
            worn.restore_slot(
                1,
                Some(Item {
                    id: c.capes[0],
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
            assert_eq!(
                c.prepare_purchase(&inv, &worn, &items, &stats, true)
                    .is_ok(),
                kind == Kind::Cooking
            );
            let p = c.prepare_hood(&inv, &worn, &items, &stats, true).unwrap();
            assert_eq!(p.inventory.get(26).unwrap().id, c.hood);
            assert_eq!(p.inventory.get(0), inv.get(0));
            inv = p.inventory;
            assert_eq!(
                c.prepare_hood(&inv, &worn, &items, &stats, true).is_ok(),
                kind == Kind::Cooking
            );
            worn.restore_slot(1, None).unwrap();
            assert!(matches!(
                c.prepare_hood(&inv, &worn, &items, &stats, true),
                Err(Error::NeedCape)
            ));
        }
    }
}
