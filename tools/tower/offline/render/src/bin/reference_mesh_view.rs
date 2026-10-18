//! Original-art reference camera submission; unchanged native renderer APIs.
#[path = "support/workbench_native.rs"]
mod native;
use gielinor_client::{disk_cache::DiskCache,model::Model,render3d::Camera};
use std::{env,fs,path::Path};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let a:Vec<_>=env::args().collect();if a.len()!=5{return Err("CACHE ORIGINAL_MODEL OUTPUT CAMERA_FILE".into());}
 let out=Path::new(&a[3]);fs::create_dir_all(out)?;
 let mut cache=DiskCache::open_legacy_reference(&a[1],64<<20)?;
 let textures=native::textures(&mut cache)?;let m=Model::decode(&fs::read(&a[2])?)?;
 let text=fs::read_to_string(&a[4])?;let f:Vec<_>=text.split_whitespace().collect();
 if f.len()!=12||f[0]!="look"{return Err("look EX EY EZ TX TY TZ FOCAL WIDTH HEIGHT OFFSET_X OFFSET_Y".into());}
 let mut camera=Camera::look_at([f[1].parse()?,f[2].parse()?,f[3].parse()?],[f[4].parse()?,f[5].parse()?,f[6].parse()?],f[7].parse()?)?;
 camera.screen_offset=[f[10].parse()?,f[11].parse()?];
 let light=m.light(72,850,[-30,-50,-30])?;
 native::render_camera(&m,&light,&textures,camera,f[8].parse()?,f[9].parse()?,&out.join("native.bmp"))?;
 fs::write(out.join("decoded.json"),native::geometry(&m))?;fs::write(out.join("submitted-camera.txt"),text)?;Ok(())
}
