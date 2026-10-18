"""Four independently authored static studies; no prior finished model imports."""
import sys,math,json,time
from pathlib import Path
HERE=Path(__file__).resolve().parent;sys.path[:0]=[str(HERE.parent/'mesh-study'),str(HERE.parent/'high-fidelity'),str(HERE.parent/'asset-workbench')]
import bpy
from mesh_tools import *
from refine import cage,union
from native_codec import model
FINAL=False

def palette():
    return {k:material(k,c,f)for k,c,f in [
        ('iron',(.245,.265,.265),False),('edge',(.44,.45,.41),False),('dark',(.12,.15,.15),False),('bronze',(.48,.30,.15),False),('green',(.22,.32,.28),False),
        ('wood',(.34,.24,.14),True),('endwood',(.42,.30,.18),True),('stone',(.40,.40,.35),True),('sheet',(.37,.40,.39),False),
        ('hide',(.24,.25,.21),False),('shell',(.34,.27,.19),False),('ridge',(.45,.36,.23),False),('joint',(.17,.19,.16),False),('horn',(.60,.53,.37),False),('eye',(.76,.39,.08),False),('black',(.045,.055,.045),False)]}
def box(name,c,s,m,b=2):return __import__('mesh_tools').box(name,c,s,m,b)
def loft(name,rows,m):
    vs=[(cx+sx*rx,sy*ry,z)for z,cx,rx,ry in rows for sx,sy in [(-1,-1),(1,-1),(1,1),(-1,1)]];n=len(rows);fs=[(0,3,2,1),tuple((n-1)*4+j for j in range(4))]
    fs += [(i*4+j,i*4+(j+1)%4,(i+1)*4+(j+1)%4,(i+1)*4+j)for i in range(n-1)for j in range(4)]
    return mesh(name,vs,fs,m)

def anvil(M,refined):
    # Visual reconstruction from the authentic rendered reference and coarse
    # visible dimensions. No cache vertex, topology or prior reconstruction input.
    metal=material('Reference-observed neutral forged iron',(.231,.221,.221))
    if not refined:
        box('foot',(-3,0,10),(66,50,20),metal,0);box('waist',(-3,0,30),(27,27,26),metal,0);box('head',(-3,0,51),(63,34,26),metal,0)
        tube('horn',[(28,0,53),(57,0,54)],[10,1],metal,6);box('heel',(-49,0,52),(14,33,24),metal,0)
    else:
        loft('continuous_foot_waist',[(0,-4,32,24),(17,-4,32,24),(23,-4,25,18),(28,-4,15,12),(40,-4,14,12)],metal)
        loft('striking_head',[(40,-4,32,16),(59,-4,32,16),(64,-4,30,14)],metal)
        box('heel_connector',(-40,0,48),(10,23,19),metal,0)
        loft('squared_heel',[(40,-50,6,17),(57,-50,6,17),(64,-48,6,14)],metal)
        cage('blunt_lifted_horn',[(26,0,55,9,9),(40,0,56,7,6),(56,0,57,4,4)],metal,sides=6,subdiv=0)

def workstation(M,refined):
    # A full framed plate-clamping/riveting bench: vise, screw, sheet and tools.
    for x in [-57,57]:
        for y in [-37,37]:
            box('bench_leg',(x,y,38),(20,20,76),M['wood'],2)
            if refined:box('iron_shoe',(x,y,8),(24,24,15),M['iron'],2)
    for i in range(4):box('top_plank',((i-1.5)*38,0,83),(36,111,17),M['wood'],2)
    for y in [-40,40]:
        box('apron',(0,y,66),(135,15,22),M['wood'],2)
        if refined:
            tube('diagonal_brace',[(-54,y,20),(45,y,64)],[6,6],M['wood'],4)
            for x in [-57,57]:
                box('corner_strap',(x,y,65),(24,19,9),M['iron'],1)
                tube('strap_pin',[(x,y-12,65),(x,y-17,65)],[3,3],M['edge'],6)
    box('vise_bed',(5,0,97),(109,70,11),M['iron'],3)
    box('fixed_jaw',(-25,0,116),(24,70,36),M['iron'],3)
    box('moving_jaw',(8,0,115),(21,63,34),M['iron'],3)
    box('fixed_cheek',(-12,-1,130),(6,65,10),M['edge'],1)
    box('moving_cheek',(-4,-1,130),(6,60,10),M['edge'],1)
    box('screw_boss',(54,0,116),(23,42,32),M['iron'],4)
    tube('vise_screw',[(16,0,116),(84,0,116)],[5,5],M['edge'],10)
    tube('cross_handle',[(84,-29,116),(84,29,116)],[3,3],M['iron'],8)
    for y in [-29,29]:ellipsoid('handle_knob',(84,y,116),(5,5,5),M['wood'],segments=10,rings=6)
    if refined:
        points=[(24+i,6*math.cos(i*math.tau/9),116+6*math.sin(i*math.tau/9))for i in range(48)]
        tube('screw_thread',points,[1.5]*len(points),M['dark'],4)
        box('clamped_plate',(-8,-1,135),(3,55,21),M['sheet'],.6)
        for y in [-18,0,18]:tube('plate_rivet',[(-11,y,138),(-5,y,138)],[2,2],M['bronze'],6)
        box('tool_tray',(-57,17,96),(27,67,9),M['endwood'],2)
        for y in [0,17,34]:tube('punch',[(-57,y,98),(-57,y,117)],[2.5,2.5],M['edge'],6)
        tube('hammer_handle',[(-40,-38,96),(3,-37,96)],[3.5,3.5],M['wood'],8)
        box('rivet_hammer',(-40,-38,98),(13,25,12),M['iron'],2)
        for x in [-25,30]:
            for y in [-24,24]:tube('bed_anchor',[(x,y,102),(x,y,107)],[3.2,3.2],M['bronze'],6)
        if FINAL:
            ring('screw_thrust_collar',(75,0,116),8,3,6,M['bronze'],'X',16)
            box('lower_stretcher',(0,0,24),(126,15,15),M['wood'],2)
            for x in [-57,57]:
                box('lower_crossrail',(x,0,24),(15,87,15),M['wood'],2)
                box('stretcher_tenon',(x,0,24),(12,28,19),M['endwood'],1)
            box('rivet_drawer',(0,-49,65),(48,6,15),M['endwood'],1)
            tube('drawer_pull',[(-9,-54,65),(9,-54,65)],[2,2],M['iron'],6)

def gear(name,c,r,width,teeth,m,phase=0):
    vs=[];n=teeth*4
    for x in [c[0]-width/2,c[0]+width/2]:
        for i in range(n):
            a=i*math.tau/n+phase;rad=r*(1 if i%4 in [1,2]else .84);vs.append((x,c[1]+rad*math.cos(a),c[2]+rad*math.sin(a)))
    fs=[tuple(reversed(range(n))),tuple(n+i for i in range(n))]+[(i,(i+1)%n,(i+1)%n+n,i+n)for i in range(n)]
    return mesh(name,vs,fs,m)

def machine(M,refined):
    # Two driven rolls, an actual feed gap and paired adjustment screws.
    box('mill_plinth',(0,0,8),(236,153,16),M['stone'],4)
    for x in [-67,67]:
        box('frame_foot',(x,0,27),(43,116,22),M['iron'],3)
        for y in [-37,37]:loft('cast_frame_post',[(30,x,15,12),(49,x,11,11),(143,x,11,11),(151,x,14,13)],M['green']).location.y=y
        box('crosshead',(x,0,150),(37,107,22),M['green'],4)
        for z in [65,109]:
            box('journal_block',(x,0,z),(32,42,31),M['iron'],3)
            ring('journal_bushing',(x,0,z),12,5,31,M['bronze'],'X',16)
        tube('pressure_spindle',[(x,0,123),(x,0,177)],[5,5],M['edge'],10)
        tube('adjustment_bar',[(x,-22,177),(x,22,177)],[3,3],M['iron'],8)
        if refined:
            points=[(x+6*math.cos(i*math.tau/8),6*math.sin(i*math.tau/8),134+i)for i in range(35)]
            tube('spindle_thread',points,[1.5]*len(points),M['dark'],4)
            for y in [-37,37]:
                tube('frame_bolt',[(x,y,161),(x,y,167)],[4,4],M['edge'],6)
                tube('base_bolt',[(x,y,37),(x,y,44)],[4,4],M['bronze'],6)
    for z in [65,109]:
        tube('polished_work_roll',[(-57,0,z),(57,0,z)],[20,20],M['edge'],24)
        tube('roll_axle',[(-102,0,z),(99,0,z)],[7,7],M['iron'],12)
        if refined:gear('meshing_drive_gear',(87,0,z),23,12,12,M['bronze'],math.pi/12 if FINAL and z==109 else 0)
    ring('operator_flywheel',(-97,0,65),44,8,9,M['iron'],'X',32 if refined else 12)
    for i in range(5):
        a=i*math.tau/5;tube('flywheel_spoke',[(-97,7*math.cos(a),65+7*math.sin(a)),(-97,40*math.cos(a),65+40*math.sin(a))],[3.5,3.5],M['iron'],6)
    tube('flywheel_hub',[(-107,0,65),(-89,0,65)],[11,11],M['iron'],12)
    if refined:
        tube('turning_handle',[(-98,-27,82),(-119,-27,82)],[4,4],M['wood'],10)
        box('feed_sheet',(0,-13,87),(94,179,2),M['sheet'],.4)
        for y in [-72,66]:
            tube('feed_support',[(-54,y,82),(54,y,82)],[4,4],M['iron'],8)
            for x in [-48,48]:tube('feed_tray_leg',[(x,y,16),(x,y,78)],[4,4],M['iron'],6)
        for x in [-67,67]:tube('oiler',[(x,-8,125),(x,-8,132)],[3,3],M['bronze'],6)
        box('mill_makers_plate',(68,-51,145),(19,3,11),M['bronze'],1)
        if FINAL:
            for x in [-67,67]:
                for y in [-37,37]:
                    box('guide_wear_strip',(x,y*.55,106),(32,4,51),M['edge'],1)
            for z in [65,109]:
                tube('axle_locknut',[(97,0,z),(106,0,z)],[10,10],M['iron'],6)

def shell(name,rows,m,sides=12):
    return cage(name,rows,m,sides=sides,subdiv=1)

def creature(M,refined):
    # Slagclaw: eight-legged furnace scorpion, independently designed. Its rigid
    # chitin joints are intentional anatomy, with a continuous cephalothorax.
    core=[ellipsoid('thoracic_volume',(0,-5,39),(34,45,25),M['hide'],segments=20,rings=12),ellipsoid('cranial_volume',(0,-40,37),(27,24,20),M['hide'],segments=20,rings=12)]
    body=union(core,'continuous_cephalothorax',M['hide'],2.5,.25)
    dorsal=shell('dorsal_shield',[(0,-53,44,17,9),(0,-39,52,29,15),(0,-13,54,35,18),(0,16,49,31,17),(0,35,42,25,14)],M['shell'])
    if FINAL:
        for v in dorsal.data.vertices:
            if v.co.z>47:
                envelope=min(1,max(0,(v.co.z-47)/12));v.co.z+=envelope*(2.4*math.cos((v.co.y+abs(v.co.x)*.35)*math.pi/14)-2*math.exp(-v.co.x*v.co.x/9))
        face=loft('frontal_clypeus',[(26,0,9,4),(38,0,18,7),(45,0,19,8)],M['shell']);face.location.y=-55
        rim=material('Carapace lateral rim',(.255,.22,.16));seam=material('Carapace median seam',(.235,.19,.14));dorsal.data.materials.append(rim);dorsal.data.materials.append(seam)
        for p in dorsal.data.polygons:
            c=p.center
            if c.z<51 or abs(c.x)>27:p.material_index=1
            elif abs(c.x)<4:p.material_index=2
        for side in [-1,1]:
            for i,y in enumerate([-30,-10,12]):
                x=side*(26 if i==0 else 31);z=59 if i<2 else 54
                cage('defensive_carapace_spine',[(x,y,z,4,4),(x+side*7,y+4,z+7,2.5,2.5),(x+side*10,y+6,z+10,.5,.5)],M['ridge'],sides=6,subdiv=0)
    for i in range(4):
        y=32+i*14;z=40-i*2;w=26-i*3
        shell('abdominal_tergite',[(0,y-7,z,w,15-i),(0,y+3,z+2,w,15-i),(0,y+10,z-2,w-3,11-i)],M['shell'])
    # Tail is a tapering series of overlapping chitin segments, with a curved telson.
    points=[(0,77,34),(0,94,47),(0,105,69),(0,102,94),(0,87,116),(0,65,127)]
    for i in range(5):
        a,b=points[i:i+2];r=11-i*.9;shell('tail_segment',[(a[0],a[1],a[2],r,r),((a[0]+b[0])/2,(a[1]+b[1])/2,(a[2]+b[2])/2,r+1,r+1),(b[0],b[1],b[2],r*.82,r*.82)],M['shell'])
        if FINAL and i>0:ellipsoid('tail_intersegment_membrane',a,(9-i*.7,9-i*.7,9-i*.7),M['joint'],segments=10,rings=6)
    shell('venom_telson',[(0,66,127,9,9),(0,48,129,9,10),(0,35,119,5,6),(0,32,105,1,1)],M['horn'])
    for side in [-1,1]:
        for i,(root_y,knee_y,tip_y)in enumerate([(-32,-49,-77),(-11,-15,-28),(12,29,34),(32,60,81)]):
            x1=side*27;x2=side*(57+i*4);x3=side*(84+i*4)
            shell('coxa',[(x1,root_y,39,8,9),(side*40,root_y,37,8,8),(x2,knee_y,32,6,7)],M['shell'],10)
            ellipsoid('flexible_leg_joint',(x2,knee_y,31),(7,7,7),M['joint'],segments=10,rings=6)
            shell('tibia',[(x2,knee_y,31,6,7),(side*(74+i*4),(knee_y+tip_y)/2,16,5,6),(x3,tip_y,4,3,4)],M['shell'],8)
            cage('tarsal_claw',[(x3,tip_y,4,3,3),(x3+side*9,tip_y-7,2,1.5,1.5),(x3+side*12,tip_y-12,1,.5,.5)],M['horn'],sides=6,subdiv=0)
        shell('pedipalp_arm',[(side*23,-40,33,10,10),(side*43,-59,28,10,10),(side*58,-82,24,9,9)],M['shell'])
        shell('pincer_palm',[(side*56,-79,25,9,9),(side*66,-96,26,15,11),(side*63,-111,25,12,9)],M['shell'])
        # Unequal opening angles create a deliberate left/right gesture.
        spread=5 if side<0 else 0
        shell('fixed_chela',[(side*55,-106,25,7,7),(side*43,-119,24,6,6),(side*39,-132,23,3,3),(side*48,-138,23,.7,.7)],M['ridge'],10)
        shell('movable_chela',[(side*72,-106,25,8,7),(side*(82+spread),-122,24,6,5),(side*69,-139-spread,23,3,3),(side*50,-142-spread,23,.7,.7)],M['shell'],10)
        # Mandibles meet around a recessed mouth; eyes are set into orbital brows.
        shell('mandible',[(side*15,-54,28,6,7),(side*12,-68,24,5,5),(side*5,-73,27,2,2)],M['horn'],8)
        ellipsoid('orbital_socket',(side*16,-50,51),(6,7,4),M['joint'],segments=12,rings=8)
        ellipsoid('amber_eye',(side*17,-55,53),(3,3,3),M['eye'],segments=10,rings=6)
        shell('orbital_brow',[(side*10,-47,55,3,3),(side*18,-49,58,4,3),(side*23,-44,55,2,2)],M['ridge'],8)
        if refined:
            for i in range(5):
                y=-33+i*14;z=65-abs(y+10)*.12
                if not FINAL:cage('carapace_rib',[(side*4,y,z+3,2.5,2.5),(side*19,y+2,z,3,3),(side*29,y+5,z-10,2,2)],M['ridge'],sides=6,subdiv=0)
            for y in [-119,-128]:
                cage('chela_tooth',[(side*44,y,25,2,2),(side*49,y-2,25,.6,.6)],M['horn'],sides=6,subdiv=0)
            for i in range(4):
                a,b=points[i+1:i+3];c=tuple(a[j]*.7+b[j]*.3 for j in range(3));ellipsoid('tail_suture',c,(10-i,6,5),M['joint'],segments=10,rings=6)
    ellipsoid('recessed_stoma',(0,-58,28),(8,6,4),M['black'],segments=10,rings=6)

def main():
    global FINAL
    kind,version,destination=sys.argv[sys.argv.index('--')+1:];out=Path(destination).resolve();out.mkdir(parents=True,exist_ok=True);start=time.time();bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
    FINAL=version.startswith('final')
    {'anvil':anvil,'workstation':workstation,'machine':machine,'creature':creature}[kind](palette(),version!='first')
    for o in list(bpy.context.scene.objects):
        if o.type=='MESH':
            if len(o.data.polygons)>(60 if kind=='creature'else 220):
                m=o.modifiers.new('Native surface delivery','DECIMATE');m.ratio=.42 if kind=='creature'else .56;m.use_collapse_triangulate=True
            uv(o)
    g=export([o for o in bpy.context.scene.objects if o.type=='MESH'],out,model)
    # Static models have no skeletal labels or classic animation payloads.
    g['vertex_skins']=None;(out/'geometry.json').write_text(json.dumps(g));(out/'asset.model').write_bytes(model(g))
    bpy.context.scene['provenance']='Original static modeling; no prior completed mesh inputs';bpy.ops.wm.save_as_mainfile(filepath=str(out/'source.blend'))
    (out/'authoring.json').write_text(json.dumps(dict(kind=kind,version=version,seconds=time.time()-start,static=True,prior_mesh_imports=False,protected_geometry_imports=False),indent=2));print('STATIC_ASSET',kind,version,len(g['vertices']),len(g['faces']))
if __name__=='__main__':main()
