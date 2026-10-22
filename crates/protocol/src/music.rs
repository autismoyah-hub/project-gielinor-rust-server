//! Revision239 MIDI_SONG_V2; stop_music is an ordinary cache MIDI identity.
use crate::framing::Packet;
/// Revision 239 MIDI_JINGLE: unsigned medium duration, then g2Alt3 identity.
pub fn jingle(id: u16) -> Packet {
    let [hi, lo] = id.to_be_bytes();
    Packet { opcode: 118, payload: vec![0, 0, 0, lo.wrapping_add(128), hi] }
}

pub fn song(
    id: u16,
    fade_out_delay: u16,
    fade_out_speed: u16,
    fade_in_delay: u16,
    fade_in_speed: u16,
) -> Packet {
    let [id_hi, id_lo] = id.to_be_bytes();
    let [fade_hi, fade_lo] = fade_out_speed.to_be_bytes();
    let mut payload = fade_in_delay.to_le_bytes().to_vec();
    payload.extend(fade_out_delay.to_be_bytes());
    payload.extend([id_lo.wrapping_add(128), id_hi]);
    payload.extend(fade_in_speed.to_be_bytes());
    payload.extend([fade_lo.wrapping_add(128), fade_hi]);
    Packet {
        opcode: 35,
        payload,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn death_jingle_matches_revision_239_wire_fields() {
        let p=super::jingle(90);assert_eq!(p.opcode,118);assert_eq!(p.payload,vec![0,0,0,218,0]);
        let p=super::jingle(0x12f0);assert_eq!(p.payload,vec![0,0,0,0x70,0x12]);
    }
}
