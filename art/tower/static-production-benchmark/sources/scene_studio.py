import sys,json,colorsys
from pathlib import Path
HERE=Path(__file__).resolve().parent;sys.path.insert(0,str(HERE.parent/'asset-workbench'))
import bpy
from mathutils import Matrix,Vector
from blender_io import import_model
root=Path(sys.argv[-1]).resolve();repo=HERE.parents[2];sources={'ground':root/'scene/ground/source.blend','workstation':root/'runs/workstation-final2/source.blend','machine':root/'runs/machine-final/source.blend','creature':root/'runs/creature-final2/source.blend','arch':repo/'art/tower/high-fidelity-mastery/assets/architecture/source.blend'}
bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
for p in json.loads((root/'scene/placements.json').read_text()):
    x,y,z=p['position'];pos=Vector((x,z,-y))
    if p['label']in sources:
        with bpy.data.libraries.load(str(sources[p['label']]),link=False)as(src,dst):dst.objects=src.objects
        for o in dst.objects:
            if o and o.type=='MESH':bpy.context.collection.objects.link(o);o.location+=pos
    else:
        g=json.loads((Path(p['path']).parent/'geometry.json').read_text());o=import_model(p['label'],g);o.matrix_world=Matrix.Translation(pos)@Matrix.Scale(128,4)
        for m in o.data.materials:
            h=int(m['native_hsl']);m.diffuse_color=(*colorsys.hls_to_rgb((h>>10)/64,(h&127)/127,((h>>7)&7)/8),1)
bpy.ops.wm.save_as_mainfile(filepath=str(root/'scene/showcase.blend'))
