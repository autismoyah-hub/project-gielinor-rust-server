use super::*;
impl World {
    pub(super) fn process_player_soft_timer(
        &mut self,
        slot: usize,
        timers: &mut crate::player_timers::Timers,
        timer: u16,
        random: &mut impl RandomSource,
    ) -> bool {
        if [crate::wine::TIMER,crate::wine::ZAMORAK_TIMER].contains(&timer){
            if crate::wine::for_timer([self.wine,self.zamorak_wine],timer).is_some(){return self.process_wine_ferment(slot,timers,timer,random);}
            timers.remove(timer);return true;
        }
        if self
            .tower_restore
            .as_ref()
            .is_some_and(|c| c.timer == timer)
        {
            self.process_tower_ash(slot);
        }
        let Some(p) = self.players.get_mut(slot) else {
            return true;
        };

        if let Some(config) = &self.music {
            if timer == config.timer {
                let context = crate::music_player::Context {
                    logged_in: p.protection.processed_clock > 0,
                    resizable: p.music_resizable,
                };
                match config.tick(&mut p.data.variables, context, random) {
                    Ok(events) => self.events.push(WorldEvent::Music {
                        actor: p.id,
                        events,
                    }),
                    Err(failure) => {
                        self.events.push(WorldEvent::Music {
                            actor: p.id,
                            events: failure.events,
                        });
                        self.events.push(WorldEvent::MusicError {
                            actor: p.id,
                            reason: failure.reason,
                        });
                        return false;
                    }
                }
            }
        }
        if let Some(config) = &self.session_vars {
            match config.tick(timer, &mut p.data.variables, self.clock.0, self.unix_millis) {
                Ok(updates) if !updates.is_empty() => {
                    self.events.push(WorldEvent::SessionVariables {
                        actor: p.id,
                        updates,
                    })
                }
                Ok(_) => {}
                Err(_) => {
                    timers.remove(timer);
                    self.events.push(WorldEvent::ActionRejected {
                        actor: p.id,
                        error: ActionError::InvalidStat,
                    });
                }
            }
        }
        if let Some(config) = &self.prayer_book {
            if timer == config.drain_timer {
                let result = config.tick(
                    &mut p.data.variables,
                    &mut p.data.stats,
                    p.overhead,
                    timers,
                    p.protection.processed_clock > 0,
                );
                match result {
                    Ok(output) => {
                        let mut queues = std::mem::take(&mut p.queues);
                        if let Err(error) =
                            prayer_book_actions::publish(p, &mut queues, output, &mut self.events)
                        {
                            self.events
                                .push(WorldEvent::ActionRejected { actor: p.id, error });
                        }
                        p.queues = queues;
                    }
                    Err(_) => {
                        timers.remove(timer);
                        self.events.push(WorldEvent::ActionRejected {
                            actor: p.id,
                            error: ActionError::InvalidStat,
                        });
                    }
                }
            }
        }
        if let Some(config) = &self.regeneration {
            for update in config.tick(
                timer,
                &mut p.data.stats,
                p.data.equipment.get(9).map(|item| item.id),
            ) {
                if let Err(error) =
                    p.record_stat_update(self.tutorial_rules_completed_varp.is_some(), update)
                {
                    self.events
                        .push(WorldEvent::ActionRejected { actor: p.id, error });
                    return false;
                }
                self.events.push(WorldEvent::Stat {
                    actor: p.id,
                    update,
                });
            }
        }
        true
    }
}
