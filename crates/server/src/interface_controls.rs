//! Immediate overlay settings; protected gameplay stays on the world action boundary.
use gielinor_cache::{game::Definitions, symbols::Symbols};
use gielinor_world::{
    bank_actions::Binding,
    variables::{Update, Variables, Varp},
};
use std::collections::BTreeMap;
#[derive(Clone, Debug)]
pub enum Control {
    Toggle(Binding),
    ToggleSlot(Vec<Binding>),
    Set(Binding, i32),
    Priority { varp: Varp, last: i32 },
    Resync(Varp),
}
impl Control {
    pub fn apply(
        &self,
        vars: &mut Variables,
        slot: i32,
        logged_in: bool,
    ) -> Result<Option<Update>, String> {
        let set = |vars: &mut Variables, b: Binding, value| {
            vars.set_bit(b.base, b.bit, value, logged_in)
                .map_err(|e| format!("{e:?}"))
        };
        match self {
            Self::Toggle(b) => {
                let value = i32::from(b.get(vars).map_err(|e| format!("{e:?}"))? != 1);
                set(vars, *b, value)
            }
            Self::ToggleSlot(bits) => {
                let bit = usize::try_from(slot)
                    .ok()
                    .and_then(|i| bits.get(i))
                    .ok_or("Invalid filter slot")?;
                Self::Toggle(*bit).apply(vars, slot, logged_in)
            }
            Self::Set(b, value) => set(vars, *b, *value),
            Self::Priority { varp, last } => {
                if !(1..=*last).contains(&slot) {
                    return Err("Invalid attack priority slot".into());
                }
                Ok(vars.set(*varp, slot - 1, logged_in))
            }
            Self::Resync(varp) => Ok(
                (varp.transmit != gielinor_world::variables::Transmit::Never).then_some(Update {
                    id: varp.id,
                    value: vars.get(varp.id),
                }),
            ),
        }
    }
}
pub fn standard(
    components: &Symbols,
    bits: &Symbols,
    varps: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<u32, Control>, String> {
    let symbol = |symbols: &Symbols, name: &str| {
        symbols
            .get(name)
            .copied()
            .ok_or_else(|| format!("Missing control symbol {name}"))
    };
    let binding = |name: &str| -> Result<Binding, String> {
        let bit = *defs
            .varbits
            .get(&(symbol(bits, name)? as u32))
            .ok_or("Missing control varbit")?;
        let base = *defs
            .varps
            .get(&(bit.base as u32))
            .ok_or("Missing control base varp")?;
        Ok(Binding { base, bit })
    };
    let mut controls = BTreeMap::new();
    let mut insert = |name: &str, control| -> Result<(), String> {
        controls.insert(symbol(components, name)? as u32, control);
        Ok(())
    };
    let filters = [
        "prayer_filter_blocklowtier",
        "prayer_filter_allowcombinedtier",
        "prayer_filter_blockhealing",
        "prayer_filter_blocklacklevel",
        "prayer_filter_blocklocked",
    ]
    .into_iter()
    .map(binding)
    .collect::<Result<Vec<_>, _>>()?;
    insert("prayerbook:filtermenu", Control::ToggleSlot(filters))?;
    for (component, bit) in [
        ("settings_side:zoom_toggle", "camera_zoom_mouse_disabled"),
        ("settings_side:skull_prevention", "skull_prevent_enabled"),
        ("settings_side:acceptaid", "option_acceptaid"),
        ("settings_side:zoom28", "music_unlock_text_toggle"),
    ] {
        insert(component, Control::Toggle(binding(bit)?))?;
    }
    let panel = binding("settings_side_panel_tab")?;
    for (value, name) in ["settings_tab", "audio_tab", "display_tab"]
        .into_iter()
        .enumerate()
    {
        insert(
            &format!("settings_side:{name}"),
            Control::Set(panel, value as i32),
        )?;
    }
    insert(
        "settings_side:brightness_bobble_container",
        Control::Resync(binding("option_brightness_remember")?.base),
    )?;
    for (name, varp, last) in [
        ("attack_priority_player_buttons", "option_attackpriority", 5),
        (
            "attack_priority_npc_buttons",
            "option_attackpriority_npc",
            4,
        ),
    ] {
        let varp = *defs
            .varps
            .get(&(symbol(varps, varp)? as u32))
            .ok_or("Missing attack priority varp")?;
        insert(
            &format!("settings_side:{name}"),
            Control::Priority { varp, last },
        )?;
    }
    Ok(controls)
}
