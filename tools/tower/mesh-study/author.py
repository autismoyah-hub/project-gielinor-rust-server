"""Three original Cinderworks studies using evaluated Blender mesh operations."""
import sys,math,json,time
from pathlib import Path
HERE=Path(__file__).resolve().parent
sys.path[:0]=[str(HERE),str(HERE.parent/'asset-workbench')]
import bpy
from mesh_tools import *
from native_codec import model

def palette():
    return {n:material(n,c,f)for n,c,f in [
        ('stone',(.39,.40,.36),True),('edge',(.53,.51,.43),True),('dark',(.18,.21,.21),False),
        ('iron',(.27,.30,.30),False),('steel',(.48,.50,.46),False),('copper',(.48,.27,.14),False),
        ('patina',(.22,.37,.31),False),('wood',(.31,.22,.13),True),('soot',(.12,.13,.12),True),
        ('hide',(.34,.37,.25),False),('belly',(.48,.43,.29),False),('plate',(.30,.20,.13),False),
        ('horn',(.68,.61,.42),False),('eye',(.77,.43,.07),False),('black',(.045,.055,.04),False)]}

FINAL=False
UNION_ONLY=False

def arch(M,refined):
    # Load-bearing voussoirs, paired stepped jambs, deep reveal, distinct sill.
    box('threshold',(0,0,6),(354,120,12),M['stone'],3)
    for side in [-1,1]:
        x=side*132
        box(f'plinth_{side}',(x,0,19),(92,110,24),M['edge'],3)
        for row in range(5):
            box(f'jamb_{side}_{row}',(x,7,45+row*30),(66,82,28),M['stone'],2 if refined else 0)
            if refined:box(f'reveal_{side}_{row}',(side*99,-35,45+row*30),(12,16,28),M['edge'],2)
        box(f'impost_{side}',(x,-1,190),(86,104,18),M['edge'],3)
        if refined:
            # Tapering external buttress; sloped coping sheds water.
            mesh('buttress_'+str(side),[(side*a,y,z)for a,y,z in [(155,-46,25),(184,-46,25),(184,40,25),(155,40,25),(155,-30,171),(165,-30,171),(165,30,171),(155,30,171)]],[(0,1,2,3),(4,7,6,5),(0,4,5,1),(1,5,6,2),(2,6,7,3),(3,7,4,0)],M['stone'])
            for z in [66,139]:
                box('jamb_strap',(x,-43,z),(68,5,10),M['iron'],1)
                for xx in [x-24,x+24]:tube('strap_pin',[(xx,-47,z),(xx,-51,z)],[3,3],M['steel'],6)
    count=13 if refined else 9
    for i in range(count):
        a=i*math.pi/count+.012;b=(i+1)*math.pi/count-.012
        # Multiple radial/depth rings form actual molding rather than colored stripes.
        profile=[(96,-42),(96,43),(138,43),(138,-38),(132,-47),(115,-47),(112,-54),(103,-54)]if refined else [(96,-42),(96,43),(138,43),(138,-42)]
        vs=[]
        for angle in [a,(a+b)/2,b]:
            vs.extend([(r*math.cos(angle),d,190+r*math.sin(angle))for r,d in profile])
        n=len(profile);fs=[tuple(reversed(range(n))),tuple(2*n+j for j in range(n))]
        for k in range(2):
            for j in range(n):fs.append((k*n+j,k*n+(j+1)%n,(k+1)*n+(j+1)%n,(k+1)*n+j))
        o=mesh('voussoir_%02d'%i,vs,fs,M['edge']if i==count//2 else M['stone']);bevel(o,1,1)
    if refined:
        # Keystone bears a simple original tongs/ember relief, not an OSRS symbol.
        box('keystone_face',(0,-52,311),(25,12,38),M['edge'],3)
        for s in [-1,1]:tube('crossed_tongs',[(s*7,-61,302),(-s*6,-61,319)],[2,2],M['iron'],5)
        ellipsoid('ember_seal',(0,-63,324),(4,2,6),M['copper'],segments=8,rings=4)
        # Raised open grate occupies crown only; entrance remains traversable.
        tube('gate_tie',[(-90,21,218),(90,21,218)],[4,4],M['iron'],8)
        for x in [-72,-48,-24,0,24,48,72]:tube('grate_bar',[(x,23,214),(x,23,190+math.sqrt(92**2-x*x))],[3,3],M['iron'],6)
        for side in [-1,1]:
            tube('sconce_bracket',[(side*144,-44,137),(side*144,-73,137),(side*144,-73,149)],[3,3,3],M['iron'],6)
            box('lamp_body',(side*144,-73,161),(18,18,25),M['dark'],2)
            box('lamp_amber',(side*144,-83,161),(10,2,15),M['copper'],1)
            for z in [151,158,165,172]:box('lamp_louvre',(side*144,-85,z),(14,2,2),M['iron'])
        # Subtle chipped corners on separate stone components; bounded noise only.
        for o in list(bpy.data.objects):
            if o.type=='MESH' and ('jamb_'in o.name or 'voussoir_'in o.name):
                for v in o.data.vertices:
                    v.co.x+=.55*math.sin(v.co.z*.31+v.co.y*.27);v.co.z+=.4*math.sin(v.co.x*.23)
    if FINAL:
        stones=[material('Sandstone batch '+str(i),(.39+i*.012,.40+i*.010,.36+i*.007),True)for i in range(5)]
        for i,o in enumerate(list(bpy.data.objects)):
            if o.type=='MESH' and ('jamb_'in o.name or 'voussoir_'in o.name):o.data.materials.clear();o.data.materials.append(stones[(i*3)%5])
        # Narrow drainage channels through the sill, with worn raised tread.
        for x in [-73,73]:box('threshold_drain',(x,-8,12.5),(5,93,1),M['dark'])
        box('worn_tread',(0,-43,13),(125,24,3),M['edge'],1)

def pump(M,refined):
    box('stone_bed',(0,0,7),(224,154,14),M['stone'],4)
    for x in [-68,68]:box('oak_sleeper',(x,0,22),(38,140,20),M['wood'],2)
    box('cast_bed',(0,0,38),(204,120,16),M['iron'],3)
    for x in [-80,80]:
        for y in [-43,43]:tube('bed_bolt',[(x,y,45),(x,y,51)],[5,5],M['steel'],6)
    # Twin pump barrels share a crankshaft; guide rods lead into cylinder caps.
    for y in [-28,28]:
        tube('pump_barrel',[(26,y,73),(94,y,73)],[23,23],M['patina'],16)
        for x in [27,91]:
            ring('flange',(x,y,73),24,6,6,M['iron'],'X',16)
            if refined:
                for a in range(6):
                    theta=a*math.tau/6;yy=y+20*math.cos(theta);zz=73+20*math.sin(theta);tube('flange_stud',[(x-5,yy,zz),(x+5,yy,zz)],[2.5,2.5],M['steel'],6)
        tube('piston_rod',[(-27,y,73),(29,y,73)],[5,5],M['steel'],10)
        box('crosshead',(-27,y,73),(14,16,19),M['iron'],2)
        for dy in [-15,15]:tube('slide_rail',[(-48,y+dy,57),(31,y+dy,57)],[3,3],M['iron'],6)
        endpoint=(-57,y,105)if FINAL else (-67,y,90)
        tube('connecting_rod',[(-27,y,73),endpoint],[5,5],M['copper'],8)
        if FINAL:
            tube('crank_web',[(-69,y+6,88),(-57,y+6,105)],[8,7],M['iron'],10)
            tube('crank_pin',[(-57,y-4,105),(-57,y+9,105)],[5,5],M['steel'],10)
            ring('big_end',endpoint,7,4,9,M['copper'],'Y',12)
    for y in [-48,48]:
        box('bearing_pedestal',(-69,y,64),(24,20,41),M['iron'],3)
        ring('bearing_shell',(-69,y,88),12,7,20,M['copper'],'Y',16)
    tube('crank_axle',[(-69,-82,88),(-69,65,88)],[6,6],M['steel'],12)
    ring('flywheel',(-69,-72,88),58,12,14,M['iron'],'Y',40 if refined else 12)
    tube('hub',[(-69,-85,88),(-69,-58,88)],[13,13],M['iron'],12)
    for i in range(6):
        a=i*math.tau/6
        if refined:
            tube('swept_spoke',[(-69+13*math.cos(a),-72,88+13*math.sin(a)),(-69+33*math.cos(a+.15),-72,88+33*math.sin(a+.15)),(-69+54*math.cos(a+.2),-72,88+54*math.sin(a+.2))],[5,4,5],M['iron'],6)
        else:tube('spoke',[(-69,-72,88),(-69+54*math.cos(a),-72,88+54*math.sin(a))],[4,4],M['iron'],4)
    if refined:
        # Intake manifold, discharge riser, pressure dome and handwheel valve.
        tube('intake',[(96,-28,73),(113,-28,73),(119,-18,73),(119,30,73),(113,40,73),(97,40,73)],[7]*6,M['copper'],12)
        tube('discharge',[(62,28,94),(62,28,121),(56,28,132),(38,28,132),(31,28,141),(31,28,163)],[7]*6,M['patina'],12)
        if FINAL:
            tube('pressure_vessel',[(31,28,149),(31,28,172),(31,28,181),(31,28,184)],[17,17,12,2],M['patina'],20)
            tube('vessel_plug',[(31,28,184),(31,28,189)],[5,5],M['copper'],6)
        else:ellipsoid('pressure_dome',(31,28,164),(17,17,23),M['patina'])
        ring('dome_collar',(31,28,150),17,5,6,M['copper'],'Z',20)
        tube('valve_stem',[(61,27,117),(61,0,117)],[3,3],M['steel'],8)
        ring('handwheel',(61,-4,117),17,4,4,M['copper'],'Y',20)
        for i in range(4):
            a=i*math.pi/2;tube('valve_spoke',[(61,-4,117),(61+15*math.cos(a),-4,117+15*math.sin(a))],[2,2],M['copper'],5)
        box('makers_tablet',(70,-53,73),(21,3,13),M['copper'],1)
        for x in [63,69,75]:box('tablet_cut',(x,-55,73),(2,1,7),M['dark'])
        for x in [-77,77]:
            for y in [-39,39]:box('sleeper_strap',(x,y,33),(42,8,4),M['dark'],1)

def creature(M,refined):
    # Continuous muscle volumes and curved tail; no box/loft torso assembly.
    masses=[]
    masses.append(ellipsoid('ribcage',(0,0,61),(36,62,35),M['hide']))
    masses.append(ellipsoid('pelvis',(0,43,59),(32,34,30),M['hide']))
    masses.append(ellipsoid('shoulders',(0,-34,64),(39,29,38),M['hide']))
    masses.append(tube('neck',[(0,-40,64),(0,-64,72),(0,-79,79)],[24,22,18],M['hide'],16))
    masses.append(tube('tail',[(0,58,57),(2,83,45),(13,110,27),(31,130,18),(47,137,21)],[21,16,10,6,1],M['hide'],14))
    for s in [-1,1]:
        for y in [-36,42]:
            masses.append(ellipsoid('muscle',(s*30,y,49),(19,23,26),M['hide']))
            masses.append(tube('limb',[(s*31,y,52),(s*44,y+8,29),(s*43,y-8,12)],[15,11,8],M['hide'],12))
            masses.append(ellipsoid('palm',(s*43,y-13,9),(15,19,8),M['hide']))
    if refined or UNION_ONLY:body=remesh_union(masses,'continuous_sculpted_body',M['hide'],3.8 if FINAL else 2.7,.38 if FINAL else .28)
    else:body=None
    if body and refined:body['height_color']=True
    # Sculpted head is a separate anatomical boundary at the neck collar.
    headparts=[ellipsoid('cranium',(0,-82,82),(27,29,22),M['hide'],1),ellipsoid('muzzle',(0,-108,73),(24,26,12),M['hide'],1)]
    if refined or UNION_ONLY:
        head=remesh_union(headparts,'sculpted_skull',M['hide'],3 if FINAL else 2,.45 if FINAL else .4);head['native_group']=1
    jaw=ellipsoid('lower_jaw',(0,-101,63),(23,28,6),M['belly'],2,20,8)
    for s in [-1,1]:
        ellipsoid('socket',(s*23,-91,86),(4,8,5)if FINAL else (5,10,7),M['black'],1,12,8)
        ellipsoid('amber_eye',(s*26,-94,87),(2.5,3.5,3)if FINAL else (3,5,4),M['eye'],1,12,8)
        ellipsoid('slit_pupil',(s*28,-95,87),(1,1.2,2.5)if FINAL else (1.5,1.5,3.5),M['black'],1,8,6)
        tube('brow',[(s*18,-104,92),(s*26,-95,95),(s*25,-83,94)],[4,5,3],M['plate'],8,1)
        ellipsoid('nostril',(s*12,-127,77),(4,2,3),M['black'],1,8,6)
        tube('swept_cheek_horn',[(s*20,-79,87),(s*31,-70,100),(s*38,-59,107),(s*38,-49,104)],[8,6,3,0.8],M['horn'],10,1)
        if refined:
            tube('mandible_rim',[(s*20,-119,64),(s*25,-101,62),(s*18,-81,64)],[2,3,2],M['plate'],8,2)
            for y in [-36,42]:
                for i in [-1,0,1]:
                    x=s*43+i*9
                    if FINAL:
                        tube('digit',[(x,y-18,9),(x,y-28,7),(x,y-32,5)],[5,4,3],M['hide'],8)
                        tube('claw',[(x,y-30,6),(x,y-36,4),(x,y-40,3)],[3.5,2,0.6],M['horn'],8)
                    else:tube('toe',[(x,y-18,9),(x,y-30,7),(x,y-36,4)],[5,4,1],M['horn'],8)
            for y in [-20,0,20,40]:
                ellipsoid('flank_scute',(s*33,y,68),(5,9,10),M['plate'],0,8,6)
    # Overlapping curved dorsal plates; orientation follows the arch of the back.
    for i in range(7 if refined else 3):
        y=-47+i*(17 if refined else 32);z=90+8*math.cos((y+20)*.025)
        o=mesh('dorsal_plate',[(0,y-12,z+14),(-19,y-10,z-3),(-16,y+12,z-3),(0,y+16,z+8),(16,y+12,z-3),(19,y-10,z-3),(0,y,z-5)],[(0,1,2,3),(0,3,4,5),(1,6,2),(2,6,3),(3,6,4),(4,6,5),(5,6,1),(0,5,1)],M['plate']);bevel(o,1.3,1)

def main():
    global FINAL,UNION_ONLY
    args=sys.argv[sys.argv.index('--')+1:];kind,version,dest=args;out=Path(dest);out.mkdir(parents=True,exist_ok=True);start=time.time()
    FINAL=version=='final'
    UNION_ONLY=version=='union'
    bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False);M=palette()
    {'architecture':arch,'machine':pump,'creature':creature}[kind](M,version not in ['first','union'])
    objs=[o for o in bpy.context.scene.objects if o.type=='MESH']
    for o in objs:uv(o)
    g=export(objs,out,model)
    bpy.context.scene['original_design']=True;bpy.context.scene['method']='Blender mesh modeling, evaluated bevels; creature volume remesh and curvature decimation; explicit export lattice.'
    bpy.ops.wm.save_as_mainfile(filepath=str(out/'source.blend'))
    (out/'authoring.json').write_text(json.dumps(dict(kind=kind,version=version,elapsed_seconds=time.time()-start,original=True,external_geometry=False,blender=bpy.app.version_string),indent=2))
    print('MESH_STUDY',kind,version,len(g['vertices']),len(g['faces']))
if __name__=='__main__':main()
