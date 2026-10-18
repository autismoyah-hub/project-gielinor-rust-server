use super::*;
use crate::{
    dialogue::{Conversation, Effect, Pending},
    reach::Rectangle,
    skillcape_vendor::{self as vendor, Config, Context, Kind, Stage},
};
impl World {
    fn vendor_context_valid(&self, actor: ActorId, npc: NpcId, ctx: Context) -> bool {
        let Ok(p) = self.player(actor) else {
            return false;
        };
        if p.pending_logout
            || p.saving
            || p.death_queued
            || p.data.stats.current(3) as u8 == 0
            || p.protection.delayed()
            || p.movement.coords != ctx.origin
            || p.movement.last_movement != ctx.last_movement
        {
            return false;
        }
        let Some(n) = self
            .npc(npc)
            .filter(|n| n.type_id == ctx.config.npc && n.visible && !n.delayed && n.stats[3] > 0)
        else {
            return false;
        };
        self.npc_handlers.get(&(n.type_id, 1)).copied()
            == Some(crate::npc_interaction::Handler::SkillcapeVendor(ctx.config))
            && self.can_operate_npc(actor, npc, 1)
            && (Rectangle {
                coords: n.movement.coords,
                width: n.movement.size,
                length: n.movement.size,
                angle: 0,
                blocked: 0,
                exclusive: true,
            })
            .reached(&self.collision, p.movement.coords, p.movement.size)
    }
    pub(super) fn start_skillcape_vendor(
        &mut self,
        actor: ActorId,
        npc: NpcId,
        config: Config,
    ) -> Result<(), ActionError> {
        let p = self.player(actor)?;
        let ctx = Context {
            config,
            origin: p.movement.coords,
            last_movement: p.movement.last_movement,
        };
        if !self.vendor_context_valid(actor, npc, ctx) {
            return Err(ActionError::InvalidItem);
        }
        let p = self.player(actor)?;
        let conversation = vendor::begin(
            ctx,
            &p.data.inventory,
            &p.data.equipment,
            &p.data.stats,
            p.member,
        );
        self.show_dialogue(
            actor,
            Pending {
                token: 0,
                npc,
                conversation,
            },
        )
    }
    fn vendor_purchase(
        &mut self,
        actor: ActorId,
        ctx: Context,
    ) -> Result<Conversation, ActionError> {
        let p = self.player(actor)?;
        let result = ctx.config.prepare_purchase(
            &p.data.inventory,
            &p.data.equipment,
            &self.object_types,
            &p.data.stats,
            p.member,
        );
        Ok(match result {
            Err(e) => vendor::message(e.message()),
            Ok(plan) => {
                self.players
                    .get_mut(actor.slot as usize)
                    .unwrap()
                    .data
                    .inventory = plan.inventory;
                vendor::message("Here are your cape and matching hood. Wear them with pride.")
            }
        })
    }
    pub(super) fn skillcape_vendor_choice(
        &mut self,
        actor: ActorId,
        npc: NpcId,
        ctx: Context,
        stage: Stage,
        choice: i32,
    ) -> Result<Option<Conversation>, ActionError> {
        if !self.vendor_context_valid(actor, npc, ctx) {
            return Ok(Some(vendor::closed()));
        }
        let owns = {
            let p = self.player(actor)?;
            ctx.config.owns_cape(&p.data.inventory, &p.data.equipment)
        };
        Ok(match(stage,choice){
   (Stage::About,1)=>Some(vendor::message("Train this skill to a base level of 99. A cape with a matching hood costs 99,000 coins.")),
   (Stage::About,2)|(Stage::Offer,1)|(Stage::Quote,1)=>Some(vendor::closed()),
   (Stage::Offer,2)=>Some(if owns{vendor::owned(ctx)}else{vendor::quote(ctx)}),
   (Stage::Owned,1)=>Some(if ctx.config.kind==Kind::Crafting{vendor::message(vendor::Error::OwnCape.message())}else{vendor::quote(ctx)}),
   (Stage::Owned,2)=>Some(vendor::hood_question(ctx)),
   (Stage::Quote,2)=>Some(if ctx.config.kind==Kind::Cooking{vendor::purchase_ack(ctx)}else{self.vendor_purchase(actor,ctx)?}),
   _=>None,
  })
    }
    pub(super) fn finish_skillcape_vendor(
        &mut self,
        actor: ActorId,
        npc: NpcId,
        ctx: Context,
        stage: Stage,
    ) -> Result<Conversation, ActionError> {
        if !self.vendor_context_valid(actor, npc, ctx) {
            return Ok(vendor::closed());
        }
        if stage == Stage::Purchase {
            return self.vendor_purchase(actor, ctx);
        }
        if matches!(stage, Stage::HoodOffer | Stage::Hood) {
            let p = self.player(actor)?;
            let result = ctx.config.prepare_hood(
                &p.data.inventory,
                &p.data.equipment,
                &self.object_types,
                &p.data.stats,
                p.member,
            );
            return Ok(match result {
                Err(e) => vendor::message(e.message()),
                Ok(_) if stage == Stage::HoodOffer => vendor::hood_ack(ctx),
                Ok(plan) => {
                    self.players
                        .get_mut(actor.slot as usize)
                        .unwrap()
                        .data
                        .inventory = plan.inventory;
                    vendor::message("Here is your matching hood.")
                }
            });
        }
        Ok(vendor::closed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        dialogue::Page,
        inventory::Item,
        npc::{BlockWalk, Mode, Restrict},
        npc_options::Definition,
    };
    struct NoDraw;
    impl RandomSource for NoDraw {
        fn below(&mut self, _: u32) -> u32 {
            panic!("Vendor dialogue must not draw RNG");
        }
    }
    fn fixture(kind: Kind, trim: bool) -> (World, ActorId, NpcId, Config) {
        let mut w = World::default();
        for x in (3192..3232).step_by(8) {
            for z in (3192..3232).step_by(8) {
                w.collision.allocate(x, z, 0);
            }
        }
        let (skill, npc, capes, hood) = if kind == Kind::Crafting {
            (12, 5810, [9780, 9781], 9782)
        } else {
            (7, 2658, [9801, 9802], 9803)
        };
        let cfg = Config {
            kind,
            npc,
            skill,
            coins: 995,
            capes,
            hood,
            active_stats: (1 << skill) | 1,
        };
        w.object_types.stackable.insert(995);
        w.npc_handlers.insert(
            (npc, 1),
            crate::npc_interaction::Handler::SkillcapeVendor(cfg),
        );
        w.npc_options.types.insert(
            npc,
            Definition {
                operations: [Some("Talk-to".into()), None, None, None, None],
                ..Default::default()
            },
        );
        let mut stats = Stats::default();
        stats.set_base(skill, 99);
        stats.set_current(skill, 99);
        stats
            .set_xp(skill, crate::skills::xp_from_level(99).unwrap())
            .unwrap();
        if trim {
            stats.set_base(0, 99);
            stats.set_current(0, 99);
            stats
                .set_xp(0, crate::skills::xp_from_level(99).unwrap())
                .unwrap();
        }
        let mut data = PlayerData::new(
            1,
            "Cape shopper".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            stats,
        );
        data.inventory
            .restore_slot(
                0,
                Some(Item {
                    id: 995,
                    count: 100_000,
                    vars: 0,
                }),
            )
            .unwrap();
        w.queue_login(1, data).unwrap();
        w.tick_with_random(&mut NoDraw).unwrap();
        let a = w.actors().next().unwrap();
        w.set_membership(a, true).unwrap();
        let mut n = Npc::new(
            npc,
            CoordGrid::new(3201, 3200, 0).unwrap(),
            1,
            Restrict::Normal,
            BlockWalk::None,
        )
        .unwrap();
        n.mode = Mode::None;
        let id = w.spawn_npc(n).unwrap();
        w.drain_events();
        (w, a, id, cfg)
    }
    fn talk(w: &mut World, a: ActorId, n: NpcId) {
        w.submit(
            a,
            Action::OperateNpc {
                target: n,
                operation: 1,
                control: false,
            },
        )
        .unwrap();
        w.tick_with_random(&mut NoDraw).unwrap();
        assert!(w.players.get(a.slot as usize).unwrap().dialogue.is_some());
    }
    fn page(w: &World, a: ActorId) -> (u64, Stage, bool) {
        let p = w
            .players
            .get(a.slot as usize)
            .unwrap()
            .dialogue
            .as_ref()
            .unwrap();
        let Effect::SkillcapeVendor { stage, .. } = p.conversation.after else {
            panic!("Not vendor stage")
        };
        (
            p.token,
            stage,
            matches!(
                p.conversation.pages.front(),
                Some(Page::Choice(_) | Page::TitledChoice { .. })
            ),
        )
    }
    fn reply(w: &mut World, a: ActorId, token: u64, choice: i32) {
        w.submit(a, Action::ResumeDialogue { token, choice })
            .unwrap();
        w.tick_with_random(&mut NoDraw).unwrap();
    }
    fn quote(w: &mut World, a: ActorId) -> u64 {
        for _ in 0..12 {
            let (t, s, c) = page(w, a);
            if s == Stage::Quote && c {
                return t;
            }
            let choice = if c {
                if s == Stage::Owned {
                    1
                } else {
                    2
                }
            } else {
                -1
            };
            reply(w, a, t, choice);
        }
        panic!("No quote");
    }
    fn units(inv: &Inventory, id: i32) -> i32 {
        inv.slots()
            .iter()
            .flatten()
            .filter(|i| i.id == id)
            .map(|i| i.count)
            .sum()
    }
    #[test]
    fn vendor_dialogue_actual_two_npcs_paid_trim_and_untrimmed_confirmation_and_no_replay() {
        for kind in [Kind::Crafting, Kind::Cooking] {
            for trim in [false, true] {
                let (mut w, a, n, c) = fixture(kind, trim);
                let before = w.data(a).unwrap().clone();
                talk(&mut w, a, n);
                let token = quote(&mut w, a);
                reply(&mut w, a, token, 2);
                if kind == Kind::Cooking {
                    assert_eq!(w.data(a).unwrap().inventory, before.inventory);
                    let (t, s, choice) = page(&w, a);
                    assert_eq!(s, Stage::Purchase);
                    assert!(!choice);
                    reply(&mut w, a, t, -1);
                }
                let data = w.data(a).unwrap();
                assert_eq!(units(&data.inventory, 995), 1000);
                assert_eq!(units(&data.inventory, c.capes[usize::from(trim)]), 1);
                assert_eq!(units(&data.inventory, c.hood), 1);
                assert_eq!(data.stats, before.stats);
                assert_eq!(data.equipment, before.equipment);
                let paid = data.inventory.clone();
                reply(&mut w, a, token, 2);
                assert_eq!(w.data(a).unwrap().inventory, paid);
            }
        }
    }
    #[test]
    fn vendor_dialogue_late_eligibility_currency_space_catalog_movement_and_close_never_charge() {
        for case in 0..8 {
            let (mut w, a, n, c) = fixture(Kind::Crafting, false);
            talk(&mut w, a, n);
            let token = quote(&mut w, a);
            match case {
                0 => w.set_membership(a, false).unwrap(),
                1 => w.data_mut(a).unwrap().stats.set_base(c.skill, 98),
                2 => {
                    w.data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(
                            0,
                            Some(Item {
                                id: 995,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                3 => {
                    for slot in 1..27 {
                        w.data_mut(a)
                            .unwrap()
                            .inventory
                            .restore_slot(
                                slot,
                                Some(Item {
                                    id: 590,
                                    count: 1,
                                    vars: 0,
                                }),
                            )
                            .unwrap();
                    }
                }
                4 => {
                    w.object_types.dummy.insert(c.hood);
                }
                5 => {
                    w.players
                        .get_mut(a.slot as usize)
                        .unwrap()
                        .movement
                        .last_movement += 1
                }
                6 => {
                    w.cancel_dialogue(a).unwrap();
                }
                7 => {
                    w.npc_mut(n).unwrap().visible = false;
                }
                _ => unreachable!(),
            }
            let before = w.data(a).unwrap().clone();
            reply(&mut w, a, token, 2);
            assert_eq!(w.data(a).unwrap().inventory, before.inventory, "case{case}");
            assert_eq!(w.data(a).unwrap().stats, before.stats);
        }
        let (mut w, a, n, _) = fixture(Kind::Cooking, false);
        talk(&mut w, a, n);
        let token = quote(&mut w, a);
        let before = w.data(a).unwrap().clone();
        reply(&mut w, a, token, 2);
        assert_eq!(page(&w, a).1, Stage::Purchase);
        let ack = page(&w, a).0;
        w.cancel_dialogue(a).unwrap();
        reply(&mut w, a, ack, -1);
        assert_eq!(w.data(a).unwrap().inventory, before.inventory);
    }
    #[test]
    fn vendor_dialogue_hood_is_only_credited_after_both_acknowledgements_and_live_cape() {
        for kind in [Kind::Crafting, Kind::Cooking] {
            let (mut w, a, n, c) = fixture(kind, false);
            w.data_mut(a)
                .unwrap()
                .equipment
                .restore_slot(
                    1,
                    Some(Item {
                        id: c.capes[0],
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            let before = w.data(a).unwrap().clone();
            talk(&mut w, a, n);
            if kind == Kind::Cooking {
                let t = page(&w, a).0;
                reply(&mut w, a, t, -1);
                let t = page(&w, a).0;
                reply(&mut w, a, t, 2);
            }
            for _ in 0..4 {
                let (t, s, is_choice) = page(&w, a);
                if s == Stage::Owned && is_choice {
                    reply(&mut w, a, t, 2);
                    break;
                }
                reply(&mut w, a, t, -1);
            }
            let (t, s, _) = page(&w, a);
            assert_eq!(s, Stage::HoodOffer);
            assert_eq!(w.data(a).unwrap().inventory, before.inventory);
            reply(&mut w, a, t, -1);
            let (t, s, _) = page(&w, a);
            assert_eq!(s, Stage::Hood);
            assert_eq!(w.data(a).unwrap().inventory, before.inventory);
            reply(&mut w, a, t, -1);
            assert_eq!(units(&w.data(a).unwrap().inventory, c.hood), 1);
            assert_eq!(units(&w.data(a).unwrap().inventory, 995), 100_000);
            assert_eq!(w.data(a).unwrap().stats, before.stats);
        }
    }
}
