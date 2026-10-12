//! PrayerTabScript's protected toggle operation. The world owns access queuing,
//! modal lifetimes and timers; this module emits their ordered effects.
use crate::{
    bank_actions::Binding,
    stats::Stats,
    variables::{self, Variables},
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Regen {
    RapidRestore,
    RapidHeal,
    Preserve,
}
#[derive(Clone, Debug)]
pub struct Prayer {
    pub id: u8,
    pub name: String,
    pub level: i32,
    pub sound: u16,
    pub enabled: Binding,
    pub overhead: Option<u8>,
    pub unlock: Option<Binding>,
    pub unlock_state: i32,
    pub defence: Option<i32>,
    pub locked_message: Option<String>,
    pub regen: Option<Regen>,
}
#[derive(Clone, Debug)]
pub struct Config {
    pub prayer_skill: u8,
    pub defence_skill: u8,
    pub all_active: Binding,
    pub quick_active: Binding,
    pub drain_counter: Binding,
    pub disable_sound: u16,
    pub prayers: BTreeMap<u8, Prayer>,
    /// Union of the attack, strength, defence and overhead groups, in source order.
    pub collisions: BTreeMap<u8, Vec<u8>>,
    pub components: BTreeMap<u32, u8>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Variable(variables::Update),
    Sound(u16),
    Message { text: String, modal: bool },
    RefreshCombatRange,
    Drain(bool),
    Regen { kind: Regen, enabled: bool },
    Overhead(Option<u8>),
}
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    UnknownPrayer,
    Variable(variables::Error),
}
impl From<variables::Error> for Error {
    fn from(value: variables::Error) -> Self {
        Self::Variable(value)
    }
}
impl Prayer {
    pub fn requirements(
        &self,
        vars: &Variables,
        stats: &Stats,
        prayer: u8,
        defence: u8,
    ) -> Result<bool, Error> {
        if let Some(binding) = self.unlock {
            if vars.bit(binding.bit)? < self.unlock_state {
                return Ok(false);
            }
        }
        Ok(self
            .defence
            .is_none_or(|level| stats.base(defence) as u8 as i32 >= level)
            && stats.base(prayer) as u8 as i32 >= self.level)
    }
    pub fn failure_message(&self, stats: &Stats, prayer: u8) -> String {
        let level = || {
            format!(
                "You need a <col=000080>Prayer</col> level of {} to use <col=000080>{}</col>.",
                self.level, self.name
            )
        };
        if stats.base(prayer) as u8 as i32 >= self.level {
            self.locked_message.clone().unwrap_or_else(level)
        } else {
            level()
        }
    }
}
impl Config {
    fn write(
        vars: &mut Variables,
        binding: Binding,
        value: i32,
        logged: bool,
        events: &mut Vec<Event>,
    ) -> Result<(), Error> {
        events.extend(
            vars.set_bit(binding.base, binding.bit, value, logged)?
                .map(Event::Variable),
        );
        Ok(())
    }
    fn resync(vars: &Variables, binding: Binding, events: &mut Vec<Event>) {
        if binding.base.transmit == variables::Transmit::Never {
            return;
        }
        events.push(Event::Variable(variables::Update {
            id: binding.base.id,
            value: vars.get(binding.base.id),
        }));
    }
    /// Protected clicks with unmet requirements fail immediately without a modal.
    /// None means the caller should attempt protected access or queue the toggle.
    pub fn protected_failure(
        &self,
        id: u8,
        vars: &Variables,
        stats: &Stats,
    ) -> Result<Option<Vec<Event>>, Error> {
        let p = self.prayers.get(&id).ok_or(Error::UnknownPrayer)?;
        if p.requirements(vars, stats, self.prayer_skill, self.defence_skill)? {
            return Ok(None);
        }
        let mut events = vec![Event::Message {
            text: p
                .failure_message(stats, self.prayer_skill)
                .replace("<br>", " "),
            modal: false,
        }];
        Self::resync(vars, p.enabled, &mut events);
        events.push(Event::Sound(self.disable_sound));
        Ok(Some(events))
    }
    pub fn toggle(
        &self,
        id: u8,
        vars: &mut Variables,
        stats: &Stats,
        overhead: Option<u8>,
        logged: bool,
    ) -> Result<Vec<Event>, Error> {
        let p = self.prayers.get(&id).ok_or(Error::UnknownPrayer)?;
        let mut next = vars.clone();
        let mut events = Vec::new();
        if next.bit(p.enabled.bit)? != 0 {
            Self::write(&mut next, p.enabled, 0, logged, &mut events)?;
            events.push(Event::Sound(self.disable_sound));
            if let Some(kind) = p.regen {
                events.push(Event::Regen {
                    kind,
                    enabled: false,
                });
            }
            if p.overhead.is_some() && p.overhead == overhead {
                events.push(Event::Overhead(None));
            }
            if next.bit(self.all_active.bit)? == 0 {
                Self::write(&mut next, self.quick_active, 0, logged, &mut events)?;
                Self::write(&mut next, self.drain_counter, 0, logged, &mut events)?;
                events.push(Event::Drain(false));
            }
        } else {
            if stats.current(self.prayer_skill) == 0 {
                return Ok(vec![Event::RefreshCombatRange]);
            }
            if !p.requirements(&next, stats, self.prayer_skill, self.defence_skill)? {
                Self::resync(&next, p.enabled, &mut events);
                events.push(Event::Message {
                    text: p.failure_message(stats, self.prayer_skill),
                    modal: true,
                });
                return Ok(events);
            }
            for id in self.collisions.get(&id).into_iter().flatten() {
                let collision = self.prayers.get(id).ok_or(Error::UnknownPrayer)?;
                if next.bit(collision.enabled.bit)? == 0 {
                    continue;
                }
                Self::write(&mut next, collision.enabled, 0, logged, &mut events)?;
                if let Some(kind) = collision.regen {
                    events.push(Event::Regen {
                        kind,
                        enabled: false,
                    });
                }
            }
            Self::write(&mut next, p.enabled, 1, logged, &mut events)?;
            events.push(Event::Sound(p.sound));
            events.push(Event::Drain(true));
            if let Some(kind) = p.regen {
                events.push(Event::Regen {
                    kind,
                    enabled: true,
                });
            }
            if let Some(icon) = p.overhead {
                events.push(Event::Overhead(Some(icon)));
            }
        }
        *vars = next;
        Ok(events)
    }

    /// Player.disablePrayers: notably this does not restart health/boost timers.
    /// The caller clears the preserve activation queue and the drain/rapid timers.
    pub fn disable_all(
        &self,
        vars: &mut Variables,
        overhead: Option<u8>,
        logged: bool,
    ) -> Result<Vec<Event>, Error> {
        let mut next = vars.clone();
        let mut events = Vec::new();
        Self::write(&mut next, self.all_active, 0, logged, &mut events)?;
        Self::write(&mut next, self.drain_counter, 0, logged, &mut events)?;
        if next.bit(self.quick_active.bit)? != 0 {
            Self::write(&mut next, self.quick_active, 0, logged, &mut events)?;
        }
        // constants.isOverhead includes combined protection and curse icons.
        if overhead.is_some_and(|icon| icon <= 14) {
            events.push(Event::Overhead(None));
        }
        events.push(Event::Drain(false));
        *vars = next;
        Ok(events)
    }
}
