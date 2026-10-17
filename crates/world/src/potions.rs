//! PotionDrinks and RecoveryDrinks, preserving dose order and replacement semantics.
use crate::{
    inventory::{Catalog, Delete, Insert, Inventory},
    stat_ops::{self, Operation, Update},
    stats::Stats,
};

pub const EMPTY_VIAL: &str = "vial_empty";
pub const EMPTY_MESSAGE: &str = "You empty the contents of the vial on the floor.";
pub const DRINK_DELAY: i32 = 3;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    Boost {
        skill: &'static str,
        constant: i32,
        percent: i32,
    },
    Recover {
        skill: &'static str,
        constant: i32,
        percent: i32,
    },
    Energy(i32),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dose {
    pub name: &'static str,
    pub symbol: &'static str,
    pub next: &'static str,
    pub left: u8,
    pub effects: Vec<Effect>,
}
const RECOVERY_STATS: [&str; 21] = [
    "attack",
    "defence",
    "strength",
    "ranged",
    "magic",
    "cooking",
    "woodcutting",
    "fletching",
    "fishing",
    "firemaking",
    "crafting",
    "smithing",
    "mining",
    "herblore",
    "agility",
    "thieving",
    "slayer",
    "farming",
    "runecrafting",
    "hunter",
    "construction",
];
pub fn catalog() -> Vec<Dose> {
    let mut doses = Vec::new();
    let mut chain = |name, symbols: [&'static str; 4], effects: Vec<Effect>| {
        for i in 0..4 {
            doses.push(Dose {
                name,
                symbol: symbols[i],
                next: symbols.get(i + 1).copied().unwrap_or(EMPTY_VIAL),
                left: 3 - i as u8,
                effects: effects.clone(),
            });
        }
    };
    for (name, symbols, skills, constant, percent) in [
        (
            "attack potion",
            [
                "4dose1attack",
                "3dose1attack",
                "2dose1attack",
                "1dose1attack",
            ],
            &["attack"][..],
            3,
            10,
        ),
        (
            "strength potion",
            [
                "strength4",
                "3dose1strength",
                "2dose1strength",
                "1dose1strength",
            ],
            &["strength"][..],
            3,
            10,
        ),
        (
            "defence potion",
            [
                "4dose1defense",
                "3dose1defense",
                "2dose1defense",
                "1dose1defense",
            ],
            &["defence"][..],
            3,
            10,
        ),
        (
            "combat potion",
            ["4dosecombat", "3dosecombat", "2dosecombat", "1dosecombat"],
            &["attack", "strength"][..],
            3,
            10,
        ),
        (
            "super attack",
            [
                "4dose2attack",
                "3dose2attack",
                "2dose2attack",
                "1dose2attack",
            ],
            &["attack"][..],
            5,
            15,
        ),
        (
            "super strength",
            [
                "4dose2strength",
                "3dose2strength",
                "2dose2strength",
                "1dose2strength",
            ],
            &["strength"][..],
            5,
            15,
        ),
        (
            "super defence",
            [
                "4dose2defense",
                "3dose2defense",
                "2dose2defense",
                "1dose2defense",
            ],
            &["defence"][..],
            5,
            15,
        ),
        (
            "super combat potion",
            [
                "4dose2combat",
                "3dose2combat",
                "2dose2combat",
                "1dose2combat",
            ],
            &["attack", "strength", "defence"][..],
            5,
            15,
        ),
        (
            "ranging potion",
            [
                "4doserangerspotion",
                "3doserangerspotion",
                "2doserangerspotion",
                "1doserangerspotion",
            ],
            &["ranged"][..],
            4,
            10,
        ),
        (
            "magic potion",
            ["4dose1magic", "3dose1magic", "2dose1magic", "1dose1magic"],
            &["magic"][..],
            5,
            0,
        ),
        (
            "agility potion",
            [
                "4dose1agility",
                "3dose1agility",
                "2dose1agility",
                "1dose1agility",
            ],
            &["agility"][..],
            3,
            0,
        ),
        (
            "fishing potion",
            [
                "4dosefisherspotion",
                "3dosefisherspotion",
                "2dosefisherspotion",
                "1dosefisherspotion",
            ],
            &["fishing"][..],
            3,
            0,
        ),
        (
            "hunter potion",
            [
                "4dosehunting",
                "3dosehunting",
                "2dosehunting",
                "1dosehunting",
            ],
            &["hunter"][..],
            3,
            0,
        ),
        (
            "magic essence",
            [
                "4dosemagicess",
                "3dosemagicess",
                "2dosemagicess",
                "1dosemagicess",
            ],
            &["magic"][..],
            3,
            0,
        ),
    ] {
        chain(
            name,
            symbols,
            skills
                .iter()
                .map(|&skill| Effect::Boost {
                    skill,
                    constant,
                    percent,
                })
                .collect(),
        );
    }
    for (name, symbols, kind) in [
        (
            "prayer potion",
            [
                "4doseprayerrestore",
                "3doseprayerrestore",
                "2doseprayerrestore",
                "1doseprayerrestore",
            ],
            0,
        ),
        (
            "restore potion",
            [
                "4dosestatrestore",
                "3dosestatrestore",
                "2dosestatrestore",
                "1dosestatrestore",
            ],
            1,
        ),
        (
            "energy potion",
            [
                "4dose1energy",
                "3dose1energy",
                "2dose1energy",
                "1dose1energy",
            ],
            2,
        ),
        (
            "super energy",
            [
                "4dose2energy",
                "3dose2energy",
                "2dose2energy",
                "1dose2energy",
            ],
            3,
        ),
        (
            "super restore",
            [
                "4dose2restore",
                "3dose2restore",
                "2dose2restore",
                "1dose2restore",
            ],
            4,
        ),
    ] {
        let effects = match kind {
            2 => vec![Effect::Energy(10)],
            3 => vec![Effect::Energy(20)],
            _ => {
                let (constant, percent) = match kind {
                    0 => (7, 25),
                    1 => (10, 30),
                    _ => (8, 25),
                };
                let mut skills = Vec::new();
                if kind != 1 {
                    skills.push("prayer");
                }
                if kind != 0 {
                    skills.extend(RECOVERY_STATS);
                }
                skills
                    .into_iter()
                    .map(|skill| Effect::Recover {
                        skill,
                        constant,
                        percent,
                    })
                    .collect()
            }
        };
        chain(name, symbols, effects);
    }
    doses
}
pub fn sip(name: &str) -> String {
    format!("You drink some of your {name}.")
}
pub fn remaining(left: u8) -> String {
    if left == 0 {
        "You have finished your potion.".into()
    } else {
        format!(
            "You have {left} {} of potion left.",
            if left == 1 { "dose" } else { "doses" }
        )
    }
}
pub fn restored_level(current: i32, base: i32, constant: i32, percent: i32) -> i32 {
    if current >= base {
        return current;
    }
    let gain = constant.wrapping_add((base as f64 * (percent as f64 / 100.0)) as i32);
    base.min(current.wrapping_add(gain))
}
pub fn restored_energy(current: i32, percent: i32, max: i32) -> i32 {
    max.min(current.wrapping_add(percent.wrapping_mul(100)))
}
#[derive(Clone, Debug)]
pub enum ResolvedEffect {
    Boost {
        skill: u8,
        constant: i32,
        percent: i32,
    },
    Recover {
        skill: u8,
        constant: i32,
        percent: i32,
    },
    Energy(i32),
}
#[derive(Clone, Debug)]
pub struct Potion {
    pub id: i32,
    pub next: i32,
    pub vial: i32,
    pub sip: String,
    pub remaining: String,
    pub effects: Vec<ResolvedEffect>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Animation(u16),
    Stat(Update),
    Energy(i32),
    Message(String),
    Delay(i32),
}
fn replace(inv: &mut Inventory, items: &Catalog, id: i32, next: i32) {
    let mut copy = inv.clone();
    if copy
        .delete(items, Delete::new(id, 1))
        .and_then(|_| copy.insert(items, Insert::new(next, 1)))
        .is_ok()
    {
        *inv = copy;
    }
}
// Shared arithmetic/messages preserve the legacy reference function below.
fn apply_drink_effects(
    potion: &Potion, stats: &mut Stats, energy: &mut i32,
    emit: &mut impl FnMut(Event),
) -> Result<(), stat_ops::Error> {
    for effect in &potion.effects {
        match *effect {
            ResolvedEffect::Boost {
                skill,
                constant,
                percent,
            } => emit(Event::Stat(stats.apply(
                skill,
                Operation::Boost,
                constant,
                percent,
                false,
            )?)),
            ResolvedEffect::Recover {
                skill,
                constant,
                percent,
            } => {
                let current = stats.current(skill) as u8 as i32;
                let next =
                    restored_level(current, stats.base(skill) as u8 as i32, constant, percent);
                let gain = next.wrapping_sub(current);
                if gain > 0 {
                    emit(Event::Stat(stats.apply(
                        skill,
                        Operation::Add,
                        gain,
                        0,
                        false,
                    )?));
                }
            }
            ResolvedEffect::Energy(percent) => {
                let next = restored_energy(*energy, percent, 10_000);
                if next != *energy {
                    *energy = next;
                    emit(Event::Energy(next));
                }
            }
        }
    }
    emit(Event::Message(potion.sip.clone()));
    emit(Event::Message(potion.remaining.clone()));
    emit(Event::Delay(DRINK_DELAY));
    Ok(())
}

/// Gameplay adapter: replace the selected dose and publish inventory/effects
/// together. The legacy item-ID API remains available for reference comparison.
pub fn consume_slot(
    potion: &Potion, slot: usize, empty: bool,
    inv: &mut Inventory, stats: &mut Stats, energy: &mut i32,
    items: &Catalog, animation: u16, mut emit: impl FnMut(Event),
) -> Result<bool, stat_ops::Error> {
    if inv.get(slot).is_none_or(|i|i.id!=potion.id || i.count!=1) {
        return Ok(false);
    }
    let next=if empty {potion.vial}else{potion.next};
    let ordinary=|id| id>=0 && !items.stackable.contains(&id)
        && !items.dummy.contains(&id)
        && !items.notes.get(&id).is_some_and(|v|v.template!=0)
        && !items.placeholders.get(&id).is_some_and(|v|v.template!=0)
        && !items.transforms.get(&id).is_some_and(|v|v.template!=0);
    if next==potion.id || !ordinary(potion.id) || !ordinary(next) {
        return Ok(false);
    }
    let mut inventory=inv.clone();
    let mut debit=Delete::new(potion.id,1);debit.strict_slot=Some(slot);
    if inventory.delete(items,debit).is_err() {return Ok(false);}
    let mut credit=Insert::new(next,1);credit.strict_slot=Some(slot);
    if inventory.insert(items,credit).is_err() {return Ok(false);}
    if inventory.get(slot).is_none_or(|i|i.id!=next || i.count!=1 || i.vars!=0) {return Ok(false);}
    let mut next_stats=stats.clone();let mut next_energy=*energy;
    let mut events=Vec::new();
    if empty {events.push(Event::Message(EMPTY_MESSAGE.into()));}
    else {
        events.push(Event::Animation(animation));
        apply_drink_effects(potion,&mut next_stats,&mut next_energy,&mut |event|events.push(event))?;
    }
    *inv=inventory;*stats=next_stats;*energy=next_energy;
    for event in events {emit(event);}
    Ok(true)
}

pub fn consume(
    potion: &Potion,
    empty: bool,
    inv: &mut Inventory,
    stats: &mut Stats,
    energy: &mut i32,
    items: &Catalog,
    animation: u16,
    mut emit: impl FnMut(Event),
) -> Result<bool, stat_ops::Error> {
    if !inv
        .slots()
        .iter()
        .flatten()
        .any(|obj| obj.id == potion.id && obj.count > 0)
    {
        return Ok(false);
    }
    if empty {
        replace(inv, items, potion.id, potion.vial);
        emit(Event::Message(EMPTY_MESSAGE.into()));
        return Ok(true);
    }
    emit(Event::Animation(animation));
    replace(inv, items, potion.id, potion.next);
    apply_drink_effects(potion, stats, energy, &mut emit)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    fn potion(effects: Vec<ResolvedEffect>) -> Potion {
        Potion {
            id: 1,
            next: 2,
            vial: 3,
            sip: "sip".into(),
            remaining: "remaining".into(),
            effects,
        }
    }
    #[test]
    fn selected_dose_stays_in_place_with_duplicates_and_a_full_inventory() {
        for empty in [false,true] {
            let mut inv=Inventory::new(28,StackType::Normal,false);
            for slot in 0..28 {inv.restore_slot(slot,Some(Item{id:if slot==0||slot==27{1}else{99},count:1,vars:if slot==27{10}else{0}})).unwrap();}
            let before=inv.clone();let mut stats=Stats::default();stats.set_base(0,50);stats.set_current(0,45);let before_stats=stats.clone();let mut energy=6000;let mut events=Vec::new();
            let p=potion(vec![ResolvedEffect::Boost{skill:0,constant:3,percent:10},ResolvedEffect::Energy(20)]);
            assert!(consume_slot(&p,27,empty,&mut inv,&mut stats,&mut energy,&Catalog::default(),829,|e|events.push(e)).unwrap());
            assert_eq!(inv.get(27),Some(Item{id:if empty{3}else{2},count:1,vars:0}));
            for slot in 0..27 {assert_eq!(inv.get(slot),before.get(slot));}
            for skill in 0..25 {assert_eq!(stats.fine_xp(skill),before_stats.fine_xp(skill));}
            if empty {assert_eq!(stats,before_stats);assert_eq!(energy,6000);assert_eq!(events,vec![Event::Message(EMPTY_MESSAGE.into())]);}
            else {assert_eq!(stats.current(0),53);assert_eq!(energy,8000);assert_eq!(events.first(),Some(&Event::Animation(829)));assert_eq!(events.last(),Some(&Event::Delay(3)));}
        }
    }
    #[test]
    fn selected_dose_failures_publish_no_items_stats_energy_events_or_delay() {
        for mode in 0..5 {
            let mut inv=Inventory::new(3,if mode==4{StackType::Always}else{StackType::Normal},false);
            inv.restore_slot(0,Some(Item{id:1,count:1,vars:0})).unwrap();
            inv.restore_slot(2,Some(Item{id:1,count:if mode==0{2}else{1},vars:0})).unwrap();
            if mode==4 {inv.restore_slot(1,Some(Item{id:2,count:1,vars:0})).unwrap();}
            let mut catalog=Catalog::default();if mode==1{catalog.dummy.insert(2);}if mode==2{catalog.stackable.insert(1);}
            let effects=if mode==3{vec![ResolvedEffect::Energy(20),ResolvedEffect::Boost{skill:0,constant:-1,percent:10}]}else{vec![ResolvedEffect::Energy(20)]};
            let p=potion(effects);let mut stats=Stats::default();let mut energy=6000;let state=(inv.clone(),stats.clone(),energy);let mut events=Vec::new();
            let result=consume_slot(&p,2,false,&mut inv,&mut stats,&mut energy,&catalog,829,|e|events.push(e));
            if mode==3 {assert!(result.is_err());}else{assert!(!result.unwrap());}
            assert_eq!((inv,stats,energy),state);assert!(events.is_empty());
        }
    }
    #[test]
    fn failed_replacement_is_atomic_but_reference_still_applies_drink_effects() {
        let mut inv = Inventory::new(1, StackType::Always, false);
        inv.restore_slot(
            0,
            Some(Item {
                id: 1,
                count: 2,
                vars: 0,
            }),
        )
        .unwrap();
        let before = inv.clone();
        let mut stats = Stats::default();
        let mut energy = 9500;
        let mut events = Vec::new();
        let potion = potion(vec![ResolvedEffect::Energy(20)]);
        assert!(consume(
            &potion,
            false,
            &mut inv,
            &mut stats,
            &mut energy,
            &Catalog::default(),
            829,
            |e| events.push(e)
        )
        .unwrap());
        assert_eq!(inv, before);
        assert_eq!(energy, 10000);
        assert_eq!(
            events,
            vec![
                Event::Animation(829),
                Event::Energy(10000),
                Event::Message("sip".into()),
                Event::Message("remaining".into()),
                Event::Delay(3)
            ]
        );
    }
    #[test]
    fn recovery_skips_boosted_stats_and_full_energy_without_reducing_them() {
        let mut inv = Inventory::new(3, StackType::Normal, false);
        inv.restore_slot(
            2,
            Some(Item {
                id: 1,
                count: 1,
                vars: 123,
            }),
        )
        .unwrap();
        let mut stats = Stats::default();
        stats.set_base(0, 99);
        stats.set_current(0, 110);
        stats.set_base(5, 99);
        stats.set_current(5, 30);
        let potion = potion(vec![
            ResolvedEffect::Recover {
                skill: 0,
                constant: 8,
                percent: 25,
            },
            ResolvedEffect::Recover {
                skill: 5,
                constant: 8,
                percent: 25,
            },
            ResolvedEffect::Energy(20),
        ]);
        let mut energy = 10000;
        let mut events = Vec::new();
        consume(
            &potion,
            false,
            &mut inv,
            &mut stats,
            &mut energy,
            &Catalog::default(),
            829,
            |e| events.push(e),
        )
        .unwrap();
        assert_eq!(stats.current(0), 110);
        assert_eq!(stats.current(5), 62);
        assert_eq!(
            inv.get(0),
            Some(Item {
                id: 2,
                count: 1,
                vars: 0
            })
        );
        assert!(inv.get(2).is_none());
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::Stat(_)))
                .count(),
            1
        );
        assert!(!events.iter().any(|e| matches!(e, Event::Energy(_))));
        assert!(!consume(
            &potion,
            false,
            &mut inv,
            &mut stats,
            &mut energy,
            &Catalog::default(),
            829,
            |_| panic!("Missing potion emitted an event")
        )
        .unwrap());
    }
}
