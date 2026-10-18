"""Allowlisted static-art evidence. Original cache geometry never copied."""
from run import *
import shutil,hashlib,html
OUT=REPO/'art/tower/static-production-benchmark';OUT.mkdir(parents=True,exist_ok=True)
selected={'anvil':'final2','workstation':'final2','machine':'final','creature':'final2'}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def cp(src,dest):
    dest=OUT/dest;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(src,dest)
def img(path,label):return f'<figure><a href="{path}"><img loading="lazy" src="{path}" alt="{html.escape(label)}"></a><figcaption>{label}</figcaption></figure>'
def grid(items):return '<div class="grid">'+''.join(img(p,l)for p,l in items)+'</div>'
manifest=dict(client_date='2026-10-17',assignment='High-Fidelity Static Asset Production Benchmark',model_label='GPT-6.1 Sol as specified by user; exact backend identity unavailable',base_commit='0f84faa',branch='experiment/static-production-benchmark',renderer_sha256=sha(NATIVE),renderer_modified=False,codec_modified=False,static_only=True,cache_geometry_published=False,finished_astra_meshes_imported=False,cindermaw_mesh_reused=False,live_map_modified=False,assets={},new_sources={p.name:sha(p)for p in HERE.glob('*.py')},shared_sources={})
for path in [HERE.parent/'mesh-study/mesh_tools.py',HERE.parent/'high-fidelity/refine.py',HERE.parent/'high-fidelity/ground.py',HERE.parent/'asset-workbench/native_codec.py']:
    manifest['shared_sources'][path.relative_to(REPO).as_posix()]=sha(path)
for kind,ver in selected.items():
    src=ROOT/'runs'/(kind+'-'+ver);dest=Path('assets')/kind
    for name in ['source.blend','asset.model','geometry.json','conversion.json','authoring.json']:cp(src/name,dest/name)
    for p in src.glob('*.png'):cp(p,dest/p.name)
    for p in src.glob('*.camera'):cp(p,dest/p.name)
    manifest['assets'][kind]=dict(source_sha256=sha(src/'source.blend'),model_sha256=sha(src/'asset.model'),conversion=json.loads((src/'conversion.json').read_text()),authoring=json.loads((src/'authoring.json').read_text()))
    for version in ['first','refined','final']:
        src2=ROOT/'runs'/(kind+'-'+version)
        if src2==src:continue
        for name in ['quarter.png','front.png','side.png','source.blend','asset.model','conversion.json']:cp(src2/name,Path('iterations')/(kind+'-'+version)/name)
for p in (ROOT/'reference-comparison').iterdir():
    if p.suffix in ['.png','.json']:cp(p,Path('reference-comparison')/p.name)
metadata=json.loads((REFERENCE/'metadata.json').read_text());manifest['reference']={k:metadata[k]for k in ['selector','name','revision','model_ids','definition_sha256']};manifest['anvil_image_metrics']=json.loads((ROOT/'reference-comparison/silhouette.json').read_text())
for label in [*selected,'scene']:
    for p in (ROOT/'presentation'/label).iterdir():
        if p.suffix in ['.png','.json']or p.name=='beauty-studio.blend':cp(p,Path('blender')/label/p.name)
for p in (ROOT/'scene').iterdir():
    if p.suffix in ['.png','.camera']or p.name=='showcase.blend':cp(p,Path('scene')/p.name)
for name in ['source.blend','asset.model','geometry.json','conversion.json']:cp(ROOT/'scene/ground'/name,Path('scene/ground')/name)
rows=[]
for row in (ROOT/'scene/scene.tsv').read_text().splitlines():
    f=row.split('\t')
    if f[0]=='owned':
        p=Path(f[1]);label=p.parent.name
        match=next((k for k,v in selected.items()if label==k+'-'+v),None)
        if match:f[1]='../assets/'+match+'/asset.model'
        elif label=='ground':f[1]='ground/asset.model'
        else:cp(p,Path('scene/support')/label/'asset.model');f[1]='support/'+label+'/asset.model'
    rows.append('\t'.join(f))
(OUT/'scene/scene.tsv').write_text('\n'.join(rows)+'\n')
for p in HERE.glob('*.py'):cp(p,Path('sources')/p.name)
if(ROOT/'validation.json').exists():cp(ROOT/'validation.json','VALIDATION.json')
manifest['gpu']=json.loads((ROOT/'presentation/creature/beauty-device.json').read_text());(OUT/'MANIFEST.json').write_text(json.dumps(manifest,indent=2))
page='''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>ASCENT · Static production benchmark</title><style>
:root{color-scheme:dark}*{box-sizing:border-box}body{margin:0;background:#10171b;color:#d9e1de;font:16px/1.65 system-ui,sans-serif}main{max-width:1420px;margin:auto;padding:42px 28px}h1{font-size:clamp(34px,5vw,66px);line-height:1.1;color:#efe3c9;margin:16px 0 25px}h2{font-size:31px;color:#ddc297;margin-top:66px}h3{font-size:22px;color:#b7cfc3}p{max-width:1080px}a{color:#dbbd83}.eyebrow{font-size:12px;letter-spacing:.16em;color:#ad9976}.lead{font-size:21px}.note{padding:19px 24px;background:#1b282c;border-left:3px solid #b39763}nav{display:flex;gap:20px;flex-wrap:wrap;margin:25px 0}.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(310px,1fr));gap:18px}figure{margin:16px 0;background:#152025;border:1px solid #2c3b3f;overflow:hidden;border-radius:5px}img{display:block;width:100%;height:auto}figcaption{padding:10px 16px;color:#b4c3c1;font-size:14px}details{padding:16px;background:#19262a;margin:15px 0}summary{cursor:pointer;color:#d9bd85}table{width:100%;border-collapse:collapse;font-size:14px;margin:22px 0}td,th{text-align:left;vertical-align:top;border-bottom:1px solid #344349;padding:12px}th{color:#d9bd85}footer{margin-top:65px;color:#94aaa6;font-size:13px}@media(max-width:700px){main{padding:24px 15px}.grid{grid-template-columns:1fr}}</style><main>
<div class="eyebrow">ASCENT / STATIC ASSET PRODUCTION / ORIGINAL MODELING</div><h1>Four static studies.<br>One native quality standard.</h1>
<p class="lead">An independently reconstructed anvil, a framed clamping workstation, an adjustable plate mill, and Slagclaw: a new furnace scorpion built from original anatomical geometry.</p>
<div class="note">All game-appearance evidence below comes from the unchanged Rust renderer. Blender images are labeled separately. These are completed review candidates; artistic acceptance remains with the user. No rigs, animation, interactions or gameplay were developed for this experiment.</div>
<nav><a href="#scene">Workshop</a><a href="#anvil">A · Reference test</a><a href="#workstation">B · Workstation</a><a href="#machine">C · Plate mill</a><a href="#creature">D · Slagclaw</a><a href="#workflow">Methods & sources</a></nav>
<h2 id="scene">The workshop · actual Rust renders</h2><p>The workstation occupies the front service position, the plate mill sits near the masonry entrance, and Slagclaw approaches over ash and scoria. The original arch, walls and barrels are retained supporting scenery from the preserved Sol work. The four benchmark models are newly authored; no completed Astra benchmark mesh or Cindermaw geometry was imported.</p>
'''
page+=img('scene/overview.png','Native whole-scene overview · 1300 × 950 · ambient 64 / contrast 768, light [-30, -50, -30]')
page+=grid([('scene/reverse-overview.png','Native reverse overview · full rear construction'),('scene/player-west.png','Native player view · west · 765 × 503'),('scene/player-east.png','Native player view · east · 765 × 503')])
page+=grid([('scene/creature-close.png','Native creature close-up in context'),('scene/machine-detail.png','Native machinery detail in context')])
page+='<h2 id="anvil">A · Independently modeled OSRS anvil</h2><p>Revision 239.1 object 2097, model 1251, is used privately as the authentic reference. The authored reconstruction uses new lofts, head and heel geometry, and a deliberately blunt lifted horn. No cache vertex array, topology, Blender round trip or previous completed anvil mesh is an input to the authoring script.</p><p>The first pass had an overly pointed horn and box-like waist. The selected result improves the foot-to-waist transition, top bevel, heel connector and horn profile. Native scale, light and reference cameras are held constant.</p>'
for view in ['front','side','rear','quarter']:
    page+=img('reference-comparison/pair-'+view+'.png','Reference left / independently authored right · '+view+' · identical crop and native camera')
page+='<details><summary>50% overlays and full-size matched renders</summary>'+grid([('reference-comparison/overlay-'+v+'.png','50% overlay · '+v)for v in ['front','side','rear','quarter']])+'</details>'
page+='<p>Measured silhouette overlap improves from 90.3% to 93.5% in front, 92.8% to 97.9% from the side, and 91.1% to 93.9% at three-quarter. These describe silhouette only. Smooth-normal gradients, waist facets and small bevel transitions still differ; the images are the primary evidence.</p>'
page+=grid([('iterations/anvil-first/quarter.png','Initial independent pass · same reference camera'),('assets/anvil/quarter.png','Selected reconstruction · same reference camera')])
page+=img('assets/anvil/close-quarter.png','Close native authored anvil · separate inspection camera')
descriptions={
'workstation':('B · Framed plate-clamping workstation','A fixed jaw and screw-driven moving jaw clamp a narrow metal plate. The vise bed is anchored to a framed bench with diagonal braces, lower crossrails, a stretcher, iron feet, tool tray, punches, rivet hammer and a storage drawer. It is a coherent workholding station, rather than an anvil on a platform.','Refinement makes the bench load path explicit, adds the clamped workpiece, develops the screw and thrust collar, and joins the lower frame to all four legs.'),
'machine':('C · Adjustable two-roll plate mill','Two polished rolls define a real feed gap. The upper journals sit between guide strips and are adjusted by vertical spindles. A lower handwheel drives a phased gear pair. Feed supports, bearing blocks, oilers, locknuts and frame bolts explain its assembly.','Refinement adds the coupled gears, adjusts their phase, supports the entering sheet and makes the upper journal guidance legible. This is a static mechanical design study, not a claim of engineering certification.'),
'creature':('D · Slagclaw, furnace scorpion','An original eight-legged arthropod with a continuous cephalothorax, paired eyes and orbital brows, a frontal mouth shield, mandibles, asymmetric pincers, chitin limbs, individual tarsal claws and a segmented curved tail. The silhouette and anatomy are independent of Cindermaw.','The first added ribs overlapped the shell poorly in native rendering. Refinement removes those floating strips, sculpts the shell itself, assigns a broad median seam and lateral rim, adds defensive spines, and fills the tail intersegment gaps with flexible membranes.')}
for kind,(title,desc,note)in descriptions.items():
    page+=f'<h2 id="{kind}">{title}</h2><p>{desc}</p><p>{note}</p>'+grid([(f'iterations/{kind}-first/quarter.png','Initial structured pass · fixed native camera'),(f'assets/{kind}/quarter.png','Selected refined model · same camera')])
    if kind=='creature':page+=grid([('iterations/creature-refined/quarter.png','Intermediate · overlapping surface ribs'),('iterations/creature-final/quarter.png','Intermediate · cleaner but overly plain carapace')])
    page+='<details open><summary>Native front, side, rear and gameplay views</summary>'+grid([(f'assets/{kind}/{v}.png',v.title())for v in ['front','side','rear','gameplay']])+'</details>'
    page+='<details><summary>Blender presentation and camera-matched comparison</summary>'+img(f'blender/{kind}/beauty.png','BLENDER CYCLES · studio presentation')+grid([(f'assets/{kind}/quarter.png','NATIVE RUST · fixed quarter view'),(f'blender/{kind}/native-match.png','BLENDER CYCLES · equivalent projection and source scale')])+'</details>'
page+='<h3>Creature surface and silhouette diagnostics</h3>'+grid([('blender/creature/clay.png','BLENDER · clay surface study'),('blender/creature/topology.png','BLENDER · topology inspection')])
page+='<details><summary>Orthographic creature reference views</summary>'+grid([('blender/creature/'+v+'.png','BLENDER orthographic '+v)for v in ['front','side','rear']])+'</details>'
page+='<h2>Blender workshop presentation</h2><p>Cycles uses the confirmed GTX 1660 Ti CUDA device. Three area lights, a neutral background and soft contact shadows support review. These physical lighting effects are not substituted for native appearance.</p>'+img('blender/scene/beauty.png','BLENDER CYCLES · original static workshop scene')
page+='<h2 id="workflow">What improved the work</h2><table><tr><th>Method</th><th>Visible contribution</th></tr><tr><td>Supported surface cages and controlled fusion</td><td>A continuous creature head/thorax with deliberately tapered limbs, curved claws and connected tail sections.</td></tr><tr><td>Purposeful industrial components</td><td>The workstation’s jaws, screw and bracing and the mill’s bearings, guide strips, feed gap and gear relationship explain their function.</td></tr><tr><td>Asset-side relief and material regions</td><td>Carapace features live in its own surface instead of competing overlapping strips; broad regions survive native shading more cleanly.</td></tr><tr><td>Fixed native review</td><td>Exposed the pointed anvil horn, weak initial bench construction, unphased gears, overlapping ribs and tail gaps.</td></tr><tr><td>Equivalent Blender/native projection</td><td>Allows geometry and source shading to be inspected without confusing studio light with a game-render improvement.</td></tr></table>'
page+='<table><tr><th>Selected model</th><th>Vertices</th><th>Triangles</th><th>Collapsed micro-triangles</th></tr>'
for kind,d in manifest['assets'].items():
    c=d['conversion'];page+=f'<tr><td>{kind.title()}</td><td>{c["vertices"]:,}</td><td>{c["triangles"]:,}</td><td>{c["collapsed_triangles"]:,}</td></tr>'
page+='</table><p>Native export evaluates the editable delivery modifiers, rounds an export copy to the integer lattice, removes collapsed triangles and encodes through the preserved codec. Maximum rounding is 0.5 native units per axis, with 128 units per tile. The dense first creature exceeded a codec section limit; controlled simplification reduced delivery density while retaining the silhouette. Model counts are constraints and diagnostics, not quality scores.</p>'
page+='<p>Broad face-color regions and native interpolated shading are the selected material strategy. The source retains UVs, but arbitrary PBR roughness, normal maps and shader networks are not claimed to survive the converter. The renderer, cache identifiers and live map remain unchanged.</p><p><strong>Remaining limitations:</strong> the anvil’s smooth shading is not exact; small screw threads, rivet details and thin creature tips soften after quantization; Slagclaw’s shell still has broad quiet areas and its facets remain visible at close scale. The mill and workstation have understandable mechanisms, but no load or manufacturing analysis has been performed. Final artistic approval remains with the user.</p>'
page+='<p><a href="WORKFLOW.txt">Reusable workflow and commands</a> · <a href="MANIFEST.json">Model, material and source manifests</a> · <a href="FILES.sha256.json">File identities</a></p><footer>Isolated branch: experiment/static-production-benchmark · preserved Sol checkpoint: 0f84faa · new static designs · authentic anvil reference images are Jagex imagery; protected cache geometry, textures and direct round trips are excluded.</footer></main></html>'
(OUT/'index.html').write_text(page,encoding='utf8')
(OUT/'WORKFLOW.txt').write_text('''STATIC ART PRODUCTION WORKFLOW

Preserved baseline: 0f84faa. No completed Astra model or Cindermaw geometry is an
input to the four new authoring functions. Generic Sol cage, union, mesh and
rendering utilities are reused. New models contain no rig or animation payload.

Dependencies: Blender 5.2.2 LTS, Python 3.13, Pillow, existing native workbench.
Set the local constants in tools/tower/static-benchmark/run.py: ROOT, BLENDER,
NATIVE, CACHE, REFERENCE. CACHE is a privately held authorized revision239.1
JS5 resource and is not distributed. The frozen renderer hash is in MANIFEST.

Generate/review the preserved selected versions:
  python tools/tower/static-benchmark/run.py final2 anvil workstation creature
  python tools/tower/static-benchmark/run.py final machine
Initial design review:
  python tools/tower/static-benchmark/run.py first
  python tools/tower/static-benchmark/run.py refined
The selected generation source corresponds to the final revisions. Historic
.blend and model files preserve earlier authoring code states. Choose a fresh
version name to preserve prior outputs during a new art iteration.

ANVIL
The author function reads no cache or prior geometry. Reference_compare.py reads
only rendered reference BMPs and authored PNGs, with one fixed camera and light.
  python tools/tower/static-benchmark/reference_compare.py
The public package includes rendered reference evidence and identity metadata,
never the original mesh, source textures, decoded arrays or round-trip files.

ORIGINAL MODEL METHODS
designs.py includes reusable loft, gear and shell construction. It imports the
preserved Sol cage/union utilities for supported contours and continuous organic
mass. Carapace relief is sculpted directly; material regions are assigned to the
same shell rather than overlapping surface strips. Components are static and
exported without skeletal labels. Mechanical threads, jaw faces, bearings and
feed clearances are deliberate functional details, not arbitrary complexity.

NATIVE REVIEW
run.py produces front/side/rear/quarter and gameplay views. Keep camera and light
constant across an iteration. Identify major primary form errors first, then
secondary component relationships and finally the useful tertiary details.
Inspect the exported model rather than assuming the Blender result survives.

BLENDER
  python tools/tower/static-benchmark/present.py
studio.py selects CUDA when available and records the device. It supports beauty,
clay, topology, orthographic views and native-match. Native-match converts the
native look camera to Blender coordinates and focal pixels to sensor/lens values.
All images are labeled; source precision and physical lighting still differ.

SCENE
  python tools/tower/static-benchmark/scene.py
  blender -b --factory-startup --python-exit-code 1 --python tools/tower/static-benchmark/scene_studio.py -- OUT
  blender -b OUT/scene/showcase.blend --python-exit-code 1 --python tools/tower/static-benchmark/studio.py -- OUT/presentation/scene beauty
The six native views include reverse overview and machine detail. Architecture
and supporting scenery are retained original Sol assets, identified by relative
paths in the portable manifest. Four benchmark models are new independent work.

Portable native replay:
  asset_workbench.exe CACHE OUT scene art/tower/static-production-benchmark/scene/scene.tsv art/tower/static-production-benchmark/scene/overview.camera

PUBLICATION
  python tools/tower/static-benchmark/publish.py
  python tools/tower/static-benchmark/verify.py
The allowlist preserves original editable sources, native models, render evidence,
reports and tools. Keep cache models and textures private. Compare visual quality
first; technical conversion and silhouette metrics are supporting evidence.
''',encoding='utf8')
(OUT/'.gitattributes').write_text('*.model binary\n*.blend binary\n*.png binary\n')
(OUT/'FILES.sha256.json').write_text(json.dumps({p.relative_to(OUT).as_posix():sha(p)for p in OUT.rglob('*')if p.is_file()and p.name!='FILES.sha256.json'},indent=2))
print('STATIC_REVIEW_PACKAGE',OUT,sum(p.stat().st_size for p in OUT.rglob('*')if p.is_file()))
