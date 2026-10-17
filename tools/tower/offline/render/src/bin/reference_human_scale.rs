//! Read-only human scale metadata; no extracted geometry is written.
use gielinor_client::{disk_cache::DiskCache,npc,npc_model,item_model};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let mut cache=DiskCache::open_legacy_reference(std::env::args().nth(1).ok_or("CACHE required")?,64<<20)?;
 let defs=npc::Definition::load_all(&mut cache)?;let d=defs.get(&3106).ok_or("NPC3106 missing")?;
 let mut models=item_model::Models::default();let mut m=npc_model::mesh(&mut cache,&mut models,d,false,None)?.ok_or("Human model unavailable")?;npc_model::resize(&mut m,d);
 let low:[f32;3]=std::array::from_fn(|i|m.vertices.iter().map(|v|v[i]).fold(f32::MAX,f32::min));
 let high:[f32;3]=std::array::from_fn(|i|m.vertices.iter().map(|v|v[i]).fold(f32::MIN,f32::max));
 println!("HUMAN_REFERENCE id={} name={:?} models={:?} scale={:?} min={low:?} max={high:?} extent_tiles={:?} vertices={} faces={}; native NPC mesh/resize; static unanimated bind pose, not live player/idle height",d.id,d.name,d.models,d.scale,std::array::from_fn::<_,3,_>(|i|(high[i]-low[i])/128.0),m.vertices.len(),m.faces.len());Ok(())
}
