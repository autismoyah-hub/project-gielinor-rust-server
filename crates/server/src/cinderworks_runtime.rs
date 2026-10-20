//! Explicit authored Floor One configuration, before account or socket admission.
use crate::{runtime::{Runtime,AccountBackend},startup::Names};
use gielinor_cache::game::GameCache;
use gielinor_world::{CoordGrid,cinderworks_floor::Spatial};
use std::io;
fn fail(message:impl Into<String>)->io::Error{io::Error::other(message.into())}
pub fn configure<B:AccountBackend>(runtime:&mut Runtime<B>,cache:&mut GameCache,names:&Names,spatial:&Spatial)->io::Result<()> {
    let tower_origin=gielinor_world::cinderworks_floor::origin();
    let outside = CoordGrid::new(3225, 3218, 0).map_err(|e| fail(format!("{e:?}")))?;
    let config = gielinor_world::tower_cinderworks::Config {
        origin:tower_origin, spatial:spatial.clone(), outside,
        plaque:names.id("loc", "plaque")?,
        portal:names.id("loc", "threshold_portal")?,
        chest:names.id("loc", "threshold_chest")?,
        tools:["bronze_axe", "bronze_pickaxe", "net", "hammer", "tinderbox", "knife"]
            .into_iter().map(|name| names.id("obj", name).map(|id| id as i32)).collect::<io::Result<Vec<_>>>()?,
        scimitar:names.id("obj", "bronze_scimitar")? as i32,
        dagger:names.id("obj", "bronze_dagger")? as i32,
        coins:names.id("obj", "coins")? as i32,
        beer:names.id("obj", "beer")? as i32,
        imp:names.short("npc", "ash_imp")?,
        warden:gielinor_world::tower_cinderworks::WARDEN,
        quench:names.id("loc", "mistmyst_barrel_water")?,
        quench_tiles:gielinor_world::tower_cinderworks::COOLANT_TILES.into_iter().map(|(x,z)|spatial.at(tower_origin,x,z).unwrap()).collect(),
        warning_graphic:names.short("spotanim", "smokepuff")?,
        strike_graphic:names.short("spotanim", "zamorak_flame")?,
        death_animation:names.short("seq", "human_death")?,
        charge_animations:[names.short("seq","imp_attack")?,names.short("seq","giant_update_fire_attack")?],
        quench_animation:names.short("seq","giant_update_fire_defend")?,
        work:Some(gielinor_world::tower_cinderworks::WorkConfig {
            fishing_spot:names.short("npc", "0_48_48_newbiefishing")?, net:names.id("obj", "net")? as i32,
            raw:names.id("obj", "raw_shrimp")? as i32,cooked:names.id("obj", "shrimp")? as i32,
            cooking_loc:names.id("loc", "newbierange")?,fish_skill:names.id("stat", "fishing")? as u8,
            cook_skill:names.id("stat", "cooking")? as u8,fish_animation:names.short("seq", "human_smallnet")?,
            cook_animation:names.short("seq", "human_cooking")?,queue:names.id("queue", "smithing_make")? as i32,
        }),
    };
    let visual=runtime.definitions.npcs.get(&(gielinor_world::tower_cinderworks::WARDEN_VISUAL as u32)).ok_or_else(||fail("Warden visual definition missing"))?;
    if visual.size!=runtime.definitions.npcs[&(config.warden as u32)].size {return Err(fail("Warden visual and authoritative footprints differ"));}
    let sequences=cache.sequences()?;
    let death=names.short("seq","giant_update_fire_death")?;
    for sequence in config.charge_animations.into_iter().chain([config.quench_animation,death]) {
        if !sequences.get(&(sequence as u32)).is_some_and(|s|s.tick_duration()>0) {return Err(fail("Cinderworks presentation requires complete cache animations"));}
    }
    // All standard attack styles share this defend-animation catalog. Match
    // the admitted Warden body without replacing combat or damage handling.
    runtime.world.melee.as_mut().ok_or_else(||fail("NPC combat configuration missing"))?
        .npc_defend.insert(config.warden,config.quench_animation);
    runtime.world.npc_death.as_mut().ok_or_else(||fail("NPC death configuration missing"))?.definitions.insert(config.warden,gielinor_world::npc_death::Definition {animation:death,animation_ticks:sequences[&(death as u32)].tick_duration(),sound:None});
    let portal = gielinor_world::locations::Location::new(outside, config.portal, 0, 0).map_err(fail)?;
    if !runtime.world.npc_death.as_ref().is_some_and(|death| [config.imp,config.warden].iter().all(|id| death.definitions.get(id).is_some_and(|definition| definition.animation_ticks>0))) {
        return Err(fail("Cinderworks guardians require cache-resolved death animations before admission"));
    }
    runtime.world.change_location(portal, true).map_err(fail)?;
    runtime.configure_cinderworks(config)?;
    Ok(())
}
