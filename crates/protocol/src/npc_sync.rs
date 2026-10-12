//! Revision 239 NPC_INFO_SMALL/LARGE_V5 movement stream. Extension masks are supplied
//! separately; policy (visibility, prioritization, and viewport capacity) belongs to the server.
use crate::buffer::{BitWriter, DecodeError};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub x: u16,
    pub z: u16,
    pub level: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Movement {
    None,
    Walk(u8),
    Crawl(u8),
    Run(u8, u8),
}
const DIRECTIONS: [(i32, i32); 8] = [
    (-1, 1),
    (0, 1),
    (1, 1),
    (-1, 0),
    (1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];
pub fn direction(dx: i32, dz: i32) -> Option<u8> {
    DIRECTIONS
        .iter()
        .position(|d| *d == (dx, dz))
        .map(|n| n as u8)
}
pub struct Npc<'a> {
    pub index: u16,
    pub generation: u64,
    pub type_id: u16,
    pub position: Position,
    pub visible: bool,
    pub teleport: bool,
    pub jump: bool,
    pub direction: u8,
    pub spawn_cycle: Option<u32>,
    pub movement: Movement,
    pub extended: &'a [u8],
    pub initial_extended: &'a [u8],
}
#[derive(Clone)]
struct Known {
    index: u16,
    generation: u64,
    type_id: u16,
    position: Position,
}
#[derive(Clone, Default)]
pub struct State {
    known: Vec<Known>,
}
fn invalid() -> DecodeError {
    DecodeError::InvalidValue("NPC synchronization")
}
fn valid(p: Position) -> bool {
    p.x < 16384 && p.z < 16384 && p.level < 4
}
fn arrival(mut p: Position, movement: Movement) -> Option<Position> {
    let steps: Vec<_> = match movement {
        Movement::None => vec![],
        Movement::Walk(d) | Movement::Crawl(d) => vec![d],
        Movement::Run(a, b) => vec![a, b],
    };
    for d in steps {
        let &(dx, dz) = DIRECTIONS.get(d as usize)?;
        p.x = u16::try_from(p.x as i32 + dx).ok()?;
        p.z = u16::try_from(p.z as i32 + dz).ok()?;
    }
    valid(p).then_some(p)
}
impl State {
    pub fn indices(&self) -> impl Iterator<Item = u16> + '_ {
        self.known.iter().map(|n| n.index)
    }
    pub fn encode(
        &mut self,
        origin: Position,
        large: bool,
        npcs: &[Npc<'_>],
        capacity: usize,
    ) -> Result<Vec<u8>, DecodeError> {
        if !valid(origin) || capacity > 255 {
            return Err(invalid());
        }
        let width = if large { 8 } else { 6 };
        let limit = 1i32 << (width - 1);
        let mut by_index = BTreeMap::new();
        for n in npcs {
            if n.index == 65535
                || n.type_id >= 16384
                || n.direction >= 8
                || !valid(n.position)
                || n.extended.len() > 8192
                || n.initial_extended.len() > 8192
                || by_index.insert(n.index, n).is_some()
            {
                return Err(invalid());
            }
            let directions_valid = match n.movement {
                Movement::None => true,
                Movement::Walk(d) | Movement::Crawl(d) => d < 8,
                Movement::Run(a, b) => a < 8 && b < 8,
            };
            if !directions_valid {
                return Err(invalid());
            }
        }
        let eligible = |n: &Npc<'_>| {
            n.visible
                && n.position.level == origin.level
                && (-limit..limit).contains(&(n.position.x as i32 - origin.x as i32))
                && (-limit..limit).contains(&(n.position.z as i32 - origin.z as i32))
        };
        let mut bits = BitWriter::default();
        let mut extended: Vec<u8> = vec![];
        let mut next = vec![];
        let mut retained = BTreeSet::new();
        let mut kept = 0;
        let candidates: Vec<_> = self
            .known
            .iter()
            .map(|old| {
                let current = by_index.get(&old.index).copied().filter(|n| {
                    eligible(n)
                        && n.generation == old.generation
                        && n.type_id == old.type_id
                        && !n.teleport
                        && arrival(old.position, n.movement) == Some(n.position)
                        && kept < capacity
                });
                if current.is_some() {
                    kept += 1;
                }
                current
            })
            .collect();
        // RSProt truncates a removed suffix before writing the retained-count byte.
        let count = candidates
            .iter()
            .rposition(Option::is_some)
            .map_or(0, |i| i + 1);
        bits.write(8, count as u32)?;
        for current in candidates.into_iter().take(count) {
            let Some(n) = current else {
                bits.write(1, 1)?;
                bits.write(2, 3)?;
                continue;
            };
            let update = !n.extended.is_empty();
            if n.movement == Movement::None && !update {
                bits.write(1, 0)?;
            } else {
                bits.write(1, 1)?;
                match n.movement {
                    Movement::None => bits.write(2, 0)?,
                    Movement::Walk(d) => {
                        bits.write(2, 1)?;
                        bits.write(3, d as u32)?;
                        bits.write(1, update as u32)?;
                    }
                    Movement::Crawl(d) => {
                        bits.write(2, 2)?;
                        bits.write(1, 0)?;
                        bits.write(3, d as u32)?;
                        bits.write(1, update as u32)?;
                    }
                    Movement::Run(a, b) => {
                        bits.write(2, 2)?;
                        bits.write(1, 1)?;
                        bits.write(3, a as u32)?;
                        bits.write(3, b as u32)?;
                        bits.write(1, update as u32)?;
                    }
                }
            }
            if update {
                extended.extend(n.extended);
            }
            retained.insert(n.index);
            next.push(Known {
                index: n.index,
                generation: n.generation,
                type_id: n.type_id,
                position: n.position,
            });
        }
        // Input order is the caller's NPC prioritization order; never sort additions by index.
        for n in npcs {
            if next.len() >= capacity {
                break;
            }
            if retained.contains(&n.index) || !eligible(n) {
                continue;
            }
            bits.write(16, n.index as u32)?;
            let spawn_cycle = n.spawn_cycle.filter(|c| *c != 0);
            bits.write(1, spawn_cycle.is_some() as u32)?;
            if let Some(cycle) = spawn_cycle {
                bits.write(32, cycle)?;
            }
            bits.write(1, (!n.initial_extended.is_empty()) as u32)?;
            let mask = (1u32 << width) - 1;
            bits.write(
                width,
                ((n.position.x as i32 - origin.x as i32) as u32) & mask,
            )?;
            bits.write(3, n.direction as u32)?;
            bits.write(
                width,
                ((n.position.z as i32 - origin.z as i32) as u32) & mask,
            )?;
            bits.write(1, n.jump as u32)?;
            bits.write(14, n.type_id as u32)?;
            extended.extend(n.initial_extended);
            next.push(Known {
                index: n.index,
                generation: n.generation,
                type_id: n.type_id,
                position: n.position,
            });
        }
        if !extended.is_empty() {
            bits.write(16, 65535)?;
        }
        let mut payload = bits.into_bytes();
        payload.extend(extended);
        if payload.len() > 65535 {
            return Err(DecodeError::InvalidValue("NPC frame length"));
        }
        self.known = next;
        Ok(payload)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn npc(index: u16) -> Npc<'static> {
        Npc {
            index,
            generation: 1,
            type_id: 42,
            position: Position {
                x: 3200,
                z: 3200,
                level: 0,
            },
            visible: true,
            teleport: false,
            jump: true,
            direction: 6,
            spawn_cycle: None,
            movement: Movement::None,
            extended: &[],
            initial_extended: &[],
        }
    }
    #[test]
    fn additions_retention_removal_and_transactional_failure() {
        let origin = npc(1).position;
        let mut state = State::default();
        let mut n = npc(1);
        let add = state.encode(origin, false, &[npc(1)], 255).unwrap();
        assert_eq!(add.len(), 7);
        assert_eq!(state.encode(origin, false, &[npc(1)], 255).unwrap(), [1, 0]);
        n.position.x += 1;
        n.movement = Movement::Walk(4);
        assert_eq!(state.encode(origin, false, &[n], 255).unwrap(), [1, 0xb0]);
        let mut bad = npc(2);
        bad.direction = 8;
        assert!(state.encode(origin, false, &[bad], 255).is_err());
        assert_eq!(state.indices().collect::<Vec<_>>(), [1]);
        assert_eq!(state.encode(origin, false, &[], 255).unwrap(), [0]);
        assert_eq!(state.indices().count(), 0);
    }
    #[test]
    fn bounds_capacity_and_generation_replacement() {
        let origin = npc(1).position;
        let mut state = State::default();
        let mut n = npc(1);
        n.position.x += 32;
        assert_eq!(state.encode(origin, false, &[n], 255).unwrap(), [0]);
        state.encode(origin, true, &[npc(1), npc(2)], 1).unwrap();
        assert_eq!(state.indices().collect::<Vec<_>>(), [1]);
        let mut replaced = npc(1);
        replaced.generation = 2;
        assert!(state.encode(origin, false, &[replaced], 255).unwrap().len() > 2);
        assert_eq!(state.indices().collect::<Vec<_>>(), [1]);
    }
}
