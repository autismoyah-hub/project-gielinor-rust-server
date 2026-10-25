//! Captured NPC identity and registered operation behavior for player interactions.
use crate::simulation::NpcId;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handler {
    SkillcapeVendor(crate::skillcape_vendor::Config),
    Bank,
    Banknote {slot:usize,note:i32,raw:u16,visible:u16},
    Shear {raw:u16,selected:Option<usize>},
    HeldItemReply {raw:u16,slot:usize,item:i32},
    Pickpocket,
    Tanning,
    TowerTanner,
    /// Floor One banker Talk-to; Bank stays on [`Handler::Bank`].
    TowerBanker,
    Combat,
    Person,
    TutorialFishing,
    CinderworksFishing,
    /// Ordinary spot; the (type, operation) binding lives in `World::fishing`.
    Fishing,
    MagicTutor(crate::tutorial_magic_tutor::Config),
    LectureTutor(crate::tutorial_lecture_tutor::Config),
    CombatTutor(crate::tutorial_combat_tutor::Config),
    MiningTutor(crate::tutorial_mining_tutor::Config),
    TutorialGuide { step_varp: u16 },
    SurvivalTutor(crate::tutorial_survival::Config),
    MasterChef(crate::tutorial_chef::Config),
    QuestTutor { step_varp: u16 },
}
#[derive(Clone, Copy, Debug)]
pub struct Pending {
    pub target: NpcId,
    pub operation: u8,
    pub handler: Option<Handler>,
    /// Captured manual spell; successful autocast continuation clears this.
    pub spell: Option<i32>,
}
#[derive(Clone, Copy, Debug)]
pub enum Event {
    Face(NpcId),
    Tanning(NpcId),
    Bank,
    Message(&'static str),
}
