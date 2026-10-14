//! Ordered ranged modifiers from RangedAccuracyOperations/RangedMaxHitOperations.
use crate::melee_modifiers::Npc;
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Ranged {
    Heavy,
    ShortFuse,
    MediumFuse,
    LongFuse,
    CrystalBow,
    CrystalHelm,
    CrystalLegs,
    CrystalBody,
    RevenantWeapon,
    DragonHunterCrossbow,
    ScorchingBow,
    AmuletOfAvarice,
    ForinthrySurge,
    SalveAmuletI,
    SalveAmuletEi,
    BlackMaskI,
    TwistedBow,
    RatBoneWeapon,
    CorpBaneWeapon,
}
#[derive(Clone, Copy, Default)]
pub struct Attributes {
    pub ranged: u64,
    pub npc: u64,
}
fn scale(v: i32, m: i32, d: i32) -> i32 {
    v.wrapping_mul(m) / d
}
impl Attributes {
    fn r(self, a: Ranged) -> bool {
        self.ranged & (1 << a as u8) != 0
    }
    fn n(self, a: Npc) -> bool {
        self.npc & (1 << a as u8) != 0
    }
    fn crystal(self, v: i32, divisor: i32) -> i32 {
        if !self.r(Ranged::CrystalBow) {
            return v;
        }
        let additive = i32::from(self.r(Ranged::CrystalHelm))
            + 2 * i32::from(self.r(Ranged::CrystalLegs))
            + 3 * i32::from(self.r(Ranged::CrystalBody));
        scale(v, divisor + additive, divisor)
    }
    fn twisted(self, v: i32, magic: i32, factor: i32, base: i32) -> i32 {
        if !self.r(Ranged::TwistedBow) {
            return v;
        }
        let magic = magic.min(if self.n(Npc::Xerician) { 350 } else { 250 });
        let triple = magic.wrapping_mul(3);
        let linear = triple.wrapping_sub(factor) / 100;
        let deviation = (triple / 10).wrapping_sub(10 * factor);
        let penalty = deviation.wrapping_mul(deviation) / 100;
        scale(v, base.wrapping_add(linear).wrapping_sub(penalty), 100)
    }
    fn fuse(self, v: i32, distance: i32) -> i32 {
        let multiplier = if self.r(Ranged::ShortFuse) {
            if distance >= 7 {
                2
            } else if distance >= 4 {
                3
            } else {
                4
            }
        } else if self.r(Ranged::MediumFuse) {
            if distance < 4 || distance >= 7 {
                3
            } else {
                4
            }
        } else if self.r(Ranged::LongFuse) {
            if distance < 4 {
                2
            } else if distance < 7 {
                3
            } else {
                4
            }
        } else {
            return v;
        };
        scale(v, multiplier, 4)
    }
    pub fn player_accuracy(self, base: i32, distance: i32) -> i32 {
        self.fuse(self.crystal(base, 20), distance)
    }
    pub fn player_damage(self, base: i32) -> i32 {
        self.crystal(base, 40)
    }
    pub fn accuracy(self, base: i32, magic: i32, distance: i32) -> i32 {
        let mut v = self.crystal(base, 20);
        if self.r(Ranged::AmuletOfAvarice) && self.n(Npc::Revenant) {
            v = scale(
                v,
                if self.r(Ranged::ForinthrySurge) {
                    27
                } else {
                    24
                },
                20,
            );
        } else if self.r(Ranged::SalveAmuletEi) && self.n(Npc::Undead) {
            v = scale(v, 6, 5);
        } else if self.r(Ranged::SalveAmuletI) && self.n(Npc::Undead) {
            v = scale(v, 7, 6);
        } else if self.r(Ranged::BlackMaskI) && self.n(Npc::SlayerTask) {
            v = scale(v, 23, 20);
        }
        v = self.twisted(v, magic, 10, 140);
        if self.r(Ranged::RevenantWeapon) && self.n(Npc::Wilderness) {
            v = scale(v, 3, 2);
        }
        if self.r(Ranged::DragonHunterCrossbow) && self.n(Npc::Draconic) {
            v = scale(v, 13, 10);
        }
        v = self.fuse(v, distance);
        if self.r(Ranged::ScorchingBow) && self.n(Npc::Demon) {
            v = scale(
                v,
                if self.n(Npc::DemonbaneResistance) {
                    121
                } else {
                    130
                },
                100,
            );
        }
        v
    }
    pub fn damage(self, base: i32, magic: i32) -> i32 {
        let mut v = self.crystal(base, 40);
        let mut revenant = self.r(Ranged::RevenantWeapon) && self.n(Npc::Wilderness);
        let mut dragon = self.r(Ranged::DragonHunterCrossbow) && self.n(Npc::Draconic);
        let mut demon = self.r(Ranged::ScorchingBow) && self.n(Npc::Demon);
        if self.r(Ranged::AmuletOfAvarice) && self.n(Npc::Revenant) {
            v = scale(
                v,
                if self.r(Ranged::ForinthrySurge) {
                    27
                } else {
                    24
                },
                20,
            );
        } else if self.r(Ranged::SalveAmuletEi) && self.n(Npc::Undead) {
            v = scale(v, 6, 5);
        } else if self.r(Ranged::SalveAmuletI) && self.n(Npc::Undead) {
            v = scale(v, 7, 6);
        } else if self.r(Ranged::BlackMaskI) && self.n(Npc::SlayerTask) {
            let mut multiplier = 23;
            if revenant {
                revenant = false;
                multiplier += 10;
            }
            if dragon {
                dragon = false;
                multiplier += 5;
            }
            if demon {
                demon = false;
                multiplier += 6;
            }
            v = scale(v, multiplier, 20);
        }
        v = self.twisted(v, magic, 14, 250);
        if revenant {
            v = scale(v, 3, 2);
        }
        if dragon {
            v = scale(v, 5, 4);
        }
        if demon {
            v = scale(
                v,
                if self.n(Npc::DemonbaneResistance) {
                    121
                } else {
                    130
                },
                100,
            );
        }
        if self.r(Ranged::RatBoneWeapon) && self.n(Npc::Rat) {
            v = v.wrapping_add(10);
        }
        v
    }
    pub fn post_spec(self, damage: i32, bolt: i32, rate: i32) -> i32 {
        let mut v = damage;
        if self.r(Ranged::Heavy) && self.n(Npc::TormentedDemonUnshielded) {
            v = v.wrapping_add(rate.wrapping_mul(rate).wrapping_sub(16).max(0));
        }
        v = v.wrapping_add(bolt);
        if self.n(Npc::CorporealBeast) && !self.r(Ranged::CorpBaneWeapon) {
            v /= 2;
        }
        v
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn slayer_damage_combines_weapon_additives_before_rounding() {
        let attrs = Attributes {
            ranged: [
                Ranged::BlackMaskI,
                Ranged::RevenantWeapon,
                Ranged::DragonHunterCrossbow,
                Ranged::ScorchingBow,
            ]
            .into_iter()
            .fold(0, |bits, attr| bits | (1 << attr as u8)),
            npc: [
                Npc::SlayerTask,
                Npc::Wilderness,
                Npc::Draconic,
                Npc::Demon,
                Npc::DemonbaneResistance,
            ]
            .into_iter()
            .fold(0, |bits, attr| bits | (1 << attr as u8)),
        };
        // The source combines 23+10+5+6 before one division, suppressing the
        // later individual modifiers (including demon resistance) on this branch.
        assert_eq!(attrs.damage(19, 0), 41);
        let mut salve = attrs;
        salve.ranged |= 1 << Ranged::SalveAmuletEi as u8;
        salve.npc |= 1 << Npc::Undead as u8;
        assert_eq!(salve.damage(19, 0), 49);
    }
    #[test]
    fn tormented_bonus_and_bolt_bonus_precede_corporeal_reduction() {
        let mut attrs = Attributes {
            ranged: 1 << Ranged::Heavy as u8,
            npc: (1 << Npc::TormentedDemonUnshielded as u8) | (1 << Npc::CorporealBeast as u8),
        };
        assert_eq!(attrs.post_spec(10, 5, 5), 12);
        attrs.ranged |= 1 << Ranged::CorpBaneWeapon as u8;
        assert_eq!(attrs.post_spec(10, 5, 5), 24);
    }
}
