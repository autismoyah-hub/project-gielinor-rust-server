"""Source-grounded route/industrial-process proposal, before scene construction."""
from pathlib import Path
import json,hashlib,subprocess,math
from PIL import Image,ImageDraw,ImageFont
T=Path(__file__).resolve().parent;S=T.parents[2];E=S.parent;W=E.parents[1]
P=S/'art/tower/cinderworks-design-correction';R=E/'cinderworks-design-correction-private'
def save(p,v):p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
ROOMS=[
 dict(id='A',name='Hearth Landing',polygon=[(48,50),(52,48),(61,49),(65,53),(63,60),(59,64),(51,63),(47,59),(47,54)],label=(56,57),color='#666a63',why='Arrival, recovery, tools and orientation',first='A worn hearth within a cut-corner masonry apse; the workshop drive is visible through a low western opening',activity='Plaque, bank, altar; three services remain available in any order',learn='This was a workers\' refuge between the wet and hot works',onward='West to forge/ore, east to quay, south to supply courts; the twin foundry stacks remain north',landmark='Hearth and twin stacks',difference='Small protected human-scale landing, not a central empty square'),
 dict(id='B',name='Hammer Gallery',polygon=[(33,48),(40,47),(48,51),(49,57),(44,60),(36,60),(32,56)],label=(40,52),color='#5d5b51',why='Restore Forge and prepare metal',first='A diagonal approach reveals the mill flywheel and its working aisle before the furnace mouth',activity='Existing furnace34,50/anvil37,50; ore processing display has no invented operation',learn='Slate and ore arrived on a short tramway; the press and benches share a protected maintenance gallery',onward='Follow the tram upstream into the cut, or return through the oblique arch to the hearth',landmark='Mill flywheel, vented forge hood',difference='Compressed stone-and-timber work bay, open to the excavation rather than boxed apart'),
 dict(id='C',name='The Slate Cut',polygon=[(32,55),(37,56),(43,54),(47,57),(47,62),(42,65),(34,63),(31,60)],label=(38,61),color='#414b50',why='Obtain copper/tin and explore extraction traces',first='Broken strata and a braced working face rise behind the rail buffer',activity='All four existing copper/tin sites retained, with approaches protected',learn='The facility grew around the rock seam; spoil and a failed retaining wall explain the dogleg',onward='Rail leads back to the mill; drainage and a collapsed retaining arch lead into the moss basin',landmark='Braced cut and rail buffer',difference='Geological edges interrupt masonry rhythm; no mining-room rectangle'),
 dict(id='D1',name='Leakwater Basin',polygon=[(31,65),(38,63),(46,65),(48,70),(45,79),(35,79),(31,75)],label=(38,72),color='#3e5c4c',why='Optional gathering/exploration and a cool relief from industry',first='Drips beneath the failed return culvert, not vegetation spread uniformly over a green tile square',activity='Existing four tree sites; optional side loop and maintenance overlook',learn='A failed water return fed soil pockets and split the western retaining wall',onward='A dry raised edge returns to the cut; an overlook foreshadows the foundry without bypassing combat unlocks',landmark='Broken culvert and a tree rooted in the wall',difference='Vegetation follows water and shade; dry ground remains slate/ash'),
 dict(id='D2',name='Headgate Quay',polygon=[(64,49),(70,48),(78,51),(79,59),(76,64),(69,66),(63,62)],label=(69,58),color='#446166',why='Restore Kitchen, fish and learn coolant',first='An inset working quay and pump intake; the channel continues into the plant perimeter',activity='Four existing water NPCs, range67,53 and training barrel69,52; source fishing reach is3tiles',learn='Water drives and cools the works; the pump supply physically leads toward the furnaces',onward='Dry walk north to the pressure gallery; return south to optional stores or west to the hearth',landmark='Pump accumulator and sluice wheel',difference='A working inlet with a shaped bank, not a rectangular swimming pool'),
 dict(id='E',name='Pressure Walk',polygon=[(54,62),(61,62),(66,66),(67,73),(63,76),(61,68),(53,67),(51,65)],label=(63,65),color='#7a6650',why='Read danger and approach the late encounters',first='A kinked low gallery passes beneath an overhead coolant main; furnace light is framed ahead',activity='Existing vent/coolant teaching and encounter approach; no new damage or gate rules',learn='Cool return and hot exhaust are different systems, marked by material and direction',onward='East to Imp then back along the manifold to the Warden threshold',landmark='Raised pressure main, exhaust buttress',difference='Narrow safe ledge beside a lowered non-walkable heat trench; no decorative obstacle in combat bounds'),
 dict(id='F1',name='Provision Yard',polygon=[(65,33),(74,32),(79,36),(78,45),(72,48),(65,46),(63,40)],label=(72,40),color='#6d634a',why='Optional early food/combat and supply exploration',first='A damaged loading wall and abandoned covered store',activity='Existing chicken spawns retained; no new NPCs or drops',learn='This was the workforce\'s provision court, not part of the molten process',onward='Broken cart path joins the quay and landing',landmark='Collapsed loading frame',difference='Dry outdoor earth pockets, broken perimeter and inward-looking shelter'),
 dict(id='F2',name='Old Stock Court',polygon=[(32,33),(43,32),(47,36),(46,44),(42,48),(34,46),(31,40)],label=(39,40),color='#626250',why='Bones/resources and optional early combat',first='Low ruined enclosure beneath the workshop wall',activity='Existing cattle/resource positions retained; bones support the existing altar service',learn='Service yards were added around the earlier stone works',onward='Short path reaches the forge and altar; no enforced tour through both courts',landmark='Broken hoist foundation',difference='Low sightlines and usable open ground instead of a decorated combat box'),
 dict(id='F3',name='Vent Retort',polygon=[(66,65),(72,64),(78,67),(79,74),(76,78),(67,78),(65,74)],label=(72,72),color='#735246',why='Defeat the Imp after the three services',first='A furnace sidewall and exhaust array surround an uncluttered dark floor',activity='Imp spawn72,72, arena68..75 unchanged; four existing quiet steam vents retained',learn='The minor guardian occupies the exhaust system feeding the main foundry',onward='A dry manifold route returns to the monumental front threshold',landmark='Retort flue at the east edge',difference='A compact attached plant bay, not a second isolated boss showroom'),
 dict(id='G',name='The Great Receiver',polygon=[(46,68),(49,64),(62,64),(66,68),(66,77),(63,81),(49,81),(46,77)],label=(56,74),color='#60534c',why='Face the Warden after the Imp',first='The paired furnace stacks and a curved load-bearing rear receiver frame the arena across a low threshold',activity='Warden56,72; exact50..62,67..77 movement region and all four coolant sites retained',learn='Every rail, coolant line and flue belongs to the facility\'s central production heart',onward='Return down the now-familiar threshold to existing reward55,51 and portal64,60',landmark='Twin flues, central receiver and cool barrel insets',difference='Monumental outer silhouette wraps a clear gameplay rectangle; no intrusive arena dressing'),
 dict(id='S',name='Broken Approach',polygon=[(52,33),(58,32),(61,38),(64,41),(61,46),(58,50),(53,49),(50,44),(51,39)],label=(56,42),color='#655b4c',why='Optional supply circuit and the repeat-arrival approach',first='A broken retaining wall frames the sanctuary arch, with the foundry stacks beyond',activity='Repeat-arrival56,49 remains accessible; early courts stay optional',learn='The once-straight haul road has been diverted around a collapsed footing',onward='Clear two-tile dry connections to both resource yards and the landing',landmark='Missing span and split haul road',difference='A repaired dogleg route; no long axial rectangle')]
ROUTES={
 'prepare-west':[(56,56),(52,56),(48,55),(44,54),(40,52),(35,50)],
 'ore-and-leak':[(40,53),(40,60),(43,64),(43,70),(39,74),(34,76)],
 'prepare-east':[(56,56),(61,56),(65,55),(69,55),(70,60)],
 'optional-supplies':[(56,49),(55,45),(54,41),(49,42),(43,42)],
 'optional-stores':[(56,46),(61,43),(66,43),(72,42)],
 'imp-approach':[(59,59),(62,61),(65,63),(67,65),(70,67),(72,70)],
 'warden-approach':[(72,70),(69,67),(66,65),(62,64),(58,64),(56,66),(56,69)],
 'departure-return':[(56,68),(56,63),(54,60),(54,55),(55,52),(60,55),(64,60)]}
SYSTEMS={
 'ore':[(40,62),(40,59),(40,55),(40,51)],
 'coolant':[(74,59),(78,61),(79,65),(79,77),(76,80),(65,81),(60,79),(56,78)],
 'leak':[(53,79),(48,79),(46,76),(45,71),(43,68),(39,67)]}
# The prototype's coordinates are not an artistic mandate. Rigid scene-fragment
# transforms and explicit functional relocations describe a future configuration;
# they do not modify tower_layout, accounts, spawns, encounter code or the live map.
MASTER={
 'A':([(47,40),(51,38),(57,39),(61,43),(60,49),(56,53),(50,52),(46,48),(46,43)],(53,46)),
 'B':([(29,39),(35,37),(42,40),(46,44),(45,49),(40,53),(33,52),(29,48)],(36,45)),
 'C':([(30,51),(35,50),(42,53),(44,58),(40,64),(34,67),(29,63),(28,57)],(35,58)),
 'D1':([(34,67),(41,64),(47,67),(51,72),(50,79),(45,82),(38,80),(34,75)],(43,75)),
 'D2':([(56,56),(60,53),(66,55),(71,59),(72,65),(68,69),(62,68),(58,64)],(63,62)),
 'E':([(59,63),(62,61),(66,63),(65,66),(67,69),(64,71),(60,68)],(62,65)),
 'F1':([(64,32),(71,31),(76,35),(75,42),(71,46),(65,44),(62,39)],(69,38)),
 'F2':([(31,31),(38,30),(43,34),(42,39),(38,42),(32,40),(29,36)],(36,35)),
 'F3':([(63,66),(67,64),(73,65),(76,69),(75,76),(71,80),(65,78),(62,73)],(69,72)),
 'G':([(72,50),(76,47),(85,47),(90,51),(91,58),(88,64),(83,66),(76,65),(71,60)],(81,56)),
 'S':([(49,31),(54,30),(58,34),(57,38),(54,41),(50,40),(47,36)],(52,35))}
RELOCATE={'arrival-first':(52,46),'arrival-return':(52,39),'bank':(49,50),'plaque':(50,41),'reward':(51,41),'altar':(56,42),'furnace':(30,40),'anvil':(33,42),'tin-south':(31,57),'tin-north':(32,62),'copper-south':(38,57),'copper-north':(40,61),'cooking-range':(61,59),'coolant-lesson':(62,60),'portal':(60,50),'imp-spawn':(68,71),'warden-spawn':(81,56),'coolant-0':(78,55),'coolant-1':(78,58),'coolant-2':(85,55),'coolant-3':(85,58),'tree-0':(40,71),'tree-1':(48,71),'tree-2':(40,77),'tree-3':(48,77)}
ROUTES={
 'prepare-west':[(52,46),(47,46),(44,45),(39,44),(35,43),(31,41)],
 'ore-and-leak':[(36,48),(35,52),(33,56),(35,61),(39,65),(40,69),(44,74)],
 'prepare-east':[(54,46),(58,49),(60,54),(61,59),(64,62)],
 'optional-supplies':[(52,39),(48,37),(44,37),(39,36),(36,35)],
 'optional-stores':[(57,40),(61,38),(65,37),(70,38)],
 'wet-exploration-loop':[(43,74),(48,74),(53,72),(57,69),(61,66),(64,63)],
 'imp-approach':[(64,63),(64,65),(66,66),(68,68),(68,70)],
 'warden-approach':[(68,70),(72,67),(74,64),(72,60),(72,54),(76,48),(80,48),(81,52)],
 'departure-return':[(81,49),(78,45),(72,44),(66,45),(61,48),(58,49),(54,48),(51,42),(60,50)]}
SYSTEMS={'ore':[(34,63),(34,58),(35,53),(37,49),(37,46)],'coolant':[(67,60),(65,62),(63,65),(66,68),(68,78),(74,79),(79,74),(82,65),(81,62)],'leak':[(79,64),(74,67),(64,74),(54,78),(47,77),(44,73)]}
for q in ROOMS:
 q['polygon'],q['label']=MASTER[q['id']]
 if q['id']=='A':q.update(first='The close hearth court frames the western mill and a northeast glimpse of the receiver stacks across the collapsed drainage cut',onward='West to metalwork and the stock court; northeast to the quay; the foundry return makes a short completion loop',activity='Plaque, bank, altar and reward remain together; their proposed positions are explicit in the migration table')
 if q['id']=='B':q['activity']='Existing Forge service at proposed30,40 and anvil33,42; press/bench remain a non-interactive production display until binding review'
 if q['id']=='C':q['activity']='The same four copper/tin resource types, explicitly relocated along a broken seam; no new resource or mining rule'
 if q['id']=='D2':q['activity']='Four existing fishing NPCs using the unchanged3tileapproach; proposed range61,59 and training coolant62,60'
 if q['id']=='F2':q['activity']='Existing cattle/bone opportunities. A fresh self-supplied run needs bones for Altar; choose this court or the provision yard, without requiring both'
 if q['id']=='F3':q.update(activity='Imp at proposed68,71 with the same8x8arena and translated quiet vent anchors',onward='Follow the manifold around the drainage cut to the receiver\'s south threshold')
 if q['id']=='G':q.update(activity='Warden at proposed81,56;13x11arena75..87/51..61 and four relative coolant positions preserved',onward='The short dry completion return reaches reward51,41 and portal60,50 without retracing the whole wet loop')
def main():
 P.mkdir(parents=True,exist_ok=True);R.mkdir(exist_ok=True)
 anchors=json.loads((S/'art/tower/cinderworks-environment/planning/interaction-contracts.json').read_text())
 migrations=[]
 for q in anchors:
  x,z=RELOCATE[q['key']];migrations.append({'key':q['key'],'current_local':q['local'],'proposed_local':[x,z,0],'current_world':q['world'],'proposed_world':[10368+x,64+z,0],'current_native_definitions':q['object_definitions'],'existing_hook':q['existing_hook'],'proposed_identity_policy':'Retain logical hook; native ID/body binding remains an explicit integration decision, never a hidden duplicate target','status':'PROPOSED ONLY. No save/coordinate/source migration has run.'})
 plan={'status':'DESIGN REVIEW CHECKPOINT ONLY; not a live map replacement','actual_prototype_extent':'Audited48x48tiles,local32..79; rejected room-grid composition.','proposed_envelope_local':[28,30,91,83],'origin':[10368,64,0],'rooms':ROOMS,'routes':ROUTES,'industrial_systems':SYSTEMS,'anchor_policy':'Explicit relocations permit a fundamentally different layout. Preserve mechanics and arena-relative dimensions; do not mistake current coordinates for engine limits. Implement only after review, with coordinate configuration, persistence relocation and native validation.','gameplay_order':'Forge/Kitchen/Altar in any order, then Imp, then Warden, then existing reward/exit. Illustrated exploration order is not a new quest gate.','elevations':'Keep interaction/combat plane0 and core floor nativeY-240. Quay water0; a proposed64unit maintenance rise remains outside encounter bounds. Overhead main minimum320units above lanes. Trench/central drainage cut nonwalkable. No new stacked-plane bridge.','anchors':migrations,'scene_fragment_transforms':{'arrival-workshop':{'local_translation':[-4,-10],'note':'Reused forge hood/workbench geometry represents the proposed visual envelope; exact functional target alignment remains an integration task.'},'pressure-walk':{'local_translation':[-4,-1],'imp_bounds_proposed':[64,67,71,74]},'warden-receiver':{'local_translation':[25,-16],'warden_bounds_proposed':[75,51,87,61]}},'warden_phase3_translation_note':'Translation(+25,-16) sums to9, preserving the existing modulo3diagonal phase relative to the arena. Future parameterization must retain that pattern equivalence, cooldowns and escape guarantees.','binding_policy':'Study envelopes are not hidden duplicate live targets. After approval either retain functional body/ID and fit architecture, or bind original custom functional artwork through the existing service configuration with explicit operations and collision; do not conceal a cache target inside decorative machinery.','source_changes_required_after_review':['tower_layout terrain/chunk template and contains bounds','tower_map spawn/functional placement coordinate configuration','tower_arrival first/recovery destinations and portal return destination','cinderworks_encounter arena translations; preserve sizes, phases and timing','tower_scenery restoration pairs/vents/coolant translations','tower_scene region/state emission bounds','safe account relocation for saved prototype coordinates, preserving run/inventory/bank/xp/reward state','server terrain/collision generation matching native authored terrain; existing opt-in placement flag alone cannot install this redesign']}
 save(P/'floor-plan.json',plan)
 save(P/'proposed-npc-relocations.json',{'scope':'Placement proposal only; existing types, behavior and counts. No new NPC artwork, rigging, animation or AI. Actual footprints, roaming and clipping still require native tests.','fishing':[{'npc_id':3317,'current_local':[72,z,0],'proposed_local':[x1,z1,0],'existing_reach':3}for z,(x1,z1)in zip([50,52,54,56],[(69,58),(70,60),(70,62),(69,65)])],'cow_spawn_proposal':[(33,33),(36,32),(39,34),(40,37),(37,39),(33,38),(31,35),(36,36)],'chicken_spawn_proposal':[(65,34),(72,34),(66,41),(72,42)],'banker_proposed':[48,50,0],'tanner_proposed':[57,48,0],'Imp_proposed':[68,71,0],'Warden_proposed':[81,56,0]})
 baseline={'server_base':subprocess.check_output(['git','rev-parse','HEAD'],cwd=S,text=True).strip(),'client_base':'29d665a9e41995632d942cea27e2f6782eca561d','preserved_prototype_server':'75e8ce364013756754873cc037b18d6a85c8a266','request_sha256':hashlib.sha256(Path('C:/Users/Groot/.codex/attachments/b8992613-00b9-4c19-8599-21eaf5aa629b/Pasted text.txt').read_bytes()).hexdigest()}
 if not(R/'baseline.json').exists():save(R/'baseline.json',baseline)
 size=(2400,1900);im=Image.new('RGB',size,'#172127');d=ImageDraw.Draw(im);font=lambda n:ImageFont.truetype('C:/Windows/Fonts/arial.ttf',n);bold=lambda n:ImageFont.truetype('C:/Windows/Fonts/arialbd.ttf',n)
 scale=27;xy=lambda p:(100+(p[0]-28)*scale,1670-(p[1]-29)*scale)
 for x in range(28,93):d.line([xy((x,29)),xy((x,83))],fill='#243239')
 for z in range(30,84):d.line([xy((28,z)),xy((92,z))],fill='#243239')
 for points in ROUTES.values():d.line([xy(v)for v in points],fill='#54594f',width=int(scale*2.4),joint='curve')
 for q in ROOMS:d.polygon([xy(p)for p in q['polygon']],fill=q['color'],outline='#a49a82',width=3)
 # Water occupies the inset channel; vegetation is concentrated near the leak.
 water=[(68,56),(71,59),(71,64),(69,67),(67,65),(67,60)]
 d.polygon([xy(p)for p in water],fill='#244a59',outline='#69a2ac',width=4)
 d.ellipse([xy((39,77)),xy((46,72))],fill='#284a4b',outline='#6b8a6b',width=3)
 d.polygon([xy(p)for p in [(45,55),(48,54),(54,57),(55,63),(51,67),(46,64),(44,59)]],fill='#202d32',outline='#718078',width=3)
 d.text(xy((49,60)),'Collapsed\ndrainage cut',font=font(17),fill='#adbfba',anchor='mm',align='center')
 def line(points,color,width,dashes=False):
  v=[xy(p)for p in points]
  for a,b in zip(v,v[1:]):
   if not dashes:d.line([a,b],fill=color,width=width)
   else:
    length=math.dist(a,b)
    for i in range(0,int(length),16):
     t=i/length;u=min(1,(i+9)/length);d.line([(a[0]+(b[0]-a[0])*t,a[1]+(b[1]-a[1])*t),(a[0]+(b[0]-a[0])*u,a[1]+(b[1]-a[1])*u)],fill=color,width=width)
  a,b=v[-2:];angle=math.atan2(b[1]-a[1],b[0]-a[0]);d.polygon([b,(b[0]-15*math.cos(angle-.5),b[1]-15*math.sin(angle-.5)),(b[0]-15*math.cos(angle+.5),b[1]-15*math.sin(angle+.5))],fill=color)
 for k,v in ROUTES.items():line(v,'#f0d18a'if k.startswith('prepare')or k=='ore-and-leak'else'#d88767'if'approach'in k else'#b1c2aa',5,True)
 for k,v in SYSTEMS.items():line(v,{'ore':'#b68b5a','coolant':'#65c8d0','leak':'#7fa884'}[k],6)
 for rect in [(75,51,87,61),(64,67,71,74)]:
  a,b,c,e=rect;d.rectangle([xy((a,e)),xy((c,b))],outline='#e5ac93',width=3)
 for q in migrations:
  x,z,_=q['proposed_local'];a,b=xy((x,z));d.ellipse((a-4,b-4,a+4,b+4),fill='#fff6d8')
 for q in ROOMS:
  x,z=q['label'];a,b=xy((x,z));d.text((a,b),q['id']+'  '+q['name'],font=bold(19),fill='#fff0d0',anchor='mm',stroke_width=2,stroke_fill='#1d2426')
 d.text((100,45),'THE CINDERWORKS / THE BROKEN WATER RETURN',font=bold(37),fill='#e1bf86');d.text((100,96),'One connected industrial place — design checkpoint, not an approved or playable replacement',font=font(23),fill='#b9c9ca')
 d.text((1940,180),'READ THE PLAN',font=bold(23),fill='#e1bf86')
 notes=['A  Hearth / any-order preparation','B  Workshop / ore-to-metal line','C  Slate cut / damaged retaining wall','D1 Leak basin / optional gathering','D2 Quay / fish, cook, coolant','E  Pressure walk / safe danger edge','F1/F2 Resource and supply courts','F3 Imp / attached exhaust retort','G  Warden / great receiver','H  Reward and departure stay at A','','Gold = service exploration','Rust = encounter approach','Pale green = optional/return','Brown = material transport','Cyan = cooling supply','Green = failed return / moisture','','Peach rectangles retain the','existing combat dimensions;','arena origins are proposed.','','White dots = PROPOSED anchors.','Every relocation is listed;','the live map is unchanged.','','NORTH ↑  /  one grid = one tile']
 for i,t in enumerate(notes):d.text((1930,225+i*37),t,font=font(18),fill='#d0d5cd')
 d.text((100,1785),'Core elevations retained. Pressure ledge +64units is proposed outside combat; overhead supply follows the perimeter.',font=font(23),fill='#bcc9c8');d.text((100,1827),'Explore and prepare in any order → Imp → Warden → return to the familiar hearth, reward and existing exit.',font=font(23),fill='#e0bd85')
 im.save(P/'floor-plan.png')
 # Clear interaction diagram, spatial plan is separate from logical gates.
 graph='''flowchart LR
  Arrival[Arrival / recovery] --> Hearth[Hearth and tools]
  Hearth --> Forge[Slate → smelt → forge]
  Hearth --> Kitchen[Quay → fish → cook]
  Hearth --> Altar[Stock court → bones → altar]
  Forge --> Ready{All three services}
  Kitchen --> Ready
  Altar --> Ready
  Ready --> Imp[Vent retort / Ash Imp]
  Imp --> Warden[Pressure threshold / Ash Warden]
  Warden --> Reward[Return to hearth reward]
  Reward --> Exit[Existing portal / departure]
  Forge -. optional .-> Grove[Leakwater / trees and overlook]
  Kitchen -. optional .-> Stores[Provision court]
'''
 (P/'route-and-interactions.mmd').write_text(graph,encoding='utf-8')
 print('DESIGN_PLAN',P/'floor-plan.png',len(ROOMS),'named areas; mechanics retained, explicit anchor relocations proposed')
if __name__=='__main__':main()
