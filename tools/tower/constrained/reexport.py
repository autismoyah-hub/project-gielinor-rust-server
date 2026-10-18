"""Byte-reproduction export of final surfaces; excludes hidden construction cages."""
import sys,json
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
import surface
import bpy
out=Path(sys.argv[-1]);out.mkdir(parents=True,exist_ok=False)
assert not bpy.data.actions and not any(o.type=='ARMATURE'for o in bpy.context.scene.objects)
objects=[o for o in bpy.context.scene.objects if o.type=='MESH'and not o.get('construction_only',False)]
g=surface.export(objects,out,surface.model);g['vertex_skins']=None;(out/'geometry.json').write_text(json.dumps(g));(out/'asset.model').write_bytes(surface.model(g))
print('STATIC_FINAL_SOURCE_REEXPORTED',len(objects))
