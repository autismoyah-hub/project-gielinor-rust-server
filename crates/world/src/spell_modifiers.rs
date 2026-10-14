//! Ordered spell modifiers from the Kotlin MagicAccuracy/MaxHitOperations.
use crate::melee_modifiers::Npc;
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Spell {
    StandardBook,
    WindSpell,
    WaterSpell,
    EarthSpell,
    FireSpell,
    BindSpell,
    MagicDart,
    SlayerStaffE,
    ChaosGauntlets,
    BoltSpell,
    ChargeSpell,
    SmokeStaff,
    SalveAmuletI,
    SalveAmuletEi,
    AmuletOfAvarice,
    ForinthrySurge,
    BlackMaskI,
    DragonHunterWand,
    DragonHunterLance,
    DragonHunterCrossbow,
    RevenantWeapon,
    WaterTome,
    EarthTome,
    FireTome,
    SunfireRunePassive,
    EfaritaysAid,
    MarkOfDarkness,
    Demonbane,
    PurgingStaff,
    AhrimPassive,
    BrimstonePassive,
}
#[derive(Clone, Copy, Default)]
pub struct Attributes {
    pub spell: u64,
    pub npc: u64,
}
fn scale(v: i32, m: i32, d: i32) -> i32 {
    v.wrapping_mul(m) / d
}
impl Attributes {
    fn s(self, a: Spell) -> bool {
        self.spell & (1 << a as u8) != 0
    }
    fn n(self, a: Npc) -> bool {
        self.npc & (1 << a as u8) != 0
    }
    fn weakness(self) -> bool {
        [
            (Spell::WindSpell, Npc::WindWeakness),
            (Spell::WaterSpell, Npc::WaterWeakness),
            (Spell::EarthSpell, Npc::EarthWeakness),
            (Spell::FireSpell, Npc::FireWeakness),
        ]
        .into_iter()
        .any(|(s, n)| self.s(s) && self.n(n))
    }
    fn tome(self) -> bool {
        [
            (Spell::WaterSpell, Spell::WaterTome),
            (Spell::EarthSpell, Spell::EarthTome),
            (Spell::FireSpell, Spell::FireTome),
        ]
        .into_iter()
        .any(|(s, t)| self.s(s) && self.s(t))
    }
    fn necklace(self) -> (i32, bool) {
        if self.s(Spell::AmuletOfAvarice) && self.n(Npc::Revenant) {
            (
                if self.s(Spell::ForinthrySurge) {
                    35
                } else {
                    20
                },
                false,
            )
        } else if self.s(Spell::SalveAmuletEi) && self.n(Npc::Undead) {
            (20, false)
        } else if self.s(Spell::SalveAmuletI) && self.n(Npc::Undead) {
            (15, false)
        } else {
            (0, self.s(Spell::BlackMaskI) && self.n(Npc::SlayerTask))
        }
    }
    pub fn accuracy(self, base: i32, weakness_percent: i32) -> i32 {
        let (mut bonus, mask) = self.necklace();
        if self.s(Spell::SmokeStaff) && self.s(Spell::StandardBook) {
            bonus += 10;
        }
        let mut v = scale(base, 100 + bonus, 100);
        if self.n(Npc::Draconic) {
            if self.s(Spell::DragonHunterLance) {
                v = scale(v, 6, 5);
            } else if self.s(Spell::DragonHunterWand) {
                v = scale(v, 3, 2);
            } else if self.s(Spell::DragonHunterCrossbow) {
                v = scale(v, 13, 10);
            }
        }
        if mask {
            v = scale(v, 23, 20);
        }
        if self.s(Spell::Demonbane) && self.n(Npc::Demon) {
            let mut percent = if self.n(Npc::DemonbaneResistance) {
                14
            } else {
                20
            };
            if self.s(Spell::MarkOfDarkness) {
                percent *= 2;
            }
            if self.s(Spell::PurgingStaff) {
                percent *= 2;
            }
            v = scale(v, 100 + percent, 100);
        }
        if self.s(Spell::RevenantWeapon) && self.n(Npc::Revenant) {
            v = scale(v, 3, 2);
        }
        if self.s(Spell::WaterTome) && (self.s(Spell::WaterSpell) || self.s(Spell::BindSpell)) {
            v = scale(v, 6, 5);
        }
        if self.weakness() {
            v = v.wrapping_add((base as f64 * (weakness_percent as f64 / 100.0)) as i32);
        }
        v
    }
    pub fn player_accuracy(self, base: i32) -> i32 {
        // Empty NPC attributes remove every PvN-only modifier.
        Self { npc: 0, ..self }.accuracy(base, 0)
    }
    pub fn damage(self, base: i32, magic: i32, equipment_bonus: i32, prayer_bonus: i32) -> i32 {
        let mut v = base;
        if self.s(Spell::MagicDart) {
            v = if self.s(Spell::SlayerStaffE) && self.n(Npc::SlayerTask) {
                13i32.wrapping_add(magic / 6)
            } else {
                10i32.wrapping_add(magic / 10)
            };
        }
        if self.s(Spell::ChaosGauntlets) && self.s(Spell::BoltSpell) {
            v = v.wrapping_add(3);
        }
        if self.s(Spell::ChargeSpell) {
            v = v.wrapping_add(10);
        }
        let (necklace, mask) = self.necklace();
        let mut bonus = equipment_bonus;
        if self.s(Spell::SmokeStaff) && self.s(Spell::StandardBook) {
            bonus = bonus.wrapping_add(100);
        }
        bonus = bonus.wrapping_add(necklace * 10).wrapping_add(prayer_bonus);
        v = v.wrapping_add(scale(v, bonus, 1000));
        if mask {
            v = scale(v, 23, 20);
        }
        if self.n(Npc::Draconic) {
            if self.s(Spell::DragonHunterLance) || self.s(Spell::DragonHunterWand) {
                v = scale(v, 6, 5);
            } else if self.s(Spell::DragonHunterCrossbow) {
                v = scale(v, 5, 4);
            }
        }
        if self.s(Spell::RevenantWeapon) && self.n(Npc::Wilderness) {
            v = scale(v, 3, 2);
        }
        v
    }
    pub fn player_damage(
        self,
        base: i32,
        magic: i32,
        equipment_bonus: i32,
        prayer_bonus: i32,
    ) -> i32 {
        Self { npc: 0, ..self }.damage(base, magic, equipment_bonus, prayer_bonus)
    }
    pub fn damage_range(
        self,
        modified: i32,
        base: i32,
        speed: i32,
        weakness_percent: i32,
    ) -> (i32, i32) {
        let mut max = modified;
        if self.weakness() {
            max = max.wrapping_add((base as f64 * (weakness_percent as f64 / 100.0)) as i32);
        }
        // The minimum is captured before tomes, darkness, Ahrim and tormented bonuses.
        let min = if self.s(Spell::SunfireRunePassive) {
            max / 10
        } else {
            0
        };
        if self.tome() {
            max = scale(max, 11, 10);
        }
        if self.s(Spell::MarkOfDarkness) && self.s(Spell::Demonbane) && self.n(Npc::Demon) {
            max = scale(
                max,
                if self.s(Spell::PurgingStaff) {
                    150
                } else {
                    125
                },
                100,
            );
        }
        if self.s(Spell::AhrimPassive) {
            max = scale(max, 13, 10);
        }
        if self.n(Npc::TormentedDemonUnshielded) {
            max = max.wrapping_add(speed.wrapping_mul(speed).wrapping_sub(16).max(0));
        }
        (min, max)
    }
    pub fn player_damage_range(self, modified: i32) -> (i32, i32) {
        let min = if self.s(Spell::SunfireRunePassive) {
            modified / 10
        } else {
            0
        };
        let mut max = modified;
        if self.tome() {
            max = scale(max, 12, 10);
        }
        if self.s(Spell::AhrimPassive) {
            max = scale(max, 13, 10);
        }
        (min, max)
    }
}
