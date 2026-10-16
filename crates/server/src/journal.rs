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

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        simulation::PlayerData,
        stats::Stats,
        tower_diary::Progress,
        variables::{Transmit, Varbit, Varp},
        CoordGrid,
    };
    #[test]
    fn task_tab_requires_known_marks_and_reopens_with_current_levels() {
        let journal = Journal {
            diary: Some(Diary {
                header: (259 << 16) | 1,
                hidden: [2, 3, 4, 5].map(|id| (259 << 16) | id),
                skills: [12, 13, 11],
            }),
            interface: 629,
            tabs: [712, 399, 259],
            parent: Component::new(629, 33),
            contents: Component::new(712, 1),
            click_layer: Component::new(712, 2),
            selected: Binding {
                base: Varp {
                    id: 1,
                    permanent: true,
                    transmit: Transmit::Always,
                    bit_protect: false,
                    client_code: -1,
                },
                bit: Varbit {
                    base: 1,
                    lsb: 0,
                    msb: 1,
                },
            },
            playtime: 2,
            switching: None,
        };
        let mut data = PlayerData::new(
            1,
            "Diary".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        );
        data.variables = Variables::restore(BTreeMap::from([(1, 2)]));
        let mut ui = State::default();
        let unknown = journal
            .opened(
                629,
                &mut ui,
                &data.variables,
                3,
                journal.marks(&data),
                &crate::interface_hooks::Hooks::default(),
            )
            .unwrap();
        assert_eq!(unknown.iter().map(|p| p.opcode).collect::<Vec<_>>(), [7]);
        assert!(journal.login_packets(&data).unwrap().is_empty());
        data.tower_diary = Some(Progress {
            done: [
                true, true, false, false, false, false, false, false, false, false, false,
            ],
            schema11: true,
            ..Default::default()
        });
        let known = journal
            .opened(
                629,
                &mut ui,
                &data.variables,
                3,
                journal.marks(&data),
                &crate::interface_hooks::Hooks::default(),
            )
            .unwrap();
        assert_eq!(
            known.iter().map(|p| p.opcode).collect::<Vec<_>>(),
            [7, 63, 63, 63, 63, 114, 80]
        );
        assert!(String::from_utf8_lossy(&known.last().unwrap().payload).contains("Crafting 1/11"));
        data.stats.set_base(12, 11);
        let reopened = journal
            .opened(
                259,
                &mut ui,
                &data.variables,
                3,
                journal.marks(&data),
                &crate::interface_hooks::Hooks::default(),
            )
            .unwrap();
        assert_eq!(reopened.len(), 6);
        assert!(
            String::from_utf8_lossy(&reopened.last().unwrap().payload).contains("Vambraces 0/1")
        );
        assert_eq!(data.tower_diary.unwrap().done[2], false);
        data.variables = Variables::restore(BTreeMap::from([(1, 0)]));
        assert_eq!(journal.login_packets(&data).unwrap(), reopened);
        let legacy=data.tower_diary;
        data.tower_run.begin();
        data.tower_run.service(gielinor_world::tower_run::Service::Forge);
        let current=journal.opened_for_run(259,&mut ui,&data.variables,3,journal.marks(&data),&crate::interface_hooks::Hooks::default(),data.tower_run).unwrap();
        let body=String::from_utf8_lossy(&current.last().unwrap().payload);
        assert!(body.contains("Cinderworks"));
        assert!(body.contains("Done: Workshop"));
        assert!(body.contains("Next: Kitchen"));
        assert!(!body.contains("Hides"));
        assert_eq!(journal.login_packets(&data).unwrap(),current);
        assert_eq!(data.tower_diary,legacy);
    }
}
pub struct Journal {
    pub diary: Option<Diary>,
    pub interface: u16,
    pub tabs: [u16; 3],
    pub parent: Component,
    pub contents: Component,
    pub click_layer: Component,
    pub selected: Binding,
    pub playtime: u16,
    pub switching: Option<Switching>,
}
pub struct Diary {
    pub header: u32,
    pub hidden: [u32; 4],
    pub skills: [u8; 3],
}
impl Diary {
    pub(crate) fn packets(
        &self,
        marks: gielinor_world::tower_diary::Marks,
    ) -> Result<Vec<Packet>, String> {
        self.text_packets(&gielinor_world::tower_diary_ui::body(marks))
    }
    pub(crate) fn cinderworks_packets(&self,run:gielinor_world::tower_run::Run)->Result<Vec<Packet>,String> {
        self.text_packets(&run.journal())
    }
    fn text_packets(&self,text:&str)->Result<Vec<Packet>,String> {
        let mut packets = self.hidden.map(|c| interfaces::hide(c, true)).to_vec();
        packets.push(
            client_script::run(600, &[0, 0, 10, self.header as i32].map(Argument::Int))
                .map_err(|e| format!("{e:?}"))?,
        );
        packets.push(
            interfaces::text(self.header, text)
                .map_err(|e| format!("{e:?}"))?,
        );
        Ok(packets)
    }
}
impl Journal {
    pub fn opened_for_run(&self,interface:u16,ui:&mut State,vars:&Variables,combat:i32,
        marks:Option<gielinor_world::tower_diary::Marks>,hooks:&crate::interface_hooks::Hooks,run:gielinor_world::tower_run::Run)->Result<Vec<Packet>,String> {
        let mut packets=self.opened(interface,ui,vars,combat,if run.id>0 {None}else{marks},hooks)?;
        if run.id>0 && (interface==self.interface || interface==self.tabs[2]) {
            if let Some(diary)=&self.diary {packets.extend(diary.cinderworks_packets(run)?);}
        }
        Ok(packets)
    }
    pub fn switch_for_run(&self,tab:usize,ui:&mut State,vars:&mut Variables,combat:i32,member:bool,logged_in:bool,
        marks:Option<gielinor_world::tower_diary::Marks>,hooks:&crate::interface_hooks::Hooks,run:gielinor_world::tower_run::Run)->Result<Vec<Packet>,String> {
        let mut packets=self.switch(tab,ui,vars,combat,member,logged_in,if run.id>0 {None}else{marks},hooks)?;
        if run.id>0 {if let Some(diary)=&self.diary {packets.extend(diary.cinderworks_packets(run)?);}}
        Ok(packets)
    }
    /// FloorOneScript.restoreDiaryCounts -> syncDiary -> persistDiaryCounts
    /// refreshes the task text on login even when another tab is selected.
    /// This covers presentation; gameplay sync side effects remain separate.
    pub fn login_packets(
        &self,
        data: &gielinor_world::simulation::PlayerData,
    ) -> Result<Vec<Packet>, String> {
        if data.tower_run.id>0 {
            return self.diary.as_ref().map_or_else(||Ok(Vec::new()),|diary|diary.cinderworks_packets(data.tower_run));
        }
        match (&self.diary, self.marks(data)) {
            (Some(diary), Some(marks)) => diary.packets(marks),
            _ => Ok(Vec::new()),
        }
    }
    pub fn marks(
        &self,
        data: &gielinor_world::simulation::PlayerData,
    ) -> Option<gielinor_world::tower_diary::Marks> {
        let diary = self.diary.as_ref()?;
        Some(data.tower_diary?.marks(
            diary.skills.map(|id| data.stats.base(id) as i32),
            data.tower_arrival.staff_taken,
        ))
    }
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
        stats: &Symbols,
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
            diary: Some(Diary {
                header: symbol(components, "area_task:header")? as u32,
                hidden: [
                    "area_task:taskbox",
                    "area_task:border",
                    "area_task:scrollbar",
                    "area_task:com_5",
                ]
                .map(|name| symbol(components, name).map(|v| v as u32))
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?
                .try_into()
                .unwrap(),
                skills: ["crafting", "smithing", "firemaking"]
                    .map(|name| {
                        u8::try_from(symbol(stats, name)?)
                            .map_err(|_| "Invalid diary skill".to_owned())
                    })
                    .into_iter()
                    .collect::<Result<Vec<_>, String>>()?
                    .try_into()
                    .unwrap(),
            }),
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
        marks: Option<gielinor_world::tower_diary::Marks>,
        hooks: &crate::interface_hooks::Hooks,
    ) -> Result<Vec<Packet>, String> {
        if interface == self.tabs[2] {
            return match (&self.diary, marks) {
                (Some(diary), Some(marks)) => diary.packets(marks),
                _ => Ok(Vec::new()),
            };
        }
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
        if index == 2 {
            if let (Some(diary), Some(marks)) = (&self.diary, marks) {
                packets.extend(diary.packets(marks)?);
            }
        }
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
        marks: Option<gielinor_world::tower_diary::Marks>,
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
        packets.extend(self.opened(self.interface, &mut next, &variables, combat, marks, hooks)?);
        *ui = next;
        *vars = variables;
        Ok(packets)
    }
}
