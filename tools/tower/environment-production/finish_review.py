"""Curate original-only publication and a separate private cache-world review."""
from pathlib import Path
import csv,json,hashlib,shutil,subprocess,html,os,statistics,re
from collections import Counter
from PIL import Image,ImageDraw,ImageFont
import native_session as n
S,E,R=n.S,n.E,n.R
P=S/'art/tower/cinderworks-environment'
ORIGIN=R/'native-20261018-130034-gpu'
def read(p):return json.loads(p.read_text())
def write(p,v):p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def rel(p,base):return os.path.relpath(p,base).replace('\\','/')
def table(headers,rows):return '<table><thead><tr>'+''.join('<th>'+html.escape(str(v))+'</th>'for v in headers)+'</tr></thead><tbody>'+''.join('<tr>'+''.join('<td>'+html.escape(str(v))+'</td>'for v in row)+'</tr>'for row in rows)+'</tbody></table>'
def figure(p,label,base):return '<figure><a href="'+rel(p,base)+'"><img loading="lazy" src="'+rel(p,base)+'" alt="'+html.escape(label)+'"></a><figcaption>'+html.escape(label)+'</figcaption></figure>'
def section(key,title,content):return '<section id="'+key+'"><h2>'+title+'</h2>'+content+'</section>'
STYLE='''body{margin:0;background:#10191c;color:#e2e4df;font:16px/1.6 system-ui}main{max-width:1400px;margin:auto;padding:28px}h1,h2{color:#e0bf87;line-height:1.2}a{color:#8bd9ce}nav{position:sticky;top:0;background:#18272cf2;padding:14px;display:flex;gap:18px;flex-wrap:wrap;z-index:2}section{scroll-margin-top:100px;padding:24px 0;border-bottom:1px solid #405056}.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(430px,1fr));gap:18px}figure{margin:0;background:#1b2a30;padding:10px}img{width:100%;height:auto;display:block}figcaption{padding:8px}table{border-collapse:collapse;width:100%;overflow:auto;display:block}td,th{text-align:left;border:1px solid #405056;padding:7px}code,pre{background:#172328}pre{white-space:pre-wrap;padding:16px}.note{border-left:4px solid #bc9359;padding:15px;background:#253033}li{margin-bottom:6px}@media(max-width:600px){.grid{grid-template-columns:1fr}main{padding:14px}}'''
def page(path,title,intro,sections):
 nav='<nav>'+''.join('<a href="#'+k+'">'+label+'</a>'for k,label,_ in sections)+'</nav>'
 text='<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>'+title+'</title><style>'+STYLE+'</style>'+nav+'<main><h1>'+title+'</h1>'+intro+''.join(section(*v)for v in sections)+'</main></html>'
 path.write_text(text,encoding='utf-8')

def main():
 catalog=read(R/'package-v1/catalog.json');props=read(R/'placements/07-whole-floor-refinement.json');nav=read(R/'07-whole-floor-refinement/navigation.json');plan=read(P/'planning/floor-plan.json');assets=read(R/'asset-verification.json');traversal=read(ORIGIN/'traversal/result.json');actions=read(ORIGIN/'interactions/result.json');collision=read(ORIGIN/'native-scene-validation.json');solids=read(R/'custom-solid-collision.json')
 # The chest assertion used fresh event UIDs, but its old summary listed the
 # oldest three messages. Correct this presentation without replaying the action.
 gated=read(ORIGIN/'interactions/reward-gated.json');actions['actions'][3]['messages']=[m['text']for m in gated['message_events']if m['uid']==11];write(ORIGIN/'interactions/result.json',actions)
 objects=list(csv.DictReader((R/'07-whole-floor-refinement/objects.tsv').open(),delimiter='\t'));counts=Counter(int(o['id'])for o in objects)
 # Publish only our authored files. No cache archive, cache-world image, password,
 # private account state or executable is copied into this directory.
 overlay=P/'overlay';overlay.mkdir(exist_ok=True)
 for p in (R/'package-v1').iterdir():
  if p.suffix in ['.model','.object']or p.name=='registry.txt':shutil.copy2(p,overlay/p.name)
 public_catalog={}
 for name,v in catalog.items():
  source=R/v['source'];out=P/'assets'/name;out.mkdir(parents=True,exist_ok=True)
  for filename in ['source.blend','asset.model','geometry.json','conversion.json','authoring.json']:shutil.copy2(source/filename,out/filename)
  g=read(source/'geometry.json');v=dict(v);v['source']='assets/'+name;v['instances']=counts[v['object_id']];v['material_colors']=len(set(g['colors']));public_catalog[name]=v
  if name in ['wall-high','wall-low']:
   shutil.copy2(R/'kit-v1'/name/'source.blend',out/'source-detail.blend')
 for name in ['arch','pump','plate-mill','workstation','ore-cart','slate-headwall','furnace-bank','pier','chimney-backdrop']:
  for mode in ['gpu','classic']:
   source=R/'asset-closeups'/(name+'-'+mode)
   if (source/'render.png').exists():
    out=P/'closeups'/(name+'-'+mode);out.mkdir(parents=True,exist_ok=True)
    for filename in ['render.png','summary.json','settings.json','camera-state.json','camera.txt']:shutil.copy2(source/filename,out/filename)
 write(P/'asset-catalog.json',public_catalog);shutil.copy2(R/'placements/07-whole-floor-refinement.tsv',P/'environment.tsv')
 # Functional IDs are recovered metadata, not replacement objects. Spawn anchors
 # remain explicit policy coordinates rather than getting a decorative inlay ID.
 contracts=read(P/'planning/interaction-contracts.json')
 for c in contracts:
  anchor=nav['anchors'][c['key']];special=c['key'].startswith(('arrival','imp-spawn','warden-spawn'));matching=anchor['existing_objects'];entries=[]
  if not special:
   for o in matching:
    w,h=int(o['width']),int(o['length']);angle=int(o['angle']);w,h=(h,w)if angle%2 else(w,h)
    entries.append({'object_id':int(o['id']),'shape':int(o['shape']),'quarter_turn':angle,'definition_size':[int(o['width']),int(o['length'])],'rotated_footprint':[w,h],'block_walk':int(o['block_walk']),'operations':o['ops']})
  c.update(object_definitions=entries,interaction_position=[anchor['standing_tile'][0]+10368,anchor['standing_tile'][1]+64,0],animation_anchor=None,orientation='Per-object quarter_turn'if entries else'Existing policy/spawn coordinate; no authored behavior',validation='Source and reachable approach. Live hook tested only for plaque, bank, altar, reward denial and portal.')
  if c['key']=='imp-spawn':c.update(npc_id=16300,encounter_bounds_local=[68,68,75,75])
  if c['key']=='warden-spawn':c.update(npc_id=16295,encounter_bounds_local=[50,67,62,77])
  if c['key']=='arrival-return':c['note']='Recovery-pad repeat-arrival policy uses local56,49. The current Tower portal return separately uses local56,56, verified live.'
 write(P/'planning/interaction-contracts.json',contracts)
 hazards={'coordinate_order':'world X,Z,plane; local relative to10368,64,0','imp_arena':[68,68,75,75],'warden_arena':[50,67,62,77],'vents':[{'key':'vent-'+str(i),'local':[x,z,0],'world':[x+10368,z+64,0],'status':'Existing source hazard anchor; behavior unchanged; no new hazard damage or animation'}for i,(x,z)in enumerate([(67,70),(67,74),(76,70),(76,74)])],'coolant':[c for c in contracts if c['key'].startswith('coolant-')],'new_decorative_controls':'No operations added to original machinery. Mesh ports and utility-connection contracts describe assembly alignment only.'}
 write(P/'planning/hazard-contracts.json',hazards)
 # Additional preserved source anchors: fishing uses NPC interaction rather than
 # a scenery object; restoration pairs are presentation state, not new controls.
 from collections import deque
 edges={}
 for q in csv.DictReader((R/'07-whole-floor-refinement/walk-edges.tsv').open(),delimiter='\t'):
  x,z,dx,dz=(int(q[k])for k in ['x','z','dx','dz']);edges.setdefault((x,z),[]).append((x+dx,z+dz))
 reachable={(56,56)};queue=deque(reachable)
 while queue:
  for q in edges.get(queue.popleft(),[]):
   if q not in reachable:reachable.add(q);queue.append(q)
 fishing=[]
 for z in [50,52,54,56]:
  # Existing CinderworksFishing deliberately accepts Chebyshev distance<=3.
  # Water NPCs need a reachable quay approach, not a fictitious adjacent tile.
  choices=sorted((q for q in reachable if max(abs(q[0]-72),abs(q[1]-z))<=3),key=lambda q:abs(q[0]-72)+abs(q[1]-z));assert choices
  x1,z1=choices[0];fishing.append({'key':'fishing-'+str(z),'npc_id':3317,'source_symbol':'0_48_48_newbiefishing','world':[10440,z+64,0],'local':[72,z,0],'standing_world':[x1+10368,z1+64,0],'existing_reach_chebyshev_tiles':3,'hook':'tower_map::npcs / npc_actions::fishing_approach / World::start_cinderworks_fishing','validation':'Source recovered, real server quay approach reachable within existing3tile policy; fishing action/full Kitchen service not exercised in this environment test','new_art_or_behavior':False})
 pairs=[]
 source=(S/'crates/world/src/tower_scenery.rs').read_text()
 pattern=r'Landmark\s*\{\s*x:\s*(\d+),\s*z:\s*(\d+),\s*dormant:\s*(\d+),\s*restored:\s*(\d+),\s*state:\s*State::(\w+),\s*angle:\s*(\d+)'
 for x,z,dormant,restored,state,angle in re.findall(pattern,source):
  x,z,dormant,restored,angle=map(int,[x,z,dormant,restored,angle]);row=next(q for q in objects if int(q['x'])==x and int(q['z'])==z and int(q['id'])==dormant)
  pairs.append({'world':[x+10368,z+64,0],'local':[x,z,0],'dormant_object':dormant,'restored_object':restored,'state':state,'quarter_turn':angle,'shape':10,'definition_footprint':[int(row['width']),int(row['length'])],'operations':[],'interaction_position':None,'animation_anchor':None,'status':'Existing per-player restoration presentation pair unchanged; not an interactable authored machine'})
 assert len(pairs)==8
 write(P/'planning/preserved-source-anchors.json',{'fishing':fishing,'restoration_pairs':pairs,'vents':[dict(v,object_id=15215,shape=22,quarter_turn=0,blocking=False,graphic=86,ambient_damage=False,source='tower_scenery::VENTS / tower_scene ambient emission')for v in hazards['vents']]})
 # Source preservation receipts cover existing heads/status and unchanged gameplay
 # files. Cache source was accessed read-only; no new pre/post cache hash claim.
 baseline=read(R/'baseline.json');preservation={}
 for name,v in {**baseline['protected_workspaces'],**{k:baseline['repos'][k]for k in ['enhanced-client','ashhorn-reference-server']}}.items():
  root=n.W/name if name in baseline['protected_workspaces']else E/name
  head=subprocess.check_output(['git','-C',str(root),'rev-parse','HEAD'],text=True).strip();status=subprocess.check_output(['git','-C',str(root),'status','--porcelain'],text=True).strip();preservation[name]={'head':head,'same_head':head==v['head'],'clean':not status};assert head==v['head']and not status,(name,status)
 source_checks={p:sha(S/p)==h for p,h in baseline['source_files'].items()if p!='crates/server/src/tower_map.rs'};assert all(source_checks.values())
 write(P/'validation/source-preservation.json',{'protected_workspaces':preservation,'unchanged_gameplay_source_hashes':source_checks,'intentional_server_changes':'Opt-in environment module and tower_map integration, no gameplay coordinate or rule changes','protected_cache':'Read-only inputs; no original cache files packaged or written. No new start/end whole-cache digest audit was performed.'})
 modes=[]
 for mode in ['classic','lite','gpu']:
  candidates=sorted(R.glob('native-*-'+mode+'/mode-result.json'));m=read(candidates[-1]);session=Path(m['session']);rows=list(csv.DictReader((session/'measured-frames.csv').open()));m['stage_mean_ms']={k:statistics.mean(float(v[k])for v in rows)for k in ['network_ms','scripts_ms','world_ms','ui_ms','present_ms']};m['session']=session.name
  if mode=='classic':m['adapter']='CPU Classic rasterizer'
  else:m['adapter']=m['renderer'].get('adapter',m['renderer'].get('stats',{}).get('adapter'))
  modes.append(m)
 write(R/'live-mode-results.json',modes)
 # Exact comparison of BOTH orthogonal and diagonal arena movement.
 a=R/'baseline-expanded/combat-eight-direction-steps.tsv';b=R/'final-expanded/combat-eight-direction-steps.tsv';assert a.read_bytes()==b.read_bytes()
 eight_cases=len(a.read_text().splitlines())-1
 errors=[s for s in solids if s['missing_blocking_tiles']];assert not errors
 collision=dict(collision)
 collision['scope']='Exact live-client versus prepared-server flags on2304floor tiles. Difference cause and whether present in a live baseline are unproven. Custom solid footprint checks and protected arena movement equality are separate tests.'
 collision['affected_source_objects']=[]
 for x,z,server_flags,client_flags in collision['collision_differences']:
  nearby=[{'local':[int(q['x']),int(q['z'])],'id':int(q['id']),'shape':int(q['shape']),'angle':int(q['angle'])}for q in objects if abs(int(q['x'])-x)<=1 and abs(int(q['z'])-z)<=1]
  collision['affected_source_objects'].append({'local':[x,z],'world':[x+10368,z+64,0],'server_flags':server_flags,'client_flags':client_flags,'xor':server_flags^client_flags,'nearby_objects':nearby})
 write(P/'validation/custom-solid-collision.json',solids);write(P/'validation/native-scene-comparison.json',collision)
 write(P/'validation/milestones.json',read(R/'milestone-validation.json'))
 compact_assets={'count':len(assets['assets']),'all_saved_source_exports_exact':assets['all_saved_source_exports_exact'],'all_native_decodes_exact':assets['all_native_decodes_exact'],'all_fresh_oriented_surfaces_and_materials_exact':assets['all_fresh_oriented_surfaces_and_materials_exact'],'all_fresh_native_bytes_exact':assets['all_fresh_native_bytes_exact'],'fresh_byte_differences':[k for k,v in assets['assets'].items()if not v['fresh_native_bytes_exact']],'note':assets['note']}
 write(P/'validation/asset-reproducibility.json',compact_assets)
 overview=R/'floor-review-v1/whole-floor-overview-gpu.png'
 scene=read(R/'floor-review-v1/arrival-gpu.bmp.renderer.json')
 technical={'status':'Integrated environment review candidate; not artistically approved or production ready','source_bases':{'server':baseline['repos']['cinderworks-environment-server']['head'],'client':baseline['repos']['cinderworks-environment-client']['head']},'floor_tiles':[48,48],'rooms':len(nav['rooms']),'reachable_tiles':nav['reachable_tiles'],'anchor_approaches':len(nav['anchors']),'new_asset_families':len(catalog),'new_placement_contracts':len(props),'actual_native_object_count':collision['native_live_objects'],'missing_native_objects':collision['missing_expected_objects'],'original_object_instances':sum(v['instances']for v in public_catalog.values()),'original_triangles_instanced_estimate':sum(v['instances']*v['triangles']for v in public_catalog.values()),'floor_scene_renderer':scene,'native_traversal':{'visited_named_areas':len(set(q['id']for q in traversal['rooms'])),'successful_segments':len(traversal['segments']),'failed_segments':len(traversal['failures']),'actual_ordinary_client_walking':True,'all_review_rooms_traversed':traversal['all_review_rooms_traversed']},'existing_live_hooks':actions['actions'],'combat_eight_direction_cases_identical':eight_cases,'custom_solid_footprints_verified':len(solids),'native_server_collision_differences':len(collision['collision_differences']),'asset_reproducibility':compact_assets,'live_mode_measurements':modes,'performance_limit':'No smooth-play target met. These are full-client frame periods, not the much shorter GPU kernel times or initialization-inclusive offline averages. Test modes ran sequentially on one consumer PC, not a controlled multi-device benchmark. CPU/client profiling and visibility work remain.','original_blend_bytes':sum((P/'assets'/name/'source.blend').stat().st_size for name in catalog),'native_conversion_losses':'Integer vertices, per-face 6/3/7-bit HSL colors; no PBR roughness/normal maps, no skin labels, no authored animation; non-flat normals supported by existing renderer. Original author colors are quantized.','renderer_handoff':'Core unchanged: src/enhanced.rs and src/enhanced/d3d11.rs. Windows D3D11 GPU/Lite, smooth normals, shadowmap, SSAO and edge AA. GDI readback/presentation remains. No PBR, dynamic furnace lights, new water reflection or volumetrics.'}
 write(P/'validation/technical-report.json',technical)
 remaining=[('Environment','Grove and lower ash courts still have sparse storytelling; the inherited rectangular circulation and broad material bands remain visible. Preserved functional cache furnaces are visually inconsistent with the new original machinery. Tall chimney/arch occlusion needs wider camera review. No claim of artistic approval.'),('Gameplay','Existing service/Imp/Warden/reward/persistence rules remain. Five existing hooks and navigation tested; complete skilling/boss/reward, defeat, reconnect grace and crash lifecycle were not accepted as a full loop. Decorative controls have no new behavior.'),('NPC artwork','Existing authorized placeholders remain. No new models, rigs, animation, AI or combat rotations were created.'),('Renderer/performance','Enhanced GPU is this environment launcher/review default. Classic and Intel Lite actually load. Full-client pacing is poor at1280x900; do not equate low GPU time with high client FPS. Core renderer/ORACLE parity untouched.'),('Server/client integration','14 native/server collision tile differences are disclosed with exact flags. Their cause and baseline status are not proven. All55 custom solid footprints and1656 arena movement cases passed; this does not erase those differences. Direct reconnect hit duplicate-account admission; mode tests used ordinary owned-server drain/restart. Optional external Kronos NPC combat catalog was absent (0 IDs/1file failure).')]
 (P/'REMAINING_WORK.txt').write_text('\n\n'.join(k+': '+v for k,v in remaining)+'\n',encoding='utf-8')
 # Reusable original-only visual catalogue.
 asset_figures=''.join(figure(P/'closeups'/(name+'-gpu/render.png'),name+' — Enhanced GPU; original geometry only',P)for name in ['arch','pump','plate-mill','workstation','ore-cart','slate-headwall','furnace-bank','pier','chimney-backdrop'])
 plan_content=figure(P/'planning/floor-plan.png','48×48 full floor registry; coordinates retained',P)+table(['Area','Name','Bounds','Purpose'],[(q['id'],q['name'],q['bounds'],q['function'])for q in plan['rooms']])
 source_links='<p><a href="planning/RECOVERED_DESIGN.md">Recovered design</a> · <a href="planning/connectivity.mmd">Connectivity</a> · <a href="planning/interaction-contracts.json">Functional contracts</a> · <a href="planning/hazard-contracts.json">Hazard anchors</a> · <a href="planning/preserved-source-anchors.json">Fishing and restoration pairs</a> · <a href="asset-catalog.json">Asset catalogue</a> · <a href="validation/technical-report.json">Technical report</a> · <a href="README.txt">Integration and reproduction</a></p>'
 intro='<p class="note">Integrated environment review candidate. The actual native-world gallery is private because it contains protected cache assets. This publishable page contains original artwork, a drawn floor plan and source-safe test metadata only. Artistic approval remains with the user.</p>'+source_links
 mode_table=table(['Actual live mode','Adapter','Frames','Median period ms','p95 period ms','Failed presentations'],[(m['mode'],m['adapter'],m['frames'],round(m['frame_period_ms']['median'],2),round(m['frame_period_ms']['p95'],2),m['failed_frames'])for m in modes])
 proof='<p>Actual native client:13 named areas,55 ordinary walking segments,671 scenery objects present. Five existing hook categories exercised. All55 original solid footprints blocked in the native scene;1656 protected arena movement outcomes match the source baseline. Fourteen native/server collision differences remain open.</p>'+mode_table+'<p>Performance samples are full-client1280×900 after warmup. No smooth-play performance claim. Same saved account retained across normally drained owned-server restarts.</p>'
 public_sections=[('plan','Floor plan',plan_content),('assets','Original assets','<div class="grid">'+asset_figures+'</div>'),('proof','Integration evidence',proof+source_links),('remaining','Remaining work',table(['Category','Outstanding work'],remaining))]
 page(P/'index.html','The Cinderworks — Floor One environment study',intro,public_sections)
 # Private, navigable evidence review links exact native cache frames rather than
 # accidentally copying them into the public Git tree.
 review=R/'review';review.mkdir(exist_ok=True)
 world=R/'floor-review-v1'
 gallery=[]
 from render_floor import VIEWS
 for name in VIEWS:
  if name=='whole-floor':continue
  gallery.append(figure(world/(name+'-gpu.png'),name+' — Enhanced GPU, prepared native scene; no actors/UI/animation',review))
 comparisons=''
 for name in ['arrival','workshop','waterworks','imp-gallery','warden-foundry']:
  comparisons+='<h3>'+name+'</h3><div class="grid">'+figure(world/(name+'-gpu.png'),'Enhanced GPU — NVIDIA',review)+figure(world/(name+'-classic.png'),'Classic — same geometry/camera',review)+'</div>'
 native_figures=''.join(figure(ORIGIN/'traversal'/q['frame'],q['id']+' actual native walk / server tick'+str(q['tick']),review)for q in traversal['rooms'])
 hook_figures=''.join(figure(ORIGIN/'interactions'/(name+'.png'),'Actual native '+name,review)for name in ['plaque-start','bank-open','altar-pray','reward-gated','portal-outside','portal-return'])
 mode_figures=''
 for m in modes:
  folder=R/m['session'];bmp=folder/('frame-'+str(m['capture_sequence'])+'.bmp');png=folder/'measured-client.png';Image.open(bmp).save(png);mode_figures+=figure(png,m['mode']+' live client, same sanctuary/camera, account state preserved',review)
 private_intro='<p class="note">PRIVATE: native world images contain protected cache assets. Do not commit or upload this directory. Review candidate only; no artistic approval, full gameplay-loop acceptance or smooth-play claim.</p><p>The level is loaded in the actual native game world. Static prepared-scene frames supplement the real walking/action evidence below. New decorative scenery remains separate from existing functional IDs and hooks.</p>'
 private_sections=[('overview','Whole floor',figure(overview,'Enhanced GPU whole-floor overview; native prepared scene, actors absent',review)+figure(P/'planning/floor-plan.png','Named registry and route',review)),('areas','Player views','<div class="grid">'+''.join(gallery)+'</div>'),('assets','Original object close-ups','<div class="grid">'+''.join(figure(P/'closeups'/(name+'-gpu/render.png'),name,review)for name in ['arch','pump','plate-mill','workstation','ore-cart','slate-headwall','furnace-bank','pier','chimney-backdrop'])+'</div>'),('walk','Native traversal','<p>55 successful ordinary Action::Walk segments;13 distinct named subareas, no failed segment, return to sanctuary. Native captures include actual actors and UI. Route files record observed positions and server ticks. No teleport/camera-only substitution.</p><p><a href="'+rel(ORIGIN/'traversal/result.json',review)+'">Full traversal log</a></p><div class="grid">'+native_figures+'</div>'),('hooks','Existing interactions',table(['Hook','Observed outcome'],[(v['hook'],v.get('outcome','Exit/return inventory and journal preserved'))for v in actions['actions']])+'<div class="grid">'+hook_figures+'</div>'),('modes','Rendering modes',comparisons+mode_table+'<div class="grid">'+mode_figures+'</div>'),('contracts','Contracts and validation',proof+'<p><a href="'+rel(P/'planning/interaction-contracts.json',review)+'">25 functional contracts</a> · <a href="'+rel(P/'planning/prop-placement-contracts.json',review)+'">105 original placement contracts</a> · <a href="'+rel(P/'planning/hazard-contracts.json',review)+'">Hazard and coolant anchors</a> · <a href="'+rel(P/'validation/native-scene-comparison.json',review)+'">14 collision differences</a> · <a href="'+rel(P/'validation/technical-report.json',review)+'">Technical evidence</a></p>'),('remaining','Remaining work',table(['Category','Outstanding work'],remaining))]
 page(review/'index.html','The Cinderworks — integrated native environment review',private_intro,private_sections)
 # Compact private contact sheet for actual visual QA. Public art stays separate.
 sheet=Image.new('RGB',(1800,5*430),'#111b20');draw=ImageDraw.Draw(sheet);font=ImageFont.truetype('C:/Windows/Fonts/arial.ttf',22)
 names=list(VIEWS)
 for i,name in enumerate(names):
  im=Image.open(overview if name=='whole-floor'else world/(name+'-gpu.png')).convert('RGB');im.thumbnail((590,390));x=(i%3)*600;y=(i//3)*430;sheet.paste(im,(x+(600-im.width)//2,y+35));draw.text((x+10,y+3),name,font=font,fill='#e1c38f')
 sheet.save(review/'enhanced-contact.png')
 # Structural verification covers all local images/links; browser execution is
 # unnecessary. The review is intentionally a static offline document.
 checks=[]
 for path in [P/'index.html',review/'index.html']:
  missing=[]
  for link in re.findall(r'(?:src|href)="([^"]+)"',path.read_text(encoding='utf-8')):
   if not link.startswith(('#','http'))and not(path.parent/link).exists():missing.append(link)
  checks.append({'page':str(path),'missing_local_links':missing})
 write(R/'review-link-validation.json',checks)
 print('REVIEW_PREPARED',review/'index.html','public',P/'index.html','links',checks,flush=True)

if __name__=='__main__':main()
