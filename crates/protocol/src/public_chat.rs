//! Revision 239 MESSAGE_PUBLIC (client opcode 69) and the player-info chat block
//! (mask 0x100) that carries public chat to observers, including the sender.
//! Payload order follows rsprox osrs-239 MessagePublicDecoder; the block follows
//! rsprox PlayerInfoClient.decodeChat and rust-client player_update.
use crate::{
    buffer::{DecodeError, Reader},
    huffman::Huffman,
};
/// The chatbox input accepts at most 80 characters.
pub const MAX_LENGTH: usize = 80;
/// Public chat; types 1..=4 address other channels (clan main channel 3 carries a trailing byte).
pub const PUBLIC: u8 = 0;
const CLAN_MAIN: u8 = 3;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub kind: u8,
    /// 0..=12 named colours and glows, 13..=20 an explicit pattern of colour - 12 entries.
    pub colour: u8,
    pub effect: u8,
    pub pattern: Vec<u8>,
    /// CP1252 bytes, at most [`MAX_LENGTH`].
    pub text: Vec<u8>,
    pub clan: Option<u8>,
}
fn smart(out: &mut Vec<u8>, value: usize) -> Result<(), DecodeError> {
    match value {
        0..=127 => out.push(value as u8),
        128..=32767 => out.extend((value as u16 + 32768).to_be_bytes()),
        _ => return Err(DecodeError::InvalidValue("smart value")),
    }
    Ok(())
}
/// Uncompressed length smart followed by the Huffman bit stream.
fn compress(text: &[u8], huffman: &Huffman) -> Result<Vec<u8>, DecodeError> {
    let mut out = vec![];
    smart(&mut out, text.len())?;
    out.extend(huffman.encode(text)?);
    Ok(out)
}
fn validate(colour: u8, effect: u8, pattern: &[u8], text: &[u8]) -> Result<(), DecodeError> {
    if colour > 20 || effect > 5 {
        return Err(DecodeError::InvalidValue("public chat colour or effect"));
    }
    let expected = if colour >= 13 { colour as usize - 12 } else { 0 };
    // Pattern entries are base-36 digits typed after "pattern".
    if pattern.len() != expected || pattern.iter().any(|&p| p > 35) {
        return Err(DecodeError::InvalidValue("public chat pattern"));
    }
    if text.len() > MAX_LENGTH || text.contains(&0) {
        return Err(DecodeError::InvalidValue("public chat text"));
    }
    Ok(())
}
impl Message {
    pub fn decode(payload: &[u8], huffman: &Huffman) -> Result<Self, DecodeError> {
        let mut r = Reader::new(payload);
        let kind = r.u8()?;
        let colour = r.u8()?;
        let effect = r.u8()?;
        let pattern = if (13..=20).contains(&colour) {
            r.take(colour as usize - 12)?.to_vec()
        } else {
            vec![]
        };
        let trailing = usize::from(kind == CLAN_MAIN);
        let body = r.take(r.remaining().checked_sub(trailing).ok_or(DecodeError::Truncated)?)?;
        let clan = if trailing == 1 { Some(r.u8()?) } else { None };
        r.finish()?;
        let mut body = Reader::new(body);
        let length = body.smart()? as usize;
        if length > MAX_LENGTH {
            return Err(DecodeError::InvalidValue("public chat text"));
        }
        let bits = body.take(body.remaining())?;
        let (text, used) = huffman.decode(bits, length, MAX_LENGTH)?;
        if used != bits.len() {
            return Err(DecodeError::TrailingBytes(bits.len() - used));
        }
        validate(colour, effect, &pattern, &text)?;
        Ok(Self {
            kind,
            colour,
            effect,
            pattern,
            text,
            clan,
        })
    }
    /// Client-side GamePacketBuffer.ag layout, for tests and drivers.
    pub fn encode(&self, huffman: &Huffman) -> Result<Vec<u8>, DecodeError> {
        validate(self.colour, self.effect, &self.pattern, &self.text)?;
        let mut out = vec![self.kind, self.colour, self.effect];
        out.extend(&self.pattern);
        out.extend(compress(&self.text, huffman)?);
        if self.kind == CLAN_MAIN {
            out.push(self.clan.ok_or(DecodeError::InvalidValue("clan channel"))?);
        }
        Ok(out)
    }
    /// Player-info chat block body (without the mask). `icon` is the sender's
    /// mod-level client code; `autotyper` is never set by MESSAGE_PUBLIC.
    pub fn block(&self, icon: u8, huffman: &Huffman) -> Result<Vec<u8>, DecodeError> {
        validate(self.colour, self.effect, &self.pattern, &self.text)?;
        let mut compressed = compress(&self.text, huffman)?;
        let size = u8::try_from(compressed.len())
            .map_err(|_| DecodeError::InvalidValue("public chat block length"))?;
        compressed.reverse();
        let mut out = vec![self.colour, self.effect.wrapping_add(128)];
        out.push(icon.wrapping_neg());
        out.push(128); // autotyper = 0, add-128 transformed
        out.push(size.wrapping_neg());
        out.extend(compressed);
        out.extend(self.pattern.iter().map(|p| p.wrapping_neg()));
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Transform;
    fn codec() -> Huffman {
        // A uniform 8-bit table is a valid prefix code that keeps tests readable.
        Huffman::new(&[8; 256]).unwrap()
    }
    fn message(text: &str) -> Message {
        Message {
            kind: PUBLIC,
            colour: 0,
            effect: 0,
            pattern: vec![],
            text: text.as_bytes().to_vec(),
            clan: None,
        }
    }
    #[test]
    fn decodes_client_layout_with_pattern_and_clan_trailer() {
        let h = codec();
        // kind 0, colour 1 (red), effect 2 (wave2), length 5, "Hello".
        assert_eq!(
            Message::decode(&[0, 1, 2, 5, b'H', b'e', b'l', b'l', b'o'], &h).unwrap(),
            Message {
                colour: 1,
                effect: 2,
                ..message("Hello")
            }
        );
        let patterned = Message {
            colour: 15,
            pattern: vec![0, 10, 35],
            ..message("hi")
        };
        let bytes = patterned.encode(&h).unwrap();
        assert_eq!(bytes, [0, 15, 0, 0, 10, 35, 2, b'h', b'i']);
        assert_eq!(Message::decode(&bytes, &h).unwrap(), patterned);
        let clan = Message {
            kind: 3,
            clan: Some(1),
            ..message("x")
        };
        let bytes = clan.encode(&h).unwrap();
        assert_eq!(bytes, [3, 0, 0, 1, b'x', 1]);
        assert_eq!(Message::decode(&bytes, &h).unwrap(), clan);
    }
    #[test]
    fn malformed_payloads_are_rejected() {
        let h = codec();
        let long = vec![b'a'; MAX_LENGTH + 1];
        let mut too_long = vec![0, 0, 0];
        smart(&mut too_long, long.len()).unwrap();
        too_long.extend(&long);
        for (case, bytes) in [
            ("empty", vec![]),
            ("header only", vec![0, 0]),
            ("no text", vec![0, 0, 0]),
            ("truncated huffman", vec![0, 0, 0, 5, b'a']),
            ("trailing bytes", vec![0, 0, 0, 1, b'a', b'b']),
            ("colour", vec![0, 21, 0, 1, b'a']),
            ("effect", vec![0, 0, 6, 1, b'a']),
            ("pattern truncated", vec![0, 14, 0, 1]),
            ("pattern digit", vec![0, 13, 0, 36, 1, b'a']),
            ("nul in text", vec![0, 0, 0, 1, 0]),
            ("clan trailer missing", vec![3, 0, 0]),
            ("too long", too_long),
        ] {
            assert!(Message::decode(&bytes, &h).is_err(), "{case}");
        }
        // The maximum length is accepted.
        let max = message(std::str::from_utf8(&long[..MAX_LENGTH]).unwrap());
        assert_eq!(Message::decode(&max.encode(&h).unwrap(), &h).unwrap(), max);
    }
    #[test]
    fn chat_block_matches_the_client_reader() {
        let h = codec();
        let m = Message {
            colour: 14,
            effect: 3,
            pattern: vec![1, 2],
            ..message("Hey")
        };
        let block = m.block(2, &h).unwrap();
        // Mirror rust-client player_update / rsprox decodeChat.
        let mut r = Reader::new(&block);
        let packed = r.u16_add(false).unwrap();
        assert_eq!((packed >> 8, packed & 0xff), (14, 3));
        assert_eq!(r.transformed_u8(Transform::Negate).unwrap(), 2);
        assert_eq!(r.transformed_u8(Transform::Add128).unwrap(), 0);
        let size = r.transformed_u8(Transform::Negate).unwrap() as usize;
        let mut compressed = r.take(size).unwrap().to_vec();
        compressed.reverse();
        assert_eq!(compressed, [3, b'H', b'e', b'y']);
        let pattern: Vec<_> = (0..2)
            .map(|_| r.transformed_u8(Transform::Negate).unwrap())
            .collect();
        assert_eq!(pattern, [1, 2]);
        r.finish().unwrap();
    }
}
