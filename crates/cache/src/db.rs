//! DbRowTypeDecoder/DbTableTypeDecoder and DbRow's explicit-value/default lookup.
//! Used by cache-backed gameplay catalogs, including music unlocks.
use gielinor_protocol::buffer::{DecodeError, Reader};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Int(i32),
    String(String),
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Row {
    pub table: i32,
    pub column_count: u8,
    pub types: BTreeMap<u8, Vec<u16>>,
    pub data: BTreeMap<u8, Vec<Value>>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Table {
    pub column_count: u8,
    pub types: BTreeMap<u8, Vec<u16>>,
    pub defaults: BTreeMap<u8, Vec<Value>>,
    pub attributes: BTreeMap<u8, u16>,
}
fn types(r: &mut Reader<'_>) -> Result<Vec<u16>, DecodeError> {
    let count = r.u8()?;
    (0..count).map(|_| r.smart()).collect()
}
fn values(r: &mut Reader<'_>, types: &[u16]) -> Result<Vec<Value>, DecodeError> {
    let count = r.smart()? as usize;
    // Every tuple member needs at least one byte. Reject before allocating.
    if count * types.len() > r.remaining() {
        return Err(DecodeError::Truncated);
    }
    let mut result = Vec::with_capacity(count * types.len());
    for _ in 0..count {
        for ty in types {
            result.push(if *ty == 36 {
                Value::String(r.string()?)
            } else {
                Value::Int(r.u32()? as i32)
            });
        }
    }
    Ok(result)
}
fn var_int(r: &mut Reader<'_>) -> Result<i32, DecodeError> {
    let mut value = 0u32;
    for _ in 0..5 {
        let next = r.u8()?;
        value = (value << 7) | (next & 127) as u32;
        if next & 128 == 0 {
            return Ok(value as i32);
        }
    }
    Err(DecodeError::InvalidValue(
        "db table varint exceeds five bytes",
    ))
}
impl Row {
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(bytes);
        let mut result = Self::default();
        let mut has_table = false;
        while r.remaining() > 0 {
            match r.u8()? {
                0 => break,
                3 => {
                    result.column_count = r.u8()?;
                    result.types.clear();
                    result.data.clear();
                    loop {
                        let column = r.u8()?;
                        if column == 255 {
                            break;
                        }
                        let types = types(&mut r)?;
                        result.data.insert(column, values(&mut r, &types)?);
                        result.types.insert(column, types);
                    }
                }
                4 => {
                    result.table = var_int(&mut r)?;
                    has_table = true;
                }
                _ => return Err(DecodeError::InvalidValue("unknown dbrow opcode")),
            }
        }
        if !has_table {
            return Err(DecodeError::InvalidValue("dbrow table missing"));
        }
        Ok(result)
    }
    pub fn column<'a>(&'a self, table: &'a Table, column: u8) -> Option<&'a [Value]> {
        self.data
            .get(&column)
            .or_else(|| table.defaults.get(&column))
            .map(Vec::as_slice)
    }
}
impl Table {
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(bytes);
        let mut result = Self::default();
        while r.remaining() > 0 {
            match r.u8()? {
                0 => break,
                1 => {
                    result.column_count = r.u8()?;
                    result.types.clear();
                    result.defaults.clear();
                    result.attributes.clear();
                    loop {
                        let attr = r.u8()?;
                        if attr == 255 {
                            break;
                        }
                        let column = attr & 127;
                        let types = types(&mut r)?;
                        if attr & 128 != 0 {
                            result.defaults.insert(column, values(&mut r, &types)?);
                        }
                        result.types.insert(column, types);
                        result.attributes.insert(column, attr as u16 | 256);
                    }
                }
                200 => {
                    result.attributes.clear();
                    loop {
                        let attr = r.u16()?;
                        if attr == 65535 {
                            break;
                        }
                        result.attributes.insert((attr & 127) as u8, attr);
                    }
                }
                _ => return Err(DecodeError::InvalidValue("unknown dbtable opcode")),
            }
        }
        Ok(result)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tuples_preserve_types_order_and_explicit_empty_overrides_defaults() {
        let row = Row::decode(&[
            4, 0x81, 0x02, 3, 2, 0, 2, 36, 0, 2, b'A', 0, 0, 0, 0, 1, b'B', 0, 255, 255, 255, 255,
            1, 1, 0, 0, 255, 0,
        ])
        .unwrap();
        assert_eq!(row.table, 130);
        assert_eq!(
            row.data[&0],
            vec![
                Value::String("A".into()),
                Value::Int(1),
                Value::String("B".into()),
                Value::Int(-1)
            ]
        );
        let table = Table::decode(&[1, 2, 129, 1, 0, 1, 0, 0, 0, 7, 255, 0]).unwrap();
        assert_eq!(table.attributes[&1], 385);
        assert_eq!(row.column(&table, 1), Some(&[][..]));
        assert_eq!(Row::default().column(&table, 1), Some(&[Value::Int(7)][..]));
    }
    #[test]
    fn repeated_blocks_replace_prior_maps_and_server_attributes() {
        let table = Table::decode(&[1, 1, 0, 1, 0, 255, 200, 2, 0, 255, 255, 0]).unwrap();
        assert_eq!(table.attributes[&0], 512);
        let row = Row::decode(&[4, 1, 3, 1, 0, 1, 0, 1, 0, 0, 0, 4, 255, 3, 0, 255, 0]).unwrap();
        assert!(row.data.is_empty());
        assert!(Row::decode(&[0]).is_err());
        assert!(Row::decode(&[4, 0x80, 0x80, 0x80, 0x80, 0x80]).is_err());
        assert!(Row::decode(&[4, 1, 3, 1, 0, 1, 36, 1, b'x']).is_err());
        assert!(Table::decode(&[2]).is_err());
    }
}
