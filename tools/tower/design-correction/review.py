"""Publish the design checkpoint and actionable source/integration handoff."""
from pathlib import Path
import json,html,hashlib,os,re,shutil
from PIL import Image,ImageDraw,ImageFont
from plan import P,R,S,ROOMS
from render import VIEWS
def read(p):return json.loads(p.read_text(encoding='utf-8'))
def esc(v):return html.escape(str(v))
def table(head,rows):return '<table><thead><tr>'+''.join('<th>'+esc(v)+'</th>'for v in head)+'</tr></thead><tbody>'+''.join('<tr>'+''.join('<td>'+esc(v)+'</td>'for v in row)+'</tr>'for row in rows)+'</tbody></table>'
def fig(path,label):return '<figure><a href="'+path+'"><img loading="lazy" src="'+path+'" alt="'+esc(label)+'"></a><figcaption>'+esc(label)+'</figcaption></figure>'
def main():
 plan=read(P/'floor-plan.json');validation=read(P/'validation.json');collision=read(P/'collision-investigation.json');perf=read(P/'performance-investigation.json');report=read(P/'render-report.json')
 # A static rendered diagram complements the editable Mermaid source. It shows
 # prerequisites rather than inventing a linear order for the three services.
 im=Image.new('RGB',(1700,800),'#172127');d=ImageDraw.Draw(im);font=lambda n:ImageFont.truetype('C:/Windows/Fonts/arial.ttf',n)
 nodes={'Arrival':(80,330),'Hearth / tools':(300,330),'Mine / smelt / forge':(560,120),'Fish / cook shrimp':(560,330),'Bones / altar':(560,540),'ALL THREE':(840,330),'Ash Imp':(1090,330),'Ash Warden':(1340,330),'Hearth reward':(1340,560),'Existing exit':(1090,560),'Optional leak basin':(840,120)}
 def edge(a,b,color='#ceb079',dashed=False):
  x,y=nodes[a];xx,yy=nodes[b];a=(x+90,y+40);b=(xx+90,yy+40)
  if xx==x:a=(x+90,y+80);b=(xx+90,yy)
  elif yy==y:a=(x+180,y+40)if xx>x else(x,y+40);b=(xx,yy+40)if xx>x else(xx+180,yy+40)
  d.line([a,b],fill=color,width=3);import math;t=math.atan2(b[1]-a[1],b[0]-a[0]);d.polygon([b,(b[0]-13*math.cos(t-.5),b[1]-13*math.sin(t-.5)),(b[0]-13*math.cos(t+.5),b[1]-13*math.sin(t+.5))],fill=color)
 for a,b in [('Arrival','Hearth / tools'),('Hearth / tools','Mine / smelt / forge'),('Hearth / tools','Fish / cook shrimp'),('Hearth / tools','Bones / altar'),('Mine / smelt / forge','ALL THREE'),('Fish / cook shrimp','ALL THREE'),('Bones / altar','ALL THREE'),('ALL THREE','Ash Imp'),('Ash Imp','Ash Warden'),('Ash Warden','Hearth reward'),('Hearth reward','Existing exit')]:edge(a,b)
 edge('Mine / smelt / forge','Optional leak basin','#719d91')
 for name,(x,y)in nodes.items():d.rounded_rectangle((x,y,x+180,y+80),radius=10,fill='#334447',outline='#ac9a77',width=2);d.text((x+90,y+40),name,font=font(18),anchor='mm',fill='#f0d7a5')
 d.text((80,45),'EXISTING GAMEPLAY ORDER / PROPOSED SPATIAL EXPERIENCE',font=font(31),fill='#e1bf86');d.text((80,690),'Three services in any order. Bones require a valid resource source; either supply court can serve that role.',font=font(23),fill='#c8d0ca');d.text((80,735),'No new combat, rewards, hazards, unlock rules or NPC production. This diagram is a source-backed design contract.',font=font(22),fill='#c8d0ca');im.save(P/'route-and-interactions.png')
 definitions=[('Established gameplay','Prepare Forge/Kitchen/Altar in any order, then Imp, then Warden, then existing first/replay reward and departure. Tool recovery, bank, defeat/recovery and durable run state remain.','tower_run.rs; simulation/cinderworks_actions.rs; tower_arrival.rs'),('Agreed artistic direction','Sanctuary identity fromA, exploration/waterways fromB, and anticipation toward the foundry fromC. The user rejected all three as final layouts and the last native floor is a technical prototype. No prior layout approval is implied.','User art-direction messages and this correction brief'),('New layout proposal','A broken horseshoe around a collapsed drainage cut, with short preparation branches, a wet exploration loop, an attached retort, a receiver hall and a short completion return. Twenty-five explicit anchor relocations; no migration executed.','floor-plan.json; proposed-npc-relocations.json'),('Technical constraints','Current coordinates are mutable content configuration, but not all are already parameterized. Bounds, spawns, portal/recovery, restoration pairs, hazards and saved positions must move together. Plane0 terrain and valid native clipping remain required.','tower_layout.rs; tower_map.rs; tower_scene.rs; cinderworks_encounter.rs; migration checklist')]
 source_audit='''RECOVERED REQUIREMENTS AND DESIGN AUTHORITY

Server source base:75e8ce364013756754873cc037b18d6a85c8a266
Client source base:29d665a9e41995632d942cea27e2f6782eca561d
The original prototype and its26assetfamilies, tests, collision snapshots and
cache-world gallery remain preserved. New design work is on its own branch.

The active Rust run has Forge/Kitchen/Altar service bits, then Imp, then Warden.
The eleven-task historical Kotlin diary is not the active progression contract.
The journal recommends mining copper+tin, smelting bronze and forging a dagger;
the current forged_weapon hook also recognizes the configured bronze scimitar.
Net fishing and cooking shrimp use the existing Cinderworks work queue. Fishing
approach is Chebyshev<=3tiles, not an imaginary adjacent water tile. Altar service
requires an actual successful bone offering; the Pray response alone is not credit.
Fresh self-supplied preparation therefore needs a bone source, although both supply
courts are not mandatory visits. Chicken/cow/death/drop hooks are existing systems,
not new content added by the art plan.

Run.begin cannot erase an active run or a pending reward. Defeat_Imp requires all
three services; defeat_Warden requires Imp. The source stages reward inventory
before committing the paid flag: first clear500coins+beer, replay25coins+beer.
The reward stays at the familiar hearth and the existing portal remains departure.
No reward amount, requirement, persistence or replay rule was changed.

Imp is an8x8movement envelope, Warden13x11. The proposed Warden translation(+25,-16)
sums to9 and preserves the modulo3phase pattern when translated within the same
floor coordinate frame. A naive arbitrary relocation could shift that pattern.
The future native test must retain telegraph timing, quench windows, ownership,
coolant relative positions, player/NPC footprints and legal one-step escapes.
New decorative trench/pressure geometry introduces no new damage system.

Art direction is an industrial sanctuary, meaningful waterways/exploration and an
increasingly imposing foundry. These were desired qualities, not an approved map.
The A/B/C studies remain studies. The current native prototype is reusable technical
work and an asset library. The revised master plan deliberately allows documented
coordinate changes rather than preserving a rejected grid out of convenience.

Full source paths inspected include:
crates/world/src/tower_layout.rs
crates/world/src/tower_run.rs
crates/world/src/tower_arrival.rs
crates/world/src/tower_cinderworks.rs
crates/world/src/tower_scenery.rs
crates/world/src/cinderworks_encounter.rs
crates/world/src/simulation/cinderworks_actions.rs
crates/world/src/simulation/cinderworks_work.rs
crates/world/src/simulation/npc_actions.rs
crates/world/src/simulation/tower_offering_actions.rs
crates/world/src/simulation/npc_death_actions.rs
crates/server/src/tower_map.rs
crates/server/src/tower_scene.rs
crates/server/src/characters.rs
crates/server/src/npc_death_config.rs
crates/cache/src/tower_shell.rs
The preserved Kotlin FloorOneScript/Rules remain historical reference only.
'''
 (P/'SOURCE_AUDIT.txt').write_text(source_audit,encoding='utf-8')
 integration='''IMPLEMENTATION AFTER DESIGN REVIEW — NOT AUTHORIZATION TO EXPAND NOW

1. Review this master plan and the three representative scenes. Resolve the
workshop transition, receiver silhouette, pressure-walk readability and amount of
open combat floor. Do not populate the remaining floor while approval is pending.

2. Turn the approved plan into a native graybox in a NEW opt-in content revision.
The proposed envelope is64x54tiles with1792concept-level connected ground cells;
it is not a rectangular floor fill. The existing constructed128tileallocation can
contain its coordinates, but the current36chunktemplate and contains32..79 logic
do not install it. Produce actual server terrain/collision data and matching native
owned-terrain chunks. The current original-only A/P scene format is a review tool,
not a server map. Do not claim the current decoration placement flag is sufficient.

3. Parameterize and update the coordinate contracts together: functional objects,
existing NPC spawns, Imp/Warden arena origins, coolant/vents, restoration landmarks,
arrival/recovery, return portal and tower_scene emission bounds. Existing activity
and reward rules remain unchanged. Preserve the exact arena sizes and relative
geometry. Validate third-phase modulo3pattern translation explicitly. The proposed
anchor table records every old/new coordinate and current native identity.
Scene fragments have explicit local-to-master translations. They are not live
coordinates or a whole-floor nav proof. Decompose the compound review shells into
properly bounded runtime objects with real footprints and per-object identifiers.

4. Select the functional artwork binding explicitly. Keep the existing functional
body/ID and fit architecture, or bind original custom functional scenery to the
existing operation/configuration. Never hide an old interactive target underneath
unrelated decorative machinery. Current library objects have no gameplay menu ops.
The small counter/altar/tablet in the study are visual envelopes, not implemented
banking/offering/instruction replacements. Water is original static coloured
terrain in these studies; animated/native water material integration remains work.

5. Make a one-time, versioned coordinate migration for accounts saved inside the
old floor. Quiesce affected interactions, move safely to the new hearth and retain
inventory, bank, XP, kit/recovery flags, services, victories, pending reward and
first-clear history. Do not reseed accounts, clear active runs, repay rewards or
modify outside-world positions. Native exit/re-entry, defeat/recovery, save/relogin
and interrupted migrations require independent acceptance.

6. Resolve client dependencies before playability acceptance. The13live clipping
sites need a captured update-order regression; the bank needs authoritative solid
policy agreement. Interface/script work and10tickcatch-up saturation dominate the
measured frame. Investigate there and in world submission/readback. Do not remove
correct input, scripts, picking, tick timing or clipping just to raise benchmark FPS.
The warning about an absent optional Kronos NPC combat catalog remains a separate
startup dependency; this checkpoint did not accept full resource/combat/drop loops.

7. Only after native graybox validation and user architectural approval, expand
room by room. Validate real2tilegateways, low foreground walls, no overhead-pipe
clipping, accessible stations, both arena escape guarantees and material continuity.
Then refine remaining quarry, leak basin and alternative supply courts. Reuse the
successful26assetfamilies rather than initiating NPC/animation research.
'''
 (P/'NEXT_IMPLEMENTATION_STEPS.txt').write_text(integration,encoding='utf-8')
 repro='''CINDERWORKS DESIGN CORRECTION / REVIEW CHECKPOINT

Open index.html. This is one proposed connected floor plan and THREE developed
representative scenes. The previous native prototype stays unchanged. These are
not an approved final level, a full-floor implementation or a playability claim.

All new published imagery is original-only: existing authored machinery/arches,
new scene construction and owned native terrain colours. No protected cache-world
pixels, textures, archives, NPC models, credentials, local accounts or executables
are published. Runtime/cache diagnostics remain private; only safe metadata is here.

tools/tower/design-correction/plan.py builds the source-grounded master plan and
explicit relocation contracts. It must precede detailed scene construction.
studies.py runs in Blender5.2.2LTS and assembles original architecture plus the
preserved art/tower/cinderworks-environment library. It refuses overwriting an
existing study. The editable study.blend contains shell and reused object meshes;
native terrain is authored separately in study.scene/ground.json. The shell's
conversion.json records export component order. reexport_study.py and verify.py
prove exact saved-source native bytes and independent geometry/material decoding.
The first construction revision is preserved privately for before/after review.

Reproduction in the sibling worktree setup:
  python tools/tower/design-correction/plan.py
  blender -b --factory-startup --python-exit-code 1 --python tools/tower/design-correction/studies.py
  python tools/tower/design-correction/render.py
  python tools/tower/design-correction/plan_validation.py
  python tools/tower/design-correction/verify.py
The Blender path in verify.py is explicit; adjust to the local installation.
Python3.13/Pillow and the preserved native codec/mesh utilities are used.
Do not rerun studies.py over the curated studies; use a fresh isolated output tree.

render.py builds a PRIVATE package from original assets and the3review shells,
then uses the existing native cinderworks-review executable. Enhanced GPU/NVIDIA
adapter1 is the verified default on this machine. Classic comparisons use identical
geometry/camera. IDs64800..64802 are review shell objects; models1231000..1231002.
The current live environment guard excludes these IDs: this is not a live overlay.
Original underlay definitions5801..5807 use native stored IDs+1. No cache material,
texture or palette is loaded in the original-only mode. Shadows/AO/AA come from the
existing native backend; no new PBR, dynamic lighting, water simulation or NPC art.

prepare_diagnostics.py derives an isolated metadata-only collision executable and
opt-in timing instrumentation in the separate experimental client worktree.
Build with the preserved offline CARGO_HOME mirror and --features enhanced-renderer:
  cargo build --offline --release --bin cinderworks-collision-review --bin cinderworks-profile-live
Use the existing build/enhanced target directory. Source core behavior is unchanged;
the only viewport additions are timings and a bounded optional log, enabled by
TOWER_CORRECTION_PROFILE. The normal allocator and ordinary native actions remain.
The metadata collision adapter inspects final prepared.scene against an authorized
local cache; it writes safe flags/definition metadata, not cache-world screenshots.
Its manual chunk path omits terrain FLOOR installation: restrict the classification
to the14reported sites, as documented. The128water omissions in that offline adapter
are not new live defects. diagnostics.py checks13lifecycle sites and1bank policy site.

profile_live.py uses only the recorded OWNED isolated server/client. It normally
drains the old server and restarts the SAME preserved prototype map/database with
the timing-only client; it never loads a redesign scene. No account progress is
seeded/rewritten. It waits for actual ready state, then a post-ready frame window,
and compares saved position, inventory, XP and journal. Loopback launch on this host
requires an approved local context. Do not launch against production accounts.
Logs, credentials, accounts, raw BMPs and runtime/cache evidence stay in the sibling
cinderworks-design-correction-private directory, outside Git.

review.py curates this HTML review, editable diagram and source/integration handoff.
Raw native BMPs and duplicate sidecars are retained privately; publish PNGs and
source-safe metadata only. No whole-floor construction follows this checkpoint
until the user has reviewed the architectural direction.
'''
 (P/'README.txt').write_text(repro,encoding='utf-8')
 # Curate, preserving raw images locally. Resolve all move targets before moving.
 raw=(R/'raw-original-renders').resolve();root=(S.parents[2]).resolve();raw.mkdir(exist_ok=True)
 for f in (P/'renders').iterdir():
  if '.bmp'in f.name:
   target=raw/f.name;assert f.resolve().is_relative_to(root)and target.is_relative_to(root)
   if target.exists():assert target.read_bytes()==f.read_bytes();f.unlink();continue
   shutil.move(str(f),str(target))
 style='''body{margin:0;background:#121d21;color:#dce1d8;font:17px/1.6 system-ui}main{max-width:1450px;margin:auto;padding:30px}h1,h2,h3{color:#dfbc82;line-height:1.2}h1{font-size:42px}a{color:#87d0c4}nav{background:#233438;position:sticky;top:0;z-index:2;padding:13px;display:flex;gap:20px;flex-wrap:wrap}section{padding:28px 0;border-bottom:1px solid #425051;scroll-margin-top:90px}.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(440px,1fr));gap:18px}figure{margin:0;background:#243337;padding:10px}img{display:block;width:100%;height:auto}figcaption{padding:10px;color:#bdcbc7}.note{border-left:4px solid #c9985c;background:#2a3534;padding:16px}table{border-collapse:collapse;display:block;overflow-x:auto;width:100%;font-size:15px}td,th{padding:10px;border:1px solid #435354;text-align:left;vertical-align:top}details{padding:12px;background:#203034;margin:8px 0}summary{font-weight:bold;color:#dfc18d;cursor:pointer}dt{font-weight:bold;color:#c5cbb1}dd{margin:0 0 12px}code{background:#25373a}li{margin:8px 0}@media(max-width:650px){main{padding:15px}.grid{grid-template-columns:1fr}h1{font-size:31px}}'''
 sections=[]
 sections.append(('plan','The connected floor',fig('floor-plan-detailed.png','Proposed horseshoe complex around a collapsed drainage cut. Dark symbols show only the three developed fragments.')+'<p>The wider envelope is not a rectangular floor fill: the conceptual ground graph has1,792connected cells. The ore route, wet side loop, Imp retort and receiver approach form different spatial experiences. A short return closes the journey at the familiar hearth.</p>'+fig('route-and-interactions.png','Source-backed prerequisites. Preparation remains any order; no new gameplay gates.')+'<p><a href="floor-plan.json">Coordinate plan and source changes</a> · <a href="route-and-interactions.mmd">Editable route diagram</a> · <a href="concept-topology.json">Concept-only topology audit</a></p>'))
 sections.append(('authority','What is established',table(['Category','Finding','Evidence'],definitions)+'<p><a href="SOURCE_AUDIT.txt">Complete source-grounded audit</a>. The current technical prototype, its original objects and prior concept studies remain preserved. No artistic approval is inferred from them.</p>'))
 scene_text={
 'arrival-workshop':('Arrival → workshop','A compressed landing opens obliquely into a shared work gallery. The rail buffer and braced geology explain the upstream material route; the mill, bench, forge hood and remaining roof carry separate, believable jobs. Low foreground edges retain orbit-camera visibility.','Remaining review points: exact interactive artwork bindings, tighter material transitions and the amount of open sanctuary floor. The small bank/altar/plaque envelopes are not implemented service replacements.'),
 'pressure-walk':('Pressure walk → furnace retort','The dry route rises outside the encounter envelope. The pump connects to an overhead supply; the paired return descends only over the water. Supports, quiet source vent grates and a lowered ash trench explain safe passage and industrial danger. The Imp\'s open floor stays attached to that system.','Remaining review points: pipe tie-ins outside the fragment and foreground occlusion, final water/retaining-edge treatment and native collision clearance. The water is a static original-colour study, not an implemented animated water system.'),
 'warden-receiver':('Ash Warden → great receiver','Canted buttresses and extended flues form the architectural culmination. The receiver crown, banked mouths and low threshold frame the same clear13×11movement region. The four coolant housings reuse pressure-vessel meshes from the successful pump.','Remaining review points: the crown silhouette, restrained floor wear and the balance between monumentality and telegraph readability. No Warden model or new encounter effect was authored.')}
 for name,views in VIEWS.items():
  title,body,remaining=scene_text[name];first=next(iter(views));content='<p>'+body+'</p><p class="note">'+remaining+'</p><div class="grid">'
  for view in views:content+=fig('renders/'+name+'--'+view+'--gpu.png',view+' / actual native Enhanced GPU / static original-only study')
  content+='</div><details><summary>Matched Classic comparison</summary><div class="grid">'+fig('renders/'+name+'--'+first+'--gpu.png','Enhanced GPU — same camera and geometry')+fig('renders/'+name+'--'+first+'--classic.png','Classic — actual CPU renderer')+'</div></details>'
  content+='<p>Fragment-to-master translation: <code>'+esc(plan['scene_fragment_transforms'][name]['local_translation'])+'</code>. <a href="studies/'+name+'/study.blend">Editable Blender composition</a> · <a href="studies/'+name+'/study.scene">Native scene source</a> · <a href="studies/'+name+'/placements.json">Object purposes and placements</a></p>'
  sections.append((name,title,content))
 experience=''
 for q in ROOMS:experience+='<details><summary>'+q['id']+' · '+q['name']+'</summary><dl>'+''.join('<dt>'+label+'</dt><dd>'+esc(q[key])+'</dd>'for key,label in [('why','Why enter'),('first','First sight'),('activity','Gameplay activity'),('learn','Environmental story'),('onward','Onward route'),('landmark','Orientation landmark'),('difference','Contrast with earlier areas')])+'</dl></details>'
 sections.append(('experience','Player experience by area',experience))
 migrations=table(['Logical anchor','Current local X,Z,plane','Proposed local X,Z,plane','Existing hook'],[(q['key'],q['current_local'],q['proposed_local'],q['existing_hook'])for q in plan['anchors']])
 sections.append(('contracts','Feasibility and migration','<p class="note">Relocations are proposed, not applied. Current coordinates are not an engine mandate, but their dependencies must move together. No save data, encounter configuration, interaction binding or live map was changed.</p>'+migrations+'<p>The Warden translation(+25,-16) also preserves the existing modulo3phase alignment. Native acceptance must still exercise actual actor footprints, collision, escape timing and coolant use. A2Dconcept graph cannot establish playability.</p><p><a href="proposed-npc-relocations.json">Existing NPC placement proposals</a> · <a href="NEXT_IMPLEMENTATION_STEPS.txt">Exact source, binding and save-migration checklist</a></p>'))
 timing=table(['Measured item','Result'],[('Post-ready instrumented full-client median',f'{perf["median_frame_period_ms"]:.1f}ms'),('p95 frame period',f'{perf["p95_frame_period_ms"]:.1f}ms'),('Context-menu interval mean',f'{perf["context_menu_mean_ms"]:.1f}ms'),('Remaining interface/script interval mean',f'{perf["remaining_interface_script_mean_ms"]:.1f}ms'),('GPU timestamp sample',f'{perf["renderer"]["gpu_ms"]:.1f}ms'),('Presentation failures',perf['failed_presentations'])])
 sites=table(['Reported site','Server','Native live','Native fresh final objects','Classification'],[(q['local'],q['server'],q['native_live'],q['native_fresh_final_objects'],q['classification'])for q in collision['sites']])
 sections.append(('diagnostics','Performance and collision investigation','<p>The preserved live map remains too slow for a playability claim. Opt-in timing isolates the expensive CPU interval without changing menus, scripts, input, scheduling or the renderer. Repeated ten-tick catch-up steps merit client investigation; dropping scenery detail is not demonstrated to solve this bottleneck.</p>'+timing+'<p>This shared-PC instrumented run is not a controlled A/B benchmark against the earlier214mssample. It proves no improvement or regression attribution. The GPU timestamp is not full-client frame time.</p>'+sites+'<p>Thirteen sites agree with the server when reconstructed fresh from the final native placements, pointing toward runtime update/rebuild lifecycle. The bank disagrees even in a fresh scene because of its decoded solid policy. Exact event replay/reference behavior are required before a safe fix. No global clipping, decoder or parity algorithm was changed.</p><p><a href="performance-investigation.json">Performance source evidence</a> · <a href="collision-investigation.json">Collision evidence and limitations</a></p>'))
 checks=table(['Study','Vertices / triangles','Saved-source native bytes','Independent native decoding'],[(q['study'],str(q['vertices'])+' / '+str(q['triangles']),'exact','exact')for q in validation['original_studies']])
 sections.append(('handoff','Checkpoint and next steps','<p class="note">Review the connected floor plan and these three architectural studies before expanding the remaining floor. No complete replacement map has been populated. This submission does not claim artistic approval, native navigation acceptance or adequate live performance.</p>'+checks+'<p>After architectural review: build an isolated native graybox, update coordinate/binding/migration contracts, resolve the client dependencies, validate actual navigation and interactions, then extend the approved language room by room. NPC artwork, rigging, animation and AI remain excluded.</p><p><a href="NEXT_IMPLEMENTATION_STEPS.txt">Detailed implementation sequence</a> · <a href="README.txt">Reproduction and source safety</a> · <a href="validation.json">Preservation and source verification</a> · <a href="render-report.json">Actual renderer/camera metadata</a></p>'))
 nav='<nav>'+''.join('<a href="#'+key+'">'+label+'</a>'for key,label,_ in sections)+'</nav>'
 text='<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>The Cinderworks — design correction checkpoint</title><style>'+style+'</style>'+nav+'<main><p>ASCENT / DESIGN REVIEW CHECKPOINT</p><h1>The Cinderworks<br>The Broken Water Return</h1><p>One connected industrial adventure, developed through three representative scenes using the preserved native pipeline and original asset library.</p><p class="note">The previous level is a technical prototype. These are original-only static native renders and a proposed floor plan. The live game still uses the preserved prototype.</p>'+''.join('<section id="'+key+'"><h2>'+title+'</h2>'+body+'</section>'for key,title,body in sections)+'</main></html>'
 (P/'index.html').write_text(text,encoding='utf-8')
 missing=[]
 for link in re.findall(r'(?:src|href)="([^"]+)"',text):
  if not link.startswith(('#','http'))and not(P/link).exists():missing.append(link)
 assert not missing,missing
 forbidden=[str(q.relative_to(P))for q in P.rglob('*')if q.is_file()and q.suffix.lower()in ['.bmp','.exe','.dll','.sqlite','.password','.bin']];assert not forbidden,forbidden
 files={str(q.relative_to(P)).replace('\\','/'):hashlib.sha256(q.read_bytes()).hexdigest()for q in sorted(P.rglob('*'))if q.is_file()and q.name!='publication-manifest.json'}
 (P/'publication-manifest.json').write_text(json.dumps({'scope':'Original-only public design study; no cache-world pixels, cache data, executable, credential or account','all_local_links_valid':True,'three_representative_scenes_only':True,'files':files},indent=2)+'\n')
 print('DESIGN_CHECKPOINT_REVIEW',P/'index.html',len(files),'source-safe files, all links valid',flush=True)
if __name__=='__main__':main()
