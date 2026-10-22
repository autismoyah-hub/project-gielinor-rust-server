//! Resolve content's symbolic references against the same immutable type inputs as Kotlin.
#[cfg(test)]
mod tzhaar_pack_tests;
#[cfg(test)]
mod remaining_pack_tests;
mod bank_booths;
mod shearing;
mod bear_drops;
pub use bear_drops::{bear_drops,BearDrops};
mod imp_drops;
pub use imp_drops::imp_drops;
mod cows;
mod farm_chicken_drops;
pub use farm_chicken_drops::{farm_chicken_drops,FarmChickenDrops};
mod ducks;
mod forging_ring;
pub use forging_ring::forging_ring;
mod smithing_catalyst;
pub use smithing_catalyst::smithing_catalyst;
mod goldsmith_smelting;
pub use goldsmith_smelting::goldsmith_smelting;
mod coal_bag;
pub use coal_bag::coal_bag;
pub use ducks::ducks;
pub use cows::{cows,Cows};
pub use shearing::{shearing,sheep_flavour};
pub use bank_booths::{ordinary_bank_booths,normal_dynamic_bank_booths};
mod chicken_coops;
pub use chicken_coops::chicken_coops;
mod spade_tool;
pub use spade_tool::spade_tool;
mod windmill_ladders;
pub use windmill_ladders::windmill_ladders;
mod wizard_tower_ladders;
pub use wizard_tower_ladders::wizard_tower_ladders;
mod signposts;
pub use signposts::{signposts,Signposts};
mod fence_crossings;
pub use fence_crossings::fence_crossings;
mod spiral_stairs;
pub use spiral_stairs::{spiral_stairs,spiral_choices};
mod straight_stairs;
pub use straight_stairs::straight_stairs;
mod ladder_climbs;
pub use ladder_climbs::ladder_climbs;
mod drawers;
pub use drawers::drawers;
mod plain_chest;
pub use plain_chest::plain_chest;
mod bookcase_search;
pub use bookcase_search::bookcase_search;
mod location_content_search;
pub use location_content_search::location_content_search;
mod nettle_pickables;
pub use nettle_pickables::nettle_pickables;
mod flax_pickables;
pub use flax_pickables::flax_pickables;
mod fruit_pickables;
pub use fruit_pickables::fruit_pickables;
mod cyclic_berry_pickables;
pub use cyclic_berry_pickables::cyclic_berry_pickables;
mod perpetual_pickables;
pub use perpetual_pickables::perpetual_pickables;
mod whole_citrus_food;
pub use whole_citrus_food::whole_citrus_food;
mod berries_cream_food;
pub use berries_cream_food::berries_cream_food;
mod hunter_roast_food;
pub use hunter_roast_food::hunter_roast_food;
mod gnome_ingredients;
pub use gnome_ingredients::gnome_ingredients;
mod cooked_karambwan;
pub use cooked_karambwan::cooked_karambwan;
mod gnome_food;
pub use gnome_food::gnome_food;
mod giant_crab_food;
pub use giant_crab_food::giant_crab_food;
mod native_consumables;
pub use native_consumables::native_consumables;
mod standard_tea;
pub use standard_tea::standard_tea;
mod magic_cabbage;
pub use magic_cabbage::magic_cabbage;
mod jangerberries;
pub use jangerberries::jangerberries;
mod spirits_keg;
pub use spirits_keg::spirits_keg;
mod other_gnome_cocktails;
pub use other_gnome_cocktails::other_gnome_cocktails;
mod sgg_blizzard;
pub use sgg_blizzard::sgg_blizzard;
mod normal_ales;
pub use normal_ales::normal_ales;
mod fruit_cocktails;
pub use fruit_cocktails::fruit_cocktails;
mod mead_drinks;
pub use mead_drinks::mead_drinks;
mod flat_drinks;
pub use flat_drinks::flat_drinks;
mod wine_drink;
pub use wine_drink::wine_drink;
mod skillcape_vendors;
pub use skillcape_vendors::skillcape_vendors;
mod cooking_cape_no_burn;
pub use cooking_cape_no_burn::cooking_cape_no_burn;
mod cooking_gauntlets;
pub use cooking_gauntlets::cooking_gauntlets;
mod plain_wine;
pub use plain_wine::{plain_wine,zamorak_wine};
mod crafting_cape_boost;
pub use crafting_cape_boost::crafting_cape_boost;
mod guild_passages;
pub use guild_passages::guild_passages;
mod ordinary_rune_altars;
pub use ordinary_rune_altars::ordinary_rune_altars;
mod shield_crafting;
pub use shield_crafting::{shield_crafting,shield_crafting_registrations};
mod costume_needle_mixedhide;
pub use costume_needle_mixedhide::{costume_needle_mixedhide,costume_needle_mixedhide_registrations};
mod costume_needle_hueycoatl;
pub use costume_needle_hueycoatl::costume_needle_hueycoatl;
mod costume_needle_splitbark;
pub use costume_needle_splitbark::costume_needle_splitbark;
use gielinor_cache::{game::Definitions, symbols::Symbols};
use gielinor_world::food::{self, Edible};
use std::collections::BTreeMap;
mod costume_needle_fabrics;
pub use costume_needle_fabrics::costume_needle_fabrics;
mod costume_needle_dragonhide;
pub use costume_needle_dragonhide::costume_needle_dragonhide;
mod costume_needle_leather;
pub use costume_needle_leather::costume_needle_leather;
mod pheasant_costume;
pub use pheasant_costume::pheasant_costume;
mod birdhouses;
pub use birdhouses::birdhouses;
mod light_sources;
pub use light_sources::light_sources;
mod prospector_outfit;
pub use prospector_outfit::prospector_outfit;
mod deposit_box_quantities;
pub use deposit_box_quantities::deposit_box_quantities;
mod guild_deposit_box;
pub use guild_deposit_box::guild_deposit_box;
mod crafting_guild_bank;
pub use crafting_guild_bank::crafting_guild_bank;
mod imcando_worn_swaps;
pub use imcando_worn_swaps::imcando_worn_swaps;
mod imcando_held_swaps;
pub use imcando_held_swaps::imcando_held_swaps;
mod sbott_tanning;
pub use sbott_tanning::sbott_tanning;
mod member_tanning;
mod standard_tanners;
pub use standard_tanners::standard_tanners;
mod redwood_sections;
pub use redwood_sections::redwood_sections;
mod arctic_pine;
pub use arctic_pine::arctic_pine;
mod special_woodcutting_axes;
pub use special_woodcutting_axes::special_woodcutting_axes;
mod magic_trees;
pub use magic_trees::magic_trees;
mod member_timed_trees;
pub use member_timed_trees::member_timed_trees;
mod special_pickaxes;
pub use special_pickaxes::special_pickaxes;
mod granite_cannonballs;
pub use granite_cannonballs::granite_cannonballs;
mod cupronickel_smelting;
pub use cupronickel_smelting::cupronickel_smelting;
mod lead_smelting;
pub use lead_smelting::lead_smelting;
mod lead_mining;
pub use lead_mining::lead_mining;
mod limestone_mining;
pub use limestone_mining::limestone_mining;
mod timed_tree_variants;
pub use timed_tree_variants::timed_tree_variants;
mod timed_trees;
pub use timed_trees::timed_trees;
mod woodcutting_axes;
pub use woodcutting_axes::woodcutting_axes;
mod ordinary_mining;
pub use ordinary_mining::ordinary_mining_rocks;
mod silver_jewellery;
pub use silver_jewellery::silver_jewellery;
mod gold_jewellery;
pub use gold_jewellery::gold_jewellery;
mod toxic_upgrades;
pub use toxic_upgrades::toxic_upgrades;
mod accursed_upgrade;
pub use accursed_upgrade::accursed_upgrade;
mod firelighter_coating;
pub use firelighter_coating::firelighter_coating;
mod glass_smelting;
pub use glass_smelting::glass_smelting;
mod sand_pits;
pub use sand_pits::sand_pits;
mod sand_containers;
pub use sand_containers::sand_containers;
mod facility_locations;
pub use facility_locations::facility_locations;
mod fibre_textiles;
pub use fibre_textiles::fibre_textiles;
mod pottery_firing;
pub use pottery_firing::pottery_firing;
mod pottery_shaping;
pub use pottery_shaping::pottery_shaping;
mod weaving_loom;
pub use weaving_loom::weaving_loom;
mod spinning_wheel;
pub use spinning_wheel::spinning_wheel;
mod soft_clay_mixing;
pub use soft_clay_mixing::soft_clay_mixing;
mod amethyst_cutting;
pub use amethyst_cutting::amethyst_cutting;
mod crafting_finishes;
pub use crafting_finishes::crafting_finishes;
mod crab_carving;
pub use crab_carving::crab_carving;
mod battlestaff_assembly;
pub use battlestaff_assembly::battlestaff_assembly;
mod sinew_crafting;
pub use sinew_crafting::sinew_crafting;
mod immediate_tool_crafting;
pub use immediate_tool_crafting::immediate_tool_crafting;
mod fallible_gems;
pub use fallible_gems::fallible_gems;
mod glassblowing;
pub use glassblowing::glassblowing;
mod ordinary_stringing;
pub use ordinary_stringing::ordinary_stringing;
mod godsword_forging;
pub use godsword_forging::godsword_forging;
mod godsword_assembly;
pub use godsword_assembly::godsword_attachments;
mod amulet_stringing;
pub use amulet_stringing::amulet_stringing;
mod snelm_carving;
pub use snelm_carving::snelm_carving;
mod gem_cutting;
pub use gem_cutting::gem_cutting;
mod citizen_thieving;
pub use citizen_thieving::citizen_pickpockets;
mod coin_pouch;
pub use coin_pouch::coin_pouches;
mod dorgesh_food;
pub use dorgesh_food::dorgesh_foods;
mod bowl_food;
pub use bowl_food::bowl_foods;
mod meal_food;
pub use meal_food::meal_foods;
mod random_food;
pub use random_food::random_foods;
mod basic_meat;
pub use basic_meat::{basic_meat_cooking, rabbit_food};
mod member_fish;
pub use member_fish::member_fish_cooking;
/// Resolve both stat requirement pairs from enriched cache parameters. Level
/// parameters are signed Kotlin Ints; do not truncate them to a byte.
pub fn equipment(
    param_symbols: &Symbols,
    defs: &Definitions,
) -> Result<gielinor_world::equipment::WearCatalog, String> {
    use gielinor_cache::definitions::Param;
    use gielinor_world::equipment::{WearCatalog, WearType};
    let mut pairs = Vec::new();
    for n in [1, 2] {
        let skill = *param_symbols
            .get(&format!("statreq{n}_skill"))
            .ok_or("Missing equipment skill parameter")? as u32;
        let level = *param_symbols
            .get(&format!("statreq{n}_level"))
            .ok_or("Missing equipment level parameter")? as u32;
        pairs.push((skill, level));
    }
    let mut catalog = WearCatalog::default();
    for (&id, ty) in &defs.items {
        let mut requirements = Vec::new();
        for &(skill, level) in &pairs {
            let Some(value) = ty.params.get(&skill) else {
                continue;
            };
            let Param::Integer(skill) = value else {
                return Err(format!("Item {id} has noninteger skill requirement"));
            };
            let skill =
                u8::try_from(*skill).map_err(|_| format!("Item {id} has invalid skill ID"))?;
            if !defs.stats.contains_key(&(skill as u32)) {
                return Err(format!("Item {id} references missing stat {skill}"));
            }
            let level = match ty.params.get(&level) {
                None => 0,
                Some(Param::Integer(level)) => *level,
                _ => return Err(format!("Item {id} has noninteger level requirement")),
            };
            requirements.push((skill, level));
        }
        let slot = |value: i8| usize::try_from(value).ok().filter(|slot| *slot < 14);
        catalog.types.insert(
            id as i32,
            WearType {
                primary: slot(ty.wear[0]),
                secondary: [slot(ty.wear[1]), slot(ty.wear[2])],
                requirements,
            },
        );
    }
    Ok(catalog)
}
/// Independently Wiki-pinned ordinary packs. Missing optional symbols are not
/// enabled; present profiles must have a valid operation and output/note pair.
pub fn item_packs(
    symbols: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<i32, gielinor_world::item_packs::Rule>, String> {
    let mut rules = BTreeMap::new();
    for (pack_name, output_name, noted) in [
        ("pack_feather", "feather", false),
        ("pack_fishing_bait", "fishing_bait", false),
        ("pack_bucket", "bucket_empty", true),
        ("pack_jug_empty", "jug_empty", true),
        ("pack_vial_empty", "vial_empty", true),
        ("pack_vial_water", "vial_water", true),
        ("pack_compost", "bucket_compost", true),
        // Wiki plant-pot packs contain filled pots; do not copy the reference
        // enum's plantpot_empty symbol without effective-definition evidence.
        ("pack_plant_pot_compost", "plantpot_compost", true),
        ("pack_sack", "sack_empty", true),
        ("pack_basket", "basket_empty", true),
        ("pack_olive_oil", "oliveoil4", true),
        ("pack_airrune", "airrune", false),
        ("pack_waterrune", "waterrune", false),
        ("pack_earthrune", "earthrune", false),
        ("pack_firerune", "firerune", false),
        ("pack_mindrune", "mindrune", false),
        ("pack_chaosrune", "chaosrune", false),
        ("pack_airrune_tzhaar", "airrune", false),
        ("pack_waterrune_tzhaar", "waterrune", false),
        ("pack_earthrune_tzhaar", "earthrune", false),
        ("pack_firerune_tzhaar", "firerune", false),
        ("pack_slayer_broad_bolt_unfinished", "slayer_broad_bolt_unfinished", false),
        ("pack_slayer_broad_arrowhead", "slayer_broad_arrowhead", false),
        ("pack_softclay", "softclay", true),
        ("pack_amylase", "amylase", false),
        ("pack_eye_newt", "eye_of_newt", true),
        ("pack_box_trap", "hunting_box_trap", false),
        ("pack_ojibway_bird_snare", "hunting_ojibway_bird_snare", false),
        ("pack_magic_imp_box", "magic_imp_box", false),
        ("pack_bone_bolt", "dttd_bone_crossbow_bolt", false),
        ("piscarilius_pack_sandworms", "piscarilius_sandworms", false),

    ] {
        let Some(&pack) = symbols.get(pack_name) else {
            continue;
        };
        let &output = symbols
            .get(output_name)
            .ok_or_else(|| format!("Missing pack output {output_name}"))?;
        let pack_type = defs
            .items
            .get(&(pack as u32))
            .ok_or_else(|| format!("Missing pack definition {pack_name}"))?;
        let output_type = defs
            .items
            .get(&(output as u32))
            .ok_or_else(|| format!("Missing pack output definition {output_name}"))?;
        if pack_type.cert.template != 0
            || pack_type.placeholder.template != 0
            || pack_type.inventory_ops[0]
                .as_deref()
                .is_none_or(|op| !op.eq_ignore_ascii_case("Open"))
            || output_type.cert.template != 0
            || output_type.placeholder.template != 0
            || output_type.stackable == noted
            || output == pack
        {
            return Err(format!("Invalid pack/output profile {pack_name}"));
        }
        if noted {
            let note_id = u32::try_from(output_type.cert.link).map_err(|_| "Invalid note ID")?;
            let note = defs
                .items
                .get(&note_id)
                .ok_or_else(|| format!("Missing pack output note {output_name}"))?;
            if note_id == 0 || note.cert.template == 0 || note.cert.link != output {
                return Err(format!("Invalid reciprocal note mapping {output_name}"));
            }
        }
        if rules
            .insert(
                pack,
                gielinor_world::item_packs::Rule {
                    output,
                    count: 100,
                    noted,
                },
            )
            .is_some()
        {
            return Err(format!("Duplicate pack binding {pack_name}"));
        }
    }
    Ok(rules)
}
/// Membership is startup metadata; keep the existing public exchange rule stable.
pub fn item_pack_members(
    packs: &BTreeMap<i32, gielinor_world::item_packs::Rule>,
    defs: &Definitions,
) -> Result<std::collections::BTreeSet<i32>, String> {
    let mut members = std::collections::BTreeSet::new();
    for &id in packs.keys() {
        let key = u32::try_from(id).map_err(|_| "Invalid registered pack ID")?;
        let ty = defs
            .items
            .get(&key)
            .ok_or("Missing registered pack membership definition")?;
        if ty.members {
            members.insert(id);
        }
    }
    Ok(members)
}
#[cfg(test)]
mod item_pack_members_tests {
    use super::*;
    #[test]
    fn pack_membership_uses_registered_pack_not_output_and_refuses_missing_or_negative_types() {
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (id, member) in [(10, true), (11, false), (20, false), (21, true)] {
            let mut item = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            item.members = member;
            defs.items.insert(id, item);
        }
        let rules: BTreeMap<_, _> = [
            (
                10,
                gielinor_world::item_packs::Rule {
                    output: 20,
                    count: 100,
                    noted: false,
                },
            ),
            (
                11,
                gielinor_world::item_packs::Rule {
                    output: 21,
                    count: 100,
                    noted: false,
                },
            ),
        ]
        .into();
        assert_eq!(item_pack_members(&rules, &defs).unwrap(), [10].into());
        defs.items.remove(&11);
        assert!(item_pack_members(&rules, &defs).is_err());
        let negative = [(
            -1,
            gielinor_world::item_packs::Rule {
                output: 20,
                count: 100,
                noted: false,
            },
        )]
        .into();
        assert!(item_pack_members(&negative, &defs).is_err());
    }
}
pub fn foods(symbols: &Symbols, defs: &Definitions) -> Result<BTreeMap<i32, Edible>, String> {
    let resolve = |name: &str| {
        symbols
            .get(name)
            .copied()
            .ok_or_else(|| format!("Missing food symbol {name}"))
    };
    let mut foods = BTreeMap::new();
    for food in food::catalog() {
        let id = resolve(food.symbol)?;
        let ty = defs
            .items
            .get(&(id as u32))
            .ok_or_else(|| format!("Missing food type {id}"))?;
        let replacement = food.replacement.map(resolve).transpose()?;
        if replacement.is_some_and(|id| !defs.items.contains_key(&(id as u32))) {
            return Err("Missing replacement type".into());
        }
        let message = food
            .message
            .unwrap_or_else(|| format!("You eat the {}.", ty.name.to_lowercase()));
        if foods
            .insert(
                id,
                Edible {
                    id,
                    heal: food.heal,
                    delay: food.delay,
                    replacement,
                    message,
                },
            )
            .is_some()
        {
            return Err(format!("Duplicate resolved food ID {id}"));
        }
    }
    Ok(foods)
}
/// Independently Wiki-specified ordinary healing profiles absent from the
/// original-reference catalog. Commit all definitions together at startup.
pub fn additional_ordinary_foods(symbols:&Symbols,defs:&Definitions,foods:&mut BTreeMap<i32,Edible>)->Result<usize,String>{
    apply_ordinary_food_profiles(symbols,defs,foods,&[
        ("anchovies","Anchovies",1),("lava_eel","Lava eel",11),
        ("onion","Onion",1),("tomato","Tomato",2),("cheese","Cheese",2),
        ("cooked_ugthanki_meat","Ugthanki meat",3),("ugthanki_kebab","Ugthanki kebab",19),
        ("locust_meat","Locust meat",3),("brut_roe","Roe",3),("brut_caviar","Caviar",5),
    ])
}
/// Prepared fruit portions, independently pinned to article/all-food Wiki rows.
pub fn prepared_fruit_foods(symbols:&Symbols,defs:&Definitions,foods:&mut BTreeMap<i32,Edible>)->Result<usize,String>{
    apply_ordinary_food_profiles(symbols,defs,foods,&[
        ("tbwt_sliced_banana","Sliced banana",2),
        ("orange_chunks","Orange chunks",2),("orange_slices","Orange slices",2),
        ("pineapple_chunks","Pineapple chunks",2),("pineapple_ring","Pineapple ring",2),
        ("lemon_chunks","Lemon chunks",2),("lemon_slices","Lemon slices",2),
        ("lime_chunks","Lime chunks",2),("lime_slices","Lime slices",2),
    ])
}
fn apply_ordinary_food_profiles(symbols:&Symbols,defs:&Definitions,foods:&mut BTreeMap<i32,Edible>,profiles:&[(&str,&str,i32)])->Result<usize,String>{
    let mut next=foods.clone();let mut count=0;
    for &(symbol,name,heal) in profiles {
        let id=*symbols.get(symbol).ok_or_else(||format!("Missing ordinary food symbol {symbol}"))?;
        let item=u32::try_from(id).ok().and_then(|id|defs.items.get(&id)).ok_or("Missing ordinary food definition")?;
        if item.name!=name || item.stackable || item.dummy==1 || item.cert.template!=0 || item.placeholder.template!=0 || item.transform.template!=0 || item.inventory_ops[0].as_deref()!=Some("Eat") || next.contains_key(&id){return Err(format!("Invalid ordinary food profile {symbol}"));}
        next.insert(id,Edible{id,heal,delay:3,replacement:None,message:format!("You eat the {}.",name.to_lowercase())});count+=1;
    }
    *foods=next;Ok(count)
}
#[cfg(test)]
mod prepared_fruit_food_tests {
    use super::*;
    #[test]
    fn prepared_fruit_foods_bind_all_profiles_and_preserve_catalog_on_late_failure(){
        let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};let mut symbols=Symbols::new();
        for (symbol,id,name) in [("tbwt_sliced_banana",3162,"Sliced banana"),("orange_chunks",2110,"Orange chunks"),("orange_slices",2112,"Orange slices"),("pineapple_chunks",2116,"Pineapple chunks"),("pineapple_ring",2118,"Pineapple ring"),("lemon_chunks",2104,"Lemon chunks"),("lemon_slices",2106,"Lemon slices"),("lime_chunks",2122,"Lime chunks"),("lime_slices",2124,"Lime slices")]{symbols.insert(symbol.into(),id);let mut item=gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();item.name=name.into();item.inventory_ops[0]=Some("Eat".into());defs.items.insert(id as u32,item);}
        let mut foods=BTreeMap::new();assert_eq!(prepared_fruit_foods(&symbols,&defs,&mut foods).unwrap(),9);
        for (&id,f) in &foods{let mut inv=gielinor_world::inventory::Inventory::new(1,gielinor_world::inventory::StackType::Normal,false);inv.restore_slot(0,Some(gielinor_world::inventory::Item{id,count:1,vars:0})).unwrap();let mut stats=gielinor_world::stats::Stats::default();stats.set_base(3,10);stats.set_current(3,9);assert!(gielinor_world::food::eat_slot(f,0,&mut inv,&mut stats,&Default::default(),3,829,|_|{}).unwrap());assert_eq!(stats.current(3),10);assert_eq!(stats.fine_xp(3),0);assert!(inv.get(0).is_none());}
        foods.clear();defs.items.get_mut(&2124).unwrap().name="Wrong fruit".into();assert!(prepared_fruit_foods(&symbols,&defs,&mut foods).is_err());assert!(foods.is_empty());
    }
}

#[cfg(test)]
mod boost_pie_food_tests{
    use super::*;
    fn fixture()->(Symbols,Symbols,Definitions){
        let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};let mut objects=Symbols::new();let mut stats=Symbols::new();
        for (symbol,id,name) in [("garden_pie",7178,"Garden pie"),("half_garden_pie",7180,"Half a garden pie"),("fish_pie",7188,"Fish pie"),("half_fish_pie",7190,"Half a fish pie"),("botanical_pie",19662,"Botanical pie"),("half_botanical_pie",19659,"Half a botanical pie"),("mushroom_pie",21690,"Mushroom pie"),("half_mushroom_pie",21687,"Half a mushroom pie"),("admiral_pie",7198,"Admiral pie"),("half_admiral_pie",7200,"Half an admiral pie"),("wild_pie",7208,"Wild pie"),("half_wild_pie",7210,"Half a wild pie"),("dragonfruit_pie",22795,"Dragonfruit pie"),("half_dragonfruit_pie",22792,"Half a dragonfruit pie"),("piedish",2313,"Pie dish")]{objects.insert(symbol.into(),id);let mut item=gielinor_cache::definitions::ItemType::decode(&[16,0]).unwrap();item.name=name.into();if symbol!="piedish"{item.inventory_ops[0]=Some("Eat".into());}defs.items.insert(id as u32,item);}
        for (name,id) in [("farming",19),("fishing",10),("herblore",15),("crafting",12),("slayer",18),("ranged",4),("fletching",9)]{stats.insert(name.into(),id);defs.stats.insert(id as u32,gielinor_cache::definitions::StatType::decode(&[0]).unwrap());}
        (objects,stats,defs)
    }
    #[test]
    fn boost_pie_foods_bind_all_bites_and_reject_missing_late_definitions_atomically(){
        let (objects,stat_symbols,defs)=fixture();let mut foods=BTreeMap::new();let binding=boost_pie_foods(&objects,&stat_symbols,&defs,&mut foods).unwrap();assert_eq!(binding.selected.len(),14);assert_eq!(binding.boosts[&7208],vec![gielinor_world::food::StatBoost{skill:18,amount:5},gielinor_world::food::StatBoost{skill:4,amount:4}]);
        for id in binding.selected.iter().copied(){let f=&foods[&id];let mut inv=gielinor_world::inventory::Inventory::new(2,gielinor_world::inventory::StackType::Normal,false);for slot in 0..2{inv.restore_slot(slot,Some(gielinor_world::inventory::Item{id,count:1,vars:0})).unwrap();}let mut stats=gielinor_world::stats::Stats::default();for skill in 0..25{stats.set_base(skill,99);stats.set_current(skill,99);}let before=stats.clone();gielinor_world::food::eat_slot_with_boosts(f,1,&mut inv,&mut stats,&Default::default(),3,829,&binding.boosts[&id],|_|{}).unwrap();assert_eq!(inv.get(1).unwrap().id,f.replacement.unwrap());assert_eq!(inv.get(0).unwrap().id,id);for effect in &binding.boosts[&id]{assert_eq!(stats.current(effect.skill),99+effect.amount as i8);}for skill in 0..25{assert_eq!(stats.fine_xp(skill),before.fine_xp(skill));}}
        for mode in 0..5{let (objects,symbols,mut defs)=fixture();match mode{0=>{defs.stats.remove(&9);},1=>{defs.items.get_mut(&22792).unwrap().cert.template=799;},2=>{defs.items.get_mut(&2313).unwrap().placeholder.template=14401;},3=>{defs.items.get_mut(&22792).unwrap().members=false;},_=>{defs.items.get_mut(&22792).unwrap().inventory_ops[0]=None;}}let mut foods=BTreeMap::new();assert!(boost_pie_foods(&objects,&symbols,&defs,&mut foods).is_err());assert!(foods.is_empty());}
    }
}

#[cfg(test)]
mod additional_ordinary_food_tests {
    use super::*;
    fn fixture()->(Symbols,Definitions){
        let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};let mut symbols=Symbols::new();
        for (symbol,id,name) in [("anchovies",319,"Anchovies"),("lava_eel",2149,"Lava eel"),("onion",1957,"Onion"),("tomato",1982,"Tomato"),("cheese",1985,"Cheese"),("cooked_ugthanki_meat",1861,"Ugthanki meat"),("ugthanki_kebab",1885,"Ugthanki kebab"),("locust_meat",9052,"Locust meat"),("brut_roe",11324,"Roe"),("brut_caviar",11326,"Caviar")]{symbols.insert(symbol.into(),id);let mut item=gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();item.name=name.into();item.inventory_ops[0]=Some("Eat".into());defs.items.insert(id as u32,item);}
        (symbols,defs)
    }
    #[test]
    fn ordinary_food_profiles_publish_together_and_reject_late_alias_or_bad_definition(){
        let (symbols,defs)=fixture();let mut foods=BTreeMap::new();assert_eq!(additional_ordinary_foods(&symbols,&defs,&mut foods).unwrap(),10);assert_eq!(foods[&11326].heal,5);
        for (&id,food) in &foods {let mut inv=gielinor_world::inventory::Inventory::new(2,gielinor_world::inventory::StackType::Normal,false);for slot in 0..2{inv.restore_slot(slot,Some(gielinor_world::inventory::Item{id,count:1,vars:0})).unwrap();}let mut stats=gielinor_world::stats::Stats::default();stats.set_base(3,99);stats.set_current(3,90);let xp=stats.fine_xp(3);assert!(gielinor_world::food::eat_slot(food,1,&mut inv,&mut stats,&Default::default(),3,829,|_|{}).unwrap());assert!(inv.get(1).is_none());assert_eq!(inv.get(0).unwrap().id,id);assert_eq!(stats.current(3),(90+food.heal).min(99) as i8);assert_eq!(stats.fine_xp(3),xp);}
        for mode in 0..7 {let (symbols,mut defs)=fixture();let mut foods=BTreeMap::new();let item=defs.items.get_mut(&11326).unwrap();match mode{0=>item.stackable=true,1=>item.cert.template=799,2=>item.placeholder.template=14401,3=>item.transform.template=1,4=>item.inventory_ops[0]=None,5=>item.name="Not caviar".into(),_=>{foods.insert(11326,Edible{id:11326,heal:99,delay:3,replacement:None,message:"Existing".into()});}}let before=foods.len();assert!(additional_ordinary_foods(&symbols,&defs,&mut foods).is_err());assert_eq!(foods.len(),before);if mode==6{assert_eq!(foods[&11326].heal,99);}}
    }
}

pub struct HealingFoodBindings{pub selected:std::collections::BTreeSet<i32>,pub healing:BTreeMap<i32,gielinor_world::food::Healing>}
/// Source/Wiki table rules are evaluated against live base HP at consumption.
pub fn dynamic_foods(objects:&Symbols,defs:&Definitions,foods:&mut BTreeMap<i32,Edible>)->Result<HealingFoodBindings,String>{
    let mut next=foods.clone();apply_ordinary_food_profiles(objects,defs,&mut next,&[("strawberry","Strawberry",0),("watermelon_slice","Watermelon slice",0),("sweetcorn_cooked","Cooked sweetcorn",0),("bowl_sweetcorn","Sweetcorn",0)])?;
    let bowl=*objects.get("bowl_empty").ok_or("Missing dynamic-food bowl")?;let item=u32::try_from(bowl).ok().and_then(|id|defs.items.get(&id)).ok_or("Missing dynamic-food bowl definition")?;if item.name!="Bowl" || item.stackable || item.dummy==1 || item.cert.template!=0 || item.placeholder.template!=0 || item.transform.template!=0{return Err("Invalid dynamic-food bowl".into());}
    let mut selected=std::collections::BTreeSet::new();let mut healing=BTreeMap::new();
    for (symbol,percent,constant,round_up) in [("strawberry",6,0,true),("watermelon_slice",5,1,false),("sweetcorn_cooked",10,1,false),("bowl_sweetcorn",10,1,false)]{let id=objects[symbol];if id==bowl || !defs.items[&(id as u32)].members{return Err("Invalid member dynamic-food profile".into());}selected.insert(id);healing.insert(id,gielinor_world::food::Healing{percent,constant,round_up});}
    next.get_mut(&objects["bowl_sweetcorn"]).unwrap().replacement=Some(bowl);next.get_mut(&objects["watermelon_slice"]).unwrap().message="You eat the slice of watermelon.".into();
    *foods=next;Ok(HealingFoodBindings{selected,healing})
}
#[cfg(test)]
mod dynamic_food_tests{
    use super::*;
    #[test]
    fn dynamic_foods_bind_live_hp_rules_and_retained_bowl_without_partial_publish(){
        let mut objects=Symbols::new();let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
        for (symbol,id,name) in [("strawberry",5504,"Strawberry"),("watermelon_slice",5984,"Watermelon slice"),("sweetcorn_cooked",5988,"Cooked sweetcorn"),("bowl_sweetcorn",7088,"Sweetcorn"),("bowl_empty",1923,"Bowl")]{objects.insert(symbol.into(),id);let mut item=gielinor_cache::definitions::ItemType::decode(&[16,0]).unwrap();item.name=name.into();if symbol!="bowl_empty"{item.inventory_ops[0]=Some("Eat".into());}defs.items.insert(id as u32,item);}
        let mut foods=BTreeMap::new();let binding=dynamic_foods(&objects,&defs,&mut foods).unwrap();assert_eq!(binding.selected.len(),4);assert_eq!(binding.healing[&5504].amount(50),Ok(3));assert_eq!(binding.healing[&5984].amount(20),Ok(2));assert_eq!(binding.healing[&7088].amount(99),Ok(10));assert_eq!(foods[&7088].replacement,Some(1923));assert_eq!(foods[&5984].message,"You eat the slice of watermelon.");
        foods.clear();defs.items.get_mut(&1923).unwrap().cert.template=799;assert!(dynamic_foods(&objects,&defs,&mut foods).is_err());assert!(foods.is_empty());
    }
}

pub struct BoostFoodBindings {
    pub selected:std::collections::BTreeSet<i32>,
    pub boosts:BTreeMap<i32,Vec<gielinor_world::food::StatBoost>>,
    pub energy:BTreeMap<i32,i32>,
}
/// Summer pie and papaya use shared food/stat preparation plus run energy.
pub fn energy_foods(objects:&Symbols,stats:&Symbols,defs:&Definitions,foods:&mut BTreeMap<i32,Edible>)->Result<BoostFoodBindings,String>{
    let mut next=foods.clone();apply_ordinary_food_profiles(objects,defs,&mut next,&[("summer_pie","Summer pie",11),("half_summer_pie","Half a summer pie",11),("papaya","Papaya fruit",8)])?;
    let whole=objects["summer_pie"];let half=objects["half_summer_pie"];let papaya=objects["papaya"];let dish=*objects.get("piedish").ok_or("Missing energy-food dish")?;
    let item=u32::try_from(dish).ok().and_then(|id|defs.items.get(&id)).ok_or("Missing energy-food dish definition")?;
    if [whole,half,papaya].contains(&dish) || item.stackable || item.dummy==1 || item.cert.template!=0 || item.placeholder.template!=0 || item.transform.template!=0{return Err("Invalid energy-food dish".into());}
    let agility=u8::try_from(*stats.get("agility").ok_or("Missing energy-food Agility symbol")?).map_err(|_|"Invalid energy-food Agility ID")?;if !defs.stats.contains_key(&(agility as u32)){return Err("Missing energy-food Agility definition".into());}
    for id in [whole,half,papaya]{if !defs.items[&(id as u32)].members{return Err("Expected member energy-food definition".into());}}
    next.get_mut(&whole).unwrap().replacement=Some(half);next.get_mut(&half).unwrap().replacement=Some(dish);for id in [whole,half]{next.get_mut(&id).unwrap().delay=1;}
    let boosts=[(whole,vec![gielinor_world::food::StatBoost{skill:agility,amount:5}]),(half,vec![gielinor_world::food::StatBoost{skill:agility,amount:5}]),(papaya,Vec::new())].into();
    *foods=next;Ok(BoostFoodBindings{selected:[whole,half,papaya].into(),boosts,energy:[(whole,10),(half,10),(papaya,5)].into()})
}
#[cfg(test)]
mod energy_food_tests{
    use super::*;
    #[test]
    fn energy_foods_bind_summer_halves_and_papaya_without_partial_metadata_publish(){
        let mut objects=Symbols::new();let stats:Symbols=[("agility".into(),16)].into();let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};defs.stats.insert(16,gielinor_cache::definitions::StatType::decode(&[0]).unwrap());
        for (symbol,id,name) in [("summer_pie",7218,"Summer pie"),("half_summer_pie",7220,"Half a summer pie"),("papaya",5972,"Papaya fruit"),("piedish",2313,"Pie dish")]{objects.insert(symbol.into(),id);let mut item=gielinor_cache::definitions::ItemType::decode(&[16,0]).unwrap();item.name=name.into();if symbol!="piedish"{item.inventory_ops[0]=Some("Eat".into());}defs.items.insert(id as u32,item);}
        let mut foods=BTreeMap::new();let binding=energy_foods(&objects,&stats,&defs,&mut foods).unwrap();assert_eq!(binding.selected.len(),3);assert_eq!(binding.energy,[(7218,10),(7220,10),(5972,5)].into());assert_eq!((foods[&7218].replacement,foods[&7220].replacement,foods[&5972].replacement),(Some(7220),Some(2313),None));assert_eq!((foods[&7218].delay,foods[&5972].heal,foods[&5972].delay),(1,8,3));assert_eq!(binding.boosts[&7218],vec![gielinor_world::food::StatBoost{skill:16,amount:5}]);
        foods.clear();defs.stats.remove(&16);assert!(energy_foods(&objects,&stats,&defs,&mut foods).is_err());assert!(foods.is_empty());
    }
}

/// Ordinary seven-family pie healing/stage/boost data, independently Wiki-pinned.
pub fn boost_pie_foods(objects:&Symbols,stats:&Symbols,defs:&Definitions,foods:&mut BTreeMap<i32,Edible>)->Result<BoostFoodBindings,String>{
    let ordinary=|symbol:&str|->Result<i32,String>{let id=*objects.get(symbol).ok_or("Missing boost-pie object")?;let item=u32::try_from(id).ok().and_then(|id|defs.items.get(&id)).ok_or("Missing boost-pie definition")?;if item.stackable || item.dummy==1 || item.cert.template!=0 || item.placeholder.template!=0 || item.transform.template!=0{return Err("Invalid ordinary boost-pie object".into());}Ok(id)};
    let dish=ordinary("piedish")?;let mut next=foods.clone();let mut selected=std::collections::BTreeSet::new();let mut boosts=BTreeMap::new();
    for (whole,half,label,heal,policy) in [
        ("garden_pie","half_garden_pie","Garden pie",6,&[("farming",3)][..]),
        ("fish_pie","half_fish_pie","Fish pie",6,&[("fishing",3)][..]),
        ("botanical_pie","half_botanical_pie","Botanical pie",7,&[("herblore",4)][..]),
        ("mushroom_pie","half_mushroom_pie","Mushroom pie",8,&[("crafting",4)][..]),
        ("admiral_pie","half_admiral_pie","Admiral pie",8,&[("fishing",5)][..]),
        ("wild_pie","half_wild_pie","Wild pie",11,&[("slayer",5),("ranged",4)][..]),
        ("dragonfruit_pie","half_dragonfruit_pie","Dragonfruit pie",10,&[("fletching",4)][..]),
    ] {
        let mut effects=Vec::new();for &(name,amount) in policy{let id=*stats.get(name).ok_or("Missing food boost stat symbol")?;let id=u8::try_from(id).map_err(|_|"Invalid food boost stat ID")?;if !defs.stats.contains_key(&(id as u32)){return Err("Missing food boost stat definition".into());}effects.push(gielinor_world::food::StatBoost{skill:id,amount});}
        let whole=ordinary(whole)?;let half=ordinary(half)?;if whole==half || whole==dish || half==dish{return Err("Aliased boost pie stages".into());}
        for (id,name,replacement) in [(whole,label.to_string(),half),(half,format!("Half {} {}",if label=="Admiral pie"{"an"}else{"a"},label.to_lowercase()),dish)]{
            let item=&defs.items[&(id as u32)];if item.name!=name || !item.members || item.inventory_ops[0].as_deref()!=Some("Eat") || next.contains_key(&id) || !selected.insert(id){return Err("Invalid boost pie stage profile".into());}
            next.insert(id,Edible{id,heal,delay:1,replacement:Some(replacement),message:format!("You eat the {}.",name.to_lowercase())});boosts.insert(id,effects.clone());
        }
    }
    *foods=next;Ok(BoostFoodBindings{selected,boosts,energy:BTreeMap::new()})
}

/// GR-24's bounded cake/plain-pizza correction. Other foods retain the reference
/// profile; selected IDs must be unnoted, nonstackable, ordinary Eat definitions.
pub fn wiki_multipart_foods(
    symbols: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<std::collections::BTreeSet<i32>, String> {
    let rows = [
        ("cake", Some("partial_cake"), 4),
        ("partial_cake", Some("cake_slice"), 4),
        ("cake_slice", None, 4),
        ("plain_pizza", Some("half_plain_pizza"), 7),
        ("half_plain_pizza", None, 7),
    ];
    apply_multipart_foods(symbols,defs,foods,&rows)
}

/// Independently Wiki-pinned remaining ordinary multipart profiles. Existing
/// GR-24 cake/plain-pizza registration remains a separate caller.
pub fn additional_multipart_foods(symbols:&Symbols,defs:&Definitions,foods:&mut BTreeMap<i32,Edible>)->Result<std::collections::BTreeSet<i32>,String>{
    apply_multipart_foods(symbols,defs,foods,&[
        ("chocolate_cake",Some("partial_chocolate_cake"),5),
        ("partial_chocolate_cake",Some("chocolate_slice"),5),
        ("chocolate_slice",None,5),
        ("meat_pizza",Some("half_meat_pizza"),8),("half_meat_pizza",None,8),
        ("anchovie_pizza",Some("half_anchovie_pizza"),9),("half_anchovie_pizza",None,9),
        ("pineapple_pizza",Some("half_pineapple_pizza"),11),("half_pineapple_pizza",None,11),
        ("redberry_pie",Some("half_a_redberry_pie"),5),("half_a_redberry_pie",Some("piedish"),5),
        ("meat_pie",Some("half_a_meat_pie"),6),("half_a_meat_pie",Some("piedish"),6),
        ("apple_pie",Some("half_an_apple_pie"),7),("half_an_apple_pie",Some("piedish"),7),
    ])
}
fn apply_multipart_foods(symbols:&Symbols,defs:&Definitions,foods:&mut BTreeMap<i32,Edible>,rows:&[(&str,Option<&str>,i32)])->Result<std::collections::BTreeSet<i32>,String>{
    let mut next = foods.clone();
    let mut selected = std::collections::BTreeSet::new();

    for &(symbol, replacement, heal) in rows {
        let id = *symbols.get(symbol).ok_or("Missing multipart food symbol")?;
        let item = u32::try_from(id)
            .ok()
            .and_then(|id| defs.items.get(&id))
            .ok_or("Missing multipart food definition")?;
        if item.stackable
            || item.transform.template != 0
            || item.dummy == 1
            || item.cert.template != 0
            || item.placeholder.template != 0
            || item.inventory_ops[0].as_deref() != Some("Eat")
            || !selected.insert(id)
        {
            return Err("Invalid multipart food definition".into());
        }
        let food = next
            .get_mut(&id)
            .ok_or("Multipart food handler is absent")?;
        food.heal = heal;
        food.replacement = replacement
            .map(|name| {
                symbols
                    .get(name)
                    .copied()
                    .ok_or("Missing multipart remainder symbol")
            })
            .transpose()?;
    }
    for food in next.values().filter(|f|selected.contains(&f.id)) {
        if let Some(id)=food.replacement {
            let item=u32::try_from(id).ok().and_then(|id|defs.items.get(&id)).ok_or("Missing multipart output definition")?;
            if id==food.id || item.stackable || item.dummy==1 || item.cert.template!=0 || item.placeholder.template!=0 || item.transform.template!=0 {return Err("Invalid multipart output definition".into());}
        }
    }
    *foods = next;
    Ok(selected)
}

#[cfg(test)]
mod additional_multipart_tests {
    use super::*;
    fn fixture()->(Symbols,Definitions,BTreeMap<i32,Edible>){
        let rows=[("chocolate_cake",1897),("partial_chocolate_cake",1899),("chocolate_slice",1901),("meat_pizza",2293),("half_meat_pizza",2295),("anchovie_pizza",2297),("half_anchovie_pizza",2299),("pineapple_pizza",2301),("half_pineapple_pizza",2303),("redberry_pie",2325),("half_a_redberry_pie",2333),("meat_pie",2327),("half_a_meat_pie",2331),("apple_pie",2323),("half_an_apple_pie",2335),("piedish",2313)];
        let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};let mut symbols=Symbols::new();let mut foods=BTreeMap::new();
        for (name,id) in rows{symbols.insert(name.into(),id);let mut item=gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();if name!="piedish"{item.inventory_ops[0]=Some("Eat".into());foods.insert(id,Edible{id,heal:1,delay:3,replacement:None,message:"Fixture".into()});}defs.items.insert(id as u32,item);}
        (symbols,defs,foods)
    }
    #[test]
    fn additional_food_stages_bind_wiki_healing_and_last_pie_dishes_atomically(){
        let (symbols,defs,mut foods)=fixture();assert_eq!(additional_multipart_foods(&symbols,&defs,&mut foods).unwrap().len(),15);
        for (chain,heal) in [(vec![1897,1899,1901],5),(vec![2293,2295],8),(vec![2297,2299],9),(vec![2301,2303],11),(vec![2325,2333,2313],5),(vec![2327,2331,2313],6),(vec![2323,2335,2313],7)]{
            let mut inv=gielinor_world::inventory::Inventory::new(2,gielinor_world::inventory::StackType::Normal,false);for slot in 0..2{inv.restore_slot(slot,Some(gielinor_world::inventory::Item{id:chain[0],count:1,vars:0})).unwrap();}let mut stats=gielinor_world::stats::Stats::default();stats.set_base(3,99);stats.set_current(3,1);let mut bites=0;
            for (i,id) in chain.iter().copied().filter(|id|*id!=2313).enumerate(){let f=&foods[&id];assert_eq!(f.heal,heal);assert_eq!(f.replacement,chain.get(i+1).copied());assert!(gielinor_world::food::eat_slot(f,1,&mut inv,&mut stats,&Default::default(),3,829,|_|{}).unwrap());bites+=1;assert_eq!(inv.get(1).map(|i|i.id),f.replacement);assert_eq!(inv.get(0).unwrap().id,chain[0]);assert_eq!(stats.fine_xp(3),0);}
            assert_eq!(stats.current(3),1+heal as i8*bites);
        }
        for mode in 0..5{let (symbols,mut defs,mut foods)=fixture();match mode{0=>{defs.items.remove(&2313);},1=>{defs.items.get_mut(&2313).unwrap().cert.template=799;},2=>{defs.items.get_mut(&2335).unwrap().inventory_ops[0]=None;},3=>{defs.items.get_mut(&1901).unwrap().stackable=true;},_=>{defs.items.get_mut(&2313).unwrap().transform.template=1;}}assert!(additional_multipart_foods(&symbols,&defs,&mut foods).is_err());assert!(foods.values().all(|f|f.heal==1 && f.replacement.is_none()));}
    }
}

#[cfg(test)]
mod multipart_tests {
    use super::*;

    #[test]
    fn selected_food_binding_validates_every_stage_before_changing_catalog() {
        let symbols: Symbols = [
            ("cake", 1891),
            ("partial_cake", 1893),
            ("cake_slice", 1895),
            ("plain_pizza", 2289),
            ("half_plain_pizza", 2291),
        ]
        .into_iter()
        .map(|(name, id)| (name.into(), id))
        .collect();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut foods = BTreeMap::new();
        for &id in symbols.values() {
            let mut item = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            item.inventory_ops[0] = Some("Eat".into());
            defs.items.insert(id as u32, item);
            foods.insert(
                id,
                Edible {
                    id,
                    heal: 1,
                    delay: 3,
                    replacement: None,
                    message: "Fixture".into(),
                },
            );
        }
        defs.items.get_mut(&1895).unwrap().stackable = true;
        assert!(wiki_multipart_foods(&symbols, &defs, &mut foods).is_err());
        assert!(foods
            .values()
            .all(|food| food.heal == 1 && food.replacement.is_none()));
        defs.items.get_mut(&1895).unwrap().stackable = false;
        assert_eq!(
            wiki_multipart_foods(&symbols, &defs, &mut foods)
                .unwrap()
                .len(),
            5
        );
        for (id, replacement, heal) in [
            (1891, Some(1893), 4),
            (1893, Some(1895), 4),
            (1895, None, 4),
            (2289, Some(2291), 7),
            (2291, None, 7),
        ] {
            assert_eq!(
                (foods[&id].replacement, foods[&id].heal),
                (replacement, heal)
            );
        }
    }
}
pub fn banking(
    varbit_symbols: &Symbols,
    param_symbols: &Symbols,
    defs: &Definitions,
) -> Result<gielinor_world::bank_actions::Config, String> {
    use gielinor_world::bank_actions::{Binding, Config};
    let binding = |name: &str| -> Result<Binding, String> {
        let id = *varbit_symbols
            .get(name)
            .ok_or_else(|| format!("Missing bank symbol {name}"))? as u32;
        let bit = *defs
            .varbits
            .get(&id)
            .ok_or_else(|| format!("Missing bank varbit {id}"))?;
        let base = *defs
            .varps
            .get(&(bit.base as u32))
            .ok_or_else(|| format!("Missing bank base varp {}", bit.base))?;
        bit.extract(0).map_err(|e| format!("{e:?}"))?;
        Ok(Binding { base, bit })
    };
    let tabs: Vec<_> = (1..=9)
        .map(|i| binding(&format!("bank_tab_{i}")))
        .chain(std::iter::once(binding("bank_tab_main")))
        .collect::<Result<_, _>>()?;
    let no_bank = *param_symbols
        .get("no_bank")
        .ok_or("Missing no_bank parameter")? as u32;
    let mut restricted = std::collections::HashSet::new();
    for (&id, item) in &defs.items {
        match item.params.get(&no_bank) {
            Some(gielinor_cache::definitions::Param::Integer(value)) if *value != 0 => {
                restricted.insert(id as i32);
            }
            Some(gielinor_cache::definitions::Param::String(_)) => {
                return Err("Noninteger no_bank parameter".into())
            }
            _ => (),
        }
    }
    Ok(Config {
        sidebar: None,
        tabs: tabs.try_into().unwrap(),
        selected: binding("bank_currenttab")?,
        capacity: binding("bank_capacity")?,
        notes: binding("bank_withdrawnotes")?,
        placeholders: binding("bank_leaveplaceholders")?,
        quantity: binding("bank_quantity_type")?,
        last_quantity: binding("bank_requestedquantity")?,
        restricted,
    })
}
/// BankSettingsScript controls; identifiers come from the same cache symbols as Kotlin.
pub fn bank_sidebar(
    param_symbols: &Symbols,
    varp_symbols: &Symbols,
    content_symbols: &Symbols,
    parameter_types: &BTreeMap<u32, gielinor_cache::parameters::ParameterType>,
    defs: &Definitions,
) -> Result<gielinor_world::bank_sidebar::Sidebar, String> {
    use gielinor_cache::definitions::Param;
    use gielinor_world::bank_sidebar::{ItemOptions, Sidebar, Special};
    let param = |name: &str| {
        param_symbols
            .get(name)
            .map(|id| *id as u32)
            .ok_or_else(|| format!("Missing parameter {name}"))
    };
    let bit = param("bankside_extraop_conditional_bit")?;
    let condition = param("bankside_extraop_conditional_varbit")?;
    let flip = param("bankside_extraop_conditional_flip")?;
    let text = param("bankside_extraop")?;
    let wear_index = param("wear_op_index")?;
    let food = *content_symbols
        .get("food")
        .ok_or("Missing food content group")?;
    let potion = *content_symbols
        .get("potion")
        .ok_or("Missing potion content group")?;
    let outputs: Vec<_> = ["if1", "if2", "if3"]
        .into_iter()
        .map(|name| {
            let id = *varp_symbols
                .get(name)
                .ok_or_else(|| format!("Missing varp {name}"))? as u32;
            defs.varps
                .get(&id)
                .copied()
                .ok_or_else(|| format!("Missing varp {id}"))
        })
        .collect::<Result<_, _>>()?;
    let mut items = BTreeMap::new();
    for (&id, item) in &defs.items {
        let int = |key| -> Result<Option<i32>, String> {
            match item.params.get(&key) {
                Some(Param::Integer(value)) => Ok(Some(*value)),
                Some(_) => Err(format!("Noninteger parameter {key} on {id}")),
                None => Ok(parameter_types.get(&key).and_then(|p| p.default_int)),
            }
        };
        let special = if let Some(bit) = int(bit)? {
            let varbit = int(condition)?
                .ok_or_else(|| format!("Missing conditional bank varbit on {id}"))?
                as u32;
            Some(Special {
                bit,
                condition: *defs
                    .varbits
                    .get(&varbit)
                    .ok_or_else(|| format!("Unknown conditional varbit {varbit}"))?,
                flip: int(flip)?.unwrap_or(0) != 0,
            })
        } else {
            None
        };
        let extra_text = match item.params.get(&text) {
            Some(Param::String(value)) => value.as_str(),
            Some(_) => return Err(format!("Nonstring bank operation on {id}")),
            None => parameter_types
                .get(&text)
                .and_then(|p| p.default_string.as_deref())
                .unwrap_or(""),
        };
        // Kotlin Char.isWhitespace includes separators 1C..1F and excludes NEL.
        let blank = |text: &str| {
            text.chars()
                .all(|c| c.is_whitespace() && c != '\u{85}' || ('\u{1c}'..='\u{1f}').contains(&c))
        };
        let wear = if item.wear[0] != -1 {
            let index = int(wear_index)?.ok_or("Missing wear operation index default")?;
            index
                .checked_sub(1)
                .and_then(|i| usize::try_from(i).ok())
                .and_then(|i| item.inventory_ops.get(i))
                .and_then(Option::as_deref)
                .is_some_and(|text| !blank(text))
        } else {
            false
        };
        items.insert(
            id as i32,
            ItemOptions {
                special,
                custom_text: !blank(extra_text),
                wear,
                consumable: item.content_group == food || item.content_group == potion,
            },
        );
    }
    Ok(Sidebar {
        outputs: outputs.try_into().unwrap(),
        items,
    })
}

/// BankSettingsScript controls; identifiers come from the same cache symbols as Kotlin.
pub fn bank_controls(
    components: &gielinor_cache::symbols::Symbols,
    bits: &gielinor_cache::symbols::Symbols,
    defs: &Definitions,
) -> Result<std::collections::BTreeMap<u32, gielinor_world::bank_actions::Control>, String> {
    use gielinor_world::bank_actions::{Binding, Control};
    let binding = |name: &str| -> Result<Binding, String> {
        let id = *bits
            .get(name)
            .ok_or_else(|| format!("Missing bank symbol {name}"))? as u32;
        let bit = *defs
            .varbits
            .get(&id)
            .ok_or_else(|| format!("Missing bank varbit {id}"))?;
        let base = *defs
            .varps
            .get(&(bit.base as u32))
            .ok_or("Missing bank base varp")?;
        bit.extract(0).map_err(|e| format!("{e:?}"))?;
        Ok(Binding { base, bit })
    };
    let mut result = std::collections::BTreeMap::new();
    let mut insert = |name: &str, control| -> Result<(), String> {
        let name = format!("bankmain:{name}");
        let id = *components
            .get(&name)
            .ok_or_else(|| format!("Missing component {name}"))? as u32;
        if result.insert(id, control).is_some() {
            return Err("Duplicate bank control component".into());
        }
        Ok(())
    };
    for (component, bit, value) in [
        ("swap", "bank_insertmode", 0),
        ("insert", "bank_insertmode", 1),
        ("item", "bank_withdrawnotes", 0),
        ("note", "bank_withdrawnotes", 1),
        ("quantity1", "bank_quantity_type", 0),
        ("quantity5", "bank_quantity_type", 1),
        ("quantity10", "bank_quantity_type", 2),
        ("quantityall", "bank_quantity_type", 4),
        ("bank_filler_1", "bank_fillermode", 1),
        ("bank_filler_10", "bank_fillermode", 2),
        ("bank_filler_50", "bank_fillermode", 3),
        ("bank_filler_x", "bank_fillermode", 4),
        ("bank_filler_all", "bank_fillermode", 0),
    ] {
        insert(
            component,
            Control::Set {
                binding: binding(bit)?,
                value,
            },
        )?;
    }
    for (component, bit) in [
        ("placeholder", "bank_leaveplaceholders"),
        ("incinerator_toggle", "bank_showincinerator"),
        ("banktut_toggle", "bank_hidebanktut"),
        ("sideops_toggle", "bank_hidesideops"),
        ("depositinv_toggle", "bank_hidedepositinv"),
        ("depositworn_toggle", "bank_hidedepositworn"),
    ] {
        insert(
            component,
            Control::Toggle {
                binding: binding(bit)?,
                refresh_side: component == "sideops_toggle",
            },
        )?;
    }
    insert(
        "dropdown_content",
        Control::TabDisplay(binding("bank_tab_display")?),
    )?;
    insert("quantityx", Control::QuantityX)?;
    insert("tabs", Control::Tabs)?;
    insert("depositinv", Control::DepositAll { worn: false })?;
    insert("depositworn", Control::DepositAll { worn: true })?;
    Ok(result)
}

pub fn equipment_requirement_messages(
    defs: &Definitions,
    item: i32,
    missing: &[(u8, i32)],
    message_params: [u32; 2],
) -> Result<Vec<String>, String> {
    use gielinor_cache::definitions::Param;
    let ty = defs
        .items
        .get(&(item as u32))
        .ok_or("Unknown equipment item")?;
    let text = |id| match ty.params.get(&id) {
        None => Ok(None),
        Some(Param::String(value)) => Ok(Some(value.clone())),
        _ => Err("Nonstring equipment failure message".to_owned()),
    };
    if missing.is_empty() || missing.len() > 2 {
        return Err("Invalid equipment requirement count".into());
    }
    let first = text(message_params[0])?
        .unwrap_or_else(|| "You are not a high enough level to use this item.".into());
    let mut second = text(message_params[1])?.unwrap_or_else(|| {
        if missing.len() == 1 {
            "You need to have {skill1} level of {level1}.".into()
        } else {
            "You need to have {skill1} level of {level1} and {skill2} level of {level2}.".into()
        }
    });
    for (i, &(skill, level)) in missing.iter().enumerate() {
        let name = &defs
            .stats
            .get(&(skill as u32))
            .ok_or("Unknown requirement stat")?
            .name;
        let first = name
            .chars()
            .next()
            .ok_or("Empty requirement stat name")?
            .to_ascii_lowercase();
        let article = if "aeiou".contains(first) { "an" } else { "a" };
        second = second
            .replace(&format!("{{skill{}}}", i + 1), &format!("{article} {name}"))
            .replace(&format!("{{level{}}}", i + 1), &level.to_string());
    }
    Ok(vec![first, second])
}

#[cfg(test)]
mod pack_binding_tests {
    use super::*;
    #[test]
    fn additional_noted_packs_select_filled_pots_and_reciprocal_notes() {
        let mut symbols = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        symbols.insert("plantpot_empty".into(), 999);
        for (index, (pack_name, output_name)) in [
            ("pack_plant_pot_compost", "plantpot_compost"),
            ("pack_sack", "sack_empty"),
            ("pack_basket", "basket_empty"),
            ("pack_olive_oil", "oliveoil4"),
        ]
        .into_iter()
        .enumerate()
        {
            let pack = 100 + index as i32;
            let output = 200 + index as i32;
            let note = 300 + index as i32;
            symbols.insert(pack_name.into(), pack);
            symbols.insert(output_name.into(), output);
            let mut pack_type = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            pack_type.inventory_ops[0] = Some("Open".into());
            let mut output_type = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            output_type.cert.link = note;
            let mut note_type = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            note_type.cert.template = 799;
            note_type.cert.link = output;
            defs.items.insert(pack as u32, pack_type);
            defs.items.insert(output as u32, output_type);
            defs.items.insert(note as u32, note_type);
        }
        let rules = item_packs(&symbols, &defs).unwrap();
        assert_eq!(rules.len(), 4);
        for index in 0..4 {
            let rule = rules[&(100 + index)];
            assert_eq!(
                (rule.output, rule.count, rule.noted),
                (200 + index, 100, true)
            );
        }
        assert_ne!(rules[&100].output, symbols["plantpot_empty"]);
        defs.items.get_mut(&300).unwrap().cert.link = 999;
        assert!(item_packs(&symbols, &defs).is_err());
    }
    #[test]
    fn rune_profiles_resolve_symbols_and_reject_malformed_outputs() {
        let mut symbols = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        // Synthetic identities ensure registration uses Names, not revision241 IDs.
        for (index, rune) in ["air", "water", "earth", "fire", "mind", "chaos"]
            .into_iter()
            .enumerate()
        {
            let pack = 10000 + index as i32;
            let output = 20000 + index as i32;
            symbols.insert(format!("pack_{rune}rune"), pack);
            symbols.insert(format!("{rune}rune"), output);
            let mut pack_type = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            pack_type.inventory_ops[0] = Some("Open".into());
            let mut output_type = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            output_type.stackable = true;
            defs.items.insert(pack as u32, pack_type);
            defs.items.insert(output as u32, output_type);
        }
        let rules = item_packs(&symbols, &defs).unwrap();
        assert_eq!(rules.len(), 6);
        for index in 0..6 {
            let rule = rules[&(10000 + index)];
            assert_eq!(
                (rule.output, rule.count, rule.noted),
                (20000 + index, 100, false)
            );
        }
        defs.items.get_mut(&20005).unwrap().placeholder.template = 1;
        assert!(item_packs(&symbols, &defs).is_err());
        defs.items.get_mut(&20005).unwrap().placeholder.template = 0;
        symbols.remove("chaosrune");
        assert!(item_packs(&symbols, &defs).is_err());
    }
    #[test]
    fn pack_registration_requires_open_operation_and_real_stackable_output() {
        let symbols: Symbols = [("pack_feather".into(), 11881), ("feather".into(), 314)]
            .into_iter()
            .collect();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut pack = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
        pack.inventory_ops[0] = Some("Open".into());
        let mut feather = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
        feather.stackable = true;
        defs.items.insert(11881, pack);
        defs.items.insert(314, feather);
        assert_eq!(item_packs(&symbols, &defs).unwrap()[&11881].count, 100);
        defs.items.get_mut(&314).unwrap().stackable = false;
        assert!(item_packs(&symbols, &defs).is_err());
        defs.items.get_mut(&314).unwrap().stackable = true;
        defs.items.get_mut(&11881).unwrap().inventory_ops[0] = None;
        assert!(item_packs(&symbols, &defs).is_err());
        assert!(item_packs(&Symbols::new(), &defs).unwrap().is_empty());
        let symbols: Symbols = [("pack_bucket".into(), 22660), ("bucket_empty".into(), 1925)]
            .into_iter()
            .collect();
        let mut pack = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
        pack.inventory_ops[0] = Some("Open".into());
        defs.items.insert(22660, pack);
        let mut bucket = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
        bucket.cert.link = 1926;
        defs.items.insert(1925, bucket);
        assert!(item_packs(&symbols, &defs).is_err());
        let mut note = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
        note.cert.template = 799;
        note.cert.link = 1925;
        defs.items.insert(1926, note);
        assert!(item_packs(&symbols, &defs).unwrap()[&22660].noted);
        defs.items.get_mut(&1926).unwrap().cert.link = 1935;
        assert!(item_packs(&symbols, &defs).is_err());
    }
}

/// Wiki-pinned assembly and topping awards; target Names and atomic mutations.
/// Manual Admiral pie stages use existing assembly with an explicit first clock.
pub fn admiral_pie_assembly(symbols:&Symbols,defs:&Definitions,skill:u8)->Result<Vec<(gielinor_world::assembly::Recipe,u8)>,String>{
    let mut ids=BTreeMap::new();
    for (symbol,name) in [("pie_shell","Pie shell"),("salmon","Salmon"),("tuna","Tuna"),("potato","Potato"),("unfinished_admiral_pie_1","Part admiral pie"),("unfinished_admiral_pie_2","Part admiral pie"),("uncooked_admiral_pie","Raw admiral pie")]{
        let id=*symbols.get(symbol).ok_or("Missing Admiral ingredient symbol")?;let item=u32::try_from(id).ok().and_then(|id|defs.items.get(&id)).ok_or("Missing Admiral ingredient definition")?;
        if item.name!=name || item.stackable || item.dummy==1 || item.cert.template!=0 || item.placeholder.template!=0 || item.transform.template!=0 || ids.values().any(|v|*v==id){return Err("Invalid Admiral ingredient definition".into());}ids.insert(symbol,id);
    }
    Ok([("pie_shell","salmon","unfinished_admiral_pie_1",1),("unfinished_admiral_pie_1","tuna","unfinished_admiral_pie_2",2),("unfinished_admiral_pie_2","potato","uncooked_admiral_pie",2)].into_iter().map(|(first,second,output,ticks)|(gielinor_world::assembly::Recipe{first:ids[first],second:ids[second],output:ids[output],skill,level:70,base_xp:0,members:true},ticks)).collect())
}
#[cfg(test)]
mod admiral_assembly_tests{
    use super::*;
    #[test]
    fn admiral_assembly_binds_three_manual_stages_without_baking_xp(){
        let mut symbols=Symbols::new();let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
        for (symbol,id,name) in [("pie_shell",2315,"Pie shell"),("salmon",329,"Salmon"),("tuna",361,"Tuna"),("potato",1942,"Potato"),("unfinished_admiral_pie_1",7192,"Part admiral pie"),("unfinished_admiral_pie_2",7194,"Part admiral pie"),("uncooked_admiral_pie",7196,"Raw admiral pie")]{symbols.insert(symbol.into(),id);let mut item=gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();item.name=name.into();defs.items.insert(id as u32,item);}
        let rs=admiral_pie_assembly(&symbols,&defs,7).unwrap();assert_eq!(rs.iter().map(|(r,t)|(r.first,r.second,r.output,*t)).collect::<Vec<_>>(),vec![(2315,329,7192,1),(7192,361,7194,2),(7194,1942,7196,2)]);
        for (r,_) in rs{assert_eq!((r.level,r.base_xp,r.members),(70,0,true));let mut inv=gielinor_world::inventory::Inventory::new(2,gielinor_world::inventory::StackType::Normal,false);inv.restore_slot(0,Some(gielinor_world::inventory::Item{id:r.first,count:1,vars:0})).unwrap();inv.restore_slot(1,Some(gielinor_world::inventory::Item{id:r.second,count:1,vars:0})).unwrap();let mut stats=gielinor_world::stats::Stats::default();stats.set_base(7,69);stats.set_current(7,70);let before=stats.clone();let job=gielinor_world::assembly::Job{recipe:r,first_slot:0,second_slot:1};assert_eq!(job.complete_awarding(&mut inv,&Default::default(),&mut stats,500,true).0,gielinor_world::assembly::Outcome::Made);assert_eq!(stats,before);assert_eq!(inv.slots().iter().flatten().filter(|i|i.id==r.output).count(),1);}
        defs.items.get_mut(&7196).unwrap().cert.template=799;assert!(admiral_pie_assembly(&symbols,&defs,7).is_err());
    }
}

pub fn assembly(
    symbols: &Symbols,
    defs: &Definitions,
    skill: u8,
) -> Result<Vec<gielinor_world::assembly::Recipe>, String> {
    let mut recipes = Vec::new();
    for (first, second, output, level, base_xp, members) in [
        ("pastry_dough", "piedish", "pie_shell", 1, 0, false),
        (
            "pie_shell",
            "redberries",
            "uncooked_redberry_pie",
            10,
            0,
            false,
        ),
        (
            "pie_shell",
            "cooked_meat",
            "uncooked_meat_pie",
            20,
            0,
            false,
        ),
        (
            "pie_shell",
            "cooked_chicken",
            "uncooked_meat_pie",
            20,
            0,
            false,
        ),
        (
            "pie_shell",
            "cooking_apple",
            "uncooked_apple_pie",
            30,
            0,
            false,
        ),
        (
            "pie_shell",
            "golovanova_top",
            "uncooked_botanical_pie",
            52,
            0,
            true,
        ),
        (
            "pie_shell",
            "fossil_sulliuscep_cap",
            "uncooked_mushroom_pie",
            60,
            0,
            true,
        ),
        (
            "pie_shell",
            "dragonfruit",
            "uncooked_dragonfruit_pie",
            73,
            0,
            true,
        ),
        (
            "pie_shell",
            "tomato",
            "unfinished_garden_pie_1",
            34,
            0,
            true,
        ),
        (
            "unfinished_garden_pie_1",
            "onion",
            "unfinished_garden_pie_2",
            34,
            0,
            true,
        ),
        (
            "unfinished_garden_pie_2",
            "cabbage",
            "uncooked_garden_pie",
            34,
            0,
            true,
        ),
        ("pie_shell", "trout", "unfinished_fish_pie_1", 47, 0, true),
        (
            "unfinished_fish_pie_1",
            "cod",
            "unfinished_fish_pie_2",
            47,
            0,
            true,
        ),
        (
            "unfinished_fish_pie_2",
            "potato",
            "uncooked_fish_pie",
            47,
            0,
            true,
        ),
        (
            "pie_shell",
            "raw_bear_meat",
            "unfinished_wild_pie_1",
            85,
            0,
            true,
        ),
        (
            "unfinished_wild_pie_1",
            "raw_chompy",
            "unfinished_wild_pie_2",
            85,
            0,
            true,
        ),
        (
            "unfinished_wild_pie_2",
            "raw_rabbit",
            "uncooked_wild_pie",
            85,
            0,
            true,
        ),
        (
            "pie_shell",
            "strawberry",
            "unfinished_summer_pie_1",
            95,
            0,
            true,
        ),
        (
            "unfinished_summer_pie_1",
            "watermelon",
            "unfinished_summer_pie_2",
            95,
            0,
            true,
        ),
        (
            "unfinished_summer_pie_2",
            "cooking_apple",
            "uncooked_summer_pie",
            95,
            0,
            true,
        ),
        ("pizza_base", "tomato", "incomplete_pizza", 35, 0, false),
        ("incomplete_pizza", "cheese", "uncooked_pizza", 35, 0, false),
        ("plain_pizza", "cooked_meat", "meat_pizza", 45, 26, false),
        ("plain_pizza", "cooked_chicken", "meat_pizza", 45, 26, false),
        ("plain_pizza", "anchovies", "anchovie_pizza", 55, 39, false),
        (
            "plain_pizza",
            "pineapple_chunks",
            "pineapple_pizza",
            65,
            45,
            true,
        ),
        (
            "plain_pizza",
            "pineapple_ring",
            "pineapple_pizza",
            65,
            45,
            true,
        ),
    ] {
        let resolve = |name: &str| -> Result<i32, String> {
            let id = *symbols
                .get(name)
                .ok_or_else(|| format!("Missing assembly symbol {name}"))?;
            let ty = defs
                .items
                .get(&(id as u32))
                .ok_or_else(|| format!("Missing assembly definition {name}"))?;
            if ty.cert.template != 0
                || ty.placeholder.template != 0
                || ty.stackable
                || ty.dummy == 1
            {
                return Err(format!("Invalid assembly profile {name}"));
            }
            Ok(id)
        };
        let first = resolve(first)?;
        let second = resolve(second)?;
        let output = resolve(output)?;
        if first == second || first == output || second == output {
            return Err("Aliased assembly ingredients/output".into());
        }
        recipes.push(gielinor_world::assembly::Recipe {
            first,
            second,
            output,
            skill,
            level,
            base_xp,
            members,
        });
    }
    Ok(recipes)
}

#[cfg(test)]
mod assembly_binding_tests {
    use super::*;
    #[test]
    fn assembly_profiles_require_real_unnoted_definitions() {
        let names = [
            "pastry_dough",
            "piedish",
            "pie_shell",
            "redberries",
            "uncooked_redberry_pie",
            "cooked_meat",
            "cooked_chicken",
            "cooking_apple",
            "uncooked_meat_pie",
            "uncooked_apple_pie",
            "golovanova_top",
            "uncooked_botanical_pie",
            "fossil_sulliuscep_cap",
            "uncooked_mushroom_pie",
            "dragonfruit",
            "uncooked_dragonfruit_pie",
            "onion",
            "cabbage",
            "unfinished_garden_pie_1",
            "unfinished_garden_pie_2",
            "uncooked_garden_pie",
            "trout",
            "cod",
            "potato",
            "unfinished_fish_pie_1",
            "unfinished_fish_pie_2",
            "uncooked_fish_pie",
            "raw_bear_meat",
            "raw_chompy",
            "raw_rabbit",
            "unfinished_wild_pie_1",
            "unfinished_wild_pie_2",
            "uncooked_wild_pie",
            "strawberry",
            "watermelon",
            "unfinished_summer_pie_1",
            "unfinished_summer_pie_2",
            "uncooked_summer_pie",
            "plain_pizza",
            "meat_pizza",
            "anchovies",
            "anchovie_pizza",
            "pineapple_chunks",
            "pineapple_ring",
            "pineapple_pizza",
            "pizza_base",
            "tomato",
            "incomplete_pizza",
            "cheese",
            "uncooked_pizza",
        ];
        let mut symbols = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (index, name) in names.iter().enumerate() {
            let id = index as i32 + 1;
            symbols.insert((*name).into(), id);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&[0]).unwrap(),
            );
        }
        let recipes = assembly(&symbols, &defs, 7).unwrap();
        assert_eq!(
            recipes.iter().map(|r| r.level).collect::<Vec<_>>(),
            [
                1, 10, 20, 20, 30, 52, 60, 73, 34, 34, 34, 47, 47, 47, 85, 85, 85, 95, 95, 95, 35,
                35, 45, 45, 55, 65, 65
            ]
        );
        for (ingredient, output, level, xp) in [
            ("cooked_meat", "meat_pizza", 45, 26),
            ("cooked_chicken", "meat_pizza", 45, 26),
            ("anchovies", "anchovie_pizza", 55, 39),
        ] {
            let recipe = recipes
                .iter()
                .find(|r| r.first == symbols["plain_pizza"] && r.second == symbols[ingredient])
                .unwrap();
            assert_eq!(
                (recipe.output, recipe.level, recipe.base_xp),
                (symbols[output], level, xp)
            );
        }
        assert!(
            recipes
                .iter()
                .filter(|r| r.first != symbols["plain_pizza"])
                .all(|r| r.base_xp == 0)
        );
        for input in ["pineapple_chunks", "pineapple_ring"] {
            let recipe = recipes
                .iter()
                .find(|r| r.first == symbols["plain_pizza"] && r.second == symbols[input])
                .unwrap();
            assert_eq!(
                (recipe.output, recipe.level, recipe.base_xp, recipe.members),
                (symbols["pineapple_pizza"], 65, 45, true)
            );
        }
        assert_eq!(recipes.iter().filter(|r| r.members).count(), 17);
        // Filling admission uses live boosts and membership; assembly gives no
        // baking XP. Unrelated input-gathering requirements are not pie gates.
        for (ingredient, output, level, member) in [
            ("cooked_meat", "uncooked_meat_pie", 20, false),
            ("cooked_chicken", "uncooked_meat_pie", 20, false),
            ("cooking_apple", "uncooked_apple_pie", 30, false),
            ("golovanova_top", "uncooked_botanical_pie", 52, true),
            ("fossil_sulliuscep_cap", "uncooked_mushroom_pie", 60, true),
            ("dragonfruit", "uncooked_dragonfruit_pie", 73, true),
        ] {
            use gielinor_world::{
                assembly::{Job, Outcome},
                inventory::{Catalog, Inventory, Item, StackType},
                stats::Stats,
            };
            let recipe = *recipes
                .iter()
                .find(|r| r.second == symbols[ingredient])
                .unwrap();
            assert_eq!(recipe.first, symbols["pie_shell"]);
            assert_eq!(recipe.output, symbols[output]);
            assert_eq!(recipe.members, member);
            let mut inv = Inventory::new(2, StackType::Normal, false);
            for (slot, id) in [(0, recipe.first), (1, recipe.second)] {
                inv.restore_slot(
                    slot,
                    Some(Item {
                        id,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
            let original = inv.clone();
            let mut stats = Stats::default();
            let job = Job {
                recipe,
                first_slot: 0,
                second_slot: 1,
            };
            if member {
                stats.set_current(7, level);
                let before = stats.clone();
                assert_eq!(
                    job.complete_awarding(&mut inv, &Catalog::default(), &mut stats, 500, false),
                    (Outcome::Members, None)
                );
                assert_eq!(inv, original);
                assert_eq!(stats, before);
            }
            for low in [-1, level - 1] {
                stats.set_current(7, low);
                assert_eq!(
                    job.complete_awarding(&mut inv, &Catalog::default(), &mut stats, 500, member)
                        .0,
                    Outcome::Level
                );
                assert_eq!(inv, original);
            }
            stats.set_current(7, level);
            assert_eq!(stats.base(7), 1);
            let before = stats.clone();
            assert_eq!(
                job.complete_awarding(&mut inv, &Catalog::default(), &mut stats, 500, member)
                    .0,
                Outcome::Made
            );
            assert_eq!(
                inv.slots()
                    .iter()
                    .flatten()
                    .map(|i| (i.id, i.count))
                    .collect::<Vec<_>>(),
                vec![(symbols[output], 1)]
            );
            assert_eq!(stats.fine_xp(7), 0);
            assert_eq!(stats, before);
        }
        // Complete each three-stage chain from a full backpack. Every stage
        // independently rechecks membership/boost and commits no baking XP.
        for (ingredients, outputs, level) in [
            (
                ["raw_bear_meat", "raw_chompy", "raw_rabbit"],
                [
                    "unfinished_wild_pie_1",
                    "unfinished_wild_pie_2",
                    "uncooked_wild_pie",
                ],
                85,
            ),
            (
                ["strawberry", "watermelon", "cooking_apple"],
                [
                    "unfinished_summer_pie_1",
                    "unfinished_summer_pie_2",
                    "uncooked_summer_pie",
                ],
                95,
            ),
            (
                ["tomato", "onion", "cabbage"],
                [
                    "unfinished_garden_pie_1",
                    "unfinished_garden_pie_2",
                    "uncooked_garden_pie",
                ],
                34,
            ),
            (
                ["trout", "cod", "potato"],
                [
                    "unfinished_fish_pie_1",
                    "unfinished_fish_pie_2",
                    "uncooked_fish_pie",
                ],
                47,
            ),
        ] {
            use gielinor_world::{
                assembly::{Job, Outcome},
                inventory::{Catalog, Inventory, Item, StackType},
                stats::Stats,
            };
            let mut inv = Inventory::new(5, StackType::Normal, false);
            for (slot, id) in std::iter::once(symbols["pie_shell"])
                .chain(ingredients.iter().map(|n| symbols[*n]))
                .enumerate()
            {
                inv.restore_slot(
                    slot,
                    Some(Item {
                        id,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
            inv.restore_slot(
                4,
                Some(Item {
                    id: 995,
                    count: 1000,
                    vars: 0,
                }),
            )
            .unwrap();
            let coin = inv.get(4);
            let mut stats = Stats::default();
            let mut catalog = Catalog::default();
            catalog.stackable.insert(995);
            let mut base = symbols["pie_shell"];
            for stage in 0..3 {
                let recipe = *recipes
                    .iter()
                    .find(|r| r.first == base && r.second == symbols[ingredients[stage]])
                    .unwrap();
                assert_eq!(
                    (recipe.output, recipe.level, recipe.base_xp, recipe.members),
                    (symbols[outputs[stage]], level as u8, 0, true)
                );
                let job = Job {
                    recipe,
                    first_slot: 0,
                    second_slot: stage + 1,
                };
                stats.set_current(7, level);
                let original = inv.clone();
                let skills = stats.clone();
                assert_eq!(
                    job.complete_awarding(&mut inv, &catalog, &mut stats, 500, false),
                    (Outcome::Members, None)
                );
                assert_eq!(inv, original);
                assert_eq!(stats, skills);
                stats.set_current(7, level - 1);
                let low = stats.clone();
                assert_eq!(
                    job.complete_awarding(&mut inv, &catalog, &mut stats, 500, true),
                    (Outcome::Level, None)
                );
                assert_eq!(inv, original);
                assert_eq!(stats, low);
                stats.set_current(7, level);
                catalog.dummy.insert(recipe.output);
                assert_eq!(
                    job.complete_awarding(&mut inv, &catalog, &mut stats, 500, true),
                    (Outcome::Capacity, None)
                );
                assert_eq!(inv, original);
                assert_eq!(stats, skills);
                catalog.dummy.remove(&recipe.output);
                assert_eq!(
                    job.complete_awarding(&mut inv, &catalog, &mut stats, 500, true),
                    (Outcome::Made, None)
                );
                assert_eq!(stats, skills);
                assert_eq!(stats.base(7), 1);
                assert_eq!(
                    inv.get(0),
                    Some(Item {
                        id: recipe.output,
                        count: 1,
                        vars: 0
                    })
                );
                assert_eq!(inv.get(4), coin);
                for slot in 1..=stage + 1 {
                    assert!(inv.get(slot).is_none());
                }
                for slot in stage + 2..4 {
                    assert_eq!(inv.get(slot), original.get(slot));
                }
                let committed = inv.clone();
                assert_eq!(
                    job.complete_awarding(&mut inv, &catalog, &mut stats, 500, true),
                    (Outcome::Stale, None)
                );
                assert_eq!(inv, committed);
                assert_eq!(stats, skills);
                base = recipe.output;
            }
        }
        for (name, alias) in [
            ("pineapple_chunks", "plain_pizza"),
            ("pineapple_pizza", "plain_pizza"),
            ("pineapple_pizza", "pineapple_chunks"),
        ] {
            let original = symbols[name];
            let alias = symbols[alias];
            symbols.insert(name.into(), alias);
            assert!(assembly(&symbols, &defs, 7).is_err());
            symbols.insert(name.into(), original);
        }
        let output = symbols["pineapple_pizza"] as u32;
        defs.items.get_mut(&output).unwrap().dummy = 1;
        assert!(assembly(&symbols, &defs, 7).is_err());
        defs.items.get_mut(&output).unwrap().dummy = -1;
        assert!(assembly(&symbols, &defs, 7).is_ok());
        defs.items.get_mut(&1).unwrap().cert.template = 799;
        assert!(assembly(&symbols, &defs, 7).is_err());
        defs.items.get_mut(&1).unwrap().cert.template = 0;
        symbols.remove("tomato");
        assert!(assembly(&symbols, &defs, 7).is_err());
    }
}

/// Ordinary bucket profile. Numeric60788 was independently attested in the
/// prepared239.1 cache (scene roots004), not imported from OpenRune241.
pub fn milking(
    obj: &Symbols,
    loc: &Symbols,
    seq: &Symbols,
    synth: &Symbols,
    defs: &Definitions,
) -> Result<gielinor_world::milking::Config, String> {
    let resolve = |map: &Symbols, key: &str| {
        map.get(key)
            .copied()
            .ok_or_else(|| format!("Missing milk symbol {key}"))
    };
    let bucket = resolve(obj, "bucket_empty")?;
    let milk = resolve(obj, "bucket_milk")?;
    for id in [bucket, milk] {
        let item = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing milk item definition")?;
        if item.stackable || item.cert.template != 0 || item.placeholder.template != 0 {
            return Err("Invalid ordinary milk vessel".into());
        }
    }
    if bucket == milk {
        return Err("Milk input/output aliases".into());
    }
    let mut cows = BTreeMap::new();
    for (id, operation) in [
        (resolve(loc, "fat_cow")?, 1),
        (resolve(loc, "fairy_fat_cow")?, 2),
        (60788, 1),
    ] {
        let Some(def) = defs.locations.get(&(id as u32)) else {
            if id == 60788 {
                continue;
            }
            return Err("Missing cow definition".into());
        };
        if !def.name.eq_ignore_ascii_case("Dairy cow")
            || !def.multi.is_empty()
            || def.ops[operation as usize - 1].as_deref() != Some("Milk")
            || def.collision.width != 1
            || def.collision.length != 2
        {
            return Err(format!("Unattested cow profile {id}"));
        }
        cows.insert(id as u32, operation);
    }
    Ok(gielinor_world::milking::Config {
        cows,
        bucket,
        milk,
        animation: u16::try_from(resolve(seq, "milkit")?).map_err(|_| "Invalid milk animation")?,
        sound: u16::try_from(resolve(synth, "milk_cow")?).map_err(|_| "Invalid milk sound")?,
    })
}

#[cfg(test)]
mod milking_binding_tests {
    use super::*;
    #[test]
    fn milk_bindings_require_ordinary_vessels_and_exact_cow_operations() {
        let obj = [("bucket_empty".into(), 10), ("bucket_milk".into(), 20)].into();
        let loc = [("fat_cow".into(), 1), ("fairy_fat_cow".into(), 2)].into();
        let seq = [("milkit".into(), 2305)].into();
        let synth = [("milk_cow".into(), 372)].into();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for id in [10, 20] {
            defs.items.insert(
                id,
                gielinor_cache::definitions::ItemType::decode(&[0]).unwrap(),
            );
        }
        for (id, op) in [(1, 0), (2, 1), (60788, 0)] {
            let mut ty = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
            ty.name = "Dairy cow".into();
            ty.collision.length = 2;
            ty.ops[op] = Some("Milk".into());
            defs.locations.insert(id, ty);
        }
        let config = milking(&obj, &loc, &seq, &synth, &defs).unwrap();
        assert_eq!(config.cows, [(1, 1), (2, 2), (60788, 1)].into());
        defs.locations.get_mut(&60788).unwrap().ops[0] = Some("Talk-to".into());
        assert!(milking(&obj, &loc, &seq, &synth, &defs).is_err());
        defs.locations.remove(&60788);
        assert!(milking(&obj, &loc, &seq, &synth, &defs).is_ok());
        defs.locations.get_mut(&1).unwrap().multi.push(2);
        assert!(milking(&obj, &loc, &seq, &synth, &defs).is_err());
        defs.locations.get_mut(&1).unwrap().multi.clear();
        defs.items.get_mut(&20).unwrap().cert.template = 1;
        assert!(milking(&obj, &loc, &seq, &synth, &defs).is_err());
    }
}

/// Narrow ordinary sink/well profiles attested in the prepared 239.1 cache.
/// Does not infer source behavior from a display name or map icon alone.
pub fn water_sources(
    obj: &Symbols,
    loc: &Symbols,
    defs: &Definitions,
) -> Result<gielinor_world::water_sources::Config, String> {
    use gielinor_world::water_sources::{Config, Source, Vessel};
    let resolve = |symbols: &Symbols, name: &str| {
        symbols
            .get(name)
            .copied()
            .ok_or_else(|| format!("Missing water-source symbol {name}"))
    };
    let mut config = Config::default();
    for (input, output, well) in [
        ("bucket_empty", "bucket_water", true),
        ("jug_empty", "jug_water", false),
        ("bowl_empty", "bowl_water", false),
        ("vial_empty", "vial_water", false),
    ] {
        let input = resolve(obj, input)?;
        let output = resolve(obj, output)?;
        if input == output {
            return Err("Water input/output aliases".into());
        }
        for id in [input, output] {
            let item = defs
                .items
                .get(&(id as u32))
                .ok_or("Missing water vessel definition")?;
            if item.stackable || item.cert.template != 0 || item.placeholder.template != 0 {
                return Err("Invalid ordinary water vessel".into());
            }
        }
        if config
            .vessels
            .insert(input, Vessel { output, well })
            .is_some()
        {
            return Err("Duplicate water vessel".into());
        }
    }
    for (symbol, kind, name, width, length) in [
        ("sink", Source::Sink, "Sink", 1, 2),
        ("sink2", Source::Sink, "Sink", 1, 2),
        ("desertwell", Source::Well, "Well", 1, 1),
        ("well", Source::Well, "Well", 2, 2),
    ] {
        let id = resolve(loc, symbol)? as u32;
        let def = defs
            .locations
            .get(&id)
            .ok_or("Missing water source definition")?;
        if def.name != name
            || !def.multi.is_empty()
            || def.collision.width != width
            || def.collision.length != length
            || def.ops.iter().any(Option::is_some)
        {
            return Err(format!("Unattested water source {symbol}"));
        }
        if config.sources.insert(id, kind).is_some() {
            return Err("Duplicate water source".into());
        }
    }
    Ok(config)
}

#[cfg(test)]
mod water_binding_tests {
    use super::*;
    #[test]
    fn names_bind_separate_source_policies_and_reject_malformed_definitions() {
        let mut obj = Symbols::new();
        let mut loc = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (index, name) in [
            "bucket_empty",
            "bucket_water",
            "jug_empty",
            "jug_water",
            "bowl_empty",
            "bowl_water",
            "vial_empty",
            "vial_water",
        ]
        .into_iter()
        .enumerate()
        {
            let id = 100 + index as i32;
            obj.insert(name.into(), id);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&[0]).unwrap(),
            );
        }
        for (index, (symbol, name, width, length)) in [
            ("sink", "Sink", 1, 2),
            ("sink2", "Sink", 1, 2),
            ("desertwell", "Well", 1, 1),
            ("well", "Well", 2, 2),
        ]
        .into_iter()
        .enumerate()
        {
            let id = 300 + index as i32;
            loc.insert(symbol.into(), id);
            let mut raw = vec![2];
            raw.extend_from_slice(name.as_bytes());
            raw.extend_from_slice(&[0, 14, width, 15, length, 0]);
            defs.locations.insert(
                id as u32,
                gielinor_cache::locations::LocationType::decode(&raw).unwrap(),
            );
        }
        let config = water_sources(&obj, &loc, &defs).unwrap();
        assert_eq!(config.output(300, 102), Some(103));
        assert_eq!(config.output(303, 100), Some(101));
        assert_eq!(config.output(303, 102), None);
        defs.items.get_mut(&101).unwrap().cert.template = 799;
        assert!(water_sources(&obj, &loc, &defs).is_err());
        defs.items.get_mut(&101).unwrap().cert.template = 0;
        defs.locations.get_mut(&300).unwrap().name = "Decorative sink".into();
        assert!(water_sources(&obj, &loc, &defs).is_err());
    }
}

/// Independently Wiki-pinned recipes. Other crushing-table rows remain unassessed.
/// Crushed nest15184159, Dragon scale dust15184318, Goat horn dust15280508:
/// one-for-one retained pestle, no XP, manual0 / automatic2 then3 ticks.
/// Target ordinary definitions attested in openrune-grinding-three-evidence-001.json.
/// Superior bones: Wiki15188636 cadence; MMG14246338 explicitly no skill/quest/XP.
/// This is crushing, not burial or offering (which have separate Prayer gates).
pub fn grinding(obj:&Symbols,seq:&Symbols,defs:&Definitions)->Result<Vec<gielinor_world::grinding::Recipe>,String>{
    let resolve=|symbols:&Symbols,name:&str|symbols.get(name).copied().ok_or_else(||format!("Missing grinding symbol {name}"));
    let animation=u16::try_from(resolve(seq,"human_herbing_grind")?).map_err(|_|"Invalid grinding sequence")?;
    let mut pairs=std::collections::BTreeSet::new();let mut recipes=Vec::new();
    for (tool,input,output,members) in [("knife","chocolate_bar","chocolate_dust",false),("pestle_and_mortar","chocolate_bar","chocolate_dust",true),("pestle_and_mortar","unicorn_horn","unicorn_horn_dust",true),("pestle_and_mortar","bird_nest_empty","crushed_bird_nest",true),("pestle_and_mortar","blue_dragon_scale","dragon_scale_dust",true),("pestle_and_mortar","desert_goat_horn","ground_desert_goat_horn",true),("pestle_and_mortar","dragon_bones_superior","crushed_dragon_bones",true)] {
        let tool=resolve(obj,tool)?;let input=resolve(obj,input)?;let output=resolve(obj,output)?;
        if tool==input || input==output || tool==output || !pairs.insert((tool,input)) {return Err("Aliased grinding recipe".into());}
        for id in [tool,input,output]{
            let item=defs.items.get(&(id as u32)).ok_or("Missing grinding item definition")?;
            if item.stackable || item.cert.template!=0 || item.placeholder.template!=0 {return Err("Invalid ordinary grinding item".into());}
        }
        recipes.push(gielinor_world::grinding::Recipe{tool,input,output,members,animation:members.then_some(animation)});
    }
    Ok(recipes)
}

#[cfg(test)]
mod grinding_binding_tests {
    use super::*;
    #[test]
    fn named_recipes_distinguish_member_tool_and_validate_items() {
        let mut obj=Symbols::new();let seq=Symbols::from([("human_herbing_grind".into(),901)]);
        let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
        for (i,name) in ["knife","pestle_and_mortar","chocolate_bar","chocolate_dust","unicorn_horn","unicorn_horn_dust","bird_nest_empty","crushed_bird_nest","blue_dragon_scale","dragon_scale_dust","desert_goat_horn","ground_desert_goat_horn","dragon_bones_superior","crushed_dragon_bones"].into_iter().enumerate(){
            obj.insert(name.into(),100+i as i32);defs.items.insert(100+i as u32,gielinor_cache::definitions::ItemType::decode(&[0]).unwrap());
        }
        let recipes=grinding(&obj,&seq,&defs).unwrap();assert_eq!(recipes.len(),7);
        for (index,input,output) in [(3,106,107),(4,108,109),(5,110,111),(6,112,113)] {
            assert_eq!((recipes[index].tool,recipes[index].input,recipes[index].output),(101,input,output));
            assert!(recipes[index].members);assert_eq!(recipes[index].animation,Some(901));
            let saved=defs.items.remove(&(output as u32)).unwrap();
            assert!(grinding(&obj,&seq,&defs).is_err());defs.items.insert(output as u32,saved);
        }
        assert_eq!((recipes[0].tool,recipes[0].input,recipes[0].output),(100,102,103));
        assert!(!recipes[0].members);assert_eq!(recipes[0].animation,None);
        assert!(recipes[1].members);assert_eq!(recipes[1].animation,Some(901));
        assert_eq!((recipes[2].tool,recipes[2].input,recipes[2].output),(101,104,105));
        defs.items.get_mut(&103).unwrap().cert.template=799;assert!(grinding(&obj,&seq,&defs).is_err());
        defs.items.get_mut(&103).unwrap().cert.template=0;obj.insert("chocolate_dust".into(),102);assert!(grinding(&obj,&seq,&defs).is_err());
    }
}

/// Ordinary dough inputs are bound independently of tutorial progression.
pub fn dough(
    symbols: &Symbols,
    defs: &Definitions,
    skill: u8,
) -> Result<gielinor_world::dough::Config, String> {
    let resolve = |name: &str| -> Result<i32, String> {
        let id = *symbols
            .get(name)
            .ok_or_else(|| format!("Missing dough symbol {name}"))?;
        let ty = defs
            .items
            .get(&(id as u32))
            .ok_or_else(|| format!("Missing dough definition {name}"))?;
        if ty.cert.template != 0 || ty.placeholder.template != 0 || ty.stackable {
            return Err(format!("Invalid ordinary dough item {name}"));
        }
        Ok(id)
    };
    let waters = [
        ("bucket_water", "bucket_empty"),
        ("jug_water", "jug_empty"),
        ("bowl_water", "bowl_empty"),
    ]
    .into_iter()
    .map(|(w, v)| Ok((resolve(w)?, resolve(v)?)))
    .collect::<Result<Vec<_>, String>>()?;
    let config = gielinor_world::dough::Config {
        skill,
        tutorial: None,
        flour: resolve("pot_flour")?,
        pot: resolve("pot_empty")?,
        waters,
        outputs: [
            gielinor_world::dough::Output {
                id: resolve("bread_dough")?,
                level: 1,
            },
            gielinor_world::dough::Output {
                id: resolve("pastry_dough")?,
                level: 1,
            },
            gielinor_world::dough::Output {
                id: resolve("pizza_base")?,
                level: 35,
            },
        ],
    };
    let ids: Vec<_> = std::iter::once(config.flour)
        .chain(std::iter::once(config.pot))
        .chain(config.waters.iter().flat_map(|&(a, b)| [a, b]))
        .chain(config.outputs.iter().map(|v| v.id))
        .collect();
    if ids.iter().enumerate().any(|(i, id)| ids[..i].contains(id)) {
        return Err("Dough symbol alias".into());
    }
    Ok(config)
}

#[cfg(test)]
mod dough_binding_tests {
    use super::*;
    #[test]
    fn ordinary_dough_has_nine_bound_choices_and_rejects_aliases_and_notes() {
        let names = [
            "pot_flour",
            "pot_empty",
            "bucket_water",
            "bucket_empty",
            "jug_water",
            "jug_empty",
            "bowl_water",
            "bowl_empty",
            "bread_dough",
            "pastry_dough",
            "pizza_base",
        ];
        let mut symbols = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (i, name) in names.iter().enumerate() {
            let id = i as i32 + 1;
            symbols.insert((*name).into(), id);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&[0]).unwrap(),
            );
        }
        let config = dough(&symbols, &defs, 7).unwrap();
        assert_eq!(config.outputs.map(|o| o.level), [1, 1, 35]);
        for (water, vessel) in config.waters.iter().copied() {
            for index in 0..3 {
                let r = config.recipe(water, index).unwrap();
                assert_eq!(r.vessel, vessel);
            }
        }
        defs.items.get_mut(&1).unwrap().cert.template = 799;
        assert!(dough(&symbols, &defs, 7).is_err());
        defs.items.get_mut(&1).unwrap().cert.template = 0;
        symbols.insert("pot_empty".into(), 1);
        assert!(dough(&symbols, &defs, 7).is_err());
    }
}

/// Selected manual herb profiles; automatic continuation timing remains pending.
pub fn herb_cleaning(
    symbols: &Symbols,
    defs: &Definitions,
    skill: u8,
) -> Result<Vec<gielinor_world::herb_cleaning::Recipe>, String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut recipes = Vec::new();
    for (input, output, level, base_fine_xp) in [
        ("unidentified_guam", "guam_leaf", 3, 25),
        ("unidentified_marentill", "marentill", 5, 38),
        ("unidentified_tarromin", "tarromin", 11, 50),
        ("unidentified_harralander", "harralander", 20, 63),
        ("unidentified_ranarr", "ranarr_weed", 25, 75),
        ("unidentified_toadflax", "toadflax", 30, 80),
        ("unidentified_irit", "irit_leaf", 40, 88),
        ("unidentified_avantoe", "avantoe", 48, 100),
        ("unidentified_kwuarm", "kwuarm", 54, 113),
        ("unidentified_huasca", "huasca", 58, 118),
        ("unidentified_snapdragon", "snapdragon", 59, 118),
        ("unidentified_cadantine", "cadantine", 65, 125),
        ("unidentified_lantadyme", "lantadyme", 67, 131),
        ("unidentified_dwarf_weed", "dwarf_weed", 70, 138),
        ("unidentified_torstol", "torstol", 75, 150),
    ] {
        let mut ids = Vec::new();
        for (name, clean) in [(input, true), (output, false)] {
            let id = *symbols
                .get(name)
                .ok_or_else(|| format!("Missing herb symbol {name}"))?;
            let item = defs
                .items
                .get(&(id as u32))
                .ok_or_else(|| format!("Missing herb definition {name}"))?;
            if id < 0
                || !seen.insert(id)
                || item.stackable
                || item.cert.template != 0
                || item.placeholder.template != 0
                || item.dummy == 1
                || !item.members
            {
                return Err(format!("Invalid ordinary herb {name}"));
            }
            if clean && item.inventory_ops[0].as_deref() != Some("Clean") {
                return Err(format!("Missing Clean operation for {name}"));
            }
            if !clean
                && item
                    .inventory_ops
                    .iter()
                    .any(|op| op.as_deref() == Some("Clean"))
            {
                return Err(format!("Clean output still has Clean operation: {name}"));
            }
            ids.push(id);
        }
        recipes.push(gielinor_world::herb_cleaning::Recipe {
            input: ids[0],
            output: ids[1],
            skill,
            level,
            base_fine_xp,
        });
    }
    Ok(recipes)
}
/// Read only the attested Druidic Ritual progress state; never infer quest completion.
pub fn herb_cleaning_config(
    objects: &Symbols,
    varps: &Symbols,
    defs: &Definitions,
    skill: u8,
) -> Result<gielinor_world::herb_cleaning::Config, String> {
    let quest = *varps
        .get("druidquest")
        .ok_or("Missing Druidic Ritual symbol")?;
    let progress = defs
        .varps
        .get(&(quest as u32))
        .ok_or("Missing Druidic Ritual progress")?;
    if quest != 80 || progress.id != 80 || !progress.permanent {
        return Err("Druidic Ritual progress differs from attested239.1 binding".into());
    }
    Ok(gielinor_world::herb_cleaning::Config {
        recipes: herb_cleaning(objects, defs, skill)?
            .into_iter()
            .map(|r| (r.input, r))
            .collect(),
        quest_progress: progress.id,
        quest_complete: 4,
    })
}
#[cfg(test)]
mod herb_binding_tests {
    use super::*;
    fn fixture() -> (Symbols, Definitions) {
        let mut symbols = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (i, name) in [
            "unidentified_guam",
            "guam_leaf",
            "unidentified_marentill",
            "marentill",
            "unidentified_tarromin",
            "tarromin",
            "unidentified_harralander",
            "harralander",
            "unidentified_ranarr",
            "ranarr_weed",
            "unidentified_toadflax",
            "toadflax",
            "unidentified_irit",
            "irit_leaf",
            "unidentified_avantoe",
            "avantoe",
            "unidentified_kwuarm",
            "kwuarm",
            "unidentified_huasca",
            "huasca",
            "unidentified_snapdragon",
            "snapdragon",
            "unidentified_cadantine",
            "cadantine",
            "unidentified_lantadyme",
            "lantadyme",
            "unidentified_dwarf_weed",
            "dwarf_weed",
            "unidentified_torstol",
            "torstol",
        ]
        .into_iter()
        .enumerate()
        {
            let mut raw = vec![16];
            if i % 2 == 0 {
                raw.extend_from_slice(&[35, b'C', b'l', b'e', b'a', b'n', 0]);
            }
            raw.push(0);
            symbols.insert(name.into(), 100 + i as i32);
            defs.items.insert(
                100 + i as u32,
                gielinor_cache::definitions::ItemType::decode(&raw).unwrap(),
            );
        }
        (symbols, defs)
    }
    #[test]
    fn all_normal_herbs_reject_below_level_and_award_exact_fine_xp_at_threshold() {
        use gielinor_world::{
            herb_cleaning::{Job, Outcome},
            inventory::{Catalog, Inventory, Item, StackType},
            stats::Stats,
        };
        let (symbols, defs) = fixture();
        for recipe in herb_cleaning(&symbols, &defs, 15).unwrap() {
            let mut inv = Inventory::new(1, StackType::Normal, false);
            inv.restore_slot(
                0,
                Some(Item {
                    id: recipe.input,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
            let mut stats = Stats::default();
            stats.set_fine_xp(15, 100_001).unwrap();
            stats.set_current(15, recipe.level as i8 - 1);
            let original = inv.clone();
            let job = Job { recipe, slot: 0 };
            assert_eq!(
                job.complete(&mut inv, &Catalog::default(), &mut stats, 500, true, true)
                    .0,
                Outcome::Level
            );
            assert_eq!(inv, original);
            assert_eq!(stats.fine_xp(15), 100_001);
            stats.set_current(15, recipe.level as i8);
            assert_eq!(
                job.complete(&mut inv, &Catalog::default(), &mut stats, 500, true, true)
                    .0,
                Outcome::Cleaned
            );
            assert_eq!(inv.get(0).unwrap().id, recipe.output);
            assert_eq!(
                stats.fine_xp(15),
                100_001 + i32::from(recipe.base_fine_xp) * 5,
                "herb{}",
                recipe.input
            );
        }
    }
    #[test]
    fn manual_herb_permission_binding_rejects_missing_or_different_quest_state() {
        let (objects, mut defs) = fixture();
        let mut symbols = Symbols::new();
        assert!(herb_cleaning_config(&objects, &symbols, &defs, 15).is_err());
        symbols.insert("druidquest".into(), 80);
        assert!(herb_cleaning_config(&objects, &symbols, &defs, 15).is_err());
        let varp = gielinor_world::variables::Varp {
            id: 80,
            permanent: true,
            transmit: gielinor_world::variables::Transmit::Always,
            bit_protect: false,
            client_code: -1,
        };
        defs.varps.insert(80, varp);
        let config = herb_cleaning_config(&objects, &symbols, &defs, 15).unwrap();
        assert_eq!(
            (
                config.quest_progress,
                config.quest_complete,
                config.recipes.len()
            ),
            (80, 4, 15)
        );
        defs.varps.get_mut(&80).unwrap().permanent = false;
        assert!(herb_cleaning_config(&objects, &symbols, &defs, 15).is_err());
        defs.varps
            .insert(80, gielinor_world::variables::Varp { id: 81, ..varp });
        assert!(herb_cleaning_config(&objects, &symbols, &defs, 15).is_err());
        symbols.insert("druidquest".into(), 81);
        defs.varps
            .insert(81, gielinor_world::variables::Varp { id: 81, ..varp });
        assert!(herb_cleaning_config(&objects, &symbols, &defs, 15).is_err());
    }
    #[test]
    fn herb_profiles_require_member_unnoted_definitions_and_exact_clean_operations() {
        let (symbols, defs) = fixture();
        let recipes = herb_cleaning(&symbols, &defs, 15).unwrap();
        assert_eq!(
            recipes
                .iter()
                .map(|r| (r.input, r.output, r.level, r.base_fine_xp))
                .collect::<Vec<_>>(),
            [
                (100, 101, 3, 25),
                (102, 103, 5, 38),
                (104, 105, 11, 50),
                (106, 107, 20, 63),
                (108, 109, 25, 75),
                (110, 111, 30, 80),
                (112, 113, 40, 88),
                (114, 115, 48, 100),
                (116, 117, 54, 113),
                (118, 119, 58, 118),
                (120, 121, 59, 118),
                (122, 123, 65, 125),
                (124, 125, 67, 131),
                (126, 127, 70, 138),
                (128, 129, 75, 150),
            ]
        );
        for mode in 0..8 {
            let (mut symbols, mut defs) = fixture();
            match mode {
                0 => defs.items.get_mut(&100).unwrap().inventory_ops[0] = None,
                1 => defs.items.get_mut(&101).unwrap().cert.template = 799,
                2 => defs.items.get_mut(&101).unwrap().placeholder.template = 14401,
                3 => defs.items.get_mut(&101).unwrap().dummy = 1,
                4 => defs.items.get_mut(&100).unwrap().stackable = true,
                5 => defs.items.get_mut(&100).unwrap().members = false,
                6 => {
                    symbols.insert("marentill".into(), 101);
                }
                _ => defs.items.get_mut(&101).unwrap().inventory_ops[0] = Some("Clean".into()),
            }
            assert!(herb_cleaning(&symbols, &defs, 15).is_err(), "mode{mode}");
        }
    }
}

/// Fifteen ordinary water-based unfinished-potion profiles for the shared Make-X dialog.
/// Per-page Wiki evidence: openrune-unfinished-potions-profiles-001.json; no revision241 IDs reused.
pub fn unfinished_potions(
    objects: &Symbols,
    defs: &Definitions,
) -> Result<Vec<gielinor_world::unfinished_potions::Recipe>, String> {
    let mut seen = std::collections::BTreeMap::<i32, String>::new();
    let mut bind = |name: &str, members: bool| -> Result<i32, String> {
        let id = *objects
            .get(name)
            .ok_or_else(|| format!("Missing potion symbol {name}"))?;
        let item = defs
            .items
            .get(&(id as u32))
            .ok_or_else(|| format!("Missing potion item {name}"))?;
        if id < 0
            || seen.insert(id, name.to_owned()).is_some_and(|prior| prior != name)
            || item.stackable
            || item.cert.template != 0
            || item.placeholder.template != 0
            || item.dummy == 1
            || item.members != members
            || item
                .inventory_ops
                .iter()
                .any(|op| op.as_deref() == Some("Clean"))
        {
            return Err(format!(
                "Invalid ordinary potion ingredient or output {name}"
            ));
        }
        Ok(id)
    };
    let vessel = bind("vial_water", false)?;
    let mut recipes = Vec::new();
    for (herb, output, level) in [
        ("guam_leaf", "guamvial", 3),
        ("marentill", "marrentillvial", 5),
        ("tarromin", "tarrominvial", 12),
        ("harralander", "harralandervial", 22),
        ("ranarr_weed", "ranarrvial", 30),
        ("toadflax", "toadflaxvial", 34),
        ("irit_leaf", "iritvial", 45),
        ("avantoe", "avantoevial", 50),
        ("kwuarm", "kwuarmvial", 55),
        ("huasca", "huascavial", 58),
        ("snapdragon", "snapdragonvial", 63),
        ("cadantine", "cadantinevial", 66),
        ("lantadyme", "lantadymevial", 69),
        ("dwarf_weed", "dwarfweedvial", 72),
        ("torstol", "torstolvial", 78),
    ] {
        recipes.push(gielinor_world::unfinished_potions::Recipe {
            herb: bind(herb, true)?,
            vessel,
            output: bind(output, true)?,
            level,
            ticks: 1,
        });
    }
    for (herb, vessel, output, level, ticks) in [
        ("toadflax", "vial_coconut_milk", "unfinished_antidote+", 68, 2),
        ("irit_leaf", "vial_coconut_milk", "unfinished_antidote++", 79, 1),
        ("cadantine", "vial_blood", "cadantine_bloodvial", 80, 1),
    ] {
        recipes.push(gielinor_world::unfinished_potions::Recipe {
            herb: bind(herb, true)?, vessel: bind(vessel, true)?,
            output: bind(output, true)?, level, ticks,
        });
    }
    Ok(recipes)
}
#[cfg(test)]
mod unfinished_potion_binding_tests {
    use super::*;
    fn fixture() -> (Symbols, Definitions) {
        let mut symbols = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (name, id, member) in [
            ("guam_leaf", 249, true),
            ("vial_water", 227, false),
            ("vial_coconut_milk", 5935, true),
            ("unfinished_antidote+", 5942, true),
            ("unfinished_antidote++", 5951, true),
            ("vial_blood", 22446, true),
            ("cadantine_bloodvial", 22443, true),
            ("guamvial", 91, true),
            ("marentill", 251, true),
            ("marrentillvial", 93, true),
            ("tarromin", 253, true),
            ("tarrominvial", 95, true),
            ("harralander", 255, true),
            ("harralandervial", 97, true),
            ("ranarr_weed", 257, true),
            ("ranarrvial", 99, true),
            ("toadflax", 2998, true),
            ("toadflaxvial", 3002, true),
            ("irit_leaf", 259, true),
            ("iritvial", 101, true),
            ("avantoe", 261, true),
            ("avantoevial", 103, true),
            ("kwuarm", 263, true),
            ("kwuarmvial", 105, true),
            ("huasca", 30097, true),
            ("huascavial", 30100, true),
            ("snapdragon", 3000, true),
            ("snapdragonvial", 3004, true),
            ("cadantine", 265, true),
            ("cadantinevial", 107, true),
            ("lantadyme", 2481, true),
            ("lantadymevial", 2483, true),
            ("dwarf_weed", 267, true),
            ("dwarfweedvial", 109, true),
            ("torstol", 269, true),
            ("torstolvial", 111, true),
        ] {
            symbols.insert(name.into(), id);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(if member { &[16, 0] } else { &[0] })
                    .unwrap(),
            );
        }
        (symbols, defs)
    }
    #[test]
    fn named_profiles_have_zero_reward_at_boosted_threshold_and_reject_invalid_definitions() {
        use gielinor_world::{
            inventory::{Catalog, Inventory, Item, StackType},
            stats::Stats,
            unfinished_potions::{Job, Outcome},
        };
        let (symbols, defs) = fixture();
        let recipes = unfinished_potions(&symbols, &defs).unwrap();
        assert_eq!(
            recipes
                .iter()
                .map(|r| (r.herb, r.vessel, r.output, r.level, r.ticks))
                .collect::<Vec<_>>(),
            [
                (249, 227, 91, 3, 1),
                (251, 227, 93, 5, 1),
                (253, 227, 95, 12, 1),
                (255, 227, 97, 22, 1),
                (257, 227, 99, 30, 1),
                (2998, 227, 3002, 34, 1),
                (259, 227, 101, 45, 1),
                (261, 227, 103, 50, 1),
                (263, 227, 105, 55, 1),
                (30097, 227, 30100, 58, 1),
                (3000, 227, 3004, 63, 1),
                (265, 227, 107, 66, 1),
                (2481, 227, 2483, 69, 1),
                (267, 227, 109, 72, 1),
                (269, 227, 111, 78, 1),
                (2998, 5935, 5942, 68, 2),
                (259, 5935, 5951, 79, 1),
                (265, 22446, 22443, 80, 1)
            ]
        );
        for recipe in recipes {
            let mut inv = Inventory::new(2, StackType::Normal, false);
            for (slot, id) in [(0, recipe.herb), (1, recipe.vessel)] {
                inv.restore_slot(
                    slot,
                    Some(Item {
                        id,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
            let mut stats = Stats::default();
            stats.set_fine_xp(15, 3881).unwrap();
            stats.set_current(15, recipe.level as i8 - 1);
            let job = Job {
                recipe,
                herb_slot: 0,
                vessel_slot: 1,
            };
            let original = inv.clone();
            assert_eq!(
                job.complete(&mut inv, &Catalog::default(), &mut stats, true, true),
                Outcome::Level
            );
            assert_eq!(inv, original);
            stats.set_current(15, recipe.level as i8);
            let unchanged = stats.clone();
            assert_eq!(
                job.complete(&mut inv, &Catalog::default(), &mut stats, true, true),
                Outcome::Made
            );
            assert_eq!(inv.get(0).unwrap().id, recipe.output);
            assert_eq!(stats, unchanged);
        }
        for mode in 0..9 {
            let (mut symbols, mut defs) = fixture();
            match mode {
                0 => {
                    symbols.remove("guamvial");
                }
                1 => {
                    defs.items.remove(&91);
                }
                2 => defs.items.get_mut(&91).unwrap().stackable = true,
                3 => defs.items.get_mut(&249).unwrap().cert.template = 799,
                4 => defs.items.get_mut(&93).unwrap().placeholder.template = 14401,
                5 => defs.items.get_mut(&91).unwrap().dummy = 1,
                6 => defs.items.get_mut(&227).unwrap().members = true,
                7 => defs.items.get_mut(&249).unwrap().inventory_ops[0] = Some("Clean".into()),
                _ => {
                    symbols.insert("marrentillvial".into(), 91);
                }
            }
            assert!(unfinished_potions(&symbols, &defs).is_err(), "mode{mode}");
        }
    }
}

/// Three ordinary finished-potion profiles; runtime and equipment modifiers are separate.
/// Pinned Wiki/239.1 evidence: work/openrune-finished-potions-readiness-001.json.
pub fn finished_potions(
    objects: &Symbols,
    defs: &Definitions,
) -> Result<Vec<gielinor_world::finished_potions::Recipe>, String> {
    let mut seen = std::collections::BTreeMap::<i32, String>::new();
    let mut bind = |name: &str, members: bool, drink: bool| -> Result<i32, String> {
        let id = *objects
            .get(name)
            .ok_or_else(|| format!("Missing finished-potion symbol {name}"))?;
        let item = defs
            .items
            .get(&(id as u32))
            .ok_or_else(|| format!("Missing finished-potion definition {name}"))?;
        if id < 0
            || seen.insert(id, name.to_owned()).is_some_and(|previous| previous != name)
            || item.stackable
            || item.cert.template != 0
            || item.placeholder.template != 0
            || item.dummy == 1
            || item.members != members
            || (item.inventory_ops[0].as_deref() == Some("Drink")) != drink
            || item
                .inventory_ops
                .iter()
                .any(|op| op.as_deref() == Some("Clean"))
        {
            return Err(format!("Invalid ordinary finished-potion item {name}"));
        }
        Ok(id)
    };
    let mut result = Vec::new();
    for (unf, secondary, output, secondary_member, output_member, level, boostable, base_fine_xp) in [
        (
            "guamvial",
            "eye_of_newt",
            "3dose1attack",
            false,
            true,
            3,
            false,
            250,
        ),
        (
            "marrentillvial",
            "unicorn_horn_dust",
            "3doseantipoison",
            true,
            false,
            5,
            true,
            375,
        ),
        (
            "tarrominvial",
            "limpwurt_root",
            "3dose1strength",
            false,
            false,
            12,
            true,
            500,
        ),
        ("harralandervial", "red_spiders_eggs", "3dosestatrestore", false, true, 22, true, 625),
        ("harralandervial", "chocolate_dust", "3dose1energy", false, false, 26, true, 675),
        ("ranarrvial", "white_berries", "3dose1defense", true, true, 30, true, 750),
        ("toadflaxvial", "toads_legs", "3dose1agility", true, true, 34, true, 800),
        ("harralandervial", "ground_desert_goat_horn", "3dosecombat", true, true, 36, true, 840),
        ("ranarrvial", "snape_grass", "3doseprayerrestore", true, true, 38, true, 875),
        ("iritvial", "eye_of_newt", "3dose2attack", false, true, 45, true, 1000),
        ("iritvial", "unicorn_horn_dust", "3dose2antipoison", true, true, 48, true, 1063),
        ("avantoevial", "snape_grass", "3dosefisherspotion", true, true, 50, true, 1125),
        ("avantoevial", "mortmyremushroom", "3dose2energy", true, true, 52, true, 1175),
        ("avantoevial", "huntingbeast_sabreteeth_dust", "3dosehunting", true, true, 53, true, 1200),
        ("kwuarmvial", "limpwurt_root", "3dose2strength", false, true, 55, true, 1250),
        ("snapdragonvial", "red_spiders_eggs", "3dose2restore", false, true, 63, true, 1425),
        ("cadantinevial", "white_berries", "3dose2defense", true, true, 66, true, 1500),
        ("lantadymevial", "dragon_scale_dust", "3dose1antidragon", true, true, 69, true, 1575),
        ("dwarfweedvial", "wine_of_zamorak", "3doserangerspotion", false, true, 72, true, 1625),
        ("lantadymevial", "cactus_potato", "3dose1magic", true, true, 76, true, 1725),
        ("torstolvial", "jangerberries", "3dosepotionofzamorak", true, true, 78, true, 1750),
        ("toadflaxvial", "crushed_bird_nest", "3dosepotionofsaradomin", true, true, 81, true, 1800),
        ("huascavial", "aldarium", "3dose1prayer_regeneration", true, true, 58, true, 1320),
        ("harralandervial", "aldarium", "3dosegoading", true, true, 54, true, 1320),
        ("unfinished_antidote+", "yew_roots", "antidote+4", true, true, 68, true, 1550),
        ("unfinished_antidote++", "magic_roots", "antidote++4", true, true, 79, true, 1775),
        ("cadantine_bloodvial", "wine_of_zamorak", "3dosebastion", false, true, 80, true, 1550),
        ("cadantine_bloodvial", "cactus_potato", "3dosebattlemage", true, true, 80, true, 1550),
        ("dwarfweedvial", "lily_of_the_sands", "3dosestatrenewal", true, true, 88, true, 2000),
    ] {
        result.push(gielinor_world::finished_potions::Recipe {
            unfinished: bind(unf, true, false)?,
            secondary: bind(secondary, secondary_member, false)?,
            output: bind(output, output_member, true)?,
            level,
            boostable,
            base_fine_xp,
        });
    }
    // Weapon poison is an ordinary two-input Herblore product, but not a drink.
    // Keep the drink validation above intact and bind this output explicitly.
    let poison = bind("weapon_poison", true, false)?;
    if defs.items[&(poison as u32)].inventory_ops != [None, None, None, Some("Empty".into()), None] {
        return Err("Invalid weapon-poison output operations".into());
    }
    result.push(gielinor_world::finished_potions::Recipe {
        unfinished: bind("kwuarmvial", true, false)?,
        secondary: bind("dragon_scale_dust", true, false)?,
        output: poison,
        level: 60,
        boostable: true,
        base_fine_xp: 1375,
    });
    Ok(result)
}
#[cfg(test)]
mod finished_potion_binding_tests {
    use super::*;
    fn fixture() -> (Symbols, Definitions) {
        let mut symbols = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (name, id, member, drink) in [
            ("guamvial", 91, true, false),
            ("eye_of_newt", 221, false, false),
            ("3dose1attack", 121, true, true),
            ("marrentillvial", 93, true, false),
            ("unicorn_horn_dust", 235, true, false),
            ("3doseantipoison", 175, false, true),
            ("tarrominvial", 95, true, false),
            ("limpwurt_root", 225, false, false),
            ("3dose1strength", 115, false, true),
            ("unfinished_antidote+", 5942, true, false),
            ("yew_roots", 6049, true, false),
            ("antidote+4", 5943, true, true),
            ("unfinished_antidote++", 5951, true, false),
            ("magic_roots", 6051, true, false),
            ("antidote++4", 5952, true, true),
            ("cadantine_bloodvial", 22443, true, false),
            ("3dosebastion", 22464, true, true),
            ("3dosebattlemage", 22452, true, true),
            ("weapon_poison", 187, true, false),
            ("crushed_bird_nest", 6693, true, false),
            ("3dosepotionofsaradomin", 6687, true, true),
            ("huascavial", 30100, true, false),
            ("aldarium", 29993, true, false),
            ("3dose1prayer_regeneration", 30128, true, true),
            ("3dosegoading", 30140, true, true),
            ("snapdragonvial", 3004, true, false),
            ("3dose2restore", 3026, true, true),
            ("cadantinevial", 107, true, false),
            ("3dose2defense", 163, true, true),
            ("lantadymevial", 2483, true, false),
            ("dragon_scale_dust", 241, true, false),
            ("3dose1antidragon", 2454, true, true),
            ("dwarfweedvial", 109, true, false),
            ("wine_of_zamorak", 245, false, false),
            ("lily_of_the_sands", 27272, true, false),
            ("3dosestatrenewal", 27205, true, true),
            ("3doserangerspotion", 169, true, true),
            ("cactus_potato", 3138, true, false),
            ("3dose1magic", 3042, true, true),
            ("torstolvial", 111, true, false),
            ("jangerberries", 247, true, false),
            ("3dosepotionofzamorak", 189, true, true),
            ("iritvial", 101, true, false),
            ("3dose2attack", 145, true, true),
            ("3dose2antipoison", 181, true, true),
            ("avantoevial", 103, true, false),
            ("3dosefisherspotion", 151, true, true),
            ("mortmyremushroom", 2970, true, false),
            ("3dose2energy", 3018, true, true),
            ("huntingbeast_sabreteeth_dust", 10111, true, false),
            ("3dosehunting", 10000, true, true),
            ("kwuarmvial", 105, true, false),
            ("3dose2strength", 157, true, true),
            ("harralandervial", 97, true, false),
            ("red_spiders_eggs", 223, false, false),
            ("3dosestatrestore", 127, true, true),
            ("chocolate_dust", 1975, false, false),
            ("3dose1energy", 3010, false, true),
            ("ranarrvial", 99, true, false),
            ("white_berries", 239, true, false),
            ("3dose1defense", 133, true, true),
            ("toadflaxvial", 3002, true, false),
            ("toads_legs", 2152, true, false),
            ("3dose1agility", 3034, true, true),
            ("ground_desert_goat_horn", 9736, true, false),
            ("3dosecombat", 9741, true, true),
            ("snape_grass", 231, true, false),
            ("3doseprayerrestore", 139, true, true),
        ] {
            symbols.insert(name.into(), id);
            let mut item =
                gielinor_cache::definitions::ItemType::decode(if member { &[16, 0] } else { &[0] })
                    .unwrap();
            if drink {
                item.inventory_ops[0] = Some("Drink".into());
            }
            if id == 187 { item.inventory_ops[3] = Some("Empty".into()); }
            defs.items.insert(id as u32, item);
        }
        (symbols, defs)
    }
    #[test]
    fn exact_profile_awards_and_target_item_membership_are_independent_of_recipe_membership() {
        let (symbols, defs) = fixture();
        let recipes = finished_potions(&symbols, &defs).unwrap();
        assert_eq!(
            recipes
                .iter()
                .map(|r| (
                    r.unfinished,
                    r.secondary,
                    r.output,
                    r.level,
                    r.boostable,
                    r.base_fine_xp
                ))
                .collect::<Vec<_>>(),
            [
                (91, 221, 121, 3, false, 250),
                (93, 235, 175, 5, true, 375),
                (95, 225, 115, 12, true, 500),
                (97, 223, 127, 22, true, 625),
                (97, 1975, 3010, 26, true, 675),
                (99, 239, 133, 30, true, 750),
                (3002, 2152, 3034, 34, true, 800),
                (97, 9736, 9741, 36, true, 840),
                (99, 231, 139, 38, true, 875),
                (101, 221, 145, 45, true, 1000),
                (101, 235, 181, 48, true, 1063),
                (103, 231, 151, 50, true, 1125),
                (103, 2970, 3018, 52, true, 1175),
                (103, 10111, 10000, 53, true, 1200),
                (105, 225, 157, 55, true, 1250),
                (3004, 223, 3026, 63, true, 1425),
                (107, 239, 163, 66, true, 1500),
                (2483, 241, 2454, 69, true, 1575),
                (109, 245, 169, 72, true, 1625),
                (2483, 3138, 3042, 76, true, 1725),
                (111, 247, 189, 78, true, 1750),
                (3002, 6693, 6687, 81, true, 1800),
                (30100, 29993, 30128, 58, true, 1320),
                (97, 29993, 30140, 54, true, 1320),
                (5942, 6049, 5943, 68, true, 1550),
                (5951, 6051, 5952, 79, true, 1775),
                (22443, 245, 22464, 80, true, 1550),
                (22443, 3138, 22452, 80, true, 1550),
                (109, 27272, 27205, 88, true, 2000),
                (105, 241, 187, 60, true, 1375),
            ]
        );
        assert!(!defs.items[&175].members && !defs.items[&115].members);
        for mode in 0..10 {
            let (mut symbols, mut defs) = fixture();
            match mode {
                0 => {
                    symbols.remove("guamvial");
                }
                1 => {
                    defs.items.remove(&121);
                }
                2 => defs.items.get_mut(&221).unwrap().stackable = true,
                3 => defs.items.get_mut(&93).unwrap().cert.template = 799,
                4 => defs.items.get_mut(&175).unwrap().placeholder.template = 14401,
                5 => defs.items.get_mut(&115).unwrap().dummy = 1,
                6 => defs.items.get_mut(&175).unwrap().members = true,
                7 => defs.items.get_mut(&121).unwrap().inventory_ops[0] = None,
                8 => defs.items.get_mut(&235).unwrap().inventory_ops[0] = Some("Clean".into()),
                _ => {
                    symbols.insert("3doseantipoison".into(), 121);
                }
            }
            assert!(finished_potions(&symbols, &defs).is_err(), "mode{mode}");
        }
    }
    #[test]
    fn weapon_poison_has_an_explicit_non_drink_output_contract() {
        for mode in 0..9 {
            let (mut symbols, mut defs) = fixture();
            match mode {
                0 => { symbols.remove("weapon_poison"); }
                1 => defs.items.get_mut(&187).unwrap().inventory_ops[0] = Some("Drink".into()),
                2 => defs.items.get_mut(&187).unwrap().inventory_ops[0] = Some("Eat".into()),
                3 => defs.items.get_mut(&187).unwrap().inventory_ops[3] = None,
                4 => defs.items.get_mut(&187).unwrap().stackable = true,
                5 => defs.items.get_mut(&187).unwrap().members = false,
                6 => defs.items.get_mut(&187).unwrap().cert.template = 799,
                7 => defs.items.get_mut(&187).unwrap().placeholder.template = 14401,
                _ => defs.items.get_mut(&187).unwrap().dummy = 1,
            }
            assert!(finished_potions(&symbols, &defs).is_err(), "mode{mode}");
        }
    }

}

/// Two target-revision cosmetic variants of prescription goggles.
pub fn finished_potion_goggles(objects: &Symbols, defs: &Definitions) -> Result<[i32; 2], String> {
    let mut result = [0; 2];
    for (index, name) in ["mm_alchemist_hat", "mm_alchemist_hat_alt"].into_iter().enumerate() {
        let id = *objects.get(name).ok_or_else(|| format!("Missing goggles symbol {name}"))?;
        let item = defs.items.get(&(id as u32)).ok_or_else(|| format!("Missing goggles definition {name}"))?;
        if id < 0 || !item.members || item.stackable || item.cert.template != 0 || item.placeholder.template != 0
            || item.dummy == 1 || item.wear[0] != 0 || item.inventory_ops[1].as_deref() != Some("Wear") {
            return Err(format!("Invalid target goggles definition {name}"));
        }
        result[index] = id;
    }
    if result[0] == result[1] { return Err("Goggles variants alias".into()); }
    Ok(result)
}
#[cfg(test)]
mod finished_goggles_binding_tests {
    use super::*;
    #[test]
    fn goggles_require_distinct_ordinary_member_head_equipment() {
        for invalid in 0..10 {
            let mut names=Symbols::new();
            let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
            for (name,id) in [("mm_alchemist_hat",29974),("mm_alchemist_hat_alt",29976)] {
                names.insert(name.into(),id);
                let mut item=gielinor_cache::definitions::ItemType::decode(&[16,13,0,0]).unwrap();
                item.inventory_ops[1]=Some("Wear".into());defs.items.insert(id as u32,item);
            }
            match invalid {
                0=>{assert_eq!(finished_potion_goggles(&names,&defs).unwrap(),[29974,29976]);continue;}
                1=>{names.remove("mm_alchemist_hat");}
                2=>{names.insert("mm_alchemist_hat_alt".into(),29974);}
                3=>defs.items.get_mut(&29974).unwrap().members=false,
                4=>defs.items.get_mut(&29974).unwrap().stackable=true,
                5=>defs.items.get_mut(&29974).unwrap().cert.template=799,
                6=>defs.items.get_mut(&29974).unwrap().placeholder.template=14401,
                7=>defs.items.get_mut(&29974).unwrap().dummy=1,
                8=>defs.items.get_mut(&29974).unwrap().wear[0]=1,
                _=>defs.items.get_mut(&29974).unwrap().inventory_ops[1]=None,
            }
            assert!(finished_potion_goggles(&names,&defs).is_err(),"case{invalid}");
        }
    }
}

/// Wiki-backed herb/tar recipes with exact target item characteristics.
pub fn swamp_tar(objects:&Symbols,defs:&Definitions)->Result<Vec<gielinor_world::swamp_tar::Recipe>,String> {
    let mut bound=std::collections::BTreeMap::new();
    let mut ids=std::collections::BTreeSet::new();
    for (name,stackable,members) in [
        ("guam_leaf",false,true),
        ("swamp_tar",true,true),
        ("salamander_tar_green",true,true),
        ("pestle_and_mortar",false,true),
        ("ground_guam",false,true),
        ("marentill",false,true),
        ("salamander_tar_orange",true,true),
        ("tarromin",false,true),
        ("salamander_tar_red",true,true),
        ("harralander",false,true),
        ("salamander_tar_black",true,true),
    ] {
        let id=*objects.get(name).ok_or_else(||format!("Missing tar symbol {name}"))?;
        let item=defs.items.get(&(id as u32)).ok_or_else(||format!("Missing tar definition {name}"))?;
        if id<0 || !ids.insert(id) || item.stackable!=stackable || item.members!=members || item.cert.template!=0 || item.placeholder.template!=0 || item.dummy==1 {
            return Err(format!("Invalid tar definition {name}"));
        }
        bound.insert(name,id);
    }
    Ok(vec![
        gielinor_world::swamp_tar::Recipe{herb:bound["guam_leaf"],tar:bound["swamp_tar"],output:bound["salamander_tar_green"],tool:bound["pestle_and_mortar"],level:19,base_fine_xp:300},
        gielinor_world::swamp_tar::Recipe{herb:bound["ground_guam"],tar:bound["swamp_tar"],output:bound["salamander_tar_green"],tool:bound["pestle_and_mortar"],level:19,base_fine_xp:300},
        gielinor_world::swamp_tar::Recipe{herb:bound["marentill"],tar:bound["swamp_tar"],output:bound["salamander_tar_orange"],tool:bound["pestle_and_mortar"],level:31,base_fine_xp:425},
        gielinor_world::swamp_tar::Recipe{herb:bound["tarromin"],tar:bound["swamp_tar"],output:bound["salamander_tar_red"],tool:bound["pestle_and_mortar"],level:39,base_fine_xp:550},
        gielinor_world::swamp_tar::Recipe{herb:bound["harralander"],tar:bound["swamp_tar"],output:bound["salamander_tar_black"],tool:bound["pestle_and_mortar"],level:44,base_fine_xp:725},
    ])
}

#[cfg(test)]
mod swamp_tar_binding_tests {
    use super::*;
    fn fixture()->(Symbols,Definitions) {
        let mut symbols=Symbols::new();
        let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
        for (name,id,stackable) in [
            ("guam_leaf",249,false),
            ("swamp_tar",1939,true),
            ("salamander_tar_green",10142,true),
            ("pestle_and_mortar",233,false),
            ("ground_guam",6681,false),
            ("marentill",251,false),
            ("salamander_tar_orange",10143,true),
            ("tarromin",253,false),
            ("salamander_tar_red",10144,true),
            ("harralander",255,false),
            ("salamander_tar_black",10145,true),
        ] {
            symbols.insert(name.into(),id);
            let mut item=gielinor_cache::definitions::ItemType::decode(&[16,0]).unwrap();item.stackable=stackable;defs.items.insert(id as u32,item);
        }
        (symbols,defs)
    }
    #[test]
    fn exact_five_profiles_reject_aliases_and_invalid_stacks_or_templates() {
        let (symbols,defs)=fixture();let recipes=swamp_tar(&symbols,&defs).unwrap();
        assert_eq!(recipes.iter().map(|r|(r.herb,r.tar,r.output,r.tool,r.level,r.base_fine_xp)).collect::<Vec<_>>(),vec![(249,1939,10142,233,19,300),(6681,1939,10142,233,19,300),(251,1939,10143,233,31,425),(253,1939,10144,233,39,550),(255,1939,10145,233,44,725)]);
        for mode in 0..9 {
            let (mut symbols,mut defs)=fixture();
            match mode {
                0=>{symbols.remove("swamp_tar");},1=>{defs.items.remove(&233);},
                2=>defs.items.get_mut(&1939).unwrap().stackable=false,
                3=>defs.items.get_mut(&10142).unwrap().stackable=false,
                4=>defs.items.get_mut(&6681).unwrap().stackable=true,
                5=>defs.items.get_mut(&233).unwrap().members=false,
                6=>defs.items.get_mut(&10143).unwrap().cert.template=799,
                7=>defs.items.get_mut(&249).unwrap().placeholder.template=14401,
                _=>{symbols.insert("ground_guam".into(),249);},
            }
            assert!(swamp_tar(&symbols,&defs).is_err(),"mode{mode}");
        }
    }
}

/// Explicit target239.1 three-to-four dose mappings, Wiki chemistry rev15338661.
const CHEMISTRY_DOSES: [(&str,&str);28] = [
    ("3dose1attack", "4dose1attack"),
    ("3doseantipoison", "4doseantipoison"),
    ("3dose1strength", "strength4"),
    ("3dosestatrestore", "4dosestatrestore"),
    ("3dose1energy", "4dose1energy"),
    ("3dose1defense", "4dose1defense"),
    ("3dose1agility", "4dose1agility"),
    ("3dosecombat", "4dosecombat"),
    ("3doseprayerrestore", "4doseprayerrestore"),
    ("3dose2attack", "4dose2attack"),
    ("3dose2antipoison", "4dose2antipoison"),
    ("3dosefisherspotion", "4dosefisherspotion"),
    ("3dose2energy", "4dose2energy"),
    ("3dosehunting", "4dosehunting"),
    ("3dose2strength", "4dose2strength"),
    ("3dose2restore", "4dose2restore"),
    ("3dose2defense", "4dose2defense"),
    ("3dose1antidragon", "4dose1antidragon"),
    ("3doserangerspotion", "4doserangerspotion"),
    ("3dose1magic", "4dose1magic"),
    ("3dosepotionofzamorak", "4dosepotionofzamorak"),
    ("3dosepotionofsaradomin", "4dosepotionofsaradomin"),
    ("3dose1prayer_regeneration", "4dose1prayer_regeneration"),
    ("3dosegoading", "4dosegoading"),
    ("3doseancientbrew", "4doseancientbrew"),
    ("3dosestatrenewal", "4dosestatrenewal"),
    ("3dosebastion", "4dosebastion"),
    ("3dosebattlemage", "4dosebattlemage")
];
pub fn finished_potion_chemistry(objects:&Symbols,params:&Symbols,defs:&Definitions)->Result<std::collections::BTreeMap<i32,gielinor_world::chemistry_modifier::Binding>,String> {
    let id=*objects.get("amulet_of_chemistry").ok_or("Missing chemistry amulet symbol")?;
    let amulet=defs.items.get(&(id as u32)).ok_or("Missing chemistry amulet definition")?;
    let wear_op=*params.get("wear_op1").ok_or("Missing worn option parameter")? as u32;
    if amulet.params.get(&wear_op)!=Some(&gielinor_cache::definitions::Param::String("Options".into())){return Err("Invalid chemistry worn option".into());}
    let ordinary=|item:&gielinor_cache::definitions::ItemType| !item.stackable && item.cert.template==0 && item.placeholder.template==0 && item.dummy!=1;
    if id<0 || !ordinary(amulet) || !amulet.members || amulet.wear[0]!=2 || amulet.inventory_ops != [None,Some("Wear".into()),Some("Options".into()),Some("Break".into()),None] {return Err("Invalid ordinary chemistry amulet".into());}
    let mut seen=std::collections::BTreeSet::from([id]);let mut result=std::collections::BTreeMap::new();
    for (three,four) in CHEMISTRY_DOSES {
        let a=*objects.get(three).ok_or_else(||format!("Missing chemistry dose {three}"))?;
        let b=*objects.get(four).ok_or_else(||format!("Missing chemistry dose {four}"))?;
        if a<0 || b<0 || !seen.insert(a) || !seen.insert(b) {return Err("Aliased chemistry dose".into());}
        let x=defs.items.get(&(a as u32)).ok_or("Missing three-dose definition")?;
        let y=defs.items.get(&(b as u32)).ok_or("Missing four-dose definition")?;
        if !ordinary(x) || !ordinary(y) || x.members!=y.members || x.inventory_ops[0].as_deref()!=Some("Drink") || y.inventory_ops[0].as_deref()!=Some("Drink") || !x.name.ends_with("(3)") || x.name.strip_suffix("(3)")!=y.name.strip_suffix("(4)") {return Err(format!("Invalid chemistry dose pair {three}/{four}"));}
        result.insert(a,gielinor_world::chemistry_modifier::Binding{amulet:id,extra_output:b});
    }
    Ok(result)
}
#[cfg(test)]
mod chemistry_binding_tests {
    use super::*;
    fn fixture()->(Symbols,Definitions) {
        let mut names=Symbols::default();let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
        names.insert("amulet_of_chemistry".into(),21163);
        let mut amulet=gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();amulet.params.insert(451,gielinor_cache::definitions::Param::String("Options".into()));amulet.members=true;amulet.wear[0]=2;
        amulet.inventory_ops=[None,Some("Wear".into()),Some("Options".into()),Some("Break".into()),None];defs.items.insert(21163,amulet);
        for (index,(a,b)) in CHEMISTRY_DOSES.into_iter().enumerate() {
            for (dose,name) in [(3,a),(4,b)] {let id=(1000+index*2+(dose-3)) as i32;names.insert(name.into(),id);
                let mut item=gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();item.name=format!("Potion{index}({dose})");item.members=index!=1;item.inventory_ops[0]=Some("Drink".into());defs.items.insert(id as u32,item);
            }
        }
        (names,defs)
    }
    #[test]
    fn exact_named_pairs_reject_malformed_amulet_and_dose_definitions() {
        for mode in 0..18 {
            let (mut names,mut defs)=fixture();
            match mode {
                0=>{let map=finished_potion_chemistry(&names,&[("wear_op1".into(),451)].into(),&defs).unwrap();assert_eq!(map.len(),28);for i in 0..28{assert_eq!(map[&(1000+i*2)].extra_output,1001+i*2);}continue;},
                1=>defs.items.get_mut(&21163).unwrap().wear[0]=0,
                2=>defs.items.get_mut(&21163).unwrap().members=false,
                3=>defs.items.get_mut(&21163).unwrap().inventory_ops[3]=None,
                4=>defs.items.get_mut(&21163).unwrap().stackable=true,
                5=>defs.items.get_mut(&1001).unwrap().cert.template=799,
                6=>defs.items.get_mut(&1001).unwrap().placeholder.template=14401,
                7=>defs.items.get_mut(&1001).unwrap().dummy=1,
                8=>defs.items.get_mut(&1001).unwrap().stackable=true,
                9=>defs.items.get_mut(&1001).unwrap().name="Potion0(3)".into(),
                10=>defs.items.get_mut(&1001).unwrap().inventory_ops[0]=Some("Empty".into()),
                11=>defs.items.get_mut(&1001).unwrap().members=false,
                12=>{names.insert("4dose1attack".into(),1000);},
                13=>{names.insert("4dose1attack".into(),21163);},
                14=>{names.remove("strength4");},
                15=>{defs.items.remove(&1000);},
                16=>{defs.items.get_mut(&21163).unwrap().params.remove(&451);},
                _=>{defs.items.get_mut(&21163).unwrap().params.insert(451,gielinor_cache::definitions::Param::String("Charge".into()));},
            }
            assert!(finished_potion_chemistry(&names,&[("wear_op1".into(),451)].into(),&defs).is_err(),"mode{mode}");
        }
    }
    #[test]
    fn ordinary_reverse_links_survive_real_catalog_registration() {
        use gielinor_world::{finished_potions::{Job,Recipe,Outcome},inventory::{Inventory,Item,StackType,Template},chemistry::State};
        struct Proc;impl gielinor_world::npc::RandomSource for Proc{fn below(&mut self,bound:u32)->u32{assert_eq!(bound,100);0}}
        for invalid in 0..4 {
            let (names,mut defs)=fixture();
            for id in [1000,1001,21163] {let item=defs.items.get_mut(&id).unwrap();item.cert=Template{link:id as i32+100000,template:0};item.placeholder=Template{link:id as i32+200000,template:0};}
            let bindings=finished_potion_chemistry(&names,&[("wear_op1".into(),451)].into(),&defs).unwrap();
            match invalid {1=>defs.items.get_mut(&1001).unwrap().cert.template=799,2=>defs.items.get_mut(&1001).unwrap().placeholder.template=14401,3=>defs.items.get_mut(&1001).unwrap().transform=Template{link:1000,template:1},_=>()}
            let catalog=defs.inventory_catalog();let mut inv=Inventory::new(28,StackType::Normal,false);let mut equip=Inventory::new(14,StackType::Normal,false);
            inv.restore_slot(0,Some(Item{id:91,count:1,vars:0})).unwrap();inv.restore_slot(1,Some(Item{id:221,count:1,vars:0})).unwrap();equip.restore_slot(2,Some(Item{id:21163,count:1,vars:0})).unwrap();
            let mut stats=gielinor_world::stats::Stats::default();stats.set_base(15,99);stats.set_current(15,99);let mut state=State::default();
            let job=Job{recipe:Recipe{unfinished:91,secondary:221,output:1000,level:3,boostable:false,base_fine_xp:250},unfinished_slot:0,secondary_slot:1};
            let before=(inv.clone(),equip.clone(),stats.clone(),state);
            let result=job.complete_with_modifiers(&mut inv,&catalog,&mut stats,&mut equip,&mut state,true,true,500,false,Some(bindings[&1000]),&mut Proc);
            if invalid==0 {assert!(matches!(result.outcome,Outcome::Made(_)));assert_eq!(inv.get(0).unwrap().id,1001);assert_eq!(state.remaining(),4);assert_eq!(stats.fine_xp(15),1250);}
            else {assert_eq!(result.outcome,Outcome::InvalidRecipe);assert_eq!((inv,equip,stats,state),before);}
        }
    }
}

/// Wiki Stamina potion15281677: level77,25.5XP and one amylase per input dose.
/// This binding is not admitted until the upgrade runtime is configured.
pub fn stamina_upgrades(objects:&Symbols,defs:&Definitions)->Result<Vec<gielinor_world::potion_upgrades::Recipe>,String> {
    let mut ids=std::collections::BTreeSet::new();
    let mut bind=|name:&str,stackable:bool,dose:Option<u8>|->Result<i32,String>{
        let id=*objects.get(name).ok_or_else(||format!("Missing upgrade symbol {name}"))?;
        let item=defs.items.get(&(id as u32)).ok_or_else(||format!("Missing upgrade definition {name}"))?;
        if id<0 || !ids.insert(id) || item.stackable!=stackable || !item.members || item.cert.template!=0 || item.placeholder.template!=0 || item.dummy==1
            || dose.is_some_and(|n|item.inventory_ops[0].as_deref()!=Some("Drink") || !item.name.ends_with(&format!("({n})"))) {
            return Err(format!("Invalid upgrade definition {name}"));
        }
        Ok(id)
    };
    let secondary=bind("amylase",true,None)?;
    let mut recipes=Vec::new();
    for doses in 1..=4 {
        let base=bind(&format!("{doses}dose2energy"),false,Some(doses))?;
        let output=bind(&format!("{doses}dosestamina"),false,Some(doses))?;
        recipes.push(gielinor_world::potion_upgrades::Recipe{base,secondary,output,doses,secondary_per_dose:1,fine_xp_per_dose:255, fine_xp_override:None,level:77,ticks:2,allow_goggles:true});
    }
    Ok(recipes)
}
#[cfg(test)]
mod stamina_binding_tests {
    use super::*;
    fn fixture()->(Symbols,Definitions){
        let mut symbols=Symbols::new();let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
        let mut secondary=gielinor_cache::definitions::ItemType::decode(&[16,0]).unwrap();secondary.stackable=true;defs.items.insert(12640,secondary);symbols.insert("amylase".into(),12640);
        for n in 1..=4 {for (name,id) in [(format!("{n}dose2energy"),3024-2*n),(format!("{n}dosestamina"),12633-2*n)] {
            let mut item=gielinor_cache::definitions::ItemType::decode(&[16,0]).unwrap();item.name=format!("Potion({n})");item.inventory_ops[0]=Some("Drink".into());defs.items.insert(id as u32,item);symbols.insert(name,id);
        }}
        (symbols,defs)
    }
    #[test]
    fn dose_bindings_reject_missing_alias_noted_and_non_drink_definitions(){
        let (symbols,defs)=fixture();let recipes=stamina_upgrades(&symbols,&defs).unwrap();assert_eq!(recipes.len(),4);
        for (i,r) in recipes.iter().enumerate(){let n=i as i32+1;assert_eq!((r.base,r.secondary,r.output,r.doses,r.fine_xp_per_dose),(3024-2*n,12640,12633-2*n,n as u8,255));}
        for mode in 0..7 {
            let (mut symbols,mut defs)=fixture();
            match mode {0=>{symbols.remove("amylase");},1=>{symbols.insert("2dosestamina".into(),12631);},2=>defs.items.get_mut(&12631).unwrap().cert.template=799,3=>defs.items.get_mut(&12640).unwrap().stackable=false,4=>defs.items.get_mut(&3022).unwrap().inventory_ops[0]=None,5=>defs.items.get_mut(&12631).unwrap().name="Stamina(4)".into(),_=>defs.items.get_mut(&12631).unwrap().members=false}
            assert!(stamina_upgrades(&symbols,&defs).is_err(),"mode{mode}");
        }
    }
}

// Wiki Extended anti-venom+15267078 corrects donor secondary, XP and cadence.
pub fn extended_venom_upgrades(objects:&Symbols,defs:&Definitions)->Result<Vec<gielinor_world::potion_upgrades::Recipe>,String> {
    let mut ids=std::collections::BTreeSet::new();
    let mut bind=|name:&str,stackable:bool,dose:Option<u8>|->Result<i32,String>{
        let id=*objects.get(name).ok_or_else(||format!("Missing upgrade symbol {name}"))?;
        let item=defs.items.get(&(id as u32)).ok_or_else(||format!("Missing upgrade definition {name}"))?;
        if id<0 || !ids.insert(id) || item.stackable!=stackable || !item.members || item.cert.template!=0 || item.placeholder.template!=0 || item.dummy==1
            || dose.is_some_and(|n|item.inventory_ops[0].as_deref()!=Some("Drink") || !item.name.ends_with(&format!("({n})"))) {
            return Err(format!("Invalid upgrade definition {name}"));
        }
        Ok(id)
    };
    let secondary=bind("araxyte_venom_sack",true,None)?;
    let mut recipes=Vec::new();
    for doses in 1..=4 {
        let base=bind(&format!("antivenom+{doses}"),false,Some(doses))?;
        let output=bind(&format!("extended_antivenom+{doses}"),false,Some(doses))?;
        recipes.push(gielinor_world::potion_upgrades::Recipe{base,secondary,output,doses,secondary_per_dose:1,fine_xp_per_dose:200, fine_xp_override:None,level:94,ticks:3,allow_goggles:true});
    }
    Ok(recipes)
}

#[cfg(test)]
mod extended_venom_binding_tests {
    use super::*;
    #[test]
    fn actual_sacs_four_doses_twenty_xp_and_three_ticks_replace_donor_row(){
        let mut symbols=Symbols::new();let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
        let mut sac=gielinor_cache::definitions::ItemType::decode(&[16,0]).unwrap();sac.stackable=true;defs.items.insert(29784,sac);symbols.insert("araxyte_venom_sack".into(),29784);
        for n in 1..=4{for (name,id) in [(format!("antivenom+{n}"),12921-2*n),(format!("extended_antivenom+{n}"),29836-3*n)]{
            let mut item=gielinor_cache::definitions::ItemType::decode(&[16,0]).unwrap();item.name=format!("Potion({n})");item.inventory_ops[0]=Some("Drink".into());defs.items.insert(id as u32,item);symbols.insert(name,id);
        }}
        let recipes=extended_venom_upgrades(&symbols,&defs).unwrap();assert_eq!(recipes.len(),4);
        for (i,r) in recipes.iter().enumerate(){let n=i as i32+1;assert_eq!((r.base,r.secondary,r.output,r.doses,r.secondary_per_dose,r.fine_xp_per_dose,r.level,r.ticks),(12921-2*n,29784,29836-3*n,n as u8,1,200,94,3));}
        symbols.remove("araxyte_venom_sack");symbols.insert("snakeboss_scale".into(),29784);assert!(extended_venom_upgrades(&symbols,&defs).is_err());
    }
}

// Wiki divine recipes: ordinary dust23964,0.5XP/dose, no goggles saving.
pub fn divine_upgrades(objects:&Symbols,defs:&Definitions)->Result<Vec<gielinor_world::potion_upgrades::Recipe>,String> {
    let mut ids=std::collections::BTreeSet::new();
    let mut bind=|name:&str,stackable:bool,dose:Option<u8>|->Result<i32,String>{
        let id=*objects.get(name).ok_or_else(||format!("Missing upgrade symbol {name}"))?;
        let item=defs.items.get(&(id as u32)).ok_or_else(||format!("Missing upgrade definition {name}"))?;
        if id<0 || !ids.insert(id) || item.stackable!=stackable || !item.members || item.cert.template!=0 || item.placeholder.template!=0 || item.dummy==1
            || dose.is_some_and(|n|item.inventory_ops[0].as_deref()!=Some("Drink") || !item.name.ends_with(&format!("({n})"))) {
            return Err(format!("Invalid upgrade definition {name}"));
        }
        Ok(id)
    };
    let secondary=bind("prif_crystal_shard_crushed",true,None)?;
    let mut recipes=Vec::new();
    for (base_suffix,output_suffix,level) in [("2attack","divineattack",70),("2strength","divinestrength",70),("2defense","divinedefence",70),("rangerspotion","divinerange",74),("1magic","divinemagic",78),("bastion","divinebastion",86),("battlemage","divinebattlemage",86),("2combat","divinecombat",97)] {
    for doses in 1..=4 {
        let base=bind(&format!("{doses}dose{base_suffix}"),false,Some(doses))?;
        let output=bind(&format!("{doses}dose{output_suffix}"),false,Some(doses))?;
        recipes.push(gielinor_world::potion_upgrades::Recipe{base,secondary,output,doses,secondary_per_dose:1,fine_xp_per_dose:5, fine_xp_override:None,level,ticks:2,allow_goggles:false});
    }
    }
    Ok(recipes)
}

#[cfg(test)]
mod divine_binding_tests {
    use super::*;
    #[test]
    fn all_thirty_two_doses_use_ordinary_dust_fractional_xp_and_no_goggles(){
        let mut symbols=Symbols::new();let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
        for (name,id,label,stackable) in [
            ("prif_crystal_shard_crushed",23964,"Crystal dust",true),
            ("1dose2attack",149,"Super attack(1)",false),
            ("2dose2attack",147,"Super attack(2)",false),
            ("3dose2attack",145,"Super attack(3)",false),
            ("4dose2attack",2436,"Super attack(4)",false),
            ("1dosedivineattack",23706,"Divine super attack potion(1)",false),
            ("2dosedivineattack",23703,"Divine super attack potion(2)",false),
            ("3dosedivineattack",23700,"Divine super attack potion(3)",false),
            ("4dosedivineattack",23697,"Divine super attack potion(4)",false),
            ("1dose2strength",161,"Super strength(1)",false),
            ("2dose2strength",159,"Super strength(2)",false),
            ("3dose2strength",157,"Super strength(3)",false),
            ("4dose2strength",2440,"Super strength(4)",false),
            ("1dosedivinestrength",23718,"Divine super strength potion(1)",false),
            ("2dosedivinestrength",23715,"Divine super strength potion(2)",false),
            ("3dosedivinestrength",23712,"Divine super strength potion(3)",false),
            ("4dosedivinestrength",23709,"Divine super strength potion(4)",false),
            ("1dose2defense",167,"Super defence(1)",false),
            ("2dose2defense",165,"Super defence(2)",false),
            ("3dose2defense",163,"Super defence(3)",false),
            ("4dose2defense",2442,"Super defence(4)",false),
            ("1dosedivinedefence",23730,"Divine super defence potion(1)",false),
            ("2dosedivinedefence",23727,"Divine super defence potion(2)",false),
            ("3dosedivinedefence",23724,"Divine super defence potion(3)",false),
            ("4dosedivinedefence",23721,"Divine super defence potion(4)",false),
            ("1doserangerspotion",173,"Ranging potion(1)",false),
            ("2doserangerspotion",171,"Ranging potion(2)",false),
            ("3doserangerspotion",169,"Ranging potion(3)",false),
            ("4doserangerspotion",2444,"Ranging potion(4)",false),
            ("1dosedivinerange",23742,"Divine ranging potion(1)",false),
            ("2dosedivinerange",23739,"Divine ranging potion(2)",false),
            ("3dosedivinerange",23736,"Divine ranging potion(3)",false),
            ("4dosedivinerange",23733,"Divine ranging potion(4)",false),
            ("1dose1magic",3046,"Magic potion(1)",false),
            ("2dose1magic",3044,"Magic potion(2)",false),
            ("3dose1magic",3042,"Magic potion(3)",false),
            ("4dose1magic",3040,"Magic potion(4)",false),
            ("1dosedivinemagic",23754,"Divine magic potion(1)",false),
            ("2dosedivinemagic",23751,"Divine magic potion(2)",false),
            ("3dosedivinemagic",23748,"Divine magic potion(3)",false),
            ("4dosedivinemagic",23745,"Divine magic potion(4)",false),
            ("1dosebastion",22470,"Bastion potion(1)",false),
            ("2dosebastion",22467,"Bastion potion(2)",false),
            ("3dosebastion",22464,"Bastion potion(3)",false),
            ("4dosebastion",22461,"Bastion potion(4)",false),
            ("1dosedivinebastion",24644,"Divine bastion potion(1)",false),
            ("2dosedivinebastion",24641,"Divine bastion potion(2)",false),
            ("3dosedivinebastion",24638,"Divine bastion potion(3)",false),
            ("4dosedivinebastion",24635,"Divine bastion potion(4)",false),
            ("1dosebattlemage",22458,"Battlemage potion(1)",false),
            ("2dosebattlemage",22455,"Battlemage potion(2)",false),
            ("3dosebattlemage",22452,"Battlemage potion(3)",false),
            ("4dosebattlemage",22449,"Battlemage potion(4)",false),
            ("1dosedivinebattlemage",24632,"Divine battlemage potion(1)",false),
            ("2dosedivinebattlemage",24629,"Divine battlemage potion(2)",false),
            ("3dosedivinebattlemage",24626,"Divine battlemage potion(3)",false),
            ("4dosedivinebattlemage",24623,"Divine battlemage potion(4)",false),
            ("1dose2combat",12701,"Super combat potion(1)",false),
            ("2dose2combat",12699,"Super combat potion(2)",false),
            ("3dose2combat",12697,"Super combat potion(3)",false),
            ("4dose2combat",12695,"Super combat potion(4)",false),
            ("1dosedivinecombat",23694,"Divine super combat potion(1)",false),
            ("2dosedivinecombat",23691,"Divine super combat potion(2)",false),
            ("3dosedivinecombat",23688,"Divine super combat potion(3)",false),
            ("4dosedivinecombat",23685,"Divine super combat potion(4)",false),
        ] {
            let mut item=gielinor_cache::definitions::ItemType::decode(&[16,0]).unwrap();item.name=label.into();item.stackable=stackable;if !stackable{item.inventory_ops[0]=Some("Drink".into());}defs.items.insert(id as u32,item);symbols.insert(name.into(),id);
        }
        let recipes=divine_upgrades(&symbols,&defs).unwrap();assert_eq!(recipes.len(),32);
        for (i,r) in recipes.iter().enumerate(){assert_eq!((r.secondary,r.doses,r.secondary_per_dose,r.fine_xp_per_dose,r.ticks,r.allow_goggles),(23964,(i%4+1) as u8,1,5,2,false));assert_eq!(r.level,[70,70,70,74,78,86,86,97][i/4]);}
        let mut quest=gielinor_cache::definitions::ItemType::decode(&[16,0]).unwrap();quest.name="Crystal dust".into();quest.inventory_ops[4]=Some("Destroy".into());defs.items.insert(23804,quest);symbols.insert("prif_crystal_shard_crushed".into(),23804);assert!(divine_upgrades(&symbols,&defs).is_err());
    }
}

/// Wiki15281186: four-dose inputs, either Torstol form, always four-dose output.
pub fn combat_mixing_recipes(objects:&Symbols,defs:&Definitions)->Result<Vec<gielinor_world::combat_mixing::Recipe>,String> {
    let mut unique=std::collections::BTreeSet::new();
    let mut bind=|symbol:&str,name:&str,drink:bool|->Result<i32,String>{
        let id=*objects.get(symbol).ok_or_else(||format!("Missing combat ingredient {symbol}"))?;
        let item=defs.items.get(&(id as u32)).ok_or_else(||format!("Missing combat definition {symbol}"))?;
        if id<0||!unique.insert(id)||item.stackable||!item.members||item.cert.template!=0||item.placeholder.template!=0||item.dummy==1||item.name!=name||drink&&item.inventory_ops[0].as_deref()!=Some("Drink") {return Err(format!("Invalid combat ingredient {symbol}"))}
        Ok(id)
    };
    let attack=bind("4dose2attack","Super attack(4)",true)?;
    let strength=bind("4dose2strength","Super strength(4)",true)?;
    let defence=bind("4dose2defense","Super defence(4)",true)?;
    let herb=bind("torstol","Torstol",false)?;
    let unfinished=bind("torstolvial","Torstol potion (unf)",false)?;
    let output=bind("4dose2combat","Super combat potion(4)",true)?;
    Ok([herb,unfinished].into_iter().map(|secondary|gielinor_world::combat_mixing::Recipe{inputs:[attack,strength,defence,secondary],output}).collect())
}

#[cfg(test)]
mod combat_mixing_binding_tests {
    use super::*;
    #[test]
    fn binds_both_torstols_and_rejects_three_dose_or_transformed_items() {
        let rows=[("4dose2attack",2436,"Super attack(4)"),("4dose2strength",2440,"Super strength(4)"),("4dose2defense",2442,"Super defence(4)"),("torstol",269,"Torstol"),("torstolvial",111,"Torstol potion (unf)"),("4dose2combat",12695,"Super combat potion(4)")];
        for mode in 0..5 {
            let mut symbols=Symbols::new();let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
            for (symbol,id,name) in rows {let mut item=gielinor_cache::definitions::ItemType::decode(&[16,0]).unwrap();item.name=name.into();if symbol.starts_with("4dose"){item.inventory_ops[0]=Some("Drink".into());}defs.items.insert(id,item);symbols.insert(symbol.into(),id as i32);}
            match mode {1=>defs.items.get_mut(&2436).unwrap().name="Super attack(3)".into(),2=>defs.items.get_mut(&269).unwrap().stackable=true,3=>defs.items.get_mut(&111).unwrap().cert.template=799,4=>{symbols.remove("torstolvial");},_=>{}}
            let result=combat_mixing_recipes(&symbols,&defs);
            if mode==0 {let rs=result.unwrap();assert_eq!(rs.len(),2);assert_eq!(rs[0].inputs,[2436,2440,2442,269]);assert_eq!(rs[1].inputs,[2436,2440,2442,111]);assert_eq!(rs[0].output,12695);}else{assert!(result.is_err(),"mode{mode}");}
        }
    }
}

pub fn antivenom_upgrades(objects:&Symbols,defs:&Definitions)->Result<Vec<gielinor_world::potion_upgrades::Recipe>,String> {
    let mut ids=std::collections::BTreeSet::new();
    let mut bind=|name:&str,stackable:bool,dose:Option<u8>|->Result<i32,String>{
        let id=*objects.get(name).ok_or_else(||format!("Missing upgrade symbol {name}"))?;
        let item=defs.items.get(&(id as u32)).ok_or_else(||format!("Missing upgrade definition {name}"))?;
        if id<0 || !ids.insert(id) || item.stackable!=stackable || !item.members || item.cert.template!=0 || item.placeholder.template!=0 || item.dummy==1
            || dose.is_some_and(|n|item.inventory_ops[0].as_deref()!=Some("Drink") || !item.name.ends_with(&format!("({n})"))) {
            return Err(format!("Invalid upgrade definition {name}"));
        }
        Ok(id)
    };
    let secondary=bind("snakeboss_scale",true,None)?;
    let mut recipes=Vec::new();
    for doses in 1..=4 {
        let base=bind(&format!("antidote++{doses}"),false,Some(doses))?;
        let output=bind(&format!("antivenom{doses}"),false,Some(doses))?;
        recipes.push(gielinor_world::potion_upgrades::Recipe{base,secondary,output,doses,secondary_per_dose:5,fine_xp_per_dose:300, fine_xp_override:None,level:87,ticks:2,allow_goggles:true});
    }
    Ok(recipes)
}

pub fn extended_antifire_upgrades(objects:&Symbols,defs:&Definitions)->Result<Vec<gielinor_world::potion_upgrades::Recipe>,String> {
    let mut ids=std::collections::BTreeSet::new();
    let mut bind=|name:&str,stackable:bool,dose:Option<u8>|->Result<i32,String>{
        let id=*objects.get(name).ok_or_else(||format!("Missing upgrade symbol {name}"))?;
        let item=defs.items.get(&(id as u32)).ok_or_else(||format!("Missing upgrade definition {name}"))?;
        if id<0 || !ids.insert(id) || item.stackable!=stackable || !item.members || item.cert.template!=0 || item.placeholder.template!=0 || item.dummy==1
            || dose.is_some_and(|n|item.inventory_ops[0].as_deref()!=Some("Drink") || !item.name.ends_with(&format!("({n})"))) {
            return Err(format!("Invalid upgrade definition {name}"));
        }
        Ok(id)
    };
    let secondary=bind("lava_shard",true,None)?;
    let mut recipes=Vec::new();
    for doses in 1..=4 {
        let base=bind(&format!("{doses}dose1antidragon"),false,Some(doses))?;
        let output=bind(&format!("{doses}dose2antidragon"),false,Some(doses))?;
        recipes.push(gielinor_world::potion_upgrades::Recipe{base,secondary,output,doses,secondary_per_dose:1,fine_xp_per_dose:275, fine_xp_override:None,level:84,ticks:2,allow_goggles:true});
    }
    Ok(recipes)
}

#[cfg(test)]
mod compatible_upgrade_binding_tests {
    use super::*;
    #[test]
    fn antivenom_five_scales_per_dose_and_extended_antifire_fractional_xp() {
        let mut symbols=Symbols::new();let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
        for (symbol,id) in [("snakeboss_scale",12934),("lava_shard",11994)] {let mut v=gielinor_cache::definitions::ItemType::decode(&[16,0]).unwrap();v.stackable=true;defs.items.insert(id,v);symbols.insert(symbol.into(),id as i32);}
        for n in 1..=4 {for (symbol,id,name) in [(format!("antidote++{n}"),5960-2*n,"Antidote++"),(format!("antivenom{n}"),12913-2*n,"Anti-venom"),(format!("{n}dose1antidragon"),2460-2*n,"Antifire potion"),(format!("{n}dose2antidragon"),11959-2*n,"Extended antifire")] {
            let mut v=gielinor_cache::definitions::ItemType::decode(&[16,0]).unwrap();v.name=format!("{name}({n})");v.inventory_ops[0]=Some("Drink".into());defs.items.insert(id as u32,v);symbols.insert(symbol,id);
        }}
        for (family,cost,xp,level,secondary) in [(antivenom_upgrades(&symbols,&defs).unwrap(),5,300,87,12934),(extended_antifire_upgrades(&symbols,&defs).unwrap(),1,275,84,11994)] {
            assert_eq!(family.len(),4);for (i,r) in family.iter().enumerate(){assert_eq!((r.doses,r.secondary,r.secondary_per_dose,r.fine_xp_per_dose,r.level,r.ticks,r.allow_goggles),((i+1) as u8,secondary,cost,xp,level,2,true));}
        }
        defs.items.get_mut(&12934).unwrap().stackable=false;assert!(antivenom_upgrades(&symbols,&defs).is_err());
        defs.items.get_mut(&11953).unwrap().inventory_ops[0]=None;assert!(extended_antifire_upgrades(&symbols,&defs).is_err());
    }
}

pub fn forgotten_brew_upgrades(objects:&Symbols,defs:&Definitions)->Result<Vec<gielinor_world::potion_upgrades::Recipe>,String> {
    let mut ids=std::collections::BTreeSet::new();
    let mut bind=|name:&str,stackable:bool,dose:Option<u8>|->Result<i32,String>{
        let id=*objects.get(name).ok_or_else(||format!("Missing upgrade symbol {name}"))?;
        let item=defs.items.get(&(id as u32)).ok_or_else(||format!("Missing upgrade definition {name}"))?;
        if id<0 || !ids.insert(id) || item.stackable!=stackable || !item.members || item.cert.template!=0 || item.placeholder.template!=0 || item.dummy==1
            || dose.is_some_and(|n|item.inventory_ops[0].as_deref()!=Some("Drink") || !item.name.ends_with(&format!("({n})"))) {
            return Err(format!("Invalid upgrade definition {name}"));
        }
        Ok(id)
    };
    let secondary=bind("ancient_essence",true,None)?;
    let mut recipes=Vec::new();
    for doses in 1..=4 {
        let base=bind(&format!("{doses}doseancientbrew"),false,Some(doses))?;
        let output=bind(&format!("{doses}doseforgottenbrew"),false,Some(doses))?;
        recipes.push(gielinor_world::potion_upgrades::Recipe{base,secondary,output,doses,secondary_per_dose:20,fine_xp_per_dose:360, fine_xp_override:if doses==4{Some(1450)}else{None},level:91,ticks:3,allow_goggles:true});
    }
    Ok(recipes)
}


#[cfg(test)]
mod forgotten_brew_binding_tests {
    use super::*;
    #[test]
    fn forgotten_brew_four_doses_bind_to_exact_cache_types_and_nonlinear_awards() {
        let mut symbols=Symbols::new();let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
        let mut essence=gielinor_cache::definitions::ItemType::decode(&[16,0]).unwrap();essence.stackable=true;defs.items.insert(27616,essence);symbols.insert("ancient_essence".into(),27616);
        for n in 1..=4 {for (symbol,id,name) in [(format!("{n}doseancientbrew"),26348-2*n,"Ancient brew"),(format!("{n}doseforgottenbrew"),27641-3*n,"Forgotten brew")] {
            let mut v=gielinor_cache::definitions::ItemType::decode(&[16,0]).unwrap();v.name=format!("{name}({n})");v.inventory_ops[0]=Some("Drink".into());defs.items.insert(id as u32,v);symbols.insert(symbol,id);
        }}
        let recipes=forgotten_brew_upgrades(&symbols,&defs).unwrap();assert_eq!(recipes.len(),4);
        for (i,r) in recipes.iter().enumerate() {let n=i as i32+1;assert_eq!((r.base,r.secondary,r.output,r.doses,r.secondary_per_dose,r.level,r.ticks),(26348-2*n,27616,27641-3*n,n as u8,20,91,3));assert_eq!(r.base_fine_xp(),[360,720,1080,1450][i]);}
        defs.items.get_mut(&27629).unwrap().inventory_ops[0]=None;assert!(forgotten_brew_upgrades(&symbols,&defs).is_err());
        defs.items.get_mut(&27629).unwrap().inventory_ops[0]=Some("Drink".into());defs.items.get_mut(&27616).unwrap().stackable=false;assert!(forgotten_brew_upgrades(&symbols,&defs).is_err());
    }
}

/// Wiki Ancient brew15342079: one Nihil dust,85Herblore,190XP,three-dose output.
pub fn ancient_brew_recipe(objects:&Symbols,defs:&Definitions)->Result<gielinor_world::finished_potions::Recipe,String> {
    let mut seen=std::collections::BTreeSet::new();
    let mut bind=|name:&str,stackable:bool,drink:bool|->Result<i32,String>{
        let id=*objects.get(name).ok_or_else(||format!("Missing Ancient brew symbol {name}"))?;
        let item=defs.items.get(&(id as u32)).ok_or_else(||format!("Missing Ancient brew definition {name}"))?;
        if id<0 || !seen.insert(id) || item.stackable!=stackable || !item.members || item.cert.template!=0 || item.placeholder.template!=0 || item.dummy==1 || (item.inventory_ops[0].as_deref()==Some("Drink"))!=drink || item.inventory_ops.iter().any(|op|op.as_deref()==Some("Clean")) {
            return Err(format!("Invalid Ancient brew definition {name}"));
        }
        Ok(id)
    };
    Ok(gielinor_world::finished_potions::Recipe{unfinished:bind("dwarfweedvial",false,false)?,secondary:bind("nihil_dust",true,false)?,output:bind("3doseancientbrew",false,true)?,level:85,boostable:true,base_fine_xp:1900})
}

#[cfg(test)]
mod ancient_brew_binding_tests {
    use super::*;
    #[test]
    fn ancient_brew_requires_stackable_dust_ordinary_vial_and_drink_output() {
        let symbols:Symbols=[("dwarfweedvial".into(),109),("nihil_dust".into(),26368),("3doseancientbrew".into(),26342)].into();
        let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
        for id in [109,26368,26342] {let mut v=gielinor_cache::definitions::ItemType::decode(&[16,0]).unwrap();v.stackable=id==26368;if id==26342{v.inventory_ops[0]=Some("Drink".into());}defs.items.insert(id,v);}
        let recipe=ancient_brew_recipe(&symbols,&defs).unwrap();assert_eq!((recipe.unfinished,recipe.secondary,recipe.output,recipe.level,recipe.base_fine_xp),(109,26368,26342,85,1900));
        defs.items.get_mut(&26368).unwrap().stackable=false;assert!(ancient_brew_recipe(&symbols,&defs).is_err());
        defs.items.get_mut(&26368).unwrap().stackable=true;defs.items.get_mut(&26342).unwrap().inventory_ops[0]=None;assert!(ancient_brew_recipe(&symbols,&defs).is_err());
    }
}

/// Wiki Nihil dust15263916: tool retained,one shard to one dust,0/2/3ticks,noXP.
pub fn nihil_grinding(obj:&Symbols,seq:&Symbols,defs:&Definitions)->Result<gielinor_world::grinding::Recipe,String>{
    let mut seen=std::collections::BTreeSet::new();
    let mut bind=|name:&str,stackable:bool|->Result<i32,String>{
        let id=*obj.get(name).ok_or_else(||format!("Missing Nihil grinding symbol {name}"))?;
        let item=defs.items.get(&(id as u32)).ok_or_else(||format!("Missing Nihil grinding definition {name}"))?;
        if id<0 || !seen.insert(id) || item.stackable!=stackable || item.cert.template!=0 || item.placeholder.template!=0 || item.dummy==1 || (stackable && !item.members) {return Err(format!("Invalid Nihil grinding definition {name}"));}
        Ok(id)
    };
    let animation=*seq.get("human_herbing_grind").ok_or("Missing Nihil grinding animation")?;
    Ok(gielinor_world::grinding::Recipe{tool:bind("pestle_and_mortar",false)?,input:bind("nihil_shard",true)?,output:bind("nihil_dust",true)?,members:true,animation:Some(u16::try_from(animation).map_err(|_|"Invalid Nihil grinding animation")?)})
}

#[cfg(test)]
mod nihil_grinding_binding_tests {
    use super::*;
    #[test]
    fn nihil_shards_and_dust_must_be_normal_member_stacks() {
        let obj:Symbols=[("pestle_and_mortar".into(),233),("nihil_shard".into(),26231),("nihil_dust".into(),26368)].into();
        let seq:Symbols=[("human_herbing_grind".into(),364)].into();
        let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
        for id in [233,26231,26368] {let mut v=gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();v.stackable=id!=233;v.members=id!=233;defs.items.insert(id,v);}
        let recipe=nihil_grinding(&obj,&seq,&defs).unwrap();assert_eq!((recipe.tool,recipe.input,recipe.output,recipe.members,recipe.animation),(233,26231,26368,true,Some(364)));
        defs.items.get_mut(&26231).unwrap().stackable=false;assert!(nihil_grinding(&obj,&seq,&defs).is_err());
        defs.items.get_mut(&26231).unwrap().stackable=true;defs.items.get_mut(&26368).unwrap().members=false;assert!(nihil_grinding(&obj,&seq,&defs).is_err());
    }
}

/// Wiki Anti-venom+15243456 explicitly attests four-dose input/output and either Torstol.
pub fn antivenom_plus_recipes(objects:&Symbols,defs:&Definitions)->Result<Vec<gielinor_world::finished_potions::Recipe>,String>{
    let mut seen=std::collections::BTreeSet::new();
    let mut bind=|symbol:&str,name:&str,drink:bool|->Result<i32,String>{
        let id=*objects.get(symbol).ok_or_else(||format!("Missing Anti-venom+ symbol {symbol}"))?;
        let item=defs.items.get(&(id as u32)).ok_or_else(||format!("Missing Anti-venom+ definition {symbol}"))?;
        if id<0 || !seen.insert(id) || item.name!=name || item.stackable || !item.members || item.cert.template!=0 || item.placeholder.template!=0 || item.dummy==1 || (item.inventory_ops[0].as_deref()==Some("Drink"))!=drink || item.inventory_ops.iter().any(|op|op.as_deref()==Some("Clean")) {return Err(format!("Invalid Anti-venom+ definition {symbol}"));}
        Ok(id)
    };
    let base=bind("antivenom4","Anti-venom(4)",true)?;
    let output=bind("antivenom+4","Anti-venom+(4)",true)?;
    let clean=bind("torstol","Torstol",false)?;let unfinished=bind("torstolvial","Torstol potion (unf)",false)?;
    Ok([clean,unfinished].into_iter().map(|secondary|gielinor_world::finished_potions::Recipe{unfinished:base,secondary,output,level:94,boostable:true,base_fine_xp:1250}).collect())
}
#[cfg(test)]
mod antivenom_plus_binding_tests {
    use super::*;
    #[test]
    fn antivenom_plus_requires_four_doses_and_accepts_both_torstols() {
        let mut symbols=Symbols::new();let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
        for (symbol,id,name,drink) in [("antivenom4",12905,"Anti-venom(4)",true),("antivenom+4",12913,"Anti-venom+(4)",true),("torstol",269,"Torstol",false),("torstolvial",111,"Torstol potion (unf)",false)] {let mut v=gielinor_cache::definitions::ItemType::decode(&[16,0]).unwrap();v.name=name.into();if drink{v.inventory_ops[0]=Some("Drink".into());}defs.items.insert(id,v);symbols.insert(symbol.into(),id as i32);}
        let recipes=antivenom_plus_recipes(&symbols,&defs).unwrap();assert_eq!(recipes.len(),2);
        for (i,r) in recipes.iter().enumerate(){assert_eq!((r.unfinished,r.secondary,r.output,r.level,r.base_fine_xp),(12905,[269,111][i],12913,94,1250));}
        defs.items.get_mut(&12905).unwrap().name="Anti-venom(3)".into();assert!(antivenom_plus_recipes(&symbols,&defs).is_err());
        defs.items.get_mut(&12905).unwrap().name="Anti-venom(4)".into();defs.items.get_mut(&269).unwrap().inventory_ops[0]=Some("Clean".into());assert!(antivenom_plus_recipes(&symbols,&defs).is_err());
    }
}


/// Three manual ordinary-fire profiles. Explicit pinned Wiki rules take precedence
/// over older enriched-cache parameters; quantities and special surfaces are separate.
pub fn manual_fire_cooking(obj:&Symbols,loc:&Symbols,seq:&Symbols,defs:&Definitions,skill:u8)->Result<gielinor_world::cooking::Config,String>{
 use gielinor_world::cooking::{Config,Recipe};
 let resolve=|s:&Symbols,n:&str|s.get(n).copied().filter(|v|*v>=0).ok_or_else(||format!("Missing cooking symbol {n}"));
 let stat=defs.stats.get(&(skill as u32)).ok_or("Missing Cooking stat")?;
 if stat.name!="Cooking" || stat.max_level!=99 || stat.min_level!=1 || stat.unreleased{return Err("Unattested Cooking stat".into());}
 let fire=resolve(loc,"fire")? as u32;let f=defs.locations.get(&fire).ok_or("Missing ordinary fire")?;
 if f.name!="Fire" || !f.multi.is_empty() || f.collision.width!=1 || f.collision.length!=1 {return Err("Unattested ordinary fire".into());}
 let animation=u16::try_from(resolve(seq,"human_firecooking")?).map_err(|_|"Invalid cooking animation")?;
 let mut recipes=BTreeMap::new();let mut ids=std::collections::BTreeSet::new();
 for (raw,cooked,burnt,raw_name,cooked_name,burnt_name,ticks) in [
  ("raw_chicken","cooked_chicken","burnt_chicken","Raw chicken","Cooked chicken","Burnt chicken",1),
  ("raw_beef","cooked_meat","burnt_meat","Raw beef","Cooked meat","Burnt meat",1),
  ("raw_shrimp","shrimp","burnt_shrimp","Raw shrimps","Shrimps","Burnt shrimp",4),
 ] {
  let raw=resolve(obj,raw)?;let cooked=resolve(obj,cooked)?;let burnt=resolve(obj,burnt)?;
  for(id,name) in [(raw,raw_name),(cooked,cooked_name),(burnt,burnt_name)]{
   let d=defs.items.get(&(id as u32)).ok_or("Missing cooking item")?;
   if !ids.insert(id) || d.name!=name || d.members || d.stackable || d.dummy==1 || d.cert.template!=0 || d.placeholder.template!=0 || d.transform.template!=0 {return Err("Unattested or aliased cooking item".into());}
  }
  if defs.items[&(cooked as u32)].inventory_ops[0].as_deref()!=Some("Eat"){return Err("Cooked food lacks Eat".into());}
  recipes.insert(raw,Recipe{fine_xp_override:None,raw,cooked,burnt,level:1,stop_burn: Some(34),low:128,high:512,xp:30,ticks});
 }
 Ok(Config{gauntlets: None, no_burn_cape: None, member_inputs:Default::default(),fire,animation,skill,max_level:stat.max_level,recipes,tutorial:None,surfaces:Default::default()})
}
#[cfg(test)]
mod cooking_binding_tests{
 use super::*;
 #[test]fn binds_exact_fire_profiles_and_rejects_late_aliases_and_bad_stat(){
  let mut obj=Symbols::new();let loc=Symbols::from([("fire".into(),26185)]);let seq=Symbols::from([("human_firecooking".into(),897)]);
  let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
  defs.stats.insert(7,gielinor_cache::definitions::StatType{name:"Cooking".into(),unreleased:false,max_level:99,min_level:1});
  defs.locations.insert(26185,gielinor_cache::locations::LocationType::decode(b"\x02Fire\0\0").unwrap());
  for(symbol,id,name,eat) in [("raw_chicken",2138,"Raw chicken",false),("cooked_chicken",2140,"Cooked chicken",true),("burnt_chicken",2144,"Burnt chicken",false),("raw_beef",2132,"Raw beef",false),("cooked_meat",2142,"Cooked meat",true),("burnt_meat",2146,"Burnt meat",false),("raw_shrimp",317,"Raw shrimps",false),("shrimp",315,"Shrimps",true),("burnt_shrimp",7954,"Burnt shrimp",false)]{
   obj.insert(symbol.into(),id);let mut d=gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();d.name=name.into();if eat{d.inventory_ops[0]=Some("Eat".into());}defs.items.insert(id as u32,d);
  }
  let c=manual_fire_cooking(&obj,&loc,&seq,&defs,7).unwrap();assert_eq!(c.recipes.len(),3);assert_eq!((c.recipes[&2138].ticks,c.recipes[&2132].ticks,c.recipes[&317].ticks),(1,1,4));assert_eq!(c.recipes[&317].burnt,7954);
  defs.items.get_mut(&7954).unwrap().cert.template=799;assert!(manual_fire_cooking(&obj,&loc,&seq,&defs,7).is_err());defs.items.get_mut(&7954).unwrap().cert.template=0;
  obj.insert("burnt_shrimp".into(),2144);assert!(manual_fire_cooking(&obj,&loc,&seq,&defs,7).is_err());obj.insert("burnt_shrimp".into(),7954);
  defs.stats.get_mut(&7).unwrap().max_level=126;assert!(manual_fire_cooking(&obj,&loc,&seq,&defs,7).is_err());
 }
}


/// Wiki-pinned F2P fish, using the existing manual cooking adapter. Shared burnt
/// fish definitions are intentional; raw/output aliases and partial publication are not.
pub fn additional_fire_fish(
    obj: &Symbols,
    defs: &Definitions,
    config: &mut gielinor_world::cooking::Config,
) -> Result<(), String> {
    use gielinor_world::cooking::Recipe;
    let mut next = config.recipes.clone();
    let mut raw_ids: std::collections::BTreeSet<_> = next.keys().copied().collect();
    let mut output_ids: std::collections::BTreeSet<_> =
        next.values().flat_map(|r| [r.cooked, r.burnt]).collect();
    let resolve = |name: &str| {
        obj.get(name)
            .copied()
            .filter(|v| *v >= 0)
            .ok_or_else(|| format!("Missing fire-fish symbol {name}"))
    };
    for (
        raw,
        cooked,
        burnt,
        raw_name,
        cooked_name,
        burnt_name,
        level,
        xp,
        stop,
        low,
        high,
        ticks,
    ) in [
        (
            "raw_anchovies",
            "anchovies",
            "burntfish1",
            "Raw anchovies",
            "Anchovies",
            "Burnt fish",
            1,
            30,
            34,
            128,
            512,
            4,
        ),
        (
            "raw_sardine",
            "sardine",
            "burntfish5",
            "Raw sardine",
            "Sardine",
            "Burnt fish",
            1,
            40,
            38,
            118,
            492,
            1,
        ),
        (
            "raw_herring",
            "herring",
            "burntfish3",
            "Raw herring",
            "Herring",
            "Burnt fish",
            5,
            50,
            41,
            108,
            472,
            4,
        ),
        (
            "raw_trout",
            "trout",
            "burntfish2",
            "Raw trout",
            "Trout",
            "Burnt fish",
            15,
            70,
            49,
            88,
            432,
            4,
        ),
        (
            "raw_pike",
            "pike",
            "burntfish2",
            "Raw pike",
            "Pike",
            "Burnt fish",
            20,
            80,
            54,
            78,
            412,
            1,
        ),
        (
            "raw_salmon",
            "salmon",
            "burntfish2",
            "Raw salmon",
            "Salmon",
            "Burnt fish",
            25,
            90,
            58,
            68,
            392,
            1,
        ),
        (
            "raw_tuna",
            "tuna",
            "burntfish4",
            "Raw tuna",
            "Tuna",
            "Burnt fish",
            30,
            100,
            63,
            58,
            372,
            4,
        ),
        (
            "raw_lobster",
            "lobster",
            "burnt_lobster",
            "Raw lobster",
            "Lobster",
            "Burnt lobster",
            40,
            120,
            74,
            38,
            332,
            4,
        ),
        (
            "raw_swordfish",
            "swordfish",
            "burnt_swordfish",
            "Raw swordfish",
            "Swordfish",
            "Burnt swordfish",
            45,
            140,
            86,
            18,
            292,
            4,
        ),
    ] {
        let raw = resolve(raw)?;
        let cooked = resolve(cooked)?;
        let burnt = resolve(burnt)?;
        if raw == cooked
            || raw == burnt
            || cooked == burnt
            || output_ids.contains(&raw)
            || !raw_ids.insert(raw)
            || raw_ids.contains(&cooked)
            || raw_ids.contains(&burnt)
        {
            return Err("Aliased fire-fish recipe".into());
        }
        for (id, name) in [(raw, raw_name), (cooked, cooked_name), (burnt, burnt_name)] {
            let d = defs
                .items
                .get(&(id as u32))
                .ok_or("Missing fire-fish definition")?;
            if d.name != name
                || d.members
                || d.stackable
                || d.dummy == 1
                || d.cert.template != 0
                || d.placeholder.template != 0
                || d.transform.template != 0
            {
                return Err(format!("Unattested ordinary fire-fish item {name}"));
            }
        }
        if defs.items[&(cooked as u32)].inventory_ops[0].as_deref() != Some("Eat") {
            return Err("Fire-fish cooked output lacks Eat".into());
        }
        output_ids.extend([cooked, burnt]);
        next.insert(
            raw,
            Recipe {fine_xp_override:None,
                raw,
                cooked,
                burnt,
                level,
                stop_burn: Some(stop),
                low,
                high,
                xp,
                ticks,
            },
        );
    }
    config.recipes = next;
    Ok(())
}

#[cfg(test)]
mod fire_fish_binding_tests {
    use super::*;
    use gielinor_world::{
        cooking::Config,
        inventory::{Catalog, Inventory, Item, StackType},
        npc::RandomSource,
        stats::Stats,
    };
    struct Draw(u32);
    impl RandomSource for Draw {
        fn below(&mut self, n: u32) -> u32 {
            assert_eq!(n, 256);
            self.0
        }
    }
    #[test]
    fn fish_bind_real_shared_outputs_award_selected_xp_and_rollback_late_bad_metadata() {
        let mut obj = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (symbol, id, name, eat) in [
            ("raw_anchovies", 321, "Raw anchovies", false),
            ("anchovies", 319, "Anchovies", true),
            ("burntfish1", 323, "Burnt fish", false),
            ("raw_sardine", 327, "Raw sardine", false),
            ("sardine", 325, "Sardine", true),
            ("raw_herring", 345, "Raw herring", false),
            ("herring", 347, "Herring", true),
            ("burntfish2", 343, "Burnt fish", false),
            ("raw_trout", 335, "Raw trout", false),
            ("trout", 333, "Trout", true),
            ("burntfish3", 357, "Burnt fish", false),
            ("raw_pike", 349, "Raw pike", false),
            ("pike", 351, "Pike", true),
            ("raw_salmon", 331, "Raw salmon", false),
            ("salmon", 329, "Salmon", true),
            ("raw_tuna", 359, "Raw tuna", false),
            ("tuna", 361, "Tuna", true),
            ("burntfish4", 367, "Burnt fish", false),
            ("burntfish5", 369, "Burnt fish", false),
            ("raw_lobster", 377, "Raw lobster", false),
            ("lobster", 379, "Lobster", true),
            ("burnt_lobster", 381, "Burnt lobster", false),
            ("raw_swordfish", 371, "Raw swordfish", false),
            ("swordfish", 373, "Swordfish", true),
            ("burnt_swordfish", 375, "Burnt swordfish", false),
        ] {
            obj.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            d.name = name.into();
            if eat {
                d.inventory_ops[0] = Some("Eat".into());
            }
            defs.items.insert(id as u32, d);
        }
        let empty = Config {
            gauntlets: None, no_burn_cape: None, member_inputs:Default::default(),
            surfaces:Default::default(),
            fire: 26185,
            animation: 897,
            skill: 7,
            max_level: 99,
            recipes: BTreeMap::new(),
            tutorial: None,
        };
        let mut c = empty.clone();
        additional_fire_fish(&obj, &defs, &mut c).unwrap();
        assert_eq!(c.recipes.len(), 9);
        // The five variants share a display name. These exact Wiki IDs must not
        // be inferred from the donor's ordinal alias or from the chosen recipe.
        for (raw,burnt) in [(321,323),(327,369),(345,357),(335,343),(349,343),(331,343),(359,367),(377,381),(371,375)] {
            assert_eq!(c.recipes[&raw].burnt,burnt);
        }

        assert_eq!(
            (c.recipes[&349].stop_burn, c.recipes[&371].stop_burn),
            (Some(54), Some(86))
        );

        // The real startup range adapter must inherit the corrected variants.
        // Other prerequisite metadata is isolated here; earlier contracts attest it.
        let mut range_config=c.clone();let mut range_objects=obj.clone();
        for(symbol,raw)in[("raw_chicken",2138),("raw_beef",2132),("raw_shrimp",317),("uncooked_stew",2001)]{
            range_objects.insert(symbol.into(),raw);let mut r=c.recipes[&321];r.raw=raw;r.cooked=10000+raw;r.burnt=20000+raw;range_config.recipes.insert(raw,r);
        }
        for(symbol,id,name)in[("bread_dough",2307,"Bread dough"),("bread",2309,"Bread"),("burnt_bread",2311,"Burnt bread")]{range_objects.insert(symbol.into(),id);let mut d=gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();d.name=name.into();if id==2309{d.inventory_ops[0]=Some("Eat".into());}defs.items.insert(id as u32,d);}
        let mut range=gielinor_cache::locations::LocationType::decode(&[0]).unwrap();range.name="Range".into();range.collision.length=2;range.ops[0]=Some("Cook".into());defs.locations.insert(26181,range);
        standard_cooking_range(&range_objects,&Symbols::from([("range".into(),26181),("newbierange".into(),9736)]),&Symbols::from([("human_cooking".into(),896)]),&defs,&mut range_config).unwrap();
        for(raw,burnt)in[(321,323),(327,369),(345,357),(335,343),(349,343),(331,343),(359,367),(377,381),(371,375)]{assert_eq!(range_config.recipe(26181,raw).unwrap().burnt,burnt);assert_eq!(range_config.recipe(26185,raw).unwrap().burnt,burnt);}
        for recipe in c.recipes.values() {
            for burned in [false, true] {
                let mut inv = Inventory::new(2, StackType::Normal, false);
                for slot in 0..2 {
                    inv.restore_slot(
                        slot,
                        Some(Item {
                            id: recipe.raw,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                let mut stats = Stats::default();
                stats.set_current(7, recipe.level as i8);
                let outcome = recipe
                    .complete(
                        1,
                        &mut inv,
                        &Catalog::default(),
                        &mut stats,
                        7,
                        99,
                        125,
                        &mut Draw(if burned { 255 } else { 0 }),
                    )
                    .unwrap();
                assert_eq!(outcome.burned, burned);
                assert_eq!(inv.get(0).unwrap().id, recipe.raw);
                assert_eq!(
                    inv.get(1).unwrap().id,
                    if burned { recipe.burnt } else { recipe.cooked }
                );
                assert_eq!(
                    stats.fine_xp(7),
                    if burned {
                        0
                    } else {
                        i32::from(recipe.xp) * 125 / 10
                    }
                );
            }
        }
        let mut c = empty.clone();
        defs.items.get_mut(&375).unwrap().members = true;
        assert!(additional_fire_fish(&obj, &defs, &mut c).is_err());
        assert!(c.recipes.is_empty());
        defs.items.get_mut(&375).unwrap().members = false;
        obj.insert("raw_swordfish".into(), 323);
        assert!(additional_fire_fish(&obj, &defs, &mut c).is_err());
        assert!(c.recipes.is_empty());
    }
}

/// Bowl-contained preparation and manual fire cooking. Other stew/curry effects
/// and automatic quantities are separate from these ordinary recipes.
pub struct StewBindings {
    pub assembly: Vec<gielinor_world::assembly::Recipe>,
    pub cooking: gielinor_world::cooking::Recipe,
}
pub fn stew_recipes(obj: &Symbols, defs: &Definitions, skill: u8) -> Result<StewBindings, String> {
    let stat = defs
        .stats
        .get(&(skill as u32))
        .ok_or("Missing Cooking stat for stew")?;
    if stat.name != "Cooking" || stat.max_level != 99 || stat.min_level != 1 || stat.unreleased {
        return Err("Unattested stew Cooking stat".into());
    }
    let mut ids = BTreeMap::new();
    for (symbol, name) in [
        ("bowl_water", "Bowl of water"),
        ("potato", "Potato"),
        ("cooked_meat", "Cooked meat"),
        ("cooked_chicken", "Cooked chicken"),
        ("stew1", "Incomplete stew"),
        ("stew2", "Incomplete stew"),
        ("uncooked_stew", "Uncooked stew"),
        ("stew", "Stew"),
        ("burnt_stew", "Burnt stew"),
    ] {
        let id = *obj.get(symbol).ok_or("Missing stew object symbol")?;
        let d = u32::try_from(id)
            .ok()
            .and_then(|v| defs.items.get(&v))
            .ok_or("Missing stew object definition")?;
        if ids.values().any(|v| *v == id)
            || d.name != name
            || d.members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err("Unattested or aliased stew object".into());
        }
        ids.insert(symbol, id);
    }
    if defs.items[&(ids["stew"] as u32)].inventory_ops[0].as_deref() != Some("Eat") {
        return Err("Stew output lacks Eat".into());
    }
    let assembly = [
        ("bowl_water", "potato", "stew1"),
        ("bowl_water", "cooked_meat", "stew2"),
        ("bowl_water", "cooked_chicken", "stew2"),
        ("stew1", "cooked_meat", "uncooked_stew"),
        ("stew1", "cooked_chicken", "uncooked_stew"),
        ("stew2", "potato", "uncooked_stew"),
    ]
    .into_iter()
    .map(|(a, b, c)| gielinor_world::assembly::Recipe {
        first: ids[a],
        second: ids[b],
        output: ids[c],
        skill,
        level: 25,
        base_xp: 0,
        members: false,
    })
    .collect();
    Ok(StewBindings {
        assembly,
        cooking: gielinor_world::cooking::Recipe {fine_xp_override:None,
            raw: ids["uncooked_stew"],
            cooked: ids["stew"],
            burnt: ids["burnt_stew"],
            level: 25,
            stop_burn: Some(58),
            low: 68,
            high: 392,
            xp: 117,
            ticks: 1,
        },
    })
}
#[cfg(test)]
mod stew_binding_tests {
    use super::*;
    use gielinor_world::{
        inventory::{Catalog, Inventory, Item, StackType},
        npc::RandomSource,
        stats::Stats,
    };
    struct Draw(u32);
    impl RandomSource for Draw {
        fn below(&mut self, n: u32) -> u32 {
            assert_eq!(n, 256);
            self.0
        }
    }
    #[test]
    fn stew_all_orders_live_level_no_assembly_xp_and_burn_or117_cooking_are_atomic() {
        let mut obj = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        defs.stats.insert(
            7,
            gielinor_cache::definitions::StatType {
                name: "Cooking".into(),
                unreleased: false,
                max_level: 99,
                min_level: 1,
            },
        );
        for (symbol, id, name) in [
            ("bowl_water", 1921, "Bowl of water"),
            ("potato", 1942, "Potato"),
            ("cooked_meat", 2142, "Cooked meat"),
            ("cooked_chicken", 2140, "Cooked chicken"),
            ("stew1", 1997, "Incomplete stew"),
            ("stew2", 1999, "Incomplete stew"),
            ("uncooked_stew", 2001, "Uncooked stew"),
            ("stew", 2003, "Stew"),
            ("burnt_stew", 2005, "Burnt stew"),
        ] {
            obj.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            d.name = name.into();
            if symbol == "stew" {
                d.inventory_ops[0] = Some("Eat".into());
            }
            defs.items.insert(id as u32, d);
        }
        let bindings = stew_recipes(&obj, &defs, 7).unwrap();
        assert_eq!(bindings.assembly.len(), 6);
        for recipe in bindings.assembly {
            let mut inv = Inventory::new(2, StackType::Normal, false);
            for (slot, id) in [(0, recipe.first), (1, recipe.second)] {
                inv.restore_slot(
                    slot,
                    Some(Item {
                        id,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
            let mut stats = Stats::default();
            stats.set_base(7, 24);
            stats.set_current(7, 24);
            let before = inv.clone();
            let job = gielinor_world::assembly::Job {
                recipe,
                first_slot: 0,
                second_slot: 1,
            };
            assert_eq!(
                job.complete_awarding(&mut inv, &Catalog::default(), &mut stats, 500, false)
                    .0,
                gielinor_world::assembly::Outcome::Level
            );
            assert_eq!(inv, before);
            stats.set_current(7, 25);
            let before_stats = stats.clone();
            assert_eq!(
                job.complete_awarding(&mut inv, &Catalog::default(), &mut stats, 500, false)
                    .0,
                gielinor_world::assembly::Outcome::Made
            );
            assert_eq!(stats, before_stats);
            assert_eq!(
                inv.slots()
                    .iter()
                    .flatten()
                    .map(|i| (i.id, i.count))
                    .collect::<Vec<_>>(),
                vec![(recipe.output, 1)]
            );
        }
        for burned in [false, true] {
            let mut inv = Inventory::new(1, StackType::Normal, false);
            inv.restore_slot(
                0,
                Some(Item {
                    id: 2001,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
            let mut stats = Stats::default();
            stats.set_current(7, 25);
            let outcome = bindings
                .cooking
                .complete(
                    0,
                    &mut inv,
                    &Catalog::default(),
                    &mut stats,
                    7,
                    99,
                    125,
                    &mut Draw(if burned { 255 } else { 0 }),
                )
                .unwrap();
            assert_eq!(outcome.burned, burned);
            assert_eq!(inv.get(0).unwrap().id, if burned { 2005 } else { 2003 });
            assert_eq!(stats.fine_xp(7), if burned { 0 } else { 1462 });
        }
        defs.items.get_mut(&2005).unwrap().dummy = 1;
        assert!(stew_recipes(&obj, &defs, 7).is_err());
        defs.items.get_mut(&2005).unwrap().dummy = -1;
        obj.insert("stew2".into(), 1997);
        assert!(stew_recipes(&obj, &defs, 7).is_err());
    }
}


/// Ordinary stew healing and retained bowl, through the existing food transaction.
pub fn stew_food(objects:&Symbols,defs:&Definitions,foods:&mut BTreeMap<i32,Edible>)->Result<i32,String>{
    let mut next=foods.clone();
    apply_ordinary_food_profiles(objects,defs,&mut next,&[("stew","Stew",11)])?;
    let id=objects["stew"];
    let bowl=*objects.get("bowl_empty").ok_or("Missing stew bowl symbol")?;
    let d=u32::try_from(bowl).ok().and_then(|v|defs.items.get(&v)).ok_or("Missing stew bowl definition")?;
    if bowl==id || d.name!="Bowl" || d.members || d.stackable || d.dummy==1 || d.cert.template!=0 || d.placeholder.template!=0 || d.transform.template!=0 || defs.items[&(id as u32)].members {return Err("Unattested stew/bowl definition".into());}
    next.get_mut(&id).unwrap().replacement=Some(bowl);
    *foods=next;Ok(id)
}
#[cfg(test)]
mod stew_food_tests{
    use super::*;
    #[test]fn stew_food_selected_fullbag_heals11_caps_and_retains_bowl_without_xp(){
        let objects=Symbols::from([("stew".into(),2003),("bowl_empty".into(),1923)]);
        let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
        for(id,name,eat)in[(2003,"Stew",true),(1923,"Bowl",false)]{let mut d=gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();d.name=name.into();if eat{d.inventory_ops[0]=Some("Eat".into());}defs.items.insert(id,d);}
        let mut foods=BTreeMap::new();assert_eq!(stew_food(&objects,&defs,&mut foods).unwrap(),2003);
        for(current,expected)in[(10,21),(29,30),(40,40)]{
            let mut inv=gielinor_world::inventory::Inventory::new(2,gielinor_world::inventory::StackType::Normal,false);for slot in 0..2{inv.restore_slot(slot,Some(gielinor_world::inventory::Item{id:2003,count:1,vars:0})).unwrap();}
            let mut stats=gielinor_world::stats::Stats::default();stats.set_base(3,30);stats.set_current(3,current);
            assert!(gielinor_world::food::eat_slot(&foods[&2003],1,&mut inv,&mut stats,&Default::default(),3,829,|_|{}).unwrap());assert_eq!(inv.get(0).unwrap().id,2003);assert_eq!(inv.get(1).unwrap().id,1923);assert_eq!(stats.current(3),expected);assert_eq!(stats.fine_xp(3),0);
        }
        foods.clear();defs.items.get_mut(&1923).unwrap().cert.template=799;assert!(stew_food(&objects,&defs,&mut foods).is_err());assert!(foods.is_empty());
    }
}

/// Chocolate topping only; four-material cake batter and baking are separate.
pub fn chocolate_cake_assembly(
    objects: &Symbols,
    defs: &Definitions,
    skill: u8,
) -> Result<Vec<gielinor_world::assembly::Recipe>, String> {
    let stat = defs
        .stats
        .get(&(skill as u32))
        .ok_or("Missing chocolate-cake Cooking stat")?;
    if stat.name != "Cooking" || stat.max_level != 99 || stat.min_level != 1 || stat.unreleased {
        return Err("Unattested chocolate-cake Cooking stat".into());
    }
    let mut ids = BTreeMap::new();
    for (symbol, name) in [
        ("cake", "Cake"),
        ("chocolate_bar", "Chocolate bar"),
        ("chocolate_dust", "Chocolate dust"),
        ("chocolate_cake", "Chocolate cake"),
    ] {
        let id = *objects.get(symbol).ok_or("Missing chocolate-cake symbol")?;
        let d = u32::try_from(id)
            .ok()
            .and_then(|v| defs.items.get(&v))
            .ok_or("Missing chocolate-cake definition")?;
        if ids.values().any(|v| *v == id)
            || d.name != name
            || d.members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err("Unattested or aliased chocolate-cake item".into());
        }
        ids.insert(symbol, id);
    }
    if defs.items[&(ids["chocolate_cake"] as u32)].inventory_ops[0].as_deref() != Some("Eat") {
        return Err("Chocolate-cake output lacks Eat".into());
    }
    Ok(["chocolate_bar", "chocolate_dust"]
        .into_iter()
        .map(|topping| gielinor_world::assembly::Recipe {
            first: ids["cake"],
            second: ids[topping],
            output: ids["chocolate_cake"],
            skill,
            level: 50,
            base_xp: 30,
            members: false,
        })
        .collect())
}
#[cfg(test)]
mod chocolate_cake_binding_tests {
    use super::*;
    use gielinor_world::{
        assembly::{Job, Outcome},
        inventory::{Catalog, Inventory, Item, StackType},
        stats::Stats,
    };
    #[test]
    fn chocolate_cake_bar_and_dust_use_live50_and30_xp_without_partial_debit() {
        let mut objects = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        defs.stats.insert(
            7,
            gielinor_cache::definitions::StatType {
                name: "Cooking".into(),
                unreleased: false,
                max_level: 99,
                min_level: 1,
            },
        );
        for (symbol, id, name) in [
            ("cake", 1891, "Cake"),
            ("chocolate_bar", 1973, "Chocolate bar"),
            ("chocolate_dust", 1975, "Chocolate dust"),
            ("chocolate_cake", 1897, "Chocolate cake"),
        ] {
            objects.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            d.name = name.into();
            if symbol == "chocolate_cake" {
                d.inventory_ops[0] = Some("Eat".into());
            }
            defs.items.insert(id as u32, d);
        }
        let recipes = chocolate_cake_assembly(&objects, &defs, 7).unwrap();
        assert_eq!(recipes.len(), 2);
        for recipe in recipes {
            let mut inv = Inventory::new(2, StackType::Normal, false);
            for (slot, id) in [(0, recipe.first), (1, recipe.second)] {
                inv.restore_slot(
                    slot,
                    Some(Item {
                        id,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
            let mut stats = Stats::default();
            stats.set_base(7, 49);
            stats.set_current(7, 49);
            let job = Job {
                recipe,
                first_slot: 0,
                second_slot: 1,
            };
            let before = inv.clone();
            assert_eq!(
                job.complete_awarding(&mut inv, &Catalog::default(), &mut stats, 125, false)
                    .0,
                Outcome::Level
            );
            assert_eq!(inv, before);
            stats.set_current(7, 50);
            let mut items = Catalog::default();
            items.dummy.insert(1897);
            assert_eq!(
                job.complete_awarding(&mut inv, &items, &mut stats, 125, false)
                    .0,
                Outcome::Capacity
            );
            assert_eq!(inv, before);
            assert_eq!(stats.fine_xp(7), 0);
            assert_eq!(
                job.complete_awarding(&mut inv, &Catalog::default(), &mut stats, 125, false)
                    .0,
                Outcome::Made
            );
            assert_eq!(stats.fine_xp(7), 375);
            assert_eq!(
                inv.slots()
                    .iter()
                    .flatten()
                    .map(|i| (i.id, i.count))
                    .collect::<Vec<_>>(),
                vec![(1897, 1)]
            );
            assert_eq!(
                job.complete_awarding(&mut inv, &Catalog::default(), &mut stats, 125, false)
                    .0,
                Outcome::Stale
            );
            assert_eq!(stats.fine_xp(7), 375);
        }
        defs.items.get_mut(&1897).unwrap().placeholder.template = 14401;
        assert!(chocolate_cake_assembly(&objects, &defs, 7).is_err());
        defs.items.get_mut(&1897).unwrap().placeholder.template = 0;
        objects.insert("chocolate_dust".into(), 1973);
        assert!(chocolate_cake_assembly(&objects, &defs, 7).is_err());
    }
}


/// Four ingredients and three returned objects over shared assembly transactions.
pub fn cake_batter_assembly(
    objects: &Symbols,
    defs: &Definitions,
    skill: u8,
) -> Result<std::sync::Arc<gielinor_world::assembly::BatchRecipe>, String> {
    let stat = defs
        .stats
        .get(&(skill as u32))
        .ok_or("Missing cake-batter Cooking stat")?;
    if stat.name != "Cooking" || stat.min_level != 1 || stat.max_level != 99 || stat.unreleased {
        return Err("Unattested cake-batter Cooking stat".into());
    }
    let mut ids = BTreeMap::new();
    for (symbol, name) in [
        ("cake_tin", "Cake tin"),
        ("egg", "Egg"),
        ("bucket_milk", "Bucket of milk"),
        ("pot_flour", "Pot of flour"),
        ("uncooked_cake", "Uncooked cake"),
        ("bucket_empty", "Bucket"),
        ("pot_empty", "Pot"),
    ] {
        let id = *objects.get(symbol).ok_or("Missing cake-batter symbol")?;
        let d = u32::try_from(id)
            .ok()
            .and_then(|v| defs.items.get(&v))
            .ok_or("Missing cake-batter item")?;
        if ids.values().any(|v| *v == id)
            || d.name != name
            || d.members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err("Unattested or aliased cake-batter item".into());
        }
        ids.insert(symbol, id);
    }
    Ok(std::sync::Arc::new(gielinor_world::assembly::BatchRecipe {
        inputs: vec![
            ids["cake_tin"],
            ids["egg"],
            ids["bucket_milk"],
            ids["pot_flour"],
        ],
        outputs: vec![ids["uncooked_cake"], ids["bucket_empty"], ids["pot_empty"]],
        skill,
        level: 40,
        base_xp: 0,
        members: false,
        ticks: 2,
    }))
}
#[cfg(test)]
mod cake_batter_tests {
    use super::*;
    #[test]
    fn cake_batter_four_real_inputs_return_uncooked_cake_bucket_and_pot() {
        let mut objects = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        defs.stats.insert(
            7,
            gielinor_cache::definitions::StatType {
                name: "Cooking".into(),
                unreleased: false,
                max_level: 99,
                min_level: 1,
            },
        );
        for (symbol, id, name) in [
            ("cake_tin", 1887, "Cake tin"),
            ("egg", 1944, "Egg"),
            ("bucket_milk", 1927, "Bucket of milk"),
            ("pot_flour", 1933, "Pot of flour"),
            ("uncooked_cake", 1889, "Uncooked cake"),
            ("bucket_empty", 1925, "Bucket"),
            ("pot_empty", 1931, "Pot"),
        ] {
            objects.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            d.name = name.into();
            defs.items.insert(id as u32, d);
        }
        let recipe = cake_batter_assembly(&objects, &defs, 7).unwrap();
        assert_eq!(recipe.inputs, vec![1887, 1944, 1927, 1933]);
        assert_eq!(recipe.outputs, vec![1889, 1925, 1931]);
        assert_eq!(
            (recipe.level, recipe.ticks, recipe.base_xp, recipe.members),
            (40, 2, 0, false)
        );
        let mut inv = gielinor_world::inventory::Inventory::new(
            4,
            gielinor_world::inventory::StackType::Normal,
            false,
        );
        for (slot, &id) in recipe.inputs.iter().enumerate() {
            inv.restore_slot(
                slot,
                Some(gielinor_world::inventory::Item {
                    id,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        for target_slot in [1, 2, 3] {
            let job = recipe
                .bind(
                    gielinor_world::held_use::Request {
                        selected_slot: 0,
                        selected_id: 1887,
                        target_slot,
                        target_id: recipe.inputs[target_slot as usize],
                    },
                    &inv,
                )
                .unwrap();
            let mut next = inv.clone();
            let mut stats = gielinor_world::stats::Stats::default();
            stats.set_base(7, 39);
            stats.set_current(7, 40);
            let before = stats.clone();
            assert_eq!(
                job.complete_awarding(&mut next, &Default::default(), &mut stats, 500, false)
                    .0,
                gielinor_world::assembly::Outcome::Made
            );
            assert_eq!(stats, before);
            assert_eq!(
                next.slots()
                    .iter()
                    .flatten()
                    .map(|v| v.id)
                    .collect::<Vec<_>>(),
                recipe.outputs
            );
        }
        defs.items.get_mut(&1931).unwrap().dummy = 1;
        assert!(cake_batter_assembly(&objects, &defs, 7).is_err());
    }
}


/// Only the attested ordinary range. Quest/Hosidius/tutorial variants and Cook
/// option menus remain separate; this is manual item-on-range dispatch.
pub fn standard_cooking_range(
    objects: &Symbols,
    locations: &Symbols,
    sequences: &Symbols,
    defs: &Definitions,
    config: &mut gielinor_world::cooking::Config,
) -> Result<(), String> {
    use gielinor_world::cooking::{Recipe, Surface};
    let id = u32::try_from(
        *locations
            .get("range")
            .ok_or("Missing ordinary range symbol")?,
    )
    .map_err(|_| "Invalid ordinary range ID")?;
    if id == config.fire
        || locations
            .get("newbierange")
            .is_some_and(|v| *v as u32 == id)
        || config.surfaces.contains_key(&id)
    {
        return Err("Aliased or duplicate ordinary range".into());
    }
    let d = defs
        .locations
        .get(&id)
        .ok_or("Missing ordinary range definition")?;
    if d.name != "Range"
        || d.collision.width != 1
        || d.collision.length != 2
        || !d.multi.is_empty()
        || d.ops[0].as_deref() != Some("Cook")
    {
        return Err("Unattested ordinary range".into());
    }
    let animation = u16::try_from(
        *sequences
            .get("human_cooking")
            .ok_or("Missing range cooking sequence")?,
    )
    .map_err(|_| "Invalid range cooking animation")?;
    let mut recipes = BTreeMap::new();
    for symbol in [
        "raw_chicken",
        "raw_beef",
        "raw_shrimp",
        "raw_anchovies",
        "raw_sardine",
        "raw_herring",
        "raw_trout",
        "raw_pike",
        "raw_salmon",
        "raw_tuna",
        "raw_lobster",
        "raw_swordfish",
        "uncooked_stew",
    ] {
        let raw = *objects
            .get(symbol)
            .ok_or("Missing existing range-food symbol")?;
        let mut recipe = *config
            .recipes
            .get(&raw)
            .ok_or("Missing prerequisite fire recipe")?;
        if recipe.raw != raw {
            return Err("Aliased prerequisite range recipe".into());
        }
        if symbol == "raw_swordfish" {
            recipe.stop_burn = Some(80);
            recipe.low = 30;
            recipe.high = 310;
        }
        recipes.insert(raw, recipe);
    }
    let mut bread = BTreeMap::new();
    for (symbol, name) in [
        ("bread_dough", "Bread dough"),
        ("bread", "Bread"),
        ("burnt_bread", "Burnt bread"),
    ] {
        let item = *objects.get(symbol).ok_or("Missing bread object symbol")?;
        let d = u32::try_from(item)
            .ok()
            .and_then(|v| defs.items.get(&v))
            .ok_or("Missing bread definition")?;
        if bread.values().any(|v| *v == item)
            || recipes.contains_key(&item)
            || d.name != name
            || d.members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err("Unattested or aliased ordinary bread".into());
        }
        bread.insert(symbol, item);
    }
    if defs.items[&(bread["bread"] as u32)].inventory_ops[0].as_deref() != Some("Eat") {
        return Err("Bread output lacks Eat".into());
    }
    recipes.insert(
        bread["bread_dough"],
        Recipe {fine_xp_override:None,
            raw: bread["bread_dough"],
            cooked: bread["bread"],
            burnt: bread["burnt_bread"],
            level: 1,
            stop_burn: Some(38),
            low: 118,
            high: 492,
            xp: 40,
            ticks: 1,
        },
    );
    config.surfaces.insert(id, Surface { animation, recipes });
    Ok(())
}
#[cfg(test)]
mod standard_range_tests {
    use super::*;
    use gielinor_world::cooking::{Config, Recipe};
    #[test]
    fn standard_range_adds_bread_and_asymmetric_swordfish_without_changing_fire() {
        let locations = Symbols::from([("range".into(), 26181), ("newbierange".into(), 9736)]);
        let sequences = Symbols::from([("human_cooking".into(), 896)]);
        let mut objects = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut config = Config {
            gauntlets: None, no_burn_cape: None, member_inputs:Default::default(),
            fire: 26185,
            animation: 897,
            skill: 7,
            max_level: 99,
            tutorial: None,
            surfaces: BTreeMap::new(),
            recipes: BTreeMap::new(),
        };
        for (symbol, raw) in [
            ("raw_chicken", 2138),
            ("raw_beef", 2132),
            ("raw_shrimp", 317),
            ("raw_anchovies", 321),
            ("raw_sardine", 327),
            ("raw_herring", 345),
            ("raw_trout", 335),
            ("raw_pike", 349),
            ("raw_salmon", 331),
            ("raw_tuna", 359),
            ("raw_lobster", 377),
            ("raw_swordfish", 371),
            ("uncooked_stew", 2001),
        ] {
            objects.insert(symbol.into(), raw);
            config.recipes.insert(
                raw,
                Recipe {fine_xp_override:None,
                    raw,
                    cooked: 10000 + raw,
                    burnt: 20000 + raw,
                    level: 1,
                    stop_burn: Some(86),
                    low: 18,
                    high: 292,
                    xp: 30,
                    ticks: 4,
                },
            );
        }
        // Earlier units attest those prerequisite profiles; this test isolates
        // surface registration, method overrides and the newly introduced bread.
        for (symbol, id, name) in [
            ("bread_dough", 2307, "Bread dough"),
            ("bread", 2309, "Bread"),
            ("burnt_bread", 2311, "Burnt bread"),
        ] {
            objects.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            d.name = name.into();
            if id == 2309 {
                d.inventory_ops[0] = Some("Eat".into());
            }
            defs.items.insert(id as u32, d);
        }
        let mut d = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
        d.name = "Range".into();
        d.collision.length = 2;
        d.ops[0] = Some("Cook".into());
        defs.locations.insert(26181, d);
        standard_cooking_range(&objects, &locations, &sequences, &defs, &mut config).unwrap();
        assert_eq!(config.surfaces[&26181].recipes.len(), 14);
        assert_eq!(config.animation_for(26181), Some(896));
        assert_eq!(config.animation_for(26185), Some(897));
        let range = config.recipe(26181, 371).unwrap();
        let fire = config.recipe(26185, 371).unwrap();
        assert_eq!((range.stop_burn, range.low, range.high), (Some(80), 30, 310));
        assert_eq!((fire.stop_burn, fire.low, fire.high), (Some(86), 18, 292));
        let bread = config.recipe(26181, 2307).unwrap();
        assert_eq!(
            (
                bread.cooked,
                bread.burnt,
                bread.xp,
                bread.stop_burn,
                bread.ticks
            ),
            (2309, 2311, 40, Some(38), 1)
        );
        assert!(config.recipe(26185, 2307).is_none());
        assert!(config.recipe(9736, 2307).is_none());
        assert!(config.recipe(114, 2307).is_none());
        config.surfaces.clear();
        defs.items.get_mut(&2311).unwrap().cert.template = 799;
        assert!(
            standard_cooking_range(&objects, &locations, &sequences, &defs, &mut config).is_err()
        );
        assert!(config.surfaces.is_empty());
    }
}

/// Range-only baking of already assembled foods. Their dishes remain in the pie
/// stages; emptying a burnt pie and quantity selection are separate operations.
pub fn ordinary_range_baking(
    objects: &Symbols,
    locations: &Symbols,
    defs: &Definitions,
    config: &mut gielinor_world::cooking::Config,
) -> Result<(), String> {
    use gielinor_world::cooking::Recipe;
    let range = u32::try_from(
        *locations
            .get("range")
            .ok_or("Missing baking range symbol")?,
    )
    .map_err(|_| "Invalid baking range ID")?;
    let surface = config
        .surfaces
        .get(&range)
        .ok_or("Configure ordinary range before baking")?;
    let mut next = surface.recipes.clone();
    let mut raw_ids: std::collections::BTreeSet<_> = next.keys().copied().collect();
    let mut output_ids: std::collections::BTreeSet<_> =
        next.values().flat_map(|r| [r.cooked, r.burnt]).collect();
    for (raw, cooked, burnt, raw_name, cooked_name, burnt_name, level, xp, stop, low, high) in [
        (
            "uncooked_redberry_pie",
            "redberry_pie",
            "burnt_pie",
            "Uncooked berry pie",
            "Redberry pie",
            "Burnt pie",
            10,
            78,
            45,
            98,
            452,
        ),
        (
            "uncooked_meat_pie",
            "meat_pie",
            "burnt_pie",
            "Uncooked meat pie",
            "Meat pie",
            "Burnt pie",
            20,
            110,
            54,
            78,
            412,
        ),
        (
            "uncooked_apple_pie",
            "apple_pie",
            "burnt_pie",
            "Uncooked apple pie",
            "Apple pie",
            "Burnt pie",
            30,
            130,
            63,
            58,
            372,
        ),
        (
            "uncooked_pizza",
            "plain_pizza",
            "burnt_pizza",
            "Uncooked pizza",
            "Plain pizza",
            "Burnt pizza",
            35,
            143,
            68,
            48,
            352,
        ),
    ] {
        let resolve = |n: &str| {
            objects
                .get(n)
                .copied()
                .ok_or_else(|| format!("Missing baking symbol {n}"))
        };
        let raw = resolve(raw)?;
        let cooked = resolve(cooked)?;
        let burnt = resolve(burnt)?;
        if raw == cooked
            || raw == burnt
            || cooked == burnt
            || output_ids.contains(&raw)
            || !raw_ids.insert(raw)
            || raw_ids.contains(&cooked)
            || raw_ids.contains(&burnt)
        {
            return Err("Aliased ordinary baking profile".into());
        }
        for (id, name) in [(raw, raw_name), (cooked, cooked_name), (burnt, burnt_name)] {
            let d = u32::try_from(id)
                .ok()
                .and_then(|v| defs.items.get(&v))
                .ok_or("Missing baking item definition")?;
            if d.name != name
                || d.members
                || d.stackable
                || d.dummy == 1
                || d.cert.template != 0
                || d.placeholder.template != 0
                || d.transform.template != 0
            {
                return Err("Unattested ordinary baking item".into());
            }
        }
        if defs.items[&(cooked as u32)].inventory_ops[0].as_deref() != Some("Eat") {
            return Err("Baking output lacks Eat".into());
        }
        output_ids.extend([cooked, burnt]);
        next.insert(
            raw,
            Recipe {fine_xp_override:None,
                raw,
                cooked,
                burnt,
                level,
                stop_burn: Some(stop),
                low,
                high,
                xp,
                ticks: 1,
            },
        );
    }
    config.surfaces.get_mut(&range).unwrap().recipes = next;
    Ok(())
}
#[cfg(test)]
mod ordinary_baking_tests {
    use super::*;
    use gielinor_world::{
        cooking::{Config, Surface},
        inventory::{Catalog, Inventory, Item, StackType},
        npc::RandomSource,
        stats::Stats,
    };
    struct Draw(u32);
    impl RandomSource for Draw {
        fn below(&mut self, n: u32) -> u32 {
            assert_eq!(n, 256);
            self.0
        }
    }
    #[test]
    fn ordinary_baking_four_real_profiles_selected_outcomes_xp_and_late_catalog_rollback() {
        let mut objects = Symbols::new();
        let locations = Symbols::from([("range".into(), 26181)]);
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (symbol, id, name, eat) in [
            ("uncooked_redberry_pie", 2321, "Uncooked berry pie", false),
            ("redberry_pie", 2325, "Redberry pie", true),
            ("uncooked_meat_pie", 2319, "Uncooked meat pie", false),
            ("meat_pie", 2327, "Meat pie", true),
            ("uncooked_apple_pie", 2317, "Uncooked apple pie", false),
            ("apple_pie", 2323, "Apple pie", true),
            ("burnt_pie", 2329, "Burnt pie", false),
            ("uncooked_pizza", 2287, "Uncooked pizza", false),
            ("plain_pizza", 2289, "Plain pizza", true),
            ("burnt_pizza", 2305, "Burnt pizza", false),
        ] {
            objects.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            d.name = name.into();
            if eat {
                d.inventory_ops[0] = Some("Eat".into());
            }
            defs.items.insert(id as u32, d);
        }
        let empty = Config {
            gauntlets: None, no_burn_cape: None, member_inputs:Default::default(),
            fire: 26185,
            animation: 897,
            skill: 7,
            max_level: 99,
            tutorial: None,
            recipes: BTreeMap::new(),
            surfaces: [(
                26181,
                Surface {
                    animation: 896,
                    recipes: BTreeMap::new(),
                },
            )]
            .into(),
        };
        let mut config = empty.clone();
        ordinary_range_baking(&objects, &locations, &defs, &mut config).unwrap();
        assert_eq!(config.surfaces[&26181].recipes.len(), 4);
        for recipe in config.surfaces[&26181].recipes.values() {
            assert!(config.recipe(26185, recipe.raw).is_none());
            assert_eq!(recipe.ticks, 1);
            for burned in [false, true] {
                let mut inv = Inventory::new(2, StackType::Normal, false);
                for slot in 0..2 {
                    inv.restore_slot(
                        slot,
                        Some(Item {
                            id: recipe.raw,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                let mut stats = Stats::default();
                stats.set_current(7, recipe.level as i8 - 1);
                let before = inv.clone();
                assert!(recipe
                    .complete(
                        1,
                        &mut inv,
                        &Catalog::default(),
                        &mut stats,
                        7,
                        99,
                        125,
                        &mut Draw(0)
                    )
                    .is_err());
                assert_eq!(inv, before);
                stats.set_current(7, recipe.level as i8);
                let outcome = recipe
                    .complete(
                        1,
                        &mut inv,
                        &Catalog::default(),
                        &mut stats,
                        7,
                        99,
                        125,
                        &mut Draw(if burned { 255 } else { 0 }),
                    )
                    .unwrap();
                assert_eq!(outcome.burned, burned);
                assert_eq!(inv.get(0).unwrap().id, recipe.raw);
                assert_eq!(
                    inv.get(1).unwrap().id,
                    if burned { recipe.burnt } else { recipe.cooked }
                );
                assert_eq!(
                    stats.fine_xp(7),
                    if burned {
                        0
                    } else {
                        i32::from(recipe.xp) * 125 / 10
                    }
                );
            }
        }
        let mut config = empty;
        defs.items.get_mut(&2305).unwrap().dummy = 1;
        assert!(ordinary_range_baking(&objects, &locations, &defs, &mut config).is_err());
        assert!(config.surfaces[&26181].recipes.is_empty());
    }
}

/// Member pie baking on the ordinary range. None preserves a probabilistic burn
/// at99 for dragonfruit/wild/summer instead of inventing a forced level99 cutoff.
pub fn member_pie_baking(
    objects: &Symbols,
    locations: &Symbols,
    defs: &Definitions,
    config: &mut gielinor_world::cooking::Config,
) -> Result<(), String> {
    use gielinor_world::cooking::Recipe;
    let range = u32::try_from(*locations.get("range").ok_or("Missing member-pie range")?)
        .map_err(|_| "Invalid member-pie range")?;
    let mut next = config
        .surfaces
        .get(&range)
        .ok_or("Configure ordinary range before member pies")?
        .recipes
        .clone();
    let mut members = config.member_inputs.clone();
    let mut raws: std::collections::BTreeSet<_> = next.keys().copied().collect();
    let mut outputs: std::collections::BTreeSet<_> =
        next.values().flat_map(|r| [r.cooked, r.burnt]).collect();
    for (raw, cooked, raw_name, cooked_name, level, xp, stop, low, high) in [
        (
            "uncooked_garden_pie",
            "garden_pie",
            "Raw garden pie",
            "Garden pie",
            34,
            138,
            Some(68),
            48,
            352,
        ),
        (
            "uncooked_fish_pie",
            "fish_pie",
            "Raw fish pie",
            "Fish pie",
            47,
            164,
            Some(74),
            38,
            332,
        ),
        (
            "uncooked_botanical_pie",
            "botanical_pie",
            "Uncooked botanical pie",
            "Botanical pie",
            52,
            180,
            Some(84),
            20,
            300,
        ),
        (
            "uncooked_mushroom_pie",
            "mushroom_pie",
            "Uncooked mushroom pie",
            "Mushroom pie",
            60,
            200,
            Some(89),
            17,
            285,
        ),
        (
            "uncooked_admiral_pie",
            "admiral_pie",
            "Raw admiral pie",
            "Admiral pie",
            70,
            210,
            Some(94),
            15,
            270,
        ),
        (
            "uncooked_dragonfruit_pie",
            "dragonfruit_pie",
            "Uncooked dragonfruit pie",
            "Dragonfruit pie",
            73,
            220,
            None,
            8,
            250,
        ),
        (
            "uncooked_wild_pie",
            "wild_pie",
            "Raw wild pie",
            "Wild pie",
            85,
            240,
            None,
            1,
            222,
        ),
        (
            "uncooked_summer_pie",
            "summer_pie",
            "Raw summer pie",
            "Summer pie",
            95,
            260,
            None,
            1,
            212,
        ),
    ] {
        let resolve = |n: &str| {
            objects
                .get(n)
                .copied()
                .ok_or_else(|| format!("Missing member-pie symbol {n}"))
        };
        let raw = resolve(raw)?;
        let cooked = resolve(cooked)?;
        let burnt = resolve("burnt_pie")?;
        if raw == cooked
            || raw == burnt
            || cooked == burnt
            || outputs.contains(&raw)
            || !raws.insert(raw)
            || raws.contains(&cooked)
            || raws.contains(&burnt)
        {
            return Err("Aliased member-pie profile".into());
        }
        for (id, name, member) in [
            (raw, raw_name, true),
            (cooked, cooked_name, true),
            (burnt, "Burnt pie", false),
        ] {
            let d = u32::try_from(id)
                .ok()
                .and_then(|v| defs.items.get(&v))
                .ok_or("Missing member-pie definition")?;
            if d.name != name
                || d.members != member
                || d.stackable
                || d.dummy == 1
                || d.cert.template != 0
                || d.placeholder.template != 0
                || d.transform.template != 0
            {
                return Err("Unattested ordinary member-pie item".into());
            }
        }
        if defs.items[&(cooked as u32)].inventory_ops[0].as_deref() != Some("Eat") {
            return Err("Member-pie cooked output lacks Eat".into());
        }
        outputs.extend([cooked, burnt]);
        members.insert(raw);
        next.insert(
            raw,
            Recipe {fine_xp_override:None,
                raw,
                cooked,
                burnt,
                level,
                stop_burn: stop,
                low,
                high,
                xp,
                ticks: 1,
            },
        );
    }
    config.surfaces.get_mut(&range).unwrap().recipes = next;
    config.member_inputs = members;
    Ok(())
}
#[cfg(test)]
mod member_pie_baking_tests {
    use super::*;
    use gielinor_world::{
        cooking::{Config, Surface},
        inventory::{Catalog, Inventory, Item, StackType},
        npc::RandomSource,
        stats::Stats,
    };
    struct Draw {
        value: u32,
        calls: u32,
    }
    impl RandomSource for Draw {
        fn below(&mut self, n: u32) -> u32 {
            assert_eq!(n, 256);
            self.calls += 1;
            self.value
        }
    }
    #[test]
    fn member_pies_eight_profiles_keep99_burns_exact_rewards_and_atomic_publication() {
        let mut objects = Symbols::new();
        let locations = Symbols::from([("range".into(), 26181)]);
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (symbol, id, name) in [
            ("uncooked_garden_pie", 7176, "Raw garden pie"),
            ("garden_pie", 7178, "Garden pie"),
            ("uncooked_fish_pie", 7186, "Raw fish pie"),
            ("fish_pie", 7188, "Fish pie"),
            ("uncooked_botanical_pie", 19656, "Uncooked botanical pie"),
            ("botanical_pie", 19662, "Botanical pie"),
            ("uncooked_mushroom_pie", 21684, "Uncooked mushroom pie"),
            ("mushroom_pie", 21690, "Mushroom pie"),
            ("uncooked_admiral_pie", 7196, "Raw admiral pie"),
            ("admiral_pie", 7198, "Admiral pie"),
            (
                "uncooked_dragonfruit_pie",
                22789,
                "Uncooked dragonfruit pie",
            ),
            ("dragonfruit_pie", 22795, "Dragonfruit pie"),
            ("uncooked_wild_pie", 7206, "Raw wild pie"),
            ("wild_pie", 7208, "Wild pie"),
            ("uncooked_summer_pie", 7216, "Raw summer pie"),
            ("summer_pie", 7218, "Summer pie"),
            ("burnt_pie", 2329, "Burnt pie"),
        ] {
            objects.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            d.name = name.into();
            d.members = id != 2329;
            if !symbol.starts_with("uncooked_") && id != 2329 {
                d.inventory_ops[0] = Some("Eat".into());
            }
            defs.items.insert(id as u32, d);
        }
        let empty = Config {
            fire: 26185,
            animation: 897,
            skill: 7,
            max_level: 99,
            tutorial: None,
            gauntlets: None, no_burn_cape: None, member_inputs: Default::default(),
            recipes: BTreeMap::new(),
            surfaces: [(
                26181,
                Surface {
                    animation: 896,
                    recipes: BTreeMap::new(),
                },
            )]
            .into(),
        };
        let mut config = empty.clone();
        member_pie_baking(&objects, &locations, &defs, &mut config).unwrap();
        assert_eq!(config.member_inputs.len(), 8);
        assert_eq!(config.surfaces[&26181].recipes.len(), 8);
        assert_eq!(config.recipe(26181, 22789).unwrap().xp, 220);
        for recipe in config.surfaces[&26181].recipes.values() {
            assert!(config.member_inputs.contains(&recipe.raw));
            assert!(config.recipe(26185, recipe.raw).is_none());
            for burned in [false, true] {
                let mut inv = Inventory::new(2, StackType::Normal, false);
                for slot in 0..2 {
                    inv.restore_slot(
                        slot,
                        Some(Item {
                            id: recipe.raw,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                let mut stats = Stats::default();
                stats.set_current(7, recipe.level as i8);
                let mut rng = Draw {
                    value: if burned { 255 } else { 0 },
                    calls: 0,
                };
                let outcome = recipe
                    .complete(
                        1,
                        &mut inv,
                        &Catalog::default(),
                        &mut stats,
                        7,
                        99,
                        125,
                        &mut rng,
                    )
                    .unwrap();
                assert_eq!(outcome.burned, burned);
                assert_eq!(inv.get(0).unwrap().id, recipe.raw);
                assert_eq!(
                    inv.get(1).unwrap().id,
                    if burned { 2329 } else { recipe.cooked }
                );
                assert_eq!(
                    stats.fine_xp(7),
                    if burned {
                        0
                    } else {
                        i32::from(recipe.xp) * 125 / 10
                    }
                );
            }
            if recipe.stop_burn.is_none() {
                let mut inv = Inventory::new(1, StackType::Normal, false);
                inv.restore_slot(
                    0,
                    Some(Item {
                        id: recipe.raw,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
                let mut stats = Stats::default();
                stats.set_current(7, 99);
                let mut rng = Draw {
                    value: 255,
                    calls: 0,
                };
                assert!(
                    recipe
                        .complete(
                            0,
                            &mut inv,
                            &Catalog::default(),
                            &mut stats,
                            7,
                            99,
                            100,
                            &mut rng
                        )
                        .unwrap()
                        .burned
                );
                assert_eq!(rng.calls, 1);
                assert_eq!(stats.fine_xp(7), 0);
            }
        }
        let mut config = empty;
        defs.items.get_mut(&7216).unwrap().cert.template = 799;
        assert!(member_pie_baking(&objects, &locations, &defs, &mut config).is_err());
        assert!(config.member_inputs.is_empty());
        assert!(config.surfaces[&26181].recipes.is_empty());
    }
}
