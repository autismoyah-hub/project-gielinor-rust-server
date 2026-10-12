//! Bounded JS5 queues and an isolated read-only disk worker. World/network pumps
//! never wait for disk I/O; repeated group requests share immutable responses.
use gielinor_protocol::js5::Command;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    io,
    path::PathBuf,
    sync::{
        mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
type Key = (u8, u16);
pub trait Provider {
    fn response(&mut self, archive: u8, group: u16) -> io::Result<Option<Arc<[u8]>>>;
}
type Loaded = (Key, Result<Arc<[u8]>, String>);
pub struct Worker {
    requests: Option<SyncSender<Key>>,
    completed: Option<Receiver<Loaded>>,
    thread: Option<JoinHandle<()>>,
    pending: BTreeSet<Key>,
    cache: BTreeMap<Key, Result<Arc<[u8]>, String>>,
    order: VecDeque<Key>,
    bytes: usize,
    budget: usize,
    pub checksums: [u32; 23],
}
impl Worker {
    pub fn start(path: PathBuf, group_limit: usize, cache_budget: usize) -> io::Result<Self> {
        if !(1..=0xff_ffff).contains(&group_limit) {
            return Err(io::Error::other("Invalid JS5 group limit"));
        }
        if cache_budget < group_limit + group_limit / 511 + 4 {
            return Err(io::Error::other(
                "JS5 cache budget smaller than maximum response",
            ));
        }
        let (requests, receive) = mpsc::sync_channel::<Key>(32);
        let (send, completed) = mpsc::sync_channel::<Loaded>(32);
        let (ready, started) = mpsc::sync_channel(1);
        let thread = thread::Builder::new()
            .name("gielinor-js5".into())
            .spawn(move || {
                let mut store = match gielinor_cache::js5::Store::open(&path, group_limit) {
                    Ok(store) => {
                        let _ = ready.send(Ok(store.checksums));
                        store
                    }
                    Err(e) => {
                        let _ = ready.send(Err(e.to_string()));
                        return;
                    }
                };
                while let Ok(key) = receive.recv() {
                    let result = store
                        .response(key.0, key.1)
                        .map(Arc::from)
                        .map_err(|e| e.to_string());
                    if send.send((key, result)).is_err() {
                        break;
                    }
                }
            })?;
        let checksums = match started.recv() {
            Ok(Ok(v)) => v,
            result => {
                let _ = thread.join();
                return Err(io::Error::other(match result {
                    Ok(Err(e)) => e,
                    _ => "JS5 worker exited at startup".into(),
                }));
            }
        };
        Ok(Self {
            requests: Some(requests),
            completed: Some(completed),
            thread: Some(thread),
            pending: BTreeSet::new(),
            cache: BTreeMap::new(),
            order: VecDeque::new(),
            bytes: 0,
            budget: cache_budget,
            checksums,
        })
    }
    fn collect(&mut self) -> io::Result<()> {
        loop {
            match self.completed.as_ref().unwrap().try_recv() {
                Ok((key, result)) => {
                    self.pending.remove(&key);
                    let size = result.as_ref().map_or(0, |b| b.len());
                    while self.bytes + size > self.budget || self.cache.len() >= 512 {
                        let Some(old) = self.order.pop_front() else {
                            break;
                        };
                        if let Some(Ok(bytes)) = self.cache.remove(&old) {
                            self.bytes -= bytes.len();
                        }
                    }
                    self.bytes += size;
                    self.order.push_back(key);
                    self.cache.insert(key, result);
                }
                Err(TryRecvError::Empty) => return Ok(()),
                Err(TryRecvError::Disconnected) => {
                    return Err(io::Error::other("JS5 worker disconnected"))
                }
            }
        }
    }
}
impl Provider for Worker {
    fn response(&mut self, archive: u8, group: u16) -> io::Result<Option<Arc<[u8]>>> {
        self.collect()?;
        let key = (archive, group);
        if let Some(result) = self.cache.get(&key) {
            return result
                .as_ref()
                .map(|b| Some(Arc::clone(b)))
                .map_err(|e| io::Error::other(e.clone()));
        }
        if !self.pending.contains(&key) {
            match self.requests.as_ref().unwrap().try_send(key) {
                Ok(()) => {
                    self.pending.insert(key);
                }
                Err(TrySendError::Full(_)) => (),
                Err(TrySendError::Disconnected(_)) => {
                    return Err(io::Error::other("JS5 worker disconnected"))
                }
            }
        }
        Ok(None)
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.requests.take();
        self.completed.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
pub struct Client {
    urgent: VecDeque<Key>,
    prefetch: VecDeque<Key>,
    awaiting: VecDeque<Key>,
    current: Option<(Arc<[u8]>, usize)>,
    high: bool,
    low_changes: i32,
    xor: u8,
    last_prefetch: Instant,
}
impl Client {
    pub fn new(now: Instant) -> Self {
        Self {
            urgent: VecDeque::new(),
            prefetch: VecDeque::new(),
            awaiting: VecDeque::new(),
            current: None,
            high: false,
            low_changes: 0,
            xor: 0,
            last_prefetch: now,
        }
    }
    pub fn push(&mut self, command: Command) -> io::Result<()> {
        match command {
            Command::Request {
                archive,
                group,
                urgent,
            } => {
                if self.urgent.len() >= 200 || self.prefetch.len() + self.awaiting.len() >= 201 {
                    return Err(io::Error::other("JS5 request queue full"));
                }
                let key = (archive, group);
                if urgent {
                    // Source deque.remove removes one occurrence, not all duplicates.
                    for queue in [&mut self.prefetch, &mut self.awaiting] {
                        if let Some(i) = queue.iter().position(|k| *k == key) {
                            queue.remove(i);
                        }
                    }
                    self.urgent.push_back(key);
                } else if !self.high && self.low_changes >= 2 {
                    self.prefetch.push_back(key);
                } else {
                    self.awaiting.push_back(key);
                }
            }
            Command::Priority { high } => {
                self.high = high;
                if !high {
                    self.low_changes = self.low_changes.wrapping_add(1);
                    if self.low_changes >= 2 {
                        self.prefetch.append(&mut self.awaiting);
                    }
                }
            }
            Command::Xor(key) => self.xor = key,
        }
        Ok(())
    }
    /// RSProt only releases background requests when the client is otherwise idle.
    /// The reference compares each group's size, rather than the accumulated size,
    /// against the threshold. Preserve that distinction for small-group batches.
    pub fn transfer_prefetch(
        &mut self,
        provider: &mut dyn Provider,
        threshold: i32,
    ) -> io::Result<bool> {
        if self.awaiting.is_empty()
            || !self.urgent.is_empty()
            || !self.prefetch.is_empty()
            || self.current.is_some()
        {
            return Ok(false);
        }
        let mut bytes = 0i32;
        while let Some(&(archive, group)) = self.awaiting.front() {
            let Some(response) = provider.response(archive, group)? else {
                break;
            };
            let size = i32::try_from(response.len())
                .map_err(|_| io::Error::other("JS5 response too large"))?;
            self.awaiting.pop_front();
            self.prefetch.push_back((archive, group));
            bytes = bytes.wrapping_add(size);
            if size >= threshold {
                break;
            }
        }
        Ok(bytes > 0)
    }
    /// Emit at most the caller's per-pump byte budget. A partial group finishes
    /// before another group, so continuation markers retain their wire position.
    pub fn output(
        &mut self,
        provider: &mut dyn Provider,
        now: Instant,
        budget: usize,
    ) -> io::Result<Vec<u8>> {
        if budget > 0 && now.duration_since(self.last_prefetch) >= Duration::from_millis(200) {
            self.transfer_prefetch(provider, 8192)?;
            self.last_prefetch = now;
        }
        let mut output = Vec::new();
        while output.len() < budget {
            if self.current.is_none() {
                let queue = if self.urgent.is_empty() {
                    &mut self.prefetch
                } else {
                    &mut self.urgent
                };
                let Some(&(archive, group)) = queue.front() else {
                    break;
                };
                let Some(bytes) = provider.response(archive, group)? else {
                    break;
                };
                if bytes.is_empty() {
                    return Err(io::Error::other("Empty JS5 response"));
                }
                queue.pop_front();
                self.current = Some((bytes, 0));
            }
            let (bytes, position) = self.current.as_mut().unwrap();
            let count = (budget - output.len()).min(bytes.len() - *position);
            output.extend(
                bytes[*position..*position + count]
                    .iter()
                    .map(|b| *b ^ self.xor),
            );
            *position += count;
            if *position == bytes.len() {
                self.current = None;
            }
        }
        Ok(output)
    }
}
