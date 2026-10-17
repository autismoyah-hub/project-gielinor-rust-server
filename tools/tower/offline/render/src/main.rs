// Private offline composition tool. Uses the existing native client library unchanged.
use gielinor_client::{
    disk_cache::DiskCache,
    floor,
    rebuild::Chunk,
    render3d::{Camera, Frame},
    scene::{PlacedObject, Scene},
    terrain::Region,
    texture,
    world_mesh::WorldMesh,
};
use std::{collections::HashMap, env, fs};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<_> = env::args().collect();
    if !(7..=10).contains(&a.len()) {
        return Err(
            "Usage: tower-scene-render JS5_CACHE SCENE OUTPUT_BMP LOCAL_X LOCAL_Z YAW [DISTANCE] [detail|overview|plan] [runtime|authored]"
                .into(),
        );
    }
    let mut c = DiskCache::open(&a[1], 64 << 20)?;
    let assembly = a.get(9).map(String::as_str).unwrap_or("runtime");
    if !matches!(assembly, "runtime" | "authored") {
        return Err("Unknown object assembly mode".into());
    }
    let text = fs::read_to_string(&a[2])?;
    let mut s = Scene::empty_sized(10368 + 24, 64 + 24, 64, 64)?;
    let mut regions = HashMap::new();
    for line in text.lines().filter(|l| l.starts_with("T ")) {
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
    let mesh = WorldMesh::build(&s, &mut c, 0)?;
    let mut textures = HashMap::new();
    for (id, def) in texture::Definition::load_all(&mut c)? {
        textures.insert(id, texture::Texture::load(&mut c, def, 128, 0.8)?);
    }
    let p = floor::palette(0.8)?;
    let x: f32 = a[4].parse()?;
    let z: f32 = a[5].parse()?;
    let tx = (x - 24.0).floor() as usize;
    let tz = (z - 24.0).floor() as usize;
    let corners = [(tx, tz), (tx + 1, tz), (tx, tz + 1), (tx + 1, tz + 1)]
        .map(|(cx, cz)| s.height(0, cx, cz));
    println!("OFFLINE pose ground vertices: {corners:?}; inspect slopes before choosing a playable approach");
    let mut center = [(x - 24.0) * 128.0, -240.0, (z - 24.0) * 128.0];
    let yaw: f32 = a[6].parse::<f32>()? * std::f32::consts::TAU / 16384.0;
    let distance: f32 = a.get(7).map(|s| s.parse()).transpose()?.unwrap_or(1800.0);
    let preset = a.get(8).map(String::as_str).unwrap_or("detail");
    let (width, height, elevation, focal) = match preset {
        "detail" => (765, 503, 1300.0, 760.0),
        "overview" => (1600, 1200, 10000.0, 2000.0),
        "plan" => (1600, 1400, 12000.0, 2200.0),
        "asset-plan" => (765, 503, 1800.0, 1100.0),
        "asset-passage" => (765, 503, 80.0, 760.0),
        _ => return Err("Unknown camera preset".into()),
    };
    if preset == "asset-passage" { center[1] -= 240.0; }
    let camera = Camera::look_at(
        [
            center[0] + yaw.sin() * distance,
            center[1] - elevation,
            center[2] - yaw.cos() * distance,
        ],
        center,
        focal,
    )?;
    let mut frame = Frame::new(width, height)?;
    frame.draw(&camera, &mesh.triangles, &p, &textures);
    fs::write(&a[3], frame.raster()?.bmp()?)?;
    fs::write(format!("{}.camera.txt", a[3]), format!(
        "OFFLINE ONLY\npreset={preset}\nobject_assembly={assembly}\ncanvas={width}x{height}\nlocal_target={x},{z}\ntarget_height={}\nyaw_units={}\ndistance={distance}\nelevation={elevation}\nfocal={focal}\nscene={}\nactors=false\nlive_animation=false\nui=false\nscene_lifecycle=false\n", center[1], a[6], a[2]))?;
    println!(
        "OFFLINE shell {} triangles; pose {x},{z} yaw {yaw}; actors/UI/runtime animations absent",
        mesh.triangles.len()
    );
    Ok(())
}
