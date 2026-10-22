use super::*;
use crate::bank_access::Request;
impl World {
    pub(super) fn is_location_bank_operation(&self, id:u32, operation:u8)->bool {
        operation==1 && self.location_banks.contains_key(&id)
            || operation==2 && self.ordinary_bank_booths.contains(&id)
    }
    pub(super) fn cinderworks_bank(&self, location:crate::locations::Location)->bool {
        location.id==crate::tower_cinderworks::BANK_BOOTH && location.shape==10 && location.angle==0
            && self.cinderworks.as_ref().is_some_and(|c|c.spatial.at(c.origin,53,60).is_ok_and(|at|at==location.coords))
    }
    pub(super) fn request_location_bank(
        &mut self,
        slot: usize,
        location: crate::locations::Location,
    ) {
        let p = self.players.get(slot).unwrap();
        let actor = p.id;
        if p.death_queued || p.data.stats.current(3) as u8 == 0 {
            let p=self.players.get_mut(slot).unwrap();p.location_interaction=None;p.movement.interacting=false;p.movement.abort();
            self.events.push(WorldEvent::ActionRejected{actor,error:ActionError::Protected});return;
        }
        let kind=if self.cinderworks_bank(location) || self.ordinary_bank_booths.contains(&location.id) {
            crate::bank_access::Kind::Full
        } else {
            let Some(gate)=self.location_banks.get(&location.id) else {return;};
            if !gate.allows(p.member, &p.data.stats, &p.data.variables) {
                let message=if gate.members && !p.member {"You need to be a member to use this bank."}
                    else {"You need 99 Crafting or the hard or elite Falador Diary reward to use this bank."};
                self.events.push(WorldEvent::LocationMessage {actor,message});return;
            }
            gate.kind
        };
        self.events.push(WorldEvent::LocationBankRequested {
            actor,
            request: Request {
                kind,
                location,
                origin: p.movement.coords,
                last_movement: p.movement.last_movement,
            },
        });
    }
    pub fn location_bank_request_valid(&self, actor: ActorId, request: Request) -> bool {
        let Ok(p) = self.player(actor) else {
            return false;
        };
        !p.death_queued && p.data.stats.current(3) as u8 != 0
            && !p.saving
            && !p.pending_logout
            && !p.protection.access_protected(p.has_modal)
            && p.movement.coords == request.origin
            && p.movement.last_movement == request.last_movement
            && self
                .locations
                .find_type(request.location.coords, request.location.id)
                == Some(request.location)
            && (self.location_banks.get(&request.location.id).is_some_and(|g| {
                    g.kind == request.kind && g.allows(p.member, &p.data.stats, &p.data.variables)
                }) || request.kind==crate::bank_access::Kind::Full && (self.cinderworks_bank(request.location) || self.ordinary_bank_booths.contains(&request.location.id)))
            && self.door_target(request.location).is_ok_and(|target| {
                target.reached(&self.collision, p.movement.coords, p.movement.size)
            })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bank_access::Gate,
        inventory::Item,
        loc_collision::{self, LocCollision},
        locations::Location,
        variables::{Varbit, Variables},
    };
    fn setup(
        base: i8,
        current: i8,
        bits: i32,
        member: bool,
        distant: bool,
    ) -> (World, ActorId, Location) {
        let mut w = World::default();
        w.object_types.stackable.insert(995);
        w.collision.allocate(3200, 3200, 0);
        let ty = LocCollision {
            width: 1,
            length: 1,
            block_walk: 2,
            block_range: true,
            break_route_finding: false,
        };
        let loc = Location::new(
            CoordGrid::new(if distant { 3202 } else { 3201 }, 3200, 0).unwrap(),
            14886,
            10,
            0,
        )
        .unwrap();
        w.location_types.insert(14886, ty);
        w.locations.load(loc);
        loc_collision::toggle(&mut w.collision, loc.coords, loc.shape, loc.angle, ty, true)
            .unwrap();
        w.location_banks.insert(
            14886,
            Gate {
                kind: crate::bank_access::Kind::Full,
                members: true,
                skill: 12,
                base_level: 99,
                any_bits: vec![
                    Varbit {
                        base: 1190,
                        lsb: 6,
                        msb: 6,
                    },
                    Varbit {
                        base: 1190,
                        lsb: 7,
                        msb: 7,
                    },
                ],
            },
        );
        let mut data = PlayerData::new(
            1,
            "Guild bank".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        );
        data.stats.set_base(12, base);
        data.stats.set_current(12, current);
        data.variables = Variables::restore([(1190, bits)].into());
        data.inventory
            .restore_slot(
                0,
                Some(Item {
                    id: 995,
                    count: 50,
                    vars: 0,
                }),
            )
            .unwrap();
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.set_membership(a, member).unwrap();
        w.drain_events();
        (w, a, loc)
    }
    #[test]
    fn guild_bank_access_reached_use_base_level_rewards_and_live_recheck() {
        for (base, current, bits, member, allowed) in [
            (99, 1, 0, true, true),
            (98, 99, 0, true, false),
            (1, 1, 64, true, true),
            (1, 1, 128, true, true),
            (99, 99, 192, false, false),
            (1, 1, 0, true, false),
        ] {
            let (mut w, a, loc) = setup(base, current, bits, member, false);
            let before = w.data(a).unwrap().clone();
            w.submit(
                a,
                Action::OperateLocation {
                    coords: loc.coords,
                    id: loc.id,
                    operation: 1,
                    control: false,
                },
            )
            .unwrap();
            w.tick().unwrap();
            let requests: Vec<_> = w
                .drain_events()
                .into_iter()
                .filter_map(|e| {
                    if let WorldEvent::LocationBankRequested { request, .. } = e {
                        Some(request)
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(requests.len(), usize::from(allowed));
            let after = w.data(a).unwrap();
            assert_eq!(after.stats, before.stats);
            assert_eq!(after.inventory, before.inventory);
            assert_eq!(after.bank, before.bank);
            assert_eq!(after.variables.get(1190), before.variables.get(1190));
            if let Some(request) = requests.first() {
                assert!(w.location_bank_request_valid(a, *request));
                w.set_membership(a, false).unwrap();
                assert!(!w.location_bank_request_valid(a, *request));
                w.set_membership(a, true).unwrap();
                w.players.get_mut(a.slot as usize).unwrap().has_modal = true;
                assert!(!w.location_bank_request_valid(a, *request));
            }
        }
    }
    #[test]
    fn location_bank_request_binds_presentation_kind_and_rejects_reconfiguration() {
        let (mut w, actor, location) = setup(99, 99, 0, true, false);
        w.location_banks.get_mut(&location.id).unwrap().kind = crate::bank_access::Kind::DepositBox;
        w.request_location_bank(actor.slot as usize, location);
        let request = w
            .drain_events()
            .into_iter()
            .find_map(|event| {
                if let WorldEvent::LocationBankRequested { request, .. } = event {
                    Some(request)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(request.kind(), crate::bank_access::Kind::DepositBox);
        assert!(w.location_bank_request_valid(actor, request));
        w.location_banks.get_mut(&location.id).unwrap().kind = crate::bank_access::Kind::Full;
        assert!(!w.location_bank_request_valid(actor, request));
    }
    #[test]
    fn guild_bank_access_walk_arrival_delay_and_disappearing_target() {
        let (mut w, a, loc) = setup(99, 99, 0, true, true);
        w.submit(
            a,
            Action::OperateLocation {
                coords: loc.coords,
                id: loc.id,
                operation: 1,
                control: false,
            },
        )
        .unwrap();
        for _ in 0..2 {
            w.tick().unwrap();
            assert!(!w
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::LocationBankRequested { .. })));
        }
        w.tick().unwrap();
        let request = w
            .drain_events()
            .into_iter()
            .find_map(|e| {
                if let WorldEvent::LocationBankRequested { request, .. } = e {
                    Some(request)
                } else {
                    None
                }
            })
            .expect("bank after walk and arrival delay");
        assert!(w.location_bank_request_valid(a, request));
        w.locations = Default::default();
        assert!(!w.location_bank_request_valid(a, request));
    }

    #[test]
    fn guild_bank_shared_request_rejects_zero_hp_unresolved_death_and_visible_hp_restoration() {
        use crate::hits::{Hit, Style, Target};
        for late in [false,true] {for phase in 0..3 {
            let(mut w,a,loc)=setup(99,99,0,true,false);
            let action=Action::OperateLocation{coords:loc.coords,id:loc.id,operation:1,control:false};
            let pending=if late {
                w.submit(a,action.clone()).unwrap();w.tick().unwrap();
                Some(w.drain_events().into_iter().find_map(|e|if let WorldEvent::LocationBankRequested{request,..}=e{Some(request)}else{None}).unwrap())
            }else{None};
            if phase==0 {w.data_mut(a).unwrap().stats.set_current(3,0);}else {
                w.queue_hit(Target::Player(a),Hit{display:None,source:None,style:Style::Typeless,damage:99},0).unwrap();w.tick().unwrap();
                assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::DeathRequested(Target::Player(id))if *id==a)));
                if phase==2 {w.data_mut(a).unwrap().stats.set_current(3,10);}
            }
            let before=w.data(a).unwrap().clone();
            if let Some(r)=pending {assert!(!w.location_bank_request_valid(a,r));}else{w.submit(a,action).unwrap();}
            for _ in 0..5 {w.tick().unwrap();assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::LocationBankRequested{..})));}
            assert_eq!(w.data(a).unwrap().inventory,before.inventory);assert_eq!(w.data(a).unwrap().bank,before.bank);assert_eq!(w.data(a).unwrap().stats,before.stats);
        }}
    }
}
