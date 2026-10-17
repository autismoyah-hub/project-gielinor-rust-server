"""One original sanctuary/workshop sample; no full-floor production changes.

Authored floor definitions and terrain are original data, rendered by the same
native floor blending and terrain lighting as the authentic scene references.
"""
import argparse,json,math
from pathlib import Path

def main():
 p=argparse.ArgumentParser();p.add_argument('package',type=Path);p.add_argument('output',type=Path);p.add_argument('--refined',action='store_true');a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
 assets={x['name']:x for x in json.loads((a.package/'catalogue.json').read_text())['assets']}
 definitions=[('U',5000,'5b5953'),('U',5001,'615b50'),('U',5002,'424944'),('U',5003,'3c4140'),('U',5004,'696254'),('O',5010,'334d50')]
 size=64 if a.refined else 48;shift=(14,10) if a.refined else (0,0)
 rows=[f'S {size} {size}']+[f'P {kind} {id} {rgb}' for kind,id,rgb in definitions];terrain=[];props=[]
 floor={(x,z) for x in range(28,40) for z in range(39,53)}|{(x,z) for x in range(41,55) for z in range(39,54)}|{(x,z) for x in range(38,44) for z in range(44,48)}
 floor={(x+shift[0],z+shift[1]) for x,z in floor}
 for x in range(24,24+size):
  for z in range(24,24+size):
   sx,sz=x-shift[0],z-shift[1]
   near=min(floor,key=lambda v:(v[0]-x)**2+(v[1]-z)**2);d=math.hypot(x-near[0],z-near[1]);elevation=0
   if (x,z) not in floor:
    height=max(0,min(4.7 if a.refined else 3.1,(d-.5)*.64))
    if sz<39:height*=.18 # Authored foreground cutaway, shared across comparison variants.
    height*=1+.10*math.sin(x*.41+z*.19);elevation=round(height*16)
   underlay=5001 if sx<=40 else 5002
   if (x,z) not in floor:underlay=5004 if d<1.5 else 5003
   elif 44<=sx<=50 and 43<=sz<=49:underlay=5003 # local damp at the spring
   elif 35<=sx<=43 and 44<=sz<=47:underlay=5005 # worn threshold
   overlay=0
   if 56<=sx<=57 and 36<=sz<=56:
    underlay=5003;overlay=5011;elevation=0
   terrain.append([x,z,elevation,underlay,overlay,0,0]);rows.append('A '+' '.join(map(str,terrain[-1])))
 def put(name,x,z,rotation=0,story=''):
  if a.refined and name=='supply_rack':z+=1
  x+=shift[0];z+=shift[1]
  d=assets[name];props.append(dict(asset=name,id=d['object_id'],x=x,z=z,shape=10,rotation=rotation,story=story));rows.append(f'L {x} {z} 0 {d["object_id"]} 10 {rotation}')
 def wallx(start,end,z):
  for x in range(start,end,2):put('wall',x,z)
 def wallz(x,start,end):
  for z in range(start,end,2):put('wall',x,z,1)
 # A complete service suite around a maintained spring, with a defined entry.
 wallx(41,55,53);wallz(54,40,52);wallz(40,39,43);wallz(40,48,53)
 put('arch',49,38,story='Main worker entry frames the spring; foreground facade is cut away.')
 put('parapet',43,38);put('parapet',46,38);put('pillar',53,38)
 put('arch',39,44,1,'Shared service threshold opens into the workshop.')
 put('restoration_font',46,45,story='Maintained water source; damp terrain is localized around its use.')
 put('worker_bed',42,51);put('worker_bed',44,51)
 put('worker_bench',46,50);put('worker_bench',51,49,1)
 put('bank_counter',41,40,1,'Wages/ledger station faces the entrance side.')
 put('ledger_table',51,45);put('stool',51,44)
 put('water_barrel',42,48);put('water_barrel',43,48);put('worker_kettle',44,47)
 put('brazier',44,46);put('stores',51,51);put('supply_rack',48,52,0)
 put('banner',45,52);put('repair_slab',48,40)
 # Hot face opens EAST into the work lane. Rotation 3 is intentional: earlier
 # full-floor furnaces faced into their west wall under rotation 1.
 wallx(28,40,52);wallz(27,40,52)
 put('furnace',28,46,3,'A compact hot face opens into a usable handling lane.')
 put('anvil',31,46);put('workbench',31,40)
 put('pressure_pump',36,49);put('pipe',37,51,1)
 put('coolant_trough',36,44,1);put('supply_rack',30,51)
 put('timber_stack',34,50);put('stores',28,40)
 put('ore_cart',34,40);put('track',34,43);put('track',34,45)
 put('ledger_table',29,50);put('worker_bench',28,43,1)
 put('brazier',38,50);put('water_barrel',37,40)
 # Structural supports, repaired failure and native-height terrain meet visibly.
 put('pit_prop',31,50);put('pillar',39,51);put('pillar',28,39)
 put('wall_broken',54,51,1);put('rubble',55,52)
 put('cave_mass',25,45);put('cave_mass',29,54);put('cave_mass',51,55)
 put('cave_shoulder',26,39);put('cave_shoulder',55,42)
 put('scale_figure',38,45,story='Static 1.72-tile ruler convention; not an actor.')
 if a.refined:
  for x,z in [(32,52),(38,52),(48,53),(54,48)]:put('wall_lantern',x,z,1 if x==54 else 0,story='Small working light fixture; no dynamic light is emitted.')
  put('wall_coats',42,53);put('personal_shelf',48,52);put('personal_shelf',27,49,3)
  put('worker_rug',43,49);put('stone_threshold',49,38)
  for x,z in [(28,45),(29,42),(35,49),(38,47),(52,50),(45,44)]:put('ground_scatter',x,z,story='A small occupational edge cluster, kept out of the central lane.')
 (a.output/'candidate.scene').write_text('\n'.join(rows)+'\n')
 # Controlled ablations retain every object and camera; only one terrain layer changes.
 for variant in ['uniform-material','flat-height']:
  adjusted=[]
  for line in rows:
   if line.startswith('A '):
    v=line.split()
    if variant=='uniform-material':v[4]='5001';v[5]='0'
    else:v[3]='0'
    line=' '.join(v)
   adjusted.append(line)
  (a.output/f'{variant}.scene').write_text('\n'.join(adjusted)+'\n')
 targets={name:[x+shift[0],z+shift[1]] for name,(x,z) in dict(suite=[41,46],sanctuary=[47,46],workshop=[34,46]).items()}
 manifest=dict(title='The Springhouse Service Suite',stage='Small complete scene calibration; not approval for full-floor production',iteration=2 if a.refined else 1,original_only=True,scene_size=[size,size],source='Entirely authored terrain, original underlay/overlay colors and original Tower meshes; no extracted geometry or textures',native_terrain=dict(underlays=definitions,terrain_tiles=len(terrain),assembly='Region/Scene.add_chunk -> native underlay neighborhood blending/terrain light/TileMesh; no new rasterizer',collision='Visual-only authored terrain; no authoritative gameplay collision changes'),floor_planning_bounds=[28+shift[0],39+shift[1],54+shift[0],53+shift[1]],props=props,camera_targets=targets,purpose='Occupied sanctuary linked by a working threshold to a compact furnace/repair/quench workshop; terrain boundary and a drainage course give surrounding context',limitations=['No gameplay actors or UI','Static flames/water; no smoke, reflection, transparency or emitted dynamic light','Original opaque color materials only; no custom textures','Foreground is an authored cutaway; no dynamic roof/occlusion lifecycle','Artistic approval required before changes are scaled across Floor One'])
 (a.output/'candidate-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print('SCENE_CALIBRATION_COMPOSITION_PASS',len(props),'original props;',len(terrain),'authored native terrain tiles')
if __name__=='__main__':main()
