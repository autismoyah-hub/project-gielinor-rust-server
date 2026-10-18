//! Read-only exact-target cache resolution; report configured reward policy.
use gielinor_cache::{game::GameCache,definitions::{ItemType,Param},symbols};
use std::path::Path;
#[test]
fn actual_loaded_bone_definitions_and_effective_reward_policy(){
    let cache=std::env::var("PG_GAME_CACHE").unwrap();let dir=std::env::var("PG_SYMBOLS").unwrap();
    let names=symbols::load(&[Path::new(&dir)],"param",None).unwrap();let xp=names["skill_xp"] as u32;
    let groups=symbols::load(&[Path::new(&dir)],"content",None).unwrap();let group=groups["prayer_bones"];
    let mut cache=GameCache::open(Path::new(&cache),64<<20).unwrap();let files=cache.files(2,10,None).unwrap().unwrap();
    for (id,name,member) in [(526,"Bones",false),(532,"Big bones",false),(536,"Dragon bones",true)]{
        let d=ItemType::decode(&files[&id]).unwrap();assert_eq!(d.name,name);assert_eq!(d.members,member);assert!(!d.stackable);assert_eq!(d.dummy,0);
        assert_eq!(d.cert.template,0);assert_eq!(d.placeholder.template,0);assert_eq!(d.transform.template,0);assert_eq!(d.content_group,group);
        assert_eq!(d.inventory_ops[0].as_deref(),Some("Bury"));
        let Param::Integer(fine)=d.params[&xp] else{panic!("bone reward must be integral fine units")};assert!(fine>=0);
        println!("PG_LOADED_BONE {{\"id\":{id},\"base_fine_xp\":{fine},\"member\":{member},\"personal_rate_hundreds\":500,\"expected_award_fine\":{}}}",fine*5);
    }
}
