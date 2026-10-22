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
#[cfg(test)]
mod tests{
    use super::*;
    use gielinor_world::{simulation::{World,Action,PlayerData},inventory::Item,stats::Stats,variables::{Varp,Varbit},CoordGrid};
    #[test]
    fn coal_bag_actual_ifbuttonx_fourth_option_six_empties_and_reserved_five_cannot_transfer(){
        let config=Config{closed:12019,open:24480,coal:453,capes:[9795,9796,13280],base:Varp{id:65531,permanent:true,transmit:Transmit::Never,bit_protect:true,client_code:-1},stored:Varbit{base:65531,lsb:25,msb:30}};
        for operation in[5,6]{let mut payload=(149u32<<16).to_be_bytes().to_vec();payload.extend(27u16.to_be_bytes());payload.extend(12019u16.to_be_bytes());payload.push(operation);let button=gielinor_protocol::buttons::button(&payload).unwrap();assert_eq!(config.accepts_operation(button.op),operation==6);
            let mut w=World::default();w.coal_bag=Some(config);w.collision.allocate(3200,3200,0);let mut p=PlayerData::new(1,"Coal packet boundary".into(),CoordGrid::new(3200,3200,0).unwrap(),Stats::default());p.inventory.restore_slot(27,Some(Item{id:12019,count:1,vars:0})).unwrap();p.variables.set_bit(config.base,config.stored,2,false).unwrap();w.queue_login(1,p).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();w.set_membership(a,true).unwrap();w.submit(a,Action::CoalBagItem{slot:button.slot as usize,item:button.object,operation:button.op}).unwrap();w.tick().unwrap();assert_eq!(w.data(a).unwrap().variables.bit(config.stored).unwrap(),if operation==6{0}else{2});assert_eq!(w.data(a).unwrap().inventory.slots().iter().flatten().filter(|i|i.id==453).count(),if operation==6{2}else{0});
        }
    }
}
