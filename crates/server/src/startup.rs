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
            "spotanim",
            "projanim",
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
            "queue",
            "midi",
            "dbtable",
        ] {
            groups.insert(kind, symbols::load(directories, kind, None)?);
        }
        groups.insert(
            "component",
            symbols::load(directories, "component", Some(&groups["interface"]))?,
        );
        groups.insert(
            "dbcol",
            symbols::load(directories, "dbcol", Some(&groups["dbtable"]))?,
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
    runtime.world.music = Some(crate::music::resolve(cache, names, &runtime.definitions)?);
    runtime.music_component = Some(names.id("component", "music:now_playing_text")?);
    let tables = cache.enums()?;
    let parameters = cache.parameters()?;
    let stat = |name| u8::try_from(names.id("stat", name)?).map_err(io::Error::other);
    runtime.world.prayer = Some(crate::prayer::resolve(names, &runtime.definitions)?);
    let varp = |name| -> io::Result<gielinor_world::variables::Varp> {
        runtime
            .definitions
            .varps
            .get(&names.id("varp", name)?)
            .copied()
            .ok_or_else(|| io::Error::other(format!("Missing session varp {name}")))
    };
    let binding = |name| -> io::Result<gielinor_world::bank_actions::Binding> {
        let bit = *runtime
            .definitions
            .varbits
            .get(&names.id("varbit", name)?)
            .ok_or_else(|| io::Error::other(format!("Missing session varbit {name}")))?;
        let base = *runtime
            .definitions
            .varps
            .get(&(bit.base as u32))
            .ok_or_else(|| io::Error::other(format!("Missing base for session varbit {name}")))?;
        bit.extract(0)
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        Ok(gielinor_world::bank_actions::Binding { base, bit })
    };
    runtime.world.login_resets = Some(gielinor_world::login_resets::Config {
        quick_prayer: binding("quickprayer_active")?,
        canoe_station: binding("canoe_startfrom")?,
        canoe_type: binding("canoe_type")?,
        canoe_avoid_interface: binding("canoe_avoid_if")?,
    });
    runtime.world.session_vars = Some(gielinor_world::session_vars::Config {
        clock_timer: names.short("timer", "map_clock")?,
        special_timer: names.short("timer", "spec_regen")?,
        milliseconds: binding("date_milliseconds_past_minute")?,
        seconds: binding("date_seconds_past_minute")?,
        map_clock: varp("map_clock")?,
        playtime: varp("playtime")?,
        energy: varp("sa_energy")?,
        new_account: binding("new_player_account")?,
    });
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
        // BaseObjs.regen_bracelet is a Kotlin property alias, not a cache symbol.
        bracelet: names.id("obj", "jewl_bracelet_regen")? as i32,
        drained_timer: names.short("timer", "stat_regen")?,
        boosted_timer: names.short("timer", "stat_boost_restore")?,
        health_timer: names.short("timer", "health_regen")?,
        rapid_timer: names.short("timer", "rapidrestore_regen")?,
    });
    let prayers = crate::prayer_book::resolve(names, &runtime.definitions, &parameters, &tables)?;
    runtime.world.prayer_book = Some(gielinor_world::prayer_runtime::Config {
        book: prayers.book,
        drain: prayers.drain,
        regeneration: runtime.world.regeneration.as_ref().unwrap().clone(),
        drain_timer: names.short("timer", "prayer_drain")?,
        depletion_sound: prayers.depletion_sound,
    });
    runtime.message_box = Some(crate::message_box::Presentation::resolve(names)?);
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
        // The owned account worker supplies persisted realm messages before listen.
        welcome: None,
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
            n("stat"),
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
    runtime.configure_wearpos(
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
        .wearpos(n("varp"), n("synth"), names.combat_stats()?)
        .map_err(io::Error::other)?,
    )?;
    runtime.configure_bulwark_wear(
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
        .bulwark(n("varp"), n("spotanim"))
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
    let mut foods =
        crate::content::foods(n("obj"), &runtime.definitions).map_err(io::Error::other)?;
    crate::content::additional_ordinary_foods(n("obj"),&runtime.definitions,&mut foods).map_err(io::Error::other)?;
    crate::content::prepared_fruit_foods(n("obj"),&runtime.definitions,&mut foods).map_err(io::Error::other)?;
    let mut slot_preserving_foods =
        crate::content::wiki_multipart_foods(n("obj"), &runtime.definitions, &mut foods)
            .map_err(io::Error::other)?;
    slot_preserving_foods.extend(crate::content::additional_multipart_foods(n("obj"),&runtime.definitions,&mut foods).map_err(io::Error::other)?);
    slot_preserving_foods.insert(crate::content::stew_food(n("obj"),&runtime.definitions,&mut foods).map_err(io::Error::other)?);
    let mut boost_foods=crate::content::boost_pie_foods(n("obj"),n("stat"),&runtime.definitions,&mut foods).map_err(io::Error::other)?;
    let energy_foods=crate::content::energy_foods(n("obj"),n("stat"),&runtime.definitions,&mut foods).map_err(io::Error::other)?;
    boost_foods.selected.extend(energy_foods.selected);boost_foods.boosts.extend(energy_foods.boosts);boost_foods.energy.extend(energy_foods.energy);
    slot_preserving_foods.extend(boost_foods.selected);
    let healing_foods=crate::content::dynamic_foods(n("obj"),&runtime.definitions,&mut foods).map_err(io::Error::other)?;
    slot_preserving_foods.extend(healing_foods.selected);
    runtime.configure_foods(held, foods, names.short("seq", "human_eat")?)?;
    runtime.world.food_healing=healing_foods.healing;
    runtime.world.food_boosts=boost_foods.boosts;
    runtime.world.food_energy=boost_foods.energy;
    runtime.world.slot_preserving_foods = slot_preserving_foods;
    runtime.configure_potions(n("obj"), n("stat"))?;
    runtime.world.item_packs =
        crate::content::item_packs(n("obj"), &runtime.definitions).map_err(io::Error::other)?;
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
    runtime.world.ranged = Some(
        crate::ranged_config::resolve(names, &parameters, &runtime.definitions)
            .map_err(io::Error::other)?,
    );
    runtime.world.ranged_weapons = Some(
        crate::ranged_weapon_config::resolve(
            names,
            &parameters,
            &runtime.definitions,
            &cache.projectiles()?,
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
    let creator_kits = crate::player_design::Kits::resolve(
        &cache.db_rows()?,
        &cache.db_tables()?,
        n("dbtable"),
        n("dbcol"),
    )
    .map_err(io::Error::other)?;
    runtime.configure_player_design(
        crate::player_design::Bindings::resolve(
            creator_kits,
            n("component"),
            n("varbit"),
            &runtime.definitions,
        )
        .map_err(io::Error::other)?,
    )?;
    runtime.configure_tutorial(crate::tutorial_character::Config::resolve(
        names,
        &runtime.definitions,
    )?)?;
    let leave_items = gielinor_world::tutorial_leave::ITEMS
        .into_iter()
        .map(|(name, _)| names.id("obj", name).map(|id| id as i32))
        .collect::<io::Result<Vec<_>>>()?;
    runtime.configure_tutorial_leave(
        leave_items
            .try_into()
            .map_err(|_| io::Error::other("Invalid tutorial leave kit length"))?,
        names.id("obj", "coins")? as i32,
    )?;
    runtime.configure_magic_tutor(
        names.id("obj", "airrune")? as i32,
        names.id("obj", "mindrune")? as i32,
    )?;
    let tutorial_gates = gielinor_world::tutorial_gates::BINDINGS
        .into_iter()
        .map(|(name, gate)| names.id("loc", name).map(|id| (id, gate)))
        .collect::<io::Result<_>>()?;
    runtime.configure_tutorial_gates(tutorial_gates)?;
    runtime.configure_survival_tutor(
        names.id("obj", "net")? as i32,
        names.id("obj", "bronze_axe")? as i32,
        names.id("obj", "tinderbox")? as i32,
    )?;
    runtime.configure_master_chef(
        names.id("obj", "pot_flour")? as i32,
        names.id("obj", "bucket_water")? as i32,
    )?;
    runtime.configure_quest_tutor()?;
    runtime.configure_lecture_tutors()?;
    runtime.configure_tutorial_bank(
        names.id("loc", "tut2_bankbooth")?,
        names.id("loc", "pollbooth_green")?,
        names.id("obj", "coins")? as i32,
    )?;
    runtime.configure_tutorial_combat(
        names.short("npc", "tut2_giantrat")?,
        names.short("npc", "tut2_chicken")?,
    )?;
    runtime.configure_combat_tutor(
        names.id("obj", "bronze_dagger")? as i32,
        names.id("obj", "bronze_sword")? as i32,
        names.id("obj", "wooden_shield")? as i32,
        names.id("obj", "shortbow")? as i32,
        names.id("obj", "bronze_arrow")? as i32,
    )?;
    runtime.configure_mining_tutor(
        names.id("obj", "bronze_pickaxe")? as i32,
        names.id("obj", "hammer")? as i32,
    )?;
    runtime.configure_tutorial_woodcutting(
        [
            names.id("loc", "tut2_tree")?,
            names.id("loc", "tut2_tree2")?,
        ],
        names.id("loc", "tut2_oak")?,
        names.id("obj", "tut2_logs")? as i32,
        names.short("seq", "human_woodcutting_bronze_axe")?,
        u8::try_from(names.id("stat", "woodcutting")?).map_err(io::Error::other)?,
    )?;
    runtime.configure_tutorial_fishing(
        names.short("npc", "0_26_95_tut2_fishing")?,
        names.id("obj", "tut2_raw_shrimp")? as i32,
        names.short("seq", "human_smallnet")?,
        u8::try_from(names.id("stat", "fishing")?).map_err(io::Error::other)?,
    )?;
    runtime.configure_tutorial_mining(
        [
            names.id("loc", "tut2_tinrock")?,
            names.id("loc", "tut2_copperrock")?,
        ],
        [
            names.id("obj", "tin_ore")? as i32,
            names.id("obj", "copper_ore")? as i32,
        ],
        names.id("obj", "bronze_pickaxe")? as i32,
        names.short("seq", "human_mining_bronze_pickaxe")?,
        u8::try_from(names.id("stat", "mining")?).map_err(io::Error::other)?,
    )?;
    runtime.configure_tutorial_bread(
        names.id("loc", "tut2_range")?,
        names.id("obj", "bread_dough")? as i32,
        names.id("obj", "bread")? as i32,
        names.short("seq", "human_cooking")?,
        u8::try_from(names.id("stat", "cooking")?).map_err(io::Error::other)?,
    )?;
    runtime.configure_tutorial_shrimp(
        names.id("loc", "fire")?,
        names.id("obj", "tut2_raw_shrimp")? as i32,
        names.id("obj", "shrimp")? as i32,
        names.short("seq", "human_firecooking")?,
        u8::try_from(names.id("stat", "cooking")?).map_err(io::Error::other)?,
    )?;
    runtime.configure_tutorial_fire(
        names.id("obj", "tinderbox")? as i32,
        names.id("obj", "tut2_logs")? as i32,
        names.id("loc", "fire")?,
        names.short("seq", "human_createfire")?,
        u8::try_from(names.id("stat", "firemaking")?).map_err(io::Error::other)?,
    )?;
    runtime.configure_tutorial_dough(
        names.id("obj", "pot_flour")? as i32,
        names.id("obj", "bucket_water")? as i32,
        names.id("obj", "pot_empty")? as i32,
        names.id("obj", "bucket_empty")? as i32,
        names.id("obj", "bread_dough")? as i32,
    )?;
    for recipe in crate::content::assembly(
        n("obj"),
        &runtime.definitions,
        u8::try_from(names.id("stat", "cooking")?).map_err(io::Error::other)?,
    )
    .map_err(io::Error::other)?
    {
        runtime
            .world
            .held_use_handlers
            .register(
                gielinor_world::held_use::Scope::Types,
                recipe.first,
                recipe.second,
                gielinor_world::held_use::Handler::Assembly(recipe),
            )
            .map_err(|e| io::Error::other(format!("Assembly registration: {e:?}")))?;
    }
    for (recipe,ticks) in crate::content::admiral_pie_assembly(n("obj"),&runtime.definitions,u8::try_from(names.id("stat","cooking")?).map_err(io::Error::other)?).map_err(io::Error::other)? {
        runtime.world.held_use_handlers.register(gielinor_world::held_use::Scope::Types,recipe.first,recipe.second,gielinor_world::held_use::Handler::AssemblyTimed{recipe,ticks}).map_err(|e|io::Error::other(format!("Admiral assembly registration: {e:?}")))?;
    }
    let stew=crate::content::stew_recipes(n("obj"),&runtime.definitions,u8::try_from(names.id("stat","cooking")?).map_err(io::Error::other)?).map_err(io::Error::other)?;
    for recipe in stew.assembly {
        runtime.world.held_use_handlers.register(gielinor_world::held_use::Scope::Types,recipe.first,recipe.second,gielinor_world::held_use::Handler::Assembly(recipe)).map_err(|e|io::Error::other(format!("Stew assembly registration: {e:?}")))?;
    }
    runtime.world.milking = Some(
        crate::content::milking(
            n("obj"),
            n("loc"),
            n("seq"),
            n("synth"),
            &runtime.definitions,
        )
        .map_err(io::Error::other)?,
    );
    let mut cooking=crate::content::manual_fire_cooking(n("obj"),n("loc"),n("seq"),&runtime.definitions,u8::try_from(names.id("stat","cooking")?).map_err(io::Error::other)?).map_err(io::Error::other)?;
    crate::content::additional_fire_fish(n("obj"),&runtime.definitions,&mut cooking).map_err(io::Error::other)?;
    if cooking.recipes.insert(stew.cooking.raw,stew.cooking).is_some() {return Err(io::Error::other("Duplicate stew cooking registration"));}
    cooking.tutorial=runtime.world.tutorial_shrimp.map(|c|c.progression);
    runtime.world.cooking=Some(cooking);
    runtime.world.water_sources = Some(crate::content::water_sources(n("obj"), n("loc"), &runtime.definitions).map_err(io::Error::other)?);
    for recipe in crate::content::grinding(n("obj"),n("seq"),&runtime.definitions).map_err(io::Error::other)?.into_iter().chain([crate::content::nihil_grinding(n("obj"),n("seq"),&runtime.definitions).map_err(io::Error::other)?]) {
        runtime.world.held_use_handlers.register(gielinor_world::held_use::Scope::Types,recipe.tool,recipe.input,gielinor_world::held_use::Handler::Grinding(recipe))
            .map_err(|e|io::Error::other(format!("Grinding registration: {e:?}")))?;
    }
    runtime.configure_smelting_names(names)?;
    runtime.configure_dough_names(names)?;
    runtime.configure_herb_cleaning_names(names)?;
    runtime.configure_finished_potions_names(names)?;
    runtime.configure_potion_upgrades_names(names)?;
    runtime.configure_combat_mixing_names(names)?;
    runtime.configure_unfinished_potions_names(names)?;
    runtime.configure_swamp_tar_names(names)?;
    runtime.configure_anvil(crate::anvil::Config::resolve(
        names,
        &runtime.definitions,
        &tables,
    )?)?;
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

/// Explicit staged setup for the implemented elemental spells. The host supplies
/// the isolated source-derived rune overrides and experience multiplier.
/// This is not called by normal startup until reference checks are accepted.
pub fn configure_elemental_spells<B: AccountBackend>(
    runtime: &mut Runtime<B>,
    cache: &mut GameCache,
    names: &Names,
    requirements: &crate::elemental_requirements::Catalog,
    global_xp_rate: f64,
) -> io::Result<()> {
    if runtime.world.melee.is_none()
        || runtime.world.combat_wear.is_none()
        || runtime.world.autocast_selection.is_none()
    {
        return Err(io::Error::other(
            "Configure base combat before elemental spells",
        ));
    }
    let tables = cache.enums()?;
    let parameters = cache.parameters()?;
    let projectiles = cache.projectiles()?;
    let projectile = projectiles
        .get(&names.id("projanim", "magic_spell")?)
        .ok_or_else(|| io::Error::other("Missing magic_spell projectile definition"))?
        .style;
    let input = crate::magic_config::Inputs {
        objs: names.group("obj"),
        params: names.group("param"),
        bits: names.group("varbit"),
        enums: names.group("enum"),
        categories: names.group("category"),
        parameter_types: &parameters,
        tables: &tables,
        defs: &runtime.definitions,
    };
    let spells = crate::elemental_spell_config::resolve(
        names,
        &runtime.definitions,
        input.combat_spells().map_err(io::Error::other)?,
        projectile,
        requirements,
        global_xp_rate,
    )
    .map_err(io::Error::other)?;
    let buttons = input
        .spell_buttons(spells.spells.keys().copied())
        .map_err(io::Error::other)?;
    let combat = crate::spell_combat_config::resolve(names, &parameters, &runtime.definitions)
        .map_err(io::Error::other)?;
    runtime.configure_elemental_magic(combat, spells, buttons)
}
