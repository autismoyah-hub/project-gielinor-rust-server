use super::{Definitions,Symbols};
use gielinor_world::{coal_bag::Config,variables::Transmit};
pub fn coal_bag(items:&Symbols,bits:&Symbols,d:&Definitions)->Result<Config,String>{
    for(name,id)in[("coal_bag",12019),("coal_bag_open",24480),("coal",453),("skillcape_smithing",9795),("skillcape_smithing_trimmed",9796),("skillcape_max",13280)]{if items.get(name)!=Some(&id){return Err(format!("Unattested coal bag symbol {name}"));}}
    if bits.get("coal_bag_storage_count")!=Some(&65519){return Err("Unattested coal storage symbol".into());}
    let stored=*d.varbits.get(&65519).ok_or("Missing coal storage bit")?;
    let base=*d.varps.get(&65531).ok_or("Missing coal storage base")?;
    if(stored.base,stored.lsb,stored.msb)!=(65531,25,30)||base.id!=65531||!base.permanent||!base.bit_protect||base.transmit!=Transmit::Never{return Err("Unattested coal storage persistence".into());}
    for(id,name,ops)in[(12019,"Coal bag",["Fill","Open","Check","Empty","Destroy"]),(24480,"Open coal bag",["Fill","Close","Check","Empty","Destroy"])]{
        let t=d.items.get(&id).ok_or("Missing coal bag item")?;
        if t.name!=name||!t.members||!t.stackable||t.inventory_ops.iter().map(Option::as_deref).collect::<Vec<_>>()!=ops.map(Some)||t.cert.template!=0||t.placeholder.template!=0||t.transform.template!=0||t.dummy==1||!t.objvars.is_empty(){return Err("Unattested coal bag profile".into());}
    }
    let coal=d.items.get(&453).ok_or("Missing coal")?;
    if coal.name!="Coal"||coal.stackable||coal.cert.template!=0||coal.placeholder.template!=0||coal.transform.template!=0||coal.dummy==1||!coal.objvars.is_empty(){return Err("Unattested stored coal".into());}
    for id in[9795,9796,13280]{let t=d.items.get(&id).ok_or("Missing coal capacity cape")?;if t.wear[0]!=1||!t.members||t.stackable||t.cert.template!=0||t.dummy==1{return Err("Unattested coal capacity cape".into());}}
    Ok(Config{closed:12019,open:24480,coal:453,capes:[9795,9796,13280],base,stored})
}
