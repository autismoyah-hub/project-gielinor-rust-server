//! Target-cache binding for Goldsmith gauntlets and the three source cape forms.
use super::{Definitions,Symbols};
use gielinor_world::{goldsmith_smelting::Config,smelting::Recipe};
const TYPES:[(&str,i32,&str,i8,bool);6]=[("gauntlets_of_goldsmithing",776,"Goldsmith gauntlets",9,true),("skillcape_smithing",9795,"Smithing cape",1,true),("skillcape_smithing_trimmed",9796,"Smithing cape(t)",1,true),("skillcape_max",13280,"Max cape",1,true),("gold_ore",444,"Gold ore",-1,false),("gold_bar",2357,"Gold bar",-1,false)];
pub fn goldsmith_smelting(items:&Symbols,defs:&Definitions,recipes:&[Recipe])->Result<Config,String>{
    for(sym,id,name,slot,members)in TYPES {
        let t=defs.items.get(&(id as u32)).ok_or("Missing goldsmith item")?;
        if items.get(sym)!=Some(&id)||t.name!=name||t.wear!=[slot,-1,-1]||t.members!=members||t.stackable||t.dummy==1||t.cert.template!=0||t.placeholder.template!=0||t.transform.template!=0||!t.objvars.is_empty(){return Err(format!("Unattested goldsmith item {id}"));}
        if slot>=0&&!t.inventory_ops.iter().any(|op|op.as_deref()==Some("Wear")){return Err("Missing goldsmith wear operation".into());}
    }
    let mut gold=recipes.iter().filter(|r|r.bar==2357);let r=gold.next().ok_or("Configure ordinary gold smelting before bonus")?;
    if gold.next().is_some()||r.ingredients!=[(444,1)]||r.level!=40||r.xp!=22.5||r.impurity||r.members{return Err("Conflicting ordinary gold recipe".into());}
    Ok(Config{ore:444,bar:2357,gauntlets:776,capes:[9795,9796,13280]})
}
#[cfg(test)]
mod tests {
    use super::*;
    fn prepared()->(Symbols,Definitions,Vec<Recipe>){
        let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};let mut names=Symbols::new();
        for(sym,id,name,slot,members)in TYPES{let mut t=gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();t.name=name.into();t.members=members;t.wear=[slot,-1,-1];if slot>=0{t.inventory_ops[1]=Some("Wear".into());}defs.items.insert(id as u32,t);names.insert(sym.into(),id);}
        let recipe=Recipe{bar:2357,label:"Gold".into(),level:40,xp:22.5,ingredients:vec![(444,1)],impurity:false,members:false,repeat_delay:None};(names,defs,vec![recipe])
    }
    #[test]
    fn goldsmith_binding_attests_six_target_types_and_exact_existing_gold_recipe(){
        let(n,d,r)=prepared();let c=goldsmith_smelting(&n,&d,&r).unwrap();assert_eq!((c.ore,c.bar,c.gauntlets,c.capes),(444,2357,776,[9795,9796,13280]));
        for(_,id,_,slot,_)in TYPES{for change in 0..11{if change==10&&slot<0{continue;}let(mut n,mut d,r)=prepared();let t=d.items.get_mut(&(id as u32)).unwrap();match change{0=>{n.retain(|_,v|*v!=id);},1=>t.name="Other".into(),2=>t.wear=[8,-1,-1],3=>t.members=!t.members,4=>t.stackable=true,5=>t.dummy=1,6=>t.cert.template=799,7=>t.placeholder.template=799,8=>t.transform.template=799,9=>t.objvars.push(1),_=>t.inventory_ops.fill(None)};assert!(goldsmith_smelting(&n,&d,&r).is_err(),"id{id}change{change}");}}
        for change in 0..7{let(n,d,mut r)=prepared();match change{0=>r[0].xp=22.0,1=>r[0].level=39,2=>r[0].ingredients=vec![(453,1)],3=>r[0].impurity=true,4=>r[0].members=true,5=>r.push(r[0].clone()),_=>r.clear()};assert!(goldsmith_smelting(&n,&d,&r).is_err(),"recipe{change}");}
    }
}
