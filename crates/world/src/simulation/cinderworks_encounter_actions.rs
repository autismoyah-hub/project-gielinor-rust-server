use super::*;
use crate::{
    cinderworks_encounter::{Boss, Encounter},
    hits::{Hit, Style, Target},
    zones::Graphic,
};
impl World {
    pub(super) fn cinderworks_boss(&self, target: NpcId) -> Option<Boss> {
        let config = self.cinderworks.as_ref()?;
        let npc = self.npc(target)?;
        if !crate::tower_layout::contains(config.origin, npc.spawn) {
            return None;
        }
        let kind = npc.type_id;
        if kind == config.imp {
            Some(Boss::Imp)
        } else if kind == config.warden {
            Some(Boss::Warden)
        } else {
            None
        }
    }
    pub(super) fn cinderworks_denial(&self, actor: ActorId, target: NpcId) -> Option<&'static str> {
        let boss = self.cinderworks_boss(target)?;
        let config = self.cinderworks.as_ref()?;
        let player = self.player(actor).ok()?;
        let run = player.data.tower_run;
        if run.id > 0 && !run.active() {
            return Some(run.guide());
        }
        if !run.active() || run.services != 7 {
            return Some(
                "Restore the workshop, kitchen and altar before challenging the foundry guardians.",
            );
        }
        if boss == Boss::Imp && run.imp {
            return Some("The Imp is already silenced for this run. Enter the northern foundry.");
        }
        if boss == Boss::Warden && !run.imp {
            return Some(
                "Silence the Ash Imp in the eastern vent gallery before challenging the Warden.",
            );
        }
        if !boss.contains(config.origin, player.movement.coords) {
            return Some("Enter the guardian's court before attacking.");
        }
        if !player
            .data
            .equipment
            .get(3)
            .is_some_and(|item| config.forged_weapon(item.id))
        {
            return Some(
                "Wield your forged bronze dagger or bronze scimitar to challenge this guardian.",
            );
        }
        if let Some(encounter) = &self.cinderworks_encounters[boss.index()] {
            if encounter.owner != actor {
                return Some(
                    "Another restorer is using this court. Wait for their encounter to finish.",
                );
            }
            if boss == Boss::Warden
                && encounter.warning.is_some()
                && self.clock.0 >= encounter.exposed_until
            {
                return Some(crate::tower_cinderworks::SHELL_HINT);
            }
        }
        None
    }
    pub(super) fn begin_cinderworks_encounter(&mut self, actor: ActorId, target: NpcId) {
        let Some(boss) = self.cinderworks_boss(target) else {
            return;
        };
        if self.cinderworks_denial(actor, target).is_some()
            || self.cinderworks_encounters[boss.index()].is_some()
        {
            return;
        }
        let run_id = self.player(actor).unwrap().data.tower_run.id;
        self.cinderworks_encounters[boss.index()] = Some(Encounter {
            owner: actor,
            run_id,
            npc: target,
            boss,
            phase: 1,
            next: self.clock.deadline(4),
            warning: None,
            tiles: vec![],
            cycle: 0,
            exposed_until: 0,
            quench_ready: 0,
        });
        self.events.push(WorldEvent::GameMessage {actor,text:match boss {
            Boss::Imp=>"The Ash Imp wakes. Smoke marks where heat will erupt: step clear before it ignites.",
            Boss::Warden=>"The Ash Warden wakes. Smoke closes its shell to attacks. Evade first; search a water barrel from safety to expose it.",
        }.into()});
    }
    fn reset_cinderworks_guardian(&mut self, target: NpcId) {
        let Some(entry) = self
            .npcs
            .get_mut(target.slot as usize)
            .filter(|entry| entry.id == target && entry.npc.stats[3] > 0)
        else {
            return;
        };
        let npc = &mut entry.npc;
        entry.hits = Default::default();
        npc.stats = npc.base_stats;
        npc.hero_points.clear();
        npc.attacking_player = None;
        npc.aggressive_player = None;
        npc.retaliate_due.clear();
        npc.movement.abort();
        let _ = npc
            .movement
            .teleport(&mut self.collision, npc.spawn, self.clock.0, true);
    }
    pub(super) fn process_cinderworks_encounters(&mut self) {
        let Some(config) = self.cinderworks.clone() else {
            return;
        };
        for index in 0..2 {
            let Some(mut encounter) = self.cinderworks_encounters[index].take() else {
                continue;
            };
            let valid = self.player(encounter.owner).is_ok_and(|p| {
                !p.saving
                    && !p.pending_logout
                    && p.data.stats.current(3) as u8 > 0
                    && p.data.tower_run.id == encounter.run_id
                    && encounter.boss.contains(config.origin, p.movement.coords)
            });
            if !valid {
                self.reset_cinderworks_guardian(encounter.npc);
                continue;
            }
            let receiver=self.player(encounter.owner).unwrap().data.character_id as i64;
            let Some(npc) = self.npc(encounter.npc) else {
                continue;
            };
            if npc.stats[3] <= 0 {
                let winner = npc.hero_points.winner();
                let p = self.players.get_mut(encounter.owner.slot as usize).unwrap();
                let mut proposed_run = p.data.tower_run;
                if winner == Some(p.data.character_id) {
                    let credited = match encounter.boss {
                        Boss::Imp => proposed_run.defeat_imp(),
                        Boss::Warden => proposed_run.defeat_warden(),
                    };
                    if credited {
                        let message = match encounter.boss {
                        Boss::Imp=>"The Ash Imp falls. Its boiler falls silent. The northern foundry is unlocked; your services and unlock are preserved if you leave.",
                        Boss::Warden=>"The Warden is quenched. The Cinderworks sanctuary is restored. Return to its chest to claim your reward.",
                    }.into();
                        let actor = p.id;
                        let inventory = p.data.inventory.clone();
                        let run = proposed_run;
                        self.checkpoint_cinderworks(actor, inventory, run, message);
                    }
                }
                continue;
            }
            let phase = encounter.boss.phase(npc.stats[3], npc.base_stats[3].max(1));
            if phase > encounter.phase {
                encounter.phase = phase;
                self.events.push(WorldEvent::NpcSpotAnimation {
                    actor: encounter.npc,
                    spot: config.strike_graphic,
                    delay: 0,
                    height: 0,
                    slot: 0,
                });
                self.events.push(WorldEvent::GameMessage {actor:encounter.owner,text:match (encounter.boss,phase) {
                    (Boss::Imp,_)=>"The Imp scatters its heat across three tiles. Leave the smoking row.",
                    (Boss::Warden,2)=>"The Warden enters phase two. Crossed smoke lanes converge; move diagonally to safety.",
                    _=>"The Warden enters its final phase. Diagonal vents alternate faster; find the gaps and quench its charge.",
                }.into()});
            }
            if encounter.warning.is_some_and(|at| self.clock.0 >= at) {
                let coords = self.player(encounter.owner).unwrap().movement.coords;
                if encounter.tiles.contains(&coords) {
                    let damage = if encounter.boss == Boss::Imp { 3 } else { 4 };
                    if let Err(_) = self.queue_hit(
                        Target::Player(encounter.owner),
                        Hit {
                            display: None,
                            source: Some(Target::Npc(encounter.npc)),
                            style: Style::Typeless,
                            damage,
                        },
                        0,
                    ) {
                        self.events.push(WorldEvent::GameMessage {actor:encounter.owner,text:"The vent attack could not be scheduled; the encounter resets safely.".into()});
                        self.reset_cinderworks_guardian(encounter.npc);
                        continue;
                    }
                }
                for &coords in &encounter.tiles {
                    self.events.push(WorldEvent::TileGraphic(Graphic {
                        coords,
                        id: config.strike_graphic,
                        receiver:Some(receiver),
                        height: 0,
                        delay: 0,
                    }));
                }
                encounter.warning = None;
                encounter.tiles.clear();
                encounter.exposed_until = self.clock.deadline(4);
                encounter.next = self
                    .clock
                    .deadline(if encounter.phase >= 3 { 3 } else { 5 });
            }
            if encounter.warning.is_none() && self.clock.0 >= encounter.next {
                let aim = self.player(encounter.owner).unwrap().movement.coords;
                encounter.tiles =
                    encounter
                        .boss
                        .pattern(config.origin, encounter.phase, aim, encounter.cycle);
                encounter.cycle = encounter.cycle.wrapping_add(1);
                // Keep the visual reaction window readable in the native client.
                // The final phase accelerates recovery/cadence, not input latency.
                encounter.warning = Some(self.clock.deadline(4));
                self.events.push(WorldEvent::NpcCombatAnimation {
                    actor: encounter.npc,
                    sequence: config.charge_animations[encounter.boss.index()],
                });
                if encounter.cycle == 1 {
                    self.events.push(WorldEvent::GameMessage {actor:encounter.owner,text:"Heat gathers beneath the smoking tiles. Move clear before the eruption!".into()});
                }
            }
            if encounter.warning.is_some() {
                for &coords in &encounter.tiles {
                    self.events.push(WorldEvent::TileGraphic(Graphic {
                        coords,
                        id: config.warning_graphic,
                        receiver:Some(receiver),
                        height: 0,
                        delay: 0,
                    }));
                }
            }
            self.cinderworks_encounters[index] = Some(encounter);
        }
    }
    pub(super) fn quench_cinderworks(
        &mut self,
        slot: usize,
        loc: crate::locations::Location,
    ) -> bool {
        let Some(config) = &self.cinderworks else {
            return false;
        };
        let (x, z) = crate::cinderworks_scenery::TRAINING_COOLANT;
        if loc.id == config.quench
            && config.spatial.at(config.origin, x, z).ok() == Some(loc.coords)
        {
            let actor = self.players.get(slot).unwrap().id;
            let receiver=self.players.get(slot).unwrap().data.character_id as i64;
            self.events.push(WorldEvent::TileGraphic(Graphic {
                coords: loc.coords,
                id: config.warning_graphic,
                receiver:Some(receiver),
                height: 0,
                delay: 0,
            }));
            self.events.push(WorldEvent::GameMessage {actor,text:"Cold water draws the heat into harmless steam. In the foundry, stand beside a water barrel and Search as the Warden charges. Coolant interrupts its warning and exposes its shell; you can also evade and strike during recovery.".into()});
            return true;
        }
        if loc.id != config.quench || !config.quench_tiles.contains(&loc.coords) {
            return false;
        }
        let actor = self.players.get(slot).unwrap().id;
        let Some(encounter) = self.cinderworks_encounters[Boss::Warden.index()]
            .as_mut()
            .filter(|e| e.owner == actor)
        else {
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: "The coolant barrels interrupt the Warden's charge during your encounter."
                    .into(),
            });
            return true;
        };
        if encounter.warning.is_none() || self.clock.0 < encounter.quench_ready {
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: "Wait for the Warden to charge before releasing more coolant.".into(),
            });
            return true;
        }
        encounter.warning = None;
        encounter.tiles.clear();
        encounter.exposed_until = self.clock.deadline(8);
        encounter.next = self.clock.deadline(10);
        encounter.quench_ready = self.clock.deadline(10);
        self.events.push(WorldEvent::NpcCombatAnimation {
            actor: encounter.npc,
            sequence: config.quench_animation,
        });
        self.events.push(WorldEvent::NpcSpotAnimation {
            actor: encounter.npc,
            spot: config.warning_graphic,
            delay: 0,
            height: 32,
            slot: 0,
        });
        self.events.push(WorldEvent::GameMessage {actor,text:"Coolant quenches the charge! The smoke clears and the Warden's shell is exposed. Attack now.".into()});
        true
    }
}
#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::{
        inventory::Item,
        loc_collision::LocCollision,
        locations::Location,
        npc::{BlockWalk, Mode, Npc, Restrict},
        tower_cinderworks::Config,
        tower_run::Service,
    };
    pub(in crate::simulation) fn setup(boss: Boss) -> (World, ActorId, NpcId) {
        let mut world = World::default();
        let origin = CoordGrid::new(10368, 64, 0).unwrap();
        for x in (0..128).step_by(8) {
            for z in (0..128).step_by(8) {
                world.collision.allocate(origin.x() + x, origin.z() + z, 0);
            }
        }
        let quench = crate::cinderworks_floor::Spatial::default()
            .at(origin, 53, 71)
            .unwrap();
        world.cinderworks = Some(Config {
            origin,
            spatial: crate::cinderworks_floor::Spatial::default(),
            outside: CoordGrid::new(3225, 3218, 0).unwrap(),
            plaque: 1,
            portal: 2,
            chest: 3,
            quench: 4,
            tools: vec![],
            scimitar: 12,
            dagger: 13,
            coins: 100,
            beer: 101,
            imp: 200,
            warden: 201,
            quench_tiles: vec![quench],
            warning_graphic: 86,
            strike_graphic: 78,
            work: None,
            death_animation: 836,
            charge_animations: [169, 4666],
            quench_animation: 4664,
        });
        let loc = Location::new(quench, 4, 10, 0).unwrap();
        world.locations.load(loc);
        world.location_types.insert(
            4,
            LocCollision {
                width: 1,
                length: 1,
                block_walk: 0,
                block_range: false,
                break_route_finding: false,
            },
        );
        let position = origin
            .translate(
                if boss == Boss::Imp { 67 } else { 79 },
                if boss == Boss::Imp { 70 } else { 55 },
                0,
            )
            .unwrap();
        let mut data = PlayerData::new(1, "Restorer".into(), position, Stats::default());
        data.stats.set_base(3, 20);
        data.stats.set_current(3, 20);
        data.equipment
            .restore_slot(
                3,
                Some(Item {
                    id: 13,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        data.tower_run.begin();
        for service in [Service::Forge, Service::Kitchen, Service::Altar] {
            data.tower_run.service(service);
        }
        if boss == Boss::Warden {
            data.tower_run.defeat_imp();
        }
        world.queue_login(1, data).unwrap();
        world.tick().unwrap();
        let actor = world.actors().next().unwrap();
        let mut npc = Npc::new(
            if boss == Boss::Imp { 200 } else { 201 },
            origin
                .translate(
                    if boss == Boss::Imp { 68 } else { 81 },
                    if boss == Boss::Imp { 71 } else { 56 },
                    0,
                )
                .unwrap(),
            1,
            Restrict::Normal,
            BlockWalk::None,
        )
        .unwrap();
        npc.stats[3] = if boss == Boss::Imp { 18 } else { 42 };
        npc.base_stats = npc.stats;
        npc.mode = Mode::None;
        npc.regen_rate = 0;
        let target = world.spawn_npc(npc).unwrap();
        world.begin_cinderworks_encounter(actor, target);
        (world, actor, target)
    }
    fn warning(world: &mut World, boss: Boss) -> i32 {
        for _ in 0..6 {
            world.tick().unwrap();
            if let Some(at) = world.cinderworks_encounters[boss.index()]
                .as_ref()
                .and_then(|encounter| encounter.warning)
            {
                return at;
            }
        }
        panic!("encounter did not warn");
    }
    #[test]
    fn locked_guardian_attack_reports_once_then_releases_the_player() {
        let (mut world, actor, target) = setup(Boss::Imp);
        world.cinderworks_encounters = [None, None];
        world.data_mut(actor).unwrap().tower_run.services = 0;
        let before = world.data(actor).unwrap().clone();
        let mut options = crate::npc_options::Definition::default();
        options.operations[1] = Some("Attack".into());
        world.npc_options.types.insert(200, options);
        world
            .npc_handlers
            .insert((200, 2), crate::npc_interaction::Handler::Combat);
        world.drain_events();
        world.request_npc(actor, target, 2, false).unwrap();
        for _ in 0..8 {
            world.tick().unwrap();
        }
        let messages = world
            .drain_events()
            .into_iter()
            .filter(|event| {
                matches!(event,
            WorldEvent::NpcInteraction { event: crate::npc_interaction::Event::Message(text), .. }
                if text.contains("Restore the workshop, kitchen and altar"))
            })
            .count();
        assert_eq!(messages, 1, "Rejected attacks must not repeat each tick");
        assert!(world.player(actor).unwrap().npc_interaction.is_none());
        assert!(world.cinderworks_encounters[0].is_none());
        assert_eq!(world.data(actor).unwrap().stats, before.stats);
        assert_eq!(world.data(actor).unwrap().tower_run, before.tower_run);
        assert_eq!(world.npc(target).unwrap().stats[3], 18);
        let destination = before.coords.translate(0, -2, 0).unwrap();
        let input = world
            .prepare_movement_input(actor, destination, false)
            .unwrap()
            .unwrap();
        world.finish_movement_input(input).unwrap();
        for _ in 0..3 {
            world.tick().unwrap();
        }
        assert_eq!(world.player(actor).unwrap().movement.coords, destination);
    }
    #[test]
    fn warned_vent_damages_only_on_expiry_and_movement_avoids_it() {
        for escape in [false, true] {
            let (mut world, actor, _) = setup(Boss::Imp);
            let at = warning(&mut world, Boss::Imp);
            assert_eq!(world.data(actor).unwrap().stats.current(3), 20);
            assert!(world
                .drain_events()
                .iter()
                .any(|event| matches!(event,WorldEvent::TileGraphic(graphic) if graphic.id==86)));
            if escape {
                let destination = world
                    .observe(actor)
                    .unwrap()
                    .coords
                    .translate(0, 1, 0)
                    .unwrap();
                let input = world
                    .prepare_movement_input(actor, destination, false)
                    .unwrap()
                    .unwrap();
                world.finish_movement_input(input).unwrap();
            }
            while world.clock.0 < at {
                world.tick().unwrap();
                assert_eq!(world.data(actor).unwrap().stats.current(3), 20);
            }
            world.tick().unwrap();
            world.tick().unwrap();
            assert_eq!(
                world.data(actor).unwrap().stats.current(3),
                if escape { 20 } else { 17 }
            );
        }
    }
    #[test]
    fn court_ownership_denies_others_and_logout_resets_only_transient_state() {
        let (mut world, actor, target) = setup(Boss::Imp);
        let mut other = world.data(actor).unwrap().clone();
        other.account_id = 2;
        other.character_id = 2;
        world.queue_login(2, other).unwrap();
        world.tick().unwrap();
        let second = world.actors().find(|id| *id != actor).unwrap();
        assert!(world
            .cinderworks_denial(second, target)
            .unwrap()
            .contains("Another restorer"));
        let run = world.data(actor).unwrap().tower_run;
        world.npc_mut(target).unwrap().stats[3] = 5;
        world.request_logout(actor).unwrap();
        world.tick().unwrap();
        assert!(world.cinderworks_encounters[0].is_none());
        assert_eq!(world.npc(target).unwrap().stats[3], 18);
        assert_eq!(world.data(actor).unwrap().tower_run, run);
    }
    #[test]
    fn routed_coolant_interaction_interrupts_warning_and_prevents_damage() {
        let (mut world, actor, _) = setup(Boss::Warden);
        let at = warning(&mut world, Boss::Warden);
        let coords = world.cinderworks.as_ref().unwrap().quench_tiles[0];
        world
            .submit(
                actor,
                Action::OperateLocation {
                    coords,
                    id: 4,
                    operation: 1,
                    control: false,
                },
            )
            .unwrap();
        world.tick().unwrap();
        assert!(world.cinderworks_encounters[1]
            .as_ref()
            .unwrap()
            .warning
            .is_none());
        while world.clock.0 <= at + 1 {
            world.tick().unwrap();
        }
        assert_eq!(world.data(actor).unwrap().stats.current(3), 20);
        assert!(
            world.cinderworks_encounters[1]
                .as_ref()
                .unwrap()
                .exposed_until
                > world.clock.0
        );
    }
    #[test]
    fn authoritative_npc_death_credits_owner_once_and_leaves_reward_pending() {
        for boss in [Boss::Imp, Boss::Warden] {
            let (mut world, actor, target) = setup(boss);
            let uuid = world.data(actor).unwrap().character_id;
            world.npc_mut(target).unwrap().hero_points.credit(uuid, 50);
            world
                .queue_hit(
                    Target::Npc(target),
                    Hit {
                        source: Some(Target::Player(actor)),
                        style: Style::Melee,
                        damage: 50,
                        display: None,
                    },
                    0,
                )
                .unwrap();
            let before = world.data(actor).unwrap().tower_run;
            for _ in 0..4 {
                world.tick().unwrap();
                if world.cinderworks_checkpoint_pending(actor) {
                    break;
                }
            }
            assert!(world.cinderworks_checkpoint_pending(actor));
            assert_eq!(world.data(actor).unwrap().tower_run, before);
            world.cinderworks_checkpoint_saved(actor).unwrap();
            let run = world.data(actor).unwrap().tower_run;
            assert!(run.imp);
            assert_eq!(run.reward_pending, boss == Boss::Warden);
            assert_eq!(run.clears, if boss == Boss::Warden { 1 } else { 0 });
            assert!(world.cinderworks_encounters[boss.index()].is_none());
            world.tick().unwrap();
            assert_eq!(world.data(actor).unwrap().tower_run, run);
        }
    }
    #[test]
    fn waterworks_lesson_never_creates_an_encounter_damage_or_progress() {
        let (mut world, actor, _) = setup(Boss::Imp);
        world.cinderworks_encounters = [None, None];
        let config = world.cinderworks.as_ref().unwrap();
        let (x, z) = crate::cinderworks_scenery::TRAINING_COOLANT;
        let location = Location::new(
            config.spatial.at(config.origin, x, z).unwrap(),
            config.quench,
            10,
            0,
        )
        .unwrap();
        let before = world.data(actor).unwrap().tower_run;
        let hp = world.data(actor).unwrap().stats.current(3);
        world.events.clear();
        assert!(world.quench_cinderworks(actor.slot as usize, location));
        assert_eq!(world.data(actor).unwrap().tower_run, before);
        assert_eq!(world.data(actor).unwrap().stats.current(3), hp);
        assert!(world.cinderworks_encounters.iter().all(Option::is_none));
        assert!(world.events.iter().any(|e|matches!(e,WorldEvent::GameMessage {text,..} if text.contains("Cold water draws the heat"))));
        assert!(world
            .events
            .iter()
            .any(|e| matches!(e, WorldEvent::TileGraphic(_))));
    }
}
