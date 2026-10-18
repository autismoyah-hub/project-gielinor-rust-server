"""Semantic surface controls for independently authored static meshes.

No cache readers, reference geometry import or reference-specific dimensions.
Coordinates are authoring X right, Y depth, Z up. Recipes and derived studies
may be protected; keep them outside the source tree.
"""
import math,re,sys,json
from pathlib import Path
HERE=Path(__file__).resolve().parent
sys.path[:0]=[str(HERE.parent/'reassessment'),str(HERE.parent/'mesh-study'),str(HERE.parent/'asset-workbench')]
from anatomy import mesh,material,native_color,solid_surface,projected_regions,landmark_limb,surface_regions,fitted_patch,shell_profile
from mesh_tools import export,active,apply,box,tube
from native_codec import model
import bpy,bmesh
from mathutils import Vector,Matrix

SECTIONS={
 'organic':[(0,1),(.66,.96),(1,.72),(.96,.35),(.65,.08),(0,0),(-.65,.08),(-.96,.35),(-1,.72),(-.66,.96)],
 'cranium':[(0,1),(.65,.92),(1,.55),(.8,.12),(0,0),(-.8,.12),(-1,.55),(-.65,.92)],
 'diamond':[(0,1),(1,0),(0,-1),(-1,0)],
 'rectangle':[(1,1),(1,0),(-1,0),(-1,1)],
 'canine':[(0,1),(.55,.98),(.92,.75),(1,.54),(.7,.25),(.25,0),(0,.08),(-.25,0),(-.7,.25),(-1,.54),(-.92,.75),(-.55,.98)],
}

def loft(name,stations,mat,section='organic',ridges=None,axis='Y'):
    """Depth stations [depth,width,bottom,top], optional [center_x,twist].

    Separate dorsal/ventral contours and a non-elliptical flank section preserve
    chest, waist and pelvic planes. Shared rings give an intrinsically connected
    surface. Alternating diagonals avoid a one-directional shading bias.
    """
    shape=SECTIONS[section];n=len(shape);vs=[]
    for row in stations:
        y,w,low,high=row[:4];cx=row[4] if len(row)>4 else 0;tw=row[5] if len(row)>5 else 0
        for a,b in shape:
            x=a*w;z=low+(high-low)*b
            vs.append((cx+x*math.cos(tw)-(z-(low+high)/2)*math.sin(tw),y,(low+high)/2+x*math.sin(tw)+(z-(low+high)/2)*math.cos(tw)))
    fs=[tuple(reversed(range(n))),tuple((len(stations)-1)*n+j for j in range(n))]
    for i in range(len(stations)-1):
        for j in range(n):
            a=i*n+j;b=i*n+(j+1)%n;c=(i+1)*n+(j+1)%n;d=(i+1)*n+j
            fs.extend([(a,b,d),(b,c,d)] if (i+j)%2 else [(a,b,c),(a,c,d)])
    if axis=='Z':vs=[(x,z,y)for x,y,z in vs]
    elif axis=='X':vs=[(y,x,z)for x,y,z in vs]
    elif axis!='Y':raise ValueError('Loft axis must be X, Y or Z')
    o=mesh(name,vs,fs,mat);o['semantic_component']=name;o['construction']='independent dorsal/ventral station loft'
    return o

def masked_deform(objects,center,radius,delta=(0,0,0),scale=(1,1,1),power=2):
    """Compact-support ellipsoidal sculpt brush; no raw vertex target fitting.

    Smooth falloff vanishes at mask boundary, preserving attached surrounding
    structure. Dimensions and displacement are in world authoring coordinates.
    """
    count=0
    for o in objects:
        inv=o.matrix_world.inverted()
        for v in o.data.vertices:
            q=o.matrix_world@v.co;r=sum(((q[i]-center[i])/max(.001,radius[i]))**2 for i in range(3))
            if r>=1:continue
            w=(1-r)**power
            change=Vector(tuple(delta[i]+(q[i]-center[i])*(scale[i]-1) for i in range(3)))
            v.co=inv@(q+change*w);count+=1
    return count

def region_transform(objects,pivot,scale=(1,1,1),translate=(0,0,0)):
    for o in objects:
        inv=o.matrix_world.inverted()
        for v in o.data.vertices:
            q=o.matrix_world@v.co;v.co=inv@Vector(tuple(pivot[i]+(q[i]-pivot[i])*scale[i]+translate[i] for i in range(3)))

def relax_masked(o,center,radius,iterations=2,strength=.2):
    """Local Laplacian relaxation with pinned mask boundary and radius recovery.

    This is a modest contour correction, not a global smoothing/remesh pass.
    Radial recovery restores the selected region's mean radius after relaxation.
    """
    adjacency=[set()for _ in o.data.vertices]
    for edge in o.data.edges:a,b=edge.vertices;adjacency[a].add(b);adjacency[b].add(a)
    original=[v.co.copy()for v in o.data.vertices];weights=[]
    for v in o.data.vertices:
        q=o.matrix_world@v.co;r=sum(((q[i]-center[i])/radius[i])**2 for i in range(3));weights.append(max(0,1-r)**2)
    pivot=o.matrix_world.inverted()@Vector(center)
    for _ in range(iterations):
        previous=[v.co.copy()for v in o.data.vertices]
        for i,v in enumerate(o.data.vertices):
            if weights[i] and adjacency[i]:v.co=previous[i].lerp(sum((previous[j]for j in adjacency[i]),Vector())/len(adjacency[i]),strength*weights[i])
    before=sum((v-pivot).length*w for v,w in zip(original,weights));after=sum((v.co-pivot).length*w for v,w in zip(o.data.vertices,weights))
    recovery=before/max(after,1e-9)
    for v,w in zip(o.data.vertices,weights):v.co=pivot+(v.co-pivot)*(1+(recovery-1)*w)

def blade(name,stations,metal,edge,height=0):
    """Curved centerline with width perpendicular to tangent; diamond section.

    Stations [depth,center_x,half_width,half_thickness]. Width/thickness vary
    independently; a very small terminal section closes a coherent tip.
    """
    vs=[]
    for i,(y,c,w,t) in enumerate(stations):
        left=stations[max(i-1,0)];right=stations[min(i+1,len(stations)-1)]
        tangent=Vector((right[1]-left[1],right[0]-left[0],0)).normalized();normal=Vector((tangent.y,-tangent.x,0))
        if normal.x<0:normal=-normal
        for u,z in [(-w,0),(0,t),(w,0),(0,-t)]:vs.append((c+normal.x*u,y+normal.y*u,height+z))
    fs=[(3,2,1,0),tuple((len(stations)-1)*4+j for j in range(4))]
    fs += [(i*4+j,i*4+(j+1)%4,(i+1)*4+(j+1)%4,(i+1)*4+j)for i in range(len(stations)-1)for j in range(4)]
    o=mesh(name,vs,fs,metal);o.data.materials.append(edge)
    for p in o.data.polygons:
        if p.index>=2 and (p.index-2)%4 in [0,3]:p.material_index=1
    o['semantic_component']='blade';return o

def select(pattern):return [o for o in bpy.context.scene.objects if o.type=='MESH' and re.search(pattern,o.name)]
def remove(pattern):
    for o in select(pattern):bpy.data.objects.remove(o,do_unlink=True)

def execute(recipe):
    """Apply a declarative, independently editable semantic component recipe."""
    mats={key:native_color(key,int(code)) for key,code in recipe.get('materials',{}).items()}
    report=[]
    for operation in recipe['operations']:
        op=dict(operation);action=op.pop('op');pattern=op.pop('target',None)
        if action=='remove':remove(pattern)
        elif action=='loft':loft(mat=mats[op.pop('material')],**op)
        elif action=='limb':landmark_limb(mat=mats[op.pop('material')],**op)
        elif action=='blade':blade(metal=mats[op.pop('material')],edge=mats[op.pop('edge')],**op)
        elif action=='box':box(mat=mats[op.pop('material')],**op)
        elif action=='tube':tube(mat=mats[op.pop('material')],**op)
        elif action=='shell':shell_profile(mat=mats[op.pop('material')],**op)
        elif action=='component_scale':
            for o in select(pattern):
                center=Vector(tuple((min(v.co[i]for v in o.data.vertices)+max(v.co[i]for v in o.data.vertices))/2 for i in range(3)))
                for v in o.data.vertices:v.co=Vector(tuple(center[i]+(v.co[i]-center[i])*op['scale'][i]for i in range(3)))
        elif action=='transform':region_transform(select(pattern),**op)
        elif action=='rotate':
            rotation=Matrix.Rotation(math.radians(op['degrees']),4,Vector(op['axis']));pivot=Vector(op['pivot'])
            for o in select(pattern):
                inv=o.matrix_world.inverted()
                for v in o.data.vertices:v.co=inv@(pivot+rotation.to_3x3()@(o.matrix_world@v.co-pivot))
        elif action=='sculpt':masked_deform(select(pattern),**op)
        elif action=='relax':
            for o in select(pattern):relax_masked(o,**op)
        elif action=='region':
            mat=mats[op['material']];axes=tuple(op.get('axes',[1,2]));polygon=op['polygon'];side_min=op.get('side_min',-1)
            for o in select(pattern):projected_regions(o,[(mat,axes,polygon,lambda x,y,z:abs(x)>side_min)])
        elif action=='band':
            axis=op['axis'];low,high=op['range'];mat=mats[op['material']]
            for o in select(pattern):surface_regions(o,[(mat,lambda x,y,z:low<=(x,y,z)[axis]<=high)])
        elif action=='box_region':
            limits=op['bounds'];mat=mats[op['material']]
            for o in select(pattern):surface_regions(o,[(mat,lambda x,y,z:all(lo<=v<=hi for v,(lo,hi)in zip((x,y,z),limits)))])
        elif action=='patch':
            body=select(pattern)
            if len(body)!=1:raise ValueError('Patch requires one semantic surface')
            fitted_patch(body=body[0],mat=mats[op.pop('material')],**op)
        elif action=='fuse':
            objects=select(pattern)
            if not objects:raise ValueError('Empty fusion selection')
            bpy.ops.object.select_all(action='DESELECT')
            for o in objects:o.select_set(True)
            bpy.context.view_layer.objects.active=objects[0];bpy.ops.object.join();o=bpy.context.object
            mod=o.modifiers.new('Local anatomical joint fusion','REMESH');mod.mode='VOXEL';mod.voxel_size=op.get('voxel',1.5);apply(o,mod)
            mod=o.modifiers.new('Bounded joint relaxation','SMOOTH');mod.factor=.25;mod.iterations=2;apply(o,mod)
            mod=o.modifiers.new('Authored plane density','DECIMATE');mod.ratio=min(1,op.get('triangles',650)/max(1,len(o.data.polygons)*2));apply(o,mod)
            o.name=op['name'];o.data.materials.clear();o.data.materials.append(mats[op['material']])
            for p in o.data.polygons:p.material_index=0
        elif action=='material':
            for o in select(pattern):
                o.data.materials.clear();o.data.materials.append(mats[op['material']])
                for p in o.data.polygons:p.material_index=0
        elif action=='shading':
            for o in select(pattern):
                for i,mat in enumerate(list(o.data.materials)):
                    copied=mat.copy();copied['native_flat']=bool(op['flat']);o.data.materials[i]=copied
        elif action=='normal_regions':
            for o in select(pattern):
                for region in op['regions']:
                    index=len(o.data.materials);o.data.materials.append(mats[region['material']])
                    for p in o.data.polygons:
                        n=o.matrix_world.to_3x3()@p.normal
                        value=n[region['axis']];value=abs(value)if region.get('absolute')else value
                        if region['range'][0]<=value<=region['range'][1]:p.material_index=index
        elif action=='decimate':
            for o in select(pattern):
                mod=o.modifiers.new('Controlled semantic surface reduction','DECIMATE');mod.ratio=op['ratio'];apply(o,mod)
        else:raise ValueError(action)
        report.append({'operation':operation,'affected_components':[o.name for o in select(pattern)]if pattern else [op.get('name')]})
    return report

def save(out,recipe,report):
    out=Path(out);out.mkdir(parents=True,exist_ok=False)
    objects=[o for o in bpy.context.scene.objects if o.type=='MESH']
    for o in objects:
        o['semantic_component']=o.get('semantic_component',re.sub(r'\.\d+$','',o.name))
        for p in o.data.polygons:p.use_smooth=not bool(o.data.materials[p.material_index].get('native_flat',False))
    g=export(objects,out,model);g['vertex_skins']=None;(out/'geometry.json').write_text(json.dumps(g));(out/'asset.model').write_bytes(model(g))
    (out/'recipe.json').write_text(json.dumps(recipe,indent=2));(out/'construction-ledger.json').write_text(json.dumps(report,indent=2))
    (out/'components.json').write_text(json.dumps([{'name':o.name,'semantic':o['semantic_component'],'vertices':len(o.data.vertices),'faces':len(o.data.polygons)} for o in objects],indent=2))
    bpy.ops.wm.save_as_mainfile(filepath=str((out/'source.blend').resolve()))

if __name__=='__main__':
    source,recipe_path,out=sys.argv[sys.argv.index('--')+1:]
    bpy.ops.wm.open_mainfile(filepath=str(Path(source).resolve()))
    recipe=json.loads(Path(recipe_path).read_text());save(out,recipe,execute(recipe))
