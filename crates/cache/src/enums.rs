//! Cache enum tables retain defaults and replace earlier maps on repeated map opcodes.
use gielinor_protocol::buffer::{DecodeError, Reader};
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Int(i32),
    String(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumType {
    pub key_literal: i16,
    pub value_literal: i16,
    pub default_int: Option<i32>,
    pub default_string: Option<String>,
    pub values: BTreeMap<i32, Value>,
    /// First occurrence of each key in the last map opcode, before Java hashing.
    pub insertion_order: Vec<i32>,
    pub transmit: bool,
    pub decoded_bytes: usize,
}
impl EnumType {
    pub fn decode(bytes: &[u8]) -> Result<Option<Self>, DecodeError> {
        let mut r = Reader::new(bytes);
        let (mut key, mut value) = (None, None);
        let mut result = Self {
            key_literal: 0,
            value_literal: 0,
            default_int: None,
            default_string: None,
            values: BTreeMap::new(),
            insertion_order: Vec::new(),
            transmit: true,
            decoded_bytes: 0,
        };
        let literal = |b: u8| {
            crate::parameters::LITERALS
                .iter()
                .find(|(_, ch)| *ch == char::from(b))
                .map(|(id, _)| *id)
        };
        while r.remaining() > 0 {
            match r.u8()? {
                0 => break,
                1 => key = literal(r.u8()?),
                2 => value = literal(r.u8()?),
                3 => result.default_string = Some(r.string()?),
                4 => result.default_int = Some(r.u32()? as i32),
                op @ (5 | 6) => {
                    result.values.clear();
                    result.insertion_order.clear();
                    let count = r.u16()?;
                    for _ in 0..count {
                        let key = r.u32()? as i32;
                        let value = if op == 5 {
                            Value::String(r.string()?)
                        } else {
                            Value::Int(r.u32()? as i32)
                        };
                        if result.values.insert(key, value).is_none() {
                            result.insertion_order.push(key);
                        }
                    }
                }
                200 => result.transmit = false,
                _ => return Err(DecodeError::InvalidValue("unknown enum opcode")),
            }
        }
        let (Some(key), Some(value)) = (key, value) else {
            return Ok(None);
        };
        result.key_literal = key;
        result.value_literal = value;
        result.decoded_bytes = r.position();
        Ok(Some(result))
    }
}
