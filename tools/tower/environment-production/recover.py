"""Recover source identities and prepare the complete floor plan before modeling."""
from pathlib import Path
import json,hashlib,subprocess,datetime,re
from PIL import Image,ImageDraw,ImageFont
T=Path(__file__).resolve().parent;S=T.parents[2];E=S.parent;W=E.parents[1]
R=E/'cinderworks-environment-private';P=S/'art/tower/cinderworks-environment';P.mkdir(parents=True,exist_ok=True);R.mkdir(exist_ok=True)
def git(p,*a):return subprocess.check_output(['git',*a],cwd=p,text=True).strip()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
sources=['crates/world/src/tower_layout.rs','crates/world/src/tower_run.rs','crates/world/src/tower_arrival.rs','crates/world/src/tower_scenery.rs','crates/world/src/tower_cinderworks.rs','crates/world/src/cinderworks_encounter.rs','crates/server/src/tower_map.rs','crates/cache/src/tower_shell.rs']
rooms=[
 dict(id='A',name='Hearth Sanctuary',bounds=[49,49,63,63],center=[56,56],status='Existing arrival and recovery/service court',function='Safe staging, plaque, altar, bank and visible three-service choices',kit=['cut-corner masonry','supported doorframes','hearth','service alcoves']),
 dict(id='B',name='Hammer Court',bounds=[33,49,47,55],center=[40,52],status='Existing forge wing; southern workshop composition proposed',function='Smelting, smithing and coherent rail-to-mill production',kit=['plate mill','anvil bench','stock bins','flue','aligned drive machinery']),
 dict(id='C',name='Slate Extraction',bounds=[33,56,47,63],center=[40,59],status='Existing ore wing; geological presentation proposed',function='Copper and tin collection, optional industrial extraction display',kit=['cut slate wall','braced rock face','ore cart','rails','processing chute']),
 dict(id='D1',name='Reclaimed Grove',bounds=[33,65,47,78],center=[40,71],status='Existing four-tree resource court',function='Surviving vegetation and mossy maintenance loop',kit=['damp masonry','root pockets','broken pipe','weathered supports']),
 dict(id='D2',name='Headgate Waterworks',bounds=[65,49,78,63],center=[69,56],status='Existing quay, blocked water, fishing and range',function='Fishing, cooking and coolant lesson; water feeds industrial return',kit=['pump','pipe network','sluice frame','quay rails','cooking shelter']),
 dict(id='E',name='Furnace Threshold',bounds=[50,64,62,66],center=[56,65],status='Existing approach to preserved Warden bounds; architectural emphasis proposed',function='Safe observation apron and deliberate furnace threshold',kit=['paired piers','pressure manifold','heat markers','ventilation frame']),
 dict(id='F1',name='Ash Provision Court',bounds=[65,33,78,47],center=[71,40],status='Existing resource combat yard',function='Early combat supplies and bone collection',kit=['ruined stock shed','ash deposits','feeding infrastructure']),
 dict(id='F2',name='Western Bone Court',bounds=[33,33,47,47],center=[40,40],status='Existing resource combat yard',function='Optional resource combat and route variation',kit=['low rubble boundary','stock bays','damaged hoist']),
 dict(id='F3',name='Imp Vent Gallery',bounds=[65,65,78,78],center=[72,72],status='Existing Imp arena and maintenance perimeter',function='Imp encounter after all three services; arena [68,68]–[75,75] unchanged',kit=['boiler banks','vent collars','ash collection','maintenance rails']),
 dict(id='G',name='Ash Warden Foundry',bounds=[50,67,62,77],center=[56,72],status='Existing exact combat bounds; monumental backdrop proposed',function='Three-phase Warden arena, four existing coolant barrels and open movement',kit=['three furnace mouths','segmented arches','buttresses','quench headers']),
 dict(id='H1',name='Reward Alcove',bounds=[53,50,58,53],center=[55,51],status='Existing sanctuary reward site; overlaps A intentionally',function='Existing first/replay reward collection, no rule changes',kit=['original chest surround','completion masonry']),
 dict(id='H2',name='Departure Threshold',bounds=[63,58,66,62],center=[64,60],status='Existing portal site; overlaps D2 threshold intentionally',function='Existing exit and return; no new departure behavior',kit=['framed departure arch','wayfinding floor bonds']),
 dict(id='S',name='Processional Service Hall',bounds=[51,33,61,47],center=[56,40],status='Existing southern corridor',function='Optional orientation and connections to both supply courts',kit=['load-bearing arcades','low foreground edges','utility conduit'])]
connections=[('A','B',[48,55]),('B','C',[40,56]),('C','D1',[40,64]),('A','D2',[64,55]),('A','E',[56,64]),('E','G',[56,67]),('D2','F3',[70,64]),('G','F3',[64,72]),('D1','G',[48,72]),('A','H1',[55,52]),('A','H2',[63,60]),('A','S',[56,48]),('S','F1',[62,40]),('S','F2',[50,40]),('B','F2',[40,48]),('D2','F1',[70,48])]
anchors=[
 ('arrival-first',56,56,'tower_arrival::State::arrival','implemented'),('arrival-return',56,49,'tower_arrival::State::arrival','implemented'),
 ('bank',53,60,'tower_map::prepare / sanctuary Bank','implemented'),('plaque',54,51,'tower_run / cinderworks_actions','implemented'),('reward',55,51,'tower_run reward_pending / chest action','implemented'),('altar',60,52,'Service::Altar','implemented'),
 ('furnace',34,50,'Service::Forge / smelting','implemented'),('anvil',37,50,'Service::Forge / bronze dagger','implemented'),
 ('tin-south',34,54,'existing mining','implemented'),('tin-north',34,58,'existing mining','implemented'),('copper-south',44,54,'existing mining','implemented'),('copper-north',44,58,'existing mining','implemented'),
 ('cooking-range',67,53,'Service::Kitchen','implemented'),('coolant-lesson',69,52,'TRAINING_COOLANT','implemented'),('portal',64,60,'cinderworks exit/return','implemented'),
 ('imp-spawn',72,72,'Boss::Imp [68,68]–[75,75]','implemented'),('warden-spawn',56,72,'Boss::Warden [50,67]–[62,77]','implemented')]
for i,(x,z)in enumerate([(53,71),(53,74),(60,71),(60,74)]):anchors.append((f'coolant-{i}',x,z,'COOLANT_TILES / Warden shell interruption','implemented'))
for i,(x,z)in enumerate([(34,70),(42,70),(34,76),(42,76)]):anchors.append((f'tree-{i}',x,z,'existing woodcutting; no new objective invented','existing'))
plan={'created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'origin':[10368,64,0],'local_extent':[32,32,79,79],'dimensions_tiles':[48,48],
 'coordinate_policy':'No gameplay coordinate migration. Current source anchors, first/return arrival, portals, encounter bounds, service state and persistence retained. Recommended A–H areas reconcile to existing wings; D and F have multiple subareas; reward/departure remain attached to the sanctuary.',
 'rooms':rooms,'connections':[dict(a=a,b=b,threshold=t)for a,b,t in connections],
 'interaction_anchors':[dict(key=k,local=[x,z,0],world=[10368+x,64+z,0],existing_hook=hook,status=status,orientation='preserve actual source',interaction_position='nearest reachable adjacent tile; verified in cache-backed audit')for k,x,z,hook,status in anchors],
 'mandatory':['Forge a bronze dagger in the western workshop','Cook shrimp at the eastern range','Offer bones at the sanctuary altar','Defeat Imp after all services','Quench/defeat Warden after Imp','Collect existing reward; use existing portal'],
 'optional':['Slate extraction dressing and processing inspection','Grove exploration and woodcutting','Both southern resource combat yards','Maintenance loops around Imp and Warden','Additional fishing/cooking supplies'],
 'traversal':['A','B','C','D1','A','D2','F1','S','F2','A','F3','G','H1','H2'],
 'traversal_note':'One review route covering all areas, not a new enforced objective order. The three preparation services remain available in any order.',
 'collision_plan':['Use authoritative server shell and real cache clipping as baseline.','Keep every arena movement/hazard tile and every required interaction approach free of new blockers.','Stamp custom collision through ordinary LocationType footprints, with compound supports only where needed.','Validate reachability against actual prepared collision after placement, not just a drawn plan.','Keep foreground walls low and preserve all existing thresholds.','Water retains existing blocked terrain and real crossings; no painted walkable water.'],
 'prop_plan':{r['id']:dict(families=r['kit'],policy='Group by physical production flow, keep clear route through center, preserve all functional anchors')for r in rooms},
 'asset_dependencies':{'reuse_original':['improved anvil','plate mill','clamping workstation','mechanical pump','segmented archway'],
 'new_original':['modular bonded wall','piers and capitals','doorframe and gate shell','drain and flanged pipe kit','furnace bank','slate headwall','ore cart and rails','waterworks railing','reward chest surround','service signs','rubble and moss variants'],
 'protected_runtime_only':['existing cache terrain','existing functional objects and authorized NPC placeholders']},
 'new_proposals':['Industrial production flow from slate working face through rail stock to mill and furnace.','Connected original flues and water supply/return infrastructure.','Distinct room silhouettes and coherent architectural kit layered around fixed gameplay positions.'],
 'source_recovery':{'current_rule_source':'tower_run.rs (three services plus guardians/reward)','historical_reference':'FloorOneRules.kt eleven-row diary; historical, not substituted for current Rust lifecycle','source_files':{p:sha(S/p)for p in sources}}}
(P/'planning').mkdir(exist_ok=True)
for name,data in [('floor-plan.json',plan),('room-registry.json',rooms),('interaction-contracts.json',plan['interaction_anchors']),('asset-dependencies.json',plan['asset_dependencies'])]:(P/'planning'/name).write_text(json.dumps(data,indent=2)+'\n')
baseline={'created_utc':plan['created_utc'],'request_sha256':sha(Path('C:/Users/Groot/.codex/attachments/95a6ea03-bce1-42a5-8dda-041cbfe8754c/Pasted text.txt')),
 'repos':{n:dict(head=git(E/n,'rev-parse','HEAD'),branch=git(E/n,'branch','--show-current'),status=git(E/n,'status','--porcelain'))for n in ['cinderworks-environment-server','cinderworks-environment-client','enhanced-client','ashhorn-reference-server']},
 'protected_workspaces':{n:dict(head=git(W/n,'rev-parse','HEAD'),status=git(W/n,'status','--porcelain'))for n in ['rust-server','rust-client-reference']},
 'source_files':plan['source_recovery']['source_files']}
if not(R/'baseline.json').exists():(R/'baseline.json').write_text(json.dumps(baseline,indent=2)+'\n')
# Static top-down coordinates and circulation diagram, authored before geometry.
size=1220;im=Image.new('RGB',(size,1450),'#111b20');d=ImageDraw.Draw(im);font=ImageFont.truetype('C:/Windows/Fonts/arial.ttf',18);small=ImageFont.truetype('C:/Windows/Fonts/arial.ttf',13);title=ImageFont.truetype('C:/Windows/Fonts/georgia.ttf',40)
d.text((40,18),'The Cinderworks — complete floor plan',font=title,fill='#e3c18c');d.text((40,72),'Existing48×48-tile floor · fixed gameplay coordinates · proposed original environment dressing',font=font,fill='#bdc7c7')
def xy(x,z):return(74+(x-32)*22,117+(80-z)*22)
colors={'A':'#59635b','B':'#775139','C':'#474d5e','D1':'#3c5d45','D2':'#365961','E':'#805333','F1':'#66554a','F2':'#66554a','F3':'#6c4840','G':'#804d36','S':'#535b60','H1':'#aa8148','H2':'#aa8148'}
for r in rooms:
    a,b,c,f=r['bounds'];d.rectangle([xy(a,f+1),xy(c+1,b)],fill=colors[r['id']],outline='#c2b497',width=2)
    cx,cz=r['center'];px,py=xy(cx,cz);d.text((px-40,py-15),r['id'],font=font,fill='white');d.text((px-68,py+10),r['name'],font=small,fill='#f2eeee')
for a,b,t in connections:
    x,y=xy(*t);d.ellipse([x-6,y-6,x+6,y+6],fill='#e4d19e',outline='#172228')
for key,x,z,hook,status in anchors:
    px,py=xy(x+.5,z+.5);d.ellipse([px-3,py-3,px+3,py+3],fill='#72e3cc')
for x in range(32,81,8):
    px,py=xy(x,32);d.text((px-10,py+8),str(x),font=small,fill='#a7b6bc')
for z in range(32,81,8):
    px,py=xy(32,z);d.text((35,py-7),str(z),font=small,fill='#a7b6bc')
d.text((60,1230),'N ↑   Teal: preserved gameplay anchor · Gold: existing threshold',font=font,fill='#c7d5d5')
d.text((60,1270),'Preparation in any order → Imp → Warden → reward → existing departure portal',font=font,fill='#e3c18c')
d.text((60,1310),'G: entire [50,67]–[62,77] arena kept open; F3: [68,68]–[75,75] encounter preserved.',font=font,fill='#c7d5d5')
d.text((60,1350),'H1/H2 are existing sanctuary-adjacent sites; D1/D2 retain the two established wet/reclaimed wings.',font=font,fill='#c7d5d5')
im.save(P/'planning/floor-plan.png')
mermaid='flowchart TD\n'+''.join(f'  {r["id"]}["{r["id"]} · {r["name"]}"]\n'for r in rooms)+''.join(f'  {a} --- {b}\n'for a,b,t in connections)
(P/'planning/connectivity.mmd').write_text(mermaid)
(P/'planning/RECOVERED_DESIGN.md').write_text('# Recovered Cinderworks environment contract\n\nCurrent Rust source, not the superseded eleven-row Kotlin diary, defines the active journey: restore Forge, Kitchen and Altar in any order; defeat the Imp; quench and defeat the Warden; claim the existing first/replay reward; use the existing portal. The personal `tower_run` state, recovery and persistence are retained.\n\nThe existing floor occupies local tiles32–79 on both axes, origin(10368,64,0). This assignment preserves all gameplay anchors. The recommended A–H structure is reconciled to the existing wings rather than migrating persistent coordinates. Reward and departure sites intentionally remain attached to the sanctuary; moss and reclaimed water occupy their two established wings.\n\nNew proposals are architecture, machinery composition, geology, connected industrial utilities, low foreground walls and local scenery. No new reward, hazard damage, combat controller, NPC model or animation is introduced. Existing cache terrain and functional entities remain protected runtime dependencies.\n\nPlanning JSON files record the room registry, connectivity, full traversal, mandatory/optional activities, props, walkability and asset dependencies before detailed modeling. Cache-backed collision and native traversal remain required acceptance work.\n',encoding='utf8')
print('PLANNING_WRITTEN',len(rooms),'subareas;',len(anchors),'preserved anchors; no detailed geometry generated')
