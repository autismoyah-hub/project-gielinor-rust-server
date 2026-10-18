"""Allowlisted original sources and native evidence; no cache geometry is copied."""
import json,shutil,hashlib,html,re,datetime
from pathlib import Path
from PIL import Image
from run import ROOT,REPO,HERE,NATIVE
OUT=REPO/'art/tower/mesh-production-study';OUT.mkdir(parents=True,exist_ok=True)
def cp(src,dst):
    dst=OUT/dst;dst.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(src,dst)
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def img(path,label):return f'<figure><a href="{path}"><img loading="lazy" src="{path}" alt="{html.escape(label)}"></a><figcaption>{label}</figcaption></figure>'
def grid(items):return '<div class="grid">'+''.join(img(p,l)for p,l in items)+'</div>'
stats={}
for kind in ['architecture','machine','creature']:
    for ver in ['first','refined','final']+(['union','control']if kind=='creature'else[]):
        src=ROOT/'runs'/(kind+'-'+ver);dest=Path('assets')/(kind+'-'+ver)
        for name in ['source.blend','asset.model','geometry.json','conversion.json','authoring.json','recipe.json','source-rigged.blend','rig.json','motion-validation.json','idle.gif']:
            if (src/name).exists():cp(src/name,dest/name)
        for name in ['front','side','quarter','rear','gameplay']:
            cp(src/(name+'.png'),dest/(name+'.png'));cp(src/(name+'.camera'),dest/(name+'.camera'))
        if (src/'conversion.json').exists():stats[kind+'-'+ver]=json.loads((src/'conversion.json').read_text())
        if kind=='creature'and ver=='final':
            for p in (src/'idle').iterdir():
                if p.suffix in ['.bin','.json']:cp(p,dest/'idle'/p.name)
for p in (ROOT/'runs/previous-best').glob('*.png'):cp(p,Path('comparisons/previous-best')/p.name)
for name in ['material-only','coarse-UV-bake','subdivided-UV-bake','flat-shading']:
    src=ROOT/'compatibility-v2'/name
    for p in src.iterdir():
        if p.is_file()and p.suffix in ['.png','.model','.json','.camera']:cp(p,Path('compatibility')/name/p.name)
for name in ['original-albedo.png','uv-source.blend']:cp(ROOT/'compatibility-v2'/name,Path('compatibility')/name)
for p in (ROOT/'scene').glob('*.png'):cp(p,Path('scene')/p.name)
for p in (ROOT/'scene').glob('*.camera'):cp(p,Path('scene')/p.name)
ground_folder=ROOT/'scene/ground-v3'
if not ground_folder.exists():ground_folder=ROOT/'scene/ground'
for name in ['source.blend','asset.model','geometry.json','conversion.json']:cp(ground_folder/name,Path('scene/ground')/name)
rows=[]
for row in (ROOT/'scene/scene.tsv').read_text().splitlines():
    f=row.split('\t')
    if f[0]=='owned':
        p=Path(f[1]);label=p.parent.name
        if label in ['architecture-final','machine-final','creature-final']:f[1]='../assets/'+label+'/asset.model'
        elif label in ['ground-v3','ground']:f[1]='ground/asset.model'
        else:cp(p,Path('scene/support')/label/'asset.model');f[1]='support/'+label+'/asset.model'
    rows.append('\t'.join(f))
(OUT/'scene/scene.tsv').write_text('\n'.join(rows)+'\n')
for p in HERE.glob('*.py'):cp(p,Path('sources')/p.name)
if (ROOT/'validation.json').exists():cp(ROOT/'validation.json','VALIDATION.json')
manifest=dict(study='Cinderworks mesh production study',date=datetime.date.today().isoformat(),branch='experiment/mesh-production-study',base='2c47342',native_renderer_sha256=digest(NATIVE),renderer_modified=False,workbench_codec_modified=False,production_integrated=False,provider_calls=0,paid_provider_cost_usd=0,hardware='GTX 1660 Ti 6GB VRAM; Blender 5.2.2 LTS; Python 3.13',sources={p.name:digest(p)for p in HERE.glob('*.py')},statistics=stats,rig=json.loads((ROOT/'runs/creature-final/motion-validation.json').read_text()),protected_reference=dict(kind='object',id=2097,name='Anvil',revision='239.1',model_id=1251,geometry_published=False,rendered_in_scene=True),model_configuration='Exact runtime model ID and thinking level are not exposed; no model-specific result claimed.',approval='User artistic review required; no production-quality claim.')
(OUT/'MANIFEST.json').write_text(json.dumps(manifest,indent=2))
head='''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Cinderworks · Mesh production study</title><style>
:root{color-scheme:dark}*{box-sizing:border-box}body{margin:0;background:#10171a;color:#d9dedb;font:16px/1.6 system-ui,sans-serif}main{max-width:1380px;margin:auto;padding:44px 28px}h1{font-size:clamp(32px,5vw,64px);line-height:1.08;margin:15px 0 24px;color:#f0e6ce}h2{font-size:30px;color:#e7d5af;margin-top:70px}h3{font-size:21px;color:#bcd1c4}p{max-width:1000px}a{color:#d2ba82}nav{display:flex;gap:18px;flex-wrap:wrap;margin:24px 0}.tag{font-size:13px;letter-spacing:.15em;color:#b5a17b}.lead{font-size:21px;max-width:1050px}.notice{padding:18px 24px;border-left:3px solid #bc9b61;background:#1b2529}.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(300px,1fr));gap:18px}figure{margin:14px 0;background:#141e22;border:1px solid #29363a;border-radius:5px;overflow:hidden}img{width:100%;height:auto;display:block}figcaption{padding:10px 15px;color:#b6c4c3;font-size:14px}table{width:100%;border-collapse:collapse;margin:25px 0;font-size:14px}th,td{text-align:left;vertical-align:top;border-bottom:1px solid #334043;padding:12px}th{color:#d2ba82}code{color:#c3d5c7}details{background:#192327;padding:16px;margin:16px 0}summary{cursor:pointer;color:#d2ba82}footer{margin-top:70px;color:#99aaa9;font-size:13px}.three{display:grid;grid-template-columns:repeat(3,1fr);gap:18px}@media(max-width:850px){.three{grid-template-columns:1fr}main{padding:24px 16px}} </style><main>
<div class="tag">ASCENT / ORIGINAL ASSET PRODUCTION / CONTROLLED LOCAL STUDY</div><h1>Better meshes.<br>A visible, bounded improvement.</h1>
<p class="lead">A molded foundry entrance, twin-cylinder pump and horned ash drake, authored with evaluated Blender meshes and reviewed through the unchanged Rust renderer.</p>
<div class="notice">This is an improvement study, not an assertion that the final art standard has been reached. The architecture and machine benefit from deliberate construction. The creature remains the weakest asset: its contours are continuous, but the torso and scute transitions still need an artist’s refinement. Nothing has been integrated into the live game.</div>
<nav><a href="#showcase">Showcase</a><a href="#assets">Three assets</a><a href="#methods">Method comparison</a><a href="#conversion">Conversion</a><a href="#providers">External methods</a><a href="#workflow">Recommendation & sources</a></nav>
'''
body=head+'<h2 id="showcase">The pump yard</h2><p>Native scene submission, ambient 64 / contrast 768, fixed light [-30, -50, -30]. The anvil at right is authentic Jagex object 2097, loaded privately from the local revision 239.1 cache. All other geometry is original, including retained earlier wall, barrel and rack assets. No original cache mesh is in this package.</p>'
body+=img('scene/overview.png','Whole-scene overview · native 1200 × 900')
body+=grid([('scene/player-west.png','Normal player camera · west · 765 × 503'),('scene/player-east.png','Normal player camera · east · 765 × 503')])
body+=img('scene/creature-detail.png','Close native composition · no Blender lighting or image-generation enhancement')
body+='<h2 id="assets">Three original assets</h2>'
descriptions={
'architecture':('01 · Ember-lock entrance','Thirteen voussoirs form a deep molded arch. Stepped jambs, buttresses, iron reinforcement, a tongs-and-ember keystone, louvers and a raised crown grate give the structure a purpose.', 'First pass: uniform stone, plain opening, weak focal point. Refinement: molded reveal and buttresses, restrained stone batches, keystone and hardware. Remaining: rear faces are simpler; wear is restrained rather than richly hand-painted.'),
'machine':('02 · Twin-cylinder foundry pump','A shared crankshaft drives two piston rods. Bearing pedestals, bolted cylinder flanges, an intake manifold, discharge vessel and handwheel establish a readable assembly.', 'First pass: polygonal wheel, unexplained plumbing, crank rods nearly meeting the axle center. Refinement: swept spokes, clear assembly hierarchy, manifold, valve, pressure vessel and eccentric crank pins. Remaining: static assembly only; no mechanical simulation, animated cycle or engineering certification.'),
'creature':('03 · Cindermaw ash drake','A low-bodied quadruped with a heavy shoulder girdle, swept cheek horns, broad muzzle, jointed digging digits and overlapping dorsal scutes. Original design and geometry.', 'First pass: overlapping muscle volumes, oversized eyes, blunt feet. Refinement: volume union and contour relaxation, reduced eyes, separate digits and claw tips, more deliberate plate rhythm. Remaining: soft torso anatomy, sparse surface patterning, visible head-neck seam at some angles. Not approved as finished creature art.')}
for kind,(title,desc,note)in descriptions.items():
    body+=f'<h3>{title}</h3><p>{desc}</p><p>{note}</p>'
    body+=grid([(f'assets/{kind}-first/quarter.png','First pass · matched camera'),(f'assets/{kind}-final/quarter.png','Refined mesh · matched camera')])
    body+='<details><summary>Front, side, rear and ordinary gameplay views; editable source</summary>'+grid([(f'assets/{kind}-final/{v}.png',v.title())for v in ['front','side','rear','gameplay']])+f'<p><a href="assets/{kind}-final/source.blend">Editable Blender source</a> · <a href="assets/{kind}-final/asset.model">Converted model</a> · <a href="assets/{kind}-final/conversion.json">Conversion audit</a></p></details>'
body+='<h3>Minimal rig compatibility</h3><p>The mesh retains component groups for the head and jaw. All 61 native pose samples match the existing independent classic-transform evaluator exactly. The GIF covers the complete 60-tick cycle at 40 ms per authored two-tick frame. This proves rigid group compatibility only; no weighted skinning, locomotion, foot-contact or combat-animation claim is made.</p>'+img('assets/creature-final/idle.gif','Native head/jaw idle · 1.2 seconds · complete loop')
body+='<h2 id="methods">A controlled change, then artistic refinement</h2><p>The key controlled pair uses the same input masses, horns, eyes, plates, materials, camera and light. Only volume union, contour relaxation and simplification change. The final model then adds separate design corrections. More triangles alone are not the success criterion.</p>'
body+=grid([('assets/creature-first/quarter.png','Separate Blender masses · before union'),('assets/creature-union/quarter.png','Same masses · union and topology refinement'),('assets/creature-final/quarter.png','Hybrid result · anatomy, toes, eyes and plates refined')])
body+='<details><summary>Existing procedural workbench comparisons</summary><p>A fresh quadruped-template control uses the existing workbench and the same broad horned-drake brief. It is a starter template, not the best previous result. The previously refined Sootsnout is therefore also shown under the same camera to avoid comparing only against a weak control. These are design comparisons, not identical-topology experiments.</p>'+grid([('assets/creature-control/quarter.png','Fresh existing-workbench starter control'),('comparisons/previous-best/quarter.png','Previous refined Sootsnout · same camera'),('assets/creature-final/quarter.png','Current refined drake')])+'</details>'
body+='<table><tr><th>Method</th><th>Actual test</th><th>Finding</th></tr><tr><td>Existing primitive workbench</td><td>Fresh template; previous best retained</td><td>Fast, reproducible structural parts. Weak at continuous anatomy without explicit topology work.</td></tr><tr><td>Blender mesh authoring</td><td>Bevels, radial arch profiles, curved sweeps, named material components</td><td>Useful architectural and mechanical improvement; structure remains editable.</td></tr><tr><td>Volume union + simplification</td><td>Same-mass controlled creature pair</td><td>Removes intersection seams. Does not independently invent convincing anatomy; can introduce lumps.</td></tr><tr><td>Hybrid refinement</td><td>Revised eyes, digits, plate rhythm, silhouette and mechanism</td><td>Strongest tested local workflow. Requires repeated native inspection.</td></tr><tr><td>Specialized AI-to-3D</td><td>Access, hardware and terms evaluated; no inference calls</td><td>Untested geometry quality. No claim that paid generation would improve these results.</td></tr></table>'
body+='<h2 id="conversion">What conversion preserves—and loses</h2><p>The new adapter exports evaluated Blender meshes into the existing native codec. It preserves component transforms, material regions, native group labels and chosen flat/interpolated shading. Source meshes retain UVs and full coordinate precision. Only the exported copy is rounded to the native integer lattice.</p>'
body+='<table><tr><th>Final asset</th><th>Vertices</th><th>Triangles</th><th>Collapsed triangles removed</th></tr>'
for kind in ['architecture','machine','creature']:
    s=stats[kind+'-final'];body+=f'<tr><td>{kind.title()}</td><td>{s["vertices"]:,}</td><td>{s["triangles"]:,}</td><td>{s["collapsed_triangles"]}</td></tr>'
body+='</table><p>Maximum coordinate rounding: 0.5 native units per axis (128 units per tile). Tiny bevel and claw triangles sometimes collapse; counts above are explicit loss, not a quality score. Zero-area triangles are excluded. Budgets remain below 12,000 vertices / 20,000 triangles per asset, but these are study caps, not established production budgets. The unchanged codec also enforces section-size and signed-index limits.</p>'
body+='<h3>Original UV albedo experiment</h3><p>The same four-color authored pattern is tested three ways. A diffuse-only material loses it. Baking onto two triangles retains only two samples. Subdividing to 512 triangles preserves the broad pattern through native face colors. This is a bounded approximation, not preservation of arbitrary texture UVs.</p>'
body+=grid([('compatibility/material-only/front.png','Diffuse only · 512 triangles · pattern lost'),('compatibility/coarse-UV-bake/front.png','UV bake · 2 triangles · undersampled'),('compatibility/subdivided-UV-bake/front.png','UV bake · 512 triangles · broad pattern retained')])
body+='<p><a href="compatibility/original-albedo.png">Original authored image</a> · <a href="compatibility/uv-source.blend">Editable UV test source</a>. The adapter samples a single albedo image at triangle UV centroids and converts RGB to native HSL. Full shader graphs, UV transforms, linear/display color-space interchange, PBR roughness, metallic and normal maps are not implemented. Native simple cache-texture mappings already exist in the codec; this study does not install new textures or alter the authoritative renderer.</p>'
body+=grid([('compatibility/flat-shading/quarter.png','Same final mesh · flat native shading'),('assets/creature-final/quarter.png','Same final mesh · interpolated native shading')])
body+='<p>The identical geometry becomes visibly noisy with flat shading. Native interpolation materially helps, but the residual contour problems remain. All showcase faces use priority 0 and alpha 0; transparent materials were not tested. Winding is recalculated before export and the coordinate conversion preserves orientation. All four viewing directions are published.</p>'
body+='<details><summary>A native scene failure and its geometry fix</summary><p>A broad ground top face drew over pavers at the player angle. Replacing it with local foundation skirt cells removes the broad overlapping face and follows the paved outline. The paired images below retain the original camera. No renderer sorting rule was changed.</p>'+grid([('scene/player-west-ground-failure.png','Original slab · painter-order failure'),('scene/ground-fix-same-camera.png','Local ground geometry · same camera')])+'</details>'
body+='<h2 id="providers">External generation: investigated, not adopted</h2><p>No Meshy, Tripo or Hunyuan connector is exposed, and no provider credential variable was present. No accounts, purchases, uploads or generation requests were made. Protected OSRS inputs stayed local. Prices below are documented API credits, not an inferred dollar conversion.</p><table><tr><th>Provider</th><th>Access, quoted cost and constraints</th><th>Data / license decision</th></tr>'
body+='<tr><td><a href="https://docs.meshy.ai/en/api/quick-start">Meshy</a></td><td>Paid-plan API key and prepaid credits. Current 7.1 image/multiview: 20 credits mesh-only, 30 with 2K textures; text preview and texture refine billed separately. Offers remesh and rigging; generated topology and native conversion remain untested. <a href="https://docs.meshy.ai/en/api/pricing">Pricing</a></td><td>Paid users own output subject to law; free output is CC BY 4.0. Terms permit non-enterprise training unless separately agreed. API output retention is three days outside Enterprise. No adoption without an authorized account and acceptable terms. <a href="https://www.meshy.ai/terms-of-use">Terms</a></td></tr>'
body+='<tr><td><a href="https://docs.tripo3d.ai/">Tripo</a></td><td>Authenticated prepaid API. H2/H3 image or multiview: 20 credits mesh-only / 30 textured; P1: 40 / 50. Segmentation, low-poly and rig checks exist; extras may add credits. No model was generated. <a href="https://docs.tripo3d.ai/get-started/pricing.html">Pricing</a></td><td>Paid-user terms grant input/output rights and exclude training use; free-user rights differ materially. Service-use license still applies. Recommend only a bounded paid evaluation after authorization, not automatic adoption. <a href="https://www.tripo3d.ai/terms">Terms §5.2</a></td></tr>'
body+='<tr><td><a href="https://github.com/Tencent-Hunyuan/Hunyuan3D-2.1">Hunyuan3D 2.1</a></td><td>Local shape inference documents 10 GB VRAM, texture 21 GB, combined 29 GB. This GTX 1660 Ti has 6 GB. The documented configuration is not supported here; no large weights downloaded. An offloaded variant would require a separate measured trial and may be slow.</td><td>Community license, not a permissive blanket license: territory excludes EU, UK and South Korea; large-scale use has additional terms. Local inference avoids provider upload, but deployment/output restrictions matter. <a href="https://github.com/Tencent-Hunyuan/Hunyuan3D-2.1/blob/main/LICENSE">License</a></td></tr></table>'
body+='<h2 id="workflow">Dominant bottleneck and recommended workflow</h2><p><strong>For these tested assets, design and geometry refinement dominate.</strong> The same native renderer and material palette render a coherent molded arch, mechanism and smoother creature after mesh-level changes. The controlled union pair removes seams without a renderer change; the creature’s remaining weak anatomy survives both shading modes. That evidence does not support blaming the renderer alone.</p><p>Material loss is a separate, demonstrated bottleneck for texture-dependent assets. The UV experiment shows exactly why a textured cloud model could still look poor after conversion. Specialized generation remains an untested hypothesis, not the established solution.</p><ol><li>Plan silhouette, component relationships and focal details from multiple views before adding density.</li><li>Use precise Blender meshes for architecture and machinery; reserve volume union for continuous organic forms.</li><li>Refine the editable mesh, then simplify by curvature. Keep horns, claws, eyes and mechanical interfaces as named components.</li><li>Author material regions for native HSL. Use bounded UV-to-face-color baking where the geometry budget supports it; do not rely on unsupported PBR effects.</li><li>Render front, side, rear, quarter and player views with fixed native cameras. Revise the three largest visual weaknesses.</li><li>Only after static approval, assign classic groups and validate a minimal motion proof. Broader locomotion is a separate task.</li></ol>'
body+='<p><strong>Measured cost/time:</strong> $0 in external provider charges. Final Blender build/export took approximately 3–4 seconds per asset on this machine; each five-view native render batch took a few seconds. These measurements exclude prompt work, modeling decisions and inspection, which were shared across assets and not timed per model. Token and electricity costs were not measured.</p><p><strong>Technical failures retained:</strong> an initial relative image-save path failed and was fixed to an absolute path; Blender desktop-thumbnail writes were denied while source saves succeeded; coarse UV baking lost detail; the scene’s broad slab produced painter-order artifacts. None is hidden by a beauty render.</p><p><strong>Conclusion:</strong> adopt the evaluated-mesh and native-review workflow for the next art iteration. It produces a visible local improvement, but this study does not establish three production-ready assets. The user retains artistic approval. A paid AI-to-3D trial would still need to prove superior geometry after this same conversion and review process.</p>'
body+='<p><a href="MANIFEST.json">Manifest and mesh statistics</a> · <a href="REPRODUCE.txt">Reproduction instructions</a> · <a href="FILES.sha256.json">File hashes</a> · <a href="sources/mesh_tools.py">Reusable Blender mesh tools</a> · <a href="sources/author.py">Original source designs</a></p><footer>Original geometry authored for The Tower. Authentic anvil imagery belongs to Jagex and is included solely as locally authorized context; no proprietary cache payload is published. Exact agent model ID / thinking level is unavailable to this runtime. Source branch: experiment/mesh-production-study. Native renderer preserved.</footer></main></html>'
(OUT/'index.html').write_text(body,encoding='utf8')
(OUT/'REPRODUCE.txt').write_text('''CINDERWORKS MESH PRODUCTION STUDY

Run from the server repository, with Python 3.13 + Pillow and Blender 5.2.2.
Use the existing private revision239.1 JS5 cache; it is not distributed here.
The frozen asset_workbench.exe is identified by MANIFEST.json SHA256.
The executable and existing workbench modules are not modified by this study.

tools/tower/mesh-study/run.py contains the local path defaults (ROOT, NATIVE,
CACHE, BLENDER). Adapt these to your workstation before running.
Commands:
  python tools/tower/mesh-study/run.py first
  python tools/tower/mesh-study/run.py refined
  python tools/tower/mesh-study/run.py final

Additional controlled examples (substitute absolute paths):
  blender -b --factory-startup --python-exit-code 1 --python tools/tower/mesh-study/author.py -- creature union OUT/runs/creature-union
  blender -b --factory-startup --python-exit-code 1 --python tools/tower/mesh-study/control.py -- OUT/runs/creature-control
  blender -b --factory-startup --python-exit-code 1 --python tools/tower/mesh-study/compatibility.py -- OUT/compatibility-v2
  blender -b OUT/runs/creature-final/source.blend --python-exit-code 1 --python tools/tower/mesh-study/animate.py -- OUT/runs/creature-final
  python tools/tower/mesh-study/evidence.py

evidence.py renders the same revised ground using scene_floor.py. The retained
ground-failure image comes from the earlier broad-slab version; it is evidence
of a failed iteration, not the current source's intended output.
finish_scene.py supports the preserved local ground-v3 run. For a clean rerun,
evidence.py already produces the corrected ground in its ground directory.

Replay the published scene without running Blender:
  asset_workbench.exe CACHE OUTDIR scene art/tower/mesh-production-study/scene/scene.tsv art/tower/mesh-production-study/scene/overview.camera
Replay a published asset using a one-row scene manifest:
  light<TAB>64<TAB>768
  owned<TAB>absolute/model/path<TAB>0<TAB>0<TAB>0<TAB>0<TAB>-<TAB>0
Then use the corresponding *.camera file. Paths in scene/scene.tsv are relative.
All camera and image dimensions are preserved; no post-render relighting.

Editable .blend sources retain separate named components and UVs. The exporter
evaluates modifiers into a copy, rounds to the native lattice, removes zero-area
triangles, records losses, and calls the unchanged native_codec.model.
The simplified image bake assumes one image on active UVs with display RGB
values; it is not a general shader/PBR or color-management translator.
Static native skin groups do not constitute a deforming locomotion rig.

The study contains only original meshes. Authentic object2097 is loaded from
the user's local cache by ID at render time. Do not copy cache geometry into
this package. Paid provider requests are not part of reproduction.
''',encoding='utf8')
(OUT/'FILES.sha256.json').write_text(json.dumps({p.relative_to(OUT).as_posix():digest(p)for p in OUT.rglob('*')if p.is_file()and p.name!='FILES.sha256.json'},indent=2))
print('PUBLISHED_LOCAL',OUT,sum(p.stat().st_size for p in OUT.rglob('*')if p.is_file()))
