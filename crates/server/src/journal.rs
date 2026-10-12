//! Nested journal overlay and summary initialization from PlayerExtensions.
use gielinor_cache::{game::Definitions, symbols::Symbols};
use gielinor_protocol::{
    client_script::{self, Argument},
    framing::Packet,
    interfaces,
};
use gielinor_world::{
    bank_actions::Binding,
    ui::{Component, State},
    variables::Variables,
};
use std::collections::BTreeMap;
pub const COLLECTION_VARS: [&str; 12] = [
    "collection_count_other_max",
    "collection_count_other",
    "collection_count_minigames_max",
    "collection_count_minigames",
    "collection_count_clues_max",
    "collection_count_clues",
    "collection_count_raids_max",
    "collection_count_raids",
    "collection_count_bosses_max",
    "collection_count_bosses",
    "collection_count_max",
    "collection_count",
];
pub struct Switching {
    pub buttons: BTreeMap<u32, usize>,
    pub collection: [gielinor_world::variables::Varp; 12],
}
pub struct Journal {
    pub interface: u16,
    pub tabs: [u16; 3],
    pub parent: Component,
    pub contents: Component,
    pub click_layer: Component,
    pub selected: Binding,
    pub playtime: u16,
    pub switching: Option<Switching>,
}
impl Journal {
    pub fn selected_by_button(&self, component: u32, slot: i32) -> Option<usize> {
        let config = self.switching.as_ref()?;
        if component == self.click_layer.0 {
            return match slot {
                3 => Some(1),
                4 => Some(2),
                _ => None,
            };
        }
        config.buttons.get(&component).copied()
    }
    pub fn resolve(
        interfaces: &Symbols,
        components: &Symbols,
        bits: &Symbols,
        varps: &Symbols,
        defs: &Definitions,
    ) -> Result<Self, String> {
        let symbol = |symbols: &Symbols, name: &str| {
            symbols
                .get(name)
                .copied()
                .ok_or_else(|| format!("Missing journal symbol {name}"))
        };
        let interface = |name: &str| {
            u16::try_from(symbol(interfaces, name)?)
                .map_err(|_| "Invalid journal interface".to_owned())
        };
        let bit = *defs
            .varbits
            .get(&(symbol(bits, "side_journal_tab")? as u32))
            .ok_or("Missing journal varbit")?;
        let base = *defs
            .varps
            .get(&(bit.base as u32))
            .ok_or("Missing journal base varp")?;
        Ok(Self {
            interface: interface("side_journal")?,
            tabs: [
                interface("account_summary_sidepanel")?,
                interface("questlist")?,
                interface("area_task")?,
            ],
            parent: Component(symbol(components, "side_journal:tab_container")? as u32),
            contents: Component(
                symbol(components, "account_summary_sidepanel:summary_contents")? as u32,
            ),
            click_layer: Component(symbol(
                components,
                "account_summary_sidepanel:summary_click_layer",
            )? as u32),
            selected: Binding { base, bit },
            playtime: u16::try_from(symbol(varps, "playtime")?)
                .map_err(|_| "Invalid playtime varp")?,
            switching: Some(Switching {
                buttons: [
                    "side_journal:summary_list",
                    "side_journal:quest_list",
                    "side_journal:task_list",
                ]
                .into_iter()
                .enumerate()
                .map(|(i, name)| Ok((symbol(components, name)? as u32, i)))
                .collect::<Result<_, String>>()?,
                collection: COLLECTION_VARS
                    .into_iter()
                    .map(|name| {
                        defs.varps
                            .get(&(symbol(varps, name)? as u32))
                            .copied()
                            .ok_or("Missing collection varp".into())
                    })
                    .collect::<Result<Vec<_>, String>>()?
                    .try_into()
                    .unwrap(),
            }),
        })
    }
    pub fn opened(
        &self,
        interface: u16,
        ui: &mut State,
        vars: &Variables,
        combat: i32,
        hooks: &crate::interface_hooks::Hooks,
    ) -> Result<Vec<Packet>, String> {
        if interface != self.interface {
            return Ok(Vec::new());
        }
        let value = self.selected.get(vars).map_err(|e| format!("{e:?}"))?;
        // Kotlin enum delegate uses Summary for unknown persisted values.
        let index = if (0..3).contains(&value) {
            value as usize
        } else {
            0
        };
        let tab = self.tabs[index];
        let mut next = ui.clone();
        let mut packets = Vec::new();
        if index == 0 {
            for (id, value) in [(3970, vars.get(self.playtime) / 100), (3954, combat)] {
                packets.push(
                    client_script::run(
                        id,
                        &[self.contents.0 as i32, self.click_layer.0 as i32, value]
                            .map(Argument::Int),
                    )
                    .map_err(|e| format!("{e:?}"))?,
                );
            }
        }
        next.open_sub(self.parent, tab, true)
            .map_err(str::to_owned)?;
        packets.push(interfaces::open_sub(
            next.translate(self.parent).0,
            tab,
            true,
        ));
        packets.extend(hooks.opened(tab, &mut next)?);
        *ui = next;
        Ok(packets)
    }
    pub fn switch(
        &self,
        tab: usize,
        ui: &mut State,
        vars: &mut Variables,
        combat: i32,
        member: bool,
        logged_in: bool,
        hooks: &crate::interface_hooks::Hooks,
    ) -> Result<Vec<Packet>, String> {
        if tab >= self.tabs.len() {
            return Err("Invalid journal tab".into());
        }
        let config = self
            .switching
            .as_ref()
            .ok_or("Journal switching is not configured")?;
        let mut next = ui.clone();
        let mut variables = vars.clone();
        let mut packets = Vec::new();
        let previous = self
            .selected
            .get(&variables)
            .map_err(|e| format!("{e:?}"))?;
        let previous = if (0..3).contains(&previous) {
            previous as usize
        } else {
            0
        };
        if let Some(update) = variables
            .set_bit(self.selected.base, self.selected.bit, tab as i32, logged_in)
            .map_err(|e| format!("{e:?}"))?
        {
            packets.push(gielinor_protocol::varp::update(update.id, update.value));
        }
        if let Some(parent) = next.overlays.parent(self.tabs[previous]) {
            for (parent, _) in next.close_sub(parent) {
                packets.push(interfaces::close_sub(next.translate(parent).0));
            }
        }
        if tab == 0 {
            for varp in config.collection {
                if varp.transmit != gielinor_world::variables::Transmit::Never {
                    packets.push(gielinor_protocol::varp::update(
                        varp.id,
                        variables.get(varp.id),
                    ));
                }
            }
        } else if tab == 1 {
            packets.push(
                client_script::run(828, &[Argument::Int(member as i32)])
                    .map_err(|e| format!("{e:?}"))?,
            );
        }
        packets.extend(self.opened(self.interface, &mut next, &variables, combat, hooks)?);
        *ui = next;
        *vars = variables;
        Ok(packets)
    }
}
