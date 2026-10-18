"""Saved Blender actions for exact classic-animation interchange."""
import bpy,json,re
from pathlib import Path
def policy(text):
    result={}
    for field,default in [('loop_frames',-1),('priority',5),('max_loops',99),('precedence',0),('priority_moving',0),('restart',2)]:
        m=re.search(r'\b'+field+r': (-?\d+)',text);result[field]=int(m.group(1))if m else default
    m=re.search(r'interleave: Some\(\s*\[([^]]+)\]',text,re.S);result['interleave']=[int(x)for x in re.findall(r'\d+',m.group(1))if int(x)!=9999999]if m else [];result['stretch']='stretch: true'in text;return result
def import_action(name,path,index):
    data=json.loads(path.read_text())
    if data.get('cached_animation')is not None:raise ValueError('Cached weighted action authoring is not supported')
    rows=data['frames'];types,labels=rows[0]['types'],rows[0]['labels']
    if any(r['types']!=types or r['labels']!=labels for r in rows):raise ValueError('Action changes classic skeleton schema')
    control=bpy.data.objects.new(name,None);bpy.context.collection.objects.link(control);control['classic_action_export']=True;control['relative_action_path']=str(path.parent.parent.name+'/'+path.parent.name);control.animation_data_create();action=bpy.data.actions.new(name);control.animation_data.action=action;action.use_fake_user=True
    active=sorted({label for r in rows for label,_ in r['transforms']})
    for row in rows:
        values=dict(row['transforms'])
        for label in active:
            control[f't_{label}']=list(map(float,values.get(label,[128]*3 if types[label]==3 else [0]*3)));control.keyframe_insert(data_path=f'["t_{label}"]',frame=row['index']+1)
    for layer in action.layers:
        for strip in layer.strips:
            for bag in strip.channelbags:
                for curve in bag.fcurves:
                    for p in curve.keyframe_points:p.interpolation='CONSTANT'
    action['native_schema']=json.dumps(dict(types=types,labels=labels));action['native_frames']=json.dumps([dict(index=r['index'],length=r['length'],active=[label for label,_ in r['transforms']])for r in rows]);action['native_policy']=json.dumps(policy((path.parent/'sequence.txt').read_text()));action['custom_skeleton_id']=64300+index;action['custom_frame_group']=62100+index;action['source_sequence']=data['source_sequence'];control['action_name']=action.name
def export_action(control):
    action=bpy.data.actions[control['action_name']];control.animation_data.action=action;schema=json.loads(action['native_schema']);frames=[]
    for row in json.loads(action['native_frames']):
        bpy.context.scene.frame_set(row['index']+1);frames.append(dict(index=row['index'],length=row['length'],types=schema['types'],labels=schema['labels'],transforms=[[label,[round(x)for x in control[f't_{label}']]]for label in row['active']]))
    return dict(skeleton_id=action['custom_skeleton_id'],frame_group=action['custom_frame_group'],source_sequence=action['source_sequence'],policy=json.loads(action['native_policy']),frames=frames,not_encoded=['sound events and non-rendering sequence fields not used by this geometry/motion study'])
