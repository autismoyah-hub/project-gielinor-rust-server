//! Stat presentation, with explicit invalidation for events that resend unchanged XP.
use gielinor_protocol::{buffer::DecodeError, framing::Packet, player_sync::update_stat};
use gielinor_world::stats::Stats;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Default)]
pub struct Observer {
    known: BTreeMap<u8, (u8, u8, i32)>,
    forced: BTreeSet<u8>,
}
impl Observer {
    pub fn invalidate(&mut self, stat: u8) {
        self.forced.insert(stat);
    }
    pub fn cycle(
        &mut self,
        stats: &Stats,
        ids: impl IntoIterator<Item = u8>,
        mut invisible: impl FnMut(u8) -> i32,
    ) -> Result<Vec<Packet>, DecodeError> {
        let mut next = self.clone();
        let mut packets = vec![];
        for id in ids {
            let current = stats.current(id) as u8;
            let hidden =
                u8::try_from((current as i32).wrapping_add(invisible(id))).map_err(|_| {
                    DecodeError::InvalidValue("Invisible stat level outside protocol range")
                })?;
            let value = (current, hidden, stats.xp(id));
            if next.forced.remove(&id) || next.known.get(&id) != Some(&value) {
                packets.push(Packet {
                    opcode: 46,
                    payload: update_stat(id, current, hidden, value.2).to_vec(),
                });
                next.known.insert(id, value);
            }
        }
        *self = next;
        Ok(packets)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unsigned_levels_invisible_modifiers_and_forced_max_xp_resends() {
        let mut stats = Stats::default();
        stats.set_current(3, 200u8 as i8);
        stats.set_xp(3, Stats::MAX_XP).unwrap();
        let mut observer = Observer::default();
        assert_eq!(
            observer.cycle(&stats, [3], |_| 5).unwrap()[0].payload[..3],
            [205, 200, 131]
        );
        assert!(observer.cycle(&stats, [3], |_| 5).unwrap().is_empty());
        observer.invalidate(3);
        assert!(observer.cycle(&stats, [3], |_| 100).is_err());
        assert_eq!(observer.cycle(&stats, [3], |_| 5).unwrap().len(), 1);
        stats.set_current(3, 199u8 as i8);
        assert_eq!(
            observer.cycle(&stats, [3], |_| 5).unwrap()[0].payload[0],
            204
        );
    }
}
