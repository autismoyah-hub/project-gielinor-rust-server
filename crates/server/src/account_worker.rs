//! A single bounded worker owns SQLite and Argon2; neither can block world ticks.
use crate::authentication::{Authenticated, Config, Service};
use gielinor_protocol::login::Login;
use gielinor_storage::{
    accounts::{Accounts, Save},
    password::PasswordHasher,
    sqlite::Database,
};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError},
    thread::{self, JoinHandle},
};

pub struct Settings {
    pub database: PathBuf,
    pub argon2_library: PathBuf,
    pub authentication: Config,
    pub queue_capacity: usize,
}
pub enum Request {
    Authenticate {
        connection: u64,
        login: Login,
        unix_seconds: u64,
        local_datetime: String,
    },
    Save {
        token: u64,
        data: Save,
    },
    /// Same transaction owner, with in-session rather than logout metadata.
    Checkpoint { token:u64, data:Save },
}
pub enum Completion {
    Authenticated {
        connection: u64,
        result: Result<Authenticated, u8>,
    },
    Saved {
        token: u64,
        result: Result<(), String>,
    },
}
pub struct Worker {
    pub login_messages: gielinor_storage::accounts::LoginMessages,
    requests: Option<SyncSender<Request>>,
    completed: Option<Receiver<Completion>>,
    thread: Option<JoinHandle<()>>,
}
impl Worker {
    pub fn start(settings: Settings) -> Result<Self, String> {
        if !(1..=1024).contains(&settings.queue_capacity) {
            return Err("Invalid account queue capacity".into());
        }
        let (requests, receive) = mpsc::sync_channel(settings.queue_capacity);
        let (complete, completed) = mpsc::sync_channel(settings.queue_capacity);
        let (ready, started) = mpsc::sync_channel(1);
        let thread = thread::Builder::new()
            .name("gielinor-accounts".into())
            .spawn(move || {
                let service = (|| {
                    let db = Database::open(&settings.database, false).map_err(|e| e.message)?;
                    let mut accounts = Accounts::from_database(db);
                    accounts.initialize().map_err(|e| e.message)?;
                    let messages = accounts
                        .login_messages(settings.authentication.realm)
                        .map_err(|e| e.message)?;
                    let passwords = PasswordHasher::load(&settings.argon2_library)
                        .map_err(|e| format!("{e:?}"))?;
                    Ok::<_, String>((
                        Service {
                            accounts,
                            passwords,
                            config: settings.authentication,
                        },
                        messages,
                    ))
                })();
                let mut service = match service {
                    Ok((service, messages)) => {
                        let _ = ready.send(Ok(messages));
                        service
                    }
                    Err(error) => {
                        let _ = ready.send(Err(error));
                        return;
                    }
                };
                while let Ok(request) = receive.recv() {
                    let result = match request {
                        Request::Authenticate {
                            connection,
                            login,
                            unix_seconds,
                            local_datetime,
                        } => Completion::Authenticated {
                            connection,
                            result: service.authenticate(login, unix_seconds, &local_datetime),
                        },
                        Request::Checkpoint {token,data} => Completion::Saved {
                            token,
                            result:service.accounts.save_checkpoint(&data).map_err(|e|e.message),
                        },
                        Request::Save { token, data } => Completion::Saved {
                            token,
                            result: service.accounts.save(&data).map_err(|e| e.message),
                        },
                    };
                    // On shutdown the receiver may be gone. Still finish already accepted saves.
                    let _ = complete.send(result);
                }
            })
            .map_err(|e| e.to_string())?;
        match started.recv() {
            Ok(Ok(login_messages)) => Ok(Self {
                login_messages,
                requests: Some(requests),
                completed: Some(completed),
                thread: Some(thread),
            }),
            result => {
                let _ = thread.join();
                Err(match result {
                    Ok(Err(error)) => error,
                    _ => "Account worker exited during startup".into(),
                })
            }
        }
    }
    /// A full queue returns ownership, allowing saves to remain pending without data loss.
    pub fn submit(&self, request: Request) -> Result<(), TrySendError<Request>> {
        match &self.requests {
            Some(sender) => sender.try_send(request),
            None => Err(TrySendError::Disconnected(request)),
        }
    }
    pub fn poll(&self) -> Result<Completion, TryRecvError> {
        self.completed
            .as_ref()
            .ok_or(TryRecvError::Disconnected)?
            .try_recv()
    }
    /// Finish every accepted request and return every remaining completion before joining.
    pub fn shutdown(mut self) -> Result<Vec<Completion>, String> {
        self.requests.take();
        let events = self.completed.take().unwrap().into_iter().collect();
        self.thread
            .take()
            .unwrap()
            .join()
            .map_err(|_| "Account worker panicked".to_string())?;
        Ok(events)
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
