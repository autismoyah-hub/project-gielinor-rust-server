//! Delayed gameframe transitions and their interface-move callbacks.
use gielinor_cache::{game::Definitions, symbols::Symbols};
use gielinor_protocol::{
    client_script::{self, Argument},
    framing::Packet,
    interfaces,
};
use gielinor_world::{
    bank_actions::Binding,
    gameframe::{Frame, Frames, Movement},
    ui::{operation_mask, Component, State},
    variables::{Transmit, Variables, Varp},
};
use std::collections::BTreeMap;
pub struct Config {
    pub button: u32,
    pub choices: [u16; 3],
    pub targets: Vec<Component>,
    pub events: BTreeMap<Component, Component>,
    pub xp_drops: Component,
    pub ehc_listener: Component,
    pub orbs: (Component, u16),
    pub resync: [Varp; 2],
    pub minimap: Binding,
}
impl Config {
    pub fn standard(
        interfaces: &Symbols,
        components: &Symbols,
        varps: &Symbols,
        bits: &Symbols,
        defs: &Definitions,
    ) -> Result<Self, String> {
        let symbol = |symbols: &Symbols, name: &str| {
            symbols
                .get(name)
                .copied()
                .ok_or_else(|| format!("Missing gameframe move symbol {name}"))
        };
        let component = |name: &str| {
            symbol(components, &format!("toplevel_osrs_stretch:{name}"))
                .map(|id| Component(id as u32))
        };
        let id = |name: &str| {
            u16::try_from(symbol(interfaces, name)?)
                .map_err(|_| "Invalid gameframe interface".to_owned())
        };
        let targets = [
            "chat_container",
            "mainmodal",
            "maincrm",
            "overlay_atmosphere",
            "overlay_hud",
            "sidemodal",
            "side0",
            "side1",
            "side2",
            "side3",
            "side4",
            "side5",
            "side6",
            "side7",
            "side8",
            "side9",
            "side10",
            "side11",
            "side12",
            "side13",
            "sidecrm",
            "pvp_icons",
            "pm_container",
            "orbs",
            "xp_drops",
            "zeah",
            "floater",
            "buff_bar",
            "stat_boosts_hud",
            "helper_content",
            "hpbar_hud",
            "popout",
            "ehc_listener",
        ]
        .into_iter()
        .map(component)
        .collect::<Result<_, _>>()?;
        let events = (0..14)
            .map(|i| {
                Ok((
                    component(&format!("side{i}"))?,
                    component(&format!("stone{i}"))?,
                ))
            })
            .collect::<Result<_, String>>()?;
        let bit = *defs
            .varbits
            .get(&(symbol(bits, "minimap_state")? as u32))
            .ok_or("Missing minimap varbit")?;
        let base = *defs
            .varps
            .get(&(bit.base as u32))
            .ok_or("Missing minimap varp")?;
        let resync = ["chat_filter_assist", "settings_tracking"]
            .into_iter()
            .map(|name| {
                defs.varps
                    .get(&(symbol(varps, name)? as u32))
                    .copied()
                    .ok_or("Missing gameframe resync varp".into())
            })
            .collect::<Result<Vec<_>, String>>()?
            .try_into()
            .unwrap();
        Ok(Self {
            button: symbol(
                components,
                "settings_side:display_dynamic_setting_1_buttons",
            )? as u32,
            choices: [
                id("toplevel")?,
                id("toplevel_osrs_stretch")?,
                id("toplevel_pre_eoc")?,
            ],
            targets,
            events,
            xp_drops: component("xp_drops")?,
            ehc_listener: component("ehc_listener")?,
            orbs: (component("orbs")?, id("orbs")?),
            resync,
            minimap: Binding { base, bit },
        })
    }
    fn orbs(
        &self,
        ui: &mut State,
        hooks: &crate::interface_hooks::Hooks,
        packets: &mut Vec<Packet>,
    ) -> Result<(), String> {
        ui.open_sub(self.orbs.0, self.orbs.1, true)
            .map_err(str::to_owned)?;
        packets.push(interfaces::open_sub(
            ui.translate(self.orbs.0).0,
            self.orbs.1,
            true,
        ));
        packets.extend(hooks.opened(self.orbs.1, ui)?);
        Ok(())
    }
    fn move_once(
        &self,
        from: &Frame,
        into: &Frame,
        ui: &mut State,
        hooks: &crate::interface_hooks::Hooks,
        packets: &mut Vec<Packet>,
    ) -> Result<(), String> {
        ui.top_level = Some(into.top);
        ui.gameframe = into.mappings.clone();
        packets.push(interfaces::open_top(into.top));
        for target in &self.targets {
            let source = from
                .mappings
                .get(target)
                .ok_or("Missing source move mapping")?;
            let destination = into
                .mappings
                .get(target)
                .ok_or("Missing destination move mapping")?;
            packets.push(interfaces::move_sub(source.0, destination.0));
            if let Some(component) = self.events.get(target) {
                let mask = operation_mask(1).unwrap();
                ui.events
                    .add(*component, -1, -1, mask)
                    .map_err(str::to_owned)?;
                packets.push(
                    interfaces::events(component.0, -1, -1, mask).map_err(|e| format!("{e:?}"))?,
                );
            }
            if *target == self.xp_drops {
                self.orbs(ui, hooks, packets)?;
            }
            if *target == self.ehc_listener {
                packets.push(
                    client_script::run(2358, &[Argument::Int(0)]).map_err(|e| format!("{e:?}"))?,
                );
                packets.push(client_script::run(5937, &[]).map_err(|e| format!("{e:?}"))?);
            }
        }
        Ok(())
    }
    pub fn execute(
        &self,
        frames: &Frames,
        movement: &Movement,
        ui: &mut State,
        vars: &mut Variables,
        logged_in: bool,
        hooks: &crate::interface_hooks::Hooks,
    ) -> Result<Vec<Packet>, String> {
        let mut next = ui.clone();
        let mut variables = vars.clone();
        let mut packets = Vec::new();
        if movement.into.resizable {
            if let Some(update) = variables
                .set_bit(
                    frames.stones.base,
                    frames.stones.bit,
                    movement.into.stones as i32,
                    logged_in,
                )
                .map_err(|e| format!("{e:?}"))?
            {
                packets.push(gielinor_protocol::varp::update(update.id, update.value));
            }
        }
        for varp in self.resync {
            if varp.transmit != Transmit::Never {
                packets.push(gielinor_protocol::varp::update(
                    varp.id,
                    variables.get(varp.id),
                ));
            }
        }
        if movement.from.top != movement.into.top {
            if let Some(intermediate) = &movement.intermediate {
                self.move_once(&movement.from, intermediate, &mut next, hooks, &mut packets)?;
                self.move_once(intermediate, &movement.into, &mut next, hooks, &mut packets)?;
            } else {
                self.move_once(
                    &movement.from,
                    &movement.into,
                    &mut next,
                    hooks,
                    &mut packets,
                )?;
            }
        }
        self.orbs(&mut next, hooks, &mut packets)?;
        let minimap = self.minimap.get(&variables).map_err(|e| format!("{e:?}"))?;
        packets.push(gielinor_protocol::login_init::minimap(
            if (0..=5).contains(&minimap) {
                minimap as u8
            } else {
                0
            },
        ));
        *ui = next;
        *vars = variables;
        Ok(packets)
    }
}
