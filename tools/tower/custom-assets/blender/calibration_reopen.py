"""Export actual saved Blender geometry/action channels in a fresh process."""
import bpy,sys,json
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent))
from calibration_roundtrip import mesh_export
out=Path(sys.argv[sys.argv.index('--')+1]).resolve();out.mkdir(parents=True,exist_ok=True)
if '--witness' in sys.argv:
    bpy.data.objects['static'].data.vertices[0].co.x+=1/128
    material=bpy.data.objects['static'].data.materials[0];material['native_hsl']+=1
    action=bpy.data.actions['walk'];curve=action.layers[0].strips[0].channelbags[0].fcurves[0];curve.keyframe_points[0].co.y+=1
for name in ['static','npc']:(out/f'{name}.json').write_text(json.dumps(mesh_export(bpy.data.objects[name])))
rig=bpy.data.objects['Classic ordered-transform action rig']
for name in ['idle','walk','attack','hit','death']:
    action=bpy.data.actions[name];rig.animation_data.action=action
    types=json.loads(action['skeleton_types']);labels=json.loads(action['skeleton_labels']);meta=json.loads(action['native_frames']);frames=[]
    for i,row in enumerate(meta):
        bpy.context.scene.frame_set(i+1)
        frames.append(dict(index=i,length=row['length'],types=types,labels=labels,transforms=[[k,[round(x)for x in rig[f't_{k}']]]for k in row['active']]))
    (out/name).mkdir(exist_ok=True);(out/name/'animation.json').write_text(json.dumps(dict(custom_skeleton=action['custom_skeleton'],custom_frame_group=action['custom_frame_group'],frames=frames)))
print('SAVED_BLENDER_REOPEN_EXPORT_COMPLETE')
