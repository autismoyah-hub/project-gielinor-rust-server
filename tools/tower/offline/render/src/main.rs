// Private offline composition tool. Uses the existing native client library unchanged.
use gielinor_client::{
    disk_cache::DiskCache,
    floor,
    rebuild::{Chunk,Normal},
    render3d::{Camera, Frame},
    scene::{PlacedObject, Scene},
    terrain::Region,
    texture,
    world_mesh::WorldMesh,
};
use std::{collections::HashMap, env, fs};
use sha2::{Digest,Sha256};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<_> = env::args().collect();
    if !(7..=10).contains(&a.len()) {
        return Err(
            "Usage: tower-scene-render JS5_CACHE SCENE OUTPUT_BMP LOCAL_X LOCAL_Z YAW [DISTANCE] [detail|overview|plan] [runtime|authored]"
                .into(),
        );
    }
    let original_only=env::var_os("TOWER_ORIGINAL_ART_ONLY").is_some();
    let legacy_reference=env::var_os("TOWER_LEGACY_REFERENCE_ONLY").is_some();
    let mut c = if original_only {DiskCache::open_package_only(&a[1],64<<20)?} else if legacy_reference {DiskCache::open_legacy_reference(&a[1],64<<20)?} else {DiskCache::open(&a[1], 64 << 20)?};
    let assembly = a.get(9).map(String::as_str).unwrap_or("runtime");
    if !matches!(assembly, "runtime" | "authored") {
        return Err("Unknown object assembly mode".into());
    }
    let text = fs::read_to_string(&a[2])?;
    if text.lines().any(|l|l.starts_with("P ")) {
        if !original_only{return Err("P original floor definitions require original-only study".into());}
        let (mut underlays,mut overlays)=(Vec::new(),Vec::new());
        for line in text.lines().filter(|l|l.starts_with("P ")) {
            let v:Vec<_>=line.split_whitespace().collect();
            if v.len()!=4{return Err("P U|O ID RGB_HEX".into());}
            let id:u32=v[2].parse()?;let rgb=u32::from_str_radix(v[3],16)?;
            if rgb>0xffffff{return Err("Original floor RGB bound".into());}
            let bytes=vec![1,(rgb>>16) as u8,(rgb>>8) as u8,rgb as u8,0];
            match v[1]{"U"=>underlays.push((id,bytes)),"O"=>overlays.push((id,bytes)),_=>return Err("Original floor kind".into())}
        }
        c.set_original_floors(underlays,overlays)?;
    }
    let dimensions=text.lines().find(|l|l.starts_with("S ")).map(|l|l.split_whitespace().skip(1).map(str::parse::<usize>).collect::<Result<Vec<_>,_>>()).transpose()?.unwrap_or(vec![64,64]);
    if dimensions.len()!=2 {return Err("S requires width height".into());}
    let mut s = Scene::empty_sized(10368 + 24, 64 + 24, dimensions[0], dimensions[1])?;
    let normal=text.lines().find(|l|l.starts_with("N ")).map(|l|l.split_whitespace().skip(1).map(str::parse::<u16>).collect::<Result<Vec<_>,_>>()).transpose()?;
    let mut draw_plane=0;
    if let Some(v)=&normal {
        if original_only || v.len()!=3 || v[2]>3 {return Err("N requires real-cache world X Z plane".into());}
        draw_plane=v[2] as usize;
        let rebuild=Normal{world_area:0,zone_x:v[0]/8,zone_y:v[1]/8,players:None};
        s=Scene::load_normal(&mut c,&rebuild)?;
        let defs=gielinor_client::object::Definition::load_all(&mut c)?;
        let mut inventory=std::collections::BTreeMap::<u32,usize>::new();
        for obj in &s.objects {*inventory.entry(obj.id).or_default()+=1;}
        let mut report=format!("OFFLINE AUTHENTIC STATIC NORMAL SCENE\nworld_focus={:?}\nscene_base={},{}\nregions={:?}\nmissing_regions={:?}\nobjects={}\n",v,s.base_x,s.base_y,rebuild.regions(),s.missing_regions,s.objects.len());
        for id in rebuild.regions() {
            if let Some(info)=c.reference(5)?.groups.iter().find(|g|g.id==id).cloned(){
                let raw=c.raw_group(5,id)?;
                report.push_str(&format!("source_group={id};crc={:08x};version={};raw_sha256={:x}\n",info.crc,info.version,Sha256::digest(&raw)));
            }
        }
        let mut placements=String::from("world_x\tworld_z\tplane\tobject_id\tshape\trotation\tname\tmodels\n");
        for obj in &s.objects {if let Some(d)=defs.get(&obj.id){placements.push_str(&format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:?}\n",s.base_x+obj.x as i32,s.base_y+obj.y as i32,obj.plane,obj.id,obj.shape,obj.rotation,d.name,d.models));}}
        fs::write(format!("{}.local-placements.tsv",a[3]),placements)?;
        report.push_str("object_id\tcount\tname\tmodels\ttextures_recolor_not_expanded_here\n");
        for (id,count) in inventory {if let Some(d)=defs.get(&id){report.push_str(&format!("{id}\t{count}\t{}\t{:?}\n",d.name,d.models));}}
        fs::write(format!("{}.inventory.txt",a[3]),report)?;
    }
    let mut regions = HashMap::new();
    if text.lines().any(|l|l.starts_with("A ")) {
        if !original_only || normal.is_some() || dimensions.iter().any(|&n|n>64 || n%8!=0){return Err("A authored native terrain requires <=64-tile original-only grid".into());}
        let mut region=Region{tiles:vec![gielinor_client::terrain::Tile::default();4*4096],extension_flags:0,height_rules:vec![Some(0);4*4096]};
        for line in text.lines().filter(|l|l.starts_with("A ")) {
            let v=line.split_whitespace().skip(1).map(str::parse::<u16>).collect::<Result<Vec<_>,_>>()?;
            if v.len()!=7 || v[0]<24 || v[1]<24 || v[0]-24>=dimensions[0] as u16 || v[1]-24>=dimensions[1] as u16 || v[2]>255 || v[3]>32767 || v[4]>32767 || v[5]>11 || v[6]>3 {return Err("A x z explicit-height-delta underlay overlay shape rotation bounds".into());}
            let i=(v[0] as usize-24)*64+v[1] as usize-24;
            region.tiles[i].underlay=v[3];region.tiles[i].overlay=v[4] as i16;region.tiles[i].overlay_shape=v[5] as u8;region.tiles[i].overlay_rotation=v[6] as u8;region.height_rules[i]=Some(v[2] as u8);
        }
        for x in 0..dimensions[0]/8 {for z in 0..dimensions[1]/8 {
            s.add_chunk(&region,Chunk{plane:0,zone_x:x as u16,zone_y:z as u16,rotation:0},0,x*8,z*8)?;
        }}
    }
    for line in text.lines().filter(|l| l.starts_with("T ")) {
        if original_only {return Err("Original-art-only studies cannot copy cache terrain".into());}
        let v = line
            .split_whitespace()
            .skip(1)
            .map(str::parse::<u16>)
            .collect::<Result<Vec<_>, _>>()?;
        let (zx, zy) = (v[3], v[4]);
        let square = ((zx as u32 / 8) << 8) | (zy as u32 / 8);
        if !regions.contains_key(&square) {
            let files = c.group(5, square, [0; 4])?;
            let bytes = files
                .iter()
                .find(|(id, _)| *id == 0)
                .ok_or("Missing native terrain")?;
            regions.insert(
                square,
                Region::decode(&bytes.1, (zx / 8) as i32 * 64, (zy / 8) as i32 * 64)?,
            );
        }
        s.add_chunk(
            &regions[&square],
            Chunk {
                plane: v[5] as u8,
                zone_x: zx,
                zone_y: zy,
                rotation: v[6] as u8,
            },
            v[2] as usize,
            v[0] as usize * 8 - 24,
            v[1] as usize * 8 - 24,
        )?;
    }
    for line in text.lines().filter(|l| l.starts_with("L ")) {
        let v = line
            .split_whitespace()
            .skip(1)
            .map(str::parse::<u32>)
            .collect::<Result<Vec<_>, _>>()?;
        let initial_index = (assembly == "authored").then_some(s.objects.len());
        s.objects.push(PlacedObject {
            initial_index,
            rejected: false,
            runtime: assembly == "runtime",
            id: v[3],
            plane: v[2] as u8,
            x: (v[0] - 24) as u8,
            y: (v[1] - 24) as u8,
            shape: v[4] as u8,
            rotation: v[5] as u8,
        });
    }
    if text.lines().any(|l|l.starts_with("A ")) {
        let definitions=gielinor_client::object::Definition::load_all(&mut c)?;
        s.add_object_collision(&definitions)?;
    }
    let mesh = WorldMesh::build(&s, &mut c, draw_plane)?;
    println!("STATIC_SOURCE objects={} missing_regions={:?} mesh_pending={} textured_triangles={}",s.objects.len(),s.missing_regions,mesh.pending_assets,mesh.triangles.iter().filter(|t|t.texture.is_some()).count());
    let mut textures = HashMap::new();
    if !original_only {for (id, def) in texture::Definition::load_all(&mut c)? {
        textures.insert(id, texture::Texture::load(&mut c, def, 128, 0.8)?);
    }}
    let p = floor::palette(0.8)?;
    let x: f32 = a[4].parse()?;
    let z: f32 = a[5].parse()?;
    let lx=if normal.is_some(){x-s.base_x as f32}else{x-24.0};
    let lz=if normal.is_some(){z-s.base_y as f32}else{z-24.0};
    let tx = lx.floor() as usize;
    let tz = lz.floor() as usize;
    let corners = [(tx, tz), (tx + 1, tz), (tx, tz + 1), (tx + 1, tz + 1)]
        .map(|(cx, cz)| s.height(draw_plane, cx, cz));
    println!("OFFLINE pose ground vertices: {corners:?}; inspect slopes before choosing a playable approach");
    let mut center = [lx * 128.0, -240.0, lz * 128.0];
    let yaw: f32 = a[6].parse::<f32>()? * std::f32::consts::TAU / 16384.0;
    let distance: f32 = a.get(7).map(|s| s.parse()).transpose()?.unwrap_or(1800.0);
    let preset = a.get(8).map(String::as_str).unwrap_or("detail");
    let (width, height, mut elevation, mut focal) = match preset {
        "detail" => (765, 503, 1300.0, 760.0),
        "overview" => (1600, 1200, 10000.0, 2000.0),
        "plan" => (1600, 1400, 12000.0, 2200.0),
        "asset-plan" => (765, 503, 1800.0, 1100.0),
        "asset-passage" => (765, 503, 80.0, 760.0),
        "floor-iso" => (1800, 1600, 10000.0, 1800.0),
        "floor-plan" => (1800, 1900, 17500.0, 2200.0),
        "room" => (1200, 900, 2200.0, 1250.0),
        "player" => (1200, 900, 1100.0, 1000.0),
        "reference" => (1100, 850, 650.0, 1100.0),
        "matched" => (765, 503, 0.0, 1.0),
        _ => return Err("Unknown camera preset".into()),
    };
    if preset == "asset-passage" { center[1] -= 240.0; }
    if original_only {center[1]=if preset=="player" {-220.0} else {-80.0};}
    if let Ok(height)=env::var("TOWER_TARGET_HEIGHT") {
        let height:f32=height.parse()?;
        if !height.is_finite() || height.abs()>8192.0 {return Err("Invalid offline target height".into());}
        center[1]=height;
    }
    let mut camera = if preset=="matched" {Camera::angles([0;3],2048,0,1.0)?} else {Camera::look_at(
        [
            center[0] + yaw.sin() * distance,
            center[1] - elevation,
            center[2] - yaw.cos() * distance,
        ],
        center,
        focal,
    )?};
    let mut matched_metadata=String::new();
    if preset=="matched" {
        let settings=gielinor_client::viewport_shape::Settings::default();
        let shape=settings.shape([width as i32,height as i32]);
        let pitch=env::var("TOWER_MATCHED_PITCH").ok().map(|s|s.parse::<i32>()).transpose()?.unwrap_or(2048);
        if !(1024..=3064).contains(&pitch){return Err("Matched pitch outside normal client limits".into());}
        let focus=[(lx*128.0) as i32,s.height_fine(draw_plane,(lx*128.0) as i32,(lz*128.0) as i32).unwrap_or(0)-50,(lz*128.0) as i32];
        let yaw=a[6].parse::<i32>()?;
        let orbit=if distance>0.0{distance as i32}else{600+3*(pitch/8)};
        let eye=gielinor_client::camera::position(focus,pitch,yaw,orbit,shape.size[1],settings.zoom.map(i32::from));
        camera=Camera::angles(eye,pitch,yaw,shape.focal as f32)?;
        camera.far=gielinor_client::device_options::DeviceOptions::default().far_clip();
        center=focus.map(|v|v as f32);elevation=(focus[1]-eye[1]) as f32;focal=shape.focal as f32;
        matched_metadata=format!("matched_pitch={pitch}\nmatched_yaw={yaw}\nmatched_focus={focus:?}\nmatched_eye={eye:?}\nmatched_focal={}\nmatched_orbit={orbit}\nmatched_zoom={:?}\nmatched_far={}\nroof_max_plane={draw_plane}\n",shape.focal,settings.zoom,camera.far);
    }
    let mut frame = Frame::new(width, height)?;
    frame.draw_iter(&camera, mesh.visible_triangles(draw_plane as u8), &p, &textures);
    fs::write(&a[3], frame.raster()?.bmp()?)?;
    fs::write(format!("{}.camera.txt", a[3]), format!(
        "OFFLINE ONLY\noriginal_art_only={original_only}\npreset={preset}\nobject_assembly={assembly}\ncanvas={width}x{height}\nlocal_target={x},{z}\ntarget_height={}\nyaw_units={}\ndistance_argument={distance}\nelevation={elevation}\nfocal={focal}\nscene={}\nactors=false\nlive_animation=false\nui=false\nscene_lifecycle=false\n", center[1], a[6], a[2]))?;
    if !matched_metadata.is_empty(){let path=format!("{}.camera.txt",a[3]);let mut metadata=fs::read_to_string(&path)?;metadata.push_str(&matched_metadata);fs::write(path,metadata)?;}
    println!(
        "OFFLINE shell {} triangles; pose {x},{z} yaw {yaw}; actors/UI/runtime animations absent",
        mesh.triangles.len()
    );
    Ok(())
}
