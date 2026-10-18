"""Derive a bounded review adapter; native scene/renderer implementations stay unchanged."""
from pathlib import Path
T=Path(__file__).resolve().parent;S=T.parents[2];C=S.parent/'cinderworks-environment-client'
source=(T.parent/'offline/render/src/main.rs').read_text()
old='''    let mut frame = Frame::new(width, height)?;
    frame.draw_iter(&camera, mesh.visible_triangles(draw_plane as u8), &p, &textures);
    fs::write(&a[3], frame.raster()?.bmp()?)?;'''
new='''    // Original prepared native scene through the existing enhanced API.
    // This executable defaults to GPU; Classic remains an explicit selection.
    let settings=if env::var_os("TOWER_RENDERER").is_some(){gielinor_client::enhanced::Settings::from_env()?}else{gielinor_client::enhanced::Settings::for_mode(gielinor_client::enhanced::Mode::Gpu)};
    let mode=settings.mode;
    if mode==gielinor_client::enhanced::Mode::Classic {
        let mut frame=Frame::new(width,height)?;
        frame.draw_iter(&camera,mesh.visible_triangles(draw_plane as u8),&p,&textures);
        fs::write(&a[3],frame.raster()?.bmp()?)?;
        fs::write(format!("{}.renderer.json",a[3]),"{\\"mode\\":\\"classic\\",\\"enhanced\\":false}")?;
    } else {
        let (submitted,stable_count)=mesh.enhanced_triangles(draw_plane as u8);
        let mut renderer=gielinor_client::enhanced::Renderer::new(settings);
        let count=env::var("TOWER_REVIEW_FRAMES").ok().map(|v|v.parse::<usize>()).transpose()?.unwrap_or(8).clamp(2,120);
        let mut pixels=Vec::new();let start=std::time::Instant::now();
        for _ in 0..count {pixels=renderer.render(gielinor_client::enhanced::SceneFrame{camera:&camera,triangles:&submitted,stable_count,stable_revision:Some(mesh.enhanced_revision),albedo:None,palette:&p,textures:&textures,size:[width as u32,height as u32],light_focus:center,ambient:64,contrast:768})?;}
        let raster=gielinor_client::raster::Raster::from_pixels(width,height,pixels)?;
        fs::write(&a[3],raster.bmp()?)?;
        fs::write(format!("{}.renderer.json",a[3]),format!("{{\\"mode\\":{:?},\\"enhanced\\":true,\\"total_ms\\":{:.3},\\"frames\\":{},\\"stats\\":{}}}",mode.name(),start.elapsed().as_secs_f64()*1000.,count,renderer.stats.json()))?;
    }'''
assert old in source;source=source.replace(old,new).replace('fn main() -> Result<(), Box<dyn std::error::Error>> {','#[cfg(feature="enhanced-renderer")]\nfn main() -> Result<(), Box<dyn std::error::Error>> {')
marker='    let mesh = WorldMesh::build(&s, &mut c, draw_plane)?;'
audit='''    // Private native height samples support exact equipment-port alignment.
    let mut heights=String::from("local_x\\tlocal_z\\theight\\n");
    for xx in 0..=s.width {for zz in 0..=s.height {if let Some(h)=s.height(draw_plane,xx,zz){heights+=&format!("{}\\t{}\\t{}\\n",xx+24,zz+24,h);}}}
    fs::write(format!("{}.heights.tsv",a[3]),heights)?;
'''
assert marker in source;source=source.replace(marker,audit+marker)
source+='\n#[cfg(not(feature="enhanced-renderer"))]\nfn main(){eprintln!("Build this experimental adapter with --features enhanced-renderer");std::process::exit(1);}\n'
source=source.replace('    let mut matched_metadata=String::new();','    if matches!(preset,"floor-iso"|"floor-plan"|"overview"|"plan"){camera.far=30000;}\n    let mut matched_metadata=String::new();')
source=source.replace('"floor-iso" => (1800, 1600, 10000.0, 1800.0)', '"floor-iso" => (1800, 1600, 7500.0, 2300.0)')
target=C/'src/bin/cinderworks-review.rs';target.write_text(source);print('BOUNDED_REVIEW_ADAPTER',target)
