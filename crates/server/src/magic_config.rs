//! Cache-backed rune validation and weapon-swap autocast restoration.
use gielinor_cache::{
    definitions::{ItemType, Param},
    enums::{EnumType, Value},
    game::Definitions,
    parameters::ParameterType,
    symbols::Symbols,
};
use gielinor_world::{autocast, bank_actions::Binding, combat_wear, magic, runes::Requirement};
use std::collections::{BTreeMap, BTreeSet};
pub struct Inputs<'a> {
    pub objs: &'a Symbols,
    pub params: &'a Symbols,
    pub bits: &'a Symbols,
    pub enums: &'a Symbols,
    pub categories: &'a Symbols,
    pub parameter_types: &'a BTreeMap<u32, ParameterType>,
    pub tables: &'a BTreeMap<u32, EnumType>,
    pub defs: &'a Definitions,
}
impl Inputs<'_> {
    pub fn spell_buttons(
        &self,
        spells: impl IntoIterator<Item = i32>,
    ) -> Result<BTreeMap<u32, i32>, String> {
        let mut buttons = BTreeMap::new();
        for id in spells {
            let item = self
                .defs
                .items
                .get(&(id as u32))
                .ok_or("Missing spell button item")?;
            let button = self
                .int(item, "spell_button")?
                .ok_or("Missing spell button parameter")?;
            let button = u32::try_from(button).map_err(|_| "Invalid spell component")?;
            if buttons.insert(button, id).is_some() {
                return Err(format!("Duplicate spell component {button}"));
            }
        }
        Ok(buttons)
    }
    /// Registry metadata for combat spells; indexed by object id rather than
    /// the client autocast selection id used by the picker.
    pub fn combat_spells(
        &self,
    ) -> Result<BTreeMap<i32, gielinor_world::spell_cast::Definition>, String> {
        self.autocast()?
            .spells
            .into_values()
            .map(|spell| {
                let item = self
                    .defs
                    .items
                    .get(&(spell.item as u32))
                    .ok_or("Missing combat spell object")?;
                Ok((
                    spell.item,
                    gielinor_world::spell_cast::Definition {
                        book: self
                            .int(item, "spell_spellbook")?
                            .filter(|id| (0..=3).contains(id)),
                        level: self
                            .int(item, "spell_levelreq")?
                            .ok_or("Missing combat spell level")?,
                        cast_xp: self
                            .int(item, "spell_castxp")?
                            .ok_or("Missing combat spell XP")?
                            as f64
                            / 10.0,
                        spell,
                    },
                ))
            })
            .collect()
    }
    pub fn selection(
        &self,
        magic_stat: u8,
    ) -> Result<gielinor_world::autocast_select::Config, String> {
        let mut spells = BTreeMap::new();
        for (key, id) in self.integers(self.table("autocast_spells")?)? {
            let Some(item) = self.defs.items.get(&(id as u32)) else {
                continue;
            };
            let book = self
                .int(item, "spell_spellbook")?
                .filter(|id| (0..=3).contains(id));
            let level = self
                .int(item, "spell_levelreq")?
                .ok_or_else(|| format!("Missing spell level {id}"))?;
            spells.insert(key, gielinor_world::autocast_select::Access { book, level });
        }
        Ok(gielinor_world::autocast_select::Config {
            book: self.binding("spellbook")?,
            magic_stat,
            spells,
        })
    }
    fn symbol(&self, map: &Symbols, name: &str) -> Result<i32, String> {
        map.get(name)
            .copied()
            .ok_or_else(|| format!("Missing magic symbol {name}"))
    }
    fn obj(&self, name: &str) -> Result<i32, String> {
        self.symbol(self.objs, name)
    }
    fn binding(&self, name: &str) -> Result<Binding, String> {
        let id = self.symbol(self.bits, name)? as u32;
        let bit = *self
            .defs
            .varbits
            .get(&id)
            .ok_or_else(|| format!("Missing magic varbit {name}"))?;
        let base = *self
            .defs
            .varps
            .get(&(bit.base as u32))
            .ok_or_else(|| format!("Missing magic base {}", bit.base))?;
        Ok(Binding { base, bit })
    }
    fn int(&self, item: &ItemType, name: &str) -> Result<Option<i32>, String> {
        let id = self.symbol(self.params, name)? as u32;
        match item.params.get(&id) {
            Some(Param::Integer(value)) => Ok(Some(*value)),
            Some(_) => Err(format!("Noninteger magic parameter {name}")),
            None => Ok(self.parameter_types.get(&id).and_then(|p| p.default_int)),
        }
    }
    fn string(&self, item: &ItemType, name: &str) -> Result<Option<String>, String> {
        let id = self.symbol(self.params, name)? as u32;
        match item.params.get(&id) {
            Some(Param::String(value)) => Ok(Some(value.clone())),
            Some(_) => Err(format!("Nonstring magic parameter {name}")),
            None => Ok(self
                .parameter_types
                .get(&id)
                .and_then(|p| p.default_string.clone())),
        }
    }
    fn table(&self, name: &str) -> Result<&EnumType, String> {
        self.tables
            .get(&(self.symbol(self.enums, name)? as u32))
            .ok_or_else(|| format!("Missing magic enum {name}"))
    }
    fn integers(&self, table: &EnumType) -> Result<Vec<(i32, i32)>, String> {
        table
            .values
            .iter()
            .map(|(&k, v)| match v {
                Value::Int(v) => Ok((k, *v)),
                _ => Err("Noninteger magic enum".into()),
            })
            .collect()
    }
    pub fn resolve(&self, combat_stats: [u8; 7]) -> Result<combat_wear::Config, String> {
        Ok(combat_wear::Config {
            autocast: self.autocast()?,
            runes: self.runes()?,
            level: self.binding("combatlevel_transmit")?,
            decimal: self.binding("combatlevel_decimal_transmit")?,
            combat_stats,
        })
    }
    pub fn bulwark(
        &self,
        varps: &Symbols,
        spots: &Symbols,
    ) -> Result<gielinor_world::bulwark_wear::Config, String> {
        let category = self.symbol(self.categories, "dinhs_bulwark")?;
        let passive_delay = *self
            .defs
            .varps
            .get(&(self.symbol(varps, "dinhs_passive_delay")? as u32))
            .ok_or("Missing bulwark passive delay")?;
        let glow = u16::try_from(self.symbol(spots, "dinhs_bulwark_glow_idle")?)
            .map_err(|_| "Invalid bulwark glow")?;
        Ok(gielinor_world::bulwark_wear::Config {
            items: self
                .defs
                .items
                .iter()
                .filter(|(_, item)| item.category == category)
                .map(|(&id, _)| id as i32)
                .collect(),
            passive_delay,
            glow,
        })
    }
    pub fn wearpos(
        &self,
        varps: &Symbols,
        synths: &Symbols,
        combat_stats: [u8; 7],
    ) -> Result<gielinor_world::wearpos::Config, String> {
        let varp = |name: &str| {
            self.defs
                .varps
                .get(&(self.symbol(varps, name)? as u32))
                .copied()
                .ok_or_else(|| format!("Missing wearpos varp {name}"))
        };
        let default_sound = self.symbol(synths, "default_equipment")?;
        // BaseObjs.snowball is a source alias for the cache's gublinch_snowball.
        let snowball = self.obj("gublinch_snowball")?;
        let items = self
            .defs
            .items
            .iter()
            .map(|(&id, item)| {
                let sound = self.int(item, "equipment_sound")?.unwrap_or(default_sound);
                let has_params =
                    item.params_present || !item.params.is_empty() || id as i32 == snowball;
                let sound = if !has_params {
                    None
                } else {
                    Some(if sound == -1 {
                        u16::MAX
                    } else {
                        u16::try_from(sound).map_err(|_| "Invalid equipment sound")?
                    })
                };
                // ObjEdits and ParamBuilds are source-side additions to cache data.
                let player_op = if id as i32 == snowball {
                    Some("Pelt".to_owned())
                } else {
                    self.string(item, "player_op5_text")?
                };
                Ok((
                    id as i32,
                    gielinor_world::wearpos::Item {
                        two_handed: item.wear[1..].contains(&5),
                        category: item.weapon_category,
                        sound,
                        player_op,
                    },
                ))
            })
            .collect::<Result<_, String>>()?;
        Ok(gielinor_world::wearpos::Config {
            special: varp("sa_attack")?,
            stance: varp("com_mode")?,
            category: self.binding("combat_weapon_category")?,
            level: self.binding("combatlevel_transmit")?,
            decimal: self.binding("combatlevel_decimal_transmit")?,
            combat_stats,
            items,
        })
    }
    fn autocast(&self) -> Result<autocast::Config, String> {
        let autocast_items: BTreeMap<_, _> = self
            .integers(self.table("autocast_spells")?)?
            .into_iter()
            .filter(|(_, id)| self.defs.items.contains_key(&(*id as u32)))
            .collect();
        let mut registered = BTreeSet::new();
        for (_, book) in self.integers(self.table("spellbooks")?)? {
            if let Some(table) = self.tables.get(&(book as u32)) {
                for (_, item) in self.integers(table)? {
                    registered.insert(item);
                }
            }
        }
        let mut spells = BTreeMap::new();
        for (&autocast, &id) in &autocast_items {
            let ty = &self.defs.items[&(id as u32)];
            if !registered.contains(&id) || self.int(ty, "spell_castxp")?.is_none() {
                return Err(format!(
                    "Autocast {autocast} references unregistered spell {id}"
                ));
            }
            let mut requirements = Vec::new();
            for n in 1..=4 {
                let Some(mut item) = self.int(ty, &format!("spell_runetype_{n}"))? else {
                    continue;
                };
                // Nullable Obj parameters resolve absent or invalid object IDs to null.
                if !self.defs.items.contains_key(&(item as u32)) {
                    continue;
                }
                if item == self.obj("pest_interface_staffs")? {
                    item = self.obj("guthix_staff")?;
                }
                let count = self
                    .int(ty, &format!("spell_runecount_{n}"))?
                    .ok_or_else(|| format!("Missing spell rune count {id}/{n}"))?;
                let worn_slot = usize::try_from(self.defs.items[&(item as u32)].wear[0]).ok();
                if worn_slot.is_some() && count != 1 {
                    return Err(format!("Worn requirement count is not one for spell {id}"));
                }
                requirements.push(Requirement {
                    item,
                    count,
                    worn_slot,
                });
            }
            if id == self.obj("50_magic_dart")? && !requirements.is_empty() {
                requirements.rotate_left(1);
            }
            spells.insert(
                autocast,
                autocast::Spell {
                    item: id,
                    requirements,
                },
            );
        }
        let mut weapons = BTreeMap::new();
        for (&id, ty) in &self.defs.items {
            let mut additional = BTreeSet::new();
            for n in 1..=3 {
                if let Some(item) = self.int(ty, &format!("additional_spell_autocast{n}"))? {
                    if self.defs.items.contains_key(&(item as u32)) {
                        additional.insert(item);
                    }
                }
            }
            weapons.insert(
                id as i32,
                autocast::Weapon {
                    category: ty.weapon_category,
                    additional,
                },
            );
        }
        let saved = |suffix: &str| -> Result<autocast::Saved, String> {
            Ok(autocast::Saved {
                spell: self.binding(&format!("saved_autocast_spell_{suffix}"))?,
                defensive: self.binding(&format!("saved_defensive_casting_{suffix}"))?,
            })
        };
        Ok(autocast::Config {
            enabled: self.binding("autocast_set")?,
            spell: self.binding("autocast_spell")?,
            defensive: self.binding("autocast_defmode")?,
            staff: saved("staff")?,
            bladed: saved("bladed_staff")?,
            weapons,
            spells,
            autocast_items,
            restricted: self
                .integers(self.table("autocast_restricted_spells")?)?
                .into_iter()
                .filter(|(id, _)| self.defs.items.contains_key(&(*id as u32)))
                .map(|(id, _)| id)
                .collect(),
        })
    }
    fn runes(&self) -> Result<magic::Config, String> {
        let catalog = crate::rune_catalog::resolve(self.enums, self.tables, self.defs)?;
        let mut packs = BTreeMap::new();
        for (spells, lms, sack) in [
            (&["80_blood_blitz", "92_blood_barrage"][..], true, None),
            (
                &["82_ice_blitz", "94_ice_barrage"][..],
                true,
                Some("blighted_sack_icebarrage"),
            ),
            (
                &["58_ice_rush", "70_ice_burst"][..],
                false,
                Some("blighted_sack_icebarrage"),
            ),
            (
                &["20_bind", "50_snare"][..],
                false,
                Some("blighted_sack_entangle"),
            ),
            (&["79_entangle"][..], true, Some("blighted_sack_entangle")),
            (
                &["85_teleport_block", "placeholder_caveorb4dot"][..],
                false,
                Some("blighted_sack_teleblock"),
            ),
            (&["71_cure_me"][..], true, None),
            (
                &["93_vengeance_other"][..],
                false,
                Some("blighted_sack_vengeance"),
            ),
            (&["94_vengeance"][..], true, Some("blighted_sack_vengeance")),
            (
                &[
                    "62_wind_wave",
                    "65_water_wave",
                    "70_earth_wave",
                    "75_fire_wave",
                ][..],
                false,
                Some("blighted_sack_surge"),
            ),
            (
                &[
                    "81_wind_surge",
                    "85_water_surge",
                    "90_earth_surge",
                    "95_fire_surge",
                ][..],
                true,
                Some("blighted_sack_surge"),
            ),
        ] {
            for spell in spells {
                packs.insert(
                    self.obj(spell)?,
                    magic::Pack {
                        lms,
                        blighted: sack.map(|s| self.obj(s)).transpose()?,
                    },
                );
            }
        }
        let mut names = magic::Names::default();
        let rune_category = self.symbol(self.categories, "rune")?;
        let special=[
            (self.obj("ibanstaff")?,"You must wield Iban's Staff to cast this spell."),
            (self.obj("saradomin_staff")?,"You must be wielding the Staff of Saradomin or the Staff of Light to cast this spell."),
            (self.obj("zamorak_staff")?,"You must be wielding the Staff of Zamorak, Staff of the Dead, Thammaron's Sceptre, or the Accursed Sceptre to cast this spell."),
            (self.obj("guthix_staff")?,"You must wield the Staff of Guthix, Staff of Balance or the Void Knight Mace to cast this spell."),
            (self.obj("slayer_staff")?,"You need to be wielding a suitable staff to cast this spell."),
        ];
        for (&id, ty) in &self.defs.items {
            let name = if ty.category == rune_category {
                format!(
                    "{} Runes",
                    ty.name
                        .chars()
                        .take(ty.name.chars().count().saturating_sub(5))
                        .collect::<String>()
                )
            } else {
                if ty.name.ends_with('s') {
                    ty.name.clone()
                } else {
                    format!("{}s", ty.name)
                }
                .to_lowercase()
            };
            names.missing.insert(
                id as i32,
                format!("You do not have enough {name} to cast this spell."),
            );
            let worn = self
                .string(ty, "spell_worn_req_message")?
                .or_else(|| {
                    special
                        .iter()
                        .find(|(special, _)| *special == id as i32)
                        .map(|(_, message)| (*message).to_owned())
                })
                .unwrap_or_else(|| format!("You need a {} to cast this spell.", ty.name));
            names.not_wearing.insert(id as i32, worn);
        }
        let four = |prefix: &str| -> Result<[Binding; 4], String> {
            let values: Vec<_> = (1..=4)
                .map(|n| self.binding(&format!("{prefix}_{n}")))
                .collect::<Result<_, _>>()?;
            Ok(values.try_into().unwrap())
        };
        let counts = four("rune_pouch_quantity")?;
        let mut pouch_counts = [(0, counts[0]); 4];
        for n in 0..4 {
            pouch_counts[n] = (
                self.symbol(self.bits, &format!("rune_pouch_quantity_{}", n + 1))? as u32,
                counts[n],
            );
        }
        Ok(magic::Config {
            catalog,
            fake: [
                self.binding("barbassault_areaexit_pending")?,
                self.binding("br_ingame")?,
            ],
            fountain: self.binding("fountain_of_rune_active")?,
            blighted: self.binding("blighted_items_allowed")?,
            regular_pouches: [
                self.obj("bh_rune_pouch")?,
                self.obj("bh_rune_pouch_trouver")?,
            ]
            .into(),
            divine_pouches: [
                self.obj("divine_rune_pouch")?,
                self.obj("divine_rune_pouch_trouver")?,
            ]
            .into(),
            pouch_types: four("rune_pouch_type")?,
            pouch_counts,
            lms_pouch: self.obj("br_rune_replacement")?,
            packs,
            names,
        })
    }
}

#[cfg(test)]
mod wearpos_tests {
    use super::*;
    use gielinor_world::variables::{Transmit, Varbit, Varp};
    #[test]
    fn source_defaults_keep_null_maps_and_snowball_operation_distinct() {
        let varp = |id| Varp {
            id,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        };
        let defs = Definitions {
            inventories: BTreeMap::new(),
            locations: BTreeMap::new(),
            npcs: BTreeMap::new(),
            stats: BTreeMap::new(),
            items: BTreeMap::from([
                (1, ItemType::decode(&[0]).unwrap()),
                (2, ItemType::decode(&[249, 0, 0]).unwrap()),
                (3, ItemType::decode(&[0]).unwrap()),
            ]),
            varps: (0..5).map(|id| (id as u32, varp(id))).collect(),
            varbits: (0..3)
                .map(|id| {
                    (
                        id as u32,
                        Varbit {
                            base: id + 2,
                            lsb: 0,
                            msb: 7,
                        },
                    )
                })
                .collect(),
        };
        let objs = Symbols::from([("gublinch_snowball".into(), 3)]);
        let params = Symbols::from([
            ("equipment_sound".into(), 50),
            ("player_op5_text".into(), 51),
        ]);
        let bits = Symbols::from([
            ("combat_weapon_category".into(), 0),
            ("combatlevel_transmit".into(), 1),
            ("combatlevel_decimal_transmit".into(), 2),
        ]);
        let empty = Symbols::new();
        let parameters = BTreeMap::new();
        let tables = BTreeMap::new();
        let config = Inputs {
            objs: &objs,
            params: &params,
            bits: &bits,
            enums: &empty,
            categories: &empty,
            parameter_types: &parameters,
            tables: &tables,
            defs: &defs,
        }
        .wearpos(
            &Symbols::from([("sa_attack".into(), 0), ("com_mode".into(), 1)]),
            &Symbols::from([("default_equipment".into(), 2238)]),
            [0, 1, 2, 3, 4, 6, 5],
        )
        .unwrap();
        assert_eq!(config.items[&1].sound, None);
        assert_eq!(config.items[&2].sound, Some(2238));
        assert_eq!(config.items[&2].player_op, None);
        assert_eq!(config.items[&3].sound, Some(2238));
        assert_eq!(config.items[&3].player_op.as_deref(), Some("Pelt"));
    }
}
