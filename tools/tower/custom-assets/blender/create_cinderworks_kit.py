"""Original Fallen Cinderworks modular kit. Headless Blender; no external art."""
import argparse
import bpy
import bmesh
import json
import math
import sys
from pathlib import Path
from mathutils import Vector

PALETTE={
 'basalt':(.075,.095,.105,1),'slate':(.125,.165,.175,1),
 'stone':(.29,.285,.235,1),'edge':(.23,.26,.26,1),'bone':(.44,.40,.30,1),
 'iron':(.035,.045,.05,1),'brass':(.32,.18,.06,1),'copper':(.23,.095,.04,1),
 'verdigris':(.055,.18,.145,1),'water':(.015,.095,.13,1),'ripple':(.065,.24,.26,1),
 'ash':(.14,.125,.105,1),'ember':(.90,.15,.015,1),'molten':(1,.48,.05,1),
 'moss':(.065,.12,.055,1),'timber':(.13,.07,.035,1),'ore':(.23,.14,.10,1),
 'cloth':(.10,.22,.23,1),'soil':(.10,.075,.045,1),
}

def main():
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--layout',type=Path)
    a=p.parse_args(sys.argv[sys.argv.index('--')+1:]);a.output.mkdir(parents=True,exist_ok=True)
    bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
    mats={}
    for name,color in PALETTE.items():
        m=bpy.data.materials.new(name);m.diffuse_color=color;m.use_nodes=True;m.use_backface_culling=True
        bsdf=m.node_tree.nodes.get('Principled BSDF');bsdf.inputs['Base Color'].default_value=color;bsdf.inputs['Roughness'].default_value=.88
        mats[name]=m
    collection=[];catalogue=[]

    def finish(obj,material,bevel=0):
        obj.data.materials.append(mats[material]);bpy.context.view_layer.objects.active=obj
        bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
        if bevel:
            mod=obj.modifiers.new('Chisel edges','BEVEL');mod.width=bevel;mod.segments=1
            bpy.ops.object.modifier_apply(modifier=mod.name)
        bm=bmesh.new();bm.from_mesh(obj.data);bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bmesh.ops.triangulate(bm,faces=list(bm.faces));bm.to_mesh(obj.data);bm.free()
        for f in obj.data.polygons:f.use_smooth=False
        collection.append(obj);obj.select_set(False);return obj

    def block(pos,size,material='stone',bevel=.025,angle=0):
        bpy.ops.mesh.primitive_cube_add(size=1,location=pos);o=bpy.context.object;o.dimensions=size;o.rotation_euler[2]=angle
        return finish(o,material,bevel)

    def cylinder(pos,radius,depth,material='iron',axis=(0,0,1),vertices=12,radius2=None):
        if radius2 is None:
            bpy.ops.mesh.primitive_cylinder_add(vertices=vertices,radius=radius,depth=depth,location=pos)
        else:
            bpy.ops.mesh.primitive_cone_add(vertices=vertices,radius1=radius,radius2=radius2,depth=depth,location=pos)
        o=bpy.context.object;o.rotation_euler=Vector(axis).to_track_quat('Z','Y').to_euler()
        return finish(o,material)

    def beam(a,b,width,material='iron'):
        mid=tuple((a[i]+b[i])/2 for i in range(3));v=Vector(b)-Vector(a)
        o=block(mid,(width,width,v.length),material,.015);o.rotation_euler=v.to_track_quat('Z','Y').to_euler();return o

    def ring(pos,radius,tube,material='brass',axis=(0,0,1),segments=12):
        bpy.ops.mesh.primitive_torus_add(major_segments=segments,minor_segments=4,location=pos,major_radius=radius,minor_radius=tube)
        o=bpy.context.object;o.rotation_euler=Vector(axis).to_track_quat('Z','Y').to_euler();return finish(o,material)

    def arch(width=4,height=3.8,material='stone',broken=False):
        side=width/2-.48;rise=height-width/2
        for x in [-side,side]:
            block((x,0,.13),(.96,1,.26),'edge')
            course=(rise-.4)/3
            for n in range(3):block((x,0,.35+(n+.5)*course),(.72,.74,course-.04),material)
            block((x,0,rise-.06),(1.02,.98,.24),'bone')
            block((x,0,.70),(.78,.80,.09),'iron',.01)
        r0=width/2-.94;r1=width/2-.06
        for k in range(9):
            if broken and k in [2,3]:continue
            lo=k*math.pi/9+.012;hi=(k+1)*math.pi/9-.012
            v=[]
            for y in [-.38,.38]:
                for r,t in [(r0,lo),(r1,lo),(r1,hi),(r0,hi)]:v.append((r*math.cos(t),y,rise+r*math.sin(t)))
            faces=[(0,3,2,1),(4,5,6,7),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7)]
            mesh=bpy.data.meshes.new('voussoir');mesh.from_pydata(v,[],faces);mesh.update();o=bpy.data.objects.new('arch-stone',mesh);bpy.context.collection.objects.link(o)
            finish(o,'brass' if k==4 else material)

    def column(height=3):
        block((0,0,.15),(.95,.95,.3),'edge')
        step=(height-.55)/5
        for n in range(5):block((0,0,.30+(n+.5)*step),(.63,.63,step-.035),'stone')
        for z in [.8,height-.6]:block((0,0,z),(.69,.69,.10),'iron',.01)
        block((0,0,height-.1),(1.02,1.02,.25),'bone')

    def wall(height=2.6,diagonal=False,low=False):
        y=.39
        courses=2 if low else 5
        for n in range(courses):
            block((0,y,.24+n*(height-.3)/courses),(.98,.36,(height-.3)/courses-.035),'basalt' if n==0 else 'stone',.018)
        block((0,y,height-.08),(1.03,.46,.17),'edge',.025)
        if diagonal:
            for o in collection:o.rotation_euler[2]+=math.pi/4;o.location.x,o.location.y=-o.location.y/math.sqrt(2),o.location.y/math.sqrt(2)

    def asset(name,size,build,use,anchor=None):
        nonlocal collection
        collection=[]
        bpy.ops.object.select_all(action='DESELECT');build()
        for o in collection:o.select_set(True)
        bpy.context.view_layer.objects.active=collection[0];bpy.ops.object.join();obj=bpy.context.object;obj.name=name
        # Move the collection into a catalogue grid after export; GLB contains
        # only this selected original asset with its evaluated local geometry.
        bpy.ops.object.transform_apply(location=False,rotation=True,scale=True)
        # Joined origin inherits first object location. Bake to the common datum.
        for vertex in obj.data.vertices:vertex.co+=obj.location
        obj.location=(0,0,0)
        bpy.ops.export_scene.gltf(filepath=str(a.output/(name+'.glb')),export_format='GLB',use_selection=True,export_apply=True,export_yup=True,export_materials='EXPORT',export_cameras=False,export_lights=False)
        points=[tuple(v.co) for v in obj.data.vertices]
        catalogue.append(dict(name=name,footprint=size,vertices=len(points),triangles=len(obj.data.polygons),materials=[m.name for m in obj.data.materials],bounds=[[min(v[i] for v in points),max(v[i] for v in points)] for i in range(3)],purpose=use))
        if anchor is not None:catalogue[-1]['world_anchor']=anchor
        obj.location=((len(catalogue)-1)%8*10,(len(catalogue)-1)//8*11,0);obj.select_set(False)

    def floor(material='slate',seams=True):
        block((0,0,-.105),(1,1,.18),material,0)
        if material.startswith(('soil','ash','moss')):
            block((0,0,-.005),(1,1,.035),material,0)
        else:block((0,0,-.005),(.985,.985,.035),material,0)

    for name,material in [('paving','slate'),('sanctuary_paving','stone'),('forge_paving','ore'),('ash_paving','ash'),('garden_soil','soil')]:
        asset(name,[1,1],lambda m=material:floor(m),'Reusable original untextured floor; top at walking datum')
    asset('wall',[1,1],lambda:wall(),'Five-course dressed masonry with a continuous cap')
    asset('parapet',[1,1],lambda:wall(.85,low=True),'Low broken wall for player-camera visibility')
    asset('wall_broken',[1,1],lambda:(wall(1.4,low=True),block((-.22,.4,1.6),(.4,.36,.4),'stone',.04)),'Failed masonry and damaged service bays')
    asset('pillar',[1,1],lambda:column(3.1),'Collared structural pier matching the arch kit')
    asset('grand_pillar',[2,2],lambda:(column(4.4),block((0,0,.1),(1.6,1.6,.2),'basalt')),'Foundry buttress and entrance landmark')
    asset('arch',[4,1],lambda:arch(),'Open four-tile modular portal with two-tile passage')
    asset('broken_arch',[4,1],lambda:arch(broken=True),'Collapsed portal; missing crown-side voussoirs')
    asset('great_arch',[6,2],lambda:arch(6,5.6),'Monumental foundry portal; four-tile opening')

    def sanctuary():
        cylinder((0,0,.12),2.4,.24,'basalt',vertices=8);cylinder((0,0,.32),1.85,.34,'stone',vertices=8)
        cylinder((0,0,.53),1.3,.15,'water',vertices=12);ring((0,0,.58),1.43,.13,'verdigris')
        for k in range(4):
            t=k*math.pi/2;block((1.6*math.cos(t),1.6*math.sin(t),.75),(.46,.46,1.3),'bone',.05)
            cylinder((1.6*math.cos(t),1.6*math.sin(t),1.43),.25,.25,'brass',vertices=6)
        cylinder((0,0,1.3),.38,1.65,'verdigris',vertices=8,radius2=.22);cylinder((0,0,2.25),.3,.28,'brass',vertices=6)
    asset('restoration_font',[5,5],sanctuary,'Central fourfold restoration monument; water and brass identify the safe hub')

    def furnace():
        block((0,.2,.25),(3.8,2.7,.5),'basalt',.08)
        for x in [-1.4,1.4]:block((x,0,1.5),(.7,2,2.4),'stone',.06)
        block((0,.75,2.6),(3.6,1,1),'stone',.08);block((0,.81,1.35),(2.1,.2,1.6),'iron',.02)
        block((0,.65,1.4),(1.5,.16,1.15),'ember',.02)
        for x in [-.6,0,.6]:block((x,.48,1.4),(.09,.12,1.3),'iron',.01)
        cylinder((0,.7,3.4),.85,1.2,'basalt',vertices=8);ring((0,.7,3.4),.87,.09,'iron')
        block((0,-1.2,.10),(1.2,1.4,.2),'copper',.03)
    asset('furnace',[4,4],furnace,'Worker-scale smelting furnace, barred orange firebox and casting runnel')

    def crucible():
        block((0,.7,.35),(8,4,.7),'basalt',.10)
        for x in [-3.1,3.1]:
            for n in range(5):block((x,.9,.9+n*.95),(1.3,3.2,.88),'stone',.07)
            block((x,.9,5.8),(1.65,3.6,.4),'brass',.04)
        block((0,1.7,4.8),(6,1.4,1.0),'stone',.08)
        block((0,1.1,2.7),(4.6,.3,3.2),'iron',.02);block((0,.88,2.7),(3.9,.16,2.6),'ember',.01)
        for x in [-1.6,-.8,0,.8,1.6]:block((x,.7,2.7),(.16,.2,2.9),'iron',.01)
        cylinder((0,1.4,5.9),1.35,1.4,'basalt',vertices=8);ring((0,1.4,5.4),1.4,.13,'brass')
        block((0,-.55,.15),(2,2.5,.3),'copper',.04);block((0,-.65,.33),(1.2,2.4,.06),'molten',.01)
    asset('great_crucible',[8,6],crucible,'Final foundry furnace: massive buttresses, containment grille and casting throat')

    def wheel():
        for x in [-1.4,1.4]:
            block((x,0,1.4),(.45,1.3,2.8),'stone',.05)
            ring((x,0,1.5),1.6,.14,'timber',axis=(1,0,0),segments=16)
        cylinder((0,0,1.5),.18,3.7,'iron',axis=(1,0,0))
        for k in range(12):
            t=k*math.tau/12;y,z=1.5*math.cos(t),1.5+1.5*math.sin(t)
            block((0,y,z),(2.5,.38,.22),'timber',.015,0)
            beam((-1.34,.3*math.cos(t),1.5+.3*math.sin(t)),(-1.34,y,z),.09,'iron');beam((1.34,.3*math.cos(t),1.5+.3*math.sin(t)),(1.34,y,z),.09,'iron')
    asset('waterwheel',[4,4],wheel,'Original broad undershot wheel, shaft, spokes and stone bearing piers')

    def pump():
        block((0,0,.17),(1.9,1.6,.34),'basalt',.06);cylinder((0,0,1.25),.64,1.9,'verdigris',vertices=12)
        for z in [.45,1.6,2.15]:ring((0,0,z),.65,.07,'brass')
        cylinder((0,0,2.35),.42,.4,'iron');cylinder((0,-.76,1.4),.22,1,'copper',axis=(0,1,0))
        ring((0,-1.25,1.4),.44,.065,'brass',axis=(0,1,0));beam((-.35,-1.25,1.4),(.35,-1.25,1.4),.075,'iron')
    asset('pressure_pump',[2,3],pump,'Riveted coolant vessel with an identifiable hand valve')

    def pipe():
        cylinder((0,0,.6),.18,1.9,'copper',axis=(0,1,0));ring((0,-.72,.6),.2,.06,'iron',axis=(0,1,0));ring((0,.72,.6),.2,.06,'iron',axis=(0,1,0))
        for y in [-.65,.65]:block((0,y,.22),(.48,.24,.44),'basalt',.02)
    asset('pipe',[1,2],pipe,'Two-tile supply pipe with visible joints and supports')

    def bench():
        for x in [-.8,.8]:block((x,0,.55),(.2,.7,1.1),'timber',.02)
        block((0,0,1.15),(2.0,1,.2),'timber',.025);block((0,.12,1.36),(.75,.45,.25),'iron',.02)
        block((.65,0,1.34),(.35,.35,.2),'brass',.02);beam((-.65,-.25,1.28),(-.3,.2,1.28),.08,'iron')
    asset('workbench',[2,1],bench,'Readable repair bench with vise, workpiece and hand tool')

    def anvil():
        cylinder((0,0,.45),.55,.9,'timber',vertices=8);block((0,0,1.08),(.85,.65,.40),'iron',.05)
        cylinder((.65,0,1.1),.22,.7,'iron',axis=(1,0,0),vertices=6,radius2=.04)
    asset('anvil',[2,1],anvil,'Iron anvil on a worker-scale timber stump')

    def vent():
        cylinder((0,0,.12),.9,.24,'basalt',vertices=8);cylinder((0,0,.7),.52,1.1,'iron',vertices=8)
        ring((0,0,1.25),.55,.10,'brass',segments=8);cylinder((0,0,1.28),.36,.05,'ember',vertices=8)
        for x in [-.2,0,.2]:block((x,0,1.33),(.08,.9,.08),'iron',.01)
    asset('vent',[2,2],vent,'Containment vent; orange core and grating communicate supernatural heat')

    def rubble():
        for x,y,z,s,t in [(-.5,-.3,.18,.5,.2),(.25,.2,.28,.7,-.3),(.55,-.4,.13,.4,.6),(-.4,.4,.12,.35,.8)]:block((x,y,z),(s,s*.6,z*2),'stone',.045,t)
        beam((-.7,0,.14),(.6,.6,.28),.13,'iron')
    asset('rubble',[2,2],rubble,'Clustered fallen masonry with one broken tie; used selectively at damage sites')

    def ore():
        for x,y,r in [(-.45,-.2,.65),(.35,.1,.8),(.1,-.6,.4)]:
            bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=1,radius=r,location=(x,y,r*.5));o=bpy.context.object;o.scale=(1,.8,.75);finish(o,'basalt')
        for x,y,z in [(-.4,-.3,.7),(.3,-.1,.8),(.6,.3,.55)]:block((x,y,z),(.38,.17,.14),'ore',.02,.4)
    asset('ore_outcrop',[2,2],ore,'Original exposed stone with copper-red mineral seams')

    def crates():
        for x,y in [(-.45,-.2),(.45,.25)]:
            block((x,y,.4),(.75,.75,.8),'timber',.02)
            for z in [.15,.65]:block((x,y-.39,z),(.8,.07,.09),'iron',.005)
    asset('stores',[2,2],crates,'Banded workshop stores and resource handling')

    def timber():
        for x in [-.4,0,.4]:cylinder((x,0,.3),.18,1.8,'timber',axis=(0,1,0),vertices=8)
        for y in [-.55,.55]:block((0,y,.2),(1.4,.18,.2),'iron',.01)
    asset('timber_stack',[2,2],timber,'Dry timber stock justified by the workshop and maintenance stores')

    def roots():
        cylinder((0,0,1.25),.26,2.5,'timber',vertices=7,radius2=.18)
        for a in [0,2,4]:
            end=(math.cos(a)*.8,math.sin(a)*.8,2.9);beam((0,0,1.7),end,.14,'timber')
            bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=1,radius=.8,location=end);o=bpy.context.object;o.scale=(1,.9,.7);finish(o,'moss')
        for a in range(5):beam((0,0,.22),(math.cos(a)*.8,math.sin(a)*.8,.06),.12,'timber')
    asset('root_tree',[3,3],roots,'A small tree only for the breached, water-fed root courtyard')

    def fungus():
        for x,y,h in [(-.35,0,.55),(.3,.2,.8),(.1,-.4,.4)]:
            cylinder((x,y,h/2),.075,h,'bone',vertices=6);cylinder((x,y,h),.35,.22,'verdigris',vertices=8,radius2=.08)
    asset('fungus',[2,2],fungus,'Water-fed fungal life in the reservoir garden')

    def bridge(span=4):
        half=span/2-.2
        block((0,0,.12),(span,2,.24),'stone',.03)
        for y in [-.9,.9]:
            beam((-half,y,.9),(half,y,.9),.1,'iron')
            for i in range(span):block((-half+i*(2*half)/(span-1),y,.48),(.12,.12,.9),'iron',.01)
        for x in [-half,half]:block((x,0,-.35),(.4,2,.8),'basalt',.03)
    asset('bridge',[4,2],bridge,'Four-tile service bridge with open railings and visible supporting abutments')

    def water():
        block((0,0,-.19),(1,1,.08),'water',0)
        for y in [-.30,.12]:block((-.05,y,-.143),(.55,.035,.015),'ripple',0)
    asset('water',[1,1],water,'Original untextured lowered coolant surface with two restrained ripples')

    def brazier():
        cylinder((0,0,.35),.22,.7,'iron',vertices=8);cylinder((0,0,.8),.48,.35,'brass',vertices=8,radius2=.56)
        cylinder((0,0,1.1),.26,.5,'ember',vertices=5,radius2=.03);cylinder((.15,0,1.05),.16,.35,'molten',vertices=5,radius2=.02)
    asset('brazier',[1,1],brazier,'Warm route marker; fixed original geometry, no simulated fire')

    def banner():
        cylinder((-.4,0,1.7),.06,3.4,'iron',vertices=6);beam((-.4,0,3.2),(.8,0,3.2),.08,'brass')
        block((.18,.03,2.5),(1,.08,1.2),'cloth',0);block((.18,-.02,2.5),(.16,.04,.8),'brass',0)
    asset('banner',[2,1],banner,'Sanctuary identity: restrained teal field and brass seam')

    def scale():
        for x in [-.13,.13]:block((x,0,.38),(.17,.22,.75),'timber',.02)
        block((0,0,1.02),(.48,.28,.6),'cloth',.035);cylinder((0,0,1.57),.19,.30,'bone',vertices=8)
        for x in [-.34,.34]:block((x,0,1.0),(.14,.18,.60),'bone',.02)
    asset('scale_figure',[1,1],scale,'Original static1.72-tile scale mannequin; not a simulated NPC')

    def stair():
        cylinder((0,0,2.0),.55,4.0,'basalt',vertices=12)
        for k in range(12):
            t=k*math.pi/8;r=1.4;block((r*math.cos(t),r*math.sin(t),.12+k*.28),(1.7,.65,.20),'stone',.025,t)
            if k%2==0:block((2.1*math.cos(t),2.1*math.sin(t),.6+k*.28),(.16,.16,1),'iron',.01)
        ring((0,0,4.15),.75,.12,'brass')
    asset('ascent_stair',[6,6],stair,'Original spiral ascent landmark; future height/plane engineering required')

    def rail():
        for x in [-.4,.4]:block((x,0,.50),(.1,.12,1),'iron',.01)
        beam((-.5,0,.95),(.5,0,.95),.1,'iron')
    asset('railing',[1,1],rail,'Open industrial railing preserves camera sightlines')
    asset('metal_plate',[1,1],lambda:(block((0,0,.015),(.94,.94,.03),'iron',.01),block((0,0,.04),(.68,.12,.025),'brass',0)),'Inspection hatch and machinery apron')

    def seal():
        for k in range(16):
            t=k*math.tau/16
            block((3*math.cos(t),3*math.sin(t),.035),(.7,.12,.045),'brass',0,t+math.pi/2)
        for k in range(8):
            t=k*math.tau/8
            beam((1.0*math.cos(t),1.0*math.sin(t),.035),(2.4*math.cos(t),2.4*math.sin(t),.035),.065,'verdigris')
    asset('sanctuary_seal',[7,7],seal,'Fourfold restoration court inlay; original brass and verdigris pattern')

    def track():
        for x in [-.30,.30]:block((x,0,.065),(.09,1.98,.08),'iron',0)
        for y in [-.72,0,.72]:block((0,y,.025),(.92,.14,.04),'timber',0)
    asset('track',[1,2],track,'Quarry-to-workshop narrow-gauge haulage track')

    def cart():
        block((0,0,.68),(1.3,1.5,.8),'iron',.05)
        for x in [-.65,.65]:
            for y in [-.52,.52]:cylinder((x,y,.24),.25,.18,'iron',axis=(1,0,0),vertices=8)
        for x,y in [(-.3,-.3),(.25,.25),(.3,-.35)]:
            bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=1,radius=.35,location=(x,y,1.12));finish(bpy.context.object,'ore')
    asset('ore_cart',[2,2],cart,'Original loaded ore cart ties quarry deposits to production')

    def desk():
        block((0,0,.65),(2.8,1.25,1.3),'timber',.04)
        block((0,0,1.34),(3,1.4,.15),'brass',.025)
        block((.65,.2,1.7),(1,.65,.58),'iron',.04)
        for x in [.3,1]:block((x,.2,1.7),(.08,.7,.63),'brass',.01)
        block((-.65,-.15,1.48),(.55,.45,.08),'bone',0)
    asset('bank_counter',[3,2],desk,'Sanctuary strongbox counter and a ledger; future bank anchor')

    def trough():
        block((0,0,.1),(2.9,1.7,.2),'basalt',.03)
        for x in [-1.3,1.3]:block((x,0,.45),(.25,1.65,.7),'stone',.025)
        for y in [-.70,.70]:block((0,y,.45),(2.4,.25,.7),'stone',.025)
        block((0,0,.55),(2.3,1.05,.05),'water',0)
        block((0,-.3,.59),(1.2,.035,.02),'ripple',0)
    asset('coolant_trough',[3,2],trough,'Readable stone coolant station with an open water surface')

    def hoist():
        for x in [-1.6,1.6]:
            block((x,0,.18),(.8,1,.36),'stone',.035);block((x,0,2),(.28,.35,3.6),'timber',.025)
            beam((x,0,2.2),(x*.45,0,3.7),.20,'iron')
        block((0,0,3.8),(3.8,.48,.4),'timber',.035)
        cylinder((0,0,3.1),.55,.30,'brass',axis=(0,1,0),vertices=12)
        for x in [-.18,.18]:cylinder((x,0,2.2),.045,1.4,'iron',vertices=6)
        block((0,0,1.25),(1.3,1.3,.3),'iron',.035)
    asset('casting_hoist',[4,3],hoist,'Timber and iron gantry gives the cast-house a readable production line')

    def rockwall():
        for x,y,z,s in [(-.65,0,1.1,1.3),(.55,.3,1.35,1.5),(.1,-.5,.8,1.1)]:
            bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=1,radius=s,location=(x,y,z));o=bpy.context.object;o.scale=(.8,.72,1.15);finish(o,'basalt')
        block((.4,-.7,1.1),(.35,.15,.5),'ore',.02,.3)
    asset('rock_buttress',[4,3],rockwall,'Original fractured rock face for quarry edges and failed excavation')

    def fissure():
        points=[(-.22,-.9),(.15,-.35),(-.1,.1),(.22,.65)]
        for u,v in zip(points,points[1:]):
            delta=Vector(v)-Vector(u);delta.normalize()
            first=tuple(u[i]+delta[i]*.04 for i in range(2));last=tuple(v[i]-delta[i]*.04 for i in range(2))
            beam((*first,-.025),(*last,-.025),.14,'basalt');beam((*first,.075),(*last,.075),.045,'ember')
    asset('heat_fissure',[1,2],fissure,'Static heat-stressed paving; visual storytelling, not a live hazard')

    def relief():
        block((0,0,1.5),(1.5,.18,1.5),'basalt',.05)
        ring((0,-.16,1.5),.47,.09,'brass',axis=(0,1,0),segments=8)
        block((0,-.18,1.5),(.14,.12,.85),'verdigris',.01)
    asset('wall_relief',[2,1],relief,'Repeated industrial sanctuary emblem, never borrowed cache art')

    def rack():
        for x in [-.8,.8]:block((x,0,1),(.12,.35,2),'timber',.01)
        for z in [.6,1.2,1.8]:block((0,0,z),(1.8,.65,.10),'timber',.01)
        for x in [-.5,0,.5]:cylinder((x,0,1.42),.11,.4,'copper',vertices=6)
    asset('supply_rack',[2,1],rack,'Shelved supplies make alcoves read as occupied working infrastructure')

    # Subtle material variation breaks repetition without procedural textures.
    for base in ['slate','stone','ore','ash','soil']:
        for variant,factor in enumerate([.92,1.07]):
            key=f'{base}-v{variant}';color=tuple(min(1,c*factor) for c in PALETTE[base][:3])+(1,)
            m=bpy.data.materials.new(key);m.diffuse_color=color;m.use_nodes=True;m.use_backface_culling=True
            m.node_tree.nodes.get('Principled BSDF').inputs['Base Color'].default_value=color;mats[key]=m
            names={'slate':'paving','stone':'sanctuary_paving','ore':'forge_paving','ash':'ash_paving','soil':'garden_soil'}
            asset(f'{names[base]}_v{variant}',[1,1],lambda material=key:floor(material),'Subtle original paving tone; no texture dependencies')

    asset('moss_ground',[1,1],lambda:floor('moss'),'Moist original ground under the roof breach and around cistern roots')
    asset('long_bridge',[6,2],lambda:bridge(6),'A full bank-to-bank reservoir crossing with open railings')

    if a.layout:
        recipe=json.loads(a.layout.read_text(encoding='utf-8'))['curtain_recipe']
        for area,segments in recipe.items():
            if not segments:continue
            xmin=math.floor(min(p[0] for segment in segments for p in segment)-.8)
            ymin=math.floor(min(p[1] for segment in segments for p in segment)-.8)
            xmax=math.ceil(max(p[0] for segment in segments for p in segment)+.8)
            ymax=math.ceil(max(p[1] for segment in segments for p in segment)+.8)
            size=[xmax-xmin,ymax-ymin];center=(xmin+size[0]/2,ymin+size[1]/2)
            def curtain(segments=segments,area=area,center=center):
                vertices=[];faces=[];material_ids=[]
                material_names=['basalt','stone','edge']
                def solid(pos,size,material,bevel,angle):
                    offset=len(vertices);c,s=math.cos(angle),math.sin(angle)
                    for x,y,z in [(-1,-1,-1),(1,-1,-1),(1,1,-1),(-1,1,-1),(-1,-1,1),(1,-1,1),(1,1,1),(-1,1,1)]:
                        x*=size[0]/2;y*=size[1]/2;z*=size[2]/2
                        vertices.append((pos[0]+x*c-y*s,pos[1]+x*s+y*c,pos[2]+z))
                    for f in [(0,3,2,1),(4,5,6,7),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7)]:
                        faces.append(tuple(offset+i for i in f));material_ids.append(material_names.index(material))
                for start,end in segments:
                    dx,dy=end[0]-start[0],end[1]-start[1];length=math.hypot(dx,dy);angle=math.atan2(dy,dx)
                    nx,ny=-dy/length,dx/length
                    x=(start[0]+end[0])/2-center[0]+nx*.12;y=(start[1]+end[1])/2-center[1]+ny*.12
                    low=dx>0 or dy>0
                    height=1.10 if low else 2.75
                    if area in ['quarry','garden']:height=1.4 if low else 2.1
                    material='basalt' if area in ['foundry','vents','quarry'] else 'stone'
                    solid((x,y,0),(length+.15,1.55,.50),'basalt',.025,angle)
                    count=max(1,math.ceil(length/1.3));step=length/count;courses=2 if height<1.5 else 5
                    for n in range(courses):
                        for k in range(count):
                            t=(k+.5)*step-length/2
                            solid((x+math.cos(angle)*t,y+math.sin(angle)*t,.18+(n+.5)*(height-.25)/courses),(step-.035,.40,(height-.25)/courses-.035),material,.015,angle)
                    solid((x,y,height),(length+.05,.53,.18),'edge',.025,angle)
                mesh=bpy.data.meshes.new('authored-masonry-sector');mesh.from_pydata(vertices,[],faces);mesh.update()
                for material in material_names:mesh.materials.append(mats[material])
                for face,index in zip(mesh.polygons,material_ids):face.material_index=index
                obj=bpy.data.objects.new('authored-masonry-sector',mesh);bpy.context.collection.objects.link(obj);finish(obj,'basalt')
            asset('curtain_'+area,size,curtain,'Continuous authored cutaway masonry sector; partition into smaller placements for live collision later',anchor=[xmin,ymin])

    (a.output/'kit-manifest.json').write_text(json.dumps(dict(blender=bpy.app.version_string,origin='Entirely original procedural geometry; no cache or external art',palette=PALETTE,assets=catalogue),indent=2)+'\n')
    bpy.ops.wm.save_as_mainfile(filepath=str(a.output/'cinderworks-kit.blend'))
    print('ORIGINAL_KIT_COMPLETE',len(catalogue),'assets',flush=True)

if __name__=='__main__':main()
