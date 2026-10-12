#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    Truncated,
    UnterminatedString,
    InvalidStringPrefix,
    TrailingBytes(usize),
    InvalidValue(&'static str),
}
#[derive(Clone, Copy)]
pub enum Transform {
    None,
    Add128,
    Negate,
    Sub128,
}
#[derive(Clone, Copy)]
pub enum Order {
    Big,
    Little,
    Middle,
    InverseMiddle,
}

pub struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }
    pub fn position(&self) -> usize {
        self.position
    }
    pub fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }
    pub fn finish(self) -> Result<(), DecodeError> {
        if self.remaining() == 0 {
            Ok(())
        } else {
            Err(DecodeError::TrailingBytes(self.remaining()))
        }
    }
    pub fn take(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        if n > self.remaining() {
            return Err(DecodeError::Truncated);
        }
        let v = &self.bytes[self.position..self.position + n];
        self.position += n;
        Ok(v)
    }
    pub fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take(1)?[0])
    }
    pub fn transformed_u8(&mut self, t: Transform) -> Result<u8, DecodeError> {
        let b = self.u8()?;
        Ok(match t {
            Transform::None => b,
            Transform::Add128 => b.wrapping_sub(128),
            Transform::Negate => 0u8.wrapping_sub(b),
            Transform::Sub128 => 128u8.wrapping_sub(b),
        })
    }
    pub fn u16(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_be_bytes(self.take(2)?.try_into().unwrap()))
    }
    pub fn u16_le(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    pub fn u16_add(&mut self, little: bool) -> Result<u16, DecodeError> {
        let b = self.take(2)?;
        Ok(if little {
            ((b[1] as u16) << 8) | b[0].wrapping_sub(128) as u16
        } else {
            ((b[0] as u16) << 8) | b[1].wrapping_sub(128) as u16
        })
    }
    pub fn u24(&mut self) -> Result<u32, DecodeError> {
        let b = self.take(3)?;
        Ok(((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32)
    }
    pub fn u32(&mut self) -> Result<u32, DecodeError> {
        self.u32_order(Order::Big)
    }
    pub fn u32_order(&mut self, order: Order) -> Result<u32, DecodeError> {
        let b = self.take(4)?;
        Ok(u32::from_be_bytes(match order {
            Order::Big => [b[0], b[1], b[2], b[3]],
            Order::Little => [b[3], b[2], b[1], b[0]],
            Order::Middle => [b[2], b[3], b[0], b[1]],
            Order::InverseMiddle => [b[1], b[0], b[3], b[2]],
        }))
    }
    pub fn u64(&mut self) -> Result<u64, DecodeError> {
        Ok(u64::from_be_bytes(self.take(8)?.try_into().unwrap()))
    }
    pub fn smart(&mut self) -> Result<u16, DecodeError> {
        let first = *self
            .bytes
            .get(self.position)
            .ok_or(DecodeError::Truncated)?;
        if first < 128 {
            Ok(self.u8()? as u16)
        } else {
            Ok(self.u16()? - 32768)
        }
    }
    pub fn string(&mut self) -> Result<String, DecodeError> {
        let len = self.bytes[self.position..]
            .iter()
            .position(|b| *b == 0)
            .ok_or(DecodeError::UnterminatedString)?;
        let bytes = self.take(len)?;
        self.position += 1;
        Ok(bytes.iter().map(|b| decode_cp1252(*b)).collect())
    }
    pub fn prefixed_string(&mut self) -> Result<String, DecodeError> {
        if self.u8()? != 0 {
            return Err(DecodeError::InvalidStringPrefix);
        }
        self.string()
    }
}

const CP1252: [char; 32] = [
    '€', '?', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '?', 'Ž', '?', '?', '‘', '’',
    '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '?', 'ž', 'Ÿ',
];
pub fn decode_cp1252(b: u8) -> char {
    if (128..160).contains(&b) {
        CP1252[(b - 128) as usize]
    } else {
        char::from(b)
    }
}

#[derive(Default)]
pub struct BitWriter {
    bytes: Vec<u8>,
    bits: usize,
}
impl BitWriter {
    pub fn write(&mut self, width: u8, value: u32) -> Result<(), DecodeError> {
        if width > 32 || (width < 32 && (value >> width) != 0) {
            return Err(DecodeError::InvalidValue("bit field"));
        }
        self.bytes
            .resize((self.bits + width as usize).div_ceil(8), 0);
        for i in (0..width).rev() {
            self.bytes[self.bits / 8] |= (((value >> i) & 1) as u8) << (7 - self.bits % 8);
            self.bits += 1;
        }
        Ok(())
    }
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_reads_do_not_advance() {
        let mut r = Reader::new(&[1]);
        assert_eq!(r.u32(), Err(DecodeError::Truncated));
        assert_eq!(r.position(), 0);
        assert_eq!(r.u8(), Ok(1));
    }
    #[test]
    fn strings_follow_jagex_cp1252() {
        assert_eq!(Reader::new(&[128, 129, 255, 0]).string().unwrap(), "€?ÿ");
        assert_eq!(
            Reader::new(&[1]).string(),
            Err(DecodeError::UnterminatedString)
        );
    }
    #[test]
    fn bits_cross_byte_boundaries() {
        let mut w = BitWriter::default();
        w.write(3, 5).unwrap();
        w.write(8, 0xAB).unwrap();
        w.write(1, 1).unwrap();
        assert_eq!(w.into_bytes(), vec![0xB5, 0x70]);
    }
}
