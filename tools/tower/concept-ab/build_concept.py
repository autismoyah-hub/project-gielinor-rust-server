"""Concept run's authored plate surfaces and calibrated image arrangement.

Uses the frozen baseline mesh/solidify/export tools. No competing geometry.
The nine-point shield cages are fitted to this run's own anatomy; their interior
samples prevent the boundary-only patch from crossing a convex body surface.
"""
import sys,json,math
from pathlib import Path
import bpy
from mathutils import Vector
T=Path(__file__).resolve().parent;sys.path.insert(0,str(T.parent/'constrained'))
import surface as s
def shell(name,points,faces,mat,edge,thickness):
    o=s.mesh(name,points,faces,mat);o.data.materials.append(edge)
    mod=o.modifiers.new('Authored shell thickness and worn rim','SOLIDIFY');mod.thickness=thickness;mod.offset=0;mod.material_offset_rim=1;s.apply(o,mod)
    o['semantic_component']=name;o['authored_shell_cage']=json.dumps(points)
    return o
def refined_shells(recipe):
    s.remove('^cap_|^volumetric_dorsal_|^shield_front')
    body=s.select('^integrated_anatomy$')[0];inv=body.matrix_world.inverted()
    plate=s.native_color('plate_shell',recipe['materials']['plate'],True);edge=s.native_color('worn_shell_rim',recipe['materials']['plate_edge'],True)
    faces=[(i*3+j,i*3+j+1,(i+1)*3+j+1,(i+1)*3+j) for i in range(2) for j in range(2)]
    dense=recipe.get('concept_stage',0)>=5
    for idx,(y,w) in enumerate([(-48,28),(-23,25),(3,21),(28,18),(55,21)]):
        points=[]
        rows=[(-14,.72),(-3,1),(13,.64)] if not dense else [(-14,.72),(-8,.91),(-2,1),(6,.85),(13,.64)]
        columns=[-1,0,1] if not dense else [-1,-.5,0,.5,1]
        for dy,scale in rows:
            for col in columns:
                x=col*w*scale;depth=y+dy+(3 if col==0 and dy>0 else -3 if col==0 and dy<0 else 0)
                hit,p,_,_=body.ray_cast(inv@Vector((x,depth,400)),Vector((0,0,-1)))
                if not hit:raise ValueError('Original shield cage misses authored anatomy')
                z=(body.matrix_world@p).z+(4.8 if dense else 3.2)+(1 if col==0 else 0)
                points.append((x,depth,z))
        n=len(columns);dorsal_faces=[(i*n+j,i*n+j+1,(i+1)*n+j+1,(i+1)*n+j) for i in range(len(rows)-1) for j in range(n-1)]
        if dense:
            # Test real triangle interiors as well as cage corners. Preserve
            # clearance without flattening or changing the authored creature.
            points=list(map(Vector,points))
            triangles=[(a,b,c) for a,b,c,d in dorsal_faces]+[(a,c,d) for a,b,c,d in dorsal_faces]
            for iteration in range(3):
                lifts=[0.0]*len(points)
                for tri in triangles:
                    q=sum((points[i] for i in tri),Vector())/3
                    hit,p,_,_=body.ray_cast(inv@Vector((q.x,q.y,400)),Vector((0,0,-1)))
                    if hit:
                        gap=(body.matrix_world@p).z+4.0-q.z
                        for i in tri:lifts[i]=max(lifts[i],gap)
                for i,delta in enumerate(lifts):points[i].z+=max(0,delta)
            points=[tuple(p) for p in points]
        shell(f'basalt_dorsal_shield_{idx}',points,dorsal_faces,plate,edge,3.0)
    for side in [-1,1]:
        for name,rows in [('upper',[
          [(29,-63,123),(36,-63,118),(39,-63,108)],
          [(29,-49,133),(42,-49,124),(47,-49,111)],
          [(29,-27,122),(40,-27,111),(43,-27,103)]]),('lower',[
          [(35,-49,112),(45,-49,104),(49,-49,95)],
          [(38,-39,115),(47,-39,102),(49,-39,91)],
          [(33,-25,104),(43,-25,96),(44,-25,87)]])]:
            shell(f'layered_shoulder_{name}_{side}',[(side*x,y,z) for row in rows for x,y,z in row],faces,plate,edge,2.5)
def references(root):
    collection=bpy.data.collections.new('Concept guides — original art, not export geometry');bpy.context.scene.collection.children.link(collection)
    manifest=json.loads((root/'consistency.json').read_text())
    for key in ['front','left','rear']:
        image=bpy.data.images.load(str((root/(key+'.png')).resolve()));image.pack()
        obj=bpy.data.objects.new('Reference '+key,None);collection.objects.link(obj);obj.empty_display_type='IMAGE';obj.data=image;obj.empty_image_depth='BACK';obj.color[3]=.45;obj.hide_render=True
        c=manifest['calibration'][key];px=c['pixels_per_native_unit'];obj.empty_display_size=image.size[0]/px
        if key=='left':
            origin=c['pixel_nose_ground'];obj.location=(100,-140+(image.size[0]/2-origin[0])/px,(origin[1]-image.size[1]/2)/px);obj.rotation_euler=(math.pi/2,0,math.pi/2)
        else:
            origin=c['pixel_origin_ground'];obj.location=((image.size[0]/2-origin[0])/px,210 if key=='front' else -210,(origin[1]-image.size[1]/2)/px);obj.rotation_euler=(math.pi/2,0,0 if key=='front' else math.pi)
        obj['calibration']=json.dumps(c);obj['known_conflicts']='Read consistency.json; numeric frozen dimensions override drawn width, tail and anatomical-label/axis conflict.'
    for key,eye in [('front',(0,-450,84)),('left',(-450,20,84)),('rear',(0,450,84))]:
        data=bpy.data.cameras.new('Guide '+key);data.type='ORTHO';data.ortho_scale=210 if key!='left' else 355
        obj=bpy.data.objects.new('Guide '+key,data);bpy.context.collection.objects.link(obj);obj.location=eye;obj.rotation_euler=(Vector((0,20,84))-obj.location).to_track_quat('-Z','Y').to_euler()
    return manifest
if __name__=='__main__':
    args=sys.argv[sys.argv.index('--')+1:];recipe_path,out=args[:2];recipe=json.loads(Path(recipe_path).read_text());bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
    report=s.execute(recipe)
    if recipe.get('concept_stage',0)>=4:refined_shells(recipe)
    references(Path(args[2]) if len(args)>2 else Path(recipe_path).resolve().parents[0])
    s.save(out,recipe,report)
    (Path(out)/'plate-authoring-snapshot.py').write_text(Path(__file__).read_text())
