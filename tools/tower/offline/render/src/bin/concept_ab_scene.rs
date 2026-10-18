//! Read-only authentic scene + original static actor comparison adapter.
//! Uses the existing native renderer. No renderer or game-state modifications.
use gielinor_client::{
    disk_cache::DiskCache,
    floor,
    model::Model,
    rebuild::Normal,
    render3d::{Camera, Frame, Triangle},
    scene::Scene,
    texture,
    world_mesh::WorldMesh,
};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, env, fs, path::Path};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<_> = env::args().collect();
    if a.len() != 7 {
        return Err("CACHE MODEL OUTPUT_DIR WORLD_X WORLD_Z quarter|gameplay|empty".into());
    }
    let x: i32 = a[4].parse()?;
    let z: i32 = a[5].parse()?;
    let out = Path::new(&a[3]);
    fs::create_dir_all(out)?;
    let mut cache = DiskCache::open_legacy_reference(&a[1], 64 << 20)?;
    let normal = Normal {
        world_area: 0,
        zone_x: (x / 8) as u16,
        zone_y: (z / 8) as u16,
        players: None,
    };
    let scene = Scene::load_normal(&mut cache, &normal)?;
    let world = WorldMesh::build(&scene, &mut cache, 0)?;
    let px = (x - scene.base_x) * 128 + 64;
    let pz = (z - scene.base_y) * 128 + 64;
    let ground = scene
        .height_fine(0, px, pz)
        .ok_or("unloaded scene ground")?;
    let mut triangles: Vec<_> = world
        .visible_triangles(0)
        .filter(|t| {
            // A small reproducible real terrain/scenery window, same for both actors.
            t.vertices
                .iter()
                .all(|v| (v[0] - px as f32).abs() < 640. && (v[2] - pz as f32).abs() < 640.)
        })
        .copied()
        .collect();
    let world_count = triangles.len();
    if a[6] != "empty" {
        let mut model = Model::decode(&fs::read(&a[2])?)?;
        model.translate([px, ground, pz]);
        let light = model.light(72, 850, [-30, -50, -30])?;
        for (i, f) in model.faces.iter().enumerate() {
            if light[i][2] == -2 {
                continue;
            }
            let vertices = f.map(|j| model.vertices[j as usize]);
            let flat = light[i][2] == -1;
            triangles.push(Triangle {
                vertices,
                local_vertices: Some(vertices),
                model_bounds: None,
                model_order: Some(gielinor_client::model_order::Submission::new(&model, i, 0)),
                depth_bias: model.face_depth_bias(i),
                colors: if flat { [light[i][0]; 3] } else { light[i] },
                flat,
                uv: [[0.; 2]; 3],
                texture: None,
                texture_mapping: None,
                alpha: model.face_alpha(i),
                near_alpha: Some(0),
                tag: 0x10000,
                view: 0,
            });
        }
    }
    let mut textures = HashMap::new();
    for (id, d) in texture::Definition::load_all(&mut cache)? {
        textures.insert(id, texture::Texture::load(&mut cache, d, 128, 0.8)?);
    }
    let focus = [px, ground - 84, pz];
    let (mut camera, width, height) = if a[6] == "gameplay" {
        let eye = gielinor_client::camera::position(focus, 2000, 14500, 600, 503, [256, 320]);
        (Camera::angles(eye, 2000, 14500, 308.)?, 765, 503)
    } else {
        (
            Camera::look_at(
                [px as f32 + 420., ground as f32 - 320., pz as f32 - 470.],
                focus.map(|v| v as f32),
                1000.,
            )?,
            1200,
            900,
        )
    };
    camera.far = 5250;
    let mut frame = Frame::new(width, height)?;
    frame.draw_iter(&camera, triangles.iter(), &floor::palette(0.8)?, &textures);
    fs::write(out.join("scene.bmp"), frame.raster()?.bmp()?)?;
    let mut report=format!("READ ONLY AUTHENTIC CACHE-BACKED SCENE\nworld={x},{z}\nbase={},{}\nactor_native_origin={px},{ground},{pz}\ncamera={}\nviewport={width}x{height}\nworld_triangles={world_count}\nactor_triangles={}\ncache_write=false\nlive_integration=false\n",scene.base_x,scene.base_y,a[6],triangles.len()-world_count);
    for id in normal.regions() {
        let raw = cache.raw_group(5, id)?;
        report.push_str(&format!(
            "region={id};raw_sha256={:x}\n",
            Sha256::digest(raw)
        ));
    }
    fs::write(out.join("identity.txt"), report)?;
    Ok(())
}
