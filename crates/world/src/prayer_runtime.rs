//! Prayer-tab effects coordinated with the player's soft timers. Strong toggle
//! queues and the 25-cycle preserve activation queue remain owned by World.
use crate::{
    player_regen,
    player_timers::Timers,
    prayer_book::{self, Event, Regen},
    prayer_drain, stat_ops,
    stats::Stats,
    variables::Variables,
};
#[derive(Clone, Debug)]
pub struct Config {
    pub book: prayer_book::Config,
    pub drain: prayer_drain::Config,
    pub regeneration: player_regen::Config,
    pub drain_timer: u16,
    pub depletion_sound: u16,
}
#[derive(Debug, Default)]
pub struct Output {
    pub events: Vec<Event>,
    pub stat: Option<stat_ops::Update>,
    /// Some(true): cancel and queue preserve activation at25. Some(false): cancel.
    pub preserve_queue: Option<bool>,
}
#[derive(Debug)]
pub enum Error {
    Book(prayer_book::Error),
    Drain(prayer_drain::Error),
}
impl Config {
    pub fn toggle(
        &self,
        id: u8,
        vars: &mut Variables,
        stats: &Stats,
        overhead: Option<u8>,
        timers: &mut Timers,
        clock: i32,
        logged: bool,
    ) -> Result<Output, Error> {
        let events = self
            .book
            .toggle(id, vars, stats, overhead, logged)
            .map_err(Error::Book)?;
        let mut preserve_queue = None;
        for event in &events {
            match *event {
                Event::Drain(true) => timers.schedule(self.drain_timer, clock, 1).unwrap(),
                Event::Drain(false) => timers.remove(self.drain_timer),
                Event::Regen { kind, enabled } => {
                    if kind == Regen::Preserve {
                        preserve_queue = Some(enabled);
                    }
                    self.regeneration
                        .prayer_toggle(timers, clock, kind, enabled);
                }
                _ => {}
            }
        }
        Ok(Output {
            events,
            stat: None,
            preserve_queue,
        })
    }
    pub fn tick(
        &self,
        vars: &mut Variables,
        stats: &mut Stats,
        overhead: Option<u8>,
        timers: &mut Timers,
        logged: bool,
    ) -> Result<Output, Error> {
        match self.drain.tick(vars, stats, logged).map_err(Error::Drain)? {
            prayer_drain::Outcome::Tick { updates, stat } => Ok(Output {
                events: updates.into_iter().map(Event::Variable).collect(),
                stat,
                preserve_queue: None,
            }),
            prayer_drain::Outcome::Depleted => {
                let mut events = vec![
                    Event::Message {
                        text: "You have run out of prayer points, you can recharge at an altar."
                            .into(),
                        modal: false,
                    },
                    Event::Sound(self.depletion_sound),
                ];
                events.extend(
                    self.book
                        .disable_all(vars, overhead, logged)
                        .map_err(Error::Book)?,
                );
                timers.remove(self.drain_timer);
                timers.remove(self.regeneration.rapid_timer);
                Ok(Output {
                    events,
                    stat: None,
                    preserve_queue: Some(false),
                })
            }
        }
    }
}
