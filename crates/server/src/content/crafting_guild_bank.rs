//! Guild chest only; deposit box, Collect and guild entry are separate.
use super::{Definitions, Symbols};
use gielinor_world::bank_access::Gate;
pub fn crafting_guild_bank(
    locations: &Symbols,
    stats: &Symbols,
    bits: &Symbols,
    defs: &Definitions,
) -> Result<(u32, Gate), String> {
    if locations.get("diary_guild_bankchest") != Some(&14886) || stats.get("crafting") != Some(&12)
    {
        return Err("Missing Guild chest or Crafting alias".into());
    }
    let loc = defs
        .locations
        .get(&14886)
        .ok_or("Missing Guild bank chest")?;
    if loc.name != "Bank chest"
        || loc.ops[0].as_deref() != Some("Use")
        || loc.collision.width != 1
        || loc.collision.length != 1
        || !loc.multi.is_empty()
        || loc.multi_default != -1
        || loc.multi_varbit != -1
        || loc.multi_varp != -1
    {
        return Err("Invalid static Guild bank chest".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    if !defs
        .varps
        .get(&1190)
        .is_some_and(|v| v.id == 1190 && v.permanent)
    {
        return Err("Missing persistent Falador reward base".into());
    }
    let mut any_bits = Vec::new();
    for (name, id, shift) in [
        ("falador_hard_reward", 4505, 6),
        ("falador_elite_reward", 4506, 7),
    ] {
        if bits.get(name) != Some(&id) {
            return Err("Missing Falador reward alias".into());
        }
        let bit = *defs
            .varbits
            .get(&(id as u32))
            .ok_or("Missing Falador reward bit")?;
        if bit.base != 1190 || bit.lsb != shift || bit.msb != shift {
            return Err("Invalid Falador reward binding".into());
        }
        any_bits.push(bit);
    }
    Ok((
        14886,
        Gate {
            kind: gielinor_world::bank_access::Kind::Full,
            members: true,
            skill: 12,
            base_level: 99,
            any_bits,
        },
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::variables::Varbit;
    #[test]
    fn crafting_guild_bank_pins_use_and_persistent_reward_bits() {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut loc = gielinor_cache::locations::LocationType::decode(b"\0").unwrap();
        loc.name = "Bank chest".into();
        loc.ops[0] = Some("Use".into());
        d.locations.insert(14886, loc);
        d.stats.insert(
            12,
            gielinor_cache::definitions::StatType::decode(b"\x01Crafting\0\0").unwrap(),
        );
        d.varps.insert(
            1190,
            gielinor_world::variables::Varp {
                id: 1190,
                permanent: true,
                transmit: gielinor_world::variables::Transmit::Always,
                bit_protect: false,
                client_code: -1,
            },
        );
        for (id, shift) in [(4505, 6), (4506, 7)] {
            d.varbits.insert(
                id,
                Varbit {
                    base: 1190,
                    lsb: shift,
                    msb: shift,
                },
            );
        }
        let l = [("diary_guild_bankchest".into(), 14886)].into();
        let n = [("crafting".into(), 12)].into();
        let b = [
            ("falador_hard_reward".into(), 4505),
            ("falador_elite_reward".into(), 4506),
        ]
        .into();
        let (id, g) = crafting_guild_bank(&l, &n, &b, &d).unwrap();
        assert_eq!(id, 14886);
        assert_eq!(g.base_level, 99);
        assert!(g.members);
        let mut deposit = gielinor_cache::locations::LocationType::decode(b"\0").unwrap();
        deposit.name = "Bank Deposit Box".into();
        deposit.ops[0] = Some("Deposit".into());
        d.locations.insert(29103, deposit);
        let mut locations = l.clone();
        locations.insert("diary_guild_deposit_box".into(), 29103);
        let (box_id, box_gate) = crate::content::guild_deposit_box(&locations, &n, &b, &d).unwrap();
        assert_eq!(box_id, 29103);
        assert_eq!(box_gate.kind, gielinor_world::bank_access::Kind::DepositBox);
        assert_eq!(g.kind, gielinor_world::bank_access::Kind::Full);
        assert_eq!(
            box_gate
                .any_bits
                .iter()
                .map(|b| (b.base, b.lsb, b.msb))
                .collect::<Vec<_>>(),
            g.any_bits
                .iter()
                .map(|b| (b.base, b.lsb, b.msb))
                .collect::<Vec<_>>()
        );
        d.locations.get_mut(&29103).unwrap().ops[0] = Some("Use".into());
        assert!(crate::content::guild_deposit_box(&locations, &n, &b, &d).is_err());
        d.varbits.get_mut(&4506).unwrap().lsb = 6;
        assert!(crafting_guild_bank(&l, &n, &b, &d).is_err());
    }
}
