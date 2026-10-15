use super::*;

#[derive(Clone)]
pub(super) struct Transmission {
    pub identity: (u32, u64),
    pub container: u16,
    pub updates: crate::inventory_sync::Transmissions,
}

impl<B: AccountBackend> Runtime<B> {
    pub(super) fn shop_value(
        &mut self,
        actor: ActorId,
        item: i32,
        player_buys: bool,
        result: Result<i32, gielinor_world::shop::Error>,
    ) -> io::Result<()> {
        let text = match result {
            Ok(price) => {
                let Some(definition) = self.definitions.items.get(&(item as u32)) else {
                    return Ok(());
                };
                let verb = if player_buys {
                    "currently costs"
                } else {
                    "shop will buy for"
                };
                let suffix = if price == 1 { "coin" } else { "coins" };
                format!("{}: {verb} {price} {suffix}.", definition.name)
            }
            Err(gielinor_world::shop::Error::NotTraded) if !player_buys => {
                "You can't sell this item to this shop.".to_owned()
            }
            _ => return Ok(()),
        };
        if let Some(connection) = self.connection(actor) {
            let packet = gielinor_protocol::chat::game(0, None, &text)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
            self.transport
                .send(connection, packet.opcode, &packet.payload)?;
        }
        Ok(())
    }
    /// Trusted host subscription after opening a validated shop session and UI.
    /// The container definition is explicit; it does not select a stock partition.
    pub fn start_shop_inventory(&mut self, actor: ActorId, container: u16) -> io::Result<()> {
        let identity = self
            .world
            .shop_session_identity(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?
            .ok_or_else(|| io::Error::other("No active shop session"))?;
        let shop = self
            .world
            .shops
            .get(&identity.0)
            .ok_or_else(|| io::Error::other("Missing shop partition"))?;
        let definition = self
            .definitions
            .inventories
            .get(&(container as u32))
            .ok_or_else(|| io::Error::other("Unknown shop inventory definition"))?;
        if definition.size != shop.inventory().slots().len()
            || definition.stack != gielinor_world::inventory::StackType::Always
        {
            return Err(io::Error::other(
                "Shop inventory definition does not match stock",
            ));
        }
        let active = self
            .active
            .get_mut(&actor)
            .ok_or_else(|| io::Error::other("Invalid shop actor"))?;
        let connection = active
            .connection
            .ok_or_else(|| io::Error::other("Disconnected shop actor"))?;
        if [
            self.policy.inventory,
            self.policy.equipment,
            self.policy.bank,
        ]
        .contains(&(container as u32))
            || active.inventories.ids().any(|id| id == container)
        {
            return Err(io::Error::other(
                "Shop inventory collides with personal container",
            ));
        }
        if let Some(mut prior) = active.shop_inventory.clone() {
            let stop = prior.updates.stop(prior.container);
            self.transport
                .send(connection, stop.opcode, &stop.payload)?;
        }
        let mut updates = crate::inventory_sync::Transmissions::default();
        updates.start(container);
        active.shop_inventory = Some(Transmission {
            identity,
            container,
            updates,
        });
        Ok(())
    }
}
