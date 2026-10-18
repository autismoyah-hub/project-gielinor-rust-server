"""Minimal compatible classic jaw/head proof on the refined sculpted mesh."""
import sys,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;sys.path[:0]=[str(HERE.parent/'asset-workbench')]
import bpy
from author_blender import actions
out=Path(sys.argv[-1]).resolve();g=json.loads((out/'geometry.json').read_text())
r=dict(rig=[dict(name='head',parent=None,origin=[0,-73,-63],groups=[1,2]),dict(name='jaw',parent='head',origin=[0,-64,-81],groups=[2])],skeleton_base=64570,frame_group_base=62370,actions={
    'idle':dict(duration_ticks=60,cadence_ticks=2,loop=True,keys=[dict(tick=t,channels=dict(head=[h,0,0],jaw=[j,0,0]))for t,h,j in [(0,0,0),(15,-2,-2),(30,1,-8),(45,2,-2),(60,0,0)]])})
actions(r,g,out);(out/'rig.json').write_text(json.dumps(r,indent=2));bpy.ops.wm.save_as_mainfile(filepath=str(out/'source-rigged.blend'))
