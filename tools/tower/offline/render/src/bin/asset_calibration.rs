//! Private diagnostic export/playback. Original payloads and derivatives stay local.
use gielinor_client::{
    animation::{Frame as Pose, Sequence, Skeleton},
    animation_runtime::{Animations, Clock},
    disk_cache::DiskCache,
    floor, item_model,
    model::Model,
    npc, npc_model, object, object_model,
    render3d::{Camera, Frame, Triangle},
};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, fmt::Debug, fs, path::Path};
fn opt<T: Debug>(v: &Option<T>) -> String {
    v.as_ref()
        .map(|x| format!("{x:?}"))
        .unwrap_or("null".into())
}
fn json(m: &Model) -> String {
    format!("{{\"vertices\":{:?},\"faces\":{:?},\"colors\":{:?},\"render_types\":{},\"priorities\":{},\"priority\":{},\"alpha\":{},\"vertex_skins\":{},\"face_skins\":{},\"bone_weights\":{},\"face_textures\":{},\"texture_coords\":{},\"texture_types\":{:?},\"texture_triangles\":{:?},\"face_flags\":{}}}",m.vertices,m.faces,m.colors,opt(&m.render_types),opt(&m.priorities),m.priority,opt(&m.alpha),opt(&m.vertex_skins),opt(&m.face_skins),opt(&m.bone_weights).replace('(',"[").replace(')',"]"),opt(&m.face_textures),opt(&m.texture_coords),m.texture_types,m.texture_triangles,opt(&m.face_flags))
}
fn local_pose(dir: &Path, index: usize) -> Result<(Skeleton, Pose), Box<dyn std::error::Error>> {
    let bytes = fs::read(dir.join(format!("frame-{index}.bin")))?;
    let sid = u16::from_be_bytes(bytes[..2].try_into()?);
    let skeleton = Skeleton::decode(sid, &fs::read(dir.join(format!("skeleton-{sid}.bin")))?)?;
    let frame = Pose::decode(&bytes, &skeleton)?;
    Ok((skeleton, frame))
}
fn render(
    m: &Model,
    colors: &[[i32; 3]],
    path: &Path,
    yaw: f32,
    offsets: &[f32],
) -> Result<(), Box<dyn std::error::Error>> {
    let entries: Vec<_> = offsets.iter().map(|&x| (m, colors, x)).collect();
    render_collection(&entries, path, yaw)
}
fn render_collection(
    entries: &[(&Model, &[[i32; 3]], f32)],
    path: &Path,
    yaw: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let m = entries[0].0;
    if m.face_textures.iter().flatten().any(|&t| t >= 0) {
        return Err(
            "Textured diagnostic submission not implemented; select untextured reference".into(),
        );
    }
    let wide = entries.len() > 1;
    let distance = if wide { 1400.0 } else { 630.0 };
    let camera = Camera::look_at(
        [yaw.sin() * distance, -340.0, -yaw.cos() * distance],
        [0.0, -95.0, 0.0],
        if wide { 1050.0 } else { 850.0 },
    )?;
    let mut frame = Frame::new(if wide { 1500 } else { 700 }, 650)?;
    let mut triangles = Vec::new();
    for (batch, &(m, colors, offset)) in entries.iter().enumerate() {
        for (i, face) in m.faces.iter().enumerate() {
            if colors[i][2] == -2 {
                continue;
            }
            let flat = colors[i][2] == -1;
            let color = if flat { [colors[i][0]; 3] } else { colors[i] };
            let mut vertices = face.map(|v| m.vertices[v as usize]);
            for v in &mut vertices {
                v[0] += offset;
            }
            triangles.push(Triangle {
                vertices,
                local_vertices: Some(vertices),
                model_bounds: None,
                model_order: Some(gielinor_client::model_order::Submission::new(m, i, batch)),
                depth_bias: m.face_depth_bias(i),
                colors: color,
                flat,
                uv: [[0.; 2]; 3],
                texture: None,
                texture_mapping: None,
                alpha: m.face_alpha(i),
                near_alpha: Some(m.alpha.as_ref().map_or(0, |a| a[i])),
                tag: batch as u64 + 1,
                view: 0,
            });
        }
    }
    frame.draw_iter(
        &camera,
        triangles.iter(),
        &floor::palette(0.8)?,
        &HashMap::new(),
    );
    fs::write(path, frame.raster()?.bmp()?)?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 4 {
        return Err(
            "asset_calibration CACHE OUTPUT export|render [MODEL_FILE SEQUENCE_DIRECTORY]".into(),
        );
    }
    let root = Path::new(&args[2]);
    fs::create_dir_all(root)?;
    let mut cache = DiskCache::open_legacy_reference(&args[1], 64 << 20)?;
    let npcs = npc::Definition::load_all(&mut cache)?;
    let mut models = item_model::Models::default();
    if args[3] == "export" {
        let mut scan = String::new();
        for d in npcs
            .values()
            .filter(|d| d.name.eq_ignore_ascii_case("goblin"))
            .take(30)
        {
            scan.push_str(&format!(
                "{} {:?} {:?} {:?}\n",
                d.id, d.models, d.animations, d.scale
            ));
        }
        fs::write(root.join("npc-candidates.txt"), scan)?;
        let d = npcs.get(&655).ok_or("Goblin 655 missing")?;
        fs::write(root.join("npc-definition.txt"), format!("{d:#?}"))?;
        let m = npc_model::mesh(&mut cache, &mut models, d, false, None)?
            .ok_or("NPC model unavailable")?;
        fs::write(root.join("npc.json"), json(&m))?;
        for &id in &d.models {
            let raw = cache.file(7, id, 0, [0; 4])?;
            fs::write(root.join(format!("source-model-{id}.bin")), raw)?;
        }
        let defs = object::Definition::load_all(&mut cache)?;
        let sd = defs.get(&6098).ok_or("Object 6098 missing")?;
        fs::write(root.join("static-definition.txt"), format!("{sd:#?}"))?;
        let shape = sd.model_shapes.as_ref().map_or(10, |v| v[0]);
        let sm = object_model::Models::default()
            .object(&mut cache, sd, shape, 0)?
            .ok_or("Static model unavailable")?;
        fs::write(root.join("static.json"), json(&sm))?;
        let mut source_hashes =
            String::from("archive\tgroup\tfile\tpayload_sha256\traw_group_sha256\n");
        let mut dependencies: Vec<_> = d
            .models
            .iter()
            .chain(sd.model_ids(shape).iter())
            .map(|&id| (7, id, 0))
            .collect();
        dependencies.extend([
            (2, 9, 655),
            (2, 6, 6098),
            (2, 12, 6181),
            (2, 12, 6180),
            (2, 12, 6185),
            (2, 12, 6183),
            (2, 12, 6182),
        ]);
        for (archive, group, file) in dependencies {
            let data = cache.file(archive, group, file, [0; 4])?;
            let raw = cache.raw_group(archive, group)?;
            source_hashes.push_str(&format!(
                "{archive}\t{group}\t{file}\t{:x}\t{:x}\n",
                Sha256::digest(&data),
                Sha256::digest(&raw)
            ));
            fs::write(
                root.join(format!("source-{archive}-{group}-{file}.bin")),
                data,
            )?;
        }
        fs::write(root.join("source-hashes.tsv"), source_hashes)?;
        let seqs = Sequence::load_all(&mut cache)?;
        let mut summary = String::new();
        for (name, id) in [
            ("idle", d.animations[0] as u32),
            ("walk", d.animations[3] as u32),
            ("attack", 6185),
            ("hit", 6183),
            ("death", 6182),
        ] {
            let s = seqs.get(&id).ok_or("Sequence missing")?;
            let out = root.join(name);
            fs::create_dir_all(&out)?;
            fs::write(out.join("sequence.bin"), cache.file(2, 12, id, [0; 4])?)?;
            fs::write(out.join("sequence.txt"), format!("{s:#?}"))?;
            if s.cached_animation.is_some() {
                return Err(format!(
                    "{name} uses cached animation; deliberate translator required"
                )
                .into());
            }
            let mut entries = Vec::new();
            for (i, &fid) in s.frames.iter().enumerate() {
                let data = cache.file(0, fid >> 16, fid & 65535, [0; 4])?;
                let sid = u16::from_be_bytes(data[..2].try_into()?);
                let skbytes = cache.file(1, sid as u32, 0, [0; 4])?;
                let sk = Skeleton::decode(sid, &skbytes)?;
                let pose = Pose::decode(&data, &sk)?;
                fs::write(out.join(format!("frame-{i}.bin")), &data)?;
                fs::write(out.join(format!("skeleton-{sid}.bin")), skbytes)?;
                entries.push(format!("{{\"index\":{i},\"source_frame\":{fid},\"length\":{},\"skeleton\":{sid},\"types\":{:?},\"labels\":{:?},\"transforms\":[{}]}}",s.lengths[i],sk.transform_types,sk.labels,pose.transforms.iter().map(|t|format!("[{},{}]",t.label,format!("{:?}",t.values))).collect::<Vec<_>>().join(",")));
            }
            fs::write(
                out.join("animation.json"),
                format!(
                    "{{\"source_sequence\":{id},\"frames\":[{}]}}",
                    entries.join(",")
                ),
            )?;
            summary.push_str(&format!("{name} sequence={id} frames={} ticks={} verified all frame/skeleton dependencies\n",s.frames.len(),s.total_length));
        }
        fs::write(root.join("availability.txt"), summary)?;
        println!("REFERENCE_EXPORT_PASS static6098 npc655; all five classic sequences decoded");
        return Ok(());
    }
    let is_static = args.get(4).map(String::as_str) == Some("static");
    let d = npcs.get(&655).ok_or("NPC missing")?;
    let model_path = args.get(5).ok_or("model source original|FILE required")?;
    let m = if model_path == "original" {
        if is_static {
            let defs = object::Definition::load_all(&mut cache)?;
            let sd = &defs[&6098];
            let shape = sd.model_shapes.as_ref().map_or(10, |v| v[0]);
            (*object_model::Models::default()
                .object(&mut cache, sd, shape, 0)?
                .ok_or("Static missing")?)
            .clone()
        } else {
            npc_model::mesh(&mut cache, &mut models, d, false, None)?.ok_or("NPC missing")?
        }
    } else {
        Model::decode(&fs::read(model_path)?)?
    };
    fs::write(root.join("decoded.json"), json(&m))?;
    if model_path != "original" {
        let dir = Path::new(model_path)
            .parent()
            .ok_or("Package directory missing")?;
        let model_id = if is_static {
            let id: u32 = fs::read_to_string(dir.join("static.id"))?.parse()?;
            let definition =
                object::Definition::decode_local(id, &fs::read(dir.join("static.definition"))?)?;
            if object::Definition::load_all(&mut cache)?.contains_key(&id) {
                return Err("Static ID collision".into());
            }
            *definition
                .model_ids(0)
                .first()
                .ok_or("Static binding missing")?
        } else {
            let id: u32 = fs::read_to_string(dir.join("npc.id"))?.parse()?;
            npc::Definition::decode_local(id, &fs::read(dir.join("npc.definition"))?)?.models[0]
        };
        if model_id < 1100200 || cache.reference(7)?.groups.iter().any(|g| g.id == model_id) {
            return Err("Model ID collision".into());
        }
        fs::write(root.join("registration.txt"),format!("Native definition binding model={model_id}; cache collision checks passed; standalone diagnostic loader; no JS5 writes"))?;
    }
    let light = m.light(
        64 + if is_static { 0 } else { d.ambient },
        if is_static { 768 } else { 850 + d.contrast },
        [-30, -50, -30],
    )?;
    for (name, yaw) in [("front", 0.), ("quarter", 0.65), ("reverse", 3.14)] {
        render(
            &m,
            &light,
            &root.join(format!("bind-{name}.bmp")),
            yaw,
            &[0.],
        )?;
    }
    for turn in 0..4u16 {
        let mut rotated = m.clone();
        rotated.vertices =
            gielinor_client::model_draw::vertices(&m.vertices, turn * 512, [0, 0, 0])?;
        render(
            &rotated,
            &light,
            &root.join(format!("rotation-{turn}.bmp")),
            0.,
            &[0.],
        )?;
    }
    if args[3] == "compare" {
        let rt = Model::decode(&fs::read(args.get(6).ok_or("roundtrip model required")?)?)?;
        let own = Model::decode(&fs::read(args.get(7).ok_or("independent model required")?)?)?;
        let lc = |x: &Model| {
            x.light(
                64 + if is_static { 0 } else { d.ambient },
                if is_static { 768 } else { 850 + d.contrast },
                [-30, -50, -30],
            )
        };
        let rl = lc(&rt)?;
        let ol = lc(&own)?;
        render_collection(
            &[(&m, &light, -320.), (&rt, &rl, 0.), (&own, &ol, 320.)],
            &root.join("three-way-scene.bmp"),
            0.,
        )?;
        // Authentic static human beside each wall; no extracted player mesh is published.
        if is_static {
            let hd = &npcs[&3106];
            let mut human = npc_model::mesh(&mut cache, &mut models, hd, false, None)?
                .ok_or("Human missing")?;
            npc_model::resize(&mut human, hd);
            let hl = human.light(64, 850, [-30, -50, -30])?;
            render_collection(
                &[
                    (&m, &light, -400.),
                    (&human, &hl, -240.),
                    (&rt, &rl, -80.),
                    (&human, &hl, 80.),
                    (&own, &ol, 240.),
                    (&human, &hl, 400.),
                ],
                &root.join("authentic-human-scale.bmp"),
                0.,
            )?;
        }
        return Ok(());
    }
    if is_static {
        return Ok(());
    }
    let seqroot = args
        .get(6)
        .ok_or("sequence source original|DIRECTORY required")?;
    if seqroot != "original" {
        let bytes = fs::read(Path::new(seqroot).join("npc.definition"))?;
        let custom_id: u32 = fs::read_to_string(Path::new(seqroot).join("npc.id"))?.parse()?;
        let custom = npc::Definition::decode_local(custom_id, &bytes)?;
        if npcs.contains_key(&custom_id)
            || custom.models.len() != 1
            || custom.models[0] < 1100200
            || custom.animations[0] != 36181
            || custom.animations[3] != 36180
            || custom.ambient != d.ambient
            || custom.contrast != d.contrast
        {
            return Err("Diagnostic NPC ID/model/sequence/lighting registration collision".into());
        }
    }
    let mut native = Animations::load(&mut cache)?;
    // Controlled action+locomotion layering, using the native classic mask operator.
    let custom = seqroot != "original";
    let attack = if custom {
        Sequence::decode(
            36185,
            &fs::read(Path::new(seqroot).join("attack/sequence.bin"))?,
        )?
    } else {
        native.sequences[&6185].clone()
    };
    let walk = if custom {
        Sequence::decode(
            36180,
            &fs::read(Path::new(seqroot).join("walk/sequence.bin"))?,
        )?
    } else {
        native.sequences[&6180].clone()
    };
    for tick in 0..=48 {
        let (mut a, mut b) = (Clock::default(), Clock::default());
        a.advance(&attack, tick)?;
        b.advance(&walk, tick)?;
        let mut posed = m.clone();
        if custom {
            let (sk, fr) = local_pose(&Path::new(seqroot).join("attack"), a.frame)?;
            posed.animate_classic_mask(&sk, &fr, attack.interleave.as_deref(), false, false)?;
            let (sk, fr) = local_pose(&Path::new(seqroot).join("walk"), b.frame)?;
            posed.animate_classic_mask(&sk, &fr, attack.interleave.as_deref(), true, false)?;
        } else {
            native.apply_blended(&mut cache, &mut posed, 6185, &a, 6180, &b)?;
        }
        npc_model::resize(&mut posed, d);
        render(
            &posed,
            &light,
            &root.join(format!("blend-{tick:03}.bmp")),
            0.35,
            &[0.],
        )?;
        fs::write(root.join(format!("blend-{tick:03}.json")), json(&posed))?;
    }
    let mut native = Animations::load(&mut cache)?;
    let mut ledger = String::from("action\ttick\tframe\telapsed\tmax_displacement\n");
    for (name, id) in [
        ("idle", d.animations[0] as u32),
        ("walk", d.animations[3] as u32),
        ("attack", 6185),
        ("hit", 6183),
        ("death", 6182),
    ] {
        let custom = seqroot != "original";
        let dir = Path::new(seqroot).join(name);
        let seq = if custom {
            Sequence::decode(30000 + id, &fs::read(dir.join("sequence.bin"))?)?
        } else {
            native.sequences[&id].clone()
        };
        let mut samples: std::collections::BTreeSet<u32> =
            (0..=seq.total_length.min(256)).collect();
        let mut t = 0;
        for &length in &seq.lengths {
            samples.insert(t);
            samples.insert(t + length as u32);
            samples.insert(t + 1);
            t += length as u32;
        }
        samples.insert(seq.total_length);
        for tick in samples {
            let mut clock = Clock::default();
            clock.advance(&seq, tick)?;
            let mut posed = m.clone();
            if custom {
                let fid = seq.frames[clock.frame];
                if fid >> 16 < 60000
                    || fid & 65535 != clock.frame as u32
                    || cache.reference(0)?.groups.iter().any(|g| g.id == fid >> 16)
                {
                    return Err("Custom frame ID collision/mapping".into());
                }
                let bytes = fs::read(dir.join(format!("frame-{}.bin", clock.frame)))?;
                let sid = u16::from_be_bytes(bytes[..2].try_into()?);
                if sid < 64000
                    || cache
                        .reference(1)?
                        .groups
                        .iter()
                        .any(|g| g.id == sid as u32)
                {
                    return Err("Custom skeleton ID collision".into());
                }
                let skeleton =
                    Skeleton::decode(sid, &fs::read(dir.join(format!("skeleton-{sid}.bin")))?)?;
                let pose = Pose::decode(&bytes, &skeleton)?;
                posed.animate_classic(&skeleton, &pose, false)?;
            } else {
                native.apply(&mut cache, &mut posed, id, &clock)?;
            }
            let displacement = posed
                .vertices
                .iter()
                .zip(&m.vertices)
                .flat_map(|(a, b)| (0..3).map(move |i| (a[i] - b[i]).abs()))
                .fold(0f32, f32::max);
            npc_model::resize(&mut posed, d);
            render(
                &posed,
                &light,
                &root.join(format!("{name}-{tick:03}.bmp")),
                0.35,
                &[0.],
            )?;
            fs::write(root.join(format!("{name}-{tick:03}.json")), json(&posed))?;
            ledger.push_str(&format!(
                "{name}\t{tick}\t{}\t{}\t{displacement}\n",
                clock.frame, clock.elapsed
            ));
        }
    }
    fs::write(root.join("temporal.tsv"), ledger)?;
    println!("NATIVE_ANIMATION_RENDER_PASS; scheduled timestamps, native clock/decode/classic integer transforms/light/raster, masked blend");
    Ok(())
}
