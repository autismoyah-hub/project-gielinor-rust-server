use super::{Definitions,Symbols};
use gielinor_world::shearing::Config;
pub fn shearing(npcs:&Symbols,items:&Symbols,content:&Symbols,params:&Symbols,sequences:&Symbols,sounds:&Symbols,d:&Definitions)->Result<Config,String> {
    for(names,name,id)in [(items,"shears",1735),(items,"wool",1737),(content,"sheep",41),(content,"sheared_sheep",42),(params,"next_npc_stage",65443),(sequences,"human_shearing",893),(sounds,"shear_sheep",761)] {
        if names.get(name)!=Some(&id){return Err(format!("Unattested sheep symbol {name}"));}
    }
    for(id,name)in [(1735,"Shears"),(1737,"Wool")] {
        let t=d.items.get(&id).ok_or("Missing shearing item")?;
        if t.name!=name||t.members||t.stackable||t.dummy!=-1||t.cert.template!=0||t.placeholder.template!=0||t.transform.template!=0||t.bought.template!=0||!t.objvars.is_empty(){return Err("Unattested shearing item profile".into());}
    }
    let mut targets=std::collections::BTreeMap::new();
    for(raw,next,raw_name,next_name)in [
        (2693,1178,"sheepunsheered","sheepsheered"),(2694,1299,"sheepunsheeredg","sheepsheeredg"),(2695,1300,"sheepunsheeredw","sheepsheeredw"),
        (2696,1301,"sheepunsheered2","sheepsheered2"),(2697,1302,"sheepunsheered2g","sheepsheered2g"),(2698,1303,"sheepunsheered2w","sheepsheered2w"),
        (2699,1304,"sheepunsheered3","sheepsheered3"),(2786,1308,"sheepunsheered3g","sheepsheered3g"),(2787,1309,"sheepunsheered3w","sheepsheered3w"),
        (2788,2691,"sheepunsheeredshaggy","sheepsheeredshaggy"),(2789,2692,"sheepunsheeredshaggy2","sheepsheeredshaggy2"),
    ] {
        if npcs.get(raw_name)!=Some(&(raw as i32))||npcs.get(next_name)!=Some(&(next as i32)){return Err("Unattested ordinary sheep identity".into());}
        for(id,group,op)in [(raw,41,Some("Shear")),(next,42,None)] {
            let t=d.npcs.get(&(id as u32)).ok_or("Missing sheep stage")?;
            if t.name!="Sheep"||t.content_group!=group||t.size!=1||t.ops[0].as_deref()!=op||t.ops[1..].iter().any(Option::is_some)
                ||t.stats!=[1,1,1,10,1,1]||t.default_mode!=1||!matches!(t.restrict,gielinor_world::npc::Restrict::Normal)||!matches!(t.block_walk,gielinor_world::npc::BlockWalk::Npc)
                ||!t.multi.is_empty()||t.multi_varbit.is_some()||t.multi_varp.is_some()||t.multi_default.is_some(){return Err(format!("Unattested sheep profile {id}"));}
        }
        if !matches!(d.npcs[&(raw as u32)].params.get(&65443),Some(gielinor_cache::definitions::Param::Integer(v))if *v==next as i32){return Err("Unattested fleece transition".into());}
        targets.insert(raw,next);
    }
    Ok(Config{targets,shears:1735,wool:1737,sequence:893,sound:761})
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_cache::definitions::Param;
    fn fixture()->(Symbols,Symbols,Symbols,Symbols,Symbols,Symbols,Definitions) {
        let mut d=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
        let raw=[2693,2694,2695,2696,2697,2698,2699,2786,2787,2788,2789];let next=[1178,1299,1300,1301,1302,1303,1304,1308,1309,2691,2692];
        let suffix=["","g","w","2","2g","2w","3","3g","3w","shaggy","shaggy2"];let mut npcs=Symbols::new();
        for i in 0..11 {
            npcs.insert(format!("sheepunsheered{}",suffix[i]),raw[i]);npcs.insert(format!("sheepsheered{}",suffix[i]),next[i]);
            for(id,group)in [(raw[i],41),(next[i],42)] {
                let mut ty=gielinor_cache::npcs::NpcType::decode(&[0]).unwrap();ty.name="Sheep".into();ty.content_group=group;
                if group==41{ty.ops[0]=Some("Shear".into());ty.params.insert(65443,Param::Integer(next[i]));}
                d.npcs.insert(id as u32,ty);
            }
        }
        for(id,name)in [(1735,"Shears"),(1737,"Wool")] {let mut t=gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();t.name=name.into();d.items.insert(id,t);}
        (npcs,[("shears".into(),1735),("wool".into(),1737)].into(),[("sheep".into(),41),("sheared_sheep".into(),42)].into(),[("next_npc_stage".into(),65443)].into(),[("human_shearing".into(),893)].into(),[("shear_sheep".into(),761)].into(),d)
    }
    #[test]
    fn shearing_binding_all_eleven_ordinary_pairs_and_late_drift_refusals() {
        let(n,i,c,p,s,a,d)=fixture();let bound=shearing(&n,&i,&c,&p,&s,&a,&d).unwrap();assert_eq!(bound.targets.len(),11);assert_eq!(bound.targets[&2789],2692);assert!(!bound.targets.contains_key(&5843));
        for mode in 0..12 {
            let(mut n,mut i,c,p,mut s,a,mut d)=fixture();
            match mode {
                0=>{n.remove("sheepunsheeredshaggy2");},
                1=>{d.npcs.remove(&2692);},
                2=>{d.npcs.get_mut(&2789).unwrap().params.insert(65443,Param::Integer(1178));},
                3=>d.npcs.get_mut(&2789).unwrap().ops[0]=Some("Talk-to".into()),
                4=>d.npcs.get_mut(&2692).unwrap().ops[1]=Some("Attack".into()),
                5=>d.npcs.get_mut(&2692).unwrap().size=2,
                6=>d.npcs.get_mut(&2692).unwrap().content_group=41,
                7=>d.npcs.get_mut(&2692).unwrap().stats[3]=0,
                8=>d.items.get_mut(&1737).unwrap().dummy=1,
                9=>d.items.get_mut(&1735).unwrap().cert.template=799,
                10=>{i.insert("wool".into(),1738);},
                _=>{s.insert("human_shearing".into(),894);},
            }
            assert!(shearing(&n,&i,&c,&p,&s,&a,&d).is_err(),"late drift {mode}");
        }
    }
}
