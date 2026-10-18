"""Blender interchange shared by reference inspection and original authoring.

Geometry is exported from saved Blender data, never returned from a source blob.
"""
import sys,json
from pathlib import Path
import bpy
sys.path.insert(0,str(Path(__file__).resolve().parent))
from blender_actions import import_action,export_action
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'custom-assets/blender'))
from calibration_roundtrip import mesh_import,mesh_export
FACE_KEYS=['face_textures','texture_coords']
def srgb(v):return v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4
def import_model(name,g,palette=None):
    obj=mesh_import(name,g)
    for key in FACE_KEYS:
        if g.get(key)is not None:
            attr=obj.data.attributes.new(key,'INT','FACE')
            for v,n in zip(attr.data,g[key]):v.value=n
    obj['native_metadata']=json.dumps(g.get('metadata'))
    obj['workbench_model']=True
    if palette:
        for mat in obj.data.materials:
            rgb=palette[int(mat['native_hsl'])];color=tuple(srgb(((rgb>>s)&255)/255)for s in [16,8,0]);mat.diffuse_color=(*color,1)
    if g.get('texture_triangles'):
        mesh=bpy.data.meshes.new(name+'_texture_maps');mesh.from_pydata([tuple(v.co)for v in obj.data.vertices],[],g['texture_triangles']);mesh.update();maps=bpy.data.objects.new(name+'_texture_maps',mesh);bpy.context.collection.objects.link(maps);maps.hide_render=True;maps.hide_viewport=True
        attr=mesh.attributes.new('native_texture_type','INT','FACE')
        for v,n in zip(attr.data,g['texture_types']):v.value=n
        obj['texture_maps_object']=maps.name
    return obj
def export_model(obj):
    g=mesh_export(obj)
    if obj.matrix_world.determinant()<=0:raise ValueError('Reflected object transform must be deliberately baked with corrected winding')
    transformed=[obj.matrix_world@v.co for v in obj.data.vertices]
    native=[[v.x*128,-v.z*128,v.y*128]for v in transformed]
    if any(abs(x-round(x))>1e-4 for v in native for x in v):raise ValueError('Object transform creates fractional native coordinates; explicit authoring quantization required')
    g['vertices']=[list(map(round,v))for v in native]
    for key in FACE_KEYS:g[key]=[v.value for v in obj.data.attributes[key].data]if key in obj.data.attributes else None
    g['metadata']=json.loads(obj.get('native_metadata','null'))
    if obj.get('texture_maps_object'):
        mesh=bpy.data.objects[obj['texture_maps_object']].data;g['texture_triangles']=[list(p.vertices)for p in mesh.polygons];g['texture_types']=[v.value for v in mesh.attributes['native_texture_type'].data]
    else:g['texture_triangles']=[];g['texture_types']=[]
    return g
def main():
    args=sys.argv[sys.argv.index('--')+1:];mode,source,output=args[:3];source,output=Path(source).resolve(),Path(output).resolve();output.mkdir(parents=True,exist_ok=True)
    if mode=='import':
        bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
        palette=json.loads((source/'palette.json').read_text())if(source/'palette.json').exists()else None
        failures=[]
        for path in sorted(source.glob('*/geometry.json')):
            g=json.loads(path.read_text())
            # Native format cannot store evaluated fractional object vertices.
            if any(abs(x-round(x))>1e-4 for v in g['vertices']for x in v):
                failures.append(dict(asset=path.parent.name,strategy='raw_model_plus_native_definition',reason='preserve original native definition transform instead of rounding evaluated coordinates'))
                for raw in sorted(path.parent.glob('raw-model-*.json')):import_model(path.parent.name+'--'+raw.stem,json.loads(raw.read_text()),palette)
                continue
            import_model(path.parent.name,g,palette)
        for index,path in enumerate(sorted(source.glob('*/*/animation.json'))):import_action('Reference action '+str(index),path,index)
        (output/'deferred.json').write_text(json.dumps(failures,indent=2));bpy.ops.wm.save_as_mainfile(filepath=str(output/'reference-roundtrips.blend'))
    elif mode=='export':
        for obj in bpy.data.objects:
            if obj.get('workbench_model'):(output/(obj.name+'.json')).write_text(json.dumps(export_model(obj)))
            if obj.get('classic_action_export'):
                destination=output/'actions'/obj['relative_action_path'];destination.mkdir(parents=True,exist_ok=True);(destination/'animation.json').write_text(json.dumps(export_action(obj)))
    else:raise ValueError('import|export')
    print('WORKBENCH_BLENDER_IO_PASS',mode)
if __name__=='__main__':main()
