//! NPC hunt modes (`.hunt` config, config archive group 2) and the player-hunt
//! checks of the reference NpcPlayerHuntProcessor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Off,
    Player,
    Npc,
    Obj,
    Scenery,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vis {
    Off,
    LineOfSight,
    LineOfWalk,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    GreaterThan,
    LessThan,
    Equals,
    NotEquals,
}
impl Operator {
    pub fn from_id(id: u8) -> Option<Self> {
        Some(match id {
            0 => Self::GreaterThan,
            1 => Self::LessThan,
            2 => Self::Equals,
            3 => Self::NotEquals,
            _ => return None,
        })
    }
    pub fn evaluate(self, actual: i32, required: i32) -> bool {
        match self {
            Self::GreaterThan => actual > required,
            Self::LessThan => actual < required,
            Self::Equals => actual == required,
            Self::NotEquals => actual != required,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VarCondition {
    pub varp: u16,
    pub operator: Operator,
    pub required: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvCondition {
    pub inv: u16,
    /// Object id (`check_inv_obj`) or param id (`check_inv_param`).
    pub key: u16,
    pub operator: Operator,
    pub required: i32,
}
/// NpcMode ids used by `find_new_mode`.
pub const OP_PLAYER2: u8 = 8;
pub const AP_PLAYER2: u8 = 16;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mode {
    pub kind: Kind,
    pub vis: Vis,
    /// `check_nottoostrong=outside_wilderness`.
    pub not_too_strong: bool,
    pub not_combat: Option<u16>,
    pub not_combat_self: Option<u16>,
    pub check_afk: bool,
    pub not_busy: bool,
    pub keep_hunting: bool,
    pub find_new_mode: u8,
    pub pause_when_nobody_near: bool,
    pub rate: i32,
    pub inv_obj: Option<InvCondition>,
    pub inv_param: Option<InvCondition>,
    pub vars: [Option<VarCondition>; 3],
    /// NPC/obj/loc target filters; only consulted by non-player hunts.
    pub has_target_filter: bool,
}
impl Default for Mode {
    fn default() -> Self {
        Self {
            kind: Kind::Off,
            vis: Vis::Off,
            not_too_strong: false,
            not_combat: None,
            not_combat_self: None,
            check_afk: true,
            not_busy: false,
            keep_hunting: false,
            find_new_mode: 0,
            pause_when_nobody_near: false,
            rate: 1,
            inv_obj: None,
            inv_param: None,
            vars: [None; 3],
            has_target_filter: false,
        }
    }
}
impl Mode {
    /// Player hunts whose finding attacks the player (op/ap player 2).
    pub fn attacks_players(&self) -> bool {
        self.kind == Kind::Player && matches!(self.find_new_mode, OP_PLAYER2 | AP_PLAYER2)
    }
    /// Attack modes this port evaluates fully. Inventory conditions are not ported,
    /// so modes using them never hunt instead of hunting with the check skipped.
    pub fn player_hunt_supported(&self) -> bool {
        self.attacks_players() && self.inv_obj.is_none() && self.inv_param.is_none()
    }
}
/// Players more than twice the NPC's combat level are ignored outside the Wilderness.
pub fn too_strong(player_level: i32, npc_level: i32) -> bool {
    player_level > npc_level.saturating_mul(2)
}
/// GameRandom.of(0, count) == 0 after counting the candidate: a reservoir draw that
/// replaces the current choice with probability 1 / (count + 1).
pub fn replaces(count: u32, random: &mut impl crate::npc::RandomSource) -> bool {
    random.below(count + 1) == 0
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn combat_level_threshold_is_strictly_above_double() {
        assert!(!too_strong(4, 2));
        assert!(too_strong(5, 2));
        assert!(!too_strong(56, 28));
        assert!(too_strong(57, 28));
        assert!(!too_strong(126, 63));
    }
    #[test]
    fn only_player_attack_modes_attack() {
        let mut mode = Mode {
            kind: Kind::Player,
            find_new_mode: OP_PLAYER2,
            ..Mode::default()
        };
        assert!(mode.attacks_players());
        mode.find_new_mode = AP_PLAYER2;
        assert!(mode.attacks_players());
        mode.find_new_mode = 7;
        assert!(!mode.attacks_players());
        mode.kind = Kind::Npc;
        mode.find_new_mode = OP_PLAYER2;
        assert!(!mode.attacks_players());
    }
}
