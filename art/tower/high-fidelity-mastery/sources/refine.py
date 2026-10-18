"""Sustained refinement of the preserved Sol assets. No cache mesh inputs."""
import sys,json,math,time
from pathlib import Path
HERE=Path(__file__).resolve().parent
sys.path[:0]=[str(HERE.parent/'mesh-study'),str(HERE.parent/'asset-workbench')]
import bpy,bmesh
from mathutils import Vector
from mesh_tools import *
from native_codec import model
import author as baseline

def palette():
    M=baseline.palette()
    for k,rgb in {'hide':(.31,.35,.30),'belly':(.48,.43,.30),'plate':(.30,.24,.19),'horn':(.64,.58,.42),'eye':(.76,.44,.13),'black':(.06,.075,.065),'fold':(.22,.25,.21),'edgeplate':(.43,.35,.25)}.items():M[k]=material(k,rgb)
    return M

def cage(name,rows,mat,group=0,sides=10,subdiv=1):
    if subdiv:
        mix=lambda a,b,t:tuple(a[i]*(1-t)+b[i]*t for i in range(5))
        rows=[rows[0],mix(rows[0],rows[1],.08),*rows[1:-1],mix(rows[-2],rows[-1],.92),rows[-1]]
    o=tube(name,[r[:3]for r in rows],[r[3:]for r in rows],mat,sides,group)
    if subdiv:
        m=o.modifiers.new('Editable surface-cage refinement','SUBSURF');m.levels=subdiv;m.render_levels=subdiv;apply(o,m)
    return o

def union(objects,name,mat,voxel=2.5,ratio=.3):
    o=remesh_union(objects,name,mat,voxel,ratio)
    m=o.modifiers.new('Final contour relaxation','SMOOTH');m.factor=.6;m.iterations=3;apply(o,m);return o

def scale_shell(name,center,width,length,height,mat,group=0):
    x,y,z=center
    vs=[(x,y-length*.48,z+height),(x-width*.5,y-length*.4,z),(x-width*.46,y+length*.28,z-2),(x,y+length*.5,z+height*.55),(x+width*.46,y+length*.28,z-2),(x+width*.5,y-length*.4,z),(x,y,z-4)]
    o=mesh(name,vs,[(0,1,2,3),(0,3,4,5),(1,6,2),(2,6,3),(3,6,4),(4,6,5),(5,6,1),(0,5,1)],mat,group);return bevel(o,1,1)

def fit_shell(o,body,center_z):
    bm=bmesh.new();bm.from_mesh(o.data);bmesh.ops.triangulate(bm,faces=list(bm.faces));bmesh.ops.subdivide_edges(bm,edges=list(bm.edges),cuts=1,use_grid_fill=True);bm.to_mesh(o.data);bm.free()
    inverse=body.matrix_world.inverted();direction=inverse.to_3x3()@Vector((0,0,-1))
    for v in o.data.vertices:
        p=o.matrix_world@v.co;hit,at,_,_=body.ray_cast(inverse@Vector((p.x,p.y,180)),direction)
        if hit:
            height=(body.matrix_world@at).z;v.co.z=height+(p.z-center_z)+2

def creature(M,version):
    # The torso, shoulder and neck read as a continuous animal; moving limbs
    # meet inside overlapping anatomical shoulders and elbow/ankle shells.
    final=version!='study'
    core=[ellipsoid('rib_mass',(0,-1,64),(35,53,31),M['hide'],segments=20,rings=12),ellipsoid('pelvic_mass',(0,43,57),(29,31,29),M['hide']),ellipsoid('shoulder_mass',(0,-35,66),(36,28,34),M['hide']),cage('neck_mass',[(0,-36,70,25,24),(0,-53,76,22,22),(0,-67,79,18,17)],M['hide'])]
    body=union(core,'torso_neck_sculpt',M['hide'],2.8,.23)
    body['height_color']=False
    # Broad ventral field follows actual surface triangles rather than random noise.
    belly=M['belly'];fold=M['fold'];body.data.materials.append(belly);body.data.materials.append(fold)
    for p in body.data.polygons:
        c=body.matrix_world@p.center
        if c.z<45 and abs(c.x)<26:p.material_index=1
        elif c.z<54 and abs(c.x)>25:p.material_index=2
    tail=cage('muscular_tail',[(0,57,52,18,17),(0,73,43,16,14),(5,95,29,11,10),(16,119,17,7,6),(32,139,13,3,3),(45,146,17,.6,.6)],M['hide'],14,12,1)
    # Planned cranial proportions: cheek breadth, low frontal wedge, heavy nasal ridge.
    skull=[ellipsoid('braincase',(0,-75,83),(25,25,20),M['hide'],1,20,12),cage('muzzle_cage',[(0,-77,78,24,14),(0,-96,74,27,12),(0,-116,69,22,10),(0,-128,69,17,8)],M['hide'],1,12,1)]
    head=union(skull,'cranial_sculpt',M['hide'],2,.28);head['native_group']=1
    # Lower jaw is a modeled hinged mandible, not a colored strip.
    jaw=cage('hinged_mandible',[(0,-79,63,20,7),(0,-99,59,25,7),(0,-119,61,20,5),(0,-128,64,15,3)],M['belly'],15,10,1)
    for side in [-1,1]:
        # Deep socket, a small amber eye, and eyelids that wrap its visible edge.
        socket=ellipsoid('orbital_socket',(side*23,-85,86),(5,10,7),M['black'],1,12,8)
        ellipsoid('amber_iris',(side*30,-88,87),(2.8,4,3.5),M['eye'],1,12,8)
        ellipsoid('vertical_pupil',(side*32.4,-89,87),(1.1,1.1,2.5),M['black'],1,8,6)
        cage('upper_eyelid',[(side*22,-96,89,2,2),(side*27,-90,92,2,2),(side*26,-81,92,2,2)],M['plate'],1,8,1)
        cage('lower_eyelid',[(side*23,-95,84,1.5,1.5),(side*27,-88,82,1.5,1.5),(side*25,-82,84,1.5,1.5)],M['hide'],1,6,1)
        cage('zygomatic_ridge',[(side*19,-69,84,8,8),(side*27,-77,90,8,7),(side*28,-92,84,4,5)],M['plate'],1,8,1)
        cage('rooted_swept_horn',[(side*18,-71,94,8,8),(side*23,-64,105,7,7),(side*31,-51,115,4.5,4.5),(side*33,-37,116,2,2),(side*29,-27,111,.6,.6)],M['horn'],1,10,1)
        ring('horn_root',(side*18,-71,94),8,3,5,M['plate'],'Y',12)['native_group']=1
        ellipsoid('nostril',(side*11,-126,74),(4,2,2.3),M['black'],1,10,6)
        # The seam is attached to the jaw; teeth open with the mandible.
        cage('labial_seam',[(side*18,-82,64,1.3,1.3),(side*24,-99,63,1.5,1.5),(side*18,-121,65,1.3,1.3)],M['black'],15,6,0)
        if final:
            for i in range(3):
                y=-91-i*10;x=side*(22-i*2)
                cage('jaw_tooth',[(x,y,64,2.5,2.5),(x,y-1,69-i*.3,1.7,1.7),(x,y-2,72-i*.5,.5,.5)],M['horn'],15,6,0)
            scale_shell('nasal_shield_'+str(side),(side*10,-105,81),18,24,3,M['plate'],1)
    # Grouped anatomical limb sections support actual classic gait, with joints
    # protected by a small natural elbow/hock overlap rather than a hard cut.
    limbs=[]
    for side in [-1,1]:
        for rear in [False,True]:
            idx=(0 if side<0 else 2)+int(rear);up=2+idx*3;low=up+1;foot=up+2
            hip=(side*27,42 if rear else -34,60 if rear else 67)
            knee=(side*43,46 if rear else -26,32 if rear else 38)
            ankle=(side*40,33 if rear else -47,10)
            upper=cage('leg_%d_upper'%idx,[(*hip,17 if rear else 15,20),((hip[0]+knee[0])/2,(hip[1]+knee[1])/2,50,16,17),(*knee,10,12)],M['hide'],up,12,1)
            shin=cage('leg_%d_shin'%idx,[(*knee,11,12),(side*44,47 if rear else -33,25,9,10),(*ankle,7,7)],M['hide'],low,10,1)
            # The overlapping tapered sections form the joint. Avoid a separate
            # spherical knee cap, which made the animal read as an articulated toy.
            ellipsoid('leg_%d_flexor'%idx,knee,(8,10,8),M['hide'],low,10,6)
            x,y,z=ankle;cage('leg_%d_palm'%idx,[(x,y+8,9,9,6),(x,y-1,7,13,6),(x,y-10,5,11,4)],M['hide'],foot,10,1)
            for digit in [-1,0,1]:
                xx=x+digit*8
                cage('leg_%d_toe_%d'%(idx,digit),[(xx,y-6,7,4.5,4.5),(xx+digit,y-17,5,4,4),(xx+digit*2,y-24,4,2.8,2.8)],M['hide'],foot,8,1)
                cage('leg_%d_claw_%d'%(idx,digit),[(xx+digit*2,y-21,6,3,3),(xx+digit*2,y-29,4,2,2),(xx+digit*3,y-34,2,.5,.5)],M['horn'],foot,6,0)
            if final:
                scale_shell('leg_%d_elbow_shield'%idx,(knee[0],knee[1]-3,knee[2]+8),14,17,4,M['plate'],low)
                scale_shell('leg_%d_wrist_shield'%idx,(x,y-2,14),17,15,3,M['plate'],foot)
            limbs.append(dict(index=idx,hip=[hip[0],-hip[2],hip[1]],knee=[knee[0],-knee[2],knee[1]],ankle=[x,-z,y],upper=up,lower=low,foot=foot))
    # Staggered carapace overlaps follow the torso surface, so no floating plates.
    for i,y in enumerate([-49,-32,-15,2,19,36,53]):
        origin=body.matrix_world.inverted()@Vector((0,y,180));direction=body.matrix_world.inverted().to_3x3()@Vector((0,0,-1));hit,pos,_,_=body.ray_cast(origin,direction)
        z=(body.matrix_world@pos).z+4 if hit else [95,98,95,91,87,85,80][i];w=[30,35,37,35,32,29,25][i]
        shell=scale_shell('central_dorsal_%d'%i,(0,y,z),w,25,7 if i<3 else 5,M['plate']);fit_shell(shell,body,z)
        if final:
            for side in [-1,1]:
                shell=scale_shell('lateral_scute_%d_%d'%(side,i),(side*(w*.55),y+4,z-12),w*.65,18,3,M['edgeplate']);fit_shell(shell,body,z-12)
    if final:
        for i in range(3):scale_shell('tail_scute_'+str(i),(2+i*5,75+i*16,56-i*10),19-i*4,17,4,M['plate'],14)
    return limbs

def architecture(M):
    baseline.FINAL=True;baseline.arch(M,True)
    # Preserve structural silhouette; masonry variety follows actual blocks.
    shades=[material('Stone batch '+str(i),(.38+i*.018,.39+i*.016,.35+i*.012),True)for i in range(5)]
    for i,o in enumerate(list(bpy.context.scene.objects)):
        if o.type=='MESH'and ('jamb_'in o.name or 'voussoir_'in o.name):
            o.data.materials.clear();o.data.materials.append(shades[(i*7)%5])
            # Local top edge chips, bounded below mortar size.
            for v in o.data.vertices:
                if v.co.x>20 and v.co.z>18:v.co.x-=.8*abs(math.sin(v.co.z*.19))
    for s in [-1,1]:
        x=s*132
        block=box('springer_iron_seat',(x,-47,188),(72,5,11),M['iron'],1)
        for xx in [x-24,x+24]:tube('seat_rivet',[(xx,-51,188),(xx,-55,188)],[3,3],M['steel'],6)
        # Low drainage and foundation course communicate a used wet workshop.
        box('footing_return',(s*169,2,12),(18,113,22),M['stone'],2)

def machine(M):
    baseline.FINAL=True;baseline.pump(M,True)
    # Make the two cylinder connections and pressure instrumentation explicit.
    for y in [-28,28]:
        ring('packing_gland',(27,y,73),12,5,8,M['copper'],'X',16)
        for z in [67,79]:tube('gland_tie',[(16,y-13,z),(32,y-13,z)],[2,2],M['steel'],6)
    tube('gauge_stem',[(62,28,125),(62,7,142),(62,-8,142)],[3,3,3],M['copper'],10)
    ring('gauge_bezel',(62,-10,142),12,4,5,M['iron'],'Y',20)
    tube('gauge_face',[(62,-12,142),(62,-14,142)],[9,9],M['horn'],20)
    tube('gauge_pointer',[(62,-15,142),(66,-15,148)],[1.4,1.4],M['black'],4)
    for a in [-.8,0,.8]:
        x=62+8*math.sin(a);z=142+8*math.cos(a);tube('dial_tick',[(x,-15,z),(x*.0+62+6*math.sin(a),-15,142+6*math.cos(a))],[.8,.8],M['black'],4)
    tube('drain_tap',[(85,-28,51),(85,-43,51),(85,-43,44)],[3,3,3],M['copper'],8)
    tube('tap_key',[(78,-42,54),(91,-42,54)],[2,2],M['iron'],6)
    for y in [-48,48]:
        tube('bearing_oiler',[(-69,y,102),(-69,y,111)],[3,3],M['copper'],8)

def main():
    kind,version,destination=sys.argv[sys.argv.index('--')+1:];out=Path(destination).resolve();out.mkdir(parents=True,exist_ok=True);start=time.time()
    bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False);M=palette();limbs=None
    if kind=='creature':limbs=creature(M,version)
    elif kind=='architecture':architecture(M)
    elif kind=='machine':machine(M)
    if kind=='creature':
        for o in list(bpy.context.scene.objects):
            if o.type=='MESH'and len(o.data.polygons)>180:
                dec=o.modifiers.new('Native delivery simplification','DECIMATE');dec.ratio=.62;dec.use_collapse_triangulate=True
    for o in list(bpy.context.scene.objects):
        if o.type=='MESH':uv(o)
    g=export([o for o in bpy.context.scene.objects if o.type=='MESH'],out,model)
    if limbs:(out/'limbs.json').write_text(json.dumps(limbs,indent=2))
    bpy.ops.wm.save_as_mainfile(filepath=str(out/'source.blend'))
    (out/'authoring.json').write_text(json.dumps(dict(asset=kind,version=version,blender=bpy.app.version_string,seconds=time.time()-start,base='9492a2d',original=True),indent=2));print(kind,len(g['vertices']),len(g['faces']))
if __name__=='__main__':main()
