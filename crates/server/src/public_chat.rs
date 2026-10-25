//! MESSAGE_PUBLIC admission (rsmod MessagePublicHandler): a valid public-chat
//! message becomes this cycle's player-info chat block. Player sync delivers it
//! to the sender (the chatbox echo) and to observers who have the sender in view.
//! Other chat types address friends/clan channels; they are not broadcast.
use gielinor_protocol::{
    huffman::Huffman,
    public_chat::{Message, PUBLIC},
};
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Encoded chat block for the sender's next player-info update.
    Public(Vec<u8>),
    /// Well-formed message for a channel this server does not implement.
    UnsupportedChannel(u8),
}
/// `icon` is the sender's mod-level client code, as sent in the login response.
pub fn accept(payload: &[u8], huffman: &Huffman, icon: u8) -> Result<Outcome, String> {
    let message = Message::decode(payload, huffman).map_err(|e| format!("MESSAGE_PUBLIC: {e:?}"))?;
    if message.kind != PUBLIC {
        return Ok(Outcome::UnsupportedChannel(message.kind));
    }
    message
        .block(icon, huffman)
        .map(Outcome::Public)
        .map_err(|e| format!("MESSAGE_PUBLIC block: {e:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn public_messages_become_blocks_other_channels_and_malformed_do_not() {
        let h = Huffman::new(&[8; 256]).unwrap();
        let Outcome::Public(block) = accept(&[0, 0, 0, 2, b'h', b'i'], &h, 2).unwrap() else {
            panic!("public chat accepted");
        };
        assert_eq!(block, [0, 128, 254, 128, 253, b'i', b'h', 2]);
        assert_eq!(
            accept(&[2, 0, 0, 2, b'h', b'i'], &h, 0),
            Ok(Outcome::UnsupportedChannel(2))
        );
        assert_eq!(
            accept(&[3, 0, 0, 2, b'h', b'i', 0], &h, 0),
            Ok(Outcome::UnsupportedChannel(3))
        );
        assert!(accept(&[0, 0, 0, 3, b'h', b'i'], &h, 0).is_err());
        assert!(accept(&[], &h, 0).is_err());
    }
}
