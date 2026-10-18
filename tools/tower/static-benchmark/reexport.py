"""Static editable-source reproduction check; no rig or action generation."""
import sys,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;sys.path[:0]=[str(HERE.parent/'mesh-study'),str(HERE.parent/'asset-workbench')]
import bpy
from mesh_tools import export
from native_codec import model
out=Path(sys.argv[-1]).resolve();assert not bpy.data.actions;assert not any(o.type=='ARMATURE'for o in bpy.context.scene.objects)
g=export([o for o in bpy.context.scene.objects if o.type=='MESH'],out,model);g['vertex_skins']=None;(out/'geometry.json').write_text(json.dumps(g));(out/'asset.model').write_bytes(model(g));print('STATIC_SOURCE_REEXPORTED')
