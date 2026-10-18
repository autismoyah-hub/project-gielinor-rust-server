"""Apply explicit local edits to owned components in a saved Blender scene.

Untouched components remain byte-identical on export. No reference mesh is read.
"""
import bpy,json,sys,math,bmesh,hashlib
from pathlib import Path
from mathutils import Vector,Matrix
sys.path.insert(0,str(Path(__file__).parent))
from author_blender import export_asset,actions
from native_codec import model
def main():
    recipe_path,patch_path,out=map(Path,sys.argv[sys.argv.index('--')+1:sys.argv.index('--')+4]);recipe=json.loads(recipe_path.read_text());patch=json.loads(patch_path.read_text());out=out.resolve();out.mkdir(parents=True,exist_ok=True);before=export_asset(recipe)
    for op in patch['operations']:
        names=op['components']
        if op['op']=='affine':
            scale=op.get('scale',[1]*3);pivot=op.get('pivot',[0]*3);translation=op.get('translation',[0]*3)
            if min(scale)<=0:raise ValueError('component reflection/collapse is not an automatic refinement')
            for name in names:
                obj=bpy.data.objects[name]
                for vertex in obj.data.vertices:
                    native=[vertex.co.x*128,-vertex.co.z*128,vertex.co.y*128];v=[round((native[i]-pivot[i])*scale[i]+pivot[i]+translation[i])for i in range(3)];vertex.co=(v[0]/128,v[2]/128,-v[1]/128)
        elif op['op']=='rotate':
            rotation=Matrix(op['matrix']);pivot=Vector(op['pivot'])
            if abs(rotation.determinant()-1)>1e-5:raise ValueError('rotation matrix must preserve orientation')
            for name in names:
                for vertex in bpy.data.objects[name].data.vertices:
                    native=Vector([vertex.co.x*128,-vertex.co.z*128,vertex.co.y*128]);v=rotation@(native-pivot)+pivot;v=list(map(round,v));vertex.co=(v[0]/128,v[2]/128,-v[1]/128)
        elif op['op']=='height_palette':
            low,high=op['bounds_y'];colors=op['colors']
            for name in names:
                mesh=bpy.data.objects[name].data;slots=[]
                for c in colors:
                    mat=bpy.data.materials.new(f'Owned region native HSL {c}');mat['native_hsl']=int(c);mesh.materials.append(mat);slots.append(len(mesh.materials)-1)
                for face in mesh.polygons:
                    y=sum(-mesh.vertices[i].co.z*128 for i in face.vertices)/len(face.vertices);band=min(len(colors)-1,max(0,int((y-low)/max(1,high-low)*len(colors))));face.material_index=slots[band]
        else:raise ValueError('unknown edit operation')
        for name in names:
            mesh=bpy.data.objects[name].data;bm=bmesh.new();bm.from_mesh(mesh);bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bm.to_mesh(mesh);bm.free();mesh.update()
    g=export_asset(recipe);changed=[name for name,v in g['components'].items()if v['geometry_sha256']!=before['components'][name]['geometry_sha256']];declared={n for op in patch['operations']for n in op['components']};unexpected=set(changed)-declared
    if unexpected:raise AssertionError('unrelated components changed '+str(unexpected))
    (out/'geometry.json').write_text(json.dumps(g));(out/'asset.model').write_bytes(model(g));(out/'recipe.json').write_text(json.dumps(recipe,indent=2));(out/'patch.json').write_text(json.dumps(patch,indent=2));(out/'local-edit-proof.json').write_text(json.dumps(dict(changed=changed,unchanged=[n for n in before['components']if n not in changed],unexpected=list(unexpected)),indent=2));actions(recipe,g,out,True);bpy.ops.wm.save_as_mainfile(filepath=str(out/'asset.blend'));print('LOCAL_COMPONENT_EDIT_PASS',len(changed),'changed;',len(before['components'])-len(changed),'untouched')
if __name__=='__main__':main()
