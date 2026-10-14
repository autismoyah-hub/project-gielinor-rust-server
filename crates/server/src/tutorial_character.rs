//! TutorialIslandBootstrap and TutorialCharacterScript's resolved identities.
use crate::{startup::Names, tutorial_ui};
use gielinor_cache::game::Definitions;
use gielinor_world::{tutorial_progression, ui::Component};
use std::io;

pub struct Config {
    pub panels: std::collections::BTreeMap<u32, gielinor_world::tutorial_panels::Panel>,
    pub progression: tutorial_progression::Config,
    pub focus: tutorial_ui::Config,
    pub mainmodal: Component,
    pub creator: u16,
    pub confirm: u32,
    pub pronouns: Component,
    pub experience: u16,
    pub experience_buttons: [Component; 3],
    pub body_bit: u32,
    pub pronoun_bit: u32,
    pub adventure_bit: u32,
}
impl Config {
    pub fn resolve(names: &Names, defs: &Definitions) -> io::Result<Self> {
        let varp = |name| {
            defs.varps
                .get(&names.id("varp", name)?)
                .copied()
                .ok_or_else(|| io::Error::other("Missing tutorial variable"))
        };
        let bit = |name| {
            let id = names.id("varbit", name)?;
            let ty = defs
                .varbits
                .get(&id)
                .ok_or_else(|| io::Error::other("Missing tutorial bit"))?;
            if !defs.varps.contains_key(&(ty.base as u32)) {
                return Err(io::Error::other("Missing tutorial bit base"));
            }
            Ok(id)
        };
        let component = |name| names.id("component", name).map(Component);
        use gielinor_world::tutorial_panels::Panel;
        let mut panels = std::collections::BTreeMap::new();
        for name in ["settings_side:settings_tab", "settings_side:master_icon"] {
            panels.insert(names.id("component", name)?, Panel::Settings);
        }
        for (stone, panel) in [
            (0, Panel::Combat),
            (1, Panel::Stats),
            (2, Panel::Quests),
            (3, Panel::Inventory),
            (4, Panel::Worn),
            (5, Panel::Prayer),
            (6, Panel::Magic),
            (8, Panel::Account),
        ] {
            for frame in ["toplevel_osrs_stretch", "toplevel", "toplevel_pre_eoc"] {
                panels.insert(
                    names.id("component", &format!("{frame}:stone{stone}"))?,
                    panel,
                );
            }
        }
        Ok(Self {
            panels,
            progression: tutorial_progression::Config {
                step: varp("tutorial_2")?,
                completed: varp("tut2_completed")?,
            },
            focus: tutorial_ui::Config::resolve(names)?,
            mainmodal: component("toplevel_osrs_stretch:mainmodal")?,
            creator: names.short("interface", "player_design")?,
            confirm: names.id("component", "player_design:confirm")?,
            pronouns: component("player_design:pronouns_buttons")?,
            experience: names.short("interface", "tutorial_player_experience")?,
            experience_buttons: [
                component("tutorial_player_experience:button_new")?,
                component("tutorial_player_experience:button_returning")?,
                component("tutorial_player_experience:button_experienced")?,
            ],
            body_bit: bit("player_design_bodytype")?,
            pronoun_bit: bit("settings_transmit_pronouns")?,
            adventure_bit: bit("adventurepath_player_participating")?,
        })
    }
}
