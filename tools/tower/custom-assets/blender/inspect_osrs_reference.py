"""Local reference geometry inspection, never a native appearance substitute.

blender -b --factory-startup --python inspect_osrs_reference.py -- --input DIR --output DIR
Input is the local-only JSON emitted by native art_reference. Do not distribute
input, .blend or resulting reference frames without distribution permission.
"""
import argparse, json, math, sys
from pathlib import Path
import bpy
from mathutils import Vector

def linear(c):
    c=c/255
    return c/12.92 if c<=.04045 else ((c+.055)/1.055)**2.4

def main():
    p=argparse.ArgumentParser();p.add_argument('--input',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    a=p.parse_args(sys.argv[sys.argv.index('--')+1:]);a.input=a.input.resolve();a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=True)
    palette=json.loads((a.input/'palette.json').read_text())
    reports=[]
    for path in sorted(a.input.glob('object-*.json')):
        bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
        data=json.loads(path.read_text());g=data['geometry'];verts=[(x/128,z/128,-y/128) for x,y,z in g['vertices']]
        mesh=bpy.data.meshes.new(path.stem);mesh.from_pydata(verts,[],g['faces']);mesh.update()
        obj=bpy.data.objects.new(data['name'],mesh);bpy.context.collection.objects.link(obj)
        # No merging, cleanup, normal recalculation, re-triangulation or guessing UVs.
        indices={};textures=g['face_textures'] or [-1]*len(g['faces'])
        for i,face in enumerate(mesh.polygons):
            key=('unsupported-texture',textures[i]) if textures[i]>=0 else ('hsl',g['colors'][i])
            if key not in indices:
                mat=bpy.data.materials.new(str(key));mat.use_nodes=True
                rgb=(.6,0,.6) if textures[i]>=0 else tuple(linear((palette[g['colors'][i]]>>shift)&255) for shift in (16,8,0))
                mat.diffuse_color=(*rgb,1);mat.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value=(*rgb,1)
                mat.node_tree.nodes['Principled BSDF'].inputs['Roughness'].default_value=1
                indices[key]=len(mesh.materials);mesh.materials.append(mat)
            face.material_index=indices[key];face.use_smooth=False
        obj['native_reference_metadata']=json.dumps({k:v for k,v in data.items() if k!='geometry'})
        text=bpy.data.texts.new('NATIVE_SOURCE_JSON');text.write(json.dumps(data))
        low=Vector(tuple(min(v[i] for v in verts) for i in range(3)));high=Vector(tuple(max(v[i] for v in verts) for i in range(3)));center=(low+high)/2;extent=max(high-low)
        scene=bpy.context.scene;scene.render.engine='BLENDER_WORKBENCH';scene.display.shading.light='STUDIO';scene.display.shading.color_type='MATERIAL';scene.display.shading.show_shadows=True
        scene.render.resolution_x=700;scene.render.resolution_y=700;scene.render.resolution_percentage=100
        scene.world.color=(.05,.05,.05);scene.view_settings.view_transform='Standard'
        cd=bpy.data.cameras.new('Inspection camera');cam=bpy.data.objects.new('Inspection camera',cd);bpy.context.collection.objects.link(cam);scene.camera=cam;cd.type='ORTHO';cd.ortho_scale=max(extent*1.5,1)
        for label,direction in [('front',(0,-1,.35)),('reverse',(0,1,.35)),('quarter',(1,-1,.7)),('plan',(0,-.001,1))]:
            cam.location=center+Vector(direction).normalized()*max(extent*3,3);cam.rotation_euler=(center-cam.location).to_track_quat('-Z','Y').to_euler()
            scene.render.filepath=str(a.output/f'{path.stem}-{label}.png');bpy.ops.render.render(write_still=True)
        assert len(mesh.vertices)==len(g['vertices']) and len(mesh.polygons)==len(g['faces'])
        assert [list(f.vertices) for f in mesh.polygons]==g['faces']
        err=max(abs(mesh.vertices[i].co[j]-v[j]) for i,v in enumerate(verts) for j in range(3))
        assert err<1e-6
        report=dict(object_id=data['object_id'],vertices=len(verts),faces=len(g['faces']),coordinate_max_error_tiles=err,winding='unchanged; determinant +1 axis mapping',native_bounds=[g['vertices'] and [min(v[i] for v in g['vertices']) for i in range(3)],[max(v[i] for v in g['vertices']) for i in range(3)]],unsupported=['Native lighting and smooth vertex shading are not reproduced','Texture IDs/triangles retained in source JSON; no inferred UV mapping; textured faces marked purple','Face priority/depth bias/alpha/skins retained as source metadata, not Blender preview semantics','Complex native texture mapping arrays are omitted by existing revision decoder; no invented replacement'])
        reports.append(report);bpy.ops.wm.save_as_mainfile(filepath=str(a.output/f'{path.stem}.blend'))
    (a.output/'inspection-report.json').write_text(json.dumps(reports,indent=2));print('REFERENCE_BLENDER_INSPECTION_PASS',len(reports))
if __name__=='__main__':main()
