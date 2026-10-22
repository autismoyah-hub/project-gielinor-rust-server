//! Read-only native-client item-definition diagnostic. No connection or game action.
use gielinor_client::{disk_cache::DiskCache,item::Definitions};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let args:Vec<_>=std::env::args().collect();if args.len()!=2{return Err("usage: pg_itemops LOCAL_JS5_CACHE".into());}
 let mut cache=DiskCache::open(&args[1],64<<20)?;let definitions=Definitions::load(&mut cache)?;
 for id in[2128u32,2152,2126,30937,2142,7521]{let d=definitions.items.get(&id).ok_or("missing native item")?;println!("PG_JS5_ITEM_OPS id={id} name={:?} members={} options={:?}",d.name,d.members,d.inventory_operations);}
 Ok(())
}
