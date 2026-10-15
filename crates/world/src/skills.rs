use crate::stats::{InvalidXp, Stats};
use std::sync::OnceLock;
pub fn xp_table() -> &'static [i32; 126] {
    static TABLE: OnceLock<[i32; 126]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = [0; 126];
        let mut accumulated = 0;
        for (level, entry) in table.iter_mut().enumerate().skip(1) {
            accumulated += (level as f64 + 300.0 * 2f64.powf(level as f64 / 7.0)) as i32;
            *entry = accumulated / 4;
        }
        table
    })
}
pub fn xp_from_level(level: u8) -> Option<i32> {
    level
        .checked_sub(1)
        .and_then(|i| xp_table().get(i as usize).copied())
}
pub fn level_from_xp(xp: i32) -> u8 {
    match xp_table().binary_search(&xp) {
        Ok(i) => (i + 1) as u8,
        Err(i) => i as u8,
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Advancement {
    pub added_xp: i32,
    pub old_level: u8,
    pub new_level: u8,
}
pub fn advance(
    stats: &mut Stats,
    skill: u8,
    xp: f64,
    rate: f64,
    max_level: u8,
) -> Result<Advancement, InvalidXp> {
    let fine = xp * rate * 10.0;
    if fine.is_infinite() {
        return Err(InvalidXp);
    }
    let current = stats.fine_xp(skill);
    let sum = (current as f64 + fine).min(Stats::MAX_FINE_XP as f64) as i32;
    // Kotlin Double.toInt maps NaN to zero. f64::min treats NaN differently.
    let sum = if fine.is_nan() { 0 } else { sum };
    let added = sum - current;
    let old = stats.base(skill) as u8;
    let mut new = old;
    if added > 0 {
        stats.set_fine_xp(skill, sum)?;
    }
    if added != 0
        && old < max_level
        && xp_from_level(old + 1).is_some_and(|next| stats.xp(skill) >= next)
    {
        new = max_level.min(level_from_xp(stats.xp(skill)));
        stats.set_base(skill, new as i8);
        if stats.current(skill) as u8 == old {
            stats.set_current(skill, new as i8);
        }
    }
    Ok(Advancement {
        added_xp: added / 10,
        old_level: old,
        new_level: new,
    })
}
pub fn success_rate(low: i32, high: i32, level: i32, max_level: i32) -> f64 {
    let lo = low.wrapping_mul(max_level.wrapping_sub(level)) as f64 / (max_level as f64 - 1.0);
    let hi = high.wrapping_mul(level.wrapping_sub(1)) as f64 / (max_level as f64 - 1.0);
    (1.0 + (lo + hi + 0.5).floor()) / 256.0
}
pub fn combat_level(
    attack: i32,
    defence: i32,
    strength: i32,
    hitpoints: i32,
    ranged: i32,
    magic: i32,
    prayer: i32,
) -> i32 {
    let base = 0.25 * (defence + hitpoints + prayer / 2) as f64;
    let melee = 0.325 * (attack + strength) as f64;
    let range = 0.325 * (ranged + ranged / 2) as f64;
    let mage = 0.325 * (magic + magic / 2) as f64;
    (base + melee.max(range.max(mage))).floor() as i32
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wiki_base_awards_preserve_tenths_at_one_and_five_times_rate() {
        for (base, one, five) in [
            (25.0, 250, 1250),
            (40.0, 400, 2000),
            (4.5, 45, 225),
            (6.2, 62, 310),
            (17.5, 175, 875),
            (37.5, 375, 1875),
        ] {
            for (rate, expected) in [(1.0, one), (5.0, five)] {
                let mut stats = Stats::default();
                for award in 1..=3 {
                    advance(&mut stats, 8, base, rate, 99).unwrap();
                    assert_eq!(stats.fine_xp(8), expected * award);
                    assert_eq!(stats.xp(8), expected * award / 10);
                }
            }
        }
    }
    #[test]
    fn per_award_truncation_and_cap_preserve_fractional_storage() {
        let mut stats = Stats::default();
        for _ in 0..3 {
            advance(&mut stats, 0, 0.09, 1.0, 99).unwrap();
        }
        assert_eq!(stats.fine_xp(0), 0);
        stats.set_fine_xp(0, Stats::MAX_FINE_XP - 7).unwrap();
        advance(&mut stats, 0, 4.5, 5.0, 99).unwrap();
        assert_eq!(stats.fine_xp(0), Stats::MAX_FINE_XP);
        let next = advance(&mut stats, 0, 4.5, 5.0, 99).unwrap();
        assert_eq!(next.added_xp, 0);
        assert_eq!(stats.fine_xp(0), Stats::MAX_FINE_XP);
    }
    #[test]
    fn xp_levels_and_boosts() {
        assert_eq!(xp_from_level(99), Some(13034431));
        assert_eq!(xp_from_level(2), Some(83));
        assert_eq!(level_from_xp(82), 1);
        let mut stats = Stats::default();
        stats.set_current(0, 5);
        let change = advance(&mut stats, 0, 100.05, 1.0, 99).unwrap();
        assert_eq!(change.new_level, 2);
        assert_eq!(stats.current(0), 5);
        assert_eq!(stats.fine_xp(0), 1000);
    }
}
