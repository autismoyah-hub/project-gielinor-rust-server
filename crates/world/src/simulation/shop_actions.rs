use super::*;
impl World {
    /// Trusted content entry after shop/stock-partition eligibility and UI binding.
    /// This does not open a client interface or infer a named shop from an ID.
    pub fn open_shop_session(&mut self, actor: ActorId, shop: u32) -> Result<u64, ActionError> {
        self.open_shop_session_with_buttons(actor, shop, None)
    }
    pub fn open_shop_session_with_buttons(
        &mut self,
        actor: ActorId,
        shop: u32,
        buttons: Option<crate::shop::ButtonBinding>,
    ) -> Result<u64, ActionError> {
        let p = self.player(actor)?;
        if p.saving || p.pending_logout {
            return Err(ActionError::LoggingOut);
        }
        if p.protection.modal_button_protected() {
            return Err(ActionError::Protected);
        }
        if !self.shops.contains_key(&shop) {
            return Err(ActionError::InvalidItem);
        }
        self.close_dialogs(actor)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.dialog_generation = p
            .dialog_generation
            .checked_add(1)
            .expect("dialog generation exhausted");
        p.shop_session = Some(crate::shop::Session {
            shop,
            generation: p.dialog_generation,
            buttons,
        });
        Ok(p.dialog_generation)
    }
    pub fn shop_session_token(&self, actor: ActorId) -> Result<Option<u64>, ActionError> {
        Ok(self.player(actor)?.shop_session.map(|s| s.generation))
    }
    pub fn shop_session_identity(&self, actor: ActorId) -> Result<Option<(u32, u64)>, ActionError> {
        Ok(self
            .player(actor)?
            .shop_session
            .map(|s| (s.shop, s.generation)))
    }
    /// Closing either bound pane invalidates this shop even when another modal
    /// remains open. A generic has-modal flag alone cannot establish ownership.
    pub fn close_shop_interface(
        &mut self,
        actor: ActorId,
        interface: u16,
    ) -> Result<(), ActionError> {
        self.player(actor)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        if p.shop_session
            .is_some_and(|s| s.buttons.is_some_and(|b| b.includes_interface(interface)))
        {
            p.shop_session = None;
        }
        Ok(())
    }
    /// Decode only a component belonging to this session; capture its generation
    /// now so a queued click cannot affect a replacement shop. The runtime must
    /// first validate its open-interface event mask through the existing UI map.
    pub fn shop_button_action(
        &self,
        actor: ActorId,
        component: crate::ui::Component,
        sub: i32,
        operation: u8,
        item: i32,
    ) -> Result<Option<Action>, ActionError> {
        let p = self.player(actor)?;
        if p.saving || p.pending_logout || !p.has_modal || p.protection.modal_button_protected() {
            return Ok(None);
        }
        Ok(p.shop_session.and_then(|session| {
            session
                .buttons?
                .request(component, sub, operation, item)
                .map(|request| Action::Shop {
                    session: session.generation,
                    request,
                })
        }))
    }
    pub(super) fn process_shop_action(
        &mut self,
        actor: ActorId,
        generation: u64,
        request: crate::shop::Request,
    ) -> Result<(), ActionError> {
        let p = self.player(actor)?;
        let Some(session) = p.shop_session.filter(|s| s.generation == generation) else {
            return Ok(());
        };
        if !p.has_modal {
            return Err(ActionError::Protected);
        }
        if p.protection.modal_button_protected() {
            return Err(ActionError::Protected);
        }
        let shop = self
            .shops
            .get_mut(&session.shop)
            .ok_or(ActionError::InvalidItem)?;
        let held = &mut self
            .players
            .get_mut(actor.slot as usize)
            .unwrap()
            .data
            .inventory;
        let result = match request {
            crate::shop::Request::Value {
                slot,
                item,
                player_buys,
            } => {
                let result = if player_buys {
                    shop.quote(slot, item, true)
                } else {
                    shop.quote_sale(held, &self.object_types, slot, item)
                };
                self.events.push(WorldEvent::ShopValue {
                    actor,
                    item,
                    player_buys,
                    result,
                });
                return Ok(());
            }
            crate::shop::Request::Buy {
                slot,
                item,
                quantity,
            } => shop.buy(held, &self.object_types, slot, item, quantity),
            crate::shop::Request::Sell {
                slot,
                item,
                quantity,
            } => shop.sell(held, &self.object_types, slot, item, quantity),
        };
        // Rejections are authoritative outcomes too. Stale client associations
        // and internal/configuration errors remain silent; ordinary gameplay
        // failures use the existing game-message transport.
        let message = match result {
            Err(crate::shop::Error::OutOfStock) => Some("That item is currently out of stock."),
            Err(crate::shop::Error::Currency) => Some("You can't afford that."),
            Err(crate::shop::Error::NotTraded) => Some("You can't sell this item to this shop."),
            Err(crate::shop::Error::Capacity)
            | Err(crate::shop::Error::Inventory(crate::inventory::Error::NotEnoughSpace)) => {
                Some("There is not enough space to complete that exchange.")
            }
            _ => None,
        };
        if let Some(text) = message {
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: text.to_owned(),
            });
        }
        self.events.push(WorldEvent::Shop {
            actor,
            shop: session.shop,
            result,
        });
        Ok(())
    }
}
