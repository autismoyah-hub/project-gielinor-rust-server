"""Saved-source export check using the preserved native interchange."""
import sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;sys.path[:0]=[str(HERE.parent/'mesh-study'),str(HERE.parent/'asset-workbench')]
import bpy
from mesh_tools import export
from native_codec import model
export([o for o in bpy.context.scene.objects if o.type=='MESH'],Path(sys.argv[-1]).resolve(),model)
