//! Ordered equipment and target modifiers from Kotlin combat formula operations.
#[derive(Clone, Copy, Default)]
pub struct Attributes {
    pub melee: u64,
    pub npc: u64,
}
impl Attributes {
    fn m(self, attribute: Melee) -> bool {
        self.melee & (1u64 << attribute as u8) != 0
    }
    fn n(self, attribute: Npc) -> bool {
        self.npc & (1u64 << attribute as u8) != 0
    }
    fn inquisitor(self, value: i32) -> i32 {
        if !self.m(Melee::Crush) {
            return value;
        }
        let pieces = [
            Melee::InquisitorHelm,
            Melee::InquisitorTop,
            Melee::InquisitorBottom,
        ]
        .into_iter()
        .filter(|&a| self.m(a))
        .count() as i32;
        let bonus = if pieces == 0 {
            0
        } else if self.m(Melee::InquisitorWeapon) {
            pieces * 5
        } else if pieces == 3 {
            5
        } else {
            pieces
        };
        if bonus > 0 {
            scale(value, 200 + bonus, 200)
        } else {
            value
        }
    }
    fn necklace(self, value: i32) -> i32 {
        if self.m(Melee::AmuletOfAvarice) && self.n(Npc::Revenant) {
            scale(
                value,
                if self.m(Melee::ForinthrySurge) {
                    27
                } else {
                    24
                },
                20,
            )
        } else if self.m(Melee::SalveAmuletE) && self.n(Npc::Undead) {
            scale(value, 6, 5)
        } else if (self.m(Melee::SalveAmulet) && self.n(Npc::Undead))
            || (self.m(Melee::BlackMask) && self.n(Npc::SlayerTask))
        {
            scale(value, 7, 6)
        } else {
            value
        }
    }
    fn demonbane(self, mut value: i32) -> i32 {
        if self.m(Melee::Arclight) && self.n(Npc::Demon) {
            value = scale(
                value,
                if self.n(Npc::DemonbaneResistance) {
                    149
                } else {
                    170
                },
                100,
            );
        }
        if self.m(Melee::BurningClaws) && self.n(Npc::Demon) {
            value = scale(
                value,
                if self.n(Npc::DemonbaneResistance) {
                    207
                } else {
                    210
                },
                200,
            );
        }
        value
    }
    pub fn damage(self, base: i32) -> i32 {
        let mut v = self.demonbane(self.necklace(base));
        if self.m(Melee::Obsidian) && self.m(Melee::TzHaarWeapon) {
            v = v.wrapping_add(base / 10);
        }
        for weapon in [Melee::DragonHunterLance, Melee::DragonHunterWand] {
            if self.m(weapon) && self.n(Npc::Draconic) {
                v = scale(v, 6, 5);
            }
        }
        for weapon in [
            Melee::KerisWeapon,
            Melee::KerisBreachPartisan,
            Melee::KerisSunPartisan,
        ] {
            if self.m(weapon) && self.n(Npc::Kalphite) {
                v = scale(v, 133, 100);
            }
        }
        if self.m(Melee::BarroniteMaceWeapon) && self.n(Npc::Golem) {
            v = scale(v, 23, 20);
        }
        if self.m(Melee::RevenantWeapon) && self.n(Npc::Wilderness) {
            v = scale(v, 3, 2);
        }
        if self.m(Melee::Silverlight) && self.n(Npc::Demon) {
            v = scale(
                v,
                if self.n(Npc::DemonbaneResistance) {
                    71
                } else {
                    80
                },
                50,
            );
        }
        if self.m(Melee::LeafBladed) && self.n(Npc::Leafy) {
            v = scale(v, 47, 40);
        }
        if self.m(Melee::ColossalBlade) {
            let add = if self.n(Npc::Size2) {
                4
            } else if self.n(Npc::Size3) {
                6
            } else if self.n(Npc::Size4) {
                8
            } else if self.n(Npc::Size5OrMore) {
                10
            } else {
                2
            };
            v = v.wrapping_add(add);
        }
        if self.m(Melee::RatBoneWeapon) && self.n(Npc::Rat) {
            v = v.wrapping_add(10);
        }
        self.inquisitor(v)
    }
    pub fn attack_roll(self, base: i32) -> i32 {
        let mut v = self.necklace(base);
        if self.m(Melee::Obsidian) && self.m(Melee::TzHaarWeapon) {
            v = v.wrapping_add(base / 10);
        }
        if self.m(Melee::RevenantWeapon) && self.n(Npc::Wilderness) {
            v = scale(v, 3, 2);
        }
        v = self.demonbane(v);
        if self.m(Melee::DragonHunterLance) && self.n(Npc::Draconic) {
            v = scale(v, 6, 5);
        }
        if self.m(Melee::DragonHunterWand) && self.n(Npc::Draconic) {
            v = scale(v, 3, 2);
        }
        if self.m(Melee::KerisBreachPartisan) && self.n(Npc::Kalphite) {
            v = scale(v, 133, 100);
        }
        if self.m(Melee::KerisSunPartisan) && self.n(Npc::Amascut) && self.n(Npc::QuarterHealth) {
            v = scale(v, 5, 4);
        }
        self.inquisitor(v)
    }
    pub fn post_special(self, damage: i32, rate: i32, health: i32, max_health: i32) -> i32 {
        let mut v = damage;
        if self.m(Melee::GadderhammerProc) && self.n(Npc::Shade) {
            v = v.wrapping_mul(2);
        } else if self.m(Melee::Gadderhammer) && self.n(Npc::Shade) {
            v = scale(v, 5, 4);
        }
        if self.m(Melee::KerisProc) && self.n(Npc::Kalphite) {
            v = v.wrapping_mul(3);
        }
        if self.m(Melee::Crush) && self.n(Npc::TormentedDemonUnshielded) {
            v = v.wrapping_add(rate.wrapping_mul(rate).wrapping_sub(16).max(0));
        }
        if self.m(Melee::Dharoks) {
            v = v.wrapping_add(scale(
                v,
                max_health.wrapping_sub(health).wrapping_mul(max_health),
                10000,
            ));
        }
        if self.m(Melee::BerserkerNeck) && self.m(Melee::TzHaarWeapon) {
            v = scale(v, 6, 5);
        }
        if self.n(Npc::CorporealBeast) && !self.m(Melee::CorpBaneWeapon) {
            v /= 2;
        }
        v
    }
    pub fn hit_chance(self, chance: i32, attack: i32, defence: i32) -> i32 {
        if !self.m(Melee::OsmumtensFang) || !self.m(Melee::Stab) {
            return chance;
        }
        if self.n(Npc::Amascut) {
            let miss = 10000i32.wrapping_sub(chance);
            10000i32.wrapping_sub(miss.wrapping_mul(miss) / 10000)
        } else {
            crate::combat::hit_chance(attack, defence, true)
        }
    }
    pub fn defence_roll(self, base: i32, invocation: i32) -> i32 {
        if self.n(Npc::Amascut) {
            scale(base, 250i32.wrapping_add(invocation), 250)
        } else {
            base
        }
    }
}
fn scale(value: i32, multiplier: i32, divisor: i32) -> i32 {
    value.wrapping_mul(multiplier) / divisor
}
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Melee {
    Crush,
    Stab,
    AmuletOfAvarice,
    SalveAmulet,
    SalveAmuletE,
    BlackMask,
    Arclight,
    BurningClaws,
    Obsidian,
    TzHaarWeapon,
    DragonHunterLance,
    DragonHunterWand,
    KerisWeapon,
    KerisBreachPartisan,
    KerisSunPartisan,
    BarroniteMaceWeapon,
    ForinthrySurge,
    RevenantWeapon,
    Silverlight,
    LeafBladed,
    ColossalBlade,
    RatBoneWeapon,
    InquisitorHelm,
    InquisitorTop,
    InquisitorBottom,
    InquisitorWeapon,
    OsmumtensFang,
    Dharoks,
    Gadderhammer,
    GadderhammerProc,
    KerisProc,
    BerserkerNeck,
    CorpBaneWeapon,
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Npc {
    SlayerTask,
    Wilderness,
    Revenant,
    Undead,
    Demon,
    Draconic,
    Kalphite,
    Golem,
    Leafy,
    Rat,
    DemonbaneResistance,
    Size2,
    Size3,
    Size4,
    Size5OrMore,
    Shade,
    Vampyre,
    TormentedDemonUnshielded,
    CorporealBeast,
    Xerician,
    Amascut,
    QuarterHealth,
    WindWeakness,
    WaterWeakness,
    EarthWeakness,
    FireWeakness,
}
