//! Isolated mixed native scene submission. No cache or gameplay mutations.
use super::native;
use gielinor_client::{
    animation::{Frame, Sequence, Skeleton},
    animation_runtime::{Animations, Clock},
    disk_cache::DiskCache,
    item_model,
    model::Model,
    npc, npc_model, object, object_model,
    render3d::Camera,
    texture,
};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
fn resolved(root: &Path, value: &str) -> PathBuf {
    let p = Path::new(value);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(p)
    }
}
pub fn run(
    cache: &mut DiskCache,
    textures: &HashMap<u16, texture::Texture>,
    out: &Path,
    manifest: &Path,
    camera_file: &Path,
    tick: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = manifest.parent().ok_or("Scene manifest parent")?;
    let objects = object::Definition::load_all(cache)?;
    let npcs = npc::Definition::load_all(cache)?;
    let mut om = object_model::Models::default();
    let mut items = item_model::Models::default();
    let mut animations = Animations::load(cache)?;
    let mut parts = Vec::new();
    let mut ambient = 64;
    let mut contrast = 768;
    let mut audit = String::new();
    for line in fs::read_to_string(manifest)?.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<_> = line.split('\t').collect();
        if f[0] == "light" {
            ambient = f[1].parse()?;
            contrast = f[2].parse()?;
            continue;
        }
        if f.len() < 6 {
            return Err(
                "Scene row requires kind,source,x,y,z,quarter_turns[,action_dir,travel_z_per_tick]"
                    .into(),
            );
        }
        let mut m = match f[0] {
            "owned" => {
                let mut m = Model::decode(&fs::read(resolved(root, f[1]))?)?;
                if let Some(action) = f.get(6).filter(|&&s| s != "-") {
                    let folder = resolved(root, action);
                    let sequence =
                        Sequence::decode(45000, &fs::read(folder.join("sequence.bin"))?)?;
                    if sequence.cached_animation.is_some() {
                        return Err("Scene owned animation supports classic frames only".into());
                    }
                    let mut clock = Clock::default();
                    clock.advance(&sequence, tick)?;
                    let bytes = fs::read(folder.join(format!("frame-{}.bin", clock.frame)))?;
                    let sid = u16::from_be_bytes(bytes[..2].try_into()?);
                    let sk = Skeleton::decode(
                        sid,
                        &fs::read(folder.join(format!("skeleton-{sid}.bin")))?,
                    )?;
                    let frame = Frame::decode(&bytes, &sk)?;
                    m.animate_classic(&sk, &frame, false)?;
                }
                m
            }
            "object" => {
                let id: u32 = f[1].parse()?;
                let d = objects.get(&id).ok_or("Scene object missing")?;
                let shape = if d.model_ids(10).is_empty() {
                    *d.model_shapes
                        .as_ref()
                        .and_then(|v| v.first())
                        .ok_or("Scene object shape")?
                } else {
                    10
                };
                (*om.object(cache, d, shape, 0)?.ok_or("Scene object model")?).clone()
            }
            "npc" => {
                let id: u32 = f[1].parse()?;
                let d = npcs.get(&id).ok_or("Scene NPC missing")?;
                let mut m =
                    npc_model::mesh(cache, &mut items, d, false, None)?.ok_or("Scene NPC mesh")?;
                if d.animations[0] >= 0 {
                    let seqs = Sequence::load_all(cache)?;
                    let seq = seqs
                        .get(&(d.animations[0] as u32))
                        .ok_or("Scene NPC idle")?;
                    let mut clock = Clock::default();
                    clock.advance(seq, tick)?;
                    animations.apply(cache, &mut m, d.animations[0] as u32, &clock)?;
                }
                m.scale([d.scale[0] as i32, d.scale[1] as i32, d.scale[0] as i32]);
                m
            }
            _ => return Err("Unknown scene row kind".into()),
        };
        m.rotate_quarter(f[5].parse()?);
        let travel: f32 = f.get(7).unwrap_or(&"0").parse()?;
        let offset = [
            f[2].parse()?,
            f[3].parse()?,
            f[4].parse::<i32>()? + (travel * tick as f32).round() as i32,
        ];
        m.translate(offset);
        audit.push_str(&format!(
            "{}\t{}\t{:?}\tvertices={}\tfaces={}\n",
            f[0],
            f[1],
            offset,
            m.vertices.len(),
            m.faces.len()
        ));
        parts.push(m);
    }
    if parts.is_empty() {
        return Err("Empty showcase scene".into());
    }
    let scene = Model::combine(&parts)?;
    let light = scene.light(ambient, contrast, [-30, -50, -30])?;
    let cfg = fs::read_to_string(camera_file)?;
    let f: Vec<_> = cfg.split_whitespace().collect();
    let (camera, width, height) = match f.first().copied() {
        Some("look") => {
            if f.len() != 10 {
                return Err(
                    "look eyeX eyeY eyeZ targetX targetY targetZ focal width height".into(),
                );
            }
            (
                Camera::look_at(
                    [f[1].parse()?, f[2].parse()?, f[3].parse()?],
                    [f[4].parse()?, f[5].parse()?, f[6].parse()?],
                    f[7].parse()?,
                )?,
                f[8].parse()?,
                f[9].parse()?,
            )
        }
        Some("player") => {
            if f.len() != 7 {
                return Err("player focusX focusY focusZ pitch yaw distance".into());
            }
            let focus = [f[1].parse()?, f[2].parse()?, f[3].parse()?];
            let pitch = f[4].parse()?;
            let yaw = f[5].parse()?;
            let eye = gielinor_client::camera::position(
                focus,
                pitch,
                yaw,
                f[6].parse()?,
                503,
                [256, 320],
            );
            (Camera::angles(eye, pitch, yaw, 308.)?, 765, 503)
        }
        _ => return Err("Unknown scene camera".into()),
    };
    native::render_camera(
        &scene,
        &light,
        textures,
        camera,
        width,
        height,
        &out.join("scene.bmp"),
    )?;
    fs::write(out.join("scene-submission.tsv"), audit)?;
    fs::write(out.join("scene-proof.json"),format!("{{\"tick\":{tick},\"submitted_parts\":{},\"vertices\":{},\"faces\":{},\"ambient\":{ambient},\"contrast\":{contrast},\"renderer\":\"unchanged native client\",\"cache_mutations\":false,\"live_integration\":false}}",parts.len(),scene.vertices.len(),scene.faces.len()))?;
    Ok(())
}
