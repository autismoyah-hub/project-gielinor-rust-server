//! TutorialBankScript gates, coin gift and monotonic booth progression.
use crate::{
    inventory::{Catalog, Insert, Inventory},
    smithing_make::total,
    tutorial_progression,
    tutorial_steps::Step,
    variables::Variables,
};
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub progression: tutorial_progression::Config,
    pub bank: u32,
    pub poll: u32,
    pub coins: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Booth {
    Bank,
    Poll,
}
pub const BANK_DENY: &str =
    "You're not ready to continue yet. You need to know about combat before you go on.";
pub const POLL_DENY: &str = "You need to open your bank first.";
pub const BANK_MESSAGE: &str = "This is your bank. You can store things here for safekeeping.";
pub const POLL_MESSAGES:[&str;2]=["This is a poll booth. Polling booths can be found throughout Gielinor and allow you to vote on important issues.","You cannot vote while on Tutorial Island."];
impl Config {
    pub fn booth(self, id: u32) -> Option<Booth> {
        if id == self.bank {
            Some(Booth::Bank)
        } else if id == self.poll {
            Some(Booth::Poll)
        } else {
            None
        }
    }
    pub fn denial(self, vars: &Variables, booth: Booth) -> Option<&'static str> {
        let (minimum, message) = match booth {
            Booth::Bank => (Step::BankOpen, BANK_DENY),
            Booth::Poll => (Step::PollView, POLL_DENY),
        };
        (!self.progression.require_at_least(vars, minimum)).then_some(message)
    }
    pub fn next(self, vars: &Variables, booth: Booth) -> Option<Step> {
        let next = match booth {
            Booth::Bank => Step::PollView,
            Booth::Poll => Step::AccountTalk,
        };
        (self.progression.current(vars) < next).then_some(next)
    }
    pub fn grant(self, bank: &mut Inventory, catalog: &Catalog) {
        // Unlike TutorialLeave, this gives nothing when even one coin is present.
        if total(bank, catalog, self.coins) <= 0 {
            let _ = bank.insert(catalog, Insert::new(self.coins, 25));
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::StackType,
        variables::{Transmit, Varp},
    };
    fn config() -> Config {
        let varp = |id| Varp {
            id,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        };
        Config {
            progression: tutorial_progression::Config {
                step: varp(1),
                completed: varp(2),
            },
            bank: 10,
            poll: 11,
            coins: 995,
        }
    }
    #[test]
    fn completed_flag_bypasses_gates_but_does_not_suppress_forward_steps() {
        let c = config();
        for raw in -1..=54 {
            for done in [0, 1, 2] {
                let vars = Variables::restore([(1, raw), (2, done)].into());
                let step = Step::from_var(raw);
                for (booth, min, next, deny) in [
                    (Booth::Bank, Step::BankOpen, Step::PollView, BANK_DENY),
                    (Booth::Poll, Step::PollView, Step::AccountTalk, POLL_DENY),
                ] {
                    assert_eq!(
                        c.denial(&vars, booth),
                        if done != 1 && step < min {
                            Some(deny)
                        } else {
                            None
                        }
                    );
                    assert_eq!(c.next(&vars, booth), (step < next).then_some(next));
                }
            }
        }
    }
    #[test]
    fn booth_gift_differs_from_leave_topup_and_full_bank_failure_is_nonfatal() {
        let c = config();
        let mut catalog = Catalog::default();
        catalog.stackable.insert(995);
        for initial in [0, 1, 24, 25, 100] {
            let mut bank = Inventory::new(1, StackType::Always, false);
            if initial > 0 {
                bank.insert(&catalog, Insert::new(995, initial)).unwrap();
            }
            c.grant(&mut bank, &catalog);
            assert_eq!(
                total(&bank, &catalog, 995),
                if initial == 0 { 25 } else { initial }
            );
        }
        let mut bank = Inventory::new(1, StackType::Always, false);
        bank.insert(&catalog, Insert::new(42, 1)).unwrap();
        c.grant(&mut bank, &catalog);
        assert_eq!(total(&bank, &catalog, 995), 0);
        assert_eq!(total(&bank, &catalog, 42), 1);
    }
}
