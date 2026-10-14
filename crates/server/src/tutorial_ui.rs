//! TutorialFocusGuidance output order. Interface lifecycle effects go through
//! Runtime so existing open/close hooks and gameframe translation remain active.
use crate::startup::Names;
use gielinor_protocol::{
    client_script::{self, Argument},
    framing::Packet,
    hint_arrow::{Arrow, TilePosition},
    interfaces,
};
use gielinor_world::{
    tutorial_focus::{self, Flash},
    tutorial_steps::Step,
    ui::Component,
};
use std::{collections::BTreeMap, io};

pub enum Effect {
    Packet(Packet),
    Overlay {
        parent: Component,
        interface: u16,
    },
    Panel {
        parent: Component,
        interface: u16,
        open: bool,
    },
    VariableBit {
        bit: u32,
        value: i32,
    },
}
pub struct Config {
    pub overlay: u16,
    pub atmosphere: Component,
    pub blocker: u32,
    pub texts: [u32; 4],
    pub mesoverlay: u16,
    pub chatmodal: Component,
    pub panels: Vec<(Component, u16, Step)>,
    pub flashside: u32,
    pub wind_strike: u32,
    pub home_teleport: u32,
    pub npcs: BTreeMap<&'static str, u16>,
    pub locations: BTreeMap<&'static str, u32>,
}
fn script(id: i32, args: &[Argument]) -> io::Result<Effect> {
    client_script::run(id, args)
        .map(Effect::Packet)
        .map_err(|e| io::Error::other(format!("{e:?}")))
}
impl Config {
    pub fn resolve(names: &Names) -> io::Result<Self> {
        let component = |name: &str| names.id("component", name);
        let mut npcs = BTreeMap::new();
        let mut locations = BTreeMap::new();
        for step in Step::ALL {
            for name in [
                tutorial_focus::hint_npc(step),
                tutorial_focus::outline_npc(step),
            ]
            .into_iter()
            .flatten()
            {
                npcs.insert(name, names.short("npc", name)?);
            }
            if let Some(name) = tutorial_focus::outline_location(step) {
                locations.insert(name, names.id("loc", name)?);
            }
        }
        let panels = tutorial_focus::SIDE_PANELS
            .iter()
            .map(|p| {
                Ok((
                    Component(component(&format!("toplevel_osrs_stretch:side{}", p.side))?),
                    names.short("interface", p.interface)?,
                    p.unlock,
                ))
            })
            .collect::<io::Result<_>>()?;
        Ok(Self {
            overlay: names.short("interface", "tutorial_overlay")?,
            atmosphere: Component(component("toplevel_osrs_stretch:overlay_atmosphere")?),
            blocker: component("tutorial_overlay:noclick")?,
            texts: [
                component("tutorial_overlay:hint_text_1")?,
                component("tutorial_overlay:hint_text_2")?,
                component("tutorial_overlay:hint_text_3")?,
                component("tutorial_overlay:hint_text_4")?,
            ],
            mesoverlay: names.short("interface", "mesoverlay")?,
            chatmodal: Component(component("chatbox:chatmodal")?),
            panels,
            flashside: names.id("varbit", "flashside")?,
            wind_strike: component("magic_spellbook:wind_strike")?,
            home_teleport: component("magic_spellbook:teleport_home_standard")?,
            npcs,
            locations,
        })
    }
    pub fn clear(&self) -> io::Result<Vec<Effect>> {
        Ok(vec![
            Effect::Packet(Arrow::Reset.encode()),
            script(5950, &[])?,
            script(5485, &[])?,
        ])
    }
    pub fn guide(&self, step: Step) -> io::Result<Vec<Effect>> {
        let mut effects = Vec::new();
        for (component, text) in self
            .texts
            .into_iter()
            .zip([step.title(), step.body(), "", ""])
        {
            effects.push(Effect::Packet(
                interfaces::text(component, text)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?,
            ));
        }
        effects.push(Effect::Overlay {
            parent: self.chatmodal,
            interface: self.mesoverlay,
        });
        effects.push(script(1974, &[Argument::String(step.guide_html())])?);
        Ok(effects)
    }
    /// `hint_npc` is the matching preferred instance or the reference search result.
    /// A missing NPC clears the prior arrow without manufacturing an index.
    pub fn apply(&self, step: Step, hint_npc: Option<u16>) -> io::Result<Vec<Effect>> {
        let mut effects = self.clear()?;
        effects.push(Effect::Overlay {
            parent: self.atmosphere,
            interface: self.overlay,
        });
        effects.push(Effect::Packet(interfaces::hide(
            self.blocker,
            !tutorial_focus::modal_step(step),
        )));
        effects.extend(
            self.panels
                .iter()
                .map(|&(parent, interface, unlock)| Effect::Panel {
                    parent,
                    interface,
                    open: step >= unlock,
                }),
        );
        effects.extend(self.guide(step)?);
        effects.push(script(867, &[Argument::Int(step.progress_percent())])?);
        if let Some(tile) = tutorial_focus::hint_tile(step) {
            effects.push(Effect::Packet(
                Arrow::Tile {
                    x: tile.x() as u16,
                    z: tile.z() as u16,
                    height: 0,
                    position: TilePosition::Center,
                }
                .encode(),
            ));
        } else if tutorial_focus::hint_npc(step).is_some() {
            if let Some(index) = hint_npc {
                effects.push(Effect::Packet(Arrow::Npc(index).encode()));
            }
        }
        effects.push(Effect::VariableBit {
            bit: self.flashside,
            value: 0,
        });
        match tutorial_focus::flash(step) {
            Flash::None => {}
            Flash::Side(value) => effects.push(Effect::VariableBit {
                bit: self.flashside,
                value: value as i32,
            }),
            Flash::Equipment => effects.push(script(2643, &[])?),
            Flash::WindStrike => {
                effects.push(script(2081, &[Argument::Int(self.wind_strike as i32)])?)
            }
            Flash::HomeTeleport => {
                effects.push(script(2081, &[Argument::Int(self.home_teleport as i32)])?)
            }
        }
        effects.push(script(5487, &[])?);
        if let Some(name) = tutorial_focus::outline_npc(step) {
            effects.push(script(4746, &[Argument::Int(self.npcs[name] as i32)])?);
        }
        if let Some(name) = tutorial_focus::outline_location(step) {
            effects.push(script(4750, &[Argument::Int(self.locations[name] as i32)])?);
        }
        Ok(effects)
    }
    pub fn end(&self) -> io::Result<Vec<Effect>> {
        let mut effects = self.clear()?;
        effects.extend(
            self.panels
                .iter()
                .map(|&(parent, interface, _)| Effect::Panel {
                    parent,
                    interface,
                    open: true,
                }),
        );
        effects.push(script(2645, &[])?);
        Ok(effects)
    }
}
