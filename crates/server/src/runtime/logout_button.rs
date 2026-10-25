//! LogoutTabScript `onIfOverlayButton(logout:logout)`: Player.ifClose, then the
//! protected manual logout. Only op 1 of the configured component dispatches.
use super::*;

impl<B: AccountBackend> Runtime<B> {
    /// The packed `logout:logout` component (interface 182, component 8 in rev 239).
    pub fn configure_logout_button(&mut self, component: u32) {
        self.logout_button = Some(component);
    }
    pub(super) fn logout_button(&mut self, actor: ActorId, component: u32, op: u8) -> io::Result<()> {
        if self.logout_button != Some(component) || op != 1 {
            return Ok(());
        }
        self.close_dialogs_and_modals(actor)?;
        self.world
            .request_manual_logout(actor)
            .map(|_| ())
            .map_err(|e| io::Error::other(format!("{e:?}")))
    }
}
