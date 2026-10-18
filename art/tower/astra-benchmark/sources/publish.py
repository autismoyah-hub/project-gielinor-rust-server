"""Allowlisted native review packaging. Never copies original cache geometry."""
import sys,json,hashlib,shutil,html,subprocess
from pathlib import Path
from PIL import Image,ImageChops,ImageDraw
HERE=Path(__file__).resolve().parent
BG=(16,21,26)
def load(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def cropbox(images,pad=20):
    boxes=[ImageChops.difference(im.convert('RGB'),Image.new('RGB',im.size,BG)).getbbox()for im in images]
    boxes=[b for b in boxes if b]
    return (max(0,min(b[0]for b in boxes)-pad),max(0,min(b[1]for b in boxes)-pad),min(images[0].width,max(b[2]for b in boxes)+pad),min(images[0].height,max(b[3]for b in boxes)+pad))
def img(file,label):
    return f'<figure><a href="{file}"><img loading="lazy" src="{file}" alt="{html.escape(label)}"></a><figcaption>{html.escape(label)}</figcaption></figure>'

def run(root,out):
    if out.exists():raise ValueError('Preserve prior packages; choose fresh output')
    out.mkdir(parents=True)
    for name in ['views','motion','sources','scene']:(out/name).mkdir()
    cache=root.parents[2]/'resource-staging/server/work/integration/a-kotlin/.data/cache/js5'
    finals={'A-anvil':root/'v3/A-anvil','B-riveter':root/'v2/B-riveter','C-stamp':root/'v3/C-stamp','D-sootsnout':root/'v2/D-sootsnout'}
    reference=root/'private/reference/object-2097';assets={}
    notes={
      'A-anvil':'Independent reconstruction. Widened and offset the foot, lifted and blunted the horn, shortened the striking block and heel, and matched the restrained iron value. Remaining differences are bevels, smooth-normal shading and the waist transition. No cache mesh was imported.',
      'B-riveter':'Original split-die riveting station. The open die slot, long bending horn, forged clamps and low timber bolster define its construction. Refinement reduced the timber saturation and added a seated rivet and hammer to explain use.',
      'C-stamp':'Original counterweighted screw press. Refinement connected the lever to the spindle, modeled a real helical thread and added guide springs and a billet. The first pass left the transmission unclear; the final load path reads through the open throat.',
      'D-sootsnout':'Original Sootsnout ash burrower. Broadened the forepaws, thickened the forelegs, replaced peg-like ears with swept wedges and subdued the dorsal plates. Attack coils back with an open jaw before striking and recovering. Death folds the legs before the roll and holds the corpse.'}
    for key,src in finals.items():
        dst=out/'assets'/key;dst.mkdir(parents=True)
        for file in ['asset.model','asset.blend','recipe.json','geometry.json','authoring-report.json']:shutil.copy2(src/file,dst/file)
        action_ids={}
        for folder in sorted(src.iterdir()):
            if folder.is_dir()and(folder/'animation.json').exists():
                d=dst/folder.name;d.mkdir();data=load(folder/'animation.json')
                for f in folder.iterdir():
                    if f.suffix in ['.bin','.json']:shutil.copy2(f,d/f.name)
                action_ids[folder.name]=dict(skeleton=data['skeleton_id'],frame_group=data['frame_group'],ticks=sum(r['length']for r in data['frames']))
        before=out/'before'/key;before.mkdir(parents=True)
        for f in ['recipe.json','asset.model']:shutil.copy2(root/'v1'/key/f,before/f)
        report=load(src/'authoring-report.json')
        assets[key]=dict(classification='independent faithful reconstruction'if key.startswith('A')else'original creature'if key.startswith('D')else'original inspired asset'if key.startswith('B')else'original industrial prop',identity=report['identity'],geometry=report['geometry']['exact_decoded'],model_sha256=sha(src/'asset.model'),blend_sha256=sha(src/'asset.blend'),actions=action_ids,native_motion_validation=report['motion'],refinement=notes[key])
        old_matched=root/'matched-before'/key
        camera=reference/'camera.txt' if key=='A-anvil' else src/'native/camera.txt'
        if not (old_matched/'front.bmp').exists():
            subprocess.run([str(root/'tools/asset_workbench.exe'),str(cache),str(old_matched),'render',str(root/'v1'/key/'asset.model'),str(camera),'64','768'],check=True)
        for view in ['front','side','quarter','gameplay']:
            final=Image.open(src/'native'/f'{view}.bmp').convert('RGB');first=Image.open(old_matched/f'{view}.bmp').convert('RGB')
            final.save(out/'views'/f'{key}-{view}-full.png');ims=[first,final]
            if key=='A-anvil':ims.insert(0,Image.open(reference/f'{view}.bmp').convert('RGB'))
            box=cropbox(ims)
            final.crop(box).save(out/'views'/f'{key}-{view}.png');first.crop(box).save(out/'views'/f'{key}-{view}-before.png')
            if key=='A-anvil':
                ref=ims[0];ref.save(out/'views'/f'A-reference-{view}-full.png');ref.crop(box).save(out/'views'/f'A-reference-{view}.png')
                Image.blend(ref,final,.5).crop(box).save(out/'views'/f'A-overlay-{view}.png')
        shutil.copy2(camera,dst/'camera.txt')
    creature=finals['D-sootsnout'];clips=[]
    for action in ['idle','walk','attack','hit','death']:
        files=sorted((creature/'motion'/action).glob('pose-*.bmp'));ticks=[int(f.stem.split('-')[1])for f in files]
        ims=[Image.open(f).convert('RGB')for f in files];box=cropbox(ims,24);frames=[im.crop(box)for im in ims]
        durations=[(ticks[i+1]-t)*20 if i+1<len(ticks)else 20 for i,t in enumerate(ticks)]
        frames[0].save(out/'motion'/f'{action}.gif',save_all=True,append_images=frames[1:],duration=durations,loop=0,disposal=2)
        selected=sorted({round(i*(len(files)-1)/6)for i in range(7)});w,h=frames[0].size
        sheet=Image.new('RGB',(w*len(selected),h+26),BG);draw=ImageDraw.Draw(sheet)
        for col,index in enumerate(selected):sheet.paste(frames[index],(col*w,26));draw.text((col*w+8,6),f'{action} / tick {ticks[index]}',fill='white')
        sheet.save(out/'motion'/f'{action}-strip.png');clips.append(dict(action=action,ticks=ticks,duration_ms=sum(durations),common_crop=list(box),native_tick_ms=20))
    for action,times in [('attack',[0,12,21,27,42]),('death',[0,21,42,63,84])]:
        ims=[Image.open(root/v/'D-sootsnout/motion'/action/f'pose-{t:04}.bmp').convert('RGB')for v in ['v1','v2']for t in times]
        box=cropbox(ims);w,h=ims[0].crop(box).size;sheet=Image.new('RGB',(w*len(times),(h+25)*2),BG);draw=ImageDraw.Draw(sheet)
        for i,im in enumerate(ims):
            x=(i%len(times))*w;y=(i//len(times))*(h+25);sheet.paste(im.crop(box),(x,y+25));draw.text((x+5,y+5),f'{"Before"if i<len(times)else"After"} / {times[i%len(times)]}',fill='white')
        sheet.save(out/'motion'/f'{action}-refinement.png')
    stage=out/'scene';rows=[];seen={}
    for line in (root/'vignette/scene.tsv').read_text().splitlines():
        f=line.split('\t')
        if f[0]=='light':rows.append(line);continue
        source=Path(f[1]);match=next((key for key,p in finals.items()if source==p/'asset.model'),None)
        if match:f[1]='../assets/'+match+'/asset.model'
        else:
            name=source.parent.name+'.model'
            if name in seen and seen[name]!=str(source):raise ValueError('Scene filename collision')
            seen[name]=str(source);shutil.copy2(source,stage/name);f[1]=name
        if f[6]!='-':f[6]='../assets/D-sootsnout/idle'
        rows.append('\t'.join(f))
    (stage/'scene.tsv').write_text('\n'.join(rows)+'\n')
    for name in ['overview','player-west-close','player-east-close','creature-close']:
        Image.open(root/'vignette/final-views'/name/'scene.bmp').save(out/'views'/f'scene-{name}.png')
        shutil.copy2(root/'vignette'/f'{name}.camera',stage/f'{name}.camera')
    shutil.copy2(root/'vignette/floor/recipe.json',stage/'paving-recipe.json')
    for p in HERE.glob('*.py'):shutil.copy2(p,out/'sources'/p.name)
    fidelity={v:load(root/v/'A-anvil/comparison/comparison.json')for v in ['v1','v2','v3']};tools=HERE.parent/'asset-workbench'
    manifest=dict(title='ASCENT — controlled artistic benchmark',date='2026-10-17',revision='239.1',base_server_commit='28db23a3498bfc54fa285765508a27b2cc07cf00',native_client_source_commit='23a94f75707f0503a9efe610fe99bfce5089b575',native_binary_sha256=sha(root/'tools/asset_workbench.exe'),blender='5.2.2 LTS / d13f752e3b9c',python=sys.version,model_configuration=dict(requested_label='Astra',actual_model_id='Not exposed to this agent; cannot independently verify',thinking_level='Not exposed to this agent; cannot independently verify',interpretation='Do not attribute a model-specific gain without the client run configuration'),tool_source_sha256={p.name:sha(p)for p in tools.glob('*.py')},benchmark_source_sha256={p.name:sha(p)for p in HERE.glob('*.py')},reference=load(reference/'metadata.json'),assets=assets,fidelity=fidelity,motion=clips,contact_diagnostics=load(root/'contact-v2/motion-diagnostics.json'),protected_reference_geometry_published=False,live_gameplay_modified=False,artistic_approval=False,limitations=['Silhouette overlap measures registration, not complete artistic fidelity. Bevel and shading differences remain.','Original assets use sparse face colors and native directional lighting, no textures or emitted light.','Creature paws and material transitions remain angular. Walk contact shows small quantized drift, not verified physical locomotion.','No gameplay integration, pathfinding, NPC combat, roof or occlusion work.'])
    (out/'BENCHMARK.json').write_text(json.dumps(manifest,indent=2))
    page='''<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>ASCENT — Artistic Benchmark</title><style>body{margin:0;background:#10151a;color:#ddd8ca;font:16px/1.55 system-ui}main{max-width:1180px;margin:auto;padding:40px 24px}h1{font:46px Georgia;margin:12px 0}h2{font:30px Georgia;margin-top:60px;color:#dfb775}h3{font:21px Georgia}p{max-width:900px}nav{display:flex;gap:24px;flex-wrap:wrap;margin:25px 0}a{color:#d5aa66}small,.muted{color:#a8aaa9}.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:18px}.pair{display:grid;grid-template-columns:1fr 1fr;gap:18px}figure{margin:0;background:#151c22;border:1px solid #30363a;padding:12px}img{display:block;max-width:100%;max-height:620px;margin:auto;image-rendering:pixelated}figcaption{padding:8px 0;color:#bbb;font-size:14px}.hero img{width:100%}.rule{padding:16px;border-left:3px solid #b89055;background:#192128}table{border-collapse:collapse}td,th{padding:8px 20px 8px 0;text-align:left;border-bottom:1px solid #30363a}.strip{overflow:auto}.strip img{max-width:none;max-height:260px}details{margin:20px 0}@media(max-width:650px){.pair{grid-template-columns:1fr}h1{font-size:34px}}</style><main><small>ASCENT / CINDERWORKS / CONTROLLED ART TEST / 17 OCTOBER 2026</small><h1>Forged forms. Living ash.</h1><p>One faithful reconstruction, three original designs and a small workshop vignette. Every new asset was authored in headless Blender, saved and reopened, converted to the native model format and rendered by the existing Rust renderer. First passes are preserved.</p><nav><a href="#A">A · Fidelity</a><a href="#B">B · Riveting station</a><a href="#C">C · Screw press</a><a href="#D">D · Sootsnout</a><a href="#scene">Vignette</a><a href="BENCHMARK.json">Evidence manifest</a></nav>'''
    page+='<div class="hero">'+img('views/scene-overview.png','Original scene assembly — B, C and D with existing original supporting scenery')+'</div>'
    page+='<h2 id="A">A · Faithful anvil reconstruction</h2><p>'+notes['A-anvil']+'</p><p class="muted">Left: authentic revision 239.1 object 2097 (model 1251), Jagex reference imagery. Right: independently authored reconstruction. Identical camera, light 64/768 and native renderer. Paired detail crops are identical; full frames are linked below. Protected cache geometry is excluded.</p>'
    for view in ['front','side','quarter']:page+='<h3>'+view.replace('quarter','Three-quarter').title()+'</h3><div class="pair">'+img(f'views/A-reference-{view}.png','Authentic reference')+img(f'views/A-anvil-{view}.png','Independent reconstruction')+'</div>'
    page+='<details><summary>Overlay, first pass and measured silhouette fidelity</summary><div class="pair">'+img('views/A-overlay-quarter.png','50% overlay; no registration or warp')+img('views/A-anvil-quarter-before.png','First independently authored pass')+'</div><table><tr><th>View</th><th>First pass IoU</th><th>Final IoU</th></tr>'
    for view in ['front','side','quarter','gameplay']:page+=f'<tr><td>{view}</td><td>{fidelity["v1"][view]["iou"]:.1%}</td><td>{fidelity["v3"][view]["iou"]:.1%}</td></tr>'
    page+='</table><p><a href="views/A-reference-front-full.png">Full reference frame</a> · <a href="views/A-anvil-front-full.png">Full authored frame</a>. Silhouette overlap is not a substitute for artistic judgment.</p></details>'
    for key,title in [('B-riveter','B · Split-die riveting station'),('C-stamp','C · Counterweighted screw press'),('D-sootsnout','D · Sootsnout ash burrower')]:
        page+=f'<h2 id="{key[0]}">{title}</h2><p>{notes[key]}</p><div class="grid">'
        for view in ['front','side','quarter']:page+=img(f'views/{key}-{view}.png',view.title())
        page+='</div><details><summary>First pass → final, and gameplay scale</summary><div class="pair">'+img(f'views/{key}-quarter-before.png','First pass — matched final camera')+img(f'views/{key}-quarter.png','Final')+'</div>'+img(f'views/{key}-gameplay-full.png','Uncropped native gameplay-scale view')+'</details>'
        if key=='D-sootsnout':
            page+='<h3>Five complete native actions</h3><p>All exported ticks are included at 20 ms per native tick. Walk moves the legs through stance and swing; no whole-creature translation is used in these previews. Death includes the authored terminal hold.</p><div class="grid">'
            for action in ['idle','walk','attack','hit','death']:page+=img(f'motion/{action}.gif',action.replace('hit','Hit reaction').title())
            page+='</div><details><summary>Action refinement and contact sheets</summary><div class="strip">'+img('motion/attack-refinement.png','Attack before / after')+img('motion/death-refinement.png','Death before / after')+'</div>'
            for action in ['idle','walk','attack','hit','death']:page+='<div class="strip">'+img(f'motion/{action}-strip.png',action+' sampled poses')+'</div>'
            page+='</details><p class="muted">329 native pose samples match the independent classic-transform calculation exactly. Walk contact proxy: no penetrating or floating stance samples; mean drift 2.23 native units, maximum 5.37 (128 units per tile). This is not physics validation. Generic body-support and attack-centroid diagnostic fields do not map to this anatomy and are not used as evidence.</p>'
    page+='<h2 id="scene">Small workshop vignette</h2><p>The press sits against the load-bearing wall; the riveting station leaves working clearance in front. The burrower occupies the floor between the service arch and work area. Supporting masonry, barrel, rack and lantern are unchanged original assets from the previous delivery.</p><div class="pair">'+img('views/scene-player-west-close.png','Player-style view — west')+img('views/scene-player-east-close.png','Player-style view — east')+'</div><div class="hero">'+img('views/scene-creature-close.png','Close composition — creature, workstations and service threshold')+'</div>'
    page+='<h2>Review outcome and reproduction</h2><p>The strongest measured result is the anvil reconstruction. Both industrial props have clearer construction than their first passes. The creature has five distinct actions, but its angular anatomy and material transitions remain the least mature part of the set. This package is for review; it does not establish production acceptance or prove that one model configuration outperforms another.</p><div class="rule">Requested run label: Astra. The runtime does not expose a verifiable model identifier or thinking-effort setting to this agent. Those fields are marked unverified rather than guessed.</div><p><a href="REPRODUCE.txt">Reproduction notes</a> · <a href="BENCHMARK.json">Tool hashes, identities, metrics and limitations</a> · <a href="sources/designs.py">Blender input generation</a> · <a href="sources/vignette.py">Scene assembly</a></p><p class="muted">Original reference mesh, cache payloads and extracted original animations are excluded. All new geometry and motion come from authored components and controls. No live gameplay changes.</p></main>'
    (out/'index.html').write_text(page,encoding='utf-8')
    (out/'REPRODUCE.txt').write_text("""Use tools/tower/asset-workbench/author_workflow.py with each assets/*/recipe.json, a fresh output directory, the recorded native binary, authorized revision-239.1 cache, Blender 5.2.2 and a separate identity ledger. Seed the ledger with the identity/asset-key pairs in BENCHMARK.json for byte-identical IDs. Keep outputs outside the cache and source checkout. A uses the authentic object:2097 camera and light 64/768; its camera.txt is included. B/C/D use render-own. Saved .blend and independently authored geometry are included. Native classic action payloads are under assets/D-sootsnout/<action>.

Generate inputs with tools/tower/astra-benchmark/designs.py OUTPUT --refined. Invoke author_workflow per recipe; do not import original cache geometry. The unrefined generator supplies the first pass. Exact saved recipe JSON is the reproduction authority.

Render the portable scene: asset_workbench.exe CACHE FRESH_OUTPUT scene scene/scene.tsv scene/overview.camera. Repeat for the player and creature-close cameras. Supporting scenery comes from the existing 28db23a original delivery, not extra benchmark categories.

The frozen native renderer SHA256 is in BENCHMARK.json. It was not rebuilt or modified for this benchmark. Existing Blender generation, model conversion, classic action compilation and comparison utilities were reused unchanged. The benchmark branch is experiment/astra-art-benchmark.

Model configuration: Astra was requested; actual model ID and thinking effort are not exposed to this agent. Consult the client run configuration before treating this as a model-to-model comparison.
""",encoding='utf-8')
    forbidden=[p for p in out.rglob('*')if p.is_file()and('raw-model' in p.name or p.name.startswith('main_file_cache')or p.name=='definition.txt')]
    if forbidden:raise AssertionError(forbidden)
    files={p.relative_to(out).as_posix():sha(p)for p in out.rglob('*')if p.is_file()}
    (out/'FILES.sha256.json').write_text(json.dumps(files,indent=2))
    print('BENCHMARK_PACKAGE_COMPLETE',len(files),'files',sum(p.stat().st_size for p in out.rglob('*')if p.is_file()),'bytes')
if __name__=='__main__':run(Path(sys.argv[1]).resolve(),Path(sys.argv[2]).resolve())
