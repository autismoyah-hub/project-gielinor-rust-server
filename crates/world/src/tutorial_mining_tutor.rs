//! MiningTutorScript dialogue branches and inventory timing, from the existing Kotlin content.
//! Reference SHA256: 17b96ead674a07b52af2c61792aa8abbeb2545245854201493e1383dbdff3bb6
use crate::{
    dialogue::{Conversation, Effect, Expression, Page, Speaker},
    inventory::{Catalog, Insert, Inventory},
    smithing_make,
    tutorial_steps::Step,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub step_varp: u16,
    pub pickaxe: i32,
    pub hammer: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    None,
    First,
    Smelt,
    Hammer,
    RecapChoice,
    RecapFinish,
}
pub const OPTIONS: &[&str] = &[
    "Tell me about Mining again.",
    "Tell me about Smelting again.",
    "Tell me about Smithing again.",
    "Nope, I'm ready to move on!",
];
pub const NO_ROOM: &str = "You need some tools for this section of the tutorial but you don't have enough inventory room. You'll need to drop something. To do so, right-click on an item you don't want and select drop. Once you've done that, speak to the mining instructor again.";
const PAGES: &[Page] = &[
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "The rocks around here contain tin and copper. You should try mining some." },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "See if you can use the anvil here to make a bronze dagger." },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "Hi there. You must be new around here. So what do I call you? 'Newcomer' seems so impersonal, and if we're going to be working together, I'd rather call you by name." },
    Page::NamedChat { speaker: Speaker::Player, expression: Expression::Happy, text: "You can call me ${player.displayName}." },
    Page::NamedChat { speaker: Speaker::Npc, expression: Expression::Happy, text: "Ok then, ${player.displayName}. My name is Dezzick and I'm a miner by trade. Let's teach you how to mine." },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "Mining is very simple, all you need is a pickaxe. The rocks around here contain tin and copper. Why don't you get started by mining some?" },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "Now that you have some ore, you should smelt it into a bronze bar. You can use the furnace over there to do so." },
    Page::Chat { speaker: Speaker::Player, expression: Expression::Quiz, text: "I have a bronze bar. What now?" },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "Now that you've got a bar, you can smith it into a weapon. To smith something, you need a hammer and an anvil. There's some anvils just here that you can use. See if you can make a bronze dagger." },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "Would you like me to recap anything?" },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "Certainly. To mine you need a pickaxe. Different pickaxes let you mine more efficiently." },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "Earlier I gave you a bronze pickaxe. It's the most inefficient pickaxe available, but it's perfect for a beginner." },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "To mine, simply click on a rock that contains ore while you have a pickaxe with you, and you will keep mining the rock until you manage to get some ore, or until it is empty." },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "The better the pickaxe you use, the faster you will get ore from the rock you're mining." },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "You will be able to buy better pickaxes from the Dwarven Mine when you reach the mainland, but they can be expensive." },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "Also, the better the pickaxe the higher the Mining level required to use it will be. Was there anything else you wanted to hear?" },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "Smelting is very easy. Simply take the ores required to make a metal to a furnace, then use the ores on the furnace to smelt them into a bar of metal." },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "Furnaces are expensive to build and maintain, so there are not that many scattered around the world. I suggest when you find one you remember its location for future use." },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "An alternative to using a furnace to smelt your ore is to use high-level magic to do it." },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "As well as letting you smelt ore anywhere, it has a guaranteed success rate in smelting all ores." },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "Some metals, such as iron, contain impurities and can be destroyed during the smelting process in a traditional furnace, but magical heat does not destroy them." },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "Anything else?" },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "When you have acquired enough bars of the metal you wish to work with, you are ready to begin smithing." },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "Take the hammer I gave you, or buy a new one from a general store, and proceed to a nearby anvil." },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "By using a metal bar on an anvil you will be presented with a screen showing the objects you are able to smith at your current level." },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "It's a pretty straightforward skill as I'm sure you discovered while making me that lovely bronze dagger." },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "The higher Smithing level you are, the better quality the metal you can work with. You start off on bronze and work your way up as your smithing skills increase." },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "Anything else?" },
    Page::Chat { speaker: Speaker::Npc, expression: Expression::Happy, text: "Okay then." },
];
fn pages(config: Config, range: std::ops::Range<usize>, stage: Stage) -> Conversation {
    Conversation {
        pages: PAGES[range].iter().cloned().collect(),
        after: Effect::MiningTutor { config, stage },
    }
}
pub fn begin(config: Config, step: Step) -> Conversation {
    if step < Step::MineOres {
        pages(config, 2..6, Stage::First)
    } else if step < Step::MineSmelt {
        pages(config, 0..1, Stage::None)
    } else if step < Step::MineHammer {
        pages(config, 6..7, Stage::Smelt)
    } else if step < Step::MineDagger {
        pages(config, 7..9, Stage::Hammer)
    } else if step < Step::MineGate {
        pages(config, 1..2, Stage::None)
    } else {
        let mut c = pages(config, 9..10, Stage::RecapChoice);
        c.pages.push_back(Page::Choice(OPTIONS));
        c
    }
}
pub fn choice(config: Config, choice: i32) -> Option<Conversation> {
    Some(pages(
        config,
        match choice {
            1 => 10..16,
            2 => 16..22,
            3 => 22..28,
            4 => 28..29,
            _ => return None,
        },
        Stage::RecapFinish,
    ))
}
pub fn prepare_recap(config: Config, inv: &mut Inventory, catalog: &Catalog) {
    if smithing_make::total(inv, catalog, config.hammer) <= 0
        && inv.slots().iter().any(Option::is_none)
    {
        let _ = inv.insert(catalog, Insert::new(config.hammer, 1));
    }
}
/// Applied only after the last valid Continue, using then-current inventory/progress.
pub fn finish(
    config: Config,
    stage: Stage,
    current: Step,
    inv: &mut Inventory,
    catalog: &Catalog,
) -> (Option<Step>, Option<&'static str>) {
    let step = match stage {
        Stage::First => {
            if smithing_make::total(inv, catalog, config.pickaxe) <= 0 {
                let _ = inv.insert(catalog, Insert::new(config.pickaxe, 1));
            }
            Some(Step::MineOres)
        }
        Stage::Smelt => Some(Step::MineSmelt),
        Stage::Hammer => {
            if smithing_make::total(inv, catalog, config.hammer) <= 0 {
                if inv.slots().iter().all(Option::is_some) {
                    return (None, Some(NO_ROOM));
                }
                let _ = inv.insert(catalog, Insert::new(config.hammer, 1));
            }
            Some(Step::MineDagger)
        }
        Stage::RecapFinish => (current >= Step::MineGate).then_some(Step::MineGate),
        Stage::None => None,
        Stage::RecapChoice => unreachable!("Choice remains suspended"),
    };
    (step, None)
}
