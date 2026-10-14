use super::*;
use crate::{
    ground::{Object, Scope},
    ranged_ammo::{self, Shot, Use},
};
impl World {
    /// Standard one-ammunition shot; called by authoritative combat after launching.
    /// The delayed drop captures ownership and collision at launch, surviving logout.
    pub fn use_ranged_ammo(
        &mut self,
        actor: ActorId,
        shot: Shot,
        random: &mut impl RandomSource,
    ) -> Result<ranged_ammo::Consumption, ActionError> {
        let player = self.player(actor)?;
        if player.saving {
            return Err(ActionError::LoggingOut);
        }
        if ![3, 13].contains(&shot.slot)
            || shot.drop_coord == CoordGrid::NULL
            || !(0..=65535).contains(&shot.id)
        {
            return Err(ActionError::InvalidItem);
        }
        let observer = player.observer_id;
        // Original CollisionFlag.BLOCK_WALK=0x200000; wall/loc flags do not count.
        let blocked = self.collision.get(
            shot.drop_coord.x(),
            shot.drop_coord.z(),
            shot.drop_coord.level(),
        ) & 0x200000
            != 0;
        let result = ranged_ammo::consume(
            &mut self
                .players
                .get_mut(actor.slot as usize)
                .unwrap()
                .data
                .equipment,
            Use {
                slot: shot.slot,
                id: shot.id,
                count: 1,
                recovery_rate: shot.recovery_rate,
                metallic_body: shot.metallic_body,
                drop_chance: 5,
                drop_blocked: blocked,
            },
            random,
        )
        .map_err(|_| ActionError::InvalidItem)?;
        if result.detracted.is_some_and(|d| d.depleted) {
            self.worn_changed(actor, shot.slot, shot.id);
        }
        self.events.push(WorldEvent::AmmoConsumed { actor, result });
        if result.drop {
            let observer = observer.ok_or(ActionError::Protected)?;
            self.objects.add_delayed(
                Object {
                    coords: shot.drop_coord,
                    item: shot.id as u16,
                    count: 1,
                    scope: Scope::Private,
                    created: self.clock.0,
                    receiver: Some(observer),
                    owner: Some(observer),
                },
                self.clock,
                shot.drop_delay,
                200,
            );
        }
        Ok(result)
    }
}
