"""Three original-only native-rendered design studies; never a live map package.
Reuses the preserved architecture/machinery. New geometry is scene construction,
not a new asset-library expansion. No NPC authoring or cache geometry is involved.
"""
from pathlib import Path
import sys,json,math,random,struct,zlib,hashlib
import bpy
from mathutils import Matrix
T=Path(__file__).resolve().parent;S=T.parents[2];P=S/'art/tower/cinderworks-design-correction';LIB=S/'art/tower/cinderworks-environment';R=S.parent/'cinderworks-design-correction-private'
sys.path[:0]=[str(T.parent/'mesh-study'),str(T.parent/'asset-workbench')]
from mesh_tools import box,mesh,tube,ring,material,export
from native_codec import model
catalog=json.loads((LIB/'asset-catalog.json').read_text(encoding='utf-8'));plan=json.loads((P/'floor-plan.json').read_text(encoding='utf-8'))
RNG=random.Random(2207);props=[];structural=[];mat={};NATIVE_BASE=240
def pos(x,z,h=240):return((x-24.5)*128,(z-24.5)*128,h)
def cube(name,x,z,h,sx,sz,sh,m,bevel=0):return box(name,pos(x,z,h),(sx*128,sz*128,sh),mat[m],bevel)
def beam(name,a,b,r,m):return tube(name,[pos(*a),pos(*b)],[r,r],mat[m],4)
def pipe(name,points,r=9):
 tube(name,[pos(*q)for q in points],[r]*len(points),mat['patina'],12)
 for i,p in enumerate(points[1:-1]):
  # Sleeve positions follow actual duct axis; no random disconnected fittings.
  a,b=points[i],points[i+1];axis='X'if abs(b[0]-a[0])>abs(b[1]-a[1])else'Y';ring(name+' flange',pos(*p),r+5,5,9,mat['copper'],axis,12)
def wall(points,height=210,thickness=.42,base=240):
 for a,b in zip(points,points[1:]):
  length=math.dist(a,b);ang=math.atan2(b[1]-a[1],b[0]-a[0]);cx,cz=(a[0]+b[0])/2,(a[1]+b[1])/2
  obj=cube('thick mortar core',cx,cz,base+height/2,length,thickness,height,'mortar');obj.rotation_euler.z=ang
  rows=max(1,int(height/32));cols=max(1,math.ceil(length/.8))
  for row in range(rows):
   for col in range(cols):
    t=(col+.5)/cols;xx=a[0]+(b[0]-a[0])*t;zz=a[1]+(b[1]-a[1])*t
    obj=cube('bonded load-bearing ashlar',xx,zz,base+16+row*32,length/cols-.018,thickness+.04,29,'stone'+str((row+col)%3));obj.rotation_euler.z=ang
  obj=cube('weathered coping',cx,cz,base+height+5,length+.05,thickness+.15,12,'edge',1);obj.rotation_euler.z=ang
def edge(points,base=240):
 for a,b in zip(points,points[1:]):
  length=math.dist(a,b);o=cube('stone curb / dry walkway edge',(a[0]+b[0])/2,(a[1]+b[1])/2,base+6,length,.22,12,'edge',1);o.rotation_euler.z=math.atan2(b[1]-a[1],b[0]-a[0])
def rock(x,z,w=1.2,h=150):
 vs=[]
 for level,rad in [(190,1),(240+h*.45,.95),(240+h,.65)]:
  for i in range(7):
   a=i*math.tau/7;rad1=rad*(.84+RNG.random()*.22);vs.append(pos(x+math.cos(a)*w*rad1,z+math.sin(a)*w*rad1,level+RNG.random()*17))
 fs=[tuple(reversed(range(7))),tuple(range(14,21))]
 for row in range(2):
  for i in range(7):j=(i+1)%7;fs.append((row*7+i,row*7+j,(row+1)*7+j,(row+1)*7+i))
 mesh('fractured local slate seam',vs,fs,mat['rock'])
def grating(x,z,w=1,l=1,h=242):
 for i in range(int(w*10)+1):cube('bearing grate bar',x-w/2+i*.1,z,h,.035,l,5,'iron')
 for q in [-l/2,l/2]:cube('grate end bearing',x,z+q,h,w+.08,.065,7,'copper')
def paving(bounds,area,broken=False):
 x0,z0,x1,z1=bounds
 for iz in range(z0,z1):
  for ix in range(x0,x1):
   xx=ix+.5;zz=iz+.5
   if not area(xx,zz):continue
   width=.965;length=.965
   # A few worn joints belong to the collapsed retaining edge, not global noise.
   h=2 if broken and (ix+iz)%7==0 else 4
   cube('bonded '+('worn 'if broken else'')+'floor course',xx,zz,terrain_height('arrival-workshop',xx,zz)+h/2,width,length,h,'pave'+str((ix//3+iz)%3))
def coolant_vessel(x,z):
 # Reuse the successful pump's actual pressure-vessel meshes as a compact
 # coolant housing. This is study composition, not another standalone asset.
 with bpy.data.libraries.load(str(LIB/'assets/pump/source.blend'),link=False)as(a,b):b.objects=[q for q in a.objects if q in ['pressure_vessel','dome_collar','vessel_plug']]
 for o in b.objects:
  if o and o.type=='MESH':
   bpy.context.collection.objects.link(o);o.matrix_world=Matrix.Translation(pos(x+.5,z+.5,250))@Matrix.Scale(2.2,4)@Matrix.Translation((-31,-28,-149))@o.matrix_world;o['reused_component']='pump pressure vessel / source coolant position';o.name='retained coolant housing '+o.name
def prop(name,x,z,q=0,reason=''):
 props.append(dict(asset=name,x=x,z=z,quarter_turn=q,object_id=catalog[name]['object_id'],purpose=reason))
def inside(p,poly):
 x,y=p;v=False;j=len(poly)-1
 for i,a in enumerate(poly):
  b=poly[j]
  if(a[1]>y)!=(b[1]>y)and x<(b[0]-a[0])*(y-a[1])/(b[1]-a[1])+a[0]:v=not v
  j=i
 return v
def terrain_height(name,x,z):
 if name=='pressure-walk':
  if x>=74 and z<65:return 0
  # Explicit four-tile64unit rise on the dry external maintenance approach.
  if x<=67 and 60<=z<=65:return 240+max(0,min(64,(z-61)*16))
 return 240
def terrain_type(name,x,z):
 if name=='arrival-workshop':
  if x<36 and z>57:return 3
  if x<45:return 2 if z<55 else 3
  if x>61 and z>58:return 5
  return 1
 if name=='pressure-walk':
  if x>=74 and z<65:return 6
  if x>=76 and z>=67:return 7
  if x<=67:return 2
  return 3
 return 3 if 50<=x<=62 and 67<=z<=77 else 2
POLYS={
 'arrival-workshop':[[[32,48],[40,47],[48,51],[52,48],[61,49],[65,53],[63,61],[57,64],[49,62],[45,60],[36,63],[31,59]]],
 'pressure-walk':[[[63,58],[70,57],[77,59],[79,63],[78,69],[79,75],[76,79],[68,78],[64,74],[63,68]]],
 'warden-receiver':[[[47,66],[50,63],[61,63],[65,67],[66,76],[63,82],[49,82],[46,77],[46,70]]]
}
def arrival():
 # No internal rectangular room division. The low oblique threshold shares its
 # bearing wall with the sheltered mill; geology forms the workshop's far edge.
 wall([(51,50),(49,51),(48.8,53)],160);wall([(48.8,58),(50,61),(53,62)],95)
 wall([(53,62),(59,62),(62,59.5),(63,55),(61,51)],200)
 wall([(33,49),(39,48),(44,49),(46.5,51)],235)
 wall([(32.7,52),(32,57),(34,60),(36,61)],135)
 wall([(42,62),(46,60),(47,58.5)],105)
 edge([(50,53),(50,59),(53,61),(59,61),(61.5,58.5)])
 paving((49,51,62,62),lambda x,z:inside((x,z),[(51,51),(59,51),(62,55),(61,59),(58,61),(52,61),(49,57)]),True)
 # The three service objects remain explicit source contracts. These original
 # architectural envelopes locate them without publishing cache geometry.
 cube('bank counter oak carcass',53.5,60.5,278,.9,.8,76,'wood',2)
 cube('bank counter worn stone top',53.5,60.5,320,1,.92,10,'edge',2)
 for x in [53.22,53.77]:cube('bank iron corner strap',x,60.08,280,.065,.035,65,'iron')
 cube('altar load-bearing plinth',61,52.5,264,1.7,.8,48,'stone1',2);cube('altar offering slab',61,52.5,295,1.9,.92,15,'edge',2)
 ring('altar cold offering bowl',pos(61,52.5,309),27,7,9,mat['iron'],'Z',16)
 cube('plaque backing',54.5,51.5,310,.7,.14,100,'wood',2);cube('three-service instruction tablet',54.5,51.4,324,.61,.025,64,'copper')
 for i in range(3):cube('three-service engraved rule',54.5,51.38,308+i*15,.43,.012,3,'iron')
 # Load-bearing half-roof hugs the rear industrial edge, leaving the orbit view.
 for x,z in [(34,49),(39,49),(44,50)]:
  cube('timber post on stone shoe',x,z,420,.18,.18,360,'wood',1);cube('post shoe',x,z,255,.36,.36,30,'iron',1)
 beam('gallery wall plate',(34,49,600),(44,50,600),13,'wood')
 for x in [35,38,41,44]:
  beam('short roof bearing',(x,49.2,600),(x,51.2,560),11,'wood');beam('knee brace',(x,49.2,465),(x,50.1,565),7,'wood')
 for x in [34.5,35.25,36,36.75,37.5]:
  mesh('remaining slate roof / collapse edge',[pos(x,49.2,602),pos(x+.71,49.2,602),pos(x+.71,50.2,582),pos(x,50.2,582)],[(0,1,2,3)],mat['stone2'])
 # A collapsed retaining course points toward the ore route, not random rubble.
 for x,z,w,h in [(33,58,1,180),(34,60,1.4,220),(37,62,1.5,260),(43,62,1,170),(45,61,1.2,190)]:rock(x,z,w,h)
 for x,z in [(34.8,60.8),(35.8,61.5),(36.5,60.8)]:cube('retaining collapse explains tram dogleg',x,z,257,.6,.35,34,'stone1',1)
 beam('fallen retaining brace',(34,59.3,248),(36.8,60.6,265),10,'wood')
 prop('arch',47,55,3,'Oblique workshop entry, shared with the retaining gallery')
 prop('pier',47,54,0,'Arch bearing, outside two-tile opening');prop('pier',47,57,0,'Arch bearing')
 prop('plate-mill',40,55,0,'Rail terminates at the feed rollers; working aisle to its west')
 prop('rail-span',40,57,0,'Gauge-matched slate tram');prop('rail-span',40,59,0,'Gauge-matched slate tram')
 prop('ore-cart',40,59,0,'Loaded cart at upstream buffer');prop('slate-headwall',39,62,0,'Working strata and braced buffer')
 prop('workstation',37,53,0,'Bench faces the same safe maintenance aisle as the press')
 prop('tool-rack',36,54,0,'Tools belong to the adjacent bench');prop('stock-crate',44,50,0,'Metal stock under the roof edge')
 prop('furnace-bank',34,49,0,'Proposed visual envelope at the existing functional furnace; hook remains separate, exact final alignment pending')
 prop('arrival-inlay',56,56,0,'Arrival within the intimate landing');prop('hearth-ring',55,58,0,'Hearth foundation, exact retained center')
 prop('reward-surround',55,50,0,'Existing reward55,51 remains in the familiar landing')
 for x,z in [(61,60),(60,61),(44,62)]:prop('moss-fern',x,z,0,'Damp wall base / leak edge only')
 # A low, small original hearth is a study representation, not the functional cache object.
 ring('sanctuary cold brazier rim',pos(56,59,290),53,10,17,mat['iron'],'Z',20)
 for dx,dz in [(-.33,-.3),(.33,-.3),(0,.35)]:beam('brazier tripod',(56+dx,59+dz,244),(56+dx*.7,59+dz*.7,289),5,'iron')
 cube('charcoal bed',56,59,285,.62,.62,8,'soot')
 # Feed water is NOT randomly plumbed into the ore mill: this line is its roof
 # exhaust/ventilation chase, running upward into the forge hood.
 pipe('forge hood extraction',[(35.5,50,464),(35.5,49,464),(38,49,500),(41,49,500)],12)
 grating(46.8,54,1,.7)
def pressure():
 wall([(65,59),(64,63),(64.5,65)],145);wall([(66,66),(65.5,70),(66,74),(68,77)],110)
 wall([(68,77),(74,78),(77,76),(78,73),(77,68)],215)
 # The elevated maintenance strip remains outside the Imp's exact rectangle.
 edge([(65,59),(65,62),(66.4,65)],304)
 for z in [61.5,62.5,63.5]:cube('ramp tread / maintenance rise',65.7,z,terrain_height('pressure-walk',65.7,z)+3,1.3,.4,6,'edge')
 for x,z in [(66,61),(66,64),(67,66)]:
  cube('overhead main pier',x,z,440,.48,.48,400,'stone1',1);beam('service main saddle',(x-.32,z,646),(x+.32,z,646),9,'iron')
 pipe('continuous cooling supply from pump',[(69.9296875,60.859375,313),(69.9296875,60.859375,400),(69,61.3,660),(66,61.3,660),(66,65,660),(64,66,660)],12)
 pipe('dry return to headgate',[(64.5,67,650),(66.8,66.5,650),(68.8,64,650),(72.5,63,650),(75,62,400),(75,62,60)],9)
 # A lowered external ash/heat trench is visible beside the dry gallery. Bars
 # and masonry distinguish it; this adds no hazard behavior.
 for z in [68,69,70,71,72,73,74]:
  cube('trench ash base',77.3,z,165,1.1,1,26,'soot');cube('muted hot seam',77.3,z,181,.13,.8,4,'ember')
  for x in [76.7,77.9]:cube('trench bearing lip',x,z,245,.15,.95,12,'edge')
 for x,z in [(67,70),(67,74),(76,70),(76,74)]:grating(x+.5,z+.5,.75,.75,243)
 prop('pump',68,60,0,'Intake on the actual quay edge; supply route climbs to the pressure main')
 prop('sluice',74,61,0,'Headgate frame at the receiving water channel')
 prop('furnace-bank',76,69,1,'Retort sidewall, entirely outside Imp movement envelope')
 prop('furnace-bank',76,73,1,'Second exhaust retort, one continuous plant wall')
 prop('chimney-backdrop',77,77,0,'Retort flue carries exhaust above the gallery')
 prop('arch',71,66,0,'Imp approach threshold at the quiet south edge')
 for x,z in [(68,59),(67,60)]:prop('moss-fern',x,z,0,'Wet inlet edge only; the exhaust bay remains dry')
 prop('tool-rack',64,61,0,'Maintenance station beside supply saddle')
 # Source encounter placeholders are intentionally absent from artwork.
 ring('Imp spawn spatial datum',pos(72,72,242),38,2,1,mat['iron'],'Z',16)
def receiver():
 # Thick canted buttresses and a twelve-sided receiver crown wrap the complete
 # existing rectangular gameplay region; all set dressing stays outside it.
 wall([(48.4,67),(47,69),(47,76),(49,79),(52,80),(60,80),(64,78),(65,75),(65,69),(63,66)],260,.65)
 wall([(49.5,65),(53,64.5)],150);wall([(59,64.5),(62.5,65.5)],150)
 paving((50,67,63,78),lambda x,z:True)
 for x,z in [(48,69),(48,74),(50,79),(55,80),(60,79),(64,74),(64,69)]:
  cube('receiver buttress plinth',x,z,275,.85,.9,70,'stone1',2)
  cube('receiver structural rib',x,z,427,.52,.6,255,'stone2',1)
  cube('buttress cornice',x,z,565,.8,.82,20,'edge',2)
  beam('iron buttress tie',(x,z,300),(x,z,545),7,'iron')
 # Main receiver: a circular backed furnace mouth between the existing flanking
 # positions, raised outside the clear arena at z79.9; no central arena plinth.
 for i in range(12):
  a=i*math.tau/12;b=(i+1)*math.tau/12
  # A vertical arch ring in X/Z-up, its lower two segments omitted for the throat.
  if i in [8,9]:continue
  r=225;cx,cy,cz=pos(56,79.5,440)
  vs=[]
  for depth in [-38,38]:
   for rad in [r-37,r+37]:
    for t in [a+.012,b-.012]:vs.append((cx+rad*math.cos(t),cy+depth,cz+rad*math.sin(t)))
  mesh('segmented receiver voussoir',vs,[(0,1,3,2),(4,6,7,5),(0,4,5,1),(2,3,7,6),(0,2,6,4),(1,5,7,3)],mat['stone1'])
 cube('receiver dark back',56,80,440,3.1,.3,330,'soot')
 cube('receiver collecting trough',56,78.7,270,3.5,.65,52,'iron',2)
 cube('receiver trough worn rim',56,78.35,298,3.6,.13,12,'copper',1)
 for x in [54.7,55.1,55.5,55.9,56.3,56.7,57.1]:cube('banked furnace rear grate',x,79.7,370,.06,.12,166,'iron')
 for x in [54.9,55.6,56.3,57]:cube('banked coal accent',x,79.5,316,.17,.15,24,'ember')
 prop('arch',55,65,0,'Deliberate foundry threshold, two-tile opening')
 prop('pier',54,65,0,'Threshold bearing');prop('pier',58,65,0,'Threshold bearing')
 prop('chimney-backdrop',49,78,0,'Western flue: silhouette visible from the landing')
 prop('chimney-backdrop',63,78,0,'Eastern flue: matches the same receiver construction')
 for x in [49.5,63.5]:
  for i in range(6):cube('continued receiver flue course',x,78.5,667+i*32,.62,.62,30,'stone'+str(i%3))
  for h in [685,780,840]:cube('flue restraining band',x,78.5,h,.7,.7,8,'iron',1)
  cube('flue terminal cornice',x,78.5,855,.91,.91,20,'edge',2);cube('sooted receiver outlet',x,78.5,872,.59,.59,14,'soot',1)
 # These original banks stand in for preserved 18525/18526 presentation pairs;
 # they are a proposed visual swap, not a changed service/reward operation.
 prop('furnace-bank',50,78,0,'Flanking foundry bank outside the arena')
 prop('furnace-bank',59,78,0,'Flanking foundry bank outside the arena')
 pipe('coolant perimeter header',[(49,78.8,560),(52,80,560),(60,80,560),(63,78.8,560)],11)
 # Insets align to the actual four coolant tile anchors but leave standing tiles
 # and the entire encounter floor free. Low rings are visual datums only.
 for x,z in [(53,71),(53,74),(60,71),(60,74)]:
  coolant_vessel(x,z)
  ring('retained coolant position datum',pos(x+.5,z+.5,242),37,3,2,mat['patina'],'Z',16)
  # Floor channels terminate at the datum; no new blocking pipe crosses an arena.
  end=50 if x<56 else 63
  beam('flush coolant feeder',(x+.5,z+.5,240),(end,z+.5,240),2,'copper')
 ring('Warden spawn spatial datum',pos(56,72,241),55,2,1,mat['iron'],'Z',24)
 for x,z in [(47,67),(64,77),(49,80)]:rock(x,z,.45,65)

def make(name,index):
 global props,structural,mat,NATIVE_BASE
 props=[];bpy.ops.wm.read_factory_settings(use_empty=True)
 mat={k:material('Cinderworks '+k,rgb,flat)for k,rgb,flat in [('stone0',(.37,.365,.32),True),('stone1',(.42,.405,.35),True),('stone2',(.34,.34,.305),True),('pave0',(.37,.38,.35),True),('pave1',(.39,.39,.355),True),('pave2',(.36,.37,.34),True),('edge',(.50,.47,.39),True),('mortar',(.22,.24,.225),True),('iron',(.145,.175,.185),False),('copper',(.46,.295,.15),False),('patina',(.22,.35,.32),False),('wood',(.32,.255,.18),True),('rock',(.26,.295,.30),True),('soot',(.095,.11,.105),True),('ember',(.60,.235,.075),True)]}
 {'arrival-workshop':arrival,'pressure-walk':pressure,'warden-receiver':receiver}[name]()
 structural=[o for o in bpy.context.scene.objects if o.type=='MESH'];out=P/'studies'/name
 if out.exists():raise RuntimeError('Refusing to overwrite a developed study; create a versioned output')
 out.mkdir(parents=True);g=export(structural,out,model);g['vertex_skins']=None;(out/'geometry.json').write_text(json.dumps(g));(out/'architecture.model').write_bytes(model(g));(out/'asset.model').unlink()
 # Preserve an editable assembled source with linked mesh reuse of original art.
 for p in props:
  v=catalog[p['asset']];w,h=v['size'];w,h=(h,w)if p['quarter_turn']%2 else(w,h);base=terrain_height(name,p['x']+w/2,p['z']+h/2)
  center=pos(p['x']+w/2,p['z']+h/2,base)
  if p['asset']=='reward-surround':center=(center[0],center[1]+128,center[2])
  with bpy.data.libraries.load(str(LIB/v['source']/'source.blend'),link=False)as(a,b):b.objects=a.objects
  for o in b.objects:
   if o and o.type=='MESH':bpy.context.collection.objects.link(o);o.matrix_world=Matrix.Translation(center)@Matrix.Rotation(-p['quarter_turn']*math.pi/2,4,'Z')@o.matrix_world;o['reused_asset']=p['asset']
 bpy.context.scene['scope']='Original-only representative design scene; not a playable or approved level';bpy.ops.wm.save_as_mainfile(filepath=str((out/'study.blend').resolve()))
 rows=['# Original-only native scene design study; no live-map integration','S 64 64']
 colors={1:'68685a',2:'55564e',3:'3e4444',4:'655b48',5:'495c4b',6:'2c505b',7:'282d2d'}
 rows+=['P U '+str(5800+k)+' '+v for k,v in colors.items()]
 ground=[]
 for x in range(30,81):
  for z in range(31,84):
   if any(inside((x+.5,z+.5),poly)for poly in POLYS[name]):
    h=terrain_height(name,x,z);kind=terrain_type(name,x,z);rows.append(f'A {x} {z} {int(h/8)} {5801+kind} 0 0 0');ground.append([x,z,h,kind])
 rows.append(f'L 24 24 0 {64800+index} 10 0')
 for p in props:rows.append(f'L {p["x"]} {p["z"]} 0 {p["object_id"]} 10 {p["quarter_turn"]}')
 (out/'study.scene').write_text('\n'.join(rows)+'\n',encoding='utf-8');(out/'placements.json').write_text(json.dumps(props,indent=2)+'\n');(out/'ground.json').write_text(json.dumps(ground))
 (out/'study.json').write_text(json.dumps({'name':name,'structural_vertices':len(g['vertices']),'structural_triangles':len(g['faces']),'reused_original_instances':len(props),'new_scene_object_id':64800+index,'new_scene_model_id':1231000+index,'scope':'Static native-rendered composition only. Compound study shell is not a runtime collision object. Per-object decomposition and terrain/server integration require approval and independent native validation.','new_behavior':False,'npc_art':False},indent=2)+'\n')
 print('DEVELOPED_STUDY',name,len(g['vertices']),len(g['faces']),len(props),flush=True)
if __name__=='__main__':
 for i,name in enumerate(POLYS):make(name,i)
