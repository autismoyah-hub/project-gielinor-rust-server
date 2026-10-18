"""Verify that editable saved meshes reproduce the published native payload."""
import sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;sys.path[:0]=[str(HERE),str(HERE.parent/'asset-workbench')]
import bpy
from mesh_tools import export
from native_codec import model
out=Path(sys.argv[-1]).resolve();export([o for o in bpy.context.scene.objects if o.type=='MESH'and o.get('owned')],out,model)
