//! Independent counted-credit and per-raw XP oracle at the amethyst contract SHA.
use gielinor_world::{grinding,tool_crafting::{Job,Recipe},inventory::{Inventory,Catalog,Item,StackType},stats::Stats};
const PROFILES:[(i32,u16,u8);4]=[(21338,15,83),(21350,15,85),(21352,5,87),(25853,8,89)];
fn fixture(raw:usize,output:i32,count:u16,level:u8)->(Job,Inventory,Catalog,Stats){
    let mut inv=Inventory::new(28,StackType::Normal,false);
    for s in 0..28{inv.restore_slot(s,Some(Item{id:1931,count:1,vars:0})).unwrap();}
    inv.restore_slot(0,Some(Item{id:1755,count:1,vars:0})).unwrap();
    for s in [raw,raw%27+1]{inv.restore_slot(s,Some(Item{id:21347,count:1,vars:0})).unwrap();}
    let mut catalog=Catalog::default();catalog.stackable.insert(output);
    let mut stats=Stats::default();stats.set_base(12,99);stats.set_current(12,99);stats.set_fine_xp(12,130344311).unwrap();
    (Job{recipe:Recipe{exchange:grinding::Recipe{tool:1755,input:21347,output,members:true,animation:Some(6295)},level,xp_tenths:600,output_count:count,ticks:1,name:"amethyst",action:"cut",failure:None},tool_slot:0,input_slot:raw},inv,catalog,stats)
}

struct NoDraw;impl gielinor_world::npc::RandomSource for NoDraw{fn below(&mut self,_:u32)->u32{panic!("unsupported multi-output failure drew RNG")}}
struct Draw(u32);impl gielinor_world::npc::RandomSource for Draw{fn below(&mut self,n:u32)->u32{assert_eq!(n,256);self.0}}
#[test]fn four_multi_output_failure_policies_refuse_before_rng(){for(output,count,level)in PROFILES{let(mut job,inv,cat,stats)=fixture(17,output,count,level);job.recipe.failure=Some(gielinor_world::tool_crafting::Failure{output:1633,xp_tenths:0,low:40,high:240,message:"independent unsupported failure"});let before=(inv.clone(),stats.clone());assert!(job.resolve(&inv,&cat,&stats,125,true,&mut NoDraw).is_err());assert_eq!((inv,stats),before);}}
#[test]fn eight_synthetic_count_one_failure_branches_preserve_legacy_semantics(){for(output,_,level)in PROFILES{for fail in[false,true]{let(mut job,inv,cat,stats)=fixture(17,output,1,level);job.recipe.failure=Some(gielinor_world::tool_crafting::Failure{output:1633,xp_tenths:0,low:40,high:240,message:"independent legacy failure"});let(p,success)=job.resolve(&inv,&cat,&stats,125,true,&mut Draw(if fail{255}else{0})).unwrap();assert_eq!(success,!fail);let mut expected=inv.clone();expected.restore_slot(17,Some(Item{id:if fail{1633}else{output},count:1,vars:0})).unwrap();let mut xp=stats.clone();if !fail{xp.set_fine_xp(12,130345061).unwrap();}assert_eq!(p.inventory,expected);assert_eq!(p.stats,xp);assert_eq!(inv.get(17).unwrap().id,21347);}}}
