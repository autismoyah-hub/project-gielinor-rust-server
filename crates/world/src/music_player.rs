//! MusicPlayer and MusicTimerScript state changes. The host supplies random draws
//! and consumes ordered effects; all durable settings remain ordinary variables.
use crate::{
    bank_actions::Binding,
    music::{Catalog, Event},
    music_playlist::Playlist,
    npc::RandomSource,
    variables::{Variables, Varp},
};
#[derive(Clone, Debug)]
pub struct Config {
    pub catalog: Catalog,
    pub mode: Varp,
    pub playlist: Varp,
    pub volume: Varp,
    pub area_mode: Binding,
    pub area: Binding,
    pub current: Binding,
    pub last: Binding,
    pub clocks: Binding,
    pub duration: Binding,
    pub suppress_message: Binding,
    pub transparency: Binding,
    pub stop_midi: i32,
    pub timer: u16,
}
#[derive(Clone, Copy)]
pub struct Context {
    pub logged_in: bool,
    pub resizable: bool,
}
/// Kotlin writes effects immediately. A later script exception does not undo
/// those writes, so the host must deliver these events before handling failure.
#[derive(Debug)]
pub struct Failure {
    pub reason: String,
    pub events: Vec<Event>,
}
impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.reason.fmt(f)
    }
}
struct Session<'a> {
    config: &'a Config,
    vars: &'a mut Variables,
    context: Context,
    events: Vec<Event>,
}
impl Session<'_> {
    fn run(
        mut self,
        action: impl FnOnce(&mut Self) -> Result<(), String>,
    ) -> Result<Vec<Event>, Failure> {
        match action(&mut self) {
            Ok(()) => Ok(self.events),
            Err(reason) => Err(Failure {
                reason,
                events: self.events,
            }),
        }
    }
    fn get(&self, bit: Binding) -> Result<i32, String> {
        self.vars
            .bit(bit.bit)
            .map_err(|e| format!("Music varbit: {e:?}"))
    }
    fn set(&mut self, bit: Binding, value: i32) -> Result<(), String> {
        if let Some(update) = self
            .vars
            .set_bit(bit.base, bit.bit, value, self.context.logged_in)
            .map_err(|e| format!("Music varbit: {e:?}"))?
        {
            self.events.push(Event::Variable(update));
        }
        Ok(())
    }
    fn set_varp(&mut self, varp: Varp, value: i32) {
        if let Some(update) = self.vars.set(varp, value, self.context.logged_in) {
            self.events.push(Event::Variable(update));
        }
    }
    fn unlock(&mut self, id: u16) -> Result<(), String> {
        let track = self
            .config
            .catalog
            .tracks
            .get(&id)
            .ok_or_else(|| format!("Unknown music {id}"))?;
        let muted = self.get(self.config.suppress_message)? == 1;
        let transparent = self.context.resizable && self.get(self.config.transparency)? == 1;
        self.events.extend(
            track
                .unlock(self.vars, muted, transparent, self.context.logged_in)
                .map_err(|_| format!("Music cannot be unlocked: {id}"))?,
        );
        Ok(())
    }
    fn play(&mut self, id: u16) -> Result<(), String> {
        let track = self
            .config
            .catalog
            .tracks
            .get(&id)
            .ok_or_else(|| format!("Unknown music {id}"))?;
        let current = self.get(self.config.current)?;
        if current != id as i32 {
            self.set(self.config.clocks, 0)?;
            self.set(self.config.duration, track.duration)?;
        }
        self.set(self.config.current, id as i32)?;
        let fade = if current == 0 { 0 } else { 60 };
        self.events.push(Event::Midi {
            id: track.midi,
            fade_out_speed: fade,
            fade_in_delay: fade,
        });
        self.events.push(Event::NowPlaying(track.name.clone()));
        Ok(())
    }
    fn unlock_play(&mut self, id: u16) -> Result<(), String> {
        self.unlock(id)?;
        self.play(id)
    }
    fn shuffled(&mut self, tracks: &[u16], random: &mut impl RandomSource) -> Result<(), String> {
        if self.vars.get(self.config.playlist.id) == 0 {
            self.set_varp(
                self.config.playlist,
                Playlist::new(0, 0, random.below(65536))
                    .map_err(|e| format!("Music playlist: {e:?}"))?
                    .0,
            );
        }
        let list = Playlist(self.vars.get(self.config.playlist.id));
        let id = list
            .track(tracks)
            .map_err(|e| format!("Music playlist: {e:?}"))?;
        self.unlock_play(id)?;
        self.set_varp(
            self.config.playlist,
            list.next(tracks.len())
                .map_err(|e| format!("Music playlist: {e:?}"))?
                .0,
        );
        Ok(())
    }
    fn enter(&mut self, area: u16, random: &mut impl RandomSource) -> Result<(), String> {
        self.set_varp(self.config.playlist, 0);
        if self.get(self.config.area_mode)? == 1 {
            if let Some(&track) = self.config.catalog.classic.get(&area) {
                self.set(self.config.area, area as i32 + 1)?;
                self.unlock_play(track)?;
            }
        } else if let Some(tracks) = self.config.catalog.modern.get(&area) {
            self.set(self.config.area, area as i32 + 1)?;
            for &track in tracks {
                self.unlock(track)?;
            }
            if self.vars.get(self.config.mode.id) == 1 {
                self.shuffled(tracks, random)?;
            }
        }
        Ok(())
    }
    fn stop(&mut self) -> Result<(), String> {
        self.set(self.config.last, self.get(self.config.current)?)?;
        self.set(self.config.current, 0)?;
        self.set(self.config.clocks, 0)?;
        self.set(self.config.duration, 0)?;
        self.events.push(Event::Midi {
            id: self.config.stop_midi,
            fade_out_speed: 20,
            fade_in_delay: 0,
        });
        if !matches!(self.vars.get(self.config.mode.id), 1 | 2) {
            self.events.push(Event::NowPlaying(" ".into()));
        }
        Ok(())
    }
    fn next(&mut self, random: &mut impl RandomSource) -> Result<(), String> {
        match self.vars.get(self.config.mode.id) {
            1 => {
                let area = self.get(self.config.area)?;
                if area == 0 {
                    return Ok(());
                }
                let area = u16::try_from(area - 1).map_err(|_| "Invalid music area")?;
                if self.get(self.config.area_mode)? == 1 {
                    match self.config.catalog.classic.get(&area) {
                        Some(&id) if self.get(self.config.last)? != id as i32 => {
                            self.unlock_play(id)?
                        }
                        _ => self.events.push(Event::NowPlaying(" ".into())),
                    }
                } else if let Some(tracks) = self.config.catalog.modern.get(&area) {
                    self.shuffled(tracks, random)?;
                }
            }
            2 => {
                let current = self.get(self.config.current)?;
                let tracks: Vec<_> = self
                    .config
                    .catalog
                    .all
                    .iter()
                    .copied()
                    .filter(|id| {
                        *id as i32 != current && self.config.catalog.tracks[id].unlocked(self.vars)
                    })
                    .collect();
                let id = if tracks.is_empty() {
                    *self
                        .config
                        .catalog
                        .all
                        .first()
                        .ok_or("Empty music catalog")?
                } else {
                    tracks[random.below(tracks.len() as u32) as usize]
                };
                self.play(id)?;
            }
            _ => {}
        }
        Ok(())
    }
}
impl Config {
    fn session<'a>(&'a self, vars: &'a mut Variables, context: Context) -> Session<'a> {
        Session {
            config: self,
            vars,
            context,
            events: Vec::new(),
        }
    }
    pub fn enter(
        &self,
        vars: &mut Variables,
        context: Context,
        area: u16,
        random: &mut impl RandomSource,
    ) -> Result<Vec<Event>, Failure> {
        self.session(vars, context)
            .run(|session| session.enter(area, random))
    }
    pub fn tick(
        &self,
        vars: &mut Variables,
        context: Context,
        random: &mut impl RandomSource,
    ) -> Result<Vec<Event>, Failure> {
        self.session(vars, context).run(|session| {
            if session.vars.get(self.volume.id) == 0 {
                return Ok(());
            }
            if session.get(self.current)? == 0 {
                session.next(random)?;
            } else {
                let clock = session.get(self.clocks)?.wrapping_add(1);
                session.set(self.clocks, clock)?;
                if session.get(self.clocks)? > session.get(self.duration)? {
                    session.stop()?;
                }
            }
            Ok(())
        })
    }
    pub fn play(
        &self,
        vars: &mut Variables,
        context: Context,
        id: u16,
        unlock: bool,
    ) -> Result<Vec<Event>, Failure> {
        self.session(vars, context).run(|session| {
            if unlock {
                session.unlock(id)?;
            }
            session.play(id)?;
            Ok(())
        })
    }
    pub fn resume(&self, vars: &mut Variables, context: Context) -> Result<Vec<Event>, Failure> {
        self.session(vars, context).run(|session| {
            let current = session.get(self.current)? as u16;
            if self.catalog.tracks.contains_key(&current) {
                session.set(self.current, 0)?;
                session.play(current)?;
            } else {
                session.stop()?;
                session.events.push(Event::NowPlaying(" ".into()));
            }
            Ok(())
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        music::Track,
        variables::{Transmit, Varbit},
    };
    fn varp(id: u16) -> Varp {
        Varp {
            id,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        }
    }
    fn binding(id: u16) -> Binding {
        Binding {
            base: varp(id),
            bit: Varbit {
                base: id,
                lsb: 0,
                msb: 15,
            },
        }
    }
    pub(crate) fn config() -> Config {
        let mut catalog = Catalog::default();
        for id in [1, 2, 3] {
            catalog.tracks.insert(
                id,
                Track {
                    id,
                    row: id as u32,
                    name: format!("Track{id}"),
                    unlock_hint: String::new(),
                    duration: 2,
                    midi: id as i32 + 100,
                    unlock_varp: Some(varp(20)),
                    unlock_bit: id as i32,
                    hidden: false,
                    secondary: None,
                },
            );
        }
        catalog.modern.insert(9, vec![1, 2]);
        catalog.classic.insert(9, 3);
        catalog.all = vec![2, 3, 1];
        Config {
            catalog,
            mode: varp(30),
            playlist: varp(31),
            volume: varp(32),
            area_mode: binding(40),
            area: binding(41),
            current: binding(42),
            last: binding(43),
            clocks: binding(44),
            duration: binding(45),
            suppress_message: binding(46),
            transparency: binding(47),
            stop_midi: -1,
            timer: 3,
        }
    }
    struct Draws(Vec<u32>);
    impl RandomSource for Draws {
        fn below(&mut self, bound: u32) -> u32 {
            let n = self.0.remove(0);
            assert!(n < bound);
            n
        }
    }
    const CONTEXT: Context = Context {
        logged_in: true,
        resizable: false,
    };
    #[test]
    fn modern_unlocks_all_even_manual_and_classic_plays_even_manual() {
        let config = config();
        let mut vars = Variables::default();
        let mut random = Draws(vec![]);
        let events = config.enter(&mut vars, CONTEXT, 9, &mut random).unwrap();
        assert_eq!(vars.get(20), 6);
        assert_eq!(vars.get(41), 10);
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::Message(_)))
                .count(),
            2
        );
        assert!(!events.iter().any(|e| matches!(e, Event::Midi { .. })));
        vars.set(varp(40), 1, false);
        let events = config.enter(&mut vars, CONTEXT, 9, &mut random).unwrap();
        assert_eq!(vars.get(20), 14);
        assert_eq!(vars.get(42), 3);
        assert!(events.contains(&Event::Midi {
            id: 103,
            fade_out_speed: 0,
            fade_in_delay: 0
        }));
        let events = config.play(&mut vars, CONTEXT, 3, false).unwrap();
        assert!(!events
            .iter()
            .any(|e| matches!(e, Event::Variable(u) if u.id == 44 || u.id == 45)));
        assert!(events.contains(&Event::Midi {
            id: 103,
            fade_out_speed: 60,
            fade_in_delay: 60
        }));
    }
    #[test]
    fn muted_timer_pause_end_boundary_and_classic_no_repeat() {
        let config = config();
        let mut vars = Variables::default();
        let mut random = Draws(vec![]);
        vars.set(varp(40), 1, false);
        vars.set(varp(30), 1, false);
        config.enter(&mut vars, CONTEXT, 9, &mut random).unwrap();
        assert!(config
            .tick(&mut vars, CONTEXT, &mut random)
            .unwrap()
            .is_empty());
        assert_eq!(vars.get(44), 0);
        vars.set(varp(32), 255, false);
        for expected in [1, 2] {
            config.tick(&mut vars, CONTEXT, &mut random).unwrap();
            assert_eq!(vars.get(44), expected);
            assert_eq!(vars.get(42), 3);
        }
        let events = config.tick(&mut vars, CONTEXT, &mut random).unwrap();
        assert_eq!(vars.get(42), 0);
        assert_eq!(vars.get(43), 3);
        assert!(events.contains(&Event::Midi {
            id: -1,
            fade_out_speed: 20,
            fade_in_delay: 0
        }));
        assert_eq!(
            config.tick(&mut vars, CONTEXT, &mut random).unwrap(),
            [Event::NowPlaying(" ".into())]
        );
    }
    #[test]
    fn modern_cursor_advances_random_uses_repository_order_and_resume_no_fade() {
        let config = config();
        let mut vars = Variables::default();
        let mut random = Draws(vec![2391, 0]);
        vars.set(varp(30), 1, false);
        config.enter(&mut vars, CONTEXT, 9, &mut random).unwrap();
        assert_eq!(vars.get(31) & 255, 1);
        let current = vars.get(42);
        let events = config.resume(&mut vars, CONTEXT).unwrap();
        assert!(events.contains(&Event::Midi {
            id: current + 100,
            fade_out_speed: 0,
            fade_in_delay: 0
        }));
        vars.set(varp(30), 2, false);
        vars.set(varp(32), 1, false);
        vars.set(varp(42), 0, false);
        config.tick(&mut vars, CONTEXT, &mut random).unwrap();
        assert_eq!(vars.get(42), 2);
        assert!(random.0.is_empty());
    }
}
