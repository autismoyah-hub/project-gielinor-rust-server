use super::{Definitions,Symbols};
use gielinor_world::{ducks::Config,npc::{Restrict,BlockWalk},npc_death};
pub fn ducks(npcs:&Symbols,content:&Symbols,sounds:&Symbols,d:&Definitions,death:&npc_death::Config)->Result<Config,String>{
    for(names,name,id)in[(npcs,"duck",1838),(npcs,"duck_female",1839),(npcs,"duck_update_ducklings",2002),(content,"duck",24),(content,"duckling",25),(sounds,"quack",413)]{
        if names.get(name)!=Some(&id){return Err(format!("Unattested duck symbol {name}"));}
    }
    let adults=[1838,1839].into_iter().collect::<std::collections::BTreeSet<_>>();
    let actual=d.npcs.iter().filter(|(_,t)|t.content_group==24).map(|(id,_)|*id).collect::<Vec<_>>();
    if actual!=[1838,1839]||d.npcs.iter().filter(|(_,t)|t.content_group==25).map(|(id,_)|*id).collect::<Vec<_>>()!=[2002]{return Err("Unattested expanded duck family".into());}
    for(id,young)in[(1838,false),(1839,false),(2002,true)]{
        let t=d.npcs.get(&(id as u32)).ok_or("Missing duck profile")?;
        let ops=if young{[None,None,None,None,None]}else{[None,Some("Attack"),None,None,None]};
        if t.name!=if young{"Ducklings"}else{"Duck"}||t.size!=1||t.timer!=if young{50}else{25}||t.stats!=[1,1,1,if young{10}else{3},1,1]
            ||t.ops.iter().map(Option::as_deref).collect::<Vec<_>>()!=ops||t.restrict!=Restrict::Blocked||!matches!(t.block_walk,BlockWalk::Npc)||t.default_mode!=u8::from(!young)||t.wander_range!=35
            ||!t.multi.is_empty()||t.multi_varp.is_some()||t.multi_varbit.is_some()||t.multi_default.is_some(){return Err("Unattested ordinary aquatic duck profile".into());}
        if !young&&!death.definitions.get(&id).is_some_and(|r|r.animation==3468&&r.animation_ticks>0){return Err("Unattested duck no-drop death lifecycle".into());}
    }
    Ok(Config{adults,male:1838,female:1839,ducklings:2002,sound:413})
}
#[cfg(test)]
mod tests{
    use super::*;
    fn fixture()->(Symbols,Symbols,Symbols,Definitions,npc_death::Config){
        let mut d=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
        for(raw,young)in[(1838,false),(1839,false),(2002,true)]{let mut t=gielinor_cache::npcs::NpcType::decode(&[0]).unwrap();t.name=if young{"Ducklings"}else{"Duck"}.into();t.timer=if young{50}else{25};t.stats=[1,1,1,if young{10}else{3},1,1];t.restrict=Restrict::Blocked;t.block_walk=BlockWalk::Npc;t.default_mode=u8::from(!young);t.wander_range=35;t.content_group=if young{25}else{24};if !young{t.ops[1]=Some("Attack".into());}d.npcs.insert(raw,t);}
        let death=npc_death::Config{definitions:[(1838,npc_death::Definition{animation:3468,animation_ticks:3,sound:None}),(1839,npc_death::Definition{animation:3468,animation_ticks:3,sound:None})].into(),bones:526,chickens:Default::default()};
        ([("duck".into(),1838),("duck_female".into(),1839),("duck_update_ducklings".into(),2002)].into(),[("duck".into(),24),("duckling".into(),25)].into(),[("quack".into(),413)].into(),d,death)
    }
    #[test]
    fn duck_family_binds_only_actual_aquatic_profiles_and_positive_deaths(){let(n,c,s,d,death)=fixture();let f=ducks(&n,&c,&s,&d,&death).unwrap();assert_eq!(f.adults.len(),2);assert!(f.contains(2002));assert!(!f.contains(2001));assert!(!f.contains(2003));for case in 0..12{let(mut n,mut c,mut s,mut d,mut death)=fixture();match case{0=>{n.remove("duck_female");},1=>{c.insert("duckling".into(),24);},2=>{s.insert("quack".into(),414);},3=>d.npcs.get_mut(&1838).unwrap().timer=1,4=>d.npcs.get_mut(&2002).unwrap().timer=25,5=>d.npcs.get_mut(&1839).unwrap().restrict=Restrict::Normal,6=>d.npcs.get_mut(&2002).unwrap().default_mode=1,7=>d.npcs.get_mut(&2002).unwrap().ops[1]=Some("Attack".into()),8=>d.npcs.get_mut(&1839).unwrap().multi.push(Some(1838)),9=>{death.definitions.remove(&1839);},10=>death.definitions.get_mut(&1838).unwrap().animation_ticks=0,_=>{let t=d.npcs[&1838].clone();d.npcs.insert(65536+1838,t);}}assert!(ducks(&n,&c,&s,&d,&death).is_err(),"duckdrift{case}");}}
}
