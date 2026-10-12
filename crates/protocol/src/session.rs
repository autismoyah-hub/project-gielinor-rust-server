use crate::{
    framing::{self, Decoder, FrameError, OpcodeWidth, Packet},
    isaac::Isaac,
    login::{self, Authentication, Login},
    rsa::PrivateKey,
    tables,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Handshake,
    Login,
    Authenticating,
    Active,
    Js5,
    Closed,
}
pub enum Event {
    Write(Vec<u8>),
    Authenticate(Login),
    Game(Packet),
    Js5(crate::js5::Command),
    Rejected(u8),
}
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Framing(FrameError),
    InvalidState,
    UnsupportedService(u16),
}
impl From<FrameError> for Error {
    fn from(e: FrameError) -> Self {
        Self::Framing(e)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Accepted {
    pub slot: u16,
    pub staff_level: u8,
    pub player_mod: bool,
    pub member: bool,
    pub account_hash: u64,
    pub user_id: u64,
    pub user_hash: u64,
    pub authenticator_code: Option<u32>,
}

/// Transport-independent, bounded session. Authentication is delegated to the account
/// service, and admission to the world must succeed before `accept` is called.
pub struct Session {
    state: State,
    decoder: Decoder,
    outgoing: Option<Isaac>,
    server_seed: u64,
    pending_seed: Option<[u32; 4]>,
    cache_crcs: [u32; 23],
}
impl Session {
    pub fn new(server_seed: u64, cache_crcs: [u32; 23]) -> Self {
        Self {
            state: State::Handshake,
            decoder: Decoder::new(tables::LOGIN_CLIENT, None, OpcodeWidth::Byte, 65535, 131072),
            outgoing: None,
            server_seed,
            pending_seed: None,
            cache_crcs,
        }
    }
    pub fn state(&self) -> State {
        self.state
    }
    pub fn close(&mut self) {
        self.state = State::Closed;
        self.pending_seed = None;
        self.outgoing = None;
    }
    pub fn reject(&mut self, status: u8) -> Vec<Event> {
        self.close();
        vec![Event::Write(vec![status]), Event::Rejected(status)]
    }
    pub fn receive(&mut self, bytes: &[u8], key: &PrivateKey) -> Result<Vec<Event>, Error> {
        if self.state == State::Closed {
            return Err(Error::InvalidState);
        }
        let result = self.receive_inner(bytes, key);
        if result.is_err() {
            self.close();
        }
        result
    }
    fn receive_inner(&mut self, bytes: &[u8], key: &PrivateKey) -> Result<Vec<Event>, Error> {
        self.decoder.feed(bytes)?;
        let mut events = Vec::new();
        while self.state != State::Authenticating {
            let Some(packet) = self.decoder.next_packet()? else {
                break;
            };
            match self.state {
                State::Handshake if packet.opcode == 14 => {
                    self.state = State::Login;
                    events.push(Event::Write(login::challenge(self.server_seed).to_vec()));
                }
                State::Handshake if packet.opcode == 15 => {
                    let handshake =
                        crate::js5::handshake(&packet.payload).map_err(|_| Error::InvalidState)?;
                    if handshake.revision != 239 {
                        events.extend(self.reject(6));
                        break;
                    }
                    let unread = self.decoder.take_unread()?;
                    self.decoder =
                        Decoder::new(crate::js5::CLIENT, None, OpcodeWidth::Byte, 3, 131072);
                    self.decoder.feed(&unread)?;
                    self.state = State::Js5;
                    events.push(Event::Write(vec![0]));
                }
                State::Handshake => return Err(Error::UnsupportedService(packet.opcode)),
                State::Login if packet.opcode == 18 => {
                    events.extend(self.reject(50));
                    break;
                }
                State::Login if packet.opcode == 16 => {
                    if packet
                        .payload
                        .get(..4)
                        .is_some_and(|v| v != 239u32.to_be_bytes())
                    {
                        events.extend(self.reject(6));
                        break;
                    }
                    let decoded = match login::decode(&packet.payload, key) {
                        Ok(v) => v,
                        Err(_) => {
                            events.extend(self.reject(22));
                            break;
                        }
                    };
                    if decoded.server_seed != self.server_seed {
                        events.extend(self.reject(10));
                        break;
                    }
                    if decoded.cache_crcs != self.cache_crcs {
                        events.extend(self.reject(68));
                        break;
                    }
                    if matches!(decoded.authentication, Authentication::Token(_)) {
                        events.extend(self.reject(22));
                        break;
                    }
                    self.pending_seed = Some(decoded.seed);
                    self.state = State::Authenticating;
                    events.push(Event::Authenticate(decoded));
                }
                State::Login => return Err(Error::InvalidState),
                State::Active => events.push(Event::Game(packet)),
                State::Js5 => events.push(Event::Js5(
                    crate::js5::command(packet.opcode as u8, &packet.payload)
                        .map_err(|_| Error::InvalidState)?,
                )),
                _ => return Err(Error::InvalidState),
            }
        }
        Ok(events)
    }
    pub fn accept(&mut self, a: Accepted) -> Result<Vec<u8>, Error> {
        if self.state != State::Authenticating || a.slot == 0 || a.slot >= 2047 {
            return Err(Error::InvalidState);
        }
        let seed = self.pending_seed.take().ok_or(Error::InvalidState)?;
        let unread = self.decoder.take_unread()?;
        self.decoder = Decoder::new(
            tables::CLIENT,
            Some(Isaac::new(&seed).unwrap()),
            OpcodeWidth::Byte,
            65535,
            131072,
        );
        self.decoder.feed(&unread)?;
        self.outgoing = Some(Isaac::new(&seed.map(|v| v.wrapping_add(50))).unwrap());
        // Revision 239 counts the first REBUILD_NORMAL_V2 opcode and u16 length
        // in this announcement. Its three-byte header follows these 34 account
        // bytes through the ordinary game encoder; TCP may fragment either part.
        let mut response = vec![2, 37];
        if let Some(code) = a.authenticator_code {
            response.push(1);
            for byte in code.to_be_bytes() {
                response.push(byte.wrapping_add(self.outgoing.as_mut().unwrap().next_u32() as u8));
            }
        } else {
            response.extend([0; 5]);
        }
        response.extend([a.staff_level, a.player_mod as u8]);
        response.extend(a.slot.to_be_bytes());
        response.push(a.member as u8);
        for v in [a.account_hash, a.user_id, a.user_hash] {
            response.extend(v.to_be_bytes());
        }
        self.state = State::Active;
        Ok(response)
    }
    pub fn encode(&mut self, opcode: u16, payload: &[u8]) -> Result<Vec<u8>, Error> {
        if self.state != State::Active {
            return Err(Error::InvalidState);
        }
        let def = *tables::SERVER
            .iter()
            .find(|d| d.opcode == opcode)
            .ok_or(FrameError::UnknownOpcode(opcode))?;
        Ok(framing::encode(
            def,
            payload,
            &mut self.outgoing,
            OpcodeWidth::Smart,
        )?)
    }
}
