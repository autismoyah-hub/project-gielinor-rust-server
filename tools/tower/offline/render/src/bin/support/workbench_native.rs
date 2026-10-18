//! Native-client submission adapter, not a second renderer.
use gielinor_client::{
    disk_cache::DiskCache,
    floor,
    model::Model,
    render3d::{texture_uv, Camera, Frame, TextureMapping, Triangle},
    texture,
};
use std::{collections::HashMap, fmt::Debug, fs, path::Path};
pub fn opt<T: Debug>(v: &Option<T>) -> String {
    v.as_ref()
        .map(|x| format!("{x:?}"))
        .unwrap_or("null".into())
}
pub fn geometry(m: &Model) -> String {
    format!("{{\"vertices\":{:?},\"faces\":{:?},\"colors\":{:?},\"render_types\":{},\"priorities\":{},\"priority\":{},\"alpha\":{},\"vertex_skins\":{},\"face_skins\":{},\"bone_weights\":{},\"face_textures\":{},\"texture_coords\":{},\"texture_types\":{:?},\"texture_triangles\":{:?},\"face_flags\":{}}}",m.vertices,m.faces,m.colors,opt(&m.render_types),opt(&m.priorities),m.priority,opt(&m.alpha),opt(&m.vertex_skins),opt(&m.face_skins),opt(&m.bone_weights).replace('(',"[").replace(')',"]"),opt(&m.face_textures),opt(&m.texture_coords),m.texture_types,m.texture_triangles,opt(&m.face_flags))
}
pub fn textures(
    c: &mut DiskCache,
) -> Result<HashMap<u16, texture::Texture>, Box<dyn std::error::Error>> {
    let mut out = HashMap::new();
    for (id, d) in texture::Definition::load_all(c)? {
        out.insert(id, texture::Texture::load(c, d, 128, 0.8)?);
    }
    Ok(out)
}
pub fn bounds(m: &Model) -> [[f32; 3]; 2] {
    [
        std::array::from_fn(|i| m.vertices.iter().map(|v| v[i]).fold(f32::MAX, f32::min)),
        std::array::from_fn(|i| m.vertices.iter().map(|v| v[i]).fold(f32::MIN, f32::max)),
    ]
}
pub fn camera(m: &Model) -> [f32; 4] {
    let b = bounds(m);
    let e = (0..3).map(|i| b[1][i] - b[0][i]).fold(128f32, f32::max);
    [
        (b[0][0] + b[1][0]) / 2.,
        (b[0][1] + b[1][1]) / 2.,
        (b[0][2] + b[1][2]) / 2.,
        e * 3.2,
    ]
}
pub fn render(
    m: &Model,
    light: &[[i32; 3]],
    textures: &HashMap<u16, texture::Texture>,
    cam: [f32; 4],
    yaw: f32,
    path: &Path,
    gameplay: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut camera = if gameplay {
        let focus = [0, -50, 0];
        let eye = gielinor_client::camera::position(
            focus,
            2560,
            (yaw * 16384. / (std::f32::consts::PI * 2.)) as i32,
            1560,
            503,
            [256, 320],
        );
        Camera::angles(
            eye,
            2560,
            (yaw * 16384. / (std::f32::consts::PI * 2.)) as i32,
            308.,
        )?
    } else {
        Camera::look_at(
            [
                cam[0] + yaw.sin() * cam[3],
                cam[1] - cam[3] * 0.45,
                cam[2] - yaw.cos() * cam[3],
            ],
            [cam[0], cam[1], cam[2]],
            850.,
        )?
    };
    camera.far = 5250;
    let mut frame = Frame::new(
        if gameplay { 765 } else { 700 },
        if gameplay { 503 } else { 650 },
    )?;
    let mut triangles = Vec::new();
    let drawn = gielinor_client::model_draw::vertices(&m.vertices, 0, [0, 0, 0])?;
    for (i, face) in m.faces.iter().enumerate() {
        if light[i][2] == -2 {
            continue;
        }
        let texture = m
            .face_textures
            .as_ref()
            .and_then(|t| (t[i] >= 0).then_some(t[i] as u16));
        if texture.is_some_and(|id| !textures.contains_key(&id)) {
            return Err("Missing required texture".into());
        }
        let mapping = m
            .texture_coords
            .as_ref()
            .and_then(|t| {
                if t[i] == -1 {
                    None
                } else {
                    m.texture_triangles.get(t[i] as u8 as usize)
                }
            })
            .unwrap_or(face)
            .map(|v| drawn[v as usize]);
        let vertices = face.map(|v| drawn[v as usize]);
        let flat = light[i][2] == -1;
        triangles.push(Triangle {
            vertices,
            local_vertices: Some(vertices),
            model_bounds: None,
            model_order: Some(gielinor_client::model_order::Submission::new(m, i, 0)),
            depth_bias: m.face_depth_bias(i),
            colors: if flat { [light[i][0]; 3] } else { light[i] },
            flat,
            uv: texture_uv(mapping, vertices),
            texture,
            texture_mapping: Some(TextureMapping {
                vertices: mapping,
                local: Some(mapping),
            }),
            alpha: m.face_alpha(i),
            near_alpha: Some(m.alpha.as_ref().map_or(0, |a| a[i])),
            tag: 1,
            view: 0,
        });
    }
    frame.draw_iter(&camera, triangles.iter(), &floor::palette(0.8)?, textures);
    fs::write(path, frame.raster()?.bmp()?)?;
    Ok(())
}
pub fn views(
    m: &Model,
    light: &[[i32; 3]],
    tex: &HashMap<u16, texture::Texture>,
    cam: [f32; 4],
    out: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(out)?;
    for (name, yaw) in [
        ("front", 0.),
        ("side", std::f32::consts::FRAC_PI_2),
        ("rear", std::f32::consts::PI),
        ("quarter", 0.65),
    ] {
        render(
            m,
            light,
            tex,
            cam,
            yaw,
            &out.join(format!("{name}.bmp")),
            false,
        )?;
    }
    render(m, light, tex, cam, 0.35, &out.join("gameplay.bmp"), true)?;
    Ok(())
}
