use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug)]
pub struct Definition {
    pub animation: u16,
    pub animation_ticks: i32,
    pub sound: Option<u16>,
}
#[derive(Clone, Debug)]
pub struct Config {
    pub definitions: BTreeMap<u16, Definition>,
    pub bones: u16,
    pub chickens: BTreeMap<u16, Chicken>,
}
/// Selected basic Wiki chicken table. Tertiary clues and conditional keys are
/// separate content and are not implied by this profile.
#[derive(Clone, Copy, Debug)]
pub struct Chicken {
    pub raw: u16,
    pub feather: u16,
}
impl Chicken {
    /// Fixture ordering for the Wiki weights, not a claim about OSRS seed order.
    pub fn feather_count(draw: u32) -> Option<i32> {
        match draw {
            0..=63 => Some(5),
            64..=95 => Some(15),
            96..=127 => Some(0),
            _ => None,
        }
    }
}
/// Additional guaranteed products use the existing private, atomic death batch.
#[derive(Clone, Debug)]
pub struct GuaranteedExtras { pub products: Vec<(u16, i32)>, pub policy: DropPolicy }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropPolicy { Chicken, Cow, Imp, Bear, Spider, Goblin, Man, Woman }
impl DropPolicy { pub fn name(self) -> &'static str { match self { Self::Chicken => "selected_chicken", Self::Cow => "selected_cow", Self::Imp => "selected_imp", Self::Bear => "selected_bear", Self::Spider => "selected_spider", Self::Goblin => "selected_goblin", Self::Man => "selected_man", Self::Woman => "selected_woman" } } }
/// Authoritative selected-profile drop boundary, independent of hit/XP acceptance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropResult {
    Committed,
    Rejected,
    NoEligibleOwner,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DropTrace {
    pub policy: DropPolicy,
    pub tick: i32,
    pub npc: crate::simulation::NpcId,
    pub npc_type: u16,
    pub coords: crate::CoordGrid,
    pub owner: Option<crate::simulation::ActorId>,
    pub draw: Option<u32>,
    pub products: Vec<(u16, i32)>,
    pub handles: Vec<Vec<u64>>,
    pub result: DropResult,
}
impl DropTrace {
    pub fn json(&self) -> String {
        let optional = |value: Option<u64>| {
            value
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".into())
        };
        let products = self
            .products
            .iter()
            .map(|(item, count)| format!("[{item},{count}]"))
            .collect::<Vec<_>>()
            .join(",");
        let result = match self.result {
            DropResult::Committed => "committed",
            DropResult::Rejected => "rejected",
            DropResult::NoEligibleOwner => "no_eligible_owner",
        };
        format!(concat!("{{\"schema\":1,\"tick\":{},\"npc_slot\":{},\"npc_generation\":{},\"npc_type\":{},",
            "\"coords\":[{},{},{}],\"actor_slot\":{},\"actor_generation\":{},",
            "\"policy\":\"{}\",\"result\":\"{}\",\"draw\":{},\"products\":[{}],\"handles\":{:?}}}"),
            self.tick,self.npc.slot,self.npc.generation,self.npc_type,
            self.coords.x(),self.coords.z(),self.coords.level(),
            optional(self.owner.map(|a| a.slot as u64)),optional(self.owner.map(|a| a.generation as u64)),
            self.policy.name(),result,optional(self.draw.map(u64::from)),products,self.handles)
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum State {
    Arrival {
        at: i32,
        drop_coords: crate::CoordGrid,
    },
    Animation {
        until: i32,
        drop_coords: crate::CoordGrid,
    },
    Respawn {
        at: i32,
    },
}
