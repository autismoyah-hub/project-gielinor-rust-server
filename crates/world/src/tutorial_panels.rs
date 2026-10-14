//! TutorialSidePanelScript decisions; input must already be an admitted overlay button.
use crate::tutorial_steps::Step;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Panel {
    Settings,
    Combat,
    Stats,
    Quests,
    Inventory,
    Worn,
    Prayer,
    Magic,
    Account,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    None,
    PointBackToGuide,
    Set(Step),
}
pub fn opened(panel: Panel, current: Step) -> Action {
    use Panel::*;
    use Step::*;
    match (panel, current) {
        (Settings, GielinorSettings) => Action::PointBackToGuide,
        (Inventory, SurvivalInv) => Action::Set(SurvivalFish),
        (Stats, SurvivalSkills) => Action::Set(SurvivalTools),
        (Quests, QuestTalk | QuestList) => Action::Set(QuestList),
        (Worn, CombatWorn) => Action::Set(CombatStats),
        (Worn, CombatStats) => Action::Set(CombatDagger),
        (Combat, CombatStyles) => Action::Set(CombatMelee),
        (Account, AccountMgmt) => Action::Set(AccountMgmt),
        (Prayer, PrayerTab) => Action::Set(PrayerTab),
        (Magic, MagicTab) => Action::Set(MagicTab),
        _ => Action::None,
    }
}
