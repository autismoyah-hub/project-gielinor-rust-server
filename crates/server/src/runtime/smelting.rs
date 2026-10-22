use super::*;
use crate::{
    skill_multi::{Dialog, OptionItem, Selection},
    smelting::Config,
};
impl<B: AccountBackend> Runtime<B> {
    pub fn configure_smelting_names(&mut self, names: &crate::startup::Names) -> io::Result<()> {
        use gielinor_world::smelting::{Furnace, Recipe};
        let definitions: &[(&str, &str, i32, f64, &[(&str, i32)])] = &[
            (
                "bronze_bar",
                "Bronze",
                1,
                6.2,
                &[("copper_ore", 1), ("tin_ore", 1)],
            ),
            ("blurite_bar", "Blurite", 8, 8.0, &[("blurite_ore", 1)]),
            ("iron_bar", "Iron", 15, 12.5, &[("iron_ore", 1)]),
            ("silver_bar", "Silver", 20, 13.7, &[("silver_ore", 1)]),
            (
                "steel_bar",
                "Steel",
                30,
                17.5,
                &[("iron_ore", 1), ("coal", 2)],
            ),
            ("gold_bar", "Gold", 40, 22.5, &[("gold_ore", 1)]),
            (
                "lovakite_bar",
                "Lovakite",
                45,
                20.0,
                &[("lovakite_ore", 1), ("coal", 2)],
            ),
            (
                "mithril_bar",
                "Mithril",
                50,
                30.0,
                &[("mithril_ore", 1), ("coal", 4)],
            ),
            (
                "adamantite_bar",
                "Adamantite",
                70,
                37.5,
                &[("adamantite_ore", 1), ("coal", 6)],
            ),
            (
                "runite_bar",
                "Runite",
                85,
                50.0,
                &[("runite_ore", 1), ("coal", 8)],
            ),
        ];
        let mut recipes = vec![];
        for &(bar, label, level, xp, ingredients) in definitions {
            recipes.push(Recipe {
                impurity: bar == "iron_bar",
                members: false,
                repeat_delay: None,
                bar: names.id("obj", bar)? as i32,
                label: label.into(),
                level,
                xp,
                ingredients: ingredients
                    .iter()
                    .map(|&(name, count)| Ok((names.id("obj", name)? as i32, count)))
                    .collect::<io::Result<_>>()?,
            });
        }
        // Append to preserve existing menu indices and the original recipe order.
        recipes.push(crate::content::lead_smelting(
            names.group("obj"), names.group("stat"), &self.definitions,
        ).map_err(io::Error::other)?);
        recipes.push(crate::content::cupronickel_smelting(
            names.group("obj"), names.group("stat"), &self.definitions,
        ).map_err(io::Error::other)?);
        let skill = u8::try_from(names.id("stat", "smithing")?).map_err(io::Error::other)?;
        let max_level = self
            .definitions
            .stats
            .get(&(skill as u32))
            .ok_or_else(|| io::Error::other("Missing Smithing stat"))?
            .max_level;
        let progression = self
            .tutorial
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure tutorial before smelting"))?
            .progression;
        let goldsmith=crate::content::goldsmith_smelting(names.group("obj"),&self.definitions,&recipes).map_err(io::Error::other)?;
        let types = crate::smelting::item_types(&self.definitions.items);
        let config = Config {
            presentation: crate::skill_multi::Presentation {
                interface: names.short("interface", "skillmulti")?,
                parent: gielinor_world::ui::Component(names.id("component", "chatbox:chatarea")?),
                setup_script: 2046,
            },
            recipes,
            types,
            quantity_varp: names.id("varp", "makexcrafting")?,
            skill,
            animation: names.short("seq", "human_fai_furnace")?,
        };
        self.configure_smelting(
            config,
            gielinor_world::smithing_make::Config {
                queue: i32::try_from(names.id("queue", "smithing_make")?)
                    .map_err(io::Error::other)?,
                skill,
                max_level,
                xp_rate: 1.0,
                tutorial: Some((
                    progression,
                    names.id("obj", "bronze_bar")? as i32,
                    names.id("obj", "bronze_dagger")? as i32,
                )),
            },
        )?;
        self.world.goldsmith_smelting=Some(goldsmith);
        let group = names.id("content", "smithing_furnace")? as i32;
        let mut furnaces: BTreeMap<_, _> = self
            .definitions
            .locations
            .iter()
            .filter(|(_, location)| location.content_group == group)
            .map(|(&id, _)| {
                (
                    id,
                    Furnace {
                        operations: (1 << 1) | (1 << 2),
                        item_use: true,
                        ore_ids: None,
                        tutorial: None,
                    },
                )
            })
            .collect();
        furnaces.insert(
            names.id("loc", "tut2_furnace")?,
            Furnace {
                operations: 1 << 1,
                item_use: true,
                ore_ids: Some([
                    names.id("obj", "tin_ore")? as i32,
                    names.id("obj", "copper_ore")? as i32,
                ]),
                tutorial: Some(progression),
            },
        );
        let group = names.id("content", "smithing_ore")? as i32;
        let mut ores: std::collections::HashSet<i32> = self
            .definitions
            .items
            .iter()
            .filter(|(_, item)| item.content_group == group)
            .map(|(&id, _)| id as i32)
            .collect();
        // Direct target lead IDs have no enriched smithing_ore group or aliases.
        ores.insert(31716);
        ores.insert(31719);
        self.configure_furnaces(furnaces, ores)
    }
    pub fn configure_furnaces(
        &mut self,
        furnaces: BTreeMap<u32, gielinor_world::smelting::Furnace>,
        ores: std::collections::HashSet<i32>,
    ) -> io::Result<()> {
        if !self.connections.is_empty()
            || self.smelting.is_none()
            || furnaces
                .keys()
                .any(|id| !self.definitions.locations.contains_key(id))
            || ores
                .iter()
                .any(|id| !self.definitions.items.contains_key(&(*id as u32)))
        {
            return Err(io::Error::other(
                "Configure valid furnaces after smelting and before admission",
            ));
        }
        let held_items: std::collections::HashSet<_> = ores
            .iter()
            .copied()
            .chain(furnaces.values().filter_map(|f| f.ore_ids).flatten())
            .collect();
        if held_items
            .iter()
            .any(|id| !self.definitions.items.contains_key(&(*id as u32)))
        {
            return Err(io::Error::other("Missing explicitly bound furnace ore"));
        }
        for id in held_items {
            self.world.held_use_types.insert(
                id,
                gielinor_world::held_use::Type {
                    id,
                    content: self.definitions.items[&(id as u32)].content_group,
                },
            );
        }
        self.world.furnaces = furnaces;
        self.world.smelting_ores = ores;
        Ok(())
    }
    pub fn configure_smelting(
        &mut self,
        config: Config,
        make: gielinor_world::smithing_make::Config,
    ) -> io::Result<()> {
        if !self.connections.is_empty()
            || config.recipes.is_empty()
            || config.recipes.len() > crate::skill_multi::MAX_OPTIONS
            || config.skill != make.skill
        {
            return Err(io::Error::other(
                "Configure valid smelting recipes before admission",
            ));
        }
        if !self.definitions.varps.contains_key(&config.quantity_varp)
            || !self.components.contains_key(&config.presentation.parent.0)
        {
            return Err(io::Error::other("Missing smelting quantity varp/parent"));
        }
        for (slot, recipe) in config.recipes.iter().enumerate() {
            let component = gielinor_world::ui::Component::new(
                config.presentation.interface,
                crate::skill_multi::FIRST_OPTION + slot as u16,
            );
            if !self.components.contains_key(&component.0)
                || !config.types.contains_key(&recipe.bar)
                || recipe
                    .ingredients
                    .iter()
                    .any(|(id, amount)| *amount <= 0 || !config.types.contains_key(id))
            {
                return Err(io::Error::other(
                    "Missing smelting option or item definition",
                ));
            }
        }
        self.world.smithing = Some(make);
        self.smelting = Some(config);
        Ok(())
    }
    pub fn open_smelting(&mut self, actor: ActorId) -> io::Result<()> {
        let config = self
            .smelting
            .as_ref()
            .ok_or_else(|| io::Error::other("Smelting is not configured"))?;
        let data = self
            .world
            .data(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        let count = data
            .variables
            .get(self.definitions.varps[&config.quantity_varp].id)
            .clamp(1, crate::skill_multi::MAX_COUNT);
        let dialog = Dialog {
            title: "What would you like to smelt?".into(),
            options: config
                .recipes
                .iter()
                .map(|recipe| OptionItem {
                    object: recipe.bar,
                    label: recipe.label.clone(),
                })
                .collect(),
            verb: 13,
            max_count: 28,
            selected_count: count,
        };
        self.open_skill_multi(actor, config.presentation, dialog)?;
        self.active
            .get_mut(&actor)
            .unwrap()
            .skill_multi
            .as_mut()
            .unwrap()
            .smelting = true;
        Ok(())
    }
    pub(super) fn select_smelting(
        &mut self,
        actor: ActorId,
        selection: Selection,
    ) -> io::Result<()> {
        let config = self
            .smelting
            .as_ref()
            .ok_or_else(|| io::Error::other("Smelting configuration disappeared"))?;
        let varp = config.quantity_varp;
        // Original openSmeltDialog mirrors quantity before checking level/ore.
        self.set_varp(actor, varp, selection.count)?;
        let config = self.smelting.as_ref().unwrap();
        let recipe = config
            .recipes
            .get(selection.index)
            .ok_or_else(|| io::Error::other("Invalid smelting recipe"))?;
        let data = self
            .world
            .data(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        let outcome = recipe
            .select_with_coal_bag(
                &data.inventory,
                &self.world.object_types,
                &config.types,
                &data.stats,
                config.skill,
                config.animation,
                selection.count,
                self.world.coal_bag,
                &data.variables,
                self.active[&actor].character.identity.member,
            )
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        match outcome {
            gielinor_world::smelting::Selection::Denied(message) => {
                if let Some(connection) = self.connection(actor) {
                    let packet = gielinor_protocol::chat::game(0, None, &message)
                        .map_err(|e| io::Error::other(format!("{e:?}")))?;
                    self.transport
                        .send(connection, packet.opcode, &packet.payload)?;
                }
            }
            gielinor_world::smelting::Selection::Start { request, count } => {
                self.world
                    .start_smelting(actor, request, count)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?;
            }
        }
        Ok(())
    }
}
