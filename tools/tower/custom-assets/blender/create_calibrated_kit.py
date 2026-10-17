"""Original Cinderworks geometry informed by measured native reference studies.

No cache meshes, texture samples, Blender reference files or extracted arrays
are read. Reference IDs in manifests describe analysis, never copied inputs.
"""
import argparse,bpy,bmesh,json,math,sys,random,colorsys
from pathlib import Path
from mathutils import Vector

# Authored packed-HSL targets translated to glTF linear RGB, then independently
# quantized by the converter. Native lighting remains entirely in Rust.
COLORS={'stone':(6,0,30),'stone_light':(6,0,35),'stone_dark':(6,0,25),'cap':(6,0,38),'mortar':(6,1,20),'iron':(0,0,25),'iron_light':(0,0,34),'soot':(0,0,16),'wood':(6,3,32),'wood_light':(6,3,38),'brass':(9,3,48),'copper':(4,3,37),'heat':(3,6,62),'hot':(7,6,79),'water':(30,3,35),'damp':(25,2,28),'soil':(6,2,29),'moss':(16,2,26),'cloth':(27,3,39),'paper':(8,2,68)}

def main():
 p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--iteration',type=int,default=1);p.add_argument('--full',action='store_true');p.add_argument('--layout',type=Path);a=p.parse_args(sys.argv[sys.argv.index('--')+1:]);a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=True)
 bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
 mats={};pieces=[];assets=[]
 for name,(h,s,l) in COLORS.items():
  rgb=colorsys.hls_to_rgb((h+.25)/64,(l+.25)/128,(s+.25)/8)
  rgb=[c/12.92 if c<=.04045 else ((c+.055)/1.055)**2.4 for c in rgb]
  for smooth in [False,True]:
   m=bpy.data.materials.new(name+('-smooth' if smooth else ''));m.diffuse_color=(*rgb,1);m.use_nodes=True;m.use_backface_culling=True;m['native_render_type']=0 if smooth else 1
   m.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value=(*rgb,1);mats[name,smooth]=m
 def mesh(name,verts,faces,material='stone',smooth=False,face_mats=None):
  me=bpy.data.meshes.new(name);me.from_pydata(verts,[],faces);me.update();o=bpy.data.objects.new(name,me);bpy.context.collection.objects.link(o)
  names=list(dict.fromkeys(face_mats or [material]));[me.materials.append(mats[n,smooth]) for n in names]
  for f,n in zip(me.polygons,face_mats or [material]*len(faces)):f.material_index=names.index(n)
  bm=bmesh.new();bm.from_mesh(me);bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bmesh.ops.triangulate(bm,faces=list(bm.faces));bm.to_mesh(me);bm.free();pieces.append(o);return o
 def box(pos,size,material='stone',angle=0):
  x,y,z=pos;dx,dy,dz=[s/2 for s in size];c,s=math.cos(angle),math.sin(angle)
  verts=[(x+u*c-v*s,y+u*s+v*c,z+w) for w in [-dz,dz] for u,v in [(-dx,-dy),(dx,-dy),(dx,dy),(-dx,dy)]]
  return mesh('block',verts,[(0,3,2,1),(4,5,6,7),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7)],material)
 def extrusion(poly,depth,material='stone',y=0,face_mats=None):
  n=len(poly);verts=[(x,v,z) for v in [y-depth/2,y+depth/2] for x,z in poly];faces=[tuple(range(n-1,-1,-1)),tuple(range(n,2*n))]+[(i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n)]
  return mesh('profile',verts,faces,material,face_mats=face_mats)
 def lathe(pos,rings,material='stone',sides=8,phase=0,smooth=False):
  x,y,z=pos;verts=[(x+r*math.cos(i*math.tau/sides+phase+turn),y+r*math.sin(i*math.tau/sides+phase+turn),z+h) for h,r,turn in rings for i in range(sides)];faces=[tuple(range(sides-1,-1,-1))]
  for j in range(len(rings)-1):
   for i in range(sides):k=j*sides+i;n=j*sides+(i+1)%sides;faces.append((k,n,n+sides,k+sides))
  faces.append(tuple(range((len(rings)-1)*sides,len(rings)*sides)));return mesh('shaped section',verts,faces,material,smooth)
 def cylinder(pos,radius,length,material='iron',axis=(0,0,1),sides=8,radius2=None,smooth=False):
  o=lathe((0,0,0),[(-length/2,radius,0),(length/2,radius if radius2 is None else radius2,0)],material,sides,smooth=smooth);o.location=pos;o.rotation_euler=Vector(axis).to_track_quat('Z','Y').to_euler();return o
 def beam(start,end,width,material='wood'):
  v=Vector(end)-Vector(start);o=box((0,0,0),(width,width,v.length),material);o.location=(Vector(start)+Vector(end))/2;o.rotation_euler=v.to_track_quat('Z','Y').to_euler();return o
 def asset(name,index,footprint,build,purpose,reference,collision):
  nonlocal pieces
  pieces=[];bpy.ops.object.select_all(action='DESELECT');build()
  for o in pieces:o.select_set(True)
  bpy.context.view_layer.objects.active=pieces[0];bpy.ops.object.join();o=bpy.context.object;o.name=name
  bpy.ops.object.transform_apply(location=False,rotation=True,scale=True)
  for v in o.data.vertices:v.co+=o.location
  o.location=(0,0,0)
  bpy.ops.export_scene.gltf(filepath=str(a.output/(name+'.glb')),export_format='GLB',use_selection=True,export_apply=True,export_yup=True,export_materials='EXPORT',export_extras=True,export_cameras=False,export_lights=False)
  coords=[tuple(v.co) for v in o.data.vertices]
  assets.append(dict(name=name,model_id=1100000+index,object_id=63000+index,footprint=footprint,vertices=len(coords),triangles=len(o.data.polygons),bounds=[[min(v[i] for v in coords),max(v[i] for v in coords)] for i in range(3)],purpose=purpose,reference_analysis_ids=reference,collision_requirement=collision,origin='common ground datum; local centered X/Z; rotations cardinal; native128 units/tile',native_shading='explicit material extras; 0 smooth / 1 flat'))
  o.location=((len(assets)-1)%8*9,(len(assets)-1)//8*10,0);o.select_set(False)
 def arch():
  for sign in [-1,1]:
   poly=[(sign*x,z) for x,z in [(.91,0),(1.28,0),(1.25,.13),(1.19,.20),(1.15,1.17),(1.25,1.29),(.91,1.29)]]
   extrusion(poly,.43,'stone')
   extrusion([(sign*x,z) for x,z in [(.88,1.17),(1.28,1.17),(1.29,1.34),(.87,1.34)]],.51,'cap')
  for k in range(9):
   low=k*math.pi/9+.005;high=(k+1)*math.pi/9-.005;r0=.90;r1=1.24+(.045 if k==4 else 0)
   poly=[(r*math.cos(t),1.27+r*math.sin(t)) for r,t in [(r0,low),(r1,low),(r1,high),(r0,high)]]
   extrusion(poly,.45 if k!=4 else .51,['stone','stone_dark','stone','stone_light','cap','stone','stone_light','stone','stone_dark'][k])
  # A localized repair clamp, not a brass stripe around every course.
  box((1.17,-.235,.63),(.15,.05,.36),'iron');box((1.17,-.27,.50),(.06,.04,.06),'iron_light')
 def wall():
  # Continuous shaped body, shallow face colour transitions and one repair joint.
  poly=[(-1,0),(1,0),(1,1.96),(.96,2.08),(-.97,2.10),(-1,1.97)]
  extrusion(poly,.35,'stone')
  box((0,0,.05),(2.02,.48,.30),'stone_dark');box((0,0,2.09),(2.04,.49,.15),'cap')
  for x,z,w,h,mat in [(-.57,.52,.76,.34,'stone_dark'),(.38,1.25,1.05,.42,'stone_light')]:box((x,-.188,z),(w,.03,h),mat)
  for x in [-.70,0,.70]:
   o=extrusion([(x-.10,1.82),(x+.10,1.82),(x+.10,2.02),(x-.10,2.02)],.10,'stone_dark',y=-.22)
  box((.66,-.21,.72),(.025,.035,.65),'mortar',.07)
 def corner():
  poly=[(-1,-1),(1,-1),(1,-.62),(-.62,-.62),(-.62,1),(-1,1)]
  verts=[(x,y,z) for z in [0,2.10] for x,y in poly];n=len(poly);faces=[tuple(range(n-1,-1,-1)),tuple(range(n,2*n))]+[(i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n)]
  mesh('bonded corner',verts,faces,'stone')
  box((-.80,-.80,1.12),(.45,.45,2.24),'stone_dark');box((-.80,-.80,2.24),(.56,.56,.17),'cap')
 def fixture():
  box((0,0,.10),(1.65,1.3,.20),'stone_dark')
  lathe((-.2,.10,.18),[(0,.54,0),(.15,.57,0),(1.16,.48,.04),(1.34,.34,.04),(1.39,.28,.04)],'iron',8,smooth=True)
  cylinder((-.2,.10,1.65),.20,.50,'iron',sides=8)
  cylinder((-.2,-.40,.93),.36,.12,'iron_light',axis=(0,1,0),sides=8)
  cylinder((-.2,-.48,.93),.27,.045,'soot',axis=(0,1,0),sides=8)
  beam((-.2,-.515,.93),(-.04,-.515,1.10),.035,'brass')
  cylinder((.43,.10,.64),.17,.80,'iron',axis=(1,0,0),sides=6,smooth=True)
  cylinder((.81,.10,.37),.17,.60,'iron',sides=6,smooth=True)
  cylinder((.81,.10,.73),.23,.07,'iron_light',sides=6)
  beam((.58,.10,.80),(1.02,.10,.80),.07,'wood')
  box((-.16,.61,.60),(.26,.04,.32),'damp')
 asset('arch',10,[3,1],arch,'Worker portal; tapered jambs, restrained stone crown and single localized repair',[1868,2109,7016],'Block jambs only; opening remains traversable; future per-tile support definitions')
 asset('wall',5,[2,1],wall,'Continuous load-bearing masonry, coping and shallow repairs',[6132,6010,6143],'Two tile barrier; visual thickness does not define collision automatically')
 asset('wall_corner',68,[2,2],corner,'Bonded L corner with substantial end pier; complements straight wall',[6001,6132],'Block only L occupied edge cells, not all four cells')
 asset('pressure_pump',17,[2,2],fixture,'Asymmetric pressure vessel with accessible gauge, outlet and shutoff',[9089,9114,9085],'Two tile machine; leave front operating apron clear')
 def figure():
  for x in [-.13,.13]:box((x,0,.39),(.17,.22,.78),'wood')
  extrusion([(-.28,.77),(.28,.77),(.23,1.38),(-.23,1.38)],.31,'cloth')
  lathe((0,0,1.40),[(0,.15,0),(.23,.17,0),(.32,.10,0)],'cap',6)
  for sign in [-1,1]:beam((sign*.24,0,1.3),(sign*.36,0,.85),.12,'wood')
 asset('scale_figure',32,[1,1],figure,'Static 1.72-tile comparison convention; not an NPC or measured original player',[],'No collision; offline ruler only')
 if a.full:
  def rock(pos=(0,0,0),size=(1.5,1.2,1.6),seed=5,material='stone_dark'):
   rng=random.Random(seed);x,y,z=pos;w,d,h=size;sides=7
   verts=[]
   for elevation,scale in [(0,.88),(.32,1),(.76,.74),(1,.38)]:
    for k in range(sides):
     t=k*math.tau/sides;rr=1+rng.uniform(-.13,.13);verts.append((x+math.cos(t)*w*.5*scale*rr+elevation*.12,y+math.sin(t)*d*.5*scale*rr,z+elevation*h))
   faces=[tuple(range(sides-1,-1,-1))]
   for j in range(3):
    for k in range(sides):u=j*sides+k;v=j*sides+(k+1)%sides;faces.append((u,v,v+sides,u+sides))
   faces.append(tuple(range(3*sides,4*sides)));return mesh('cleaved rock',verts,faces,material)
  def pier(height=2.3,large=False):
   r=.50 if not large else .70
   lathe((0,0,0),[(0,r,0),(.15,r,0),(.29,r*.77,0),(.43,r*.63,0),(height-.38,r*.56,.10),(height-.23,r*.92,.10),(height,r,.10)],'stone',8)
   box((0,0,height+.06),(r*1.8,r*1.8,.12),'cap')
  def damaged_wall():
   extrusion([(-1,0),(1,0),(1,.82),(.68,.96),(.42,1.30),(.18,1.25),(-.02,1.66),(-.28,1.92),(-1,2.04)],.36,'stone')
   box((-.65,0,2.04),(.66,.47,.14),'cap');rock((.65,-.28,0),(.55,.37,.29),7)
  def parapet():
   extrusion([(-1,0),(1,0),(1,.70),(.94,.82),(-1,.85)],.36,'stone_dark');box((0,0,.86),(2.03,.47,.14),'cap')
  def buttress():
   extrusion([(-.42,0),(.42,0),(.35,.36),(.24,2.72),(-.24,2.72),(-.32,.76)],.75,'stone_dark')
   box((0,0,2.71),(.72,.94,.20),'cap');box((0,-.44,.58),(.30,.12,.36),'iron')
  asset('pillar',8,[1,1],pier,'Tapered polygonal service pier with distinct base and capital',[7016,7017],'Single occupied tile')
  asset('grand_pillar',9,[2,2],lambda:pier(3.8,True),'Major foundry bearing; hierarchy through profile, not stacked collars',[7016,6147],'Two tile structural support')
  asset('wall_broken',7,[2,1],damaged_wall,'Directional failure; irregular break rather than random missing cubes',[6011,174],'Barrier edge with separate rubble access check')
  asset('parapet',6,[2,1],parapet,'Low coping for visible water banks and foreground cutaway',[6143],'Retaining edge; cannot imply safe traversal')
  asset('rock_buttress',42,[1,1],buttress,'Leaning masonry bearing with broad foot and repaired strap',[23505],'One tile occupied support')
  def great_arch():
   arch()
   # Scale only this landmark's coherent geometry around its ground datum.
   for o in pieces:o.scale=(1.6,1.8,1.55)
  asset('great_arch',12,[5,2],great_arch,'Foundry threshold derived from service architecture, with deliberate monumental scale',[1868,2109],'Columns block; central passage requires dedicated supports')
  def broken_arch():
   extrusion([(-1.28,0),(-.91,0),(-.91,1.50),(-1.05,1.72),(-1.15,1.52),(-1.28,1.67)],.48,'stone')
   extrusion([(.90,0),(1.28,0),(1.28,1.26),(.96,1.4),(.90,1.16)],.47,'stone')
   rock((.25,0,0),(.7,.55,.38),13)
  asset('broken_arch',11,[3,1],broken_arch,'Old blocked maintenance threshold, with load-bearing remnants',[2106,174],'Rubble-defined access; not an open portal')
  def furnace():
   # Vaulted working shell is kept below the worker's shoulder.
   extrusion([(-.92,0),(.92,0),(.90,.74),(.64,1.14),(.32,1.31),(-.34,1.29),(-.68,1.1),(-.92,.68)],1.55,'soot')
   extrusion([(-.46,.08),(.46,.08),(.46,.55),(.30,.79),(-.28,.78),(-.46,.55)],.065,'heat',y=-.82)
   for x in [-.55,.55]:extrusion([(x-.10,0),(x+.10,0),(x+.10,.58),(x,.87),(x-.10,.58)],.17,'stone_dark',y=-.85)
   box((0,-1.04,.08),(.85,.40,.16),'iron');cylinder((-.38,.35,1.38),.20,.42,'soot',sides=6)
  asset('furnace',14,[2,2],furnace,'Compact vaulted worker furnace with a localized hot mouth',[6189,9085],'Two tile fixture; leave front handling apron')
  def bench():
   for x in [-.59,.59]:
    box((x,0,.33),(.14,.55,.66),'wood');beam((x,-.25,.12),(x,.25,.59),.10,'wood_light')
   for y in [-.24,0,.24]:box((0,y,.70),(1.6,.22,.12),'wood_light')
   box((.48,0,.83),(.25,.20,.20),'iron');beam((-.45,-.14,.80),(-.08,.12,.80),.055,'iron_light')
  asset('workbench',19,[2,1],bench,'Usable-height repair table with vise and a single hand tool',[6196,6188],'One tile wide access apron at front')
  def anvil():
   lathe((0,0,0),[(0,.35,0),(.30,.31,.08)],'wood',7)
   extrusion([(-.42,.33),(.34,.33),(.32,.45),(.57,.49),(.75,.59),(.35,.65),(-.43,.65),(-.43,.55),(-.27,.48)],.34,'iron_light')
  asset('anvil',20,[2,1],anvil,'Low, unmistakable heel and horn over a short working stump',[6150],'Keep striking side clear')
  def table():
   for x in [-.37,.37]:
    for y in [-.32,.32]:box((x,y,.30),(.10,.10,.60),'wood')
   box((0,0,.64),(.94,.84,.12),'wood_light');box((.16,.10,.72),(.36,.28,.035),'paper')
  asset('ledger_table',69,[1,1],table,'Worker ledger and intake/meal table; compact occupied scale',[6196],'Accessible interaction edge required later')
  def bed():
   for x in [-.38,.38]:
    for y in [-.75,.75]:box((x,y,.22),(.12,.12,.44),'wood')
   box((0,0,.32),(.86,1.75,.20),'wood_light');box((0,-.15,.47),(.74,1.28,.15),'cloth');box((0,.61,.46),(.66,.32,.13),'paper')
   beam((-.4,.77,.22),(-.4,.77,.74),.10);beam((.4,.77,.22),(.4,.77,.74),.10);box((0,.77,.66),(.88,.10,.19),'wood')
  asset('worker_bed',70,[1,2],bed,'Sanctuary bunk: evidence of occupation and a maintained sleeping bay',[6183],'Sleeping bay not a walk lane')
  def stool():
   for x,y in [(-.19,-.14),(.19,-.14),(0,.21)]:beam((x*1.25,y*1.25,0),(x,y,.47),.09)
   lathe((0,0,.46),[(0,.28,0),(.10,.29,0)],'wood_light',7)
  asset('stool',71,[1,1],stool,'Small movable furniture for occupied service pockets',[6196],'Scenery obstruction only if later enabled')
  def seat():
   for x in [-.66,.66]:box((x,0,.24),(.14,.46,.48),'wood')
   box((0,0,.50),(1.66,.56,.13),'wood_light');beam((-.65,.22,.26),(-.65,.22,1.02),.09,'wood');beam((.65,.22,.26),(.65,.22,1.02),.09,'wood')
   box((0,.22,.91),(1.65,.10,.22),'wood')
  asset('worker_bench',78,[2,1],seat,'Backed worker seating; makes maintained rest bays visibly habitable',[6196,6183],'Leave front approach clear')
  def barrel():
   lathe((0,0,0),[(0,.31,0),(.12,.36,0),(.42,.40,0),(.77,.36,0),(.88,.31,0)],'wood',9)
   for z in [.19,.67]:cylinder((0,0,z),.405,.075,'iron',sides=9)
   box((0,0,.91),(.15,.14,.06),'wood_light')
  asset('water_barrel',79,[1,1],barrel,'Faceted worker water/fuel store with restrained iron hoops',[6178],'Stored scenery; future water interaction not implemented')
  def cooking_pot():
   lathe((0,0,.04),[(0,.20,0),(.17,.34,0),(.42,.31,0),(.48,.23,0)],'iron',8,smooth=True)
   cylinder((0,0,.53),.25,.08,'iron_light',sides=8);cylinder((0,0,.58),.18,.025,'soot',sides=8)
   for sign in [-1,1]:beam((sign*.27,0,.22),(sign*.41,0,.37),.065,'iron')
  asset('worker_kettle',80,[1,1],cooking_pot,'Small kettle beside the rest-bay fire, localized signs of occupation',[6196,11020],'Static scenery, no cooking system')
  def rack():
   box((0,.12,1.35),(1.6,.13,.18),'wood')
   for x,z in [(-.54,.83),(-.10,.74),(.40,.90)]:
    beam((x,.01,z),(x,.01,1.37),.055,'wood_light');box((x,.01,z),(.27,.10,.11),'iron_light')
   box((.64,.06,1.04),(.14,.09,.50),'iron')
  asset('supply_rack',45,[2,1],rack,'Wall-hung smith tools, below eye level and visibly reachable',[6188,6055],'Wall attachment; avoid phantom floor blockage')
  def stores():
   for x,y,z,s in [(-.38,0,0,.7),(.37,.06,0,.65),(-.40,0,.73,.58)]:
    box((x,y,z+s/2),(s,s*.83,s),'wood')
    for zz in [z+.07,z+s-.07]:box((x,y-s*.43,zz),(s+.03,.045,.08),'wood_light')
    beam((x-s*.4,y-s*.46,z+.11),(x+s*.4,y-s*.46,z+s-.11),.075,'wood_light')
  asset('stores',24,[2,1],stores,'Uneven braced crates with an accessible stock face',[6178],'Stock edges block; front lane remains clear')
  def bank():
   box((0,0,.38),(2.6,.70,.76),'wood');box((0,0,.82),(2.76,.86,.13),'wood_light');box((-.74,0,.94),(.42,.30,.12),'iron')
   box((.37,-.10,.91),(.45,.32,.04),'paper');box((.83,.12,.99),(.23,.18,.20),'brass')
  asset('bank_counter',39,[3,1],bank,'Converted wages/ledger station, part of the sanctuary habitation',[6081,6196],'Future bank action; keep customer side clear')
  def font():
   lathe((0,0,0),[(0,1.23,0),(.17,1.30,0),(.28,1.02,0),(.67,.89,0),(.78,1.02,0)],'stone',10)
   cylinder((0,0,.80),.84,.045,'water',sides=10)
   extrusion([(-.18,0),(.18,0),(.17,1.35),(0,1.57),(-.17,1.35)],.32,'cap',y=.76)
   cylinder((0,.54,1.12),.075,.46,'copper',axis=(0,1,0),sides=6)
  asset('restoration_font',13,[3,3],font,'Maintained service spring with a simple dedication stone and spout',[6097,7016],'Water basin blocks; safe ambulatory outside')
  def seal():
   n=16;verts=[]
   for height in [.022,.05]:
    for radius in [2.46,2.61]:verts += [(radius*math.cos(k*math.tau/n),radius*math.sin(k*math.tau/n),height) for k in range(n)]
   faces=[]
   for k in range(n):j=(k+1)%n;faces.extend([(k,j,n+j,n+k),(2*n+k,3*n+k,3*n+j,2*n+j),(k,2*n+k,2*n+j,j),(n+k,n+j,3*n+j,3*n+k)])
   mesh('springhouse floor dedication',verts,faces,'stone_light')
  asset('sanctuary_seal',36,[6,6],seal,'Restrained angular floor dedication around the maintained spring',[],'Nonblocking inlay, no altered movement height')
  def slab():
   extrusion([(-.65,0),(.56,0),(.72,.07),(.45,.15),(-.58,.14)],.85,'stone_light')
  asset('repair_slab',72,[2,1],slab,'Placed replacement paving; use only where repairs or a threshold explain it',[174],'Low slab requires later height/collision treatment')
  def rubble():
   for x,y,w,h,seed in [(-.44,.1,.63,.38,1),(.27,-.10,.81,.29,2),(.60,.34,.32,.16,3)]:rock((x,y,0),(w,w*.65,h),seed)
   beam((-.7,-.1,.07),(.45,.45,.16),.08,'iron')
  asset('rubble',22,[2,2],rubble,'Directional failure cluster with one broken tie',[174,5950],'Place next to source failure, outside key route')
  def ore():
   rock((-.35,0,0),(1.35,1.10,1.0),12);rock((.47,.25,0),(1.1,.9,.68),9)
   extrusion([(-.55,.32),(.22,.64),(.12,.75),(-.62,.40)],.06,'copper',y=-.44)
  asset('ore_outcrop',23,[2,2],ore,'Cleaved lode with a small directional mineral seam',[5950,6147],'Future resource binding and respawn ownership required')
  asset('cave_mass',73,[3,3],lambda:rock((0,0,-.16),(3.7,3.0,3.2),23),'Large directional bedrock support, no uniform sphere piles',[6147,6000],'Approximate bounds; author walk edge explicitly')
  asset('cave_shoulder',74,[2,2],lambda:rock((0,0,-.15),(2.6,2.2,1.2),42),'Low exposed shoulder to blend floor and high cave mass',[5950,6001],'Low obstacle; no implied climbability')
  def wheel():
   # The wheel lies in X/Z; axle runs into the headgate building along Y.
   for y in [-.36,.36]:
    n=12;r0=1.22;r1=1.45;verts=[]
    for yy in [y-.07,y+.07]:
     for r in [r0,r1]:
      verts += [(r*math.cos(k*math.tau/n),yy,1.53+r*math.sin(k*math.tau/n)) for k in range(n)]
    faces=[]
    for k in range(n):j=(k+1)%n;faces.extend([(k,j,n+j,n+k),(2*n+k,3*n+k,3*n+j,2*n+j),(k,2*n+k,2*n+j,j),(n+k,n+j,3*n+j,3*n+k)])
    mesh('wheel rim',verts,faces,'wood')
   for k in range(10):
    t=k*math.tau/10;x,z=1.28*math.cos(t),1.53+1.28*math.sin(t)
    box((x,0,z),(.25,.86,.16),'wood_light',0)
   for t in [0,math.pi/3,2*math.pi/3]:
    beam((-1.22*math.cos(t),-.40,1.53-1.22*math.sin(t)),(1.22*math.cos(t),-.40,1.53+1.22*math.sin(t)),.14,'wood')
   cylinder((0,0,1.53),.17,1.8,'iron',axis=(0,1,0),sides=8)
  asset('waterwheel',16,[3,2],wheel,'Restrained broad rim and paddle silhouette; axis clearly meets bearings',[5589],'Wheel is scenery; separate bearings and channel collision')
  def pipe():
   cylinder((0,0,.43),.16,1.8,'iron',axis=(0,1,0),sides=6,smooth=True)
   for y in [-.64,.64]:box((0,y,.13),(.47,.20,.26),'stone_dark');cylinder((0,y,.43),.20,.08,'iron_light',axis=(0,1,0),sides=6)
  asset('pipe',18,[1,2],pipe,'Ground supply pipe with two credible supports and flanges',[9114],'Future scenery collision; not a walkable railing')
  def gate():
   for x in [-1.15,1.15]:box((x,0,1.25),(.40,.70,2.5),'stone')
   box((0,0,2.54),(2.8,.64,.26),'cap');box((0,.03,.99),(1.9,.22,1.67),'wood')
   for z in [.43,1.47]:box((0,-.12,z),(1.96,.05,.14),'iron')
   cylinder((0,0,2.20),.055,1.25,'iron',sides=6);beam((-.42,0,2.79),(.42,0,2.79),.07,'iron_light')
  asset('sluice_gate',75,[3,1],gate,'Headgate with visible lifting screw and braced leaf',[9114,5589],'Blocks water lane; maintenance bridge separate')
  def pit_prop():
   for x in [-1.02,1.02]:beam((x,0,0),(x*.94,0,2.18),.19,'wood')
   beam((-1.25,0,2.24),(1.25,0,2.24),.23,'wood_light')
   for sign in [-1,1]:beam((sign*.97,0,1.48),(sign*.32,0,2.24),.13,'wood')
  asset('pit_prop',76,[3,1],pit_prop,'Original timber working-face support with braced load path',[23505],'Posts block only; opening must remain usable')
  def exhaust():
   box((0,0,.12),(1.6,1.35,.24),'stone_dark')
   lathe((0,0,.23),[(0,.48,0),(1.9,.43,.04),(2.12,.29,.04)],'iron',7,smooth=True)
   extrusion([(-.59,2.12),(.59,2.12),(.79,2.32),(.60,2.52),(-.61,2.49),(-.76,2.30)],.95,'iron')
   box((0,-.50,2.29),(.76,.045,.16),'heat')
  asset('exhaust_stack',77,[2,2],exhaust,'Capped containment exhaust links vent grates to a coherent industrial system',[9085,9114],'Two tile fixture; no simulated smoke/light')
  def bridge():
   for x in [-.63,.63]:beam((x,-1.60,.05),(x,1.60,.05),.17,'wood')
   for k in range(10):box((0,-1.44+k*.32,.18),(1.60,.29,.16),'wood_light' if k in [2,7] else 'wood')
   for x in [-.76,.76]:
    for y in [-1.48,1.48]:box((x,y,.57),(.12,.12,.92),'wood')
    beam((x,-1.48,.92),(x,1.48,.92),.09,'wood')
  asset('bridge',28,[2,3],bridge,'Narrow timber maintenance bridge with visible bearers',[5589],'Future walking height .26 tile, banks must connect')
  def longbridge():
   bridge()
   for o in pieces:
    for v in o.data.vertices:v.co.y*=1.8
    o.location.y*=1.8
  asset('long_bridge',57,[2,6],longbridge,'Longer beam-and-plank crossing for a reservoir neck',[5589],'Requires height plane and connected landings')
  def trough():
   box((0,0,.10),(2.7,.93,.20),'stone_dark');box((0,0,.25),(2.35,.60,.08),'water')
   for y in [-.42,.42]:box((0,y,.33),(2.7,.17,.46),'stone')
   for x in [-1.27,1.27]:box((x,0,.33),(.17,.74,.46),'stone')
  asset('coolant_trough',40,[3,1],trough,'Accessible quenching basin beside hot work, not floating decoration',[6189],'Basin solid; water/actions future integration')
  def rail():
   for x in [-.90,.90]:box((x,0,.42),(.10,.10,.84),'iron')
   beam((-.90,0,.76),(.90,0,.76),.07,'iron_light')
  asset('railing',34,[2,1],rail,'Thin iron safety rail at falls and wheel apron',[9114],'Barrier collision explicitly required later')
  def brazier():
   for k in range(3):
    t=k*math.tau/3;beam((.36*math.cos(t),.36*math.sin(t),0),(.22*math.cos(t),.22*math.sin(t),.60),.08,'iron')
   lathe((0,0,.54),[(0,.20,0),(.20,.37,0),(.28,.32,0)],'iron',7)
   for x,y,h in [(-.12,0,.35),(.10,.05,.46),(.02,-.12,.29)]:lathe((x,y,.77),[(0,.12,0),(h*.48,.10,.15),(h,.025,.30)],'heat',5)
  asset('brazier',30,[1,1],brazier,'Small localized warm flame in a tripod bowl; no fake emitted light',[11020,5981],'Future fire animation/light is not implemented')
  def vent():
   box((0,0,.09),(1.5,1.20,.18),'stone_dark');box((0,0,.21),(1.16,.88,.11),'heat')
   for x in [-.47,-.23,0,.23,.47]:box((x,0,.32),(.09,1.0,.12),'iron')
   for y in [-.55,.55]:box((0,y,.31),(1.42,.15,.18),'iron_light')
  asset('vent',21,[2,2],vent,'Recessed containment grate; connected ducts explain danger',[5981,9114],'Future encounter hazard binding; no dynamic heat now')
  def fissure():
   extrusion([(-.95,0),(.93,0),(.67,.07),(.22,.10),(-.20,.07),(-.72,.10)],.14,'heat')
  asset('heat_fissure',43,[2,1],fissure,'Narrow contained leak, only at a failed joint',[5981],'Hazard behavior remains future work')
  def cart():
   box((0,0,.55),(1.2,1.5,.18),'wood')
   for x in [-.60,.60]:box((x,0,.90),(.12,1.5,.65),'wood')
   for y in [-.73,.73]:box((0,y,.9),(1.2,.12,.65),'wood')
   for x in [-.68,.68]:
    for y in [-.48,.48]:cylinder((x,y,.29),.27,.13,'iron',axis=(1,0,0),sides=7)
   rock((0,.16,.64),(.85,1,.62),9)
  asset('ore_cart',38,[2,2],cart,'Loaded haulage cart with visible running gear',[6178,5950],'Future scenery route must remain clear')
  def track():
   for z in [-.7,.0,.7]:box((0,z,.05),(1.5,.13,.10),'wood')
   for x in [-.52,.52]:box((x,0,.12),(.08,2,.10),'iron')
  asset('track',37,[2,2],track,'Two-tile ore transport section; sparse defined route, not floor wallpaper',[9114],'Near-ground obstruction/decoration only')
  def timber():
   for k in range(4):cylinder(((k%2)*.43-.22,0,.20+(k//2)*.36),.19,1.9,'wood',axis=(0,1,0),sides=6)
  asset('timber_stack',25,[2,2],timber,'Grouped replacement pit props near the working face',[23505],'Stored material blocks its bay only')
  def roots():
   lathe((0,0,0),[(0,.68,0),(.48,.47,.10),(1.25,.27,.20),(2.27,.20,.28),(3.10,.07,.42)],'wood',7)
   for u,v in [((0,0,1.7),(1.2,.3,2.5)),((0,0,2.2),(-.9,-.1,2.8)),((0,0,.25),(1.2,-.6,0)),((0,0,.2),(-.9,.8,0))]:beam(u,v,.17,'wood')
  asset('root_tree',26,[3,3],roots,'One directional root breach rather than repeated upright trees',[6147],'Original gathering identity; behavior not changed')
  def fungus():
   for x,y,h in [(-.35,0,.35),(.22,.14,.61),(.40,-.27,.25)]:
    cylinder((x,y,h/2),.055,h,'wood_light',sides=5);lathe((x,y,h),[(0,.29,0),(.10,.31,0),(.26,.06,0)],'moss',7)
  asset('fungus',27,[2,2],fungus,'Damp low growth clusters confined to the leaking cistern',[6158],'Future ordinary resource/creature placement separate')
  def hoist():
   for x in [-1.5,1.5]:beam((x,0,0),(x,.15,3.8),.23,'wood')
   beam((-1.75,.15,3.8),(1.75,.15,3.8),.28,'wood');beam((-1.5,0,2.5),(-.5,.15,3.8),.13,'wood_light')
   cylinder((0,.15,3.38),.25,.28,'iron',axis=(0,1,0),sides=8);beam((.08,.15,3.35),(.08,.15,1.47),.045,'iron')
   box((.10,.15,1.4),(.37,.14,.14),'iron_light')
  asset('casting_hoist',41,[4,2],hoist,'Working load path and overhead structure beside the casting apron',[9114],'Posts block; hanging load is scenery until animated')
  def crucible():
   lathe((0,.65,.15),[(0,2.05,0),(.35,2.25,0),(2.40,1.85,.03),(3.10,1.53,.03),(3.60,1.18,.03)],'soot',10,smooth=True)
   for sign in [-1,1]:
    extrusion([(sign*1.72,0),(sign*2.65,0),(sign*2.12,3.0),(sign*1.65,3.55),(sign*1.44,3.15)],1.18,'stone_dark',y=.65)
   cylinder((0,.65,3.75),.70,.48,'iron',sides=8)
   extrusion([(-.73,.28),(.73,.28),(.64,1.38),(.39,1.65),(-.39,1.65),(-.64,1.38)],.08,'heat',y=-1.35)
   for x in [-.54,0,.54]:box((x,-1.42,.99),(.09,.15,1.39),'iron')
   extrusion([(-.42,0),(.42,0),(.42,.30),(-.42,.30)],1.5,'iron',y=-1.79)
   box((0,-2.10,.33),(.54,1.50,.045),'hot')
  asset('great_crucible',15,[6,6],crucible,'Single bound vessel with buttresses, barred tap and casting throat; foundry landmark',[6189,9085],'Containment blocks; preserve broad combat apron in front')
  def stair():
   n=14
   for k in range(n):
    t=k*.24;x,z=1.15*math.cos(t),1.15*math.sin(t);box((x,z,.13+k*.20),(.90,.58,.24),'stone',t)
   lathe((0,0,0),[(0,.40,0),(3.15,.31,.12)],'stone_dark',8)
  asset('ascent_stair',33,[4,4],stair,'Distinct rise around a central bearing at the completion destination',[6085,7016],'Offline visual; future height planes and ascent interaction required')
  def banner():
   box((0,.03,1.35),(.76,.06,1.32),'cloth');beam((-.50,0,2.04),(.50,0,2.04),.07,'wood');box((0,-.025,1.47),(.11,.035,.56),'brass')
  asset('banner',31,[1,1],banner,'Single maintained worker sign rather than repeated ceremonial ribbons',[6046],'Wall mounted; no floor blockage')
  # Continuous closed original terrain, generated from the authored floor mask.
  if a.layout:
   layout=json.loads(a.layout.resolve().read_text());groups={}
   authored_cells={(x,z) for x,z,area,material in layout['ground_cells'] if area!='geology'}
   for x,z,area,material in layout['ground_cells']:
    # Native GameModel draw radius must remain below 3000 units. Partition on a
    # fixed 22-tile lattice; IDs remain stable when other chunks are added.
    cx,cz=(x-24)//22,(z-24)//22
    groups.setdefault((area,material,cx,cz),set()).add((x,z))
   ordered=sorted(groups)
   for area,material,cx,cz in ordered:
    area_index=['arrival','sanctuary','workshop','quarry','waterworks','garden','vents','narthex','foundry','ascent','spine','reservoir','geology'].index(area)
    group_index=area_index*48+(24 if material=='water' else 0)+cz*4+cx
    cells=groups[area,material,cx,cz];minx=min(x for x,z in cells);minz=min(z for x,z in cells);w=max(x for x,z in cells)-minx+1;l=max(z for x,z in cells)-minz+1
    def terrain(cells=cells,material=material,minx=minx,minz=minz,w=w,l=l,area=area):
     points=sorted(set((x+dx,z+dz) for x,z in cells for dx,dz in [(0,0),(1,0),(1,1),(0,1)]));lookup={p:i for i,p in enumerate(points)};n=len(points);height=-.12 if material=='water' else 0
     def surface_height(x,z):
      if area!='geology':return height
      if any((x+dx,z+dz) in authored_cells for dx,dz in [(-1,-1),(0,-1),(-1,0),(0,0)]):return 0
      nearby=[(u,v) for u in range(x-4,x+4) for v in range(z-4,z+4) if (u,v) in authored_cells]
      if not nearby:return 2.0
      near=min(nearby,key=lambda q:math.hypot(x-q[0]-.5,z-q[1]-.5));d=math.hypot(x-near[0]-.5,z-near[1]-.5)
      cutaway=.26 if z<near[1] else 1.0
      return max(.10,(.4+min(2.8,max(0,d-.7)*1.25))*(1+.12*math.sin(x*.8+z*.27))*cutaway)
     verts=[(x-minx-w/2,z-minz-l/2,height-.40) for x,z in points]+[(x-minx-w/2,z-minz-l/2,surface_height(x,z)) for x,z in points];faces=[];names=[]
     for x,z in sorted(cells):
      ids=[lookup[x+dx,z+dz] for dx,dz in [(0,0),(1,0),(1,1),(0,1)]];i,j,k,v=ids
      faces.extend([(n+i,n+j,n+k),(n+i,n+k,n+v),(i,k,j),(i,v,k)])
      # Broad contextual wear patches, never alternating every tile.
      tone=material
      if material in ['stone_dark','soil'] and math.sin(x*.22+z*.09)+math.cos(z*.16)>.95:tone='mortar' if material=='stone_dark' else 'moss'
      names.extend([tone,tone,'stone_dark','stone_dark'])
      for missing,u,v in [((x,z-1),i,j),((x+1,z),j,k),((x,z+1),k,ids[3]),((x-1,z),ids[3],i)]:
       if missing not in cells:faces.append((u,v,n+v,n+u));names.append('stone_dark')
     mesh('continuous ground',verts,faces,material,face_mats=names)
    asset(f'ground_{area}_{material}_{cx}_{cz}',200+group_index,[w,l],terrain,'Continuous original ground/water/cavern surface partitioned below native model-radius limit',[],'Visual datum only; future terrain height/collision authoring required; geology is nonwalkable shoulder')
    assets[-1]['world_anchor']=[minx,minz]
 bpy.ops.wm.save_as_mainfile(filepath=str(a.output/'calibrated-kit.blend'))
 (a.output/'kit-manifest.json').write_text(json.dumps(dict(blender=bpy.app.version_string,origin='Entirely original geometry; measured reference analysis only; no proprietary inputs',iteration=a.iteration,palette=COLORS,assets=assets),indent=2));print('CALIBRATED_KIT_SOURCE_PASS',len(assets))
if __name__=='__main__':main()
