//! Batch read-only reference inspection and native rendering for asset authoring.
#[path = "support/workbench_native.rs"]
mod native;
#[path = "support/workbench_scene.rs"]
mod scene;
use gielinor_client::{
    animation::{Frame, Sequence, Skeleton},
    animation_runtime::{Animations, Clock},
    disk_cache::DiskCache,
    item, item_model,
    model::Model,
    npc, npc_model, object, object_model,
};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, env, fs, path::Path};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<_> = env::args().collect();
    if a.len() < 5 {
        return Err("asset_workbench CACHE OUTPUT inspect SELECTOR... | render MODEL CAMERA_FILE AMBIENT CONTRAST | animate SELECTOR SEQUENCE".into());
    }
    let out = Path::new(&a[2]);
    fs::create_dir_all(out)?;
    let mut cache = DiskCache::open_legacy_reference(&a[1], 64 << 20)?;
    let tex = native::textures(&mut cache)?;
    fs::write(
        out.join("palette.json"),
        format!("{:?}", gielinor_client::floor::palette(0.8)?),
    )?;
    if a[3] == "scene" {
        if a.len() < 6 {
            return Err("scene MANIFEST CAMERA [TICK]".into());
        }
        scene::run(
            &mut cache,
            &tex,
            out,
            Path::new(&a[4]),
            Path::new(&a[5]),
            a.get(6).map(|s| s.parse()).transpose()?.unwrap_or(0),
        )?;
        return Ok(());
    }
    if a[3] == "texture-stats" {
        let palette = gielinor_client::floor::palette(0.8)?;
        for arg in &a[4..] {
            let id: u16 = arg.parse()?;
            let t = tex.get(&id).ok_or("Missing texture")?;
            let mut mean = [0f64; 3];
            for &rgb in &t.pixels {
                for (i, shift) in [16, 8, 0].iter().enumerate() {
                    mean[i] += ((rgb >> shift) & 255) as f64;
                }
            }
            for v in &mut mean {
                *v /= t.pixels.len() as f64;
            }
            let best = palette
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    let cost = |rgb: u32| {
                        [16, 8, 0]
                            .iter()
                            .enumerate()
                            .map(|(i, s)| (((rgb >> s) & 255) as f64 - mean[i]).powi(2))
                            .sum::<f64>()
                    };
                    cost(**a).total_cmp(&cost(**b))
                })
                .ok_or("Empty palette")?
                .0;
            fs::write(out.join(format!("texture-{id}.json")),format!("{{\"texture_id\":{id},\"sampled_native_pixels\":{},\"mean_rgb\":{:?},\"nearest_unlit_native_palette_code\":{best},\"basis\":\"aggregate color proxy only; does not reproduce source texture or grain\"}}",t.pixels.len(),mean))?;
        }
        return Ok(());
    }
    if a[3] == "package" {
        let mut package = DiskCache::open_package_only(&a[4], 64 << 20)?;
        let id: u32 = a[5].parse()?;
        let shape: u8 = a[6].parse()?;
        let defs = object::Definition::load_all(&mut package)?;
        let d = defs.get(&id).ok_or("Custom definition missing")?;
        let m = object_model::Models::default()
            .object(&mut package, d, shape, 0)?
            .ok_or("Custom package model unavailable")?;
        let cam: Vec<f32> = fs::read_to_string(&a[7])?
            .split_whitespace()
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        let light = m.light(a[8].parse()?, a[9].parse()?, [-30, -50, -30])?;
        native::views(
            &m,
            &light,
            &tex,
            cam.try_into().map_err(|_| "Camera length")?,
            out,
        )?;
        fs::write(out.join("decoded.json"), native::geometry(&m))?;
        return Ok(());
    }
    if a[3] == "play" {
        if a.len() < 9 {
            return Err("play MODEL CAMERA AMBIENT CONTRAST SEQUENCE_DIR".into());
        }
        let m = Model::decode(&fs::read(&a[4])?)?;
        let cam: Vec<f32> = fs::read_to_string(&a[5])?
            .split_whitespace()
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        let cam: [f32; 4] = cam.try_into().map_err(|_| "Camera length")?;
        let light = m.light(a[6].parse()?, a[7].parse()?, [-30, -50, -30])?;
        let dir = Path::new(&a[8]);
        let sequence = Sequence::decode(45000, &fs::read(dir.join("sequence.bin"))?)?;
        if sequence.cached_animation.is_some() {
            return Err("Custom cached animation playback not supported".into());
        }
        let mut times: BTreeSet<_> = (0..=sequence.total_length.min(256)).collect();
        times.insert(sequence.total_length);
        times.extend([
            sequence.total_length / 4,
            sequence.total_length / 2,
            sequence.total_length * 3 / 4,
        ]);
        for tick in times {
            let mut clock = Clock::default();
            clock.advance(&sequence, tick)?;
            let bytes = fs::read(dir.join(format!("frame-{}.bin", clock.frame)))?;
            let sid = u16::from_be_bytes(bytes.get(..2).ok_or("Truncated frame")?.try_into()?);
            let skeleton =
                Skeleton::decode(sid, &fs::read(dir.join(format!("skeleton-{sid}.bin")))?)?;
            let frame = Frame::decode(&bytes, &skeleton)?;
            let mut posed = m.clone();
            posed.animate_classic(&skeleton, &frame, false)?;
            if a.len() >= 12 {
                posed.scale([a[9].parse()?, a[10].parse()?, a[11].parse()?]);
            }
            fs::write(
                out.join(format!("pose-{tick:04}.json")),
                native::geometry(&posed),
            )?;
            native::render(
                &posed,
                &light,
                &tex,
                cam,
                0.65,
                &out.join(format!("pose-{tick:04}.bmp")),
                false,
            )?;
        }
        return Ok(());
    }
    if a[3] == "mask" {
        if a.len() < 7 {
            return Err("mask MODEL CAMERA FACE_SELECTION [SCALE_X SCALE_Y SCALE_Z]".into());
        }
        let mut m = Model::decode(&fs::read(&a[4])?)?;
        let selected: BTreeSet<usize> = fs::read_to_string(&a[6])?
            .split_whitespace()
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        if selected.iter().any(|&i| i >= m.faces.len()) {
            return Err("Component mask face index out of range".into());
        }
        let original = m.light(64, 768, [-30, -50, -30])?;
        let light: Vec<_> = original
            .iter()
            .enumerate()
            .map(|(i, l)| {
                if l[2] == -2 {
                    [0, 0, -2]
                } else {
                    [if selected.contains(&i) { 127 } else { 0 }, 0, -1]
                }
            })
            .collect();
        m.face_textures = None;
        m.texture_coords = None;
        if a.len() >= 10 {
            m.scale([a[7].parse()?, a[8].parse()?, a[9].parse()?]);
        }
        let cam: Vec<f32> = fs::read_to_string(&a[5])?
            .split_whitespace()
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        native::views(
            &m,
            &light,
            &tex,
            cam.try_into().map_err(|_| "Mask camera length")?,
            out,
        )?;
        return Ok(());
    }
    if a[3] == "render-own" {
        if a.len() != 7 {
            return Err("render-own MODEL AMBIENT CONTRAST".into());
        }
        let m = Model::decode(&fs::read(&a[4])?)?;
        let light = m.light(a[5].parse()?, a[6].parse()?, [-30, -50, -30])?;
        let cam = native::camera(&m);
        fs::write(
            out.join("camera.txt"),
            cam.iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        )?;
        native::views(&m, &light, &tex, cam, out)?;
        fs::write(out.join("decoded.json"), native::geometry(&m))?;
        return Ok(());
    }
    if a[3] == "render" {
        let m = Model::decode(&fs::read(&a[4])?)?;
        let cam: Vec<f32> = fs::read_to_string(&a[5])?
            .split_whitespace()
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        if cam.len() != 4 {
            return Err("Camera contract requires four numbers".into());
        }
        let light = m.light(a[6].parse()?, a[7].parse()?, [-30, -50, -30])?;
        let mut world = m.clone();
        if a.len() >= 11 {
            world.scale([a[8].parse()?, a[9].parse()?, a[10].parse()?]);
        }
        native::views(&world, &light, &tex, cam.try_into().unwrap(), out)?;
        fs::write(out.join("decoded.json"), native::geometry(&m))?;
        return Ok(());
    }
    let objects = object::Definition::load_all(&mut cache)?;
    let npcs = npc::Definition::load_all(&mut cache)?;
    let items = item::Definitions::load(&mut cache)?;
    let sequences = Sequence::load_all(&mut cache)?;
    if a[3] == "catalog-npcs" {
        let mut definitions: Vec<_> = npcs.values().collect();
        definitions.sort_by_key(|d| d.id);
        let mut rows = String::from("id\tname\tmodels\tanimations\ttransforms\tsize\tcombat\n");
        for d in definitions {
            rows.push_str(&format!(
                "{}\t{:?}\t{:?}\t{:?}\t{:?}\t{}\t{}\n",
                d.id, d.name, d.models, d.animations, d.transforms, d.size, d.combat
            ));
            if a[4..].iter().any(|id| id.parse::<u32>().ok() == Some(d.id)) {
                fs::write(
                    out.join(format!("npc-{}-definition.txt", d.id)),
                    format!("{d:#?}"),
                )?;
            }
        }
        fs::write(out.join("npc-catalog.tsv"), rows)?;
        return Ok(());
    }
    if a[3] == "catalog-sequences" {
        let mut definitions: Vec<_> = sequences.iter().collect();
        definitions.sort_by_key(|(id, _)| **id);
        let mut rows =
            String::from("id\tclassic_frame_count\tduration\tcached_animation\tcached_range\n");
        for (id, d) in definitions {
            rows.push_str(&format!(
                "{id}\t{}\t{}\t{:?}\t{:?}\n",
                d.frames.len(),
                d.total_length,
                d.cached_animation,
                d.cached_range
            ));
            if a[4..]
                .iter()
                .any(|value| value.parse::<u32>().ok() == Some(*id))
            {
                fs::write(
                    out.join(format!("sequence-{id}-definition.txt")),
                    format!("{d:#?}"),
                )?;
            }
        }
        fs::write(out.join("sequence-catalog.tsv"), rows)?;
        return Ok(());
    }
    if a[3] == "beside" {
        // One shared coordinate system, camera, light and native submission.
        // Cache models remain read-only; this scene and its pixels are private.
        if a.len() != 6 {
            return Err("beside SELECTOR CUSTOM_MODEL".into());
        }
        let s: Vec<_> = a[4].split(':').collect();
        let id: u32 = s.get(1).ok_or("Missing selector id")?.parse()?;
        let mut reference = match s[0] {
            "object" => {
                let d = objects.get(&id).ok_or("Unknown reference object")?;
                let shape = if d.model_ids(10).is_empty() {
                    *d.model_shapes
                        .as_ref()
                        .and_then(|v| v.first())
                        .ok_or("No object shape")?
                } else {
                    10
                };
                (*object_model::Models::default()
                    .object(&mut cache, d, shape, 0)?
                    .ok_or("Reference object unavailable")?)
                .clone()
            }
            "npc" => {
                let d = npcs.get(&id).ok_or("Unknown reference NPC")?;
                let mut m = npc_model::mesh(
                    &mut cache,
                    &mut item_model::Models::default(),
                    d,
                    false,
                    None,
                )?
                .ok_or("Reference NPC unavailable")?;
                m.scale([d.scale[0] as i32, d.scale[1] as i32, d.scale[0] as i32]);
                m
            }
            "item" => item_model::Models::default().inventory(&mut cache, &items, id, 1, None)?,
            _ => return Err("Unsupported reference kind".into()),
        };
        let mut owned = Model::decode(&fs::read(&a[5])?)?;
        let rb = native::bounds(&reference);
        let cb = native::bounds(&owned);
        // Ground both as rigid objects; retain native dimensions, never fit one
        // model to the other. Put an explicit 80-unit gap between their bounds.
        reference.translate([(-40. - rb[1][0]).floor() as i32, -rb[1][1].ceil() as i32, 0]);
        owned.translate([(40. - cb[0][0]).ceil() as i32, -cb[1][1].ceil() as i32, 0]);
        let scene = Model::combine(&[reference, owned])?;
        let light = scene.light(64, 768, [-30, -50, -30])?;
        native::views(&scene, &light, &tex, native::camera(&scene), out)?;
        fs::write(out.join("scene.json"),format!("{{\"reference\":{:?},\"owned_sha256\":\"{:x}\",\"gap_native\":80,\"ambient\":64,\"contrast\":768,\"artistic_acceptance\":false,\"publication\":\"protected local comparison\"}}",a[4],Sha256::digest(fs::read(&a[5])?)))?;
        return Ok(());
    }
    fs::write(out.join("original-id-bounds.json"),format!("{{\"model_max\":{},\"object_max\":{},\"npc_max\":{},\"item_max\":{},\"sequence_max\":{},\"frame_group_max\":{},\"skeleton_max\":{}}}",cache.reference(7)?.groups.iter().map(|g|g.id).max().unwrap_or(0),objects.keys().max().unwrap(),npcs.keys().max().unwrap(),items.items.keys().max().unwrap(),sequences.keys().max().unwrap(),cache.reference(0)?.groups.iter().map(|g|g.id).max().unwrap_or(0),cache.reference(1)?.groups.iter().map(|g|g.id).max().unwrap_or(0)))?;
    if a[3] == "discover" {
        let mut rows: Vec<_> = objects.values().filter(|d| d.animation.is_some()).collect();
        rows.sort_by_key(|d| d.id);
        fs::write(
            out.join("animated-objects.tsv"),
            rows.iter()
                .map(|d| format!("{}\t{}\t{:?}\t{:?}\n", d.id, d.name, d.models, d.animation))
                .collect::<String>(),
        )?;
        return Ok(());
    }
    let mut models = item_model::Models::default();
    let mut om = object_model::Models::default();
    let mut status = String::from("selector\tstatus\tdetail\n");
    let explicit_animation = a[3] == "animate" || a[3] == "animate-complete";
    let selectors = if explicit_animation {
        &a[4..5]
    } else {
        &a[4..]
    };
    for selector in selectors {
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            let s: Vec<_> = selector.split(':').collect();
            if s.len() < 2 {
                return Err("selector kind:id[:variant]".into());
            }
            let id: u32 = s[1].parse()?;
            let dir = out.join(selector.replace(':', "-"));
            fs::create_dir_all(&dir)?;
            let (m, name, ids, ambient, contrast, scaling, roles, definition, archive_group) =
                match s[0] {
                    "object" => {
                        let d = objects.get(&id).ok_or("Unknown object")?;
                        let shape = if let Some(shape) = s.get(2) {
                            shape.parse()?
                        } else if d.model_ids(10).is_empty() {
                            *d.model_shapes
                                .as_ref()
                                .and_then(|x| x.first())
                                .ok_or("Object has no shape")?
                        } else {
                            10
                        };
                        let m = (*om
                            .object(&mut cache, d, shape, 0)?
                            .ok_or("Object unavailable")?)
                        .clone();
                        fs::write(dir.join("factory.json"),format!("{{\"shape\":{shape},\"scale\":{:?},\"offset\":{:?},\"size\":{:?},\"rotated\":{},\"non_flat_shading\":{},\"recolor\":{},\"retexture\":{}}}",d.scale,d.offset,d.size,d.rotated,d.non_flat_shading,format!("{:?}",d.recolor).replace('(',"[").replace(')',"]"),format!("{:?}",d.retexture).replace('(',"[").replace(')',"]")))?;
                        (
                            m,
                            d.name.clone(),
                            d.model_ids(shape),
                            64 + d.ambient as i32,
                            768 + d.contrast,
                            [128; 3],
                            d.animation
                                .map(|v| vec![("object", v as i32)])
                                .unwrap_or_default(),
                            format!("{d:#?}"),
                            6,
                        )
                    }
                    "npc" => {
                        let d = npcs.get(&id).ok_or("Unknown NPC")?;
                        (
                            npc_model::mesh(&mut cache, &mut models, d, false, None)?
                                .ok_or("NPC unavailable")?,
                            d.name.clone(),
                            d.models.clone(),
                            64 + d.ambient as i32,
                            850 + d.contrast,
                            [d.scale[0] as i32, d.scale[1] as i32, d.scale[0] as i32],
                            vec![("idle", d.animations[0]), ("walk", d.animations[3])],
                            format!("{d:#?}"),
                            9,
                        )
                    }
                    "item" => {
                        let d = items.items.get(&id).ok_or("Unknown item")?;
                        let worn = s.get(2) == Some(&"worn");
                        let m = if worn {
                            models
                                .wearable(&mut cache, d, 0, false, None)?
                                .ok_or("No male worn model")?
                        } else {
                            models.inventory(&mut cache, &items, id, 1, None)?
                        };
                        let ids = if worn {
                            d.wear_models[0]
                                .iter()
                                .filter(|&&x| x >= 0)
                                .map(|&x| x as u32)
                                .collect()
                        } else {
                            vec![d.model as u32]
                        };
                        (
                            m,
                            d.name.clone(),
                            ids,
                            64 + d.ambient as i32,
                            768 + d.contrast,
                            [128; 3],
                            Vec::new(),
                            format!("{d:#?}"),
                            10,
                        )
                    }
                    _ => return Err("Unknown selector kind".into()),
                };
            fs::write(dir.join("geometry.json"), native::geometry(&m))?;
            fs::write(dir.join("definition.txt"), definition)?;
            let light = m.light(ambient, contrast, [-30, -50, -30])?;
            let mut world = m.clone();
            world.scale(scaling);
            let submitted_faces: Vec<_> = light
                .iter()
                .enumerate()
                .filter(|(_, c)| c[2] != -2)
                .map(|(i, _)| i)
                .collect();
            let used: BTreeSet<_> = submitted_faces.iter().flat_map(|&i| m.faces[i]).collect();
            if !used.is_empty() {
                let mut surface = world.clone();
                surface.vertices = used.iter().map(|&i| world.vertices[i as usize]).collect();
                fs::write(dir.join("submitted-surface.json"),format!("{{\"face_indices\":{:?},\"world_bounds\":{:?},\"basis\":\"native light output third channel != -2; includes alpha/translucent surfaces, no occlusion inference\"}}",submitted_faces,native::bounds(&surface)))?;
            }
            let cam = native::camera(&world);
            fs::write(
                dir.join("camera.txt"),
                cam.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
            )?;
            let b = native::bounds(&world);
            let mut sources = Vec::new();
            for &mid in &ids {
                let bytes = cache.file(7, mid, 0, [0; 4])?;
                fs::write(
                    dir.join(format!("raw-model-{mid}.json")),
                    native::geometry(&Model::decode(&bytes)?),
                )?;
                let raw = cache.raw_group(7, mid)?;
                fs::write(dir.join(format!("raw-model-{mid}.model")), &bytes)?;
                sources.push(format!("{{\"model_id\":{mid},\"payload_sha256\":\"{:x}\",\"raw_group_sha256\":\"{:x}\"}}",Sha256::digest(&bytes),Sha256::digest(&raw)));
            }
            let def = cache.file(2, archive_group, id, [0; 4])?;
            let rolejson = roles
                .iter()
                .filter(|(_, v)| *v >= 0)
                .map(|(name, v)| format!("{name:?}:{v}"))
                .collect::<Vec<_>>()
                .join(",");
            fs::write(dir.join("metadata.json"),format!("{{\"schema\":1,\"selector\":{selector:?},\"name\":{name:?},\"revision\":\"239.1\",\"model_ids\":{ids:?},\"definition_sha256\":\"{:x}\",\"sources\":[{}],\"world_bounds\":{b:?},\"render_scale\":{scaling:?},\"ambient\":{ambient},\"contrast\":{contrast},\"camera\":{cam:?},\"sequences\":{{{rolejson}}},\"coordinate_contract\":\"native X,Y down,Z;128 units/tile;bind pose\"}}",Sha256::digest(&def),sources.join(",")))?;
            native::views(&world, &light, &tex, cam, &dir)?;
            let actions: Vec<_> = if a[3] == "inspect-static" {
                vec![]
            } else if explicit_animation {
                vec![("selected", a[5].parse::<i32>()?)]
            } else {
                roles
            };
            let mut animations = Animations::load(&mut cache)?;
            for (role, seqid) in actions.into_iter().filter(|(_, v)| *v >= 0) {
                let sq = sequences.get(&(seqid as u32)).ok_or("Sequence missing")?;
                let action = dir.join(format!("{role}-{seqid}"));
                fs::create_dir_all(&action)?;
                let mut skeletons = BTreeSet::new();
                let mut frames = Vec::new();
                if let Some(cached) = sq.cached_animation {
                    let bytes = cache.file(22, cached >> 16, cached & 65535, [0; 4])?;
                    let sid = u16::from_be_bytes(bytes[1..3].try_into()?);
                    let sk = Skeleton::decode(sid, &cache.file(1, sid as u32, 0, [0; 4])?)?;
                    gielinor_client::cached_animation::CachedAnimation::decode(&bytes, &sk)?;
                    skeletons.insert(sid);
                } else {
                    for (i, &fid) in sq.frames.iter().enumerate() {
                        let bytes = cache.file(0, fid >> 16, fid & 65535, [0; 4])?;
                        let sid = u16::from_be_bytes(bytes[..2].try_into()?);
                        let skbytes = cache.file(1, sid as u32, 0, [0; 4])?;
                        let sk = Skeleton::decode(sid, &skbytes)?;
                        let frame = Frame::decode(&bytes, &sk)?;
                        skeletons.insert(sid);
                        fs::write(action.join(format!("frame-{i}.bin")), bytes)?;
                        fs::write(action.join(format!("skeleton-{sid}.bin")), skbytes)?;
                        frames.push(format!("{{\"index\":{i},\"length\":{},\"source_frame\":{fid},\"skeleton\":{sid},\"types\":{:?},\"labels\":{:?},\"transforms\":[{}]}}",sq.lengths[i],sk.transform_types,sk.labels,frame.transforms.iter().map(|t|format!("[{},{}]",t.label,format!("{:?}",t.values))).collect::<Vec<_>>().join(",")));
                    }
                }
                fs::write(action.join("sequence.txt"), format!("{sq:#?}"))?;
                fs::write(
                    action.join("sequence.bin"),
                    cache.file(2, 12, seqid as u32, [0; 4])?,
                )?;
                fs::write(action.join("animation.json"),format!("{{\"source_sequence\":{seqid},\"cached_animation\":{},\"skeleton_ids\":{:?},\"frames\":[{}]}}",sq.cached_animation.map(|x|x.to_string()).unwrap_or("null".into()),skeletons.into_iter().collect::<Vec<_>>(),frames.join(",")))?;
                let duration = if sq.cached_animation.is_some() {
                    (sq.cached_range[1] - sq.cached_range[0]) as u32
                } else {
                    sq.total_length
                };
                let mut times: BTreeSet<_> = (0..=duration.min(80)).step_by(4).collect();
                if a[3] == "animate-complete" {
                    times = (0..=duration).collect();
                }
                times.extend([0, duration / 4, duration / 2, duration * 3 / 4, duration]);
                for tick in times {
                    let mut clock = Clock::default();
                    clock.advance(sq, tick)?;
                    let mut posed = m.clone();
                    animations.apply(&mut cache, &mut posed, seqid as u32, &clock)?;
                    posed.scale(scaling);
                    fs::write(
                        action.join(format!("pose-{tick:04}.json")),
                        native::geometry(&posed),
                    )?;
                    native::render(
                        &posed,
                        &light,
                        &tex,
                        cam,
                        0.65,
                        &action.join(format!("pose-{tick:04}.bmp")),
                        false,
                    )?;
                }
            }
            Ok(())
        })();
        match result {
            Ok(()) => {
                status.push_str(&format!(
                    "{selector}\tPASS\tinspected; protected local outputs\n"
                ));
                println!("INSPECT_PASS {selector}");
            }
            Err(e) => {
                status.push_str(&format!("{selector}\tFAIL\t{e}\n"));
                println!("INSPECT_FAIL {selector}: {e}");
            }
        }
    }
    let failed = status.lines().filter(|l| l.contains("\tFAIL\t")).count();
    fs::write(out.join("batch-status.tsv"), status)?;
    if failed > 0 {
        return Err(format!("{failed} reference inspections failed; see batch-status.tsv").into());
    }
    Ok(())
}
