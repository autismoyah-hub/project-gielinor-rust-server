//! Revision239 HintArrowEncoder. Unused bytes are initialized to zero; the
//! reference merely advances the buffer cursor over those padding bytes.
use crate::framing::Packet;
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum TilePosition {
    Center = 2,
    West = 3,
    East = 4,
    South = 5,
    North = 6,
}
#[derive(Clone, Copy, Debug)]
pub enum Arrow {
    Reset,
    Npc(u16),
    Player(u16),
    Tile {
        x: u16,
        z: u16,
        height: u8,
        position: TilePosition,
    },
    WorldEntity {
        index: u16,
        height: u32,
    },
}
impl Arrow {
    pub fn encode(self) -> Packet {
        let mut payload = vec![0; 6];
        match self {
            Self::Reset => {}
            Self::Npc(index) | Self::Player(index) => {
                payload[0] = if matches!(self, Self::Npc(_)) { 1 } else { 10 };
                payload[1..3].copy_from_slice(&index.to_be_bytes());
            }
            Self::Tile {
                x,
                z,
                height,
                position,
            } => {
                payload[0] = position as u8;
                payload[1..3].copy_from_slice(&x.to_be_bytes());
                payload[3..5].copy_from_slice(&z.to_be_bytes());
                payload[5] = height;
            }
            Self::WorldEntity { index, height } => {
                payload[0] = 11;
                payload[1..3].copy_from_slice(&index.to_be_bytes());
                payload[3..6].copy_from_slice(&height.to_be_bytes()[1..]);
            }
        }
        Packet {
            opcode: 50,
            payload,
        }
    }
}
