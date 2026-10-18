"""Source-safe high-fidelity review package; allowlisted original assets only."""
from review import *
import hashlib,shutil,html
OUT=REPO/'art/tower/high-fidelity-mastery';OUT.mkdir(parents=True,exist_ok=True)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def cp(src,dest):
    dest=OUT/dest;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(src,dest)
def image(path,label):return f'<figure><a href="{path}"><img loading="lazy" src="{path}" alt="{html.escape(label)}"></a><figcaption>{label}</figcaption></figure>'
def grid(items):return '<div class="grid">'+''.join(image(p,l)for p,l in items)+'</div>'
selected={'architecture':'architecture-revision1','machine':'machine-revision1','creature':'creature-final'}
manifest=dict(client_date='2026-10-17',assignment='High-Fidelity Modeling and Rendering Mastery',attribution='GPT-6.1 Sol as specified by user mandate; exact runtime ID unavailable',base_commit='9492a2d59c4ef6f67f62ed0faa8b40b5cb531236',branch='experiment/high-fidelity-mastery',native_renderer_sha256=sha(NATIVE),renderer_modified=False,codec_modified=False,live_game_modified=False,protected_content_in_package=False,blender='5.2.2 LTS d13f752e3b9c',dependencies='Python 3.13, Pillow; bundled Blender bpy/bmesh; existing workbench modules',assets={},new_tool_hashes={p.name:sha(p)for p in HERE.glob('*.py')},shared_tool_hashes={})
for name in ['mesh_tools.py','author.py']:
    p=HERE.parent/'mesh-study'/name;manifest['shared_tool_hashes'][str(p.relative_to(REPO))]=sha(p)
for name in ['native_codec.py','author_blender.py','kinematics.py','motion_authoring.py']:
    p=HERE.parent/'asset-workbench'/name;manifest['shared_tool_hashes'][str(p.relative_to(REPO))]=sha(p)
for kind,version in selected.items():
    src=ROOT/'runs'/version;dest=Path('assets')/kind
    for name in ['asset.model','geometry.json','conversion.json','authoring.json','source.blend','source-rigged.blend','rig.json','limbs.json','motion-validation.json']:
        if(src/name).exists():cp(src/name,dest/name)
    for p in src.glob('*.png'):cp(p,dest/p.name)
    for p in src.glob('*.gif'):cp(p,dest/p.name)
    for p in src.glob('*.camera'):cp(p,dest/p.name)
    if kind=='creature':
        for action in ['idle','walk','attack','hit','death']:
            for p in (src/action).iterdir():
                if p.suffix in ['.bin','.json']:cp(p,dest/action/p.name)
    baseline=REPO/'art/tower/mesh-production-study/assets'/(kind+'-final')
    for p in baseline.glob('*.png'):cp(p,Path('baseline')/kind/p.name)
    manifest['assets'][kind]=dict(selection=version,source_sha256=sha(src/'source.blend'),model_sha256=sha(src/'asset.model'),baseline_source_sha256=sha(baseline/'source.blend'),baseline_model_sha256=sha(baseline/'asset.model'),conversion=json.loads((src/'conversion.json').read_text()))
for version in ['creature-study','creature-refined','creature-revision2','creature-revision3','creature-revision4']:
    src=ROOT/'runs'/version
    for name in ['quarter.png','side.png','head-close.png','source.blend','asset.model','conversion.json']:
        cp(src/name,Path('iterations')/version/name)
for name in ['death-before','flat','single-material','welded']:
    src=ROOT/'comparisons'/name
    for p in src.iterdir():
        if p.is_file()and p.suffix in ['.png','.gif','.json','.model']:cp(p,Path('comparisons')/name/p.name)
for label in ['creature','architecture','machine','scene']:
    src=ROOT/'presentation'/label
    for p in src.iterdir():
        if p.suffix in ['.png','.json']or p.name=='beauty-studio.blend':cp(p,Path('blender')/label/p.name)
for p in (ROOT/'scene').glob('*.png'):cp(p,Path('scene')/p.name)
for p in (ROOT/'scene').glob('*.camera'):cp(p,Path('scene')/p.name)
cp(ROOT/'scene/showcase.blend','scene/showcase.blend')
for name in ['source.blend','asset.model','geometry.json','conversion.json']:cp(ROOT/'scene/ground'/name,Path('scene/ground')/name)
rows=[]
for line in (ROOT/'scene/scene.tsv').read_text().splitlines():
    f=line.split('\t')
    if f[0]=='owned':
        p=Path(f[1]);label=p.parent.name
        if label in selected.values():f[1]='../assets/'+next(k for k,v in selected.items()if v==label)+'/asset.model'
        elif label=='ground':f[1]='ground/asset.model'
        else:cp(p,Path('scene/support')/label/'asset.model');f[1]='support/'+label+'/asset.model'
    rows.append('\t'.join(f))
(OUT/'scene/scene.tsv').write_text('\n'.join(rows)+'\n')
for p in HERE.glob('*.py'):cp(p,Path('sources')/p.name)
cp(ROOT/'diagnostics.json','DIAGNOSTICS.json')
manifest['motion']=json.loads((ROOT/'runs/creature-final/motion-validation.json').read_text());manifest['diagnostics']=json.loads((ROOT/'diagnostics.json').read_text());manifest['gpu']=json.loads((ROOT/'presentation/creature/beauty-device.json').read_text())
if(ROOT/'validation.json').exists():cp(ROOT/'validation.json','VALIDATION.json')
(OUT/'MANIFEST.json').write_text(json.dumps(manifest,indent=2))
page='''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>ASCENT · Cinderworks refinement</title><style>
:root{color-scheme:dark}*{box-sizing:border-box}body{margin:0;background:#10171b;color:#dbe0dd;font:16px/1.65 system-ui,sans-serif}main{max-width:1420px;margin:auto;padding:42px 28px}h1{font-size:clamp(36px,5vw,64px);line-height:1.1;color:#eee2c9;margin:16px 0 25px}h2{font-size:31px;color:#dfc69b;margin-top:65px}h3{font-size:22px;color:#b9cfc4}p{max-width:1080px}a{color:#d9bc84}.eyebrow{letter-spacing:.16em;font-size:12px;color:#b8a17e}.lead{font-size:21px;max-width:1050px}.note{padding:20px 24px;background:#1b272c;border-left:3px solid #b99d6c}nav{display:flex;gap:20px;flex-wrap:wrap;margin:26px 0}.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(310px,1fr));gap:18px}figure{margin:16px 0;background:#162126;border:1px solid #2d3a3e;overflow:hidden;border-radius:5px}img{display:block;width:100%;height:auto}figcaption{padding:11px 16px;font-size:14px;color:#b5c3c2}table{width:100%;border-collapse:collapse;font-size:14px;margin:24px 0}td,th{border-bottom:1px solid #344247;padding:12px;text-align:left;vertical-align:top}th{color:#d9bc84}details{padding:16px;background:#19252a;margin:14px 0}summary{cursor:pointer;color:#d9bc84}code{color:#bdd4c8}footer{margin-top:65px;color:#9bafad;font-size:13px}.clip{max-width:1050px}@media(max-width:700px){main{padding:24px 15px}.grid{grid-template-columns:1fr}}</style><main>
<div class="eyebrow">ASCENT / SOL REFINEMENT CYCLE / ORIGINAL TOWER ASSETS</div><h1>Cinderworks:<br>anatomy, construction & motion.</h1>
<p class="lead">The preserved Sol archway and twin-cylinder pump, with a substantially rebuilt Cindermaw: a hinged mandible, defined orbital anatomy, rooted horns, grouped limbs, individual digits and fitted armor.</p>
<div class="note">These are refined review candidates. The native Rust images are the game-appearance evidence. Cycles images support inspection of surfaces and contact shadows. The user retains artistic approval; neither technical validation nor presentation lighting certifies professional art quality.</div>
<nav><a href="#native">Native showcase</a><a href="#creature">Cindermaw</a><a href="#motion">Motion</a><a href="#structure">Arch & pump</a><a href="#studio">Blender studio</a><a href="#process">Workflow & limitations</a></nav>
<h2 id="native">The maintenance yard · native Rust</h2><p>The arch organizes the rear wall, the pump occupies a paved service area, and the drake approaches over compacted ash and scoria. Original retained wall, barrel and stock-rack assets provide context. The scene contains no authentic cache mesh or proprietary reference imagery.</p>
'''
page+=image('scene/overview.png','Native overview · 1200 × 900 · fixed ambient 64 / contrast 768, light [-30, -50, -30]')
page+=grid([('scene/player-west.png','Normal player camera · west · 765 × 503'),('scene/player-east.png','Normal player camera · east · 765 × 503')])
page+=image('scene/creature-close.png','Native close scene · original source geometry, no post-render relighting')
page+='<h2 id="creature">Cindermaw · the primary refinement</h2><p>The previous continuous body was a useful starting point. This cycle develops the face and material hierarchy, gives the toes and mandible a clearer construction, fits armor to the actual torso surface and separates anatomical groups for full-body animation.</p>'
page+=grid([('baseline/creature/quarter.png','Preserved Sol result · commit 9492a2d'),('assets/creature/quarter.png','Selected refinement · identical native camera and light')])
page+=image('assets/creature/head-close.png','Native facial close-up · upper and lower lids, small iris, horn roots, nasal shields, teeth and articulated jaw')
page+='<details open><summary>Multi-angle native review</summary>'+grid([(f'assets/creature/{v}.png',v.title())for v in ['front','side','rear','gameplay']])+'</details>'
page+=image('assets/creature/turntable.gif','Native turntable · eight equally spaced views, fixed geometry and lighting')
page+='<h3>What changed between iterations</h3><p>The early material mask used local coordinates and incorrectly colored the back. Knee spheres made the limbs look mechanical. Later passes correct the mask in world space, add subdivision support sections, reduce the joint caps, move the eyes clear of the brows and fit subdivided armor shells to the torso. A final flexor overlap reduces joint gaps.</p>'
page+=grid([('iterations/creature-refined/quarter.png','Early refined pass · wrong mask and spherical knees'),('iterations/creature-revision2/quarter.png','Mask fixed · armor still floating'),('iterations/creature-revision3/quarter.png','Support sections · armor fit still incomplete'),('iterations/creature-revision4/quarter.png','Fitted subdivided shells and visible iris')])
page+='<p><strong>Remaining artistic weaknesses:</strong> the torso still has broad quiet areas, classic rigid-group joints remain visible at some angles, and very small details soften on the native integer grid. The head and armor carry more visual identity, but this is not a claim that every angle has reached the intended final standard.</p>'
page+='<h2 id="motion">Five complete native actions</h2><p>The rig includes body, head, jaw, tail and three joints per limb. Walking uses the existing two-bone solver and sole-surface correction. Attack has a windup, forward strike and recovery; hit recoils and returns; death folds the limbs, rolls onto the side and holds the final pose.</p><p><strong>409 native pose samples</strong> match the independent integer transform evaluator exactly. This validates exported deformation, not artistic excellence. Walking has no measured ground penetration or fully airborne frames; mean stance surface-height error is 0.55 native units, maximum 4, at 128 units per tile.</p>'
for name,desc in [('idle','Breathing posture, head, jaw and tail'),('walk','Complete in-place quadruped cycle; no whole-model travel used to fake stepping'),('attack','Anticipation, jaw opening, strike and recovery'),('hit','Body recoil, tail response and return'),('death','Limb folding, side fall, settling and a 60-tick final hold')]:
    page+=f'<h3>{name.title()}</h3><p>{desc}.</p><div class="clip">'+image(f'assets/creature/{name}.gif',f'Native {name} · complete authored duration')+'</div><details><summary>Representative action sequence</summary>'+image(f'assets/creature/{name}-sheet.png','Start · quarter · midpoint · three-quarter · end')+'</details>'
page+='<details><summary>Death-pose correction</summary><p>The first fall balanced on a toe and left the torso 43 units above the lowest point. Additional hip folding reduces that offset to 7 units; the shoulder and head now support the corpse. This correction came from native pose coordinates and visual inspection.</p>'+grid([('comparisons/death-before/death-sheet.png','Before · extended toes prevent the body from settling'),('assets/creature/death-sheet.png','After · folded limbs and stable final hold')])+'</details>'
page+='<h2 id="structure">Preserving and refining the service architecture</h2><p>The arch retains its load-bearing voussoirs, deep reveal, jambs and buttresses. Stone batches, restrained chips, iron springer seats, rivets and foundation returns refine the existing design without adding an unrelated mechanism.</p>'
page+=grid([('baseline/architecture/quarter.png','Preserved arch · same native camera'),('assets/architecture/quarter.png','Refined masonry and reinforcement')])
page+='<p>The twin-cylinder pump retains its crankshaft, eccentric pins, guided piston rods, intake manifold and pressure vessel. Packing glands, oil cups, a legible gauge, a drain tap and supporting fittings clarify maintenance and operation. It remains a static visual assembly; mechanical simulation has not been claimed.</p>'
page+=grid([('baseline/machine/quarter.png','Preserved pump · same native camera'),('assets/machine/quarter.png','Refined pump · glands, pressure gauge and fittings')])
for kind in ['architecture','machine']:
    page+=f'<details><summary>{kind.title()}: front, side, rear and gameplay</summary>'+grid([(f'assets/{kind}/{v}.png',v.title())for v in ['front','side','rear','gameplay']])+'</details>'
page+='<h2 id="studio">A reusable Blender presentation studio</h2><p>Cycles runs on the confirmed GTX 1660 Ti CUDA device, with CPU rendering deselected. The source is normalized for presentation, lit by three area lights against a neutral background, with soft physical contact shadows. Native model units and source assets are preserved separately. These images are clearly labeled as Blender presentation.</p>'
page+=image('blender/scene/beauty.png','BLENDER CYCLES · integrated environment · physical lighting and contact shadows')
page+=grid([('blender/creature/beauty.png','BLENDER CYCLES · creature'),('blender/architecture/beauty.png','BLENDER CYCLES · arch'),('blender/machine/beauty.png','BLENDER CYCLES · pump')])
page+=grid([('blender/creature/clay.png','BLENDER · clay surface inspection'),('blender/creature/topology.png','BLENDER · delivered topology inspection')])
page+='<details><summary>Blender orthographic reference views</summary>'+grid([(f'blender/creature/{v}.png',f'BLENDER orthographic {v}')for v in ['front','side','rear']])+'</details>'
page+='<h2 id="process">Reusable refinement and delivery</h2><table><tr><th>Utility</th><th>Purpose and actual use</th></tr><tr><td><a href="sources/refine.py">refine.py</a></td><td>Supported surface cages, controlled volume fusion, contour relaxation, world-space material regions, fitted shell projection, grouped anatomy, targeted source simplification.</td></tr><tr><td><a href="sources/review.py">review.py</a></td><td>Fixed native asset cameras and light; repeatable close, multi-angle and gameplay views.</td></tr><tr><td><a href="sources/presentation.py">presentation.py</a></td><td>CUDA selection, three-point Cycles studio, contact shadows, clay, wireframe, orthographic and orbit camera presets.</td></tr><tr><td><a href="sources/motion.py">motion.py</a></td><td>Explicit anatomical classic groups and authored actions, reusing the existing joint compiler and gait solver.</td></tr><tr><td><a href="sources/motion_review.py">motion_review.py</a></td><td>Complete native playback, independent pose comparison, full previews and action sheets.</td></tr><tr><td><a href="sources/diagnostics.py">diagnostics.py</a></td><td>Native stance measurements, identical-mesh shading/material comparisons and native turntable.</td></tr><tr><td><a href="sources/scene.py">scene.py</a> / <a href="sources/scene_blender.py">scene_blender.py</a></td><td>Portable native placement and original-source Blender assembly, using the same asset scale and layout.</td></tr><tr><td><a href="sources/native_delivery.py">native_delivery.py</a></td><td>Optional evaluated experiment: weld coincident quantized vertices within each component. Removes 782 duplicate vertices in this creature. Subtle shading benefit; not selected for the validated final delivery or animation rig.</td></tr></table>'
page+='<h3>Material and native-render findings</h3><p>Material regions and native interpolated shading remain more reliable than tiny surface noise. The same geometry is shown below with flat shading and with one material. UVs remain in the editable source, but arbitrary image textures, shader graphs, roughness and normal maps are not silently claimed to survive. The previous source-safe UV-baking test remains preserved in the base study.</p>'
page+=grid([('comparisons/flat/quarter.png','Identical mesh · flat native shading'),('assets/creature/quarter.png','Selected interpolated shading and material regions'),('comparisons/single-material/quarter.png','Identical mesh · one native HSL material')])
page+='<p>Geometry is rounded only in the exported copy, with maximum error 0.5 native units per axis. Collapsed micro-triangles are counted and removed. A dense support-loop pass exceeded the codec’s section limit, so delivery simplification was added as an editable modifier. Mesh density is a delivery constraint, not an artistic score. The codec and authoritative Rust renderer are unchanged.</p>'
page+='<table><tr><th>Asset</th><th>Vertices</th><th>Triangles</th><th>Collapsed micro-triangles</th></tr>'
for kind,data in manifest['assets'].items():
    c=data['conversion'];page+=f'<tr><td>{kind.title()}</td><td>{c["vertices"]:,}</td><td>{c["triangles"]:,}</td><td>{c["collapsed_triangles"]:,}</td></tr>'
page+='</table><h3>Production workflow</h3><ol><li>Write the silhouette, construction and material plan. Preserve the strongest prior source and native images.</li><li>Refine primary proportions in front, side and perspective views before secondary detail.</li><li>Fuse only continuous organic masses. Keep moving anatomy in explicit groups, with overlaps designed for articulation.</li><li>Use support sections and restrained bevels for the intended edge treatment. Project fitted details to the real surface.</li><li>Assign broad material regions using world-space coordinates; inspect native results before adding density.</li><li>Export, compare under fixed native cameras, name the largest shortcomings, and correct them in the source.</li><li>After static review, author full action timing, validate actual native movement and check final contact poses.</li><li>Use Cycles clay, orthographic and topology views as diagnostics, then return to native acceptance images.</li></ol>'
page+='<p><strong>Remaining limitations:</strong> visible classic rigid-joint transitions; native lattice loss on very small details; painter-order and smoothing differences compared with Cycles; no arbitrary PBR transfer; a broad torso that could still use more intentional surface rhythm. The architecture and pump improvements are incremental, while the creature and animation receive most of the cycle’s work. Final artistic approval remains with the user.</p><p>No paid external generation was used. No original cache data, private references, credentials or live-map changes are included. The interrupted Astra experiment remains preserved in its separate workspace and is not presented as a completed controlled model comparison.</p>'
page+='<p><a href="WORKFLOW.txt">Detailed workflow and commands</a> · <a href="MANIFEST.json">Sources, tool identities and mesh reports</a> · <a href="DIAGNOSTICS.json">Native motion diagnostics</a> · <a href="FILES.sha256.json">File hashes</a></p><footer>Isolated branch: experiment/high-fidelity-mastery · preserved base: 9492a2d · original Tower assets · native game appearance remains the final authority.</footer></main></html>'
(OUT/'index.html').write_text(page,encoding='utf8')
(OUT/'WORKFLOW.txt').write_text('''ASCENT HIGH-FIDELITY WORKFLOW
Sources and original geometry are provided for reuse. Protected cache data is not.

Dependencies: Blender 5.2.2 LTS, Python 3.13, Pillow, preserved workbench modules.
The local path constants are in tools/tower/high-fidelity/review.py. Set BLENDER,
NATIVE, CACHE and ROOT for your workstation. The manifest identifies the frozen
native binary and shared tool source hashes. The renderer itself is unchanged.

MODEL GENERATION AND NATIVE REVIEW
  python tools/tower/high-fidelity/review.py creature final
  python tools/tower/high-fidelity/review.py architecture revision1
  python tools/tower/high-fidelity/review.py machine revision1
The output directories preserve existing source.blend files. Use a fresh output
version for a new art iteration rather than overwriting an accepted source.

CREATURE METHOD
Core torso/shoulder/neck volumes are fused and relaxed. Skull and muzzle are
fused separately; the mandible is independent. Supported sweep cages define
horns, limbs, toes and claws. Limb roots overlap inside the body; three classic
groups per limb support motion. Separate plates are fitted to the torso with
ray projection and local subdivision. World-space masks assign broad material
regions. Editing a material mask in local coordinates caused a preserved failed
iteration and should be avoided when components have nonidentity transforms.

Native delivery simplification remains an editable modifier on dense surfaces.
All source meshes retain UVs. Export evaluates modifiers into a copy and calls
the preserved codec. Source precision is preserved; native quantization losses
are recorded. Toggle delivery modifiers for a higher resolution source study.

ANIMATION
  blender -b OUT/runs/creature-final/source.blend --python-exit-code 1 --python tools/tower/high-fidelity/motion.py -- OUT/runs/creature-final
  python tools/tower/high-fidelity/motion_review.py
  python tools/tower/high-fidelity/diagnostics.py
The rig is explicit; group names alone do not imply correct contact. Review the
complete GIFs and native poses. Grounded global translation alone can balance a
corpse on a toe: the death action was corrected by folding the limbs further.

BLENDER PRESENTATION
  python tools/tower/high-fidelity/present_all.py
presentation.py supports beauty, clay, topology, front, side, rear and turn-N
camera modes. It selects CUDA when present and records the selected device.
Presentation normalizes scale, applies a three-point studio and uses Cycles
soft shadows. These scenes and images are not native acceptance evidence.

ENVIRONMENT
  python tools/tower/high-fidelity/scene.py
  python tools/tower/high-fidelity/present_all.py
scene.tsv uses 128 units per tile. The published scene uses relative model paths:
  asset_workbench.exe CACHE OUT scene art/tower/high-fidelity-mastery/scene/scene.tsv art/tower/high-fidelity-mastery/scene/overview.camera
The Blender scene loads original sources; retained original support meshes are
converted from their existing geometry, not from proprietary cache inputs.

REVIEW AND PUBLICATION
  python tools/tower/high-fidelity/publish.py
  python tools/tower/high-fidelity/verify.py
Publish.py allowlists original sources and review evidence. Preserve fixed native
camera and lighting conditions for before/after comparison. Source generation
scripts are authoritative for the selected final version; historical .blend
files and native exports preserve earlier iterations that used earlier code.

The optional native_delivery.py weld test is not the selected final model. It
welds only within a component, preserving group boundaries. Adopting its altered
vertex weighting would require recompiling and revalidating animations.

Art approval is separate from conversion success. Keep the face expressive, the
mechanism understandable, the masonry load path legible and the material palette
coherent. Do not add details that disappear or corrupt in the native renderer.
''',encoding='utf8')
(OUT/'FILES.sha256.json').write_text(json.dumps({p.relative_to(OUT).as_posix():sha(p)for p in OUT.rglob('*')if p.is_file()and p.name!='FILES.sha256.json'},indent=2))
print('REVIEW_PACKAGE',OUT,sum(p.stat().st_size for p in OUT.rglob('*')if p.is_file()))
