//! TutorialFocusGuidance's step selection. Resolving NPC instances and applying
//! interface effects is deliberately owned by the runtime adapter.
use crate::{tutorial_steps::Step, CoordGrid};

pub fn modal_step(step: Step) -> bool {
    matches!(step, Step::CharCreate | Step::ExpSelect)
}
pub fn hint_tile(step: Step) -> Option<CoordGrid> {
    use Step::*;
    let (square_z, x, z) = match step {
        GielinorDoor => (95, 26, 35),
        SurvivalGate => (95, 17, 20),
        ChefDoor => (95, 6, 12),
        ChefExit => (95, 0, 18),
        QuestDoor => (95, 14, 54),
        QuestLadder => (95, 16, 47),
        MineGate => (195, 22, 31),
        CombatLadder => (195, 39, 54),
        BankOpen => (95, 48, 52),
        PollView => (95, 47, 49),
        AccountExit => (95, 58, 52),
        PrayerExit => (95, 57, 34),
        _ => return None,
    };
    Some(CoordGrid::new(26 * 64 + x, square_z * 64 + z, 0).unwrap())
}
pub fn hint_npc(step: Step) -> Option<&'static str> {
    use Step::*;
    Some(match step {
        GielinorTalk => "tut2_basics_tutor",
        SurvivalTalk | SurvivalTools => "tut2_survival_tutor",
        SurvivalFish => "0_26_95_tut2_fishing",
        ChefTalk => "tut2_nav_tutor",
        QuestTalk => "tut2_quest_tutor",
        MineTalk | MineHammer => "tut2_mining_tutor",
        CombatTalk | CombatGear | CombatRange => "tut2_combat_tutor",
        CombatMelee => "tut2_giantrat",
        AccountTalk => "tut2_bank_tutor",
        PrayerTalk => "tut2_prayer_tutor",
        Ironman => "tut2_ironman_tutor",
        MagicTalk | LeaveTalk => "tut2_magic_tutor",
        MagicRunes | MagicCast => "tut2_chicken",
        _ => return None,
    })
}
pub fn outline_npc(step: Step) -> Option<&'static str> {
    match step {
        Step::QuestList => Some("tut2_quest_tutor"),
        Step::AccountMgmt => Some("tut2_bank_tutor"),
        Step::PrayerTab => Some("tut2_prayer_tutor"),
        Step::MagicTab => Some("tut2_magic_tutor"),
        _ => hint_npc(step),
    }
}
pub fn outline_location(step: Step) -> Option<&'static str> {
    use Step::*;
    Some(match step {
        GielinorDoor => "tut2_survival_entry",
        SurvivalWc => "tut2_tree",
        SurvivalGate => "tut2_survival_exit_l",
        ChefDoor => "tut2_nav_entry",
        ChefBread => "tut2_range",
        ChefExit => "tut2_nav_exit",
        QuestDoor => "tut2_quest_entry",
        QuestLadder => "tut2_quest_ladder",
        MineOres => "tut2_tinrock",
        MineSmelt => "tut2_furnace",
        MineDagger => "anvil",
        MineGate => "tut2_mining_exit_l",
        CombatLadder => "tut2_combat_ladder",
        BankOpen => "tut2_bankbooth",
        PollView => "pollbooth_green",
        AccountExit => "tut2_bank_exit",
        PrayerExit => "tut2_prayer_exit_l",
        _ => return None,
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flash {
    None,
    Side(u8),
    Equipment,
    WindStrike,
    HomeTeleport,
}
pub fn flash(step: Step) -> Flash {
    use Step::*;
    match step {
        GielinorSettings => Flash::Side(12),
        SurvivalInv => Flash::Side(4),
        SurvivalSkills => Flash::Side(2),
        QuestList => Flash::Side(3),
        CombatWorn => Flash::Side(5),
        CombatStyles => Flash::Side(1),
        AccountMgmt => Flash::Side(9),
        PrayerTab => Flash::Side(6),
        MagicTab => Flash::Side(7),
        CombatStats => Flash::Equipment,
        MagicRunes | MagicCast => Flash::WindStrike,
        HomeTele => Flash::HomeTeleport,
        _ => Flash::None,
    }
}
pub struct SidePanel {
    pub interface: &'static str,
    pub side: u8,
    pub unlock: Step,
}
pub const SIDE_PANELS: [SidePanel; 14] = [
    SidePanel {
        interface: "settings_side",
        side: 11,
        unlock: Step::GielinorSettings,
    },
    SidePanel {
        interface: "inventory",
        side: 3,
        unlock: Step::SurvivalInv,
    },
    SidePanel {
        interface: "stats",
        side: 1,
        unlock: Step::SurvivalSkills,
    },
    SidePanel {
        interface: "side_journal",
        side: 2,
        unlock: Step::QuestList,
    },
    SidePanel {
        interface: "wornitems",
        side: 4,
        unlock: Step::CombatWorn,
    },
    SidePanel {
        interface: "combat_interface",
        side: 0,
        unlock: Step::CombatStyles,
    },
    SidePanel {
        interface: "account",
        side: 8,
        unlock: Step::AccountMgmt,
    },
    SidePanel {
        interface: "prayerbook",
        side: 5,
        unlock: Step::PrayerTab,
    },
    SidePanel {
        interface: "magic_spellbook",
        side: 6,
        unlock: Step::MagicTab,
    },
    SidePanel {
        interface: "logout",
        side: 10,
        unlock: Step::Completed,
    },
    SidePanel {
        interface: "friends",
        side: 9,
        unlock: Step::Completed,
    },
    SidePanel {
        interface: "side_channels",
        side: 7,
        unlock: Step::Completed,
    },
    SidePanel {
        interface: "emote",
        side: 12,
        unlock: Step::Completed,
    },
    SidePanel {
        interface: "music",
        side: 13,
        unlock: Step::Completed,
    },
];
