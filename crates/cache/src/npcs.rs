//! NPC cache fields used by simulation, with visual data retained for future consumers.
use crate::definitions::{params, Param};
use gielinor_protocol::buffer::{DecodeError, Reader};
use gielinor_world::{
    npc::{BlockWalk, Mode, Npc, Restrict},
    CoordGrid,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct NpcType {
    pub name: String,
    pub description: String,
    pub size: u8,
    pub ops: [Option<String>; 5],
    /// Attack, defence, strength, hitpoints, ranged, magic (cache opcode order).
    pub stats: [i32; 6],
    pub category: i32,
    pub active: bool,
    pub follower: bool,
    pub restrict: Restrict,
    pub block_walk: BlockWalk,
    pub default_mode: u8,
    pub patrol: Vec<(CoordGrid, u8)>,
    pub respawn_rate: u16,
    pub max_range: u8,
    pub wander_range: u8,
    pub attack_range: u8,
    pub hunt_range: u8,
    pub hunt_mode: Option<u16>,
    pub give_chase: bool,
    pub timer: i32,
    pub respawn_direction: u8,
    pub content_group: i32,
    pub hero_count: u16,
    pub regen_rate: u16,
    pub multi_varp: Option<u16>,
    pub multi_varbit: Option<u16>,
    pub multi_default: Option<u16>,
    pub multi: Vec<Option<u16>>,
    pub params: BTreeMap<u32, Param>,
    /// RspCycle transformation order: ready, turn-back, walk, walk, left, right, run.
    pub animations: [u16; 7],
    pub encoded: Vec<u8>,
}
fn nullable(r: &mut Reader<'_>) -> Result<Option<u16>, DecodeError> {
    let value = r.u16()?;
    Ok((value != 65535).then_some(value))
}
impl NpcType {
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut d = Self {
            name: "null".into(),
            description: String::new(),
            size: 1,
            ops: std::array::from_fn(|_| None),
            stats: [1, 1, 1, 10, 1, 1],
            category: -1,
            active: true,
            follower: false,
            restrict: Restrict::Normal,
            block_walk: BlockWalk::Npc,
            default_mode: 1,
            patrol: vec![],
            respawn_rate: 100,
            max_range: 7,
            wander_range: 5,
            attack_range: 1,
            hunt_range: 5,
            hunt_mode: None,
            give_chase: true,
            timer: -1,
            respawn_direction: 6,
            content_group: -1,
            hero_count: 16,
            regen_rate: 100,
            multi_varp: None,
            multi_varbit: None,
            multi_default: None,
            multi: vec![],
            params: BTreeMap::new(),
            animations: [u16::MAX; 7],
            encoded: bytes.to_vec(),
        };
        let mut r = Reader::new(bytes);
        while r.remaining() > 0 {
            let code = r.u8()?;
            match code {
                0 => break,
                1 | 60 => {
                    let count = r.u8()? as usize;
                    r.take(count * 2)?;
                }
                61 | 62 => {
                    let count = r.u8()? as usize;
                    r.take(count * 4)?;
                }
                2 => d.name = r.string()?,
                3 => d.description = r.string()?,
                12 => d.size = r.u8()?,
                13 => d.animations[0] = r.u16()?,
                14 => {
                    let walk = r.u16()?;
                    d.animations[2] = walk;
                    d.animations[3] = walk;
                }
                15 => d.animations[4] = r.u16()?,
                16 => d.animations[5] = r.u16()?,
                17 => {
                    let walk = r.u16()?;
                    d.animations[2] = walk;
                    d.animations[3] = walk;
                    d.animations[1] = r.u16()?;
                    d.animations[4] = r.u16()?;
                    d.animations[5] = r.u16()?;
                }
                114 => d.animations[6] = r.u16()?,
                115 => {
                    d.animations[6] = r.u16()?;
                    r.take(6)?;
                }
                95 | 97 | 98 | 103 | 116 | 124 | 126 | 146 => {
                    r.u16()?;
                }
                117 => {
                    r.take(8)?;
                }
                18 => d.category = r.u16()? as i32,
                30..=34 => d.ops[(code - 30) as usize] = Some(r.string()?),
                40 | 41 => {
                    let count = r.u8()? as usize;
                    r.take(count * 4)?;
                }
                74..=79 => d.stats[(code - 74) as usize] = r.u16()? as i32,
                93 | 99 | 109 | 111 | 123 | 129 | 130 | 145 | 147 => {}
                100 | 101 => {
                    r.u8()?;
                }
                102 => {
                    let flags = r.u8()?;
                    for bit in 0..8 {
                        if flags & (1 << bit) == 0 {
                            continue;
                        }
                        let first = r.u8()?;
                        if first & 128 != 0 {
                            r.take(3)?;
                        } else {
                            r.u8()?;
                        }
                        r.smart()?;
                    }
                }
                106 | 118 => {
                    d.multi_varbit = nullable(&mut r)?;
                    d.multi_varp = nullable(&mut r)?;
                    d.multi_default = if code == 118 { nullable(&mut r)? } else { None };
                    let count = r.u8()? as usize + 1;
                    d.multi.clear();
                    for _ in 0..count {
                        d.multi.push(nullable(&mut r)?);
                    }
                }
                107 => d.active = false,
                122 => d.follower = true,
                200 => {
                    d.restrict = match r.u8()? {
                        0 => Restrict::Normal,
                        1 => Restrict::Blocked,
                        2 => Restrict::BlockedNormal,
                        3 => Restrict::Indoors,
                        4 => Restrict::Outdoors,
                        5 => Restrict::NoMove,
                        6 => Restrict::PassThru,
                        _ => return Err(DecodeError::InvalidValue("NPC restriction")),
                    }
                }
                201 => {
                    d.default_mode = r.u8()?;
                    if d.default_mode > 72 {
                        return Err(DecodeError::InvalidValue("NPC mode"));
                    }
                }
                202 => {
                    d.block_walk = match r.u8()? {
                        0 => BlockWalk::None,
                        1 => BlockWalk::Npc,
                        2 => BlockWalk::All,
                        _ => return Err(DecodeError::InvalidValue("NPC block walk")),
                    }
                }
                203 => {
                    let count = r.u8()? as usize + 1;
                    d.patrol.clear();
                    for _ in 0..count {
                        d.patrol
                            .push((CoordGrid::from_packed(r.u32()? as i32), r.u8()?));
                    }
                }
                204 => d.respawn_rate = r.u16()?,
                205 => d.max_range = r.u8()?,
                206 => d.wander_range = r.u8()?,
                207 => d.attack_range = r.u8()?,
                208 => d.hunt_range = r.u8()?,
                209 => d.hunt_mode = Some(r.u8()? as u16),
                210 => d.give_chase = false,
                211 => d.timer = r.u16()? as i32,
                212 => {
                    d.respawn_direction = r.u8()?;
                    if d.respawn_direction > 7 {
                        return Err(DecodeError::InvalidValue("NPC direction"));
                    }
                }
                213 => d.content_group = r.u16()? as i32,
                214 => d.hero_count = r.u16()?,
                215 => d.regen_rate = r.u16()?,
                216 => d.hunt_mode = nullable(&mut r)?,
                249 => d.params = params(&mut r)?,
                251 => {
                    r.take(2)?;
                    r.string()?;
                }
                252 => {
                    r.take(13)?;
                    r.string()?;
                }
                253 => {
                    r.take(15)?;
                    r.string()?;
                }
                _ => return Err(DecodeError::InvalidValue("unknown NPC opcode")),
            }
        }
        if d.stats[3] < 1 {
            return Err(DecodeError::InvalidValue("NPC hitpoints"));
        }
        Ok(d)
    }
    pub fn transform(&self, selector: i32) -> Option<u16> {
        if selector >= 0 && (selector as usize) < self.multi.len() {
            self.multi[selector as usize]
        } else {
            self.multi_default
        }
    }
    /// Unsupported behavior modes are explicit errors rather than silently becoming wanderers.
    pub fn spawn(&self, id: u16, coords: CoordGrid) -> Result<Npc, &'static str> {
        let mode = match self.default_mode {
            0 => Mode::None,
            1 => Mode::Wander,
            2 => Mode::Patrol,
            _ => return Err("NPC behavior mode is not implemented"),
        };
        let mut npc = Npc::new(id, coords, self.size as i32, self.restrict, self.block_walk)?;
        npc.mode = mode;
        npc.ai_timer = self.timer;
        npc.respawn_direction = self.respawn_direction;
        npc.wander_radius = self.wander_range as i32;
        npc.base_stats = self.stats;
        npc.stats = self.stats;
        npc.regen_rate = self.regen_rate as i32;
        npc.respawn_rate = self.respawn_rate as i32;
        npc.hero_points = gielinor_world::hero_points::HeroPoints::new(self.hero_count as usize);
        npc.patrol = self.patrol.clone();
        Ok(npc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn npc_ai_countdown_is_loaded_without_changing_spawn_stats(){
        let mut d=NpcType::decode(&[0]).unwrap();d.timer=1;let npc=d.spawn(2693,CoordGrid::new(3200,3200,0).unwrap()).unwrap();assert_eq!(npc.ai_timer,1);assert_eq!(npc.stats,d.stats);assert_eq!(npc.type_id,2693);assert_eq!(npc.visible_type(),2693);
        d.timer=-1;assert_eq!(d.spawn(2693,CoordGrid::new(3200,3200,0).unwrap()).unwrap().ai_timer,-1);
    }
    #[test]
    fn enriched_fields_and_null_transform_do_not_fall_back() {
        let d = NpcType::decode(&[
            77, 0, 25, 200, 6, 201, 0, 202, 2, 206, 9, 215, 0, 50, 118, 255, 255, 0, 7, 0, 42, 1,
            255, 255, 0, 55, 0,
        ])
        .unwrap();
        assert_eq!(d.transform(0), None);
        assert_eq!(d.transform(1), Some(55));
        assert_eq!(d.transform(-1), Some(42));
        let npc = d.spawn(12, CoordGrid::from_packed(0)).unwrap();
        assert_eq!(npc.stats, [1, 1, 1, 25, 1, 1]);
        assert_eq!(npc.regen_rate, 50);
        assert_eq!(npc.wander_radius, 9);
        assert_eq!(npc.restrict, Restrict::PassThru);
        assert_eq!(npc.movement.collision_mask, Some(0x180000));
    }
    #[test]
    fn unsupported_modes_and_invalid_cache_fields_are_reported() {
        let d = NpcType::decode(&[201, 3, 0]).unwrap();
        assert!(d.spawn(1, CoordGrid::from_packed(0)).is_err());
        assert!(NpcType::decode(&[77, 0, 0, 0]).is_err());
        assert!(NpcType::decode(&[200, 7, 0]).is_err());
        assert!(NpcType::decode(&[252, 0]).is_err());
    }
}
