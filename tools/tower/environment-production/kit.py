"""Original Cinderworks architecture and machinery, editable Blender sources.

Uses the preserved hard-surface authors for the arch, pump, mill and bench.
No creature author is called. Authoring axes X lateral, Y depth, Z up.
"""
from pathlib import Path
import sys,math,json,time,os
import bpy
T=Path(__file__).resolve().parent;sys.path[:0]=[str(T.parent/'mesh-study'),str(T.parent/'asset-workbench')]
from mesh_tools import box,tube,ring,ellipsoid,mesh,material,export
from native_codec import model
import author
import importlib.util
spec=importlib.util.spec_from_file_location('industrial',T.parent/'static-benchmark/designs.py');industrial=importlib.util.module_from_spec(spec);spec.loader.exec_module(industrial)
R=Path(os.environ.get('TOWER_CINDERWORKS_ASSET_OUTPUT',str(T.parents[3]/'cinderworks-environment-private')))
WALL_RUNTIME=False
ARCH_RUNTIME=False
UTILITY_RUNTIME=False
FINISHING=False

def palette():
    m=author.palette()
    for name,rgb,flat in [('stone',(.34,.36,.35),True),('edge',(.47,.46,.40),True),('dark',(.13,.16,.17),False),('iron',(.23,.27,.28),False),('steel',(.48,.51,.49),False),('copper',(.53,.31,.16),False),('patina',(.22,.37,.31),False),('wood',(.34,.26,.18),True),('soot',(.105,.115,.12),True),('moss',(.24,.34,.20),True),('ash',(.36,.34,.30),True),('ember',(.78,.32,.08),True)]:m[name]=material('Cinderworks '+name,rgb,flat)
    if FINISHING:
        m['stone']=material('Warm soot weathered masonry',(.39,.37,.325),True);m['edge']=material('Worn limestone coping',(.49,.465,.40),True)
    return m
def bolt(name,p,m,axis='Z',r=3):
    q=list(p);i={'X':0,'Y':1,'Z':2}[axis];q[i]+=5;tube(name,[p,q],[r,r],m['copper'],6)
def beam(name,a,b,m,r=7):return tube(name,[a,b],[r,r],m,4)
def wall(m,low=False):
    height=82 if low else 224;box('continuous mortar backing',(-61,0,height/2),(14,128,height),m['dark'])
    colors=[material('Slate batch '+str(i),(.29+i*.015,.29+i*.014,.275+i*.012)if FINISHING else(.31+i*.018,.33+i*.016,.33+i*.013),True)for i in range(4)]
    for row in range(2 if low else 6):
        z=18+row*34
        for col,(y,length)in enumerate([(-43,39),(0,43),(44,39)]if row%2==0 else[(-52,21),(-20,39),(22,41),(54,18)]):
            box(f'bonded ashlar {row}-{col}',(-60,y,z),(22,length,31),colors[(row+col)%4],0 if WALL_RUNTIME else 1.8)
    box('overhanging weathered coping',(-60,0,height-3),(30,127,12),m['edge'],2)
    if not low:
        box('iron tie plate',(-46,-2,121),(4,45,15),m['iron'],1)
        for y in [-15,14]:bolt('masonry tie',(-43,y,121),m,'X')
def pier(m):
    box('stepped footing',(0,0,10),(104,104,20),m['stone'],3)
    box('upper plinth',(0,0,28),(86,86,17),m['edge'],2)
    for i in range(5):box('coursed load bearing shaft',(0,0,54+i*33),(65,65,31),m['stone'],2)
    box('capital neck',(0,0,213),(75,75,13),m['edge'],2);box('abacus',(0,0,227),(96,96,18),m['stone'],2)
    for z in [73,172]:
        for y in [-35,35]:box('structural strap',(0,y,z),(68,4,11),m['iron'],1)
def arch(m):
    author.FINAL=True;author.arch(m,True)
    # 192-unit source opening becomes288: more than two complete walking tiles.
    for o in bpy.context.scene.objects:
        if o.type=='MESH':
            for v in o.data.vertices:v.co.x*=1.5
            o.location.x*=1.5
            if ARCH_RUNTIME:o.location.x+=64;o.location.y-=64
def furnace(m):
    for x in [-101,101]:
        box('furnace footing',(x,7,13),(76,140,26),m['edge'],3)
        for i in range(6):box('firebrick jamb',(x,5,44+i*34),(57,117,31),m['stone'],2)
    box('rear firebox wall',(0,47,115),(220,32,203),m['soot'],2)
    box('deep hearth',(0,-3,27),(154,124,24),m['dark'],2)
    for i in range(9):
        a=i*math.pi/9;b=(i+1)*math.pi/9;v=[(r*math.cos(t),y,177+r*math.sin(t))for t in[a,b]for y in[-53,55]for r in[73,112]]
        mesh('radial furnace voussoir',v,[(0,1,3,2),(4,6,7,5),(0,4,5,1),(2,3,7,6),(1,5,7,3),(0,2,6,4)],m['edge']if i==4 else m['stone'])
    for x in [-64,-32,0,32,64]:tube('hearth grate',[(x,-55,46),(x,30,46)],[4,4],m['iron'],6)
    for i in range(7):ellipsoid('banked coal',(i*18-54,-5,52),(13,30,7),m['soot'],segments=8,rings=4)
    # Tapered chimney sheet transitions into a supported flue, with real depth.
    mesh('riveted extraction hood',[(-129,-67,275),(129,-67,275),(129,75,275),(-129,75,275),(-52,-27,346),(52,-27,346),(52,42,346),(-52,42,346)],[(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7),(4,5,6,7)],m['iron'])
    box('upper flue',(0,8,386),(104,70,80),m['dark'],2)
    for x in [-117,117]:beam('hood knee brace',(x,37,222),(x,56,283),m['iron'],5)
    for x in [-96,-64,-32,0,32,64,96]:bolt('hood rivet',(x,-68,282),m,'Y')
def pipe(m,elbow=False):
    pts=[(0,-128,92),(0,128,92)]if not elbow else[(0,-128,92),(0,-20,92),(10,10,92),(40,20,92),(128,20,92)]
    tube('connected water main',pts,[11]*len(pts),m['patina'],12)
    for p in [pts[0],pts[-1]]:
        ring('bolted flange',p,17,7,8,m['copper'],'Y',12)
        for i in range(6):a=i*math.tau/6;bolt('flange bolt',(p[0]+15*math.cos(a),p[1]-6,p[2]+15*math.sin(a)),m,'Y',2)
    for y in [-75,75]:
        box('pipe saddle',(0,y,41),(46,26,82),m['stone'],2);ring('cradle strap',(0,y,92),14,4,11,m['iron'],'Y',12)
def rail(m):
    for y in [-112,-56,0,56,112]:box('sleeper',(0,y,6),(109,18,12),m['wood'],1)
    for x in [-48,48]:
        box('rail web',(x,0,15),(5,256,16),m['iron'],.6);box('running head',(x,0,25),(11,256,5),m['steel'],.5)
        for y in [-112,0,112]:bolt('rail spike',(x+9,y,14),m,r=2)
def cart(m):
    for x in [-48,48]:
        for y in [-58,58]:
            ring('flanged cart wheel',(x,y,22),22,9,10,m['iron'],'X',12);tube('wheel hub',[(x-9,y,22),(x+9,y,22)],[8,8],m['steel'],8)
    for y in [-58,58]:tube('wheel axle',[(-59,y,22),(59,y,22)],[5,5],m['iron'],8)
    box('cart sole',(0,0,48),(113,153,16),m['wood'],2)
    for x in [-56,56]:
        for z in [65,88,111]:box('cart side board',(x,0,z),(12,146,21),m['wood'],1)
        for y in [-62,62]:box('corner binding',(x+(-7 if x<0 else 7),y,84),(3,17,85),m['iron'],1)
    for y in [-70,70]:
        for z in [65,88,111]:box('cart end board',(0,y,z),(102,11,21),m['wood'],1)
    for i,(x,y,z)in enumerate([(-24,-31,109),(21,-24,115),(-16,25,121),(28,30,110)]):ellipsoid('loaded slate',(x,y,z),(28,33,20),m['stone'],segments=7,rings=4)
def quarry(m):
    for i in range(5):
        x=(i-2)*65;v=[(x-45,-48,0),(x+43,-48,0),(x+36,52,0),(x-42,51,0),(x-35,-15,164+i%3*27),(x+26,-8,175+i%2*22),(x+12,48,193+i%2*19),(x-31,45,173)]
        mesh('bedded slate headwall',v,[(0,3,2,1),(4,5,6,7),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7)],m['stone'])
        for z in [44,83,119]:beam('visible geological bedding',(x-35,-49,z),(x+34,-49,z+8),m['dark'],2)
    for x in [-178,178]:
        beam('rock support post',(x,-37,0),(x,-30,207),m['wood'],11);beam('headwall knee',(x,-35,120),(x+(-51 if x>0 else 51),-32,211),m['wood'],7)
    beam('headwall lintel',(-185,-30,215),(185,-30,215),m['wood'],13)
def railing(m):
    for x in [-122,0,122]:
        box('guard post',(x,0,38),(12,14,76),m['iron'],1);box('post shoe',(x,0,6),(25,28,12),m['stone'],1)
    for z in [28,69]:beam('guard rail',(-125,0,z),(125,0,z),m['iron'],4)
def sluice(m):
    for x in [-109,109]:box('headgate pier',(x,0,90),(43,70,180),m['stone'],3)
    box('headgate bridge',(0,0,191),(269,84,26),m['edge'],3)
    for x in [-75,-50,-25,0,25,50,75]:box('lifted gate slat',(x,13,132),(23,17,93),m['wood'],1)
    for z in [104,157]:box('gate iron binding',(0,0,z),(187,8,13),m['iron'],1)
    tube('gate screw',[(0,0,164),(0,0,256)],[6,6],m['steel'],10);ring('headgate control wheel',(0,0,244),37,7,8,m['iron'],'Z',20)
    for i in range(5):a=i*math.tau/5;beam('wheel spoke',(0,0,244),(33*math.cos(a),33*math.sin(a),244),m['iron'],3)
def storage(m):
    box('crate base',(0,0,7),(93,83,14),m['wood'],1)
    for x in [-43,43]:
        for z in [25,44,63]:box('crate board',(x,0,z),(9,81,17),m['wood'],1)
    for y in [-37,37]:
        for z in [25,44,63]:box('crate end',(0,y,z),(80,9,17),m['wood'],1)
    for x in [-29,29]:box('iron crate strap',(x,0,74),(8,84,4),m['iron'],.5)
    for i in range(3):box('stored plate',(-19+i*20,0,73),(14,58,8+i*4),m['steel'],1)
def toolrack(m):
    for x in [-41,41]:box('rack upright',(x,6,70),(14,16,140),m['wood'],1);box('rack foot',(x,0,7),(25,63,14),m['wood'],1)
    for z in [62,125]:box('tool rail',(0,6,z),(101,15,13),m['wood'],1)
    for i in range(5):
        x=-32+i*16;tube('hanging tool shank',[(x,-6,39),(x,-6,118)],[3,3],m['iron'],6)
        if i%2:box('hammer head',(x,-6,114),(20,13,12),m['steel'],1)
        else:beam('tongs jaw',(x-7,-6,127),(x,-6,113),m['steel'],2);beam('tongs jaw',(x+7,-6,127),(x,-6,113),m['steel'],2)
def vegetation(m):
    for i in range(5):
        a=i*2.4;x=27*math.cos(a);y=25*math.sin(a);ellipsoid('moss bed',(x,y,5),(25,18,7),m['moss'],segments=7,rings=4)
        pts=[(x,y,3),(x+4,y,31+i*3),(x+10,y+3,55+i*2)];tube('fern stem',pts,[2,1.5,.6],m['wood'],5)
        for side in [-1,1]:
            for j in range(3):
                z=19+j*10;mesh('folded fern frond',[(x,y,z),(x+side*(22-j*4),y+4,z+7),(x+side*6,y+7,z+5),(x+side*5,y+2,z+2)],[(0,1,2),(0,3,1),(1,3,2),(2,3,0)],m['moss'])
def rubble(m):
    for i in range(7):ellipsoid('spalled masonry',((i%3-1)*24,(i//3-1)*23,9+(i%2)*7),(19+i%3*3,16,10+i%2*6),m['ash']if i%3 else m['stone'],segments=6,rings=3)
def floor_patch(m):
    if FINISHING:
        box('dark mortar bed',(0,0,-3),(256,256,6),m['dark'])
        batches=[material('Worn floor batch '+str(i),(.36+i*.013,.345+i*.012,.30+i*.01),True)for i in range(4)]
        for row in range(8):
            for col in range(4):
                box('bonded floor sett',(-96+col*64,-112+row*32,0),(61,29,4),batches[(row+col*3)%4])
        for x,y in [(-110,-109),(110,109)]:box('small wayfinding bond',(x,y,2),(17,9,2),m['copper'])
        return
    for x in [-64,64]:
        for y in [-64,64]:box('worn paving slab',(x,y,-2),(124,124,8),m['stone'],2)
    for x in [-112,112]:box('bronze guide bond',(x,0,3),(6,250,2),m['copper'],.5)
def reward_surround(m):
    for x in [-105,105]:
        box('reward alcove footing',(x,15,11),(50,70,22),m['edge'],2)
        for i in range(4):box('reward alcove jamb',(x,15,42+i*29),(37,54,27),m['stone'],1.5)
    box('completion lintel',(0,15,157),(257,68,29),m['edge'],2)
    ring('completion roundel',(0,-23,158),22,6,7,m['copper'],'Y',12)
    # The real reward chest remains a separate preserved functional object.

def water_return(m):
    # Relative to the size4x1 anchor(70,61): centre(72,61.5).
    # Matches pump at(68,61), centre(69,62), intake(119,-18,73).
    pts=[(-265,46,73),(-230,46,73),(-192,46,80),(128,46,80),(160,46,40),(160,46,-100)]
    if UTILITY_RUNTIME:pts=[(-265,-82,73),(-252,-82,73),(-252,-52,80),(128,-52,80),(160,-52,40),(160,-52,-256 if FINISHING else-100)]
    tube('continuous pump to water return',pts,[9]*len(pts),m['patina'],12)
    for x in [-215,-100,100]:
        y=-52 if UTILITY_RUNTIME else 46;ring('return flange',(x,y,80),16,7,9,m['copper'],'X',12)
        for i in range(6):a=i*math.tau/6;bolt('return flange stud',(x-7,y+13*math.cos(a),80+13*math.sin(a)),m,'X',2)
    for x in [-170,-50]:box('return pipe support',(x,-52 if UTILITY_RUNTIME else 46,34),(33,35,68),m['stone'],2)

def arch_foot(m):box('collision bearing stone footing',(0,0,5),(96,96,10),m['stone'],1)

def chimney(m):
    box('retort buttress footing',(0,0,15),(124,124,30),m['edge'],3)
    for i in range(9):
        w=110 if i<3 else 91 if i<7 else 80
        box('bonded flue masonry',(0,0,48+i*37),(w,w,34),m['stone'],1.5)
        if i in[2,5,7]:box('chimney iron bond',(0,0,49+i*37),(w+7,w+7,9),m['iron'],1)
    box('flue cornice',(0,0,387),(116,116,23),m['edge'],2);box('sooted terminal',(0,0,405),(79,79,15),m['soot'],1)
    for x in [-48,48]:beam('backstay brace',(x,43,33),(x,43,274),m['iron'],4)

def stone_ring(m,arrival=False):
    outer,inner=(112,59)if arrival else(163,108)
    for i in range(16):
        a=i*math.tau/16+.009;b=(i+1)*math.tau/16-.009
        vs=[(r*math.cos(t),r*math.sin(t),z)for z in[-3,4]for t in[a,b]for r in[inner,outer]]
        mesh('radial worn floor voussoir',vs,[(0,2,3,1),(4,5,7,6),(0,1,5,4),(2,6,7,3),(1,3,7,5),(0,4,6,2)],m['stone']if i%4 else m['edge'])
    ring('inlaid rim',(0,0,4),outer-5,3,2,m['copper'],'Z',32)
    if arrival:
        for side in [-1,1]:beam('arrival tongs inlay',(side*28,-25,3),(-side*20,27,3),m['iron'],3)

BUILDERS={'wall-high':lambda m:wall(m),'wall-low':lambda m:wall(m,True),'pier':pier,'arch':arch,'furnace-bank':furnace,'water-main':pipe,'pipe-elbow':lambda m:pipe(m,True),'rail-span':rail,'ore-cart':cart,'slate-headwall':quarry,'railing':railing,'sluice':sluice,'stock-crate':storage,'tool-rack':toolrack,'moss-fern':vegetation,'ash-rubble':rubble,'paving-bond':floor_patch,'reward-surround':reward_surround,'water-return':water_return,'arch-foot':arch_foot,
 'chimney-backdrop':chimney,
 'arrival-inlay':lambda m:stone_ring(m,True),'hearth-ring':stone_ring,
 'pump':lambda m:(setattr(author,'FINAL',True),author.pump(m,True)),
 'plate-mill':lambda m:(setattr(industrial,'FINAL',True),industrial.machine(industrial.palette(),True)),
 'workstation':lambda m:(setattr(industrial,'FINAL',True),industrial.workstation(industrial.palette(),True))}

if __name__=='__main__':
    names=sys.argv[sys.argv.index('--')+1:];names=names or list(BUILDERS);summary=[]
    version='kit-v1'
    if names==['runtime-walls']:WALL_RUNTIME=True;version='kit-v2';names=['wall-high','wall-low']
    if names==['refined-fit']:ARCH_RUNTIME=True;version='kit-v2';names=['arch','rail-span','water-return']
    if names==['utility-clearance']:UTILITY_RUNTIME=True;version='kit-v3';names=['water-return','chimney-backdrop']
    if names==['finishing']:WALL_RUNTIME=True;ARCH_RUNTIME=True;UTILITY_RUNTIME=True;FINISHING=True;version='kit-v4';names=['wall-high','wall-low','pier','arch','paving-bond','water-return','arrival-inlay','hearth-ring']
    for name in names:
        out=R/version/name
        if out.exists():raise RuntimeError('Preserved asset already exists: '+str(out))
        out.mkdir(parents=True);bpy.ops.wm.read_factory_settings(use_empty=True);started=time.monotonic();BUILDERS[name](palette())
        objects=[o for o in bpy.context.scene.objects if o.type=='MESH']
        for o in objects:
            o['original_cinderworks_component']=o.name
            for p in o.data.polygons:p.use_smooth=not bool(o.data.materials[p.material_index].get('native_flat',False))
        g=export(objects,out,model);g['vertex_skins']=None;(out/'geometry.json').write_text(json.dumps(g));(out/'asset.model').write_bytes(model(g))
        bpy.context.scene['asset_identity']=name;bpy.context.scene['scope']='Original static environment; no creature/animation/cache geometry';bpy.ops.wm.save_as_mainfile(filepath=str((out/'source.blend').resolve()))
        info={'name':name,'vertices':len(g['vertices']),'triangles':len(g['faces']),'seconds':time.monotonic()-started,'source':'Preserved improved authoring functions and original environment kit','native_units_per_tile':128,'static_only':True};(out/'authoring.json').write_text(json.dumps(info,indent=2));summary.append(info);print('KIT',name,info['triangles'],flush=True)
    catalog=R/version/'catalog.json';previous=json.loads(catalog.read_text())if catalog.exists()else[]
    catalog.write_text(json.dumps(previous+summary,indent=2)+'\n')
