use crate::isaac::Isaac;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    Fixed(usize),
    U8,
    U16,
}
#[derive(Clone, Copy, Debug)]
pub struct PacketDef {
    pub opcode: u16,
    pub size: Size,
    pub name: &'static str,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Packet {
    pub opcode: u16,
    pub payload: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameError {
    UnknownOpcode(u16),
    PayloadTooLarge(usize),
    WrongSize,
    BufferLimit,
    Poisoned,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpcodeWidth {
    Byte,
    Smart,
}

enum State {
    Opcode,
    SecondOpcode(u8),
    Length(PacketDef),
    Payload(PacketDef, usize),
}

/// Streaming framing with bounded storage and terminal errors. Each opcode byte
/// consumes exactly one cipher word, including when TCP divides the header.
pub struct Decoder {
    defs: &'static [PacketDef],
    cipher: Option<Isaac>,
    width: OpcodeWidth,
    state: State,
    buffer: Vec<u8>,
    offset: usize,
    max_payload: usize,
    max_buffer: usize,
    poisoned: bool,
}

impl Decoder {
    /// Used only at a protocol transition after a complete packet was consumed.
    pub fn take_unread(&mut self) -> Result<Vec<u8>, FrameError> {
        if self.poisoned || !matches!(self.state, State::Opcode) {
            return Err(FrameError::Poisoned);
        }
        let unread = self.buffer[self.offset..].to_vec();
        self.buffer.clear();
        self.offset = 0;
        Ok(unread)
    }
    pub fn new(
        defs: &'static [PacketDef],
        cipher: Option<Isaac>,
        width: OpcodeWidth,
        max_payload: usize,
        max_buffer: usize,
    ) -> Self {
        Self {
            defs,
            cipher,
            width,
            state: State::Opcode,
            buffer: Vec::new(),
            offset: 0,
            max_payload,
            max_buffer,
            poisoned: false,
        }
    }
    pub fn feed(&mut self, bytes: &[u8]) -> Result<(), FrameError> {
        if self.poisoned {
            return Err(FrameError::Poisoned);
        }
        if bytes.len()
            > self
                .max_buffer
                .saturating_sub(self.buffer.len() - self.offset)
        {
            self.poisoned = true;
            return Err(FrameError::BufferLimit);
        }
        if self.offset != 0 {
            self.buffer.drain(..self.offset);
            self.offset = 0;
        }
        self.buffer.extend_from_slice(bytes);
        Ok(())
    }
    fn byte(&mut self) -> u8 {
        let v = self.buffer[self.offset];
        self.offset += 1;
        v
    }
    fn opcode_byte(&mut self) -> u8 {
        let value = self.byte();
        value.wrapping_sub(self.cipher.as_mut().map_or(0, |c| c.next_u32() as u8))
    }
    fn set_opcode(&mut self, opcode: u16) -> Result<(), FrameError> {
        let def = *self
            .defs
            .iter()
            .find(|d| d.opcode == opcode)
            .ok_or(FrameError::UnknownOpcode(opcode))?;
        self.state = match def.size {
            Size::Fixed(n) => {
                self.check_size(n)?;
                State::Payload(def, n)
            }
            _ => State::Length(def),
        };
        Ok(())
    }
    fn check_size(&self, size: usize) -> Result<(), FrameError> {
        if size > self.max_payload {
            Err(FrameError::PayloadTooLarge(size))
        } else {
            Ok(())
        }
    }
    pub fn next_packet(&mut self) -> Result<Option<Packet>, FrameError> {
        if self.poisoned {
            return Err(FrameError::Poisoned);
        }
        let result = self.decode();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    fn decode(&mut self) -> Result<Option<Packet>, FrameError> {
        loop {
            let available = self.buffer.len() - self.offset;
            match self.state {
                State::Opcode => {
                    if available == 0 {
                        return Ok(None);
                    }
                    let first = self.opcode_byte();
                    if self.width == OpcodeWidth::Smart && first >= 128 {
                        self.state = State::SecondOpcode(first - 128);
                    } else {
                        self.set_opcode(first as u16)?;
                    }
                }
                State::SecondOpcode(high) => {
                    if available == 0 {
                        return Ok(None);
                    }
                    let low = self.opcode_byte();
                    self.set_opcode(((high as u16) << 8) | low as u16)?;
                }
                State::Length(def) => {
                    let size = match def.size {
                        Size::U8 if available >= 1 => self.byte() as usize,
                        Size::U16 if available >= 2 => {
                            ((self.byte() as usize) << 8) | self.byte() as usize
                        }
                        _ => return Ok(None),
                    };
                    self.check_size(size)?;
                    self.state = State::Payload(def, size);
                }
                State::Payload(def, size) => {
                    if available < size {
                        return Ok(None);
                    }
                    let payload = self.buffer[self.offset..self.offset + size].to_vec();
                    self.offset += size;
                    self.state = State::Opcode;
                    if self.offset == self.buffer.len() {
                        self.buffer.clear();
                        self.offset = 0;
                    }
                    return Ok(Some(Packet {
                        opcode: def.opcode,
                        payload,
                    }));
                }
            }
        }
    }
}

/// Exact wire cost, validated without advancing ISAAC or allocating a payload.
pub fn encoded_len(
    def: PacketDef,
    payload_len: usize,
    width: OpcodeWidth,
) -> Result<usize, FrameError> {
    if def.opcode
        > if width == OpcodeWidth::Byte {
            255
        } else {
            32767
        }
    {
        return Err(FrameError::UnknownOpcode(def.opcode));
    }
    match def.size {
        Size::Fixed(n) if payload_len != n => return Err(FrameError::WrongSize),
        Size::U8 if payload_len > 255 => return Err(FrameError::PayloadTooLarge(payload_len)),
        Size::U16 if payload_len > 65535 => return Err(FrameError::PayloadTooLarge(payload_len)),
        _ => (),
    }
    let overhead = if width == OpcodeWidth::Smart && def.opcode >= 128 {
        2
    } else {
        1
    } + match def.size {
        Size::Fixed(_) => 0,
        Size::U8 => 1,
        Size::U16 => 2,
    };
    payload_len
        .checked_add(overhead)
        .ok_or(FrameError::PayloadTooLarge(payload_len))
}

/// Validate before consuming any cipher words so a failed encode is retryable.
pub fn encode(
    def: PacketDef,
    payload: &[u8],
    cipher: &mut Option<Isaac>,
    width: OpcodeWidth,
) -> Result<Vec<u8>, FrameError> {
    let capacity = encoded_len(def, payload.len(), width)?;
    let mut out = Vec::with_capacity(capacity);
    let mut push_opcode =
        |b: u8| out.push(b.wrapping_add(cipher.as_mut().map_or(0, |c| c.next_u32() as u8)));
    if width == OpcodeWidth::Smart && def.opcode >= 128 {
        push_opcode(((def.opcode >> 8) as u8) + 128);
    }
    push_opcode(def.opcode as u8);
    match def.size {
        Size::U8 => out.push(payload.len() as u8),
        Size::U16 => out.extend_from_slice(&(payload.len() as u16).to_be_bytes()),
        _ => (),
    }
    out.extend_from_slice(payload);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wire_budget_rejects_size_overflow_without_allocating_or_consuming_cipher() {
        let def = PacketDef {
            opcode: 128,
            size: Size::Fixed(usize::MAX),
            name: "oversize",
        };
        assert_eq!(
            encoded_len(def, usize::MAX, OpcodeWidth::Smart),
            Err(FrameError::PayloadTooLarge(usize::MAX))
        );
        let mut cipher = Some(Isaac::new(&[1, 2, 3, 4]).unwrap());
        let mut expected = Some(Isaac::new(&[1, 2, 3, 4]).unwrap());
        assert_eq!(
            encode(def, &[], &mut cipher, OpcodeWidth::Smart),
            Err(FrameError::WrongSize)
        );
        assert_eq!(
            encode(DEFS[0], &[], &mut cipher, OpcodeWidth::Smart),
            encode(DEFS[0], &[], &mut expected, OpcodeWidth::Smart)
        );
        assert_eq!(encoded_len(DEFS[1], 255, OpcodeWidth::Smart), Ok(257));
        assert_eq!(encoded_len(DEFS[2], 65535, OpcodeWidth::Smart), Ok(65539));
    }
    static DEFS: &[PacketDef] = &[
        PacketDef {
            opcode: 5,
            size: Size::Fixed(0),
            name: "empty",
        },
        PacketDef {
            opcode: 127,
            size: Size::U8,
            name: "small",
        },
        PacketDef {
            opcode: 128,
            size: Size::U16,
            name: "large",
        },
        PacketDef {
            opcode: 32767,
            size: Size::Fixed(2),
            name: "last",
        },
    ];
    #[test]
    fn every_fragment_size_preserves_cipher_and_headers() {
        let payloads = [vec![], vec![239; 255], vec![1; 300], vec![2, 3]];
        let expected: Vec<_> = DEFS
            .iter()
            .zip(&payloads)
            .map(|(d, p)| Packet {
                opcode: d.opcode,
                payload: p.clone(),
            })
            .collect();
        let mut cipher = Some(Isaac::new(&[1, 2, 3, 4]).unwrap());
        let wire: Vec<_> = DEFS
            .iter()
            .zip(&payloads)
            .flat_map(|(d, p)| encode(*d, p, &mut cipher, OpcodeWidth::Smart).unwrap())
            .collect();
        for chunk in 1..=wire.len() {
            let mut decoder = Decoder::new(
                DEFS,
                Some(Isaac::new(&[1, 2, 3, 4]).unwrap()),
                OpcodeWidth::Smart,
                1024,
                2048,
            );
            let mut actual = Vec::new();
            for part in wire.chunks(chunk) {
                decoder.feed(part).unwrap();
                while let Some(p) = decoder.next_packet().unwrap() {
                    actual.push(p);
                }
            }
            assert_eq!(actual, expected, "chunk size {chunk}");
        }
    }
    #[test]
    fn malformed_streams_fail_closed() {
        let mut d = Decoder::new(DEFS, None, OpcodeWidth::Byte, 10, 100);
        d.feed(&[127, 11]).unwrap();
        assert_eq!(d.next_packet(), Err(FrameError::PayloadTooLarge(11)));
        assert_eq!(d.feed(&[0]), Err(FrameError::Poisoned));
        let mut d = Decoder::new(DEFS, None, OpcodeWidth::Byte, 10, 100);
        d.feed(&[250]).unwrap();
        assert_eq!(d.next_packet(), Err(FrameError::UnknownOpcode(250)));
        let mut d = Decoder::new(DEFS, None, OpcodeWidth::Byte, 10, 2);
        assert_eq!(d.feed(&[1, 2, 3]), Err(FrameError::BufferLimit));
    }
}
