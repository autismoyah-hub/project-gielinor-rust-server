//! Core rolls and effective levels. Content-specific equipment modifiers are applied separately.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidMultiplier;
fn percent(multiplier: f64) -> i32 {
    (multiplier * 100.0 + 0.00001) as i32
}
fn scale(base: i32, multiplier: i32) -> i32 {
    base.wrapping_mul(multiplier) / 100
}
fn validate(values: &[f64]) -> Result<(), InvalidMultiplier> {
    if values.iter().any(|v| !(*v >= 1.0)) {
        Err(InvalidMultiplier)
    } else {
        Ok(())
    }
}
pub fn effective_attack(
    visible: i32,
    style: i32,
    prayer: f64,
    void: f64,
) -> Result<i32, InvalidMultiplier> {
    validate(&[prayer, void])?;
    Ok(scale(
        scale(visible, percent(prayer)).wrapping_add(style),
        percent(void),
    ))
}
pub fn effective_defence(
    visible: i32,
    style: i32,
    prayer: f64,
    armour: f64,
) -> Result<i32, InvalidMultiplier> {
    validate(&[prayer, armour])?;
    let level = scale(visible, percent(prayer));
    Ok(level
        .wrapping_add(scale(level, percent(armour - 1.0)))
        .wrapping_add(style))
}
pub fn effective_strength(
    visible: i32,
    style: i32,
    prayer: f64,
    void: f64,
    weapon: f64,
) -> Result<i32, InvalidMultiplier> {
    validate(&[prayer, void, weapon])?;
    let level = if visible <= 20 && prayer > 1.0 && prayer <= 1.05 {
        visible.wrapping_add(1)
    } else {
        scale(visible, percent(prayer))
    };
    Ok(scale(
        level
            .wrapping_add(scale(visible, percent(weapon - 1.0)))
            .wrapping_add(style),
        percent(void),
    ))
}
pub fn effective_ranged(
    visible: i32,
    style: i32,
    prayer: f64,
    void: f64,
) -> Result<i32, InvalidMultiplier> {
    effective_strength(visible, style, prayer, void, 1.0)
}
pub fn base_roll(effective: i32, bonus: i32) -> i32 {
    effective.wrapping_mul(bonus.wrapping_add(64))
}
pub fn base_damage(effective: i32, bonus: i32) -> i32 {
    base_roll(effective, bonus).wrapping_add(320) / 640
}
pub fn hit_chance(attack: i32, defence: i32, fang: bool) -> i32 {
    let mut a = attack as f64;
    let mut d = defence as f64;
    if a < 0.0 {
        a = (a + 2.0).min(0.0)
    }
    if d < 0.0 {
        d = (d + 2.0).min(0.0)
    }
    let standard = |a: f64, d: f64| {
        if fang {
            if a > d {
                1.0 - (d + 2.0) * (2.0 * d + 3.0) / ((a + 1.0) * (a + 1.0) * 6.0)
            } else {
                a * (4.0 * a + 5.0) / (6.0 * (a + 1.0) * (d + 1.0))
            }
        } else if a > d {
            1.0 - (d + 2.0) / (2.0 * (a + 1.0))
        } else {
            a / (2.0 * (d + 1.0))
        }
    };
    let result = if a >= 0.0 && d >= 0.0 {
        standard(a, d)
    } else if a >= 0.0 {
        1.0 - 1.0 / (-d + 1.0) / (a + 1.0)
    } else if d >= 0.0 {
        0.0
    } else if fang {
        let (a, d) = (-d, -a);
        if a < d {
            a * (d * 6.0 - 2.0 * a + 5.0) / (6.0 * (d + 1.0) * (d + 1.0))
        } else {
            1.0 - (d + 2.0) * (2.0 * d + 3.0) / (6.0 * (d + 1.0) * (a + 1.0))
        }
    } else {
        standard(-d, -a)
    };
    (result * 10000.0 + 0.5).floor() as i32
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn low_level_prayer_exception_and_roll_signs() {
        assert_eq!(effective_strength(1, 8, 1.05, 1.0, 1.0), Ok(10));
        assert_eq!(effective_attack(1, 8, 1.05, 1.0), Ok(9));
        assert_eq!(effective_strength(99, 11, 1.23, 1.1, 1.12), Ok(157));
        assert_eq!(hit_chance(-10, 100, false), 0);
        assert_eq!(hit_chance(0, -2, false), 0);
        assert!(hit_chance(100, 100, true) > hit_chance(100, 100, false));
    }
}
