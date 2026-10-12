//! PrayerDrainScript arithmetic and variable-write order. The caller owns the
//! soft timer and depletion effects; zero points are noticed on the next tick.
use crate::{
    bank_actions::Binding,
    stat_ops,
    stats::Stats,
    variables::{self, Variables},
};
#[derive(Clone, Debug)]
pub struct Config {
    pub enabled: Binding,
    pub counter: Binding,
    pub resistance: Binding,
    pub skill: u8,
    pub effects: Vec<(u8, i32)>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InactiveTimer,
    Variable(variables::Error),
    Stat(stat_ops::Error),
}
impl From<variables::Error> for Error {
    fn from(value: variables::Error) -> Self {
        Self::Variable(value)
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Depleted,
    Tick {
        updates: Vec<variables::Update>,
        stat: Option<stat_ops::Update>,
    },
}
impl Config {
    pub fn update_resistance(
        &self,
        vars: &mut Variables,
        bonus: i32,
        logged_in: bool,
    ) -> Result<Option<variables::Update>, Error> {
        Ok(vars.set_bit(
            self.resistance.base,
            self.resistance.bit,
            60i32.wrapping_add(bonus.wrapping_mul(2)),
            logged_in,
        )?)
    }
    pub fn tick(
        &self,
        vars: &mut Variables,
        stats: &mut Stats,
        logged_in: bool,
    ) -> Result<Outcome, Error> {
        let enabled = vars.bit(self.enabled.bit)?;
        if enabled == 0 {
            return Err(Error::InactiveTimer);
        }
        let level = stats.current(self.skill) as u8 as i32;
        if level == 0 {
            return Ok(Outcome::Depleted);
        }
        let effect = self
            .effects
            .iter()
            .filter(|(id, _)| enabled & 1i32.wrapping_shl(*id as u32) != 0)
            .fold(0i32, |sum, (_, value)| sum.wrapping_add(*value));
        let mut updates = Vec::new();
        let counter = vars.bit(self.counter.bit)?.wrapping_add(effect);
        if let Some(update) =
            vars.set_bit(self.counter.base, self.counter.bit, counter, logged_in)?
        {
            updates.push(update)
        }
        let resistance = vars.bit(self.resistance.bit)?.max(1);
        let cost = vars.bit(self.counter.bit)?.wrapping_sub(1) / resistance;
        let stat = if cost > 0 {
            let remainder = vars
                .bit(self.counter.bit)?
                .wrapping_sub(cost.wrapping_mul(resistance));
            if let Some(update) =
                vars.set_bit(self.counter.base, self.counter.bit, remainder, logged_in)?
            {
                updates.push(update)
            }
            Some(
                stats
                    .apply(
                        self.skill,
                        stat_ops::Operation::Subtract,
                        level.min(cost),
                        0,
                        false,
                    )
                    .map_err(Error::Stat)?,
            )
        } else {
            None
        };
        Ok(Outcome::Tick { updates, stat })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::variables::{Transmit, Varbit, Varp};
    fn binding(id: u16) -> Binding {
        Binding {
            base: Varp {
                id,
                permanent: false,
                transmit: Transmit::Always,
                bit_protect: false,
                client_code: -1,
            },
            bit: Varbit {
                base: id,
                lsb: 0,
                msb: 30,
            },
        }
    }
    fn setup() -> (Config, Variables, Stats) {
        let config = Config {
            enabled: binding(0),
            counter: binding(1),
            resistance: binding(2),
            skill: 5,
            effects: vec![(0, 1), (1, 6)],
        };
        let mut vars = Variables::default();
        vars.set(config.enabled.base, 1, false);
        vars.set(config.resistance.base, 60, false);
        (config, vars, Stats::default())
    }
    #[test]
    fn threshold_is_strict_and_depletion_effects_wait_until_next_tick() {
        let (c, mut v, mut s) = setup();
        for _ in 0..60 {
            assert!(matches!(
                c.tick(&mut v, &mut s, true).unwrap(),
                Outcome::Tick { stat: None, .. }
            ));
        }
        assert_eq!(s.current(5), 1);
        assert_eq!(c.counter.get(&v).unwrap(), 60);
        assert!(matches!(
            c.tick(&mut v, &mut s, true).unwrap(),
            Outcome::Tick { stat: Some(_), .. }
        ));
        assert_eq!(s.current(5), 0);
        assert_eq!(c.counter.get(&v).unwrap(), 1);
        assert_eq!(c.tick(&mut v, &mut s, true).unwrap(), Outcome::Depleted);
    }
    #[test]
    fn simultaneous_prayers_sum_and_cost_caps_at_available_points() {
        let (c, mut v, mut s) = setup();
        v.set(c.enabled.base, 3, false);
        v.set(c.resistance.base, 0, false);
        let Outcome::Tick { updates, stat } = c.tick(&mut v, &mut s, true).unwrap() else {
            panic!()
        };
        assert_eq!(
            updates.iter().map(|u| u.value).collect::<Vec<_>>(),
            vec![7, 1]
        );
        assert_eq!(stat.unwrap().level, 0);
        v.set(c.enabled.base, 0, false);
        assert_eq!(c.tick(&mut v, &mut s, true), Err(Error::InactiveTimer));
        assert_eq!(
            c.update_resistance(&mut v, 15, true)
                .unwrap()
                .unwrap()
                .value,
            90
        );
    }
}
