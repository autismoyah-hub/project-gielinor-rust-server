"""Owned mesh production helpers. Blender units here are native units (128/tile).

No cache inputs. Evaluated modifiers, explicit quantization, UV sampling and
component labels bridge arbitrary Blender meshes to the unchanged native codec.
"""
import bpy, bmesh, math, json, colorsys
from mathutils import Vector
from pathlib import Path

def active(o):
    bpy.ops.object.select_all(action='DESELECT');o.select_set(True);bpy.context.view_layer.objects.active=o

def apply(o,modifier):
    active(o);bpy.ops.object.modifier_apply(modifier=modifier.name)

def material(name,rgb,flat=False):
    m=bpy.data.materials.new(name);m.diffuse_color=(*rgb,1);m['native_flat']=flat
    return m

def tag(o,name,mat,group=0):
    o.name=name;o.data.materials.clear();o.data.materials.append(mat);o['owned']=True;o['native_group']=group
    return o

def mesh(name,vs,fs,mat,group=0):
    m=bpy.data.meshes.new(name);m.from_pydata(vs,[],fs);m.update();o=bpy.data.objects.new(name,m);bpy.context.collection.objects.link(o)
    bm=bmesh.new();bm.from_mesh(m);bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bm.to_mesh(m);bm.free()
    return tag(o,name,mat,group)

def bevel(o,width=2,segments=2):
    m=o.modifiers.new('Construction edge bevel','BEVEL');m.width=width;m.segments=segments;apply(o,m);return o

def box(name,c,s,mat,bevel_width=0):
    bpy.ops.mesh.primitive_cube_add(size=1,location=c);o=bpy.context.object;o.scale=s;active(o);bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);tag(o,name,mat)
    return bevel(o,bevel_width) if bevel_width else o

def ellipsoid(name,c,s,mat,group=0,segments=20,rings=12):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=segments,ring_count=rings,radius=1,location=c);o=bpy.context.object;o.scale=s;active(o);bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
    return tag(o,name,mat,group)

def tube(name,points,radii,mat,sides=12,group=0):
    pts=list(map(Vector,points));vs=[]
    for i,p in enumerate(pts):
        t=(pts[min(i+1,len(pts)-1)]-pts[max(0,i-1)]).normalized();a=t.cross(Vector((0,0,1)))
        if a.length<.1:a=t.cross(Vector((0,1,0)))
        a.normalize();b=t.cross(a).normalized();r=radii[i];rx,ry=(r,r)if isinstance(r,(int,float))else r
        vs.extend([tuple(p+a*(rx*math.cos(j*math.tau/sides))+b*(ry*math.sin(j*math.tau/sides)))for j in range(sides)])
    fs=[tuple(reversed(range(sides))),tuple((len(pts)-1)*sides+j for j in range(sides))]
    for i in range(len(pts)-1):
        for j in range(sides):k=i*sides+j;n=i*sides+(j+1)%sides;fs.append((k,n,n+sides,k+sides))
    return mesh(name,vs,fs,mat,group)

def ring(name,c,r,thickness,depth,mat,axis='Y',steps=32):
    vs=[]
    for d in [-depth/2,depth/2]:
        for rad in [r-thickness/2,r+thickness/2]:
            for i in range(steps):
                a=i*math.tau/steps;p=[rad*math.cos(a),d,rad*math.sin(a)]
                if axis=='Z':p=[p[0],p[2],p[1]]
                if axis=='X':p=[p[1],p[0],p[2]]
                vs.append(tuple(c[j]+p[j]for j in range(3)))
    fs=[]
    for i in range(steps):
        n=(i+1)%steps
        fs.extend([(i,n,steps+n,steps+i),(2*steps+i,3*steps+i,3*steps+n,2*steps+n),(i,2*steps+i,2*steps+n,n),(steps+i,steps+n,3*steps+n,3*steps+i)])
    return mesh(name,vs,fs,mat)

def remesh_union(objects,name,mat,voxel=3,ratio=.35):
    bpy.ops.object.select_all(action='DESELECT')
    for o in objects:o.select_set(True)
    bpy.context.view_layer.objects.active=objects[0];bpy.ops.object.join();o=bpy.context.object
    r=o.modifiers.new('Continuous volume union','REMESH');r.mode='VOXEL';r.voxel_size=voxel;apply(o,r)
    s=o.modifiers.new('Contour relaxation','SMOOTH');s.factor=1.1;s.iterations=4;apply(o,s)
    d=o.modifiers.new('Curvature preserving simplification','DECIMATE');d.ratio=ratio;apply(o,d)
    return tag(o,name,mat)

def uv(o):
    active(o);bpy.ops.object.mode_set(mode='EDIT');bpy.ops.mesh.select_all(action='SELECT');bpy.ops.uv.smart_project(angle_limit=1.05,island_margin=.02);bpy.ops.object.mode_set(mode='OBJECT')

def hsl(rgb):
    h,l,s=colorsys.rgb_to_hls(*[max(0,min(1,x))for x in rgb]);return (min(63,int(h*64))<<10)|(min(7,int(s*8))<<7)|max(2,min(126,int(l*127)))

def export(objects,out,codec,sample_images=True):
    """Material diffuse / UV albedo -> native HSL. Unsupported PBR is reported.

    Quantize only an evaluated export copy; preserve full source precision.
    Drop triangles that collapse on the native lattice, recording every loss.
    """
    out=Path(out);out.mkdir(parents=True,exist_ok=True)
    g=dict(vertices=[],faces=[],colors=[],render_types=[],vertex_skins=[],face_skins=None,priority=0)
    report=dict(components=[],quantization_max_error=0,collapsed_triangles=0,uv_loops=0,material_conversion='UV image albedo sampled at triangle centroid, otherwise diffuse color; RGB to 6/3/7-bit HSL. No PBR/normal/roughness transfer.',texture_image_samples=0)
    deps=bpy.context.evaluated_depsgraph_get()
    for o in objects:
        ev=o.evaluated_get(deps);m=ev.to_mesh();m.calc_loop_triangles();off=len(g['vertices']);verts=[];start=len(g['faces'])
        for v in m.vertices:
            w=o.matrix_world@v.co;p=[w.x,-w.z,w.y];q=list(map(round,p));report['quantization_max_error']=max(report['quantization_max_error'],max(abs(a-b)for a,b in zip(p,q)));verts.append(q)
        g['vertices']+=verts;g['vertex_skins'] += [o.get('native_group',0)]*len(verts)
        report['uv_loops']+=len(m.uv_layers.active.data)if m.uv_layers.active else 0
        for tri in m.loop_triangles:
            a,b,c=[Vector(verts[i])for i in tri.vertices]
            if (b-a).cross(c-a).length==0:report['collapsed_triangles']+=1;continue
            mat=m.materials[tri.material_index];rgb=list(mat.diffuse_color[:3]);im=None
            if mat.use_nodes:
                im=next((n.image for n in mat.node_tree.nodes if n.type=='TEX_IMAGE' and n.image),None)
            if sample_images and im and m.uv_layers.active:
                uvs=[m.uv_layers.active.data[k].uv for k in tri.loops];u=sum(x.x for x in uvs)/3%1;v=sum(x.y for x in uvs)/3%1;w,h=im.size;k=(min(h-1,int(v*h))*w+min(w-1,int(u*w)))*4;rgb=list(im.pixels[k:k+3]);report['texture_image_samples']+=1
            # Deliberate material regions survive smoothing; no random face noise.
            if o.get('height_color'):
                center=sum((o.matrix_world@m.vertices[i].co).z for i in tri.vertices)/3
                factor=.83+.17*max(0,min(1,center/110));rgb=[x*factor for x in rgb]
            g['faces'].append([i+off for i in tri.vertices]);g['colors'].append(hsl(rgb));g['render_types'].append(int(mat.get('native_flat',False)))
        report['components'].append(dict(name=o.name,vertices=len(verts),triangles=len(g['faces'])-start,group=o.get('native_group',0)))
        ev.to_mesh_clear()
    report.update(vertices=len(g['vertices']),triangles=len(g['faces']),budget=dict(soft_vertices=12000,soft_triangles=20000),coordinates='X right, Y down, Z depth; 128 units/tile',face_priority=0,alpha=0)
    if len(g['vertices'])>12000 or len(g['faces'])>20000:raise ValueError('Study budget exceeded')
    (out/'geometry.json').write_text(json.dumps(g));(out/'asset.model').write_bytes(codec(g));(out/'conversion.json').write_text(json.dumps(report,indent=2));return g
