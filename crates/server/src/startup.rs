//! Resolve the implemented runtime content from read-only cache and symbol inputs.
use crate::runtime::{AccountBackend, Runtime};
use gielinor_cache::{
    game::GameCache,
    symbols::{self, Symbols},
};
use std::{collections::BTreeMap, io, path::Path};

pub struct Names(BTreeMap<&'static str, Symbols>);
impl Names {
    pub fn load(directories: &[&Path]) -> io::Result<Self> {
        let mut groups = BTreeMap::new();
        for kind in [
            "interface",
            "inv",
            "stat",
            "seq",
            "synth",
            "param",
            "obj",
            "npc",
            "loc",
            "headbar",
            "hitmark",
            "area",
            "category",
            "varp",
            "varbit",
            "enum",
            "content",
            "font",
            "timer",
        ] {
            groups.insert(kind, symbols::load(directories, kind, None)?);
        }
        groups.insert(
            "component",
            symbols::load(directories, "component", Some(&groups["interface"]))?,
        );
        Ok(Self(groups))
    }
    pub fn group(&self, kind: &str) -> &Symbols {
        &self.0[kind]
    }
    pub fn id(&self, kind: &str, name: &str) -> io::Result<u32> {
        self.0
            .get(kind)
            .and_then(|g| g.get(name))
            .and_then(|id| u32::try_from(*id).ok())
            .ok_or_else(|| io::Error::other(format!("Missing or invalid {kind} symbol {name}")))
    }
    pub fn short(&self, kind: &str, name: &str) -> io::Result<u16> {
        u16::try_from(self.id(kind, name)?).map_err(io::Error::other)
    }
    pub fn combat_stats(&self) -> io::Result<[u8; 7]> {
        let mut result = [0; 7];
        for (slot, name) in [
            "attack",
            "defence",
            "strength",
            "hitpoints",
            "ranged",
            "magic",
            "prayer",
        ]
        .into_iter()
        .enumerate()
        {
            result[slot] = u8::try_from(self.id("stat", name)?).map_err(io::Error::other)?;
        }
        Ok(result)
    }
}

/// Configure before publication/admission. The caller discards the entire runtime
/// on any error, so an incomplete configuration is never exposed to connections.
pub fn configure<B: AccountBackend>(
    runtime: &mut Runtime<B>,
    cache: &mut GameCache,
    names: &Names,
) -> io::Result<()> {
    let n = |kind| names.group(kind);
    let tables = cache.enums()?;
    let parameters = cache.parameters()?;
    let stat = |name| u8::try_from(names.id("stat", name)?).map_err(io::Error::other);
    runtime.world.prayer = Some(crate::prayer::resolve(names, &runtime.definitions)?);
    runtime.world.regeneration = Some(gielinor_world::player_regen::Config {
        skills: runtime
            .definitions
            .stats
            .keys()
            .map(|id| u8::try_from(*id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(io::Error::other)?,
        hitpoints: stat("hitpoints")?,
        prayer: stat("prayer")?,
        bracelet: names.id("obj", "regen_bracelet")? as i32,
        drained_timer: names.short("timer", "stat_regen")?,
        boosted_timer: names.short("timer", "stat_boost_restore")?,
        health_timer: names.short("timer", "health_regen")?,
        rapid_timer: names.short("timer", "rapidrestore_regen")?,
    });
    runtime.floor_progression = Some(gielinor_world::floor_progression::Policy {
        attack: stat("attack")?,
        defence: stat("defence")?,
        strength: stat("strength")?,
        hitpoints: stat("hitpoints")?,
        runecrafting: stat("runecrafting")?,
        herblore: stat("herblore")?,
    });
    let vial = names.id("obj", "vial_water")?;
    if !runtime.definitions.items.contains_key(&vial) {
        return Err(io::Error::other("Missing vial-water definition"));
    }
    runtime.world.location_handlers.insert(
        (names.id("loc", "threshold_vial_crate")?, 1),
        gielinor_world::location_content::Handler::VialCrate {
            origin: gielinor_world::CoordGrid::new(10368, 64, 0)
                .map_err(|e| io::Error::other(format!("{e:?}")))?,
            item: i32::try_from(vial).map_err(io::Error::other)?,
        },
    );
    runtime.world.location_handlers.insert(
        (names.id("loc", "altar")?, 1),
        gielinor_world::location_content::Handler::PrayerAltar {
            origin: gielinor_world::CoordGrid::new(10368, 64, 0)
                .map_err(|e| io::Error::other(format!("{e:?}")))?,
            prayer: stat("prayer")?,
        },
    );
    runtime.world.multiway_area = Some(names.short("area", "multiway")?);
    runtime.combat_display = Some(crate::combat_display::resolve(
        names,
        &runtime.definitions,
        &parameters,
        &cache.headbars()?,
    )?);
    runtime.configure_interfaces(cache.components()?)?;
    runtime.enable_player_synchronization(crate::runtime::Presentation {
        animations: crate::appearance::AnimationPolicy::resolve(n("seq"), n("param"))
            .map_err(io::Error::other)?,
        combat_stats: names.combat_stats()?,
        radius: 15,
    })?;
    runtime.configure_login(crate::login_init::Presentation {
        variables: crate::login_init::Config::resolve(n("varp"), n("varbit"), &runtime.definitions)
            .map_err(io::Error::other)?,
        frames: crate::gameframe::standard(
            n("interface"),
            n("component"),
            n("enum"),
            n("varbit"),
            &tables,
            &runtime.definitions,
        )
        .map_err(io::Error::other)?,
        welcome: Some("Welcome to Project Gielinor.".into()),
        broadcast: None,
    })?;
    let mut hooks =
        crate::interface_hooks::Hooks::standard(n("interface"), n("component"), n("enum"), &tables)
            .map_err(io::Error::other)?;
    let inventory_id = names.id("inv", "inv")?;
    let inventory_capacity = runtime
        .definitions
        .inventories
        .get(&inventory_id)
        .ok_or_else(|| io::Error::other("Missing player inventory definition"))?
        .size;
    hooks
        .inventory(
            gielinor_world::ui::Component(names.id("component", "inventory:items")?),
            inventory_capacity,
        )
        .map_err(io::Error::other)?;
    runtime.configure_interface_hooks(hooks)?;
    runtime.configure_interface_controls(
        crate::interface_controls::standard(
            n("component"),
            n("varbit"),
            n("varp"),
            &runtime.definitions,
        )
        .map_err(io::Error::other)?,
    )?;
    runtime.configure_journal(
        crate::journal::Journal::resolve(
            n("interface"),
            n("component"),
            n("varbit"),
            n("varp"),
            &runtime.definitions,
        )
        .map_err(io::Error::other)?,
    )?;
    runtime.configure_combat_tab(
        crate::combat_tab::resolve(
            n("interface"),
            n("component"),
            n("varp"),
            n("varbit"),
            n("enum"),
            n("category"),
            &tables,
            &runtime.definitions,
        )
        .map_err(io::Error::other)?,
    )?;
    runtime.configure_combat_wear(
        crate::magic_config::Inputs {
            objs: n("obj"),
            params: n("param"),
            bits: n("varbit"),
            enums: n("enum"),
            categories: n("category"),
            parameter_types: &parameters,
            tables: &tables,
            defs: &runtime.definitions,
        }
        .resolve(names.combat_stats()?)
        .map_err(io::Error::other)?,
    )?;
    runtime.configure_autocast_picker(
        crate::autocast_picker::Presentation::resolve(n("interface"), n("component"), n("varp"))
            .map_err(io::Error::other)?,
        crate::magic_config::Inputs {
            objs: n("obj"),
            params: n("param"),
            bits: n("varbit"),
            enums: n("enum"),
            categories: n("category"),
            parameter_types: &parameters,
            tables: &tables,
            defs: &runtime.definitions,
        }
        .selection(u8::try_from(names.id("stat", "magic")?).map_err(io::Error::other)?)
        .map_err(io::Error::other)?,
    )?;
    runtime.configure_gameframe_moves(
        crate::gameframe_move::Config::standard(
            n("interface"),
            n("component"),
            n("varp"),
            n("varbit"),
            &runtime.definitions,
        )
        .map_err(io::Error::other)?,
    )?;
    let held = names.id("component", "inventory:items")?;
    let mut worn = BTreeMap::new();
    for slot in [0, 1, 2, 3, 4, 5, 7, 9, 10, 12, 13] {
        worn.insert(
            names.id("component", &format!("wornitems:slot{slot}"))?,
            slot,
        );
    }
    runtime.configure_equipment(held, worn, n("param"))?;
    runtime.configure_foods(
        held,
        crate::content::foods(n("obj"), &runtime.definitions).map_err(io::Error::other)?,
        names.short("seq", "human_eat")?,
    )?;
    runtime.configure_potions(n("obj"), n("stat"))?;
    runtime.world.doors =
        crate::doors::resolve(n("content"), n("param"), &parameters, &runtime.definitions)
            .map_err(io::Error::other)?;
    runtime.world.pickup_effects = Some(gielinor_world::ground_transfer::PickupEffects {
        sound: names.short("synth", "pick2")?,
        animation: names.short("seq", "human_pickuptable")?,
    });
    runtime.configure_dropping(
        crate::drop_dialog::Presentation {
            parent: gielinor_world::ui::Component(names.id("component", "chatbox:chatmodal")?),
            unclamp: names.id("varbit", "chatmodal_unclamp")?,
            object_interface: names.short("interface", "objectbox")?,
            object_button: gielinor_world::ui::Component(
                names.id("component", "objectbox:universe")?,
            ),
            object_model: gielinor_world::ui::Component(names.id("component", "objectbox:item")?),
            object_text: gielinor_world::ui::Component(names.id("component", "objectbox:text")?),
            choice_interface: names.short("interface", "chatmenu")?,
            choice_button: gielinor_world::ui::Component(
                names.id("component", "chatmenu:options")?,
            ),
            destroy_interface: names.short("interface", "confirmdestroy")?,
            destroy_button: gielinor_world::ui::Component(
                names.id("component", "confirmdestroy:universe")?,
            ),
        },
        crate::drop_dialog::resolve(
            n("param"),
            n("varbit"),
            n("synth"),
            &parameters,
            &runtime.definitions,
        )
        .map_err(io::Error::other)?,
    )?;
    runtime.configure_banking(
        crate::runtime::BankPresentation {
            main_interface: names.short("interface", "bankmain")?,
            side_interface: names.short("interface", "bankside")?,
            main_components: vec![
                names.id("component", "bankmain:items")?,
                names.id("component", "bankmain:com_12")?,
            ],
            side_component: names.id("component", "bankside:items")?,
            default_capacity: 800,
        },
        n("varbit"),
        n("param"),
    )?;
    runtime.configure_bank_controls(n("component"), n("varbit"))?;
    runtime.configure_bank_sidebar(n("param"), n("varp"), n("content"), &parameters)?;
    let bonuses = crate::bonuses::resolve(
        n("param"),
        n("obj"),
        n("category"),
        n("varp"),
        &parameters,
        &runtime.definitions,
    )
    .map_err(io::Error::other)?;
    runtime.world.melee = Some(
        crate::melee_config::resolve(
            names,
            &parameters,
            &runtime.definitions,
            bonuses.items.clone(),
        )
        .map_err(io::Error::other)?,
    );
    runtime.world.npc_melee = Some(
        crate::npc_melee_config::resolve(names, &runtime.definitions, &parameters)
            .map_err(io::Error::other)?,
    );
    let sequences = cache.sequences()?;
    runtime.world.npc_death = Some(
        crate::npc_death_config::resolve(names, &runtime.definitions, &parameters, &sequences)
            .map_err(io::Error::other)?,
    );
    let combat_catalog =
        crate::kronos_combat::Catalog::load(Path::new(crate::kronos_combat::DEFAULT_DIRECTORY));
    if !combat_catalog.failures.is_empty() {
        eprintln!(
            "NPC combat data: {} ids loaded, {} file failures",
            combat_catalog.by_id.len(),
            combat_catalog.failures.len()
        );
    }
    runtime
        .world
        .configure_npc_bindings(combat_catalog.bindings(&sequences));
    runtime.configure_bonuses(
        bonuses,
        crate::bonuses::Styles::resolve(n("enum"), &tables).map_err(io::Error::other)?,
        Some(crate::bonuses::BankDisplay::resolve(n("component")).map_err(io::Error::other)?),
    )?;
    runtime.enable_ground_synchronization()?;
    runtime.world.configure_npc_combat(
        crate::npc_combat_config::resolve(
            n("param"),
            n("category"),
            n("npc"),
            &parameters,
            &runtime.definitions,
        )
        .map_err(io::Error::other)?,
    );
    runtime.world.attack_profiles = crate::attack_config::resolve(
        n("param"),
        n("enum"),
        &parameters,
        &tables,
        &runtime.definitions,
    )
    .map_err(io::Error::other)?;
    let combat_varp = |name: &str| -> io::Result<gielinor_world::variables::Varp> {
        runtime
            .definitions
            .varps
            .get(&names.id("varp", name)?)
            .copied()
            .ok_or_else(|| io::Error::other(format!("Missing combat state varp {name}")))
    };
    runtime.world.combat_state = Some(gielinor_world::combat_state::Config {
        last_npc: combat_varp("lastcombat")?,
        last_pvp: combat_varp("lastcombat_pvp")?,
        aggressive_npc: combat_varp("aggressive_npc")?,
    });
    runtime.configure_bankers(
        &[
            names.id("content", "banker")? as i32,
            names.id("content", "banker_tutor")? as i32,
        ],
        (
            gielinor_world::ui::Component(
                names.id("component", "toplevel_osrs_stretch:mainmodal")?,
            ),
            gielinor_world::ui::Component(
                names.id("component", "toplevel_osrs_stretch:sidemodal")?,
            ),
        ),
    )?;
    let font_id = names.id("font", "q8_full")?;
    let font_files = cache
        .files(13, font_id, None)?
        .ok_or_else(|| io::Error::other("Missing dialogue font"))?;
    let font = gielinor_cache::fonts::decode(
        font_files
            .get(&0)
            .ok_or_else(|| io::Error::other("Missing dialogue font metrics"))?,
    )
    .map_err(|e| io::Error::other(format!("{e:?}")))?;
    runtime.configure_dialogue(
        crate::dialogue::Presentation::resolve(names, font)?,
        names.id("content", "person")? as i32,
        names.id("obj", "flier")? as i32,
    )?;
    for (&id, npc) in &runtime.definitions.npcs {
        if npc.ops[1]
            .as_deref()
            .is_some_and(|op| op.eq_ignore_ascii_case("Attack"))
        {
            runtime
                .world
                .npc_handlers
                .entry((id as u16, 2))
                .or_insert(gielinor_world::npc_interaction::Handler::Combat);
        }
    }
    Ok(())
}
