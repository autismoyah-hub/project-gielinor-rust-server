//! PlayerInvUpdateProcessor transmission ordering. Cleanup is separate so shared
//! inventories remain dirty until every observer has encoded its update.
use gielinor_protocol::{
    buffer::DecodeError,
    framing::Packet,
    game::{self, InventoryItem},
};
use gielinor_world::inventory::{Inventory, Item};

#[derive(Clone, Default)]
pub struct Transmissions {
    active: Vec<u16>,
    pending: Vec<u16>,
}
pub struct Batch {
    pub packets: Vec<Packet>,
    pub processed: Vec<u16>,
}
fn item(value: Option<Item>) -> Result<Option<InventoryItem>, DecodeError> {
    value
        .map(|item| {
            Ok(InventoryItem {
                id: u16::try_from(item.id)
                    .map_err(|_| DecodeError::InvalidValue("inventory item id"))?,
                count: u32::try_from(item.count)
                    .map_err(|_| DecodeError::InvalidValue("inventory item count"))?,
            })
        })
        .transpose()
}
impl Transmissions {
    pub fn ids(&self) -> impl Iterator<Item = u16> + '_ {
        self.active.iter().chain(&self.pending).copied()
    }
    /// InvWeight adds a type's grams once per occupied slot, irrespective of stack
    /// count. Pending subscriptions become active before the run-update phase.
    pub fn weight<'a>(
        &self,
        mut lookup: impl FnMut(u16) -> Option<(&'a Inventory, bool)>,
        mut weight: impl FnMut(i32) -> Option<i32>,
    ) -> Result<i32, DecodeError> {
        let mut grams = 0i32;
        for &id in self.active.iter().chain(&self.pending) {
            let (inv, affects_weight) =
                lookup(id).ok_or(DecodeError::InvalidValue("weight inventory missing"))?;
            if !affects_weight {
                continue;
            }
            for item in inv.slots().iter().flatten() {
                grams = grams.wrapping_add(
                    weight(item.id).ok_or(DecodeError::InvalidValue("weight item missing"))?,
                );
            }
        }
        Ok(grams)
    }
    pub fn start(&mut self, id: u16) {
        self.active.retain(|other| *other != id);
        if !self.pending.contains(&id) {
            self.pending.push(id);
        }
    }
    pub fn stop(&mut self, id: u16) -> Packet {
        self.active.retain(|other| *other != id);
        self.pending.retain(|other| *other != id);
        Packet {
            opcode: 45,
            payload: game::inventory_stop(id).to_vec(),
        }
    }
    /// Advance only after successful encoding. Call on a clone when transport
    /// acceptance must be included in the caller's commit boundary.
    pub fn cycle<'a>(
        &mut self,
        mut lookup: impl FnMut(u16) -> Option<&'a Inventory>,
    ) -> Result<Batch, DecodeError> {
        let mut batch = Batch {
            packets: vec![],
            processed: vec![],
        };
        for &id in &self.active {
            let inv =
                lookup(id).ok_or(DecodeError::InvalidValue("transmitted inventory missing"))?;
            if inv.modified().is_empty() {
                continue;
            }
            let changed = inv
                .modified()
                .iter()
                .map(|&slot| {
                    Ok((
                        u16::try_from(slot)
                            .map_err(|_| DecodeError::InvalidValue("inventory slot"))?,
                        item(inv.get(slot))?,
                    ))
                })
                .collect::<Result<Vec<_>, DecodeError>>()?;
            batch.packets.push(Packet {
                opcode: 18,
                payload: game::inventory_partial(-(1234 + id as i32), id, &changed)?,
            });
            batch.processed.push(id);
        }
        for &id in &self.pending {
            let inv = lookup(id).ok_or(DecodeError::InvalidValue("queued inventory missing"))?;
            let items = inv.slots()[..inv.last_occupied_slot()]
                .iter()
                .copied()
                .map(item)
                .collect::<Result<Vec<_>, _>>()?;
            batch.packets.push(Packet {
                opcode: 22,
                payload: game::inventory_full(-(1234 + id as i32), id, &items)?,
            });
            batch.processed.push(id);
        }
        self.active.append(&mut self.pending);
        Ok(batch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::inventory::StackType;
    #[test]
    fn subscriptions_preserve_full_partial_reorder_and_shared_cleanup() {
        let mut inventory = Inventory::new(28, StackType::Normal, false);
        inventory
            .restore_slot(
                3,
                Some(Item {
                    id: 7,
                    count: 300,
                    vars: 0,
                }),
            )
            .unwrap();
        let mut tx = Transmissions::default();
        tx.start(93);
        tx.start(94);
        tx.start(93);
        let full = tx.cycle(|_| Some(&inventory)).unwrap();
        assert_eq!(full.processed, [93, 94]);
        assert!(full.packets.iter().all(|p| p.opcode == 22));
        assert_eq!(&full.packets[0].payload[6..8], &[0, 4]);
        // Encoding does not clear dirtiness needed by another player.
        assert_eq!(inventory.modified().len(), 1);
        inventory.clear_modified();
        assert!(tx.cycle(|_| Some(&inventory)).unwrap().packets.is_empty());
        inventory.restore_slot(3, None).unwrap();
        tx.start(93);
        let changed = tx.cycle(|_| Some(&inventory)).unwrap();
        assert_eq!(changed.processed, [94, 93]);
        assert_eq!(
            changed.packets.iter().map(|p| p.opcode).collect::<Vec<_>>(),
            [18, 22]
        );
        assert_eq!(&changed.packets[0].payload[6..], &[3, 0, 0]);
        assert_eq!(&changed.packets[1].payload[6..], &[0, 0]);
        assert_eq!(tx.stop(94).payload, [0, 222]);
        inventory.clear_modified();
        assert!(tx
            .cycle(|id| if id == 93 { Some(&inventory) } else { None })
            .unwrap()
            .packets
            .is_empty());
    }
    #[test]
    fn missing_inventory_does_not_consume_pending_subscription() {
        let mut tx = Transmissions::default();
        tx.start(95);
        assert!(tx.cycle(|_| None).is_err());
        let inv = Inventory::new(10, StackType::Normal, false);
        assert_eq!(tx.cycle(|_| Some(&inv)).unwrap().packets[0].opcode, 22);
    }
    #[test]
    fn weight_uses_subscriptions_and_slots_not_stack_counts() {
        let mut inv = Inventory::new(3, StackType::Normal, false);
        inv.restore_slot(
            0,
            Some(Item {
                id: 1,
                count: 2000000,
                vars: 0,
            }),
        )
        .unwrap();
        inv.restore_slot(
            1,
            Some(Item {
                id: 2,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        let mut tx = Transmissions::default();
        tx.start(93);
        tx.start(95);
        assert_eq!(
            tx.weight(
                |id| Some((&inv, id == 93)),
                |id| Some(if id == 1 { 1200 } else { -500 })
            )
            .unwrap(),
            700
        );
        tx.stop(93);
        assert_eq!(tx.weight(|_| Some((&inv, false)), |_| None).unwrap(), 0);
    }
}
