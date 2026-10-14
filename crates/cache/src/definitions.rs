//! Gameplay definitions from the enriched server cache, including custom opcodes.
use gielinor_protocol::buffer::{DecodeError, Reader};
use gielinor_world::inventory::{Catalog, Inventory, StackType, Template};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Param {
    Integer(i32),
    String(String),
}
pub fn params(r: &mut Reader<'_>) -> Result<BTreeMap<u32, Param>, DecodeError> {
    let count = r.u8()?;
    let mut values = BTreeMap::new();
    for _ in 0..count {
        let string = r.u8()? == 1;
        let id = r.u24()?;
        let value = if string {
            Param::String(r.string()?)
        } else {
            Param::Integer(r.u32()? as i32)
        };
        values.insert(id, value);
    }
    Ok(values)
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stock {
    pub id: u16,
    pub count: i32,
    pub restock_cycles: u16,
}
#[derive(Debug, Clone)]
pub struct InventoryType {
    pub size: usize,
    pub scope: u8,
    pub stack: StackType,
    pub flags: u32,
    pub stock: Vec<Option<Stock>>,
}
impl InventoryType {
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(bytes);
        let mut def = Self {
            size: 1,
            scope: 0,
            stack: StackType::Normal,
            flags: 1,
            stock: vec![],
        };
        while r.remaining() > 0 {
            match r.u8()? {
                0 => break,
                2 | 203 => def.size = r.u16()? as usize,
                200 => {
                    def.scope = r.u8()?;
                    if def.scope > 2 {
                        return Err(DecodeError::InvalidValue("inventory scope"));
                    }
                }
                201 => {
                    def.stack = match r.u8()? {
                        0 => StackType::Normal,
                        1 => StackType::Always,
                        2 => StackType::Never,
                        _ => return Err(DecodeError::InvalidValue("inventory stack mode")),
                    }
                }
                202 => def.size = r.u8()? as usize,
                204 => def.flags = r.u32()?,
                205 => {
                    let count = r.u8()? as usize + 1;
                    def.stock = Vec::with_capacity(count);
                    for _ in 0..count {
                        let header = r.u8()?;
                        if header == 0 {
                            def.stock.push(None);
                            continue;
                        }
                        let count = if header == 255 {
                            r.u32()? as i32
                        } else {
                            header as i32 - 1
                        };
                        def.stock.push(Some(Stock {
                            id: r.u16()?,
                            count,
                            restock_cycles: r.u16()?,
                        }));
                    }
                }
                _ => return Err(DecodeError::InvalidValue("unknown inventory opcode")),
            }
        }
        Ok(def)
    }
    pub fn create(&self) -> Inventory {
        Inventory::new(self.size, self.stack, self.flags & 0x20 != 0)
    }
}
#[derive(Debug, Clone)]
pub struct ItemType {
    pub name: String,
    pub description: String,
    pub stackable: bool,
    pub cost: i32,
    pub wear: [i8; 3],
    pub tradeable: bool,
    pub members: bool,
    pub weight: i16,
    pub ground_ops: [Option<String>; 5],
    pub inventory_ops: [Option<String>; 5],
    pub sub_ops: BTreeMap<(u8, u8), String>,
    pub sub_ops_present: [bool; 5],
    pub shift_click: i8,
    pub cert: Template,
    pub placeholder: Template,
    pub transform: Template,
    pub bought: Template,
    pub dummy: i8,
    pub respawn_rate: u16,
    pub category: i32,
    pub content_group: i32,
    pub weapon_category: u8,
    pub team: i8,
    pub stockmarket: bool,
    pub player_cost: i32,
    pub player_cost_derived: i32,
    pub player_cost_constant: i32,
    pub buy_limit: u16,
    pub recalc_users: u16,
    pub objvars: Vec<u16>,
    /// A null Kotlin ParamMap differs from an encoded empty map for defaults.
    pub params_present: bool,
    pub params: BTreeMap<u32, Param>,
    /// Preserve every encoded field for later appearance/model consumers.
    pub encoded: Vec<u8>,
}
impl ItemType {
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let empty = Template {
            link: 0,
            template: 0,
        };
        let mut d = Self {
            name: "null".into(),
            description: String::new(),
            stackable: false,
            cost: 1,
            wear: [-1; 3],
            tradeable: true,
            members: false,
            weight: 0,
            ground_ops: std::array::from_fn(|_| None),
            inventory_ops: std::array::from_fn(|_| None),
            sub_ops: BTreeMap::new(),
            sub_ops_present: [false; 5],
            shift_click: -1,
            cert: empty,
            placeholder: empty,
            transform: empty,
            bought: empty,
            dummy: -1,
            respawn_rate: 100,
            category: -1,
            content_group: -1,
            weapon_category: 0,
            team: 0,
            stockmarket: false,
            player_cost: 0,
            player_cost_derived: 0,
            player_cost_constant: 0,
            buy_limit: 0,
            recalc_users: 0,
            objvars: vec![],
            params_present: false,
            params: BTreeMap::new(),
            encoded: bytes.to_vec(),
        };
        let mut r = Reader::new(bytes);
        while r.remaining() > 0 {
            let code = r.u8()?;
            match code {
                0 => break,
                2 => d.name = r.string()?,
                3 => d.description = r.string()?,
                9 => {
                    r.string()?;
                }
                11 | 160 => d.stackable = true,
                12 => d.cost = r.u32()? as i32,
                13 => d.wear[0] = r.u8()? as i8,
                14 => d.wear[1] = r.u8()? as i8,
                27 => d.wear[2] = r.u8()? as i8,
                15 | 206 => d.tradeable = false,
                16 => d.members = true,
                30..=34 => d.ground_ops[(code - 30) as usize] = Some(r.string()?),
                35..=39 => d.inventory_ops[(code - 35) as usize] = Some(r.string()?),
                40 | 41 => {
                    let count = r.u8()? as usize;
                    r.take(count * 4)?;
                }
                42 => d.shift_click = r.u8()? as i8,
                43 => {
                    let parent = r.u8()?;
                    if parent >= 5 {
                        return Err(DecodeError::InvalidValue("item sub-operation parent"));
                    }
                    d.sub_ops_present[parent as usize] = true;
                    d.sub_ops.retain(|(op, _), _| *op != parent);
                    loop {
                        let sub = r.u8()?;
                        if sub == 0 {
                            break;
                        }
                        if sub > 20 {
                            return Err(DecodeError::InvalidValue("item sub-operation index"));
                        }
                        d.sub_ops.insert((parent, sub - 1), r.string()?);
                    }
                }
                1 | 4..=8 | 24 | 26 | 78 | 79 | 90..=93 | 95 | 110..=112 => {
                    r.u16()?;
                }
                23 | 25 => {
                    r.take(3)?;
                }
                44 | 46 | 47 | 49..=54 => {
                    r.u32()?;
                }
                45 | 48 => {
                    r.take(5)?;
                }
                65 => d.stockmarket = true,
                75 => d.weight = r.u16()? as i16,
                94 => d.category = r.u16()? as i32,
                97 => d.cert.link = r.u16()? as i32,
                98 => d.cert.template = r.u16()? as i32,
                100..=109 => {
                    r.take(4)?;
                }
                113 | 114 => {
                    r.u8()?;
                }
                115 => d.team = r.u8()? as i8,
                139 => d.bought.link = r.u16()? as i32,
                140 => d.bought.template = r.u16()? as i32,
                148 => d.placeholder.link = r.u16()? as i32,
                149 => d.placeholder.template = r.u16()? as i32,
                200 => {
                    let count = r.u8()?;
                    d.objvars.clear();
                    for _ in 0..count {
                        d.objvars.push(r.u16()?);
                    }
                }
                201 => d.player_cost = r.u32()? as i32,
                202 => d.player_cost_derived = r.u32()? as i32,
                203 => d.player_cost_constant = r.u32()? as i32,
                204 => d.buy_limit = r.u16()?,
                205 => d.recalc_users = r.u16()?,
                207 => d.respawn_rate = r.u16()?,
                208 => d.dummy = r.u8()? as i8,
                209 => d.content_group = r.u16()? as i32,
                210 => d.transform.link = r.u16()? as i32,
                211 => d.transform.template = r.u16()? as i32,
                212 => d.weapon_category = r.u8()?,
                249 => {
                    d.params_present = true;
                    d.params = params(&mut r)?;
                }
                _ => return Err(DecodeError::InvalidValue("unknown item opcode")),
            }
        }
        Ok(d)
    }
    pub fn register(&self, id: i32, catalog: &mut Catalog) {
        if self.stackable {
            catalog.stackable.insert(id);
        } else {
            catalog.stackable.remove(&id);
        }
        for (template, lookup) in [
            (self.cert, &mut catalog.notes),
            (self.placeholder, &mut catalog.placeholders),
            (self.transform, &mut catalog.transforms),
        ] {
            if template.link != 0 {
                lookup.insert(id, template);
            } else {
                lookup.remove(&id);
            }
        }
        if self.dummy == 1 {
            catalog.dummy.insert(id);
        } else {
            catalog.dummy.remove(&id);
        }
    }
}
#[derive(Debug, Clone)]
pub struct StatType {
    pub name: String,
    pub unreleased: bool,
    pub max_level: u8,
    pub min_level: u8,
}
impl StatType {
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(bytes);
        let mut d = Self {
            name: String::new(),
            unreleased: false,
            max_level: 99,
            min_level: 1,
        };
        while r.remaining() > 0 {
            match r.u8()? {
                0 => break,
                1 => d.name = r.string()?,
                2 => d.unreleased = true,
                3 => d.max_level = r.u8()?,
                4 => d.min_level = r.u8()?,
                _ => return Err(DecodeError::InvalidValue("unknown stat opcode")),
            }
        }
        Ok(d)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn custom_inventory_stock_and_flags() {
        let d = InventoryType::decode(&[
            203, 0, 28, 200, 1, 201, 1, 204, 0, 0, 0, 33, 205, 1, 0, 3, 3, 227, 0, 50, 0,
        ])
        .unwrap();
        assert_eq!(d.size, 28);
        assert_eq!(d.scope, 1);
        assert_eq!(d.stack, StackType::Always);
        assert!(d.create().placeholders);
        assert_eq!(
            d.stock,
            [
                None,
                Some(Stock {
                    id: 995,
                    count: 2,
                    restock_cycles: 50
                })
            ]
        );
    }
    #[test]
    fn absent_and_empty_item_params_preserve_kotlin_nullable_map() {
        let absent = ItemType::decode(&[0]).unwrap();
        let empty = ItemType::decode(&[249, 0, 0]).unwrap();
        assert_eq!(absent.params, empty.params);
        assert!(!absent.params_present);
        assert!(empty.params_present);
    }
    #[test]
    fn item_templates_signed_fields_and_params() {
        let d = ItemType::decode(&[
            2, b'C', b'o', b'i', b'n', 0, 11, 75, 255, 250, 208, 0, 97, 4, 15, 249, 1, 0, 0, 0, 7,
            255, 255, 255, 255, 0,
        ])
        .unwrap();
        assert_eq!(d.name, "Coin");
        assert_eq!(d.weight, -6);
        assert_eq!(d.params[&7], Param::Integer(-1));
        let mut c = Catalog::default();
        d.register(995, &mut c);
        assert!(c.stackable.contains(&995));
        assert_eq!(c.notes[&995].link, 1039);
        assert!(!c.dummy.contains(&995));
    }
}
