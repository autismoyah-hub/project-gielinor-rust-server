//! Resolve WornBonuses from cache parameters and original symbolic equipment groups.
use gielinor_cache::{
    definitions::Param, game::Definitions, parameters::ParameterType, symbols::Symbols,
};
use gielinor_world::{
    bonuses::{self, ItemBonuses},
    inventory::Inventory,
};
use std::collections::BTreeMap;

pub struct Config {
    pub items: BTreeMap<i32, ItemBonuses>,
    pub speeds: BTreeMap<i32, i32>,
    pub attack_style_varp: u16,
}
pub const BANK_COMPONENTS: [&str; 18] = [
    "stabatt",
    "slashatt",
    "crushatt",
    "magicatt",
    "rangeatt",
    "attackspeedbase",
    "attackspeedactual",
    "stabdef",
    "slashdef",
    "crushdef",
    "rangedef",
    "magicdef",
    "meleestrength",
    "rangestrength",
    "magicdamage",
    "prayer",
    "typemultiplier",
    "slayermultiplier",
];
pub struct BankDisplay {
    pub components: [u32; 18],
    pub tooltip: u32,
}
impl BankDisplay {
    pub fn resolve(symbols: &Symbols) -> Result<Self, String> {
        let component = |name: &str| {
            symbols
                .get(&format!("bankmain:{name}"))
                .map(|id| *id as u32)
                .ok_or_else(|| format!("Missing bank component {name}"))
        };
        let components = BANK_COMPONENTS
            .iter()
            .map(|name| component(name))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            components: components.try_into().unwrap(),
            tooltip: component("tooltip")?,
        })
    }
    pub fn packets(
        &self,
        bonuses: &bonuses::Bonuses,
        base_speed: i32,
        actual_speed: i32,
    ) -> Result<Vec<gielinor_protocol::framing::Packet>, gielinor_protocol::buffer::DecodeError>
    {
        use gielinor_protocol::{
            client_script::{self, Argument},
            interfaces,
        };
        let text = bank_text(bonuses, base_speed, actual_speed);
        let mut packets = Vec::new();
        for (i, (&component, text)) in self.components.iter().zip(text).enumerate() {
            packets.push(interfaces::text(component, &text)?);
            if i == 16 {
                packets.push(client_script::run(7065,&[Argument::Int(self.tooltip as i32),Argument::Int(component as i32),Argument::String("Increases your effective accuracy and damage against undead creatures. For multi-target Ranged and Magic attacks, this applies only to the primary target. It does not stack with the Slayer multiplier.".into())])?);
            }
        }
        Ok(packets)
    }
}
pub fn bank_text(b: &bonuses::Bonuses, base: i32, actual: i32) -> [String; 18] {
    let signed = |label: &str, value: i32| format!("{label}: {value:+}");
    let seconds = |speed: i32| {
        let value = speed.wrapping_mul(600) as f64 / 1000.0;
        if value.fract() == 0.0 {
            format!("{value:.1}")
        } else {
            value.to_string()
        }
    };
    let magic = b.displayed_magic_damage() as f64 / 10.0;
    let magic = if magic.abs() >= 10_000_000.0 {
        let scientific = format!("{magic:e}");
        let (mantissa, exponent) = scientific.split_once('e').unwrap();
        format!(
            "{}{}E{}",
            mantissa,
            if mantissa.contains('.') { "" } else { ".0" },
            exponent
        )
    } else if magic.fract() == 0.0 {
        format!("{magic:.1}")
    } else {
        magic.to_string()
    };
    let magic_suffix = if b.magic_additive == 0 {
        String::new()
    } else {
        format!("<col=be66f4> ({}%)</col>", b.magic_additive)
    };
    let undead_suffix = if b.values[14] == 0 {
        " "
    } else if b.undead_melee_only {
        " (melee)"
    } else {
        " (all styles)"
    };
    let slayer_suffix = if b.values[15] == 0 {
        ""
    } else if b.slayer_melee_only {
        " (melee)"
    } else {
        " (all styles)"
    };
    [
        signed("Stab", b.values[0]),
        signed("Slash", b.values[1]),
        signed("Crush", b.values[2]),
        signed("Magic", b.values[3]),
        signed("Range", b.values[4]),
        format!("Base: {}s", seconds(base)),
        format!("Actual: {}s", seconds(actual)),
        signed("Stab", b.values[5]),
        signed("Slash", b.values[6]),
        signed("Crush", b.values[7]),
        signed("Range", b.values[8]),
        signed("Magic", b.values[9]),
        signed("Melee STR", b.values[10]),
        signed("Ranged STR", b.values[11]),
        format!("Magic DMG: +{magic}%{magic_suffix}"),
        signed("Prayer", b.values[13]),
        format!("Undead: +{}%{undead_suffix}", b.values[14] / 10),
        format!("Slayer: +{}%{slayer_suffix}", b.values[15] / 10),
    ]
}
impl Config {
    pub fn calculate(&self, data: &gielinor_world::simulation::PlayerData) -> bonuses::Bonuses {
        bonuses::calculate(
            &data.equipment,
            &self.items,
            data.variables.get(self.attack_style_varp) == 0,
        )
    }
    pub fn base_speed(&self, worn: &Inventory) -> Result<i32, String> {
        match worn.get(3) {
            None => Ok(4),
            Some(item) => self
                .speeds
                .get(&item.id)
                .copied()
                .ok_or_else(|| format!("Missing weapon {}", item.id)),
        }
    }
}
pub struct Styles {
    pub packed: BTreeMap<i32, i32>,
}
impl Styles {
    pub fn resolve(
        symbols: &Symbols,
        enums: &BTreeMap<u32, gielinor_cache::enums::EnumType>,
    ) -> Result<Self, String> {
        let id = *symbols
            .get("weapon_attack_styles")
            .ok_or("Missing weapon attack styles enum")? as u32;
        let table = enums.get(&id).ok_or("Missing weapon attack styles table")?;
        let mut packed = BTreeMap::new();
        for (&key, value) in &table.values {
            let gielinor_cache::enums::Value::Int(value) = value else {
                return Err("Noninteger attack style".into());
            };
            packed.insert(key, *value);
        }
        Ok(Self { packed })
    }
    pub fn get(&self, category: Option<u8>, stance: i32) -> Result<Option<u8>, &'static str> {
        let category = match category {
            Some(id @ (0..=24 | 27 | 28)) => id as i32,
            _ => 0,
        };
        bonuses::attack_style(self.packed.get(&category).copied().unwrap_or(0), stance)
    }
}
pub fn resolve(
    params: &Symbols,
    objects: &Symbols,
    categories: &Symbols,
    varps: &Symbols,
    parameter_types: &BTreeMap<u32, ParameterType>,
    defs: &Definitions,
) -> Result<Config, String> {
    let symbol = |symbols: &Symbols, name: &str| {
        symbols
            .get(name)
            .copied()
            .ok_or_else(|| format!("Missing symbol {name}"))
    };
    let ids: Vec<_> = bonuses::PARAMS
        .iter()
        .map(|name| symbol(params, name))
        .collect::<Result<_, _>>()?;
    let undead = symbol(params, "bonus_undead_meleeonly")?;
    let slayer = symbol(params, "bonus_slayer_meleeonly")?;
    let attack_rate = symbol(params, "attackrate")?;
    let shadow = symbol(objects, "tumekens_shadow")?;
    let charge = symbol(categories, "chargebow")?;
    let thrown = symbol(categories, "throwing_weapon")?;
    let bulwark = symbol(categories, "dinhs_bulwark")?;
    let mut virtus = BTreeMap::new();
    for (slot, names) in [
        (0, ["virtus_mask", "virtus_mask_ornament"]),
        (4, ["virtus_top", "virtus_top_ornament"]),
        (7, ["virtus_legs", "virtus_legs_ornament"]),
    ] {
        for name in names {
            virtus.insert(symbol(objects, name)?, 1u16 << slot);
        }
    }
    let mut void = BTreeMap::new();
    for (slot, names) in [
        (
            0,
            [
                "game_pest_mage_helm",
                "game_pest_mage_helm_trouver",
                "league_3_void_mage_helm",
                "league_3_void_mage_helm_trouver",
            ],
        ),
        (
            4,
            [
                "elite_void_knight_top",
                "elite_void_knight_top_trouver",
                "league_3_void_knight_top_elite",
                "league_3_void_knight_top_elite_trouver",
            ],
        ),
        (
            7,
            [
                "elite_void_knight_robes",
                "elite_void_knight_robes_trouver",
                "league_3_void_knight_robes_elite",
                "league_3_void_knight_robes_elite_trouver",
            ],
        ),
        (
            9,
            [
                "pest_void_knight_gloves",
                "pest_void_knight_gloves_trouver",
                "league_3_void_knight_gloves",
                "league_3_void_knight_gloves_trouver",
            ],
        ),
    ] {
        for name in names {
            void.insert(symbol(objects, name)?, 1u16 << slot);
        }
    }
    let mut items = BTreeMap::new();
    let mut speeds = BTreeMap::new();
    for (&id, item) in &defs.items {
        let int = |key: i32| -> Result<i32, String> {
            match item.params.get(&(key as u32)) {
                Some(Param::Integer(value)) => Ok(*value),
                Some(_) => Err(format!("Noninteger bonus parameter {key} on {id}")),
                None => parameter_types
                    .get(&(key as u32))
                    .and_then(|p| p.default_int)
                    .ok_or_else(|| format!("Missing parameter default {key}")),
            }
        };
        let mut values = [0; 17];
        for (value, &key) in values.iter_mut().zip(&ids) {
            *value = int(key)?;
        }
        items.insert(
            id as i32,
            ItemBonuses {
                values,
                undead_melee_only: int(undead)? != 0,
                slayer_melee_only: int(slayer)? != 0,
                ignore_quiver: item.category == charge || item.category == thrown,
                bulwark: item.category == bulwark,
                shadow: id as i32 == shadow,
                virtus_slots: virtus.get(&(id as i32)).copied().unwrap_or(0),
                elite_void_slots: void.get(&(id as i32)).copied().unwrap_or(0),
            },
        );
        speeds.insert(id as i32, int(attack_rate)?);
    }
    let attack_style_varp =
        u16::try_from(symbol(varps, "com_mode")?).map_err(|_| "Invalid attack style varp")?;
    if !defs.varps.contains_key(&(attack_style_varp as u32)) {
        return Err("Missing attack style varp".into());
    }
    Ok(Config {
        items,
        speeds,
        attack_style_varp,
    })
}
