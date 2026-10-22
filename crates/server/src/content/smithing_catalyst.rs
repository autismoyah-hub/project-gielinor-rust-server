//! Attested catalyst and four existing ordinary coal-bearing furnace recipes.
use super::{Definitions,Symbols};
use gielinor_world::{smithing_catalyst::{Config,Profile},smelting::Recipe};
const RECIPES:[(&str,i32,&str,i32,i32,i32,f64);4]=[("steel_bar",2353,"iron_ore",440,2,30,17.5),("mithril_bar",2359,"mithril_ore",447,4,50,30.0),("adamantite_bar",2361,"adamantite_ore",449,6,70,37.5),("runite_bar",2363,"runite_ore",451,8,85,50.0)];
pub fn smithing_catalyst(items:&Symbols,defs:&Definitions,recipes:&[Recipe])->Result<Config,String>{
    if items.get("smithing_catalyst")!=Some(&27017)||items.get("coal")!=Some(&453){return Err("Unattested catalyst symbols".into());}
    let catalyst=defs.items.get(&27017).ok_or("Missing catalyst")?;
    if catalyst.name!="Smithing catalyst"||!catalyst.members||catalyst.wear!=[-1,-1,-1]||catalyst.inventory_ops.iter().any(Option::is_some){return Err("Unattested catalyst profile".into());}
    let mut ids=vec![27017,453];let mut profiles=std::collections::BTreeMap::new();
    for(bar_name,bar,ore_name,ore,coal,level,xp)in RECIPES{
        if items.get(bar_name)!=Some(&bar)||items.get(ore_name)!=Some(&ore){return Err("Unattested catalyst ore/bar symbols".into());}
        let mut matches=recipes.iter().filter(|r|r.bar==bar);let r=matches.next().ok_or("Configure ordinary coal bars before catalyst")?;
        if matches.next().is_some()||r.ingredients!=[(ore,1),(453,coal)]||r.level!=level||r.xp!=xp||r.impurity||r.members{return Err("Conflicting ordinary catalyst recipe".into());}
        profiles.insert(bar,Profile{ore,coal,level,xp});ids.extend([ore as u32,bar as u32]);
    }
    let names:[(u32,&str);9]=[(453,"Coal"),(440,"Iron ore"),(447,"Mithril ore"),(449,"Adamantite ore"),(451,"Runite ore"),(2353,"Steel bar"),(2359,"Mithril bar"),(2361,"Adamantite bar"),(2363,"Runite bar")];
    for id in ids {let t=defs.items.get(&id).ok_or("Missing catalyst material")?;if id!=27017&&(names.iter().find(|(raw,_)|*raw==id).is_none_or(|(_,name)|t.name!=*name)||t.members||t.wear!=[-1,-1,-1]){return Err("Unattested catalyst ore/bar profile".into());}if t.stackable||t.dummy==1||t.cert.template!=0||t.placeholder.template!=0||t.transform.template!=0||!t.objvars.is_empty(){return Err(format!("Unattested catalyst material {id}"));}}
    Ok(Config{item:27017,coal:453,profiles})
}
#[cfg(test)]
mod tests {
    use super::*;
    fn prepared()->(Symbols,Definitions,Vec<Recipe>){
        let mut d=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};let mut n=Symbols::new();n.insert("smithing_catalyst".into(),27017);n.insert("coal".into(),453);
        let names=[(27017,"Smithing catalyst"),(453,"Coal"),(440,"Iron ore"),(447,"Mithril ore"),(449,"Adamantite ore"),(451,"Runite ore"),(2353,"Steel bar"),(2359,"Mithril bar"),(2361,"Adamantite bar"),(2363,"Runite bar")];for(id,name)in names{let mut t=gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();t.name=name.into();t.members=id==27017;d.items.insert(id,t);}
        let recipes=RECIPES.into_iter().map(|(bar_name,bar,ore_name,ore,coal,level,xp)|{n.insert(bar_name.into(),bar);n.insert(ore_name.into(),ore);Recipe{bar,label:bar_name.into(),level,xp,ingredients:vec![(ore,1),(453,coal)],impurity:false,members:false,repeat_delay:None}}).collect();(n,d,recipes)
    }
    #[test]
    fn catalyst_binding_attests_consumable_and_four_unchanged_recipe_profiles(){
        let(n,d,r)=prepared();let c=smithing_catalyst(&n,&d,&r).unwrap();assert_eq!((c.item,c.coal,c.profiles.len()),(27017,453,4));
        for id in [27017,453,440,447,449,451,2353,2359,2361,2363]{for change in 0..10{let(n,mut d,r)=prepared();let t=d.items.get_mut(&id).unwrap();match change{0=>t.name="Other".into(),1=>t.members=!t.members,2=>t.wear=[9,-1,-1],3=>t.stackable=true,4=>t.dummy=1,5=>t.cert.template=799,6=>t.placeholder.template=799,7=>t.transform.template=799,8=>t.objvars.push(1),_=>{if id!=27017{continue;}t.inventory_ops[0]=Some("Other".into());}}assert!(smithing_catalyst(&n,&d,&r).is_err(),"id{id}change{change}");}}
        for index in 0..4{for change in 0..8{let(mut n,d,mut r)=prepared();match change{0=>r[index].xp+=1.0,1=>r[index].level+=1,2=>r[index].ingredients[1].1+=1,3=>r[index].impurity=true,4=>r[index].members=true,5=>r.push(r[index].clone()),6=>{r.remove(index);},_=>{n.retain(|_,v|*v!=r[index].bar);}}assert!(smithing_catalyst(&n,&d,&r).is_err(),"recipe{index}change{change}");}}
    }
}
