"""Headless component authoring, saved Blender export, and native-group actions."""
import bpy,bmesh,json,sys,math,argparse,copy,hashlib
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent))
from primitives import make
from blender_io import import_model,export_model
from kinematics import compile_joints
from native_codec import model,classic
from semantic_materials import colors as semantic_colors
from mesh_validation import clean_owned,degenerate_faces
from owned_textures import merge as merge_textures,apply as apply_texture
def export_asset(recipe):
    fields=['render_types','priorities','alpha','face_skins','vertex_skins','face_flags'];g=dict(vertices=[],faces=[],colors=[],priority=0,texture_types=[],texture_triangles=[],face_textures=None,texture_coords=None,metadata=None);g.update({k:[]for k in fields});components={}
    for spec in recipe['components']:
        obj=bpy.data.objects[spec['name']]
        if not obj.get('owned_component'):raise ValueError('Reference-import mesh is not an owned authoring component: '+obj.name)
        part=export_model(obj);offset=len(g['vertices']);firstface=len(g['faces']);g['vertices']+=part['vertices'];g['faces'] += [[x+offset for x in f]for f in part['faces']];g['colors']+=part['colors']
        if recipe.get('geometry_validation',{}).get('remove_zero_area_faces')and degenerate_faces(part['vertices'],part['faces']):raise ValueError('Edited owned mesh contains zero-area faces; clean or rebuild the mesh before export: '+spec['name'])
        merge_textures(g,part,offset,firstface)
        for key in fields:
            n=len(part['vertices'])if key=='vertex_skins'else len(part['faces']);g[key]+=part.get(key)or [0]*n
        components[spec['name']]=dict(vertices=list(range(offset,len(g['vertices']))),faces=list(range(firstface,len(g['faces']))),group=spec.get('group',0),role=spec.get('role',spec['name']),material=spec['material'],material_family=recipe['materials'][spec['material']].get('family'),parameters=spec.get('parameters',{}),geometry_sha256=hashlib.sha256(json.dumps(part,sort_keys=True).encode()).hexdigest())
        components[spec['name']]['geometry_validation']=json.loads(obj.get('geometry_validation_report','{}'))
    if not(recipe.get('rig')or recipe.get('preserve_authored_groups')):g['vertex_skins']=None;g['face_skins']=None
    g['components']=components;return g
def build(recipe,palette):
    bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
    for spec in recipe['components']:
        vs,fs=make(spec);validation={}
        if recipe.get('geometry_validation',{}).get('remove_zero_area_faces'):vs,fs,validation=clean_owned(vs,fs)
        material=recipe['materials'][spec['material']];g=dict(vertices=vs,faces=fs,colors=semantic_colors(vs,fs,material,spec),render_types=[material.get('render_type',0)]*len(fs),priorities=[material.get('priority',0)]*len(fs),priority=0,alpha=[material.get('alpha',0)]*len(fs),face_skins=[spec.get('face_group',spec.get('group',0))]*len(fs),vertex_skins=[spec.get('group',0)]*len(vs),face_flags=[0]*len(fs))
        apply_texture(g,material,spec)
        obj=import_model(spec['name'],g,palette);obj['owned_component']=True;obj['role']=spec.get('role',spec['name']);obj['authoring_spec']=json.dumps(spec)
        obj['geometry_validation_report']=json.dumps(validation)
        bm=bmesh.new();bm.from_mesh(obj.data);bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bm.to_mesh(obj.data);bm.free()
def actions(recipe,g,out,reopen=False):
    if not recipe.get('rig'):return
    control=bpy.data.objects.get('Native action controls')or bpy.data.objects.new('Native action controls',None)
    if control.name not in bpy.context.collection.objects:bpy.context.collection.objects.link(control)
    control.animation_data_create();rig=recipe['rig']
    authored_actions=set()
    for ai,(name,spec)in enumerate(recipe.get('actions',{}).items()):
        if not reopen and bpy.data.actions.get(name):
            old=bpy.data.actions[name]
            if not old.get('native_rig'):raise ValueError('Existing action name is not owned by this authoring system')
            old.name=name+' (previous authored version)';old.use_fake_user=True
        action=bpy.data.actions.get(name)if reopen else bpy.data.actions.new(name)
        if action is None:raise ValueError('Saved action missing: '+name)
        action.use_fake_user=True;control.animation_data.action=action
        for joint in rig:control[joint['name']]=[0.,0.,0.]
        control['root_translation']=[0.,0.,0.]
        authored_keys=spec.get('keys',[])
        if spec.get('bridge_controls')and not reopen:
            authored_keys=[]
            for endpoint,frame_tick in [('source',0),('destination',spec['duration_ticks']-spec.get('cadence_ticks',2))]:
                source=spec['bridge_controls'][endpoint];source_action=bpy.data.actions.get(source)
                if source_action is None or source not in authored_actions:raise ValueError('Bridge source action must be authored first: '+source)
                control.animation_data.action=source_action;bpy.context.scene.frame_set(1);channels={j['name']:list(control[j['name']])for j in rig};translation=list(map(round,control['root_translation']))
                if recipe['actions'][source].get('grounded'):
                    _,posed,_=compile_joints(g,rig,channels,translation);translation[1]-=max(v[1]for v in posed)
                authored_keys.append(dict(tick=frame_tick,channels=channels,translation=translation))
            control.animation_data.action=action
        for key in ([]if reopen else authored_keys):
            for joint in rig:
                control[joint['name']]=list(map(float,key.get('channels',{}).get(joint['name'],[0]*3)));control.keyframe_insert(data_path=f'["{joint["name"]}"]',frame=key['tick']+1,group=joint['name'])
            control['root_translation']=list(map(float,key.get('translation',[0]*3)));control.keyframe_insert(data_path='["root_translation"]',frame=key['tick']+1)
        for layer in action.layers:
            for strip in layer.strips:
                for bag in strip.channelbags:
                    for curve in bag.fcurves:
                        for point in curve.keyframe_points:point.interpolation=spec.get('interpolation','LINEAR')
        if spec.get('gait')and not reopen:
            from motion_authoring import gait_controls
            guide=[];duration=spec['duration_ticks'];step=spec.get('cadence_ticks',3)
            for tick in range(0,duration,step):
                bpy.context.scene.frame_set(tick+1);guide.append((tick,{j['name']:list(control[j['name']])for j in rig},list(map(round,control['root_translation']))))
            baked=[];diagnostics=[]
            for tick,channels,translation in guide:
                channels,translation,targets=gait_controls(g,rig,channels,translation,spec,tick);baked.append((tick,channels,translation));diagnostics.append(dict(tick=tick,targets=targets))
            if spec.get('loop'):baked.append((duration,baked[0][1],baked[0][2]))
            for layer in action.layers:
                for strip in layer.strips:
                    for bag in strip.channelbags:
                        for curve in bag.fcurves:curve.keyframe_points.clear()
            for tick,channels,translation in baked:
                for joint in rig:
                    control[joint['name']]=list(map(float,channels.get(joint['name'],[0]*3)));control.keyframe_insert(data_path=f'["{joint["name"]}"]',frame=tick+1,group=joint['name'])
                control['root_translation']=list(map(float,translation));control.keyframe_insert(data_path='["root_translation"]',frame=tick+1)
            action['contact_guide']=json.dumps(spec['gait']);(out/(name+'-contact-solve.json')).write_text(json.dumps(diagnostics,indent=2))
        frames=[];duration=spec['duration_ticks'];step=spec.get('cadence_ticks',3);samples=list(range(0,duration,step))
        for i,tick in enumerate(samples):
            bpy.context.scene.frame_set(tick+1);channels={j['name']:list(control[j['name']])for j in rig};translation=list(map(round,control['root_translation']));frame,posed,origins=compile_joints(g,rig,channels,translation)
            if spec.get('grounded'):
                translation[1]-=max(v[1]for v in posed);frame,posed,origins=compile_joints(g,rig,channels,translation)
            contacts=spec.get('foot_contacts',[])
            active=[c for c in contacts if ((tick/duration+c.get('phase_offset',0))%1)<c.get('stance_fraction',.5)]
            if active:
                indices=[v for c in active for n in c['components']for v in g['components'][n]['vertices']]
                translation[1]-=max(posed[v][1]for v in indices);frame,posed,origins=compile_joints(g,rig,channels,translation)
            frame.update(index=i,length=min(step,duration-tick));frames.append(frame)
            (out/f'{name}-authored-{tick:04}.json').write_text(json.dumps(dict(vertices=posed,joint_origins=origins)))
        if spec.get('terminal_zero'):
            labels=sorted(set(g['vertex_skins']));index=len(frames[0]['types'])
            for row in frames:row['types'] += [0,3];row['labels'] += [[],labels];row['transforms'] += [[index,[0,0,0]],[index+1,[128,128,128]]]
            last=copy.deepcopy(frames[-1]);last['index']=len(frames);last['length']=spec.get('terminal_hold',1);last['transforms'][-1][1]=[0,0,0];frames.append(last)
        elif spec.get('terminal_pose_hold'):
            last=copy.deepcopy(frames[-1]);last['index']=len(frames);last['length']=spec['terminal_pose_hold'];frames.append(last)
        data=dict(skeleton_id=recipe.get('skeleton_base',64500)+ai,frame_group=recipe.get('frame_group_base',62000)+ai,frames=frames,policy=dict(loop_frames=len(frames)if spec.get('loop')else -1,priority=spec.get('priority',5),max_loops=99,interleave=spec.get('interleave',[])))
        classic(data,out/name);(out/name/'animation.json').write_text(json.dumps(data));action['native_policy']=json.dumps(data['policy']);action['duration_ticks']=duration;action['native_rig']=json.dumps(rig)
        authored_actions.add(name)
def main():
    p=argparse.ArgumentParser();p.add_argument('recipe',type=Path);p.add_argument('output',type=Path);p.add_argument('--palette',type=Path);p.add_argument('--reopen',action='store_true');p.add_argument('--replace-actions',action='store_true');a=p.parse_args(sys.argv[sys.argv.index('--')+1:]);recipe=json.loads(a.recipe.read_text());a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=True);palette=json.loads(a.palette.read_text())if a.palette else None
    if recipe.get('rig_validation'):
        from rig_contracts import validate
        validate(recipe)
    if not a.reopen:build(recipe,palette)
    g=export_asset(recipe);(a.output/'geometry.json').write_text(json.dumps(g));(a.output/'asset.model').write_bytes(model(g));(a.output/'recipe.json').write_text(json.dumps(recipe,indent=2));actions(recipe,g,a.output,a.reopen and not a.replace_actions);bpy.ops.wm.save_as_mainfile(filepath=str(a.output/'asset.blend'));print('OWNED_COMPONENT_AUTHORING_PASS',recipe['name'],len(g['vertices']),len(g['faces']))
if __name__=='__main__':main()
