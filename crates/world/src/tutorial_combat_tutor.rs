//! Existing CombatTutorScript dialogue and inventory semantics. Source SHA256:70bb7c740d9a276e9ef854cf92e0269012ada799c776eef0cd0aa26a870fad0e
use crate::{
    dialogue::{Conversation, Effect, Expression, Page, Speaker},
    inventory::{Catalog, Insert, Inventory},
    smithing_make::total,
    tutorial_steps::Step,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub step_varp: u16,
    pub dagger: i32,
    pub sword: i32,
    pub shield: i32,
    pub bow: i32,
    pub arrows: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    None,
    First,
    DaggerMissing,
    GiveGear,
    ReplaceGear,
    Styles,
    Melee,
    Range,
    RecapChoice,
    RecapFinish,
}
pub const OPTIONS: &[&str] = &[
    "Tell me about combat stats.",
    "Tell me about melee combat again.",
    "Tell me about ranged combat again.",
    "Tell me about the Wilderness.",
    "Nope, I'm ready to move on!",
];
pub const NO_ROOM: &str = "You need some new weapons for this section of the tutorial but you don't have enough inventory room. You'll need to drop something. To do so, right-click on an item you don't want and select drop. Once you've done that, speak to the combat instructor again.";
pub const SHIELD_ROOM: &str = "Vannaka gives you a bronze sword. He would have given you a wooden shield as well but you didn't have room for it. To get one, drop something and talk to the combat instructor again.";
pub const ARROW_ROOM: &str = "Vannaka gives you a shortbow. He would have given you some bronze arrows as well but you didn't have room for them. To get some, drop something and talk to the combat instructor again.";
const PAGES: &[Page] = &[
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Let's get started by teaching you to wield a weapon."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Time for some actual combat! Head into the pen through the nearby gate and kill one of those rats."},
    Page::NamedChat {speaker: Speaker::Player, expression: Expression::Happy, text: "Hi! My name's ${player.displayName}."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Quiz, text: "Do I look like I care? To me you're just another newcomer who thinks they're ready to fight."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "I am Vannaka, the greatest swordsman alive, and I'm here to teach you the basics of combat."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Let's get started by teaching you to wield a weapon."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Very good, but that little butter knife isn't going to protect you much. Let's get you something a bit better. Once you're properly equipped we can move on to some actual combat."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Let's get started by teaching you to wield a weapon."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Let's get you equipped."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "You should check out the combat interface."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Let's get you equipped."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Time for some actual combat! Head into the pen through the nearby gate and kill one of those rats."},
    Page::Chat {speaker: Speaker::Player, expression: Expression::Happy, text: "I did it! I killed a giant rat!"},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "I saw. You seem better at this than I thought. Now that you have grasped basic swordplay, let's move on."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Next up we have ranged combat. With this you can kill foes from a distance. With ranged, you'll be able to attack the rats without entering the pit meaning they won't be able to fight back."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Do you need something?"},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Certainly. You have three main combat stats: Strength, Defence and Attack."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Strength determines the maximum hit you will be able to deal with your blows, Defence determines the amount of damage you will be able to defend and Attack determines the accuracy of your blows."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Other stats are used in combat such as Prayer, Hitpoints, Magic and Ranged. All of these stats can go towards determining your combat level, which is shown near the top of your combat interface screen."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "You will find out on the mainland that certain items can also affect your stats. There are potions that can be drunk that can alter your stats temporarily, such as raising Strength."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "You will also raise your Defence and Attack values by using different weapons and armours."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Before going into combat with an opponent it is wise to put the mouse over them and see what combat level they are."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Green coloured writing usually means it will be an easy fight for you, red means you will probably lose, yellow means they are around your level and orange means they are slightly stronger."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Sometimes things will go your way, sometimes they won't. There is no such thing as a guaranteed win, but if the odds are on your side, you stand the best chance of walking away victorious."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Now, was there something else you wanted to hear about again?"},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Ah, my speciality. Close combat fighting, which is also known as melee fighting, covers all combat done at close range to your opponent."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Melee fighting depends entirely upon your three basic combat stats: Attack, Defence, and Strength."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Also, of course, it depends on the quality of your armour and weaponry. A high-levelled fighter with good armour and weapons will be deadly up close."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "If this is the path you wish to take, remember your success depends on getting as close to your enemy as quickly as possible."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Personally, I consider the fine art of melee combat to be the ONLY combat method worth using."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Now, did you want to hear anything else?"},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Thinking of being a ranger, eh? Well, okay. I don't enjoy it myself, but I can see the appeal."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Ranging employs a lot of different weapons as a skill, not just the shortbow you have there. Spears, throwing knives, and crossbows are all used best at a distance from your enemy."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Now, those rats were easy pickings, but on the mainland you will be very lucky if you can find a spot where you can shoot at your enemies without them being able to retaliate."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "At close range, rangers often do badly in combat. Your best tactic as a ranger is to hit and run, keeping your foe at a distance."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Your effectiveness as a ranger is almost entirely dependent on your Ranged stat. As with all skills, the more you train it, the more powerful it will become."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Anything else you wanted to know?"},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Ah yes, the Wilderness. It is a place of evil, mark my words. Many is the colleague I have lost in that foul place."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "It is also a place of both adventure and wealth, so if you are brave enough and strong enough to survive it, you will make a killing. Literally!"},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "It is also the only place in the land of Gielinor where players are able to attack each other at will, and as such is the haunt of many Player Killers, or PKers if you will."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "There are a few things different in the Wilderness in comparison to the rest of the lands of Gielinor. Firstly, as I just mentioned, you can and will be attacked by other players."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "For this reason, you will be given a warning when you approach the Wilderness, as it is not a place you would wish to enter by accident."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Secondly, there are a number of 'levels' to it. The further into it you travel, the greater the range of people you can attack."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "In level 1 wilderness you will only be able to attack, or be attacked by, those players within one combat level of yourself."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "In level 50, any player within fifty levels of you will be able to attack, or be attacked by you. Always keep an eye on what level of the Wilderness you are currently in."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Your current Wilderness level is shown at the bottom right of the screen. The final thing you should know about the Wilderness is being 'skulled'."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "If you attack another player without them having attacked first, you will gain a skull above your character's head."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "What this means is that if you die while skulled, you will lose EVERYTHING that your character was carrying."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "When skulled, you should try to avoid dying for the twenty minutes or so it will take for the skull to wear off."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "If you wish to find the Wilderness, head north from where you start on the mainland. It is rather large and hard to miss."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "If you don't wish to end up there, take notice of the warning you will receive when getting near to it."},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Now, was there anything more?"},
    Page::Chat {speaker: Speaker::Npc, expression: Expression::Happy, text: "Okay then."},
 ];
fn pages(config: Config, range: std::ops::Range<usize>, stage: Stage) -> Conversation {
    Conversation {
        pages: PAGES[range].iter().cloned().collect(),
        after: Effect::CombatTutor { config, stage },
    }
}
fn has(inv: &Inventory, worn: &Inventory, catalog: &Catalog, id: i32) -> bool {
    total(inv, catalog, id) > 0 || total(worn, catalog, id) > 0
}
pub fn begin(
    config: Config,
    step: Step,
    inv: &Inventory,
    worn: &Inventory,
    catalog: &Catalog,
) -> Conversation {
    if step < Step::CombatWorn {
        pages(config, 2..6, Stage::First)
    } else if step < Step::CombatDagger {
        pages(config, 0..1, Stage::None)
    } else if step < Step::CombatGear {
        if total(worn, catalog, config.dagger) > 0 {
            pages(config, 6..7, Stage::GiveGear)
        } else {
            pages(config, 7..8, Stage::DaggerMissing)
        }
    } else if step < Step::CombatStyles {
        if !has(inv, worn, catalog, config.sword) || !has(inv, worn, catalog, config.shield) {
            pages(config, 8..9, Stage::ReplaceGear)
        } else if total(worn, catalog, config.sword) > 0 && total(worn, catalog, config.shield) > 0
        {
            pages(config, 9..10, Stage::Styles)
        } else {
            pages(config, 10..11, Stage::ReplaceGear)
        }
    } else if step < Step::CombatMelee {
        pages(config, 11..12, Stage::Melee)
    } else if step < Step::CombatRange {
        pages(config, 1..2, Stage::None)
    } else if step < Step::CombatLadder {
        pages(config, 12..15, Stage::Range)
    } else {
        let mut c = pages(config, 15..16, Stage::RecapChoice);
        c.pages.push_back(Page::Choice(OPTIONS));
        c
    }
}
pub fn choice(config: Config, choice: i32) -> Option<Conversation> {
    Some(pages(
        config,
        match choice {
            1 => 16..25,
            2 => 25..31,
            3 => 31..37,
            4 => 37..52,
            5 => 52..53,
            _ => return None,
        },
        Stage::RecapFinish,
    ))
}
fn give_gear(
    config: Config,
    inv: &mut Inventory,
    worn: &Inventory,
    catalog: &Catalog,
) -> Vec<&'static str> {
    let sword = !has(inv, worn, catalog, config.sword);
    let shield = !has(inv, worn, catalog, config.shield);
    if !sword && !shield {
        return vec![];
    }
    if inv.free_space() == 0 {
        return vec![NO_ROOM];
    }
    if sword && inv.free_space() > 0 {
        let _ = inv.insert(catalog, Insert::new(config.sword, 1));
    }
    if shield && inv.free_space() > 0 {
        let _ = inv.insert(catalog, Insert::new(config.shield, 1));
    } else if shield {
        return vec![SHIELD_ROOM];
    }
    vec![]
}
fn give_range(
    config: Config,
    inv: &mut Inventory,
    worn: &Inventory,
    catalog: &Catalog,
) -> Vec<&'static str> {
    let bow = !has(inv, worn, catalog, config.bow);
    // Original counts backpack arrows only, even when arrows are already equipped.
    let arrows = total(inv, catalog, config.arrows);
    let need_arrows = arrows < 50;
    if !bow && !need_arrows {
        return vec![];
    }
    if inv.free_space() == 0 && bow {
        return vec![NO_ROOM];
    }
    if bow && inv.free_space() > 0 {
        let _ = inv.insert(catalog, Insert::new(config.bow, 1));
    }
    if need_arrows {
        let count = 50i32.wrapping_sub(arrows);
        if count > 0 && (inv.free_space() > 0 || arrows > 0) {
            let _ = inv.insert(catalog, Insert::new(config.arrows, count));
        } else if bow {
            return vec![ARROW_ROOM];
        }
    }
    vec![]
}
/// Final Continue rechecks supply needs, but preserves the branch chosen before suspension.
pub fn finish(
    config: Config,
    stage: Stage,
    current: Step,
    inv: &mut Inventory,
    worn: &Inventory,
    catalog: &Catalog,
) -> (Option<(Step, Option<i32>)>, Vec<&'static str>) {
    let mut messages = vec![];
    let next = match stage {
        Stage::None => None,
        Stage::First => Some((Step::CombatWorn, Some(4))),
        Stage::DaggerMissing => Some((Step::CombatDagger, Some(4))),
        Stage::GiveGear => {
            messages = give_gear(config, inv, worn, catalog);
            Some((Step::CombatGear, None))
        }
        Stage::ReplaceGear => {
            messages = give_gear(config, inv, worn, catalog);
            None
        }
        Stage::Styles => Some((Step::CombatStyles, Some(0))),
        Stage::Melee => Some((Step::CombatMelee, None)),
        Stage::Range => {
            messages = give_range(config, inv, worn, catalog);
            Some((Step::CombatRange, None))
        }
        Stage::RecapFinish => (current >= Step::CombatLadder).then_some((Step::CombatLadder, None)),
        Stage::RecapChoice => unreachable!("Choice remains suspended"),
    };
    (next, messages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    fn config() -> Config {
        Config {
            step_varp: 5003,
            dagger: 1205,
            sword: 1277,
            shield: 1171,
            bow: 841,
            arrows: 882,
        }
    }
    fn inv() -> Inventory {
        Inventory::new(4, StackType::Normal, false)
    }
    fn put(inv: &mut Inventory, slot: usize, id: i32, count: i32) {
        inv.restore_slot(slot, Some(Item { id, count, vars: 0 }))
            .unwrap();
    }
    fn catalog() -> Catalog {
        let mut c = Catalog::default();
        c.stackable.insert(882);
        c
    }
    #[test]
    fn sword_then_shield_commit_independently_and_failure_does_not_cancel_gear_step() {
        let c = config();
        let types = catalog();
        for free in 0..=2 {
            let mut bag = inv();
            let worn = inv();
            for i in 0..4 - free {
                put(&mut bag, i, 995, 1);
            }
            let (next, messages) = finish(
                c,
                Stage::GiveGear,
                Step::CombatDagger,
                &mut bag,
                &worn,
                &types,
            );
            assert_eq!(next, Some((Step::CombatGear, None)));
            assert_eq!(total(&bag, &types, c.sword), i32::from(free >= 1));
            assert_eq!(total(&bag, &types, c.shield), i32::from(free >= 2));
            assert_eq!(
                messages,
                match free {
                    0 => vec![NO_ROOM],
                    1 => vec![SHIELD_ROOM],
                    _ => vec![],
                }
            );
        }
    }
    #[test]
    fn worn_gear_is_not_duplicated_and_branch_survives_inventory_mutation() {
        let c = config();
        let types = catalog();
        let mut bag = inv();
        let mut worn = inv();
        put(&mut worn, 0, c.sword, 1);
        put(&mut worn, 1, c.shield, 1);
        let selected = begin(c, Step::CombatGear, &bag, &worn, &types);
        let Effect::CombatTutor { stage, .. } = selected.after else {
            panic!()
        };
        assert_eq!(stage, Stage::Styles);
        let (_, messages) = finish(
            c,
            Stage::ReplaceGear,
            Step::CombatGear,
            &mut bag,
            &worn,
            &types,
        );
        assert!(messages.is_empty());
        assert_eq!(bag.free_space(), 4);
        let (next, _) = finish(c, stage, Step::CombatGear, &mut bag, &inv(), &types);
        assert_eq!(next, Some((Step::CombatStyles, Some(0))));
    }
    #[test]
    fn range_refills_backpack_stack_when_full_but_does_not_count_equipped_arrows() {
        let c = config();
        let types = catalog();
        let mut bag = inv();
        let mut worn = inv();
        put(&mut worn, 0, c.bow, 1);
        put(&mut worn, 1, c.arrows, 100);
        put(&mut bag, 0, c.arrows, 12);
        for i in 1..4 {
            put(&mut bag, i, 995, 1);
        }
        let (next, messages) = finish(c, Stage::Range, Step::CombatRange, &mut bag, &worn, &types);
        assert_eq!(next, Some((Step::CombatRange, None)));
        assert!(messages.is_empty());
        assert_eq!(total(&bag, &types, c.arrows), 50);
        bag = inv();
        for i in 0..3 {
            put(&mut bag, i, 995, 1);
        }
        finish(c, Stage::Range, Step::CombatRange, &mut bag, &worn, &types);
        assert_eq!(total(&bag, &types, c.arrows), 50);
        assert_eq!(total(&worn, &types, c.arrows), 100);
    }
    #[test]
    fn missing_bow_space_guard_precedes_arrow_refill_and_one_slot_only_grants_bow() {
        let c = config();
        let types = catalog();
        let mut bag = inv();
        let worn = inv();
        put(&mut bag, 0, c.arrows, 12);
        for i in 1..4 {
            put(&mut bag, i, 995, 1);
        }
        assert_eq!(
            finish(c, Stage::Range, Step::CombatRange, &mut bag, &worn, &types).1,
            vec![NO_ROOM]
        );
        assert_eq!(total(&bag, &types, c.arrows), 12);
        bag = inv();
        for i in 0..3 {
            put(&mut bag, i, 995, 1);
        }
        assert_eq!(
            finish(c, Stage::Range, Step::CombatRange, &mut bag, &worn, &types).1,
            vec![ARROW_ROOM]
        );
        assert_eq!(total(&bag, &types, c.bow), 1);
        assert_eq!(total(&bag, &types, c.arrows), 0);
    }
    #[test]
    fn recap_has_five_choices_and_uses_current_progress_at_final_continue() {
        let c = config();
        let types = catalog();
        for (selection, count) in [(1, 9), (2, 6), (3, 6), (4, 15), (5, 1)] {
            assert_eq!(choice(c, selection).unwrap().pages.len(), count);
        }
        assert!(choice(c, 0).is_none());
        assert!(choice(c, 6).is_none());
        for current in [
            Step::CharCreate,
            Step::CombatRange,
            Step::CombatLadder,
            Step::LeaveTalk,
        ] {
            let (next, messages) =
                finish(c, Stage::RecapFinish, current, &mut inv(), &inv(), &types);
            assert_eq!(
                next,
                (current >= Step::CombatLadder).then_some((Step::CombatLadder, None))
            );
            assert!(messages.is_empty());
        }
    }
}
