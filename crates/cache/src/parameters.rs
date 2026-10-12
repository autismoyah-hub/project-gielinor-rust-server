//! Defaults and declared cache literal types used when resolving object parameters.
use gielinor_protocol::buffer::{DecodeError, Reader};

pub(crate) const LITERALS: &[(i16, char)] = &[
    (1, '1'),
    (117, '-'),
    (6, 'A'),
    (7, 'C'),
    (98, 'F'),
    (8, 'H'),
    (9, 'I'),
    (73, 'J'),
    (10, 'K'),
    (99, 'L'),
    (11, 'M'),
    (12, 'N'),
    (13, 'O'),
    (14, 'P'),
    (16, 'R'),
    (17, 'S'),
    (18, 'T'),
    (21, '`'),
    (97, 'a'),
    (22, 'c'),
    (23, 'd'),
    (25, 'f'),
    (26, 'g'),
    (28, 'j'),
    (0, 'i'),
    (30, 'l'),
    (31, 'm'),
    (32, 'n'),
    (33, 'o'),
    (34, 'p'),
    (36, 's'),
    (37, 't'),
    (38, 'u'),
    (39, 'v'),
    (40, 'x'),
    (41, 'y'),
    (42, 'z'),
    (59, 'µ'),
    (62, '×'),
    (93, '¯'),
    (-1, '¸'),
    (118, 'Ø'),
    (74, 'Ð'),
    (101, 'Ý'),
    (209, '7'),
    (254, ']'),
    (253, '['),
];
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParameterType {
    pub literal: Option<i16>,
    pub default_int: Option<i32>,
    pub default_string: Option<String>,
    pub auto_disable: bool,
    pub transmit: bool,
    pub decoded_bytes: usize,
}
impl ParameterType {
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(bytes);
        let mut type_char = None;
        let mut value = Self {
            literal: None,
            default_int: None,
            default_string: None,
            auto_disable: true,
            transmit: true,
            decoded_bytes: 0,
        };
        while r.remaining() != 0 {
            match r.u8()? {
                0 => break,
                // Kotlin casts the unsigned byte directly to Char, not through CP1252.
                1 => type_char = Some(char::from(r.u8()?)),
                2 => value.default_int = Some(r.u32()? as i32),
                4 => value.auto_disable = false,
                5 => value.default_string = Some(r.string()?),
                7 => {
                    r.u64()?;
                } // DefaultLong is also discarded by Kotlin.
                8 => {
                    let id = r.u8()? as i16;
                    if let Some((_, ch)) = LITERALS.iter().find(|(literal, _)| *literal == id) {
                        type_char = Some(*ch);
                    }
                }
                200 => value.transmit = false,
                _ => return Err(DecodeError::InvalidValue("unknown parameter opcode")),
            }
        }
        value.literal =
            type_char.and_then(|ch| LITERALS.iter().find(|(_, c)| *c == ch).map(|(id, _)| *id));
        value.decoded_bytes = r.position();
        Ok(value)
    }
}
