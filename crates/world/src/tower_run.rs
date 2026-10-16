//! Durable Cinderworks lifecycle. Encounter timers and ownership are transient;
//! these fields are saved in the existing character/inventory transaction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Run {
    pub id: u32,
    pub services: u8,
    pub imp: bool,
    pub victory: bool,
    pub reward_pending: bool,
    pub first_reward_paid: bool,
    pub clears: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Service {
    Forge = 1,
    Kitchen = 2,
    Altar = 4,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reward {
    FirstClear,
    Replay,
}
impl Run {
    pub fn journal(self) -> String {
        let rows = [
            (
                "Workshop: forge a bronze dagger",
                self.services & 1 != 0,
                true,
            ),
            (
                "Kitchen: catch and cook shrimp",
                self.services & 2 != 0,
                true,
            ),
            (
                "Altar: offer a creature's bones",
                self.services & 4 != 0,
                true,
            ),
            ("Silence the Ash Imp", self.imp, self.services == 7),
            ("Quench the Ash Warden", self.victory, self.imp),
            (
                "Claim the sanctuary reward",
                self.victory && !self.reward_pending,
                self.victory,
            ),
        ];
        let mut text = format!("<col=ffb83f>Cinderworks — run {}</col><br>", self.id);
        for (label, done, available) in rows {
            let color = if done {
                "0dc10d"
            } else if available {
                "ffb83f"
            } else {
                "9f9f9f"
            };
            text.push_str(&format!(
                "<col={color}>{} {label}</col><br>",
                if done { "Done:" } else { "Next:" }
            ));
        }
        text.push_str(&format!(
            "<br>{}<br><br>Victories: {}. First-clear reward: {}.",
            self.guide(),
            self.clears,
            if self.first_reward_paid {
                "claimed"
            } else {
                "unclaimed"
            }
        ));
        text
    }
    pub fn valid(self) -> bool {
        self.services <= 7
            && (!self.imp || self.services == 7)
            && (!self.victory || self.imp)
            && (!self.victory || self.clears > 0)
            && (!self.reward_pending || self.victory)
            && (self.id > 0 || self == Self::default())
            && self.clears <= self.id
            && (!self.first_reward_paid || self.clears > 0)
    }
    pub fn active(self) -> bool {
        self.id > 0 && !self.victory
    }
    pub fn begin(&mut self) -> bool {
        if self.active() || self.reward_pending {
            return false;
        }
        let Some(id) = self.id.checked_add(1) else {
            return false;
        };
        *self = Self {
            id,
            first_reward_paid: self.first_reward_paid,
            clears: self.clears,
            ..Self::default()
        };
        true
    }
    pub fn service(&mut self, service: Service) -> bool {
        if !self.active() || self.imp {
            return false;
        }
        let before = self.services;
        self.services |= service as u8;
        self.services != before
    }
    pub fn defeat_imp(&mut self) -> bool {
        if !self.active() || self.services != 7 || self.imp {
            return false;
        }
        self.imp = true;
        true
    }
    pub fn defeat_warden(&mut self) -> bool {
        if !self.active() || !self.imp {
            return false;
        }
        let Some(clears) = self.clears.checked_add(1) else {
            return false;
        };
        self.victory = true;
        self.reward_pending = true;
        self.clears = clears;
        true
    }
    pub fn reward(self) -> Option<Reward> {
        self.reward_pending.then_some(if self.first_reward_paid {
            Reward::Replay
        } else {
            Reward::FirstClear
        })
    }
    /// Caller must stage the full inventory grant before committing this flag.
    pub fn paid(&mut self, run_id: u32) -> bool {
        if self.id != run_id || !self.reward_pending {
            return false;
        }
        self.reward_pending = false;
        self.first_reward_paid = true;
        true
    }
    pub fn guide(self) -> &'static str {
        if self.id == 0 {
            "Read the sanctuary plaque to begin restoring the Cinderworks."
        } else if self.reward_pending {
            "The foundry is quiet. Claim your reward from the sanctuary chest."
        } else if self.victory {
            "The sanctuary is restored. Leave through the portal, or read the plaque to begin another run."
        } else if self.services & 1 == 0 {
            "Restore the workshop: mine copper and tin, smelt bronze, then smith a bronze dagger."
        } else if self.services & 2 == 0 {
            "Restore the kitchen: catch and cook shrimp in the eastern waterworks."
        } else if self.services & 4 == 0 {
            "Restore the altar: offer bones from a defeated creature at the sanctuary altar."
        } else if !self.imp {
            "Follow the eastern path to the vent gallery. Defeat the Ash Imp to unlock the foundry."
        } else {
            "Enter the northern foundry. Watch the heat warnings and quench the Ash Warden's charge."
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn win(run: &mut Run) {
        for service in [Service::Forge, Service::Kitchen, Service::Altar] {
            assert!(run.service(service));
        }
        assert!(run.defeat_imp());
        assert!(run.defeat_warden());
    }
    #[test]
    fn prerequisites_payment_and_replay_preserve_first_clear_history() {
        let mut run = Run::default();
        assert!(!run.defeat_imp());
        assert!(!run.defeat_warden());
        assert!(run.begin());
        assert!(!run.begin());
        win(&mut run);
        assert_eq!(run.reward(), Some(Reward::FirstClear));
        assert!(!run.begin());
        assert!(!run.paid(run.id + 1));
        assert!(run.paid(run.id));
        assert!(!run.paid(run.id));
        assert!(run.begin());
        assert_eq!(run.services, 0);
        win(&mut run);
        assert_eq!(run.reward(), Some(Reward::Replay));
        assert_eq!(run.clears, 2);
        assert!(run.valid());
    }
    #[test]
    fn duplicate_credits_and_counter_overflow_do_not_change_state() {
        let mut run = Run::default();
        run.begin();
        win(&mut run);
        let before = run;
        assert!(!run.defeat_imp());
        assert!(!run.defeat_warden());
        assert_eq!(run, before);
        run.id = u32::MAX;
        run.paid(run.id);
        let before = run;
        assert!(!run.begin());
        assert_eq!(run, before);
    }
}
