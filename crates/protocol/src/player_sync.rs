//! Stateful four-pass player movement synchronization. Visibility policy belongs to the world.
//! Extension blocks are supplied in revision-specific wire form, including their mask.
use crate::buffer::{BitWriter, DecodeError};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub x: u16,
    pub z: u16,
    pub level: u8,
}
impl Position {
    fn valid(self) -> bool {
        self.x < 16384 && self.z < 16384 && self.level < 4
    }
    fn packed(self) -> u32 {
        (self.level as u32) << 28 | (self.x as u32) << 14 | self.z as u32
    }
    fn low(self) -> u32 {
        (self.level as u32) << 16
            | ((self.x as u32 >> 13) & 255) << 8
            | ((self.z as u32 >> 13) & 255)
    }
}
pub struct Player<'a> {
    pub slot: u16,
    pub generation: u64,
    pub position: Position,
    pub visible: bool,
    pub teleport: bool,
    pub extended: &'a [u8],
    pub initial_extended: &'a [u8],
}
#[derive(Clone, Copy, Default)]
struct Tracked {
    high: bool,
    flags: u8,
    generation: u64,
    position: Option<Position>,
    low: u32,
}
#[derive(Clone)]
pub struct State {
    local: u16,
    tracked: Vec<Tracked>,
}
impl State {
    /// The login GPI block is 30 local-coordinate bits followed by 2,046 18-bit regions.
    pub fn initialize(
        local: u16,
        position: Position,
        generation: u64,
        regions: &[u32; 2048],
    ) -> Result<(Self, Vec<u8>), DecodeError> {
        if !(1..2048).contains(&local) || !position.valid() || regions.iter().any(|v| *v >= 1 << 18)
        {
            return Err(DecodeError::InvalidValue("player initialization"));
        }
        let mut tracked = vec![Tracked::default(); 2048];
        let mut bits = BitWriter::default();
        bits.write(30, position.packed())?;
        for slot in 1..2048 {
            tracked[slot].low = regions[slot];
            if slot == local as usize {
                tracked[slot].high = true;
                tracked[slot].position = Some(position);
                tracked[slot].generation = generation;
                tracked[slot].low = position.low();
            } else {
                bits.write(18, regions[slot])?;
            }
        }
        Ok((Self { local, tracked }, bits.into_bytes()))
    }
    /// Commit tracking only if the entire frame validates and fits a variable-short packet.
    pub fn encode(&mut self, players: &[Player<'_>]) -> Result<Vec<u8>, DecodeError> {
        let mut image = self.clone();
        let bytes = image.encode_inner(players)?;
        *self = image;
        Ok(bytes)
    }
    fn encode_inner(&mut self, players: &[Player<'_>]) -> Result<Vec<u8>, DecodeError> {
        let mut by_slot = vec![None; 2048];
        for player in players {
            let slot = player.slot as usize;
            if slot == 0
                || slot >= 2048
                || !player.position.valid()
                || by_slot[slot].is_some()
                || player.extended.len() > 8192
                || player.initial_extended.len() > 8192
            {
                return Err(DecodeError::InvalidValue("player snapshot"));
            }
            by_slot[slot] = Some(player);
        }
        let local = by_slot[self.local as usize]
            .ok_or(DecodeError::InvalidValue("local player missing"))?;
        if !local.visible || local.generation != self.tracked[self.local as usize].generation {
            return Err(DecodeError::InvalidValue("local player replaced"));
        }
        let previous = self.tracked.clone();
        let mut output = vec![];
        let mut extended = vec![];
        for (high, flag) in [(true, 0), (true, 1), (false, 1), (false, 0)] {
            let slots: Vec<_> = (1..2048)
                .filter(|slot| previous[*slot].high == high && previous[*slot].flags & 1 == flag)
                .collect();
            let mut bits = BitWriter::default();
            let mut cursor = 0;
            while cursor < slots.len() {
                let slot = slots[cursor];
                let old = previous[slot];
                let player = by_slot[slot];
                let visible = player.is_some_and(|p| p.visible);
                let current_low = player.map_or(old.low, |p| p.position.low());
                let unchanged = if high {
                    player.is_some_and(|p| {
                        p.visible
                            && p.generation == old.generation
                            && Some(p.position) == old.position
                            && p.extended.is_empty()
                            && !p.teleport
                    })
                } else {
                    !visible && current_low == old.low
                };
                if unchanged {
                    let mut end = cursor + 1;
                    while end < slots.len() {
                        let s = slots[end];
                        let t = previous[s];
                        let p = by_slot[s];
                        let same = if high {
                            p.is_some_and(|p| {
                                p.visible
                                    && p.generation == t.generation
                                    && Some(p.position) == t.position
                                    && p.extended.is_empty()
                                    && !p.teleport
                            })
                        } else {
                            !p.is_some_and(|p| p.visible)
                                && p.map_or(t.low, |p| p.position.low()) == t.low
                        };
                        if !same {
                            break;
                        }
                        end += 1;
                    }
                    skip(&mut bits, (end - cursor - 1) as u32)?;
                    for s in &slots[cursor..end] {
                        self.tracked[*s].flags |= 2;
                    }
                    cursor = end;
                    continue;
                }
                bits.write(1, 1)?;
                if high {
                    if !visible || player.is_some_and(|p| p.generation != old.generation) {
                        bits.write(3, 0)?;
                        let baseline = old.position.unwrap().low();
                        bits.write(1, (current_low != baseline) as u32)?;
                        if current_low != baseline {
                            low_change(&mut bits, baseline, current_low)?;
                        }
                        self.tracked[slot].high = false;
                        self.tracked[slot].position = None;
                        self.tracked[slot].low = current_low;
                    } else {
                        let p = player.unwrap();
                        bits.write(1, (!p.extended.is_empty()) as u32)?;
                        movement(&mut bits, old.position.unwrap(), p.position, p.teleport)?;
                        if !p.extended.is_empty() {
                            extended.extend_from_slice(p.extended);
                        }
                        self.tracked[slot].position = Some(p.position);
                        self.tracked[slot].low = p.position.low();
                    }
                } else if visible {
                    let p = player.unwrap();
                    bits.write(2, 0)?;
                    bits.write(1, (current_low != old.low) as u32)?;
                    if current_low != old.low {
                        low_change(&mut bits, old.low, current_low)?;
                    }
                    bits.write(13, p.position.x as u32 & 8191)?;
                    bits.write(13, p.position.z as u32 & 8191)?;
                    bits.write(1, (!p.initial_extended.is_empty()) as u32)?;
                    extended.extend_from_slice(p.initial_extended);
                    self.tracked[slot] = Tracked {
                        high: true,
                        flags: old.flags | 2,
                        generation: p.generation,
                        position: Some(p.position),
                        low: current_low,
                    };
                } else {
                    low_change(&mut bits, old.low, current_low)?;
                    self.tracked[slot].low = current_low;
                }
                cursor += 1;
            }
            output.extend(bits.into_bytes());
        }
        output.extend(extended);
        if output.len() > 65535 {
            return Err(DecodeError::InvalidValue("player info packet too large"));
        }
        for t in &mut self.tracked {
            t.flags >>= 1;
        }
        Ok(output)
    }
}
fn skip(bits: &mut BitWriter, count: u32) -> Result<(), DecodeError> {
    bits.write(1, 0)?;
    let (kind, width) = if count == 0 {
        (0, 0)
    } else if count < 32 {
        (1, 5)
    } else if count < 256 {
        (2, 8)
    } else {
        (3, 11)
    };
    bits.write(2, kind)?;
    bits.write(width, count)
}
fn walk(dx: i32, dz: i32) -> Option<u32> {
    [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ]
    .iter()
    .position(|p| *p == (dx, dz))
    .map(|i| i as u32)
}
fn run(dx: i32, dz: i32) -> Option<u32> {
    [
        (-2, -2),
        (-1, -2),
        (0, -2),
        (1, -2),
        (2, -2),
        (-2, -1),
        (2, -1),
        (-2, 0),
        (2, 0),
        (-2, 1),
        (2, 1),
        (-2, 2),
        (-1, 2),
        (0, 2),
        (1, 2),
        (2, 2),
    ]
    .iter()
    .position(|p| *p == (dx, dz))
    .map(|i| i as u32)
}
fn movement(
    bits: &mut BitWriter,
    from: Position,
    to: Position,
    teleport: bool,
) -> Result<(), DecodeError> {
    let dx = to.x as i32 - from.x as i32;
    let dz = to.z as i32 - from.z as i32;
    let dl = to.level.wrapping_sub(from.level) & 3;
    if !teleport && dl == 0 {
        if dx == 0 && dz == 0 {
            return bits.write(2, 0);
        }
        if let Some(dir) = walk(dx, dz) {
            bits.write(2, 1)?;
            return bits.write(3, dir);
        }
        if let Some(dir) = run(dx, dz) {
            bits.write(2, 2)?;
            return bits.write(4, dir);
        }
    }
    bits.write(2, 3)?;
    if (-16..16).contains(&dx) && (-16..16).contains(&dz) {
        bits.write(1, 0)?;
        bits.write(
            12,
            (dl as u32) << 10 | ((dx as u32) & 31) << 5 | (dz as u32 & 31),
        )
    } else {
        bits.write(1, 1)?;
        bits.write(
            30,
            (dl as u32) << 28 | ((dx as u32) & 16383) << 14 | (dz as u32 & 16383),
        )
    }
}
fn low_change(bits: &mut BitWriter, from: u32, to: u32) -> Result<(), DecodeError> {
    let dl = ((to >> 16).wrapping_sub(from >> 16)) & 3;
    let dx = (((to >> 8) & 255) as i32) - (((from >> 8) & 255) as i32);
    let dz = (to & 255) as i32 - (from & 255) as i32;
    if dx == 0 && dz == 0 {
        bits.write(2, 1)?;
        bits.write(2, dl)
    } else if let Some(dir) = walk(dx, dz) {
        bits.write(2, 2)?;
        bits.write(5, dl << 3 | dir)
    } else {
        bits.write(2, 3)?;
        bits.write(18, dl << 16 | ((dx as u32) & 255) << 8 | (dz as u32 & 255))
    }
}

pub fn rebuild_normal(zone_x: u16, zone_z: u16, world_area: u16) -> [u8; 6] {
    let a = world_area.to_le_bytes();
    let x = zone_x.to_be_bytes();
    let z = zone_z.to_be_bytes();
    [
        a[0],
        a[1],
        z[0],
        z[1].wrapping_add(128),
        x[0],
        x[1].wrapping_add(128),
    ]
}
pub fn rebuild_login(
    init: &[u8],
    zone_x: u16,
    zone_z: u16,
    world_area: u16,
) -> Result<Vec<u8>, DecodeError> {
    if init.len() != 4608 {
        return Err(DecodeError::InvalidValue("GPI initialization length"));
    }
    let mut bytes = init.to_vec();
    bytes.extend(rebuild_normal(zone_x, zone_z, world_area));
    Ok(bytes)
}
pub fn update_stat(stat: u8, current: u8, invisible: u8, xp: i32) -> [u8; 7] {
    let b = xp.to_be_bytes();
    [
        invisible,
        current,
        stat.wrapping_add(128),
        b[2],
        b[3],
        b[0],
        b[1],
    ]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn initialization_and_stationary_passes() {
        let p = Position {
            x: 3200,
            z: 3200,
            level: 0,
        };
        let (mut s, init) = State::initialize(1, p, 1, &[0; 2048]).unwrap();
        assert_eq!(init.len(), 4608);
        assert_eq!(rebuild_login(&init, 400, 400, 0).unwrap().len(), 4614);
        let player = Player {
            slot: 1,
            generation: 1,
            position: p,
            visible: true,
            teleport: false,
            extended: &[],
            initial_extended: &[],
        };
        assert_eq!(s.encode(&[player]).unwrap(), [0, 0x7f, 0xf4]);
        let player = Player {
            slot: 1,
            generation: 1,
            position: p,
            visible: true,
            teleport: false,
            extended: &[],
            initial_extended: &[],
        };
        assert_eq!(s.encode(&[player]).unwrap(), [0, 0x7f, 0xf4]);
    }
    #[test]
    fn invalid_snapshot_does_not_advance_state() {
        let p = Position {
            x: 3200,
            z: 3200,
            level: 0,
        };
        let (mut s, _) = State::initialize(1, p, 1, &[0; 2048]).unwrap();
        assert!(s.encode(&[]).is_err());
        assert_eq!(s.tracked[1].flags, 0);
    }
}
