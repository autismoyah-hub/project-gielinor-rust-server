use super::*;
use std::cmp::Reverse;

pub(super) fn zone(coords: CoordGrid) -> [i32; 3] {
    [coords.level(), coords.x() >> 3, coords.z() >> 3]
}
impl World {
    /// NpcRegistry.change removes and appends even when a respawn stays in its
    /// current zone. Ordinary post-tick movement only changes a different zone.
    pub(super) fn update_npc_hunt_zone(&mut self, id: NpcId, force: bool) {
        let Some(entry) = self.npcs.get_mut(id.slot as usize).filter(|e| e.id == id) else {
            return;
        };
        let next = zone(entry.npc.movement.coords);
        if !force && next == entry.hunt_zone {
            return;
        }
        self.npc_hunt_order = self
            .npc_hunt_order
            .checked_add(1)
            .expect("NPC zone order exhausted");
        entry.hunt_zone = next;
        entry.hunt_order = self.npc_hunt_order;
    }
    /// NpcSearch.find with HuntVis.Off. Unlike combat hunt, no attack operation
    /// is required. Ties select the last encountered NPC: descending zone X/Z
    /// traversal, then zone insertion order. Registry zones intentionally lag
    /// movement until NpcPostTickProcess, while distance uses current coords.
    pub fn find_npc(
        &self,
        center: CoordGrid,
        type_id: u16,
        distance: u16,
    ) -> Result<Option<NpcId>, &'static str> {
        self.find_npc_with_sight(center,type_id,distance,false)
    }
    pub fn find_npc_line_of_sight(&self,center:CoordGrid,type_id:u16,distance:u16)->Result<Option<NpcId>,&'static str>{
        self.find_npc_with_sight(center,type_id,distance,true)
    }
    pub fn find_all_npcs_line_of_sight(&self,center:CoordGrid,type_id:u16,distance:u16)->Result<Vec<NpcId>,&'static str>{
        let mut candidates=self.npc_search_candidates(center,type_id,distance,true)?.collect::<Vec<_>>();
        candidates.sort_by_key(|((_,x,z,Reverse(order)),_)|(Reverse(*x),Reverse(*z),*order));
        Ok(candidates.into_iter().map(|(_,id)|id).collect())
    }
    fn find_npc_with_sight(&self,center:CoordGrid,type_id:u16,distance:u16,sight:bool)->Result<Option<NpcId>,&'static str>{
        Ok(self.npc_search_candidates(center,type_id,distance,sight)?.into_iter().min_by_key(|(key,_)|*key).map(|(_,id)|id))
    }
    fn npc_search_candidates(&self,center:CoordGrid,type_id:u16,distance:u16,sight:bool)->Result<impl Iterator<Item=((i32,i32,i32,Reverse<u64>),NpcId)>+'_ ,&'static str>{
        let origin = zone(center);
        let zone_distance = (i32::from(distance) + 7) / 8;
        // Hunt consumes the entire zone square; ZoneKey.translate throws at
        // map edges even if a matching NPC was encountered earlier.
        if origin[1] - zone_distance < 0
            || origin[2] - zone_distance < 0
            || origin[1] + zone_distance > 2047
            || origin[2] + zone_distance > 2047
        {
            return Err("NPC hunt zone outside map bounds");
        }
        Ok(self
            .npcs
            .iter()
            .filter_map(move |(_, entry)| {
                let npc = &entry.npc;
                if !npc.visible
                    || npc.type_id != type_id
                    || entry.hunt_zone[0] != origin[0]
                    || (entry.hunt_zone[1] - origin[1]).abs() > zone_distance
                    || (entry.hunt_zone[2] - origin[2]).abs() > zone_distance
                {
                    return None;
                }
                let dx = npc.movement.coords.x() - center.x();
                let dz = npc.movement.coords.z() - center.z();
                if dx.abs().max(dz.abs()) > i32::from(distance) {
                    return None;
                }
                if sight&&!crate::raycast::clear(&self.collision,crate::raycast::Bounds{coords:center,width:1,length:1},crate::raycast::Bounds{coords:npc.movement.coords,width:1,length:1},true,0){return None;}
                Some((
                    (
                        dx * dx + dz * dz,
                        entry.hunt_zone[1],
                        entry.hunt_zone[2],
                        Reverse(entry.hunt_order),
                    ),
                    entry.id,
                ))
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::{BlockWalk, Npc, Restrict};
    fn spawn(world: &mut World, x: i32, z: i32) -> NpcId {
        world
            .spawn_npc(
                Npc::new(
                    1,
                    CoordGrid::new(x, z, 0).unwrap(),
                    1,
                    Restrict::NoMove,
                    BlockWalk::None,
                )
                .unwrap(),
            )
            .unwrap()
    }
    #[test]
    fn zone_membership_changes_in_post_tick_not_at_coordinate_assignment() {
        let mut world = World::default();
        let center = CoordGrid::new(3200, 3200, 0).unwrap();
        let older = spawn(&mut world, 3201, 3200);
        let newer = spawn(&mut world, 3201, 3200);
        world.npc_mut(older).unwrap().movement.coords = CoordGrid::new(3199, 3200, 0).unwrap();
        world.npc_mut(older).unwrap().movement.pending_telejump = true;
        assert_eq!(world.find_npc(center, 1, 64).unwrap(), Some(newer));
        world.tick().unwrap();
        assert_eq!(world.find_npc(center, 1, 64).unwrap(), Some(older));
        assert!(world.find_npc(CoordGrid::ZERO, 1, 64).is_err());
    }
    #[test]
    fn hunt_ties_follow_zone_order_then_insertion_not_slot_order() {
        let mut world = World::default();
        let center = CoordGrid::new(3200, 3200, 0).unwrap();
        let west = spawn(&mut world, 3199, 3200);
        let east = spawn(&mut world, 3201, 3200);
        assert_eq!(world.find_npc(center, 1, 64).unwrap(), Some(west));
        world.despawn_npc(west).unwrap();
        let later = spawn(&mut world, 3201, 3200);
        assert_eq!(world.find_npc(center, 1, 64).unwrap(), Some(later));
        world.npc_mut(later).unwrap().visible = false;
        assert_eq!(world.find_npc(center, 1, 64).unwrap(), Some(east));
        world.npc_mut(later).unwrap().visible = true;
        world.update_npc_hunt_zone(east, true);
        assert_eq!(world.find_npc(center, 1, 64).unwrap(), Some(east));
        assert_eq!(world.find_npc(center, 1, 0).unwrap(), None);
        assert_eq!(world.find_npc(center, 2, 64).unwrap(), None);
        assert_eq!(
            world
                .find_npc(CoordGrid::new(3200, 3200, 1).unwrap(), 1, 64)
                .unwrap(),
            None
        );
    }
}
