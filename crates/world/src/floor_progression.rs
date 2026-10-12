//! FloorOneScript's unconditional login progression rules. Floor geometry and
//! diary restoration are separate content and are not implied by this policy.
use crate::{simulation::PlayerData, skills::xp_from_level};

#[derive(Clone, Copy)]
pub struct Policy {
    pub attack: u8,
    pub strength: u8,
    pub defence: u8,
    pub hitpoints: u8,
    pub runecrafting: u8,
    pub herblore: u8,
}
impl Policy {
    pub fn login(self, data: &mut PlayerData) -> Vec<&'static str> {
        let mut messages = Vec::new();
        // The runtime uses a single effective multiplier; Kotlin sets player=5
        // and global=1, rather than multiplying two 5x rates.
        if data.xp_rate_hundreds != 500 {
            data.xp_rate_hundreds = 500;
            messages.push("Floor 1 uses 5× OSRS XP.");
        }
        for (skill, level, message) in [
            (
                self.runecrafting,
                2,
                "Your Runecrafting is 2 — enough for mind runes.",
            ),
            (
                self.herblore,
                3,
                "Your Herblore is 3 — enough for an Attack potion.",
            ),
        ] {
            if data.stats.base(skill) < level {
                Self::reset(data, skill, level);
                messages.push(message);
            }
        }
        let hp = data.stats.base(self.hitpoints);
        if [self.attack, self.strength, self.defence]
            .into_iter()
            .all(|id| data.stats.base(id) <= 1)
            && !(10..=20).contains(&hp)
        {
            Self::reset(data, self.hitpoints, 10);
            messages.push("Your Hitpoints are 10 — as they should be.");
        }
        messages
    }
    fn reset(data: &mut PlayerData, skill: u8, level: i8) {
        let xp = xp_from_level(level as u8).expect("fixed starter level is valid");
        data.stats
            .set_xp(skill, xp)
            .expect("fixed starter XP is valid");
        data.stats.set_base(skill, level);
        data.stats.set_current(skill, level);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{stats::Stats, CoordGrid};
    const POLICY: Policy = Policy {
        attack: 0,
        defence: 1,
        strength: 2,
        hitpoints: 3,
        runecrafting: 20,
        herblore: 15,
    };
    fn player() -> PlayerData {
        PlayerData::new(
            1,
            "Test".into(),
            CoordGrid::new(3222, 3218, 0).unwrap(),
            Stats::default(),
        )
    }
    #[test]
    fn login_repairs_starter_levels_and_is_idempotent() {
        let mut data = player();
        assert_eq!(POLICY.login(&mut data).len(), 4);
        assert_eq!(data.xp_rate_hundreds, 500);
        assert_eq!(
            (
                data.stats.xp(20),
                data.stats.base(20),
                data.stats.current(20)
            ),
            (83, 2, 2)
        );
        assert_eq!(
            (
                data.stats.xp(15),
                data.stats.base(15),
                data.stats.current(15)
            ),
            (174, 3, 3)
        );
        assert_eq!(
            (data.stats.xp(3), data.stats.base(3), data.stats.current(3)),
            (1154, 10, 10)
        );
        assert!(POLICY.login(&mut data).is_empty());
    }
    #[test]
    fn preserves_earned_progress_and_drained_current_levels() {
        for hp in 10..=20 {
            let mut data = player();
            data.stats.set_base(3, hp);
            data.stats.set_current(3, 4);
            data.stats.set_fine_xp(3, 12345).unwrap();
            data.stats.set_base(20, 5);
            data.stats.set_current(20, 1);
            data.stats.set_base(15, 6);
            POLICY.login(&mut data);
            assert_eq!(
                (
                    data.stats.base(3),
                    data.stats.current(3),
                    data.stats.fine_xp(3)
                ),
                (hp, 4, 12345)
            );
            assert_eq!(
                (
                    data.stats.base(20),
                    data.stats.current(20),
                    data.stats.base(15)
                ),
                (5, 1, 6)
            );
        }
    }
    #[test]
    fn high_hp_repair_matches_reference_melee_gate() {
        for melee in [0, 1, 2] {
            let mut data = player();
            data.stats.set_base(3, 100);
            data.stats.set_base(melee, 2);
            POLICY.login(&mut data);
            assert_eq!(data.stats.base(3), 100);
            data.stats.set_base(melee, 1);
            POLICY.login(&mut data);
            assert_eq!(data.stats.base(3), 10);
        }
    }
}
