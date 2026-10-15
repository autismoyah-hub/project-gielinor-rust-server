//! Hit queues and damage resolution. Attack selection, modifiers and death scripts are separate.
use crate::simulation::{ActorId, NpcId};
/// Resolved script queue IDs. Player hits use Strong and death uses Normal.
#[derive(Debug, Clone, Copy)]
pub struct PlayerQueues {
    pub standard: i32,
    pub death: i32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    Player(ActorId),
    Npc(NpcId),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Melee,
    Ranged,
    Magic,
    Typeless,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hit {
    pub display: Option<crate::hit_display::Mark>,
    pub source: Option<Target>,
    pub style: Style,
    pub damage: i32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidDamage,
    InvalidDisplay,
    QueueFull,
    InvalidTarget,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Impact {
    pub hit: Hit,
    pub applied: i32,
    pub displayed: i32,
    pub health: i32,
    pub death: bool,
}
/// Selected NPC's authoritative impact; a zero impact alone is not a hit-roll verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcImpactTrace {
    pub tick: i32,
    pub npc: NpcId,
    pub npc_type: u16,
    pub hp_before: i32,
    pub impact: Impact,
}
impl NpcImpactTrace {
    pub fn json(self) -> String {
        let source = match self.impact.hit.source {
            Some(Target::Player(id)) => format!(
                "{{\"kind\":\"player\",\"slot\":{},\"generation\":{}}}",
                id.slot, id.generation
            ),
            Some(Target::Npc(id)) => format!(
                "{{\"kind\":\"npc\",\"slot\":{},\"generation\":{}}}",
                id.slot, id.generation
            ),
            None => "null".into(),
        };
        let style = match self.impact.hit.style {
            Style::Melee => "melee",
            Style::Ranged => "ranged",
            Style::Magic => "magic",
            Style::Typeless => "typeless",
        };
        format!(
            concat!(
                "{{\"schema\":1,\"tick\":{},\"npc_slot\":{},\"npc_generation\":{},\"npc_type\":{},",
                "\"source\":{},\"style\":\"{}\",\"hp_before\":{},\"hp_after\":{},",
                "\"queued_damage\":{},\"applied_damage\":{},\"displayed_damage\":{},\"death\":{}}}"
            ),
            self.tick,
            self.npc.slot,
            self.npc.generation,
            self.npc_type,
            source,
            style,
            self.hp_before,
            self.impact.health,
            self.impact.hit.damage,
            self.impact.applied,
            self.impact.displayed,
            self.impact.death
        )
    }
}
impl Hit {
    pub fn validate(self) -> Result<(), Error> {
        if self.display.is_some_and(|m| {
            m.target > 255 || m.source > 255 || m.public.is_some_and(|id| id > 255) || m.delay > 255
        }) {
            return Err(Error::InvalidDisplay);
        }
        if (0..=32767).contains(&self.damage) {
            Ok(())
        } else {
            Err(Error::InvalidDamage)
        }
    }
    /// Player hitmarks retain original damage; NPC hitmarks clamp to remaining health.
    pub fn impact(self, health: i32, npc: bool) -> Impact {
        let applied = health.min(self.damage).max(0);
        let health = health - applied;
        Impact {
            hit: self,
            applied,
            displayed: if npc { applied } else { self.damage },
            health,
            death: health == 0,
        }
    }
    pub fn valid_player_source(self, npc_alive: impl FnOnce(NpcId) -> bool) -> bool {
        match (self.style, self.source) {
            (Style::Melee, Some(Target::Npc(id))) => npc_alive(id),
            _ => true,
        }
    }
}
#[derive(Debug, Clone)]
struct Pending {
    hit: Hit,
    remaining: i32,
    processed: i32,
}
#[derive(Debug, Clone, Default)]
pub struct Queue {
    entries: Vec<Pending>,
}
impl Queue {
    pub fn push(&mut self, hit: Hit, delay: i32) -> Result<(), Error> {
        hit.validate()?;
        if self.entries.len() >= 1024 {
            return Err(Error::QueueFull);
        }
        self.entries.push(Pending {
            hit,
            remaining: delay,
            processed: -1,
        });
        Ok(())
    }
    /// Called in entity-main; a protected entity retains expired hits. A cycle decrements once.
    pub fn process(&mut self, clock: i32, protected: bool) -> Vec<Hit> {
        let mut ready = vec![];
        self.entries.retain_mut(|pending| {
            if pending.processed != clock {
                pending.processed = clock;
                pending.remaining = pending.remaining.wrapping_sub(1);
            }
            if pending.remaining <= 0 && !protected {
                ready.push(pending.hit);
                false
            } else {
                true
            }
        });
        ready
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn impact_preserves_explicit_zero_and_clamps_positive_overkill() {
        for npc in [false, true] {
            let zero = Hit {
                display: None,
                source: None,
                style: Style::Melee,
                damage: 0,
            }
            .impact(3, npc);
            assert_eq!((zero.applied, zero.health, zero.death), (0, 3, false));
            let hit = Hit {
                damage: 30,
                ..zero.hit
            }
            .impact(3, npc);
            assert_eq!((hit.applied, hit.health, hit.death), (3, 0, true));
            assert_eq!(hit.displayed, if npc { 3 } else { 30 });
        }
    }
    #[test]
    fn protected_hits_count_down_once_per_cycle_and_keep_order() {
        let hit = |damage| Hit {
            display: None,
            source: None,
            style: Style::Typeless,
            damage,
        };
        let mut queue = Queue::default();
        queue.push(hit(1), 2).unwrap();
        queue.push(hit(2), 1).unwrap();
        assert!(queue.process(5, true).is_empty());
        assert_eq!(queue.process(5, false), [hit(2)]);
        assert_eq!(queue.process(6, false), [hit(1)]);
        assert_eq!(queue.push(hit(-1), 1), Err(Error::InvalidDamage));
    }
    #[test]
    fn npc_melee_requires_live_source_but_projectiles_survive_source_deletion() {
        let mut hit = Hit {
            display: None,
            source: Some(Target::Npc(NpcId {
                slot: 0,
                generation: 1,
            })),
            style: Style::Melee,
            damage: 50,
        };
        assert!(!hit.valid_player_source(|_| false));
        hit.style = Style::Ranged;
        assert!(hit.valid_player_source(|_| false));
        let player = hit.impact(10, false);
        assert_eq!(
            (player.applied, player.displayed, player.health),
            (10, 50, 0)
        );
        let npc = hit.impact(10, true);
        assert_eq!((npc.applied, npc.displayed, npc.health), (10, 10, 0));
        assert!(npc.death);
    }
}
