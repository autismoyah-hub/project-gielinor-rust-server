"""Authored original Floor One: continuous terrain and functional occupation.

Offline coordinates and planning masks only. Never writes production maps.
"""
import argparse,json,math
from pathlib import Path
from collections import deque
from compose_floor import inside,distance

AREAS=[
 ('arrival','The Workers\' Descent',[(57,29),(68,29),(71,35),(69,43),(65,46),(57,43),(54,37)],'stone_dark',(62,37)),
 ('sanctuary','The Springhouse Refuge',[(53,46),(68,46),(73,50),(73,61),(69,66),(52,66),(50,60),(50,51)],'stone',(62,55)),
 ('workshop','The Repair and Casting Hall',[(35,56),(46,55),(53,60),(52,71),(48,77),(33,77),(30,70),(31,61)],'stone_dark',(43,65)),
 ('quarry','The Worked Seam',[(30,78),(40,77),(48,82),(49,91),(44,98),(31,97),(27,91),(27,84)],'soil',(38,87)),
 ('waterworks','The Headgate House',[(79,50),(90,50),(95,57),(94,69),(90,76),(78,75),(75,69),(76,56)],'stone_dark',(85,63)),
 ('garden','The Root-fed Cistern',[(84,76),(94,77),(99,84),(96,93),(88,97),(79,92),(78,84)],'soil',(89,86)),
 ('vents','The Broken Exhaust',[(70,79),(78,78),(84,83),(85,91),(80,97),(70,97),(67,92),(67,85)],'stone_dark',(75,88)),
 ('narthex','The Cooling Walk',[(56,73),(64,73),(67,79),(66,91),(63,98),(55,96),(52,89),(53,80)],'stone_dark',(60,85)),
 ('foundry','The Bound Vessel',[(53,97),(65,97),(73,103),(74,115),(69,122),(53,123),(46,117),(46,106)],'stone_dark',(60,109)),
 ('ascent','The Last Bearing',[(65,121),(72,122),(76,128),(73,134),(63,134),(60,129)],'stone',(68,128)),
]
PATHS=[([(62,28),(62,38),(62,49)],4),([(52,56),(49,59),(46,63)],4),([(71,55),(77,56),(81,60)],4),
 ([(41,74),(38,81),(38,87)],4),([(62,63),(61,73),(60,83),(60,100)],4),
 ([(50,70),(54,76),(59,78)],3),([(89,72),(93,80),(93,88),(86,93),(81,93)],3),
 ([(80,74),(77,80),(75,87)],3),([(65,76),(70,80),(72,85)],3),
 ([(69,94),(66,99),(64,103)],3),([(43,94),(48,100),(51,105)],3),
 ([(70,118),(71,124),(69,127)],3)]
CLEAR={'ash_imp':[72,84,79,91],'ash_warden':[53,103,66,113]}

def main():
 p=argparse.ArgumentParser();p.add_argument('package',type=Path);p.add_argument('output',type=Path);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
 assets={x['name']:x for x in json.loads((a.package/'catalogue.json').read_text())['assets']};cells={};props=[];blocked=set();bridges=set()
 for x in range(26,101):
  for z in range(27,136):
   if any(any(distance(x+.5,z+.5,u,v)<=width/2 for u,v in zip(points,points[1:])) for points,width in PATHS):cells[x,z]=('spine','stone_dark')
   for key,title,poly,material,center in AREAS:
    if inside(x+.5,z+.5,poly):cells[x,z]=(key,material)
 water=set()
 for x,z in cells:
  if cells[x,z][0]=='waterworks' and x in [84,85,86] and 52<=z<=75:water.add((x,z))
  if cells[x,z][0]=='garden' and ((x+.5-88.5)/4.1)**2+((z+.5-84.0)/3.3)**2<1:water.add((x,z))
  if cells[x,z][0]=='garden' and 75<=z<=82 and 84<=x<=86:water.add((x,z))
 for key in water:cells[key]=(cells[key][0],'water')
 def put(name,x,z,rotation=0,obstacle=True,story=''):
  if name not in assets:raise ValueError('Missing original kit asset '+name)
  entry=assets[name];w,l=entry['footprint'];w,l=(l,w) if rotation%2 else (w,l)
  props.append(dict(asset=name,id=entry['object_id'],x=x,z=z,shape=10,rotation=rotation,footprint=[w,l],story=story))
  occupied={(x+dx,z+dz) for dx in range(w) for dz in range(l)}
  if obstacle:
   if name in ['arch','great_arch','pit_prop']:
    occupied={(x+dx,z+dz) for dx in range(w) for dz in range(l) if (dz if rotation%2 else dx) in [0,(l if rotation%2 else w)-1]}
   blocked.update(occupied)
  if name in ['bridge','long_bridge']:bridges.update(occupied)
 def wall_x(x0,x1,z,low=False):
  for x in range(x0,x1,2):put('parapet' if low else 'wall',x,z,obstacle=not low)
 def wall_z(x,z0,z1,low=False):
  for z in range(z0,z1,2):put('parapet' if low else 'wall',x,z,1,obstacle=not low)
 # Entry architecture is incomplete at a visible failure, not uniformly ruined.
 put('arch',61,31,story='Worker portal frames the maintained springhouse route.')
 wall_z(56,32,42);wall_z(68,33,41,True)
 put('wall_broken',57,41,1,story='One collapsed service bay explains adjacent debris.')
 put('rubble',56,42);put('repair_slab',61,39,obstacle=False);put('brazier',59,36);put('banner',66,40)
 put('scale_figure',63,39,obstacle=False)
 # The refuge is a converted service house: a spring, rest bay and wages desk.
 wall_x(52,70,65);wall_z(50,49,63);wall_z(72,51,55);wall_z(72,59,63)
 put('wall_corner',50,64,obstacle=False);put('wall_corner',71,64,1,False)
 put('restoration_font',61,54,story='Repaired gravity-fed spring remains the safe hub landmark.')
 put('sanctuary_seal',59,52,obstacle=False)
 put('bank_counter',52,55,1,story='The old wages office now faces the public ambulatory.')
 put('ledger_table',66,59);put('stool',66,58);put('stores',68,61)
 for x in [54,57]:put('worker_bed',x,62,story='Maintained bunks in a screened north service bay.')
 put('parapet',54,60);put('parapet',57,60);put('supply_rack',51,51,1,False)
 put('workbench',68,51,1);put('banner',61,64);put('brazier',58,55)
 put('worker_bench',57,58);put('worker_bench',64,58)
 put('water_barrel',52,52);put('water_barrel',53,52);put('worker_kettle',58,56)
 put('stores',68,63);put('stool',66,61);put('supply_rack',65,64,0,False)
 for x in [55,69]:put('pillar',x,49)
 put('arch',53,57,1);put('arch',72,55,1)
 put('repair_slab',59,51,obstacle=False);put('repair_slab',65,61,1,False)
 # Workshop: fuel/stock north, hot work west, quench east and a clear handling lane.
 wall_x(34,48,76);wall_z(31,61,73)
 put('pit_prop',36,74);put('supply_rack',34,75,0,False);put('timber_stack',44,73)
 put('stores',39,73);put('stores',46,59,1)
 for x,z in [(33,64),(33,68)]:put('furnace',x,z,1,story='Hot face opens onto the shared handling aisle.')
 for x,z in [(38,64),(38,68)]:put('anvil',x,z)
 put('workbench',45,61,story='Repair and finishing bench receives the cooled stock.')
 put('coolant_trough',47,67,1,story='Quench sits beside output, supplied from the headrace return.')
 put('casting_hoist',42,69);put('ore_cart',40,59)
 for z in [59,61,63]:put('track',42,z,obstacle=False)
 put('ledger_table',35,59);put('brazier',48,74);put('scale_figure',44,65,obstacle=False)
 # Water is infrastructure: intake, controlled head, wheel, spill and return.
 wall_x(78,80,75);wall_x(82,92,75);wall_z(94,57,69,True)
 wall_z(83,54,72,True);wall_z(87,54,72,True)
 put('sluice_gate',84,73,story='A lifting gate controls water from the upper cistern.')
 put('waterwheel',84,61,1,story='Wheel plane follows the race; bearings sit on the retaining banks.')
 put('grand_pillar',81,61);put('grand_pillar',88,61)
 put('bridge',84,57,1,False,story='Maintenance crossing joins both dry banks below the wheel.')
 put('pressure_pump',78,65);put('pipe',80,66,1,False);put('pipe',78,70,0,False)
 put('workbench',90,65,1);put('stores',90,54);put('ledger_table',89,70)
 put('repair_slab',89,72,obstacle=False,story='Dry fishing/maintenance landing above the race.')
 put('railing',90,72);put('scale_figure',89,59,obstacle=False)
 # Quarry: the worked seam, braced face, loaded cart and a rear exploratory route.
 for x,z in [(28,85),(29,89),(33,94),(43,93)]:put('ore_outcrop',x,z)
 put('pit_prop',31,80);put('pit_prop',30,92,1);put('timber_stack',35,79)
 put('ore_cart',39,82);put('stores',44,84,1);put('rubble',28,82)
 for z in [81,83,85,87]:put('track',36,z,obstacle=False)
 put('brazier',44,90);put('scale_figure',40,88,obstacle=False)
 # Root breach and cistern: a softer inhabited margin with dry exploration access.
 put('root_tree',94,84,story='One roof breach introduced roots and soil beside the leaking reservoir.')
 put('long_bridge',87,82,1,False,story='A timber crossing reaches both dry banks of the cistern.')
 for x,z in [(82,88),(91,91),(95,80)]:put('fungus',x,z,obstacle=False)
 put('wall_broken',95,89,1);put('rubble',96,91);put('stores',92,78)
 put('railing',84,81);put('ledger_table',81,90,story='A small gathering/rest place uses the dry western bank.')
 # Exhaust danger is legible through a connected system; preserve an 8x8 apron.
 put('exhaust_stack',70,80);put('exhaust_stack',81,92)
 for x,z in [(69,86),(80,85),(75,94)]:put('vent',x,z)
 for x,z,r in [(71,82,1),(73,82,1),(79,94,1),(81,89,0)]:put('pipe',x,z,r,False)
 put('pressure_pump',68,91);put('heat_fissure',80,90,1,False)
 put('wall_broken',68,95);put('rubble',68,96)
 # Cooling walk: narrow repeated structural bays with occupied rest pockets.
 for z in [76,83,90]:
  put('pillar',54,z);put('pillar',65,z)
 put('coolant_trough',54,86,1);put('ledger_table',55,80);put('stool',56,81)
 put('banner',64,75);put('brazier',63,94)
 put('worker_bench',55,91,1);put('water_barrel',55,93)
 # Foundry is a single constructed destination: bearing, bound vessel, casting apron.
 put('great_arch',58,98,story='The ordinary portal language grows into a substantial foundry threshold.')
 wall_x(51,70,121);wall_z(47,106,116);wall_z(72,105,116)
 for x,z in [(49,104),(70,104),(49,115),(70,115)]:put('grand_pillar',x,z)
 put('great_crucible',58,115,story='The bound vessel dominates the far wall without occupying the combat apron.')
 put('casting_hoist',48,112);put('coolant_trough',49,108,1);put('coolant_trough',69,108,1)
 for x,z,r in [(50,117,0),(69,116,0),(52,119,1),(67,119,1)]:put('pipe',x,z,r,False)
 put('heat_fissure',55,116,0,False);put('heat_fissure',66,116,0,False)
 put('brazier',52,101);put('brazier',69,101);put('scale_figure',60,110,obstacle=False)
 # Completion is visibly above the foundry; only a future interaction will ascend.
 put('ascent_stair',66,127,story='The spiral bearing is the distinct completion destination beyond the vessel.')
 put('arch',68,124);put('banner',73,128)
 wall_x(63,73,133,True);put('grand_pillar',62,130);put('brazier',73,132)
 # Bedrock envelopes selected outer shoulders. Authored gaps keep player sightlines.
 for x,z in [(52,35),(69,44),(48,49),(29,59),(27,73),(26,89),(30,96),(45,94),(76,51),(94,52),(96,68),(96,77),(97,91),(83,97),(46,101),(44,116),(52,123),(74,117),(75,130)]:
  put('cave_mass',x,z,obstacle=False,story='Native bedrock carries the remaining roof load beside built structure.')
 for x,z in [(54,41),(70,48),(30,77),(47,78),(28,93),(92,73),(96,82),(82,94),(71,96),(48,119),(73,124)]:put('cave_shoulder',x,z,obstacle=False)
 for name,bounds in CLEAR.items():
  x0,z0,x1,z1=bounds;intersections=[r for r in props if r['asset'] not in ['scale_figure','heat_fissure','track'] and any(x0<=x<=x1 and z0<=z<=z1 for x in range(r['x'],r['x']+r['footprint'][0]) for z in range(r['z'],r['z']+r['footprint'][1]))]
  if intersections:raise ValueError(f'{name} clear apron has scenery: {[(r["asset"],r["x"],r["z"]) for r in intersections]}')
 walk=(set(cells)-water-blocked)|bridges
 nodes={'arrival':(63,39),'sanctuary':(62,51),'workshop':(44,65),'quarry':(40,88),'waterworks':(89,59),'garden':(92,86),'vents':(75,88),'narthex':(60,85),'foundry':(60,110),'ascent':(70,123)}
 start=nodes['arrival'];seen={start};queue=deque([start])
 while queue:
  x,z=queue.popleft()
  for nxt in [(x-1,z),(x+1,z),(x,z-1),(x,z+1)]:
   if nxt in walk and nxt not in seen:seen.add(nxt);queue.append(nxt)
 reach={k:v in seen for k,v in nodes.items()}
 if not all(reach.values()):raise ValueError(f'Authored planning access disconnected: {reach}; failed anchors '+str({k:dict(in_ground=v in cells,blocked=v in blocked,water=v in water) for k,v in nodes.items() if not reach[k]}))
 ground=[]
 for name,entry in assets.items():
  if name.startswith('ground_'):ground.append(dict(asset=name,id=entry['object_id'],x=entry['world_anchor'][0],z=entry['world_anchor'][1],shape=10,rotation=0))
 scene='S 80 112\n'+''.join(f'L {r["x"]} {r["z"]} 0 {r["id"]} {r["shape"]} {r["rotation"]}\n' for r in ground+props)
 (a.output/'springhouse-cinderworks.scene').write_text(scene)
 geology={(x+dx,z+dz) for x,z in cells for dx in range(-3,4) for dz in range(-3,4) if dx*dx+dz*dz<=10 and 25<=x+dx<=102 and 27<=z+dz<=135}-set(cells)
 ground_cells=[[x,z,*data] for (x,z),data in sorted(cells.items())]+[[x,z,'geology','stone_dark'] for x,z in sorted(geology)]
 manifest=dict(title='Cinderworks — The Springhouse and Bound Vessel',stage='Reference-informed complete offline proposal; awaiting artistic review',origin=[24,24],scene_size=[80,112],tile_units=128,areas=[dict(key=k,title=t,polygon=poly,material=m,center=c) for k,t,poly,m,c in AREAS],routes=PATHS,ground_cells=ground_cells,geology_cells=len(geology),props=props,planning_access=dict(anchors=nodes,reachable=reach,walk_cells=len(walk),reachable_cells=len(seen),limitation='Authored 2D mask only; not engine collision/height/action/encounter acceptance'),encounter_visual_clear_aprons=CLEAR,water_system='Root-fed cistern -> headgate -> race/wheel -> southern spill; maintenance crossings and dry fishing landings',counts=dict(ground_cells=len(cells),water_cells=len(water),props=len(props),ground_models=len(ground)))
 (a.output/'design-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print('REFERENCE_FLOOR_COMPOSITION_PASS',manifest['counts'],reach)
if __name__=='__main__':main()
