"""Concept-level topology and migration audit; never live playability acceptance."""
from pathlib import Path
import json,math
from collections import deque
from PIL import Image,ImageDraw,ImageFont
from plan import P,ROOMS,ROUTES,RELOCATE
def inside(p,poly):
 x,y=p;v=False;j=len(poly)-1
 for i,a in enumerate(poly):
  b=poly[j]
  if(a[1]>y)!=(b[1]>y)and x<(b[0]-a[0])*(y-a[1])/(b[1]-a[1])+a[0]:v=not v
  j=i
 return v
def distance(p,a,b):
 dx,dy=b[0]-a[0],b[1]-a[1];t=max(0,min(1,((p[0]-a[0])*dx+(p[1]-a[1])*dy)/(dx*dx+dy*dy)))if dx or dy else 0
 return math.dist(p,(a[0]+dx*t,a[1]+dy*t))
def main():
 plan=json.loads((P/'floor-plan.json').read_text());water=[(68,56),(71,59),(71,64),(69,67),(67,65),(67,60)];cut=[(45,55),(48,54),(54,57),(55,63),(51,67),(46,64),(44,59)]
 nodes=set()
 for x in range(28,93):
  for z in range(29,84):
   c=(x+.5,z+.5);land=any(inside(c,q['polygon'])for q in ROOMS);corridor=any(distance(c,a,b)<=1.2 for route in ROUTES.values()for a,b in zip(route,route[1:]))
   if(land or corridor)and not inside(c,water)and not inside(c,cut):nodes.add((x,z))
 start=RELOCATE['arrival-first'];reachable={start};queue=deque([start]);parent={start:None}
 while queue:
  x,z=queue.popleft()
  for q in [(x-1,z),(x+1,z),(x,z-1),(x,z+1)]:
   if q in nodes and q not in reachable:reachable.add(q);queue.append(q);parent[q]=(x,z)
 anchors={}
 for key,(x,z)in RELOCATE.items():
  choices=[p for p in [(x-1,z),(x+1,z),(x,z-1),(x,z+1),(x,z)]if p in reachable];anchors[key]={'proposed_target':[x,z,0],'reachable_concept_approaches':choices};assert choices,key
 arenas=[]
 for name,bounds in [('Imp',(64,67,71,74)),('Warden',(75,51,87,61))]:
  a,b,c,d=bounds;missing=[(x,z)for x in range(a,c+1)for z in range(b,d+1)if(x,z)not in reachable];assert not missing,(name,missing)
  arenas.append({'name':name,'bounds':bounds,'dimensions':[c-a+1,d-b+1],'all_concept_tiles_reachable':True})
 report={'scope':'PROPOSED2D footprint union only. Two/fractional-tile corridor brush, room ground, water/collapse exclusions. No custom prop clipping, actor occupancy, native pathfinder, terrain integration or gameplay interaction acceptance.','concept_walkable_tiles':len(nodes),'connected_from_arrival':len(reachable),'all_region_centers_have_reachable_ground':all(any(abs(x-q['label'][0])+abs(z-q['label'][1])<=3 for x,z in reachable)for q in ROOMS),'anchors':anchors,'arenas':arenas,'playability_claim':False,'native_validation_after_approval':['Decompose study shell into valid per-object placements and actual collision footprints','Generate authoritative terrain/height/collision data matching native rendering','Assert2tileclearance at gateways and proposed safe ledges with actual server edges','Validate existing interaction policies and NPC footprints, including3tilefishing approach','Measure full native frame times after performance dependency resolution','Migrate saved prototype positions to safe landing without resetting progression or rewards']}
 (P/'concept-topology.json').write_text(json.dumps(report,indent=2)+'\n')
 # Add detailed compositional footprints only for the THREE developed fragments.
 # This enriches the master plan without pretending the other areas are built.
 im=Image.open(P/'floor-plan.png').convert('RGB');d=ImageDraw.Draw(im);scale=27;xy=lambda p:(100+(p[0]-28)*scale,1670-(p[1]-29)*scale);font=ImageFont.truetype('C:/Windows/Fonts/arial.ttf',14)
 lib=json.loads((P.parent/'cinderworks-environment/asset-catalog.json').read_text());symbols={'arch':'⌒','plate-mill':'M','workstation':'W','pump':'P','rail-span':'=','ore-cart':'C','furnace-bank':'F','chimney-backdrop':'↑','slate-headwall':'S','tool-rack':'T','arrival-inlay':'A','reward-surround':'R'}
 for name,transform in plan['scene_fragment_transforms'].items():
  dx,dz=transform['local_translation']
  for v in json.loads((P/'studies'/name/'placements.json').read_text()):
   if v['asset']in ['moss-fern','hearth-ring','arrival-inlay','rail-span']:continue
   x,z=v['x']+dx,v['z']+dz;w,h=lib[v['asset']]['size'];w,h=(h,w)if v['quarter_turn']%2 else(w,h);d.rectangle([xy((x,z+h)),xy((x+w,z))],fill='#253233',outline='#ccac78',width=2);a,b=xy((x+w/2,z+h/2));d.text((a,b),symbols.get(v['asset'],'·'),font=font,anchor='mm',fill='#ecd1a3')
 d.text((1930,1350),'Developed fragments:',font=ImageFont.truetype('C:/Windows/Fonts/arialbd.ttf',19),fill='#e1bf86')
 for i,t in enumerate(['M mill / W workbench / C cart','P pump / F furnace / S slate','Dark footprints show composition,','not accepted collision or bindings.','Only three scenes are developed.']):d.text((1930,1390+i*27),t,font=ImageFont.truetype('C:/Windows/Fonts/arial.ttf',17),fill='#d0d5cd')
 im.save(P/'floor-plan-detailed.png')
 print('CONCEPT_TOPOLOGY',len(reachable),'tiles;',len(anchors),'approaches; both translated arena dimensions retained. NOT native acceptance.')
if __name__=='__main__':main()
