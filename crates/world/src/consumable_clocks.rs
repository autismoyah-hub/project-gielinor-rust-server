//! Independent native consumable locks over the existing item/stat transactions.
//! Wiki policy: extend an active attack clock; consuming while ready adds no new lock.
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Food,
    Potion,
    ComboFood,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Policy {
    pub kind: Kind,
    pub consume_delay: i32,
    pub combat_delay: i32,
    pub members: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Rules {
    pub foods: BTreeMap<i32, Policy>,
    pub potions: BTreeMap<i32, Policy>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    food_until: i32,
    potion_until: i32,
    combo_until: i32,
    attack_until: i32,
}
impl Default for State {
    fn default() -> Self {
        Self {
            food_until: -1,
            potion_until: -1,
            combo_until: -1,
            attack_until: -1,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Delayed,
    InvalidPolicy,
    ClockOverflow,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plan {
    pub state: State,
    pub action_delay: i32,
}
impl State {
    pub fn can_consume(self, now: i32, kind: Kind) -> bool {
        now >= 0
            && self.combo_until <= now
            && match kind {
                Kind::Food => self.food_until <= now,
                Kind::Potion => self.potion_until <= now,
                Kind::ComboFood => true,
            }
    }
    /// Prepare before any inventory, RNG or stat changes; publish only on success.
    pub fn plan(self, now: i32, policy: Policy, action_delay: i32) -> Result<Plan, Error> {
        if now < 0 || policy.consume_delay < 0 || policy.combat_delay < 0 {
            return Err(Error::InvalidPolicy);
        }
        if !self.can_consume(now, policy.kind) {
            return Err(Error::Delayed);
        }
        let mut next = self;
        if policy.consume_delay > 0 {
            let until = now
                .checked_add(policy.consume_delay)
                .ok_or(Error::ClockOverflow)?;
            let key = match policy.kind {
                Kind::Food => &mut next.food_until,
                Kind::Potion => &mut next.potion_until,
                Kind::ComboFood => &mut next.combo_until,
            };
            *key = (*key).max(until);
        }
        let active = self.effective_attack_delay(action_delay);
        let action_delay = if policy.combat_delay > 0 && active > now {
            let until = active
                .checked_add(policy.combat_delay)
                .ok_or(Error::ClockOverflow)?;
            next.attack_until = until;
            until
        } else {
            action_delay
        };
        Ok(Plan {
            state: next,
            action_delay,
        })
    }
    /// Keep a committed food contribution intact if a skilling clock is replaced.
    pub fn effective_attack_delay(self, action_delay: i32) -> i32 {
        action_delay.max(self.attack_until)
    }
    pub fn remaining_attack_delay(self, now: i32, action_delay: i32) -> i32 {
        self.effective_attack_delay(action_delay)
            .saturating_sub(now)
            .max(0)
    }
    pub fn deadlines(self) -> [i32; 4] {
        [
            self.food_until,
            self.potion_until,
            self.combo_until,
            self.attack_until,
        ]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn policy(kind: Kind) -> Policy {
        Policy {
            kind,
            consume_delay: 3,
            combat_delay: match kind {
                Kind::Food => 3,
                Kind::Potion => 0,
                Kind::ComboFood => 2,
            },
            members: false,
        }
    }
    #[test]
    fn category_order_allows_food_potion_combo_and_combo_blocks_remaining_categories() {
        use Kind::*;
        for (order, want) in [
            ([Food, Potion, ComboFood], [true, true, true]),
            ([Potion, Food, ComboFood], [true, true, true]),
            ([Food, ComboFood, Potion], [true, true, false]),
            ([Potion, ComboFood, Food], [true, true, false]),
            ([ComboFood, Food, Potion], [true, false, false]),
            ([ComboFood, Potion, Food], [true, false, false]),
        ] {
            let mut s = State::default();
            let mut action = 0;
            for (i, kind) in order.into_iter().enumerate() {
                match s.plan(1, policy(kind), action) {
                    Ok(p) => {
                        assert!(want[i]);
                        s = p.state;
                        action = p.action_delay;
                    }
                    Err(Error::Delayed) => assert!(!want[i]),
                    other => panic!("unexpected plan {other:?}"),
                }
            }
            assert_eq!(action, 0);
            assert_eq!(s.deadlines()[2], 4);
            for kind in [Food, Potion, ComboFood] {
                assert!(!s.can_consume(3, kind));
                assert!(s.can_consume(4, kind));
            }
        }
    }
    #[test]
    fn active_weapon_delay_is_extended_but_ready_weapon_has_no_new_attack_lock() {
        let ready = State::default().plan(10, policy(Kind::Food), 10).unwrap();
        assert_eq!(ready.action_delay, 10);
        assert_eq!(ready.state.deadlines()[3], -1);
        assert_eq!(ready.state.remaining_attack_delay(10, 10), 0);
        let food = State::default().plan(10, policy(Kind::Food), 30).unwrap();
        assert_eq!(food.action_delay, 33);
        let potion = food.state.plan(10, policy(Kind::Potion), 33).unwrap();
        assert_eq!(potion.action_delay, 33);
        let combo = potion.state.plan(10, policy(Kind::ComboFood), 33).unwrap();
        assert_eq!(combo.action_delay, 35);
        assert_eq!(combo.state.remaining_attack_delay(10, 0), 25);
        assert_eq!(combo.state.remaining_attack_delay(35, 0), 0);
        assert_eq!(State::default().deadlines(), [-1; 4]);
    }
    #[test]
    fn invalid_policy_and_deadline_overflow_refuse_before_a_plan_can_commit() {
        let s = State::default();
        for (now, consume, combat, attack, want) in [
            (-1, 3, 3, 0, Error::InvalidPolicy),
            (1, -1, 3, 0, Error::InvalidPolicy),
            (1, 3, -1, 0, Error::InvalidPolicy),
            (i32::MAX - 1, 3, 3, 0, Error::ClockOverflow),
            (1, 0, 3, i32::MAX - 1, Error::ClockOverflow),
        ] {
            let mut p = policy(Kind::Food);
            p.consume_delay = consume;
            p.combat_delay = combat;
            assert_eq!(s.plan(now, p, attack), Err(want));
        }
        let p = s.plan(1, policy(Kind::Food), 0).unwrap();
        assert_eq!(p.state.plan(2, policy(Kind::Food), 0), Err(Error::Delayed));
        assert!(p.state.plan(4, policy(Kind::Food), 0).is_ok());
    }
}
