use super::*;
use crate::interface_hooks::Action as Hook;
use gielinor_world::ui::{operation_mask, Component};
impl<B: AccountBackend> Runtime<B> {
    pub fn configure_deposit_box(&mut self, names: &crate::startup::Names) -> io::Result<()> {
        if !self.connections.is_empty()
            || self.bank_presentation.is_none()
            || self.npc_bank_parents.is_none()
            || self.deposit_box_presentation.is_some()
        {
            return Err(io::Error::other(
                "Configure deposit box once with shared banking before admission",
            ));
        }
        let mut components = gielinor_cache::symbols::Symbols::new();
        for name in [
            "bank_depositbox:inventory",
            "bank_depositbox:deposit_inv",
            "bank_depositbox:deposit_worn",
        ] {
            components.insert(
                name.into(),
                i32::try_from(names.id("component", name)?).map_err(io::Error::other)?,
            );
        }
        let p = crate::deposit_box_ui::Presentation::resolve(
            names.group("interface"),
            &components,
            names.group("seq"),
        )
        .map_err(io::Error::other)?;
        if self.held_component.map(|c| Component(c).interface()) != Some(p.inventory)
            || [p.grid, p.deposit_inventory, p.deposit_worn]
                .iter()
                .any(|c| !self.components.contains_key(&c.0))
            || !self
                .components
                .keys()
                .any(|id| *id >> 16 == u32::from(p.disabled_inventory))
        {
            return Err(io::Error::other("Missing deposit UI dependency"));
        }
        let (id, gate) = crate::content::guild_deposit_box(
            names.group("loc"),
            names.group("stat"),
            names.group("varbit"),
            &self.definitions,
        )
        .map_err(io::Error::other)?;
        if self.world.location_banks.contains_key(&id)
            || self.world.location_handlers.contains_key(&(id, 1))
        {
            return Err(io::Error::other("Conflicting Guild deposit box"));
        }
        self.interface_hooks
            .opens
            .entry(p.interface)
            .or_default()
            .extend([
                Hook::SwapOverlay {
                    from: p.inventory,
                    to: p.disabled_inventory,
                },
                Hook::Events {
                    component: p.grid,
                    start: 0,
                    end: 27,
                    mask: 0,
                },
                Hook::Events {
                    component: p.deposit_inventory,
                    start: -1,
                    end: -1,
                    mask: operation_mask(1).unwrap(),
                },
                Hook::Events {
                    component: p.deposit_worn,
                    start: -1,
                    end: -1,
                    mask: operation_mask(1).unwrap(),
                },
            ]);
        self.world.location_banks.insert(id, gate);
        self.world.deposit_box = Some(gielinor_world::deposit_box::Config {
            animation: p.animation,
        });
        self.deposit_box_presentation = Some(p);
        Ok(())
    }
    pub(super) fn open_deposit_box(&mut self, actor: ActorId) -> io::Result<()> {
        let p = self
            .deposit_box_presentation
            .ok_or_else(|| io::Error::other("Deposit presentation missing"))?;
        let (main, _) = self
            .npc_bank_parents
            .ok_or_else(|| io::Error::other("Deposit modal parent missing"))?;
        self.ensure_bank_capacity(actor)?;
        self.open_interface(actor, Some(main), p.interface, false)
    }
    pub(super) fn deposit_box_button(
        &self,
        actor: ActorId,
        button: &gielinor_protocol::buttons::Button,
    ) -> Option<gielinor_world::simulation::Action> {
        let p = self.deposit_box_presentation?;
        let a = self.active.get(&actor)?;
        if !a.ui.modals.contains_interface(p.interface) {
            return None;
        }
        p.bulk(Component(button.component), button.op)
            .map(|worn| gielinor_world::simulation::Action::DepositBoxAll { worn })
    }
}
