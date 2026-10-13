//! Cache-backed music identity and unlock effects. Playback/area dispatch is
//! supplied by the world host; this policy is also usable by headless actors.
use crate::variables::{Update, Variables, Varp};
use std::collections::BTreeMap;
#[derive(Clone, Debug)]
pub struct Track {
    pub id: u16,
    pub row: u32,
    pub name: String,
    pub unlock_hint: String,
    pub duration: i32,
    pub midi: i32,
    pub unlock_varp: Option<Varp>,
    pub unlock_bit: i32,
    pub hidden: bool,
    pub secondary: Option<u32>,
}
#[derive(Clone, Debug, Default)]
pub struct Catalog {
    pub tracks: BTreeMap<u16, Track>,
    pub rows: BTreeMap<u32, u16>,
    pub modern: BTreeMap<u16, Vec<u16>>,
    pub classic: BTreeMap<u16, u16>,
    /// MusicRepository.getAll(): hash-map values order, not music ID order.
    pub all: Vec<u16>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Message(String),
    Variable(Update),
    Midi {
        id: i32,
        fade_out_speed: u16,
        fade_in_delay: u16,
    },
    NowPlaying(String),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CannotUnlock;
impl Track {
    pub fn unlocked(&self, vars: &Variables) -> bool {
        self.unlock_varp
            .is_some_and(|varp| vars.get(varp.id) & 1i32.wrapping_shl(self.unlock_bit as u32) != 0)
    }
    pub fn unlock(
        &self,
        vars: &mut Variables,
        suppress_message: bool,
        transparent_chatbox: bool,
        logged_in: bool,
    ) -> Result<Vec<Event>, CannotUnlock> {
        let varp = self.unlock_varp.ok_or(CannotUnlock)?;
        if self.unlocked(vars) {
            return Ok(Vec::new());
        }
        let value = vars.get(varp.id) | 1i32.wrapping_shl(self.unlock_bit as u32);
        let mut events = Vec::new();
        if !suppress_message {
            let color = if transparent_chatbox {
                "ff3045"
            } else {
                "e00a19"
            };
            events.push(Event::Message(format!(
                "You have unlocked a new music track: <col={color}>{}",
                self.name
            )));
        }
        if let Some(update) = vars.set(varp, value, logged_in) {
            events.push(Event::Variable(update));
        }
        Ok(events)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unlock_is_once_only_preserves_bits_and_suppresses_text_without_suppressing_state() {
        let mut track = Track {
            id: 1,
            row: 10,
            name: "Harmony".into(),
            unlock_hint: String::new(),
            duration: 100,
            midi: 1,
            unlock_varp: Some(Varp {
                id: 20,
                permanent: true,
                transmit: crate::variables::Transmit::Different,
                bit_protect: false,
                client_code: -1,
            }),
            unlock_bit: 31,
            hidden: false,
            secondary: None,
        };
        let mut vars = Variables::default();
        vars.set(track.unlock_varp.unwrap(), 5, false);
        let events = track.unlock(&mut vars, false, true, true).unwrap();
        assert_eq!(
            events,
            vec![
                Event::Message("You have unlocked a new music track: <col=ff3045>Harmony".into()),
                Event::Variable(Update {
                    id: 20,
                    value: i32::MIN | 5
                })
            ]
        );
        assert!(track
            .unlock(&mut vars, false, false, true)
            .unwrap()
            .is_empty());
        track.unlock_bit = 34;
        assert!(track.unlocked(&vars));
        track.unlock_bit = 1;
        assert_eq!(
            track.unlock(&mut vars, true, false, true).unwrap(),
            vec![Event::Variable(Update {
                id: 20,
                value: i32::MIN | 7
            })]
        );
        track.unlock_varp = None;
        assert_eq!(
            track.unlock(&mut vars, true, false, true),
            Err(CannotUnlock)
        );
    }
}
