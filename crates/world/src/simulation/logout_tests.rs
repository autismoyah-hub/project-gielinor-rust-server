use super::*;

#[test]
fn original_pending_logout_process_matches_save_boundary() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/pending-logout.expected"
    ))
    .expect("Run scripts/pending-logout-parity.ps1");
    assert_eq!(fixture.lines().count(), 112);
    for line in fixture.lines() {
        let fields: Vec<_> = line.split('|').collect();
        let category: i32 = fields[0].parse().unwrap();
        let mut world = World::default();
        let coords = CoordGrid::new(3200, 3200, 0).unwrap();
        world
            .queue_login(
                1,
                PlayerData::new(1, "Logout".into(), coords, Stats::default()),
            )
            .unwrap();
        world.tick().unwrap();
        world.drain_events();
        let actor = world.actors().next().unwrap();
        let p = world.players.get_mut(actor.slot as usize).unwrap();
        p.protection.current_clock = 0;
        p.protection.processed_clock = 0;
        if fields[1] == "1" {
            p.protection.delay(10);
        }
        if category >= 0 {
            let kind = [
                crate::queues::Category::Soft,
                crate::queues::Category::Normal,
                crate::queues::Category::Weak,
                crate::queues::Category::Strong,
                crate::queues::Category::LongAccelerate,
                crate::queues::Category::LongDiscard,
            ][category as usize];
            p.queues
                .add(1, kind, 100, Deferred::BankCompression)
                .unwrap();
        }
        if fields[3] == "1" {
            let mut index = crate::areas::Areas::default();
            index.register_coord(coords, [7]).unwrap();
            p.areas.process(coords, coords, false, &index);
        }
        if fields[4] == "1" {
            p.pending_areas
                .push_back(crate::area_tracking::Event::Enter(99));
        }
        if fields[2] == "1" {
            world.request_shutdown(actor).unwrap();
        } else {
            world.request_logout(actor).unwrap();
        }
        let p = world.players.get(actor.slot as usize).unwrap();
        assert_eq!(p.pending_logout, fields[5] == "true", "{line}");
        assert_eq!(p.saving, fields[6] == "true", "{line}");
        assert_eq!(p.areas.active().len().to_string(), fields[8], "{line}");
        assert_eq!(!p.pending_areas.is_empty(), fields[9] == "true", "{line}");
        let saves = world
            .drain_events()
            .iter()
            .filter(|e| matches!(e, WorldEvent::Save { .. }))
            .count();
        assert_eq!(saves.to_string(), fields[7], "{line}");
    }
}
