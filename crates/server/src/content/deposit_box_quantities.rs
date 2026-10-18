//! Exact target quantity bindings; donor names alone do not establish these ids.
use super::{Definitions, Symbols};
use gielinor_world::{bank_actions::Binding, deposit_box_items::Quantities};
pub fn deposit_box_quantities(
    bits: &Symbols,
    varps: &Symbols,
    defs: &Definitions,
) -> Result<Quantities, String> {
    if bits.get("depositbox_mode") != Some(&4430)
        || varps.get("depositbox_requestedquantity") != Some(&1794)
    {
        return Err("Missing deposit quantity aliases".into());
    }
    let mode = *defs.varbits.get(&4430).ok_or("Missing deposit mode")?;
    if (mode.base, mode.lsb, mode.msb) != (867, 28, 31) {
        return Err("Invalid deposit mode binding".into());
    }
    let base = *defs.varps.get(&867).ok_or("Missing deposit mode base")?;
    let requested = *defs.varps.get(&1794).ok_or("Missing deposit X varp")?;
    if base.id != 867 || requested.id != 1794 || !base.permanent || !requested.permanent {
        return Err("Invalid deposit quantity persistence".into());
    }
    Ok(Quantities {
        mode: Binding { base, bit: mode },
        requested,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::variables::{Transmit, Varbit, Varp};
    #[test]
    fn deposit_box_quantities_pins_target_high_bits_and_persistent_requested_value() {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        d.varbits.insert(
            4430,
            Varbit {
                base: 867,
                lsb: 28,
                msb: 31,
            },
        );
        for id in [867, 1794] {
            d.varps.insert(
                id,
                Varp {
                    id: id as u16,
                    permanent: true,
                    transmit: Transmit::Always,
                    bit_protect: false,
                    client_code: -1,
                },
            );
        }
        let bits = [("depositbox_mode".into(), 4430)].into();
        let varps = [("depositbox_requestedquantity".into(), 1794)].into();
        let q = deposit_box_quantities(&bits, &varps, &d).unwrap();
        assert_eq!(q.mode.bit.base, 867);
        assert_eq!(q.requested.id, 1794);
        d.varbits.get_mut(&4430).unwrap().lsb = 27;
        assert!(deposit_box_quantities(&bits, &varps, &d).is_err());
        d.varbits.get_mut(&4430).unwrap().lsb = 28;
        d.varps.get_mut(&1794).unwrap().permanent = false;
        assert!(deposit_box_quantities(&bits, &varps, &d).is_err());
    }
}
