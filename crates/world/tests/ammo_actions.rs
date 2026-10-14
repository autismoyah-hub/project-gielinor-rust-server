use gielinor_world::{
    inventory::Item,
    npc::RandomSource,
    ranged_ammo::Shot,
    simulation::{PlayerData, World, WorldEvent},
    stats::Stats,
    CoordGrid,
};
struct Replay(std::collections::VecDeque<(u32, u32)>);
impl RandomSource for Replay {
    fn below(&mut self, bound: u32) -> u32 {
        let (b, v) = self.0.pop_front().expect("Unexpected draw");
        assert_eq!(b, bound);
        v
    }
}
#[test]
fn consumed_ammunition_captures_launch_collision_owner_and_private_drop_delay() {
    let pos = CoordGrid::new(3200, 3200, 0).unwrap();
    for slot in [3, 13] {
        for flag in [0, 0x100, 0x200000] {
            let mut world = World::default();
            world.collision.set(3200, 3200, 0, flag);
            world.object_types.stackable.insert(100);
            world
                .queue_login(
                    1,
                    PlayerData::new(1, "Archer".into(), pos, Stats::default()),
                )
                .unwrap();
            world.tick().unwrap();
            let actor = world.actors().next().unwrap();
            world.set_observer_id(actor, 77).unwrap();
            world
                .data_mut(actor)
                .unwrap()
                .equipment
                .restore_slot(
                    slot,
                    Some(Item {
                        id: 100,
                        count: 1,
                        vars: 11,
                    }),
                )
                .unwrap();
            world.drain_events();
            let mut random = Replay([(5, 0)].into());
            let shot = Shot {
                slot,
                id: 100,
                recovery_rate: None,
                metallic_body: false,
                drop_coord: pos,
                drop_delay: 2,
            };
            let result = world.use_ranged_ammo(actor, shot, &mut random).unwrap();
            assert!(random.0.is_empty());
            assert!(world.data(actor).unwrap().equipment.get(slot).is_none());
            assert_eq!(result.detracted.unwrap().rebuild_appearance, slot == 3);
            assert_eq!(result.drop, flag != 0x200000);
            assert!(world
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::AmmoConsumed { .. })));
            // Neither a later collision change nor an ownership change rewrites the launch.
            world
                .collision
                .set(3200, 3200, 0, if flag == 0x200000 { 0 } else { 0x200000 });
            world.set_observer_id(actor, 88).unwrap();
            for _ in 0..2 {
                world.tick().unwrap();
                assert_eq!(world.objects.at(pos).count(), 0);
            }
            world.tick().unwrap();
            let drops: Vec<_> = world.objects.at(pos).map(|h| h.snapshot()).collect();
            assert_eq!(drops.len(), usize::from(flag != 0x200000));
            if let Some(drop) = drops.first() {
                assert_eq!(
                    (drop.item, drop.count, drop.receiver, drop.owner),
                    (100, 1, Some(77), Some(77))
                );
                assert!(drop.visible_to(Some(77)));
                assert!(!drop.visible_to(Some(88)));
            }
        }
    }
}
