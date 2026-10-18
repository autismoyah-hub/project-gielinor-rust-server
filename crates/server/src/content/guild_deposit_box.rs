//! Guild deposit box shares the exact chest access rule, with a distinct UI kind.
use super::{Definitions, Symbols};
pub fn guild_deposit_box(
    locations: &Symbols,
    stats: &Symbols,
    bits: &Symbols,
    defs: &Definitions,
) -> Result<(u32, gielinor_world::bank_access::Gate), String> {
    let (_, mut gate) = super::crafting_guild_bank(locations, stats, bits, defs)?;
    if locations.get("diary_guild_deposit_box") != Some(&29103) {
        return Err("Missing Guild deposit box alias".into());
    }
    let d = defs
        .locations
        .get(&29103)
        .ok_or("Missing Guild deposit box")?;
    if d.name != "Bank Deposit Box"
        || d.ops[0].as_deref() != Some("Deposit")
        || d.collision.width != 1
        || d.collision.length != 1
        || !d.multi.is_empty()
        || d.multi_default != -1
        || d.multi_varbit != -1
        || d.multi_varp != -1
    {
        return Err("Invalid static Guild deposit box".into());
    }
    gate.kind = gielinor_world::bank_access::Kind::DepositBox;
    Ok((29103, gate))
}
