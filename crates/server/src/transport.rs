//! Bounded nonblocking TCP transport. Authentication and world ticks are owned by the runtime.
use gielinor_protocol::{
    framing::Packet,
    login::Login,
    rsa::PrivateKey,
    session::{Accepted, Event, Session, State},
};
use std::{
    collections::BTreeMap,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    time::{Duration, Instant},
};
pub enum NetworkEvent {
    Authenticate { connection: u64, login: Login },
    Packet { connection: u64, packet: Packet },
    Disconnected { connection: u64, reason: String },
}
pub struct Limits {
    pub connections: usize,
    pub output_bytes: usize,
    pub login_timeout: Duration,
    pub idle_timeout: Duration,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            connections: 64,
            output_bytes: 262144,
            login_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(60),
        }
    }
}
struct Connection {
    socket: TcpStream,
    session: Session,
    output: Vec<u8>,
    pending: [Vec<Packet>; 2],
    pending_bytes: usize,
    written: usize,
    created: Instant,
    last_read: Instant,
    closing: bool,
    js5: crate::js5::Client,
}
pub struct Transport {
    listener: Option<TcpListener>,
    accepting: bool,
    batching: bool,
    tick_queued: bool,
    connections: BTreeMap<u64, Connection>,
    next_id: u64,
    key: PrivateKey,
    crcs: [u32; 23],
    limits: Limits,
    js5: Option<Box<dyn crate::js5::Provider>>,
}
impl Transport {
    pub fn bind(
        address: SocketAddr,
        key: PrivateKey,
        crcs: [u32; 23],
        limits: Limits,
    ) -> io::Result<Self> {
        let mut transport = Self::prepare(key, crcs, limits)?;
        transport.listen(address)?;
        Ok(transport)
    }
    /// Allocate transport state without opening a socket, for startup validation.
    pub fn prepare(key: PrivateKey, crcs: [u32; 23], limits: Limits) -> io::Result<Self> {
        if limits.connections == 0 || limits.connections > 2046 || limits.output_bytes < 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid connection limits",
            ));
        }
        Ok(Self {
            listener: None,
            accepting: false,
            batching: false,
            tick_queued: false,
            connections: BTreeMap::new(),
            next_id: 0,
            key,
            crcs,
            limits,
            js5: None,
        })
    }
    /// Game output from worker completions and input handlers belongs to the
    /// next post-tick flush too. Login/JS5 bytes still bypass these game queues.
    pub fn queue_until_tick(&mut self) -> io::Result<()> {
        if self.batching || !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure tick output before accepting connections",
            ));
        }
        self.tick_queued = true;
        Ok(())
    }
    pub fn listen(&mut self, address: SocketAddr) -> io::Result<()> {
        if self.listener.is_some() {
            return Err(io::Error::other("Transport already bound"));
        }
        if address.port() == 43594 || !address.ip().is_loopback() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Development transport requires loopback and forbids port 43594",
            ));
        }
        let listener = TcpListener::bind(address)?;
        listener.set_nonblocking(true)?;
        self.listener = Some(listener);
        self.accepting = true;
        Ok(())
    }
    pub fn address(&self) -> io::Result<SocketAddr> {
        self.listener
            .as_ref()
            .ok_or_else(|| io::Error::other("Transport is not listening"))?
            .local_addr()
    }
    pub fn configure_js5(&mut self, provider: Box<dyn crate::js5::Provider>) -> io::Result<()> {
        if !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure JS5 before accepting connections",
            ));
        }
        self.js5 = Some(provider);
        Ok(())
    }
    pub fn connection_ids(&self) -> Vec<u64> {
        self.connections.keys().copied().collect()
    }
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }
    pub fn stop_accepting(&mut self) {
        self.accepting = false;
    }
    fn append(c: &mut Connection, bytes: &[u8], limit: usize) -> io::Result<()> {
        if bytes.len() > limit.saturating_sub(c.output.len() - c.written) {
            c.session.close();
            c.closing = true;
            c.pending = [vec![], vec![]];
            c.pending_bytes = 0;
            c.output.clear();
            c.written = 0;
            return Err(io::Error::new(
                io::ErrorKind::OutOfMemory,
                "Session output limit exceeded",
            ));
        }
        if c.written > 0 {
            c.output.drain(..c.written);
            c.written = 0;
        }
        c.output.extend(bytes);
        Ok(())
    }
    pub fn accept_login(&mut self, id: u64, accepted: Accepted) -> io::Result<()> {
        let c = self
            .connections
            .get_mut(&id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "Session disconnected"))?;
        let bytes = c
            .session
            .accept(accepted)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{e:?}")))?;
        Self::append(c, &bytes, self.limits.output_bytes)
    }
    pub fn reject_login(&mut self, id: u64, status: u8) -> io::Result<()> {
        let c = self
            .connections
            .get_mut(&id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "Session disconnected"))?;
        c.session.reject(status);
        c.closing = true;
        Self::append(c, &[status], self.limits.output_bytes)
    }
    pub fn send(&mut self, id: u64, opcode: u16, payload: &[u8]) -> io::Result<()> {
        let c = self
            .connections
            .get_mut(&id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "Session disconnected"))?;
        if self.batching || self.tick_queued {
            use gielinor_protocol::{
                framing::{encoded_len, OpcodeWidth},
                outgoing_category::{category, Category},
                tables,
            };
            if c.session.state() != State::Active || c.closing {
                return Err(io::Error::new(
                    io::ErrorKind::NotConnected,
                    "Session is not active",
                ));
            }
            let category = category(opcode).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Unsupported outgoing category")
            })?;
            let def = *tables::SERVER
                .iter()
                .find(|def| def.opcode == opcode)
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Unknown outgoing opcode")
                })?;
            let cost = encoded_len(def, payload.len(), OpcodeWidth::Smart)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{e:?}")))?;
            if cost
                > self
                    .limits
                    .output_bytes
                    .saturating_sub(c.output.len() - c.written + c.pending_bytes)
            {
                c.session.close();
                c.closing = true;
                c.pending = [vec![], vec![]];
                c.pending_bytes = 0;
                c.output.clear();
                c.written = 0;
                return Err(io::Error::new(
                    io::ErrorKind::OutOfMemory,
                    "Session output limit exceeded",
                ));
            }
            let index = match category {
                Category::High => 0,
                Category::Low => 1,
            };
            c.pending[index].push(Packet {
                opcode,
                payload: payload.to_vec(),
            });
            c.pending_bytes += cost;
            return Ok(());
        }
        let bytes = c
            .session
            .encode(opcode, payload)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{e:?}")))?;
        Self::append(c, &bytes, self.limits.output_bytes)
    }
    /// World ticks must queue before encryption: RSProt flushes high, then low priority.
    /// With queue_until_tick, calls outside this scope join the next tick batch.
    pub fn begin_batch(&mut self) -> io::Result<()> {
        if self.batching {
            return Err(io::Error::other("Outgoing batch already active"));
        }
        self.batching = true;
        Ok(())
    }
    fn flush_connection(c: &mut Connection, limit: usize) -> io::Result<()> {
        let queues = std::mem::replace(&mut c.pending, [vec![], vec![]]);
        c.pending_bytes = 0;
        for queue in queues {
            for packet in queue {
                let bytes = match c.session.encode(packet.opcode, &packet.payload) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        c.session.close();
                        c.closing = true;
                        c.output.clear();
                        c.written = 0;
                        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("{e:?}")));
                    }
                };
                Self::append(c, &bytes, limit)?;
            }
        }
        Ok(())
    }
    /// Always leaves batch mode, including when one peer's flush fails.
    pub fn end_batch(&mut self) -> io::Result<()> {
        self.batching = false;
        let mut error = None;
        for c in self.connections.values_mut() {
            if let Err(e) = Self::flush_connection(c, self.limits.output_bytes) {
                if error.is_none() {
                    error = Some(e);
                }
            }
        }
        match error {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
    /// PlayerPostTickProcess.closeClient sends LOGOUT and flushes only high priority.
    pub fn logout(&mut self, id: u64) -> io::Result<()> {
        if let Some(c) = self.connections.get_mut(&id) {
            use gielinor_protocol::framing::{encoded_len, OpcodeWidth};
            // Bytes already encoded by earlier completed batches cannot be withdrawn.
            let discarded: usize = c.pending[1]
                .iter()
                .map(|packet| {
                    let def = *gielinor_protocol::tables::SERVER
                        .iter()
                        .find(|def| def.opcode == packet.opcode)
                        .unwrap();
                    encoded_len(def, packet.payload.len(), OpcodeWidth::Smart)
                        .expect("Queued packet already validated")
                })
                .sum();
            c.pending[1].clear();
            c.pending_bytes -= discarded;
        }
        let result = self.send(id, 57, &[]);
        self.disconnect(id);
        result
    }
    pub fn disconnect(&mut self, id: u64) {
        if let Some(c) = self.connections.get_mut(&id) {
            // Preserve already queued logout output before close destroys ISAAC state.
            let _ = Self::flush_connection(c, self.limits.output_bytes);
            c.session.close();
            c.closing = true;
        }
    }
    pub fn poll(
        &mut self,
        mut seed: impl FnMut() -> io::Result<u64>,
    ) -> io::Result<Vec<NetworkEvent>> {
        let now = Instant::now();
        let mut events = Vec::new();
        for _ in 0..if self.accepting { 16 } else { 0 } {
            match self
                .listener
                .as_ref()
                .expect("Accepting transport has a listener")
                .accept()
            {
                Ok((socket, _)) => {
                    if self.connections.len() >= self.limits.connections {
                        drop(socket);
                        continue;
                    }
                    socket.set_nonblocking(true)?;
                    socket.set_nodelay(true)?;
                    self.next_id = self
                        .next_id
                        .checked_add(1)
                        .ok_or_else(|| io::Error::other("Connection ids exhausted"))?;
                    self.connections.insert(
                        self.next_id,
                        Connection {
                            socket,
                            session: Session::new(seed()?, self.crcs),
                            output: vec![],
                            pending: [vec![], vec![]],
                            pending_bytes: 0,
                            written: 0,
                            created: now,
                            last_read: now,
                            closing: false,
                            js5: crate::js5::Client::new(now),
                        },
                    );
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }
        let mut removed = Vec::new();
        for (id, c) in &mut self.connections {
            let mut reason = None;
            let timed_out = if matches!(c.session.state(), State::Active | State::Js5) {
                now.duration_since(c.last_read) > self.limits.idle_timeout
            } else {
                now.duration_since(c.created) > self.limits.login_timeout
            };
            if timed_out {
                reason = Some("Session timed out".into());
            }
            if reason.is_none() && !c.closing {
                let mut buffer = [0u8; 8192];
                for _ in 0..8 {
                    let size = match c.socket.read(&mut buffer) {
                        Ok(0) => {
                            reason = Some("Peer disconnected".into());
                            break;
                        }
                        Ok(n) => n,
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                        Err(e) => {
                            reason = Some(e.to_string());
                            break;
                        }
                    };
                    c.last_read = now;
                    match c.session.receive(&buffer[..size], &self.key) {
                        Ok(packets) => {
                            for packet in packets {
                                match packet {
                                    Event::Write(bytes) => {
                                        if let Err(e) =
                                            Self::append(c, &bytes, self.limits.output_bytes)
                                        {
                                            reason = Some(e.to_string());
                                        }
                                    }
                                    Event::Authenticate(login) => {
                                        events.push(NetworkEvent::Authenticate {
                                            connection: *id,
                                            login,
                                        })
                                    }
                                    Event::Game(packet) => events.push(NetworkEvent::Packet {
                                        connection: *id,
                                        packet,
                                    }),
                                    Event::Js5(command) => {
                                        if let Err(e) = c.js5.push(command) {
                                            reason = Some(e.to_string());
                                        }
                                    }
                                    Event::Rejected(_) => c.closing = true,
                                }
                            }
                        }
                        Err(e) => {
                            reason = Some(format!("Protocol error: {e:?}"));
                        }
                    }
                    if reason.is_some() || c.closing {
                        break;
                    }
                }
                // An authentication transition may have retained an already received game packet.
                if reason.is_none() && c.session.state() == State::Active {
                    match c.session.receive(&[], &self.key) {
                        Ok(packets) => {
                            for event in packets {
                                if let Event::Game(packet) = event {
                                    events.push(NetworkEvent::Packet {
                                        connection: *id,
                                        packet,
                                    });
                                }
                            }
                        }
                        Err(e) => reason = Some(format!("Protocol error: {e:?}")),
                    }
                }
            }
            if reason.is_none() && c.session.state() == State::Js5 {
                if let Some(provider) = self.js5.as_mut() {
                    let budget = self
                        .limits
                        .output_bytes
                        .saturating_sub(c.output.len() - c.written)
                        .min(65536);
                    match c.js5.output(provider.as_mut(), now, budget) {
                        Ok(bytes) => {
                            if let Err(e) = Self::append(c, &bytes, self.limits.output_bytes) {
                                reason = Some(e.to_string());
                            }
                        }
                        Err(e) => reason = Some(e.to_string()),
                    }
                } else {
                    reason = Some("JS5 provider not configured".into());
                }
            }
            if reason.is_none() {
                let end = c.output.len().min(c.written + 65536);
                while c.written < end {
                    match c.socket.write(&c.output[c.written..end]) {
                        Ok(0) => {
                            reason = Some("Socket write closed".into());
                            break;
                        }
                        Ok(n) => c.written += n,
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                        Err(e) => {
                            reason = Some(e.to_string());
                            break;
                        }
                    }
                }
                if c.written == c.output.len() {
                    c.output.clear();
                    c.written = 0;
                    if c.closing {
                        reason = Some("Session closed".into());
                    }
                }
            }
            if let Some(reason) = reason {
                removed.push(*id);
                events.push(NetworkEvent::Disconnected {
                    connection: *id,
                    reason,
                });
            }
        }
        for id in removed {
            self.connections.remove(&id);
        }
        Ok(events)
    }
}
