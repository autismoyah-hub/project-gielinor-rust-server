use super::{Definitions,Symbols};
use std::collections::BTreeMap;
pub struct Cows {
    pub flavour:gielinor_world::npc_flavour::Config,
    pub replies:BTreeMap<u16,gielinor_world::npc_item_reply::Rule>,
    pub drops:BTreeMap<u16,gielinor_world::npc_death::GuaranteedExtras>,
}
pub fn cows(npcs:&Symbols,content:&Symbols,items:&Symbols,sequences:&Symbols,sounds:&Symbols,d:&Definitions)->Result<Cows,String>{
    for(names,name,id)in[(content,"cow",43),(content,"cow_calf",45),(items,"bucket_empty",1925),(sequences,"cow_update_graze",5854),(sounds,"cow_atmospheric",3044)]{
        if names.get(name)!=Some(&id){return Err(format!("Unattested cow symbol {name}"));}
    }
    let bucket=d.items.get(&1925).ok_or("Missing empty bucket")?;
    if bucket.name!="Bucket"||bucket.members||bucket.stackable||bucket.dummy!=-1||bucket.cert.template!=0||bucket.placeholder.template!=0||!bucket.objvars.is_empty(){return Err("Unattested empty bucket profile".into());}
    for(symbol,id,name)in[("bones",526,"Bones"),("cow_hide",1739,"Cowhide"),("raw_beef",2132,"Raw beef")]{
        let t=d.items.get(&id).ok_or("Missing cow drop item")?;
        if items.get(symbol)!=Some(&(id as i32))||t.name!=name||t.members||t.stackable||t.wear!=[-1,-1,-1]||t.dummy==1||t.cert.template!=0||t.placeholder.template!=0||t.transform.template!=0||!t.objvars.is_empty(){return Err("Unattested ordinary cow drop item".into());}
    }
    let mut drops=BTreeMap::new();
    let mut flavour=gielinor_world::npc_flavour::Config::default();let mut replies=BTreeMap::new();
    for(id,name,calf)in[(2790,"cow",false),(2791,"cow2",false),(2793,"cow3",false),(2795,"cow_beef",false),(2792,"cow2_calf",true),(2794,"cow3_calf",true)]{
        if npcs.get(name)!=Some(&(id as i32)){return Err("Unattested ordinary cow identity".into());}
        let t=d.npcs.get(&(id as u32)).ok_or("Missing cow profile")?;
        if t.name!=if calf{"Cow calf"}else{"Cow"}||t.content_group!=if calf{45}else{43}||t.size!=2||t.timer!=1
            ||t.stats!=[1,1,1,if calf{6}else{8},1,1]||t.ops[0].as_deref()!=if id==2790{Some("Attack")}else{None}
            ||t.ops[1].as_deref()!=Some("Attack")||t.ops[2..].iter().any(Option::is_some)||!t.multi.is_empty()||t.multi_varp.is_some()||t.multi_varbit.is_some()||t.multi_default.is_some(){return Err("Unattested ordinary cow profile".into());}
        drops.insert(id,gielinor_world::npc_death::GuaranteedExtras{products:vec![(1739,1),(2132,1)],policy:gielinor_world::npc_death::DropPolicy::Cow});
        flavour.types.insert(id,gielinor_world::npc_flavour::Rule{text:"Moo",sound:3044,radius:10,animation:Some(5854)});
        replies.insert(id,gielinor_world::npc_item_reply::Rule{special_item:1925,special:if calf{"Calves are too young to be milked."}else{"Only dairy cows are suitable for milking."},otherwise:if calf{"The calf doesn't want that."}else{"The cow doesn't want that."}});
    }
    Ok(Cows{flavour,replies,drops})
}
#[cfg(test)]
mod tests{
    use super::*;
    fn fixture()->(Symbols,Symbols,Symbols,Symbols,Symbols,Definitions){
        let npcs=[("cow".into(),2790),("cow2".into(),2791),("cow3".into(),2793),("cow_beef".into(),2795),("cow2_calf".into(),2792),("cow3_calf".into(),2794)].into();
        let mut d=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
        for id in 2790..=2795{let calf=matches!(id,2792|2794);let mut t=gielinor_cache::npcs::NpcType::decode(&[0]).unwrap();t.name=if calf{"Cow calf"}else{"Cow"}.into();t.size=2;t.timer=1;t.content_group=if calf{45}else{43};t.stats=[1,1,1,if calf{6}else{8},1,1];t.ops[1]=Some("Attack".into());if id==2790{t.ops[0]=Some("Attack".into());}d.npcs.insert(id,t);}
        for(id,name)in[(526,"Bones"),(1739,"Cowhide"),(2132,"Raw beef")]{let mut t=gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();t.name=name.into();d.items.insert(id,t);}
        let mut bucket=gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();bucket.name="Bucket".into();d.items.insert(1925,bucket);
        (npcs,[("cow".into(),43),("cow_calf".into(),45)].into(),[("bucket_empty".into(),1925),("bones".into(),526),("cow_hide".into(),1739),("raw_beef".into(),2132)].into(),[("cow_update_graze".into(),5854)].into(),[("cow_atmospheric".into(),3044)].into(),d)
    }
    #[test]
    fn cow_drops_bind_six_ordinary_profiles_and_reject_material_drift(){
        let(n,c,i,s,a,d)=fixture();let x=cows(&n,&c,&i,&s,&a,&d).unwrap();assert_eq!(x.drops.len(),6);
        for id in 2790..=2795{let p=&x.drops[&id];assert_eq!(p.products,[(1739,1),(2132,1)]);assert_eq!(p.policy,gielinor_world::npc_death::DropPolicy::Cow);}
        for id in[526,1739,2132]{for case in 0..10{let(n,c,mut i,s,a,mut d)=fixture();match case{0=>{i.retain(|_,v|*v!=id);},1=>d.items.get_mut(&(id as u32)).unwrap().name="Not cow drops".into(),2=>d.items.get_mut(&(id as u32)).unwrap().members=true,3=>d.items.get_mut(&(id as u32)).unwrap().stackable=true,4=>d.items.get_mut(&(id as u32)).unwrap().wear[0]=3,5=>d.items.get_mut(&(id as u32)).unwrap().dummy=1,6=>d.items.get_mut(&(id as u32)).unwrap().cert.template=799,7=>d.items.get_mut(&(id as u32)).unwrap().placeholder.template=14401,8=>d.items.get_mut(&(id as u32)).unwrap().transform.template=1,_=>{d.items.remove(&(id as u32));}}assert!(cows(&n,&c,&i,&s,&a,&d).is_err(),"material{id}drift{case}");}}
    }
    #[test]
    fn cows_bind_six_profiles_and_fail_closed_on_definition_drift(){
        let(n,c,i,s,a,d)=fixture();let x=cows(&n,&c,&i,&s,&a,&d).unwrap();assert_eq!(x.flavour.types.len(),6);assert_eq!(x.replies.len(),6);assert_eq!(x.flavour.types[&2792].animation,Some(5854));assert_eq!(x.flavour.types[&2790].radius,10);
        for case in 0..10{let(mut n,mut c,i,mut s,mut a,mut d)=fixture();match case{0=>{n.remove("cow_beef");},1=>{c.insert("cow".into(),44);},2=>{s.insert("cow_update_graze".into(),5853);},3=>{a.insert("cow_atmospheric".into(),3045);},4=>d.npcs.get_mut(&2794).unwrap().timer=-1,5=>d.npcs.get_mut(&2791).unwrap().size=1,6=>d.npcs.get_mut(&2793).unwrap().ops[2]=Some("Milk".into()),7=>d.npcs.get_mut(&2792).unwrap().multi.push(Some(2794)),8=>d.npcs.get_mut(&2795).unwrap().stats[3]=80,_=>d.items.get_mut(&1925).unwrap().members=true}assert!(cows(&n,&c,&i,&s,&a,&d).is_err(),"profiledrift{case}");}
    }
}
