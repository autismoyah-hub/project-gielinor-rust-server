//! RspCycle's appearance selection, separated from wire encoding and observer state.
use gielinor_cache::{
    definitions::{ItemType, Param},
    game::Definitions,
};
use gielinor_protocol::appearance::Appearance;
use gielinor_world::inventory::Inventory;

#[derive(Clone, Debug)]
pub struct Design {
    pub body_type: u8,
    pub pronoun: u8,
    pub kits: [Option<u16>; 7],
    pub colours: [u8; 5],
    pub skull: i8,
    pub overhead: i8,
    pub hidden: bool,
    pub name_extras: [String; 3],
    pub transmog: Option<u16>,
    pub animation_override: Option<[u16; 7]>,
}
impl Default for Design {
    fn default() -> Self {
        Self {
            body_type: 0,
            pronoun: 0,
            kits: [0, 10, 18, 26, 33, 36, 42].map(Some),
            colours: [0; 5],
            skull: -1,
            overhead: -1,
            hidden: false,
            name_extras: std::array::from_fn(|_| String::new()),
            transmog: None,
            animation_override: None,
        }
    }
}
/// IDs come from the server type registry; they cannot be inferred from an item's name.
pub struct AnimationPolicy {
    pub human: [u16; 7],
    pub weapon_params: Option<[u32; 7]>,
}
impl AnimationPolicy {
    pub fn resolve(
        sequences: &gielinor_cache::symbols::Symbols,
        params: &gielinor_cache::symbols::Symbols,
    ) -> Result<Self, &'static str> {
        let mut human = [0; 7];
        let mut weapon_params = [0; 7];
        for (i, name) in [
            "human_ready",
            "human_turnonspot",
            "human_walk_f",
            "human_walk_b",
            "human_walk_l",
            "human_walk_r",
            "human_running",
        ]
        .into_iter()
        .enumerate()
        {
            human[i] = u16::try_from(
                *sequences
                    .get(name)
                    .ok_or("Missing human animation symbol")?,
            )
            .map_err(|_| "Invalid human animation symbol")?;
        }
        for (i, name) in [
            "bas_readyanim",
            "bas_turnonspot",
            "bas_walk_f",
            "bas_walk_b",
            "bas_walk_l",
            "bas_walk_r",
            "bas_running",
        ]
        .into_iter()
        .enumerate()
        {
            weapon_params[i] =
                u32::try_from(*params.get(name).ok_or("Missing weapon animation symbol")?)
                    .map_err(|_| "Invalid weapon animation symbol")?;
        }
        Ok(Self {
            human,
            weapon_params: Some(weapon_params),
        })
    }
    fn weapon(&self, item: &ItemType) -> Result<[u16; 7], &'static str> {
        let ids = self
            .weapon_params
            .ok_or("Weapon animation parameter IDs are unresolved")?;
        let mut animations = self.human;
        for (slot, id) in ids.into_iter().enumerate() {
            if let Some(param) = item.params.get(&id) {
                let Param::Integer(value) = param else {
                    return Err("Non-integer weapon animation");
                };
                animations[slot] = if *value == -1 {
                    u16::MAX
                } else {
                    u16::try_from(*value).map_err(|_| "Animation out of range")?
                };
            }
        }
        Ok(animations)
    }
}
pub fn build(
    design: &Design,
    name: &str,
    combat: u8,
    worn: &Inventory,
    defs: &Definitions,
    policy: &AnimationPolicy,
) -> Result<Appearance, &'static str> {
    let mut base_equipment = [0u16; 12];
    for (kit, slot) in design.kits.iter().zip([8, 11, 4, 6, 9, 7, 10]) {
        if let Some(kit) = kit {
            if *kit > 1791 {
                return Err("Identikit exceeds RSProt range");
            }
            base_equipment[slot] = *kit + 256;
        }
    }
    let mut equipment = base_equipment;
    let mut hidden = 0u16;
    for slot in [0, 1, 2, 3, 4, 5, 7, 9, 10] {
        if let Some(item) = worn.get(slot) {
            let id = u32::try_from(item.id).map_err(|_| "Invalid worn item")?;
            let ty = defs.items.get(&id).ok_or("Unknown worn item")?;
            equipment[slot] = u16::try_from(id.checked_add(2048).ok_or("Worn item overflow")?)
                .map_err(|_| "Worn item exceeds protocol range")?;
            if equipment[slot] == u16::MAX {
                return Err("Worn item collides with NPC sentinel");
            }
            for secondary in ty.wear[1..].iter().copied() {
                if secondary == -1 {
                    continue;
                }
                if !(0..14).contains(&secondary) {
                    return Err("Invalid secondary wear slot");
                }
                hidden |= 1u16 << secondary;
            }
        }
    }
    for (slot, value) in equipment.iter_mut().enumerate() {
        if hidden & (1 << slot) != 0 {
            *value = 0;
        }
    }
    let npc = design
        .transmog
        .map(|id| {
            defs.npcs
                .get(&(id as u32))
                .ok_or("Unknown transformation NPC")
        })
        .transpose()?;
    let animations = if let Some(anims) = design.animation_override {
        anims
    } else if let Some(npc) = npc {
        npc.animations
    } else if let Some(weapon) = worn.get(3) {
        policy.weapon(
            defs.items
                .get(&(weapon.id as u32))
                .ok_or("Unknown weapon")?,
        )?
    } else {
        policy.human
    };
    Ok(Appearance {
        body_type: design.body_type,
        skull: design.skull,
        overhead: design.overhead,
        equipment,
        base_equipment,
        npc: design.transmog,
        colours: design.colours,
        animations,
        name: name.into(),
        combat,
        skill: 0,
        hidden: design.hidden,
        force_refresh: false,
        name_extras: design.name_extras.clone(),
        pronoun: design.pronoun,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::inventory::{Item, StackType};
    use std::collections::BTreeMap;
    fn definitions() -> Definitions {
        Definitions {
            items: BTreeMap::new(),
            npcs: BTreeMap::new(),
            inventories: BTreeMap::new(),
            locations: BTreeMap::new(),
            varps: BTreeMap::new(),
            varbits: BTreeMap::new(),
            stats: BTreeMap::new(),
        }
    }
    #[test]
    fn hidden_slots_do_not_destroy_base_design_and_removal_restores_them() {
        let mut defs = definitions();
        let mut helmet = ItemType::decode(&[0]).unwrap();
        helmet.wear = [0, 8, 11];
        defs.items.insert(100, helmet);
        let mut worn = Inventory::new(14, StackType::Never, false);
        let policy = AnimationPolicy {
            human: [1; 7],
            weapon_params: None,
        };
        worn.restore_slot(
            0,
            Some(Item {
                id: 100,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        let dressed = build(&Design::default(), "Player", 3, &worn, &defs, &policy).unwrap();
        assert_eq!(dressed.equipment[0], 2148);
        assert_eq!(dressed.equipment[8], 0);
        assert_eq!(dressed.equipment[11], 0);
        assert_eq!(dressed.base_equipment[8], 256);
        assert_eq!(dressed.base_equipment[11], 266);
        worn.restore_slot(0, None).unwrap();
        let bare = build(&Design::default(), "Player", 3, &worn, &defs, &policy).unwrap();
        assert_eq!(bare.equipment, bare.base_equipment);
    }
    #[test]
    fn animation_override_then_transmog_then_weapon_then_human() {
        let mut defs = definitions();
        let mut weapon = ItemType::decode(&[0]).unwrap();
        weapon.params.insert(50, Param::Integer(20));
        defs.items.insert(1, weapon);
        let npc = gielinor_cache::npcs::NpcType::decode(&[
            13, 0, 11, 17, 0, 12, 0, 13, 0, 14, 0, 15, 115, 0, 16, 0, 17, 0, 18, 0, 19, 0,
        ])
        .unwrap();
        assert_eq!(npc.animations, [11, 13, 12, 12, 14, 15, 16]);
        defs.npcs.insert(2, npc);
        let policy = AnimationPolicy {
            human: [1, 2, 3, 4, 5, 6, 7],
            weapon_params: Some([50, 51, 52, 53, 54, 55, 56]),
        };
        let mut worn = Inventory::new(14, StackType::Never, false);
        let mut design = Design::default();
        assert_eq!(
            build(&design, "P", 3, &worn, &defs, &policy)
                .unwrap()
                .animations,
            policy.human
        );
        worn.restore_slot(
            3,
            Some(Item {
                id: 1,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        assert_eq!(
            build(&design, "P", 3, &worn, &defs, &policy)
                .unwrap()
                .animations,
            [20, 2, 3, 4, 5, 6, 7]
        );
        design.transmog = Some(2);
        assert_eq!(
            build(&design, "P", 3, &worn, &defs, &policy)
                .unwrap()
                .animations,
            [11, 13, 12, 12, 14, 15, 16]
        );
        design.animation_override = Some([30; 7]);
        assert_eq!(
            build(&design, "P", 3, &worn, &defs, &policy)
                .unwrap()
                .animations,
            [30; 7]
        );
    }
}
