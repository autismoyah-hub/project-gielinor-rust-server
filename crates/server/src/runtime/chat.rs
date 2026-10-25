use super::*;
impl<B: AccountBackend> Runtime<B> {
    /// MESSAGE_PUBLIC: a valid public message replaces this cycle's chat block
    /// (latest wins, as rsmod's single `publicMessage`). Malformed payloads and
    /// unimplemented channels are rejected without broadcasting anything.
    pub(super) fn public_chat(&mut self, connection: u64, actor: ActorId, opcode: u16, payload: &[u8]) {
        let reject = |reason: String| Event::PacketRejected {
            connection,
            actor,
            opcode,
            reason,
        };
        let Some(huffman) = self.huffman.as_ref() else {
            self.events.push(reject("Public chat needs the cache Huffman table".into()));
            return;
        };
        let Some(active) = self.active.get_mut(&actor) else {
            return;
        };
        let icon = active.character.identity.staff_level;
        match crate::public_chat::accept(payload, huffman, icon) {
            Ok(crate::public_chat::Outcome::Public(block)) => active.public_chat = Some(block),
            Ok(crate::public_chat::Outcome::UnsupportedChannel(kind)) => {
                self.events.push(reject(format!("Chat channel type {kind} is not implemented")))
            }
            Err(reason) => self.events.push(reject(reason)),
        }
    }
}
