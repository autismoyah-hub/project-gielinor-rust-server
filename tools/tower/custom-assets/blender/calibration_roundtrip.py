"""Headless classic-model and ordered-transform Blender interchange.

The export reads mesh vertices, topology, materials, face attributes, vertex
groups and evaluated action channels. Original binary payloads are never used
as exported models or frames. Files produced by this study remain private.
"""
import bpy,json,sys,argparse
from pathlib import Path

def mesh_import(name,g):
    mesh=bpy.data.meshes.new(name);mesh.from_pydata([(x/128,z/128,-y/128)for x,y,z in g['vertices']],[],g['faces']);mesh.update()
    obj=bpy.data.objects.new(name,mesh);bpy.context.collection.objects.link(obj)
    colors={}
    for i,p in enumerate(mesh.polygons):
        color=g['colors'][i]
        if color not in colors:
            mat=bpy.data.materials.new(f'Native HSL {color}');mat['native_hsl']=color
            mesh.materials.append(mat);colors[color]=len(mesh.materials)-1
        p.material_index=colors[color]
    for key in ['render_types','priorities','alpha','face_skins','face_flags']:
        if g.get(key) is not None:
            attr=mesh.attributes.new(key,'INT','FACE')
            for v,n in zip(attr.data,g[key]):v.value=n
    skins=g.get('vertex_skins')
    if skins is not None:
        for label in sorted(set(skins)):
            group=obj.vertex_groups.new(name=f'native_{label}');group.add([i for i,v in enumerate(skins)if v==label],1.0,'REPLACE')
    obj['native_priority']=g['priority'];obj['has_skin_labels']=skins is not None
    return obj

def mesh_export(obj):
    mesh=obj.data
    vertices=[[v.co.x*128,-v.co.z*128,v.co.y*128]for v in mesh.vertices]
    require=all(abs(x-round(x))<1e-4 for v in vertices for x in v)
    if not require:raise ValueError('non-integral native export; no hidden quantization')
    out=dict(vertices=[list(map(round,v))for v in vertices],faces=[list(f.vertices)for f in mesh.polygons],colors=[int(mesh.materials[f.material_index]['native_hsl'])for f in mesh.polygons],priority=obj['native_priority'])
    for key in ['render_types','priorities','alpha','face_skins','face_flags']:
        out[key]=[v.value for v in mesh.attributes[key].data]if key in mesh.attributes else None
    out['vertex_skins']=None
    if obj['has_skin_labels']:
        labels=[]
        for v in mesh.vertices:
            groups=[g for g in v.groups if g.weight>.999]
            if len(groups)!=1:raise ValueError('classic skin requires exactly one group per vertex')
            labels.append(int(obj.vertex_groups[groups[0].group].name.split('_')[1]))
        out['vertex_skins']=labels
    return out

def main():
    p=argparse.ArgumentParser();p.add_argument('--source',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args(sys.argv[sys.argv.index('--')+1:]);a.source=a.source.resolve();a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=True)
    bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
    for name in ['static','npc']:
        obj=mesh_import(name,json.loads((a.source/f'{name}.json').read_text()))
        (a.output/f'{name}.json').write_text(json.dumps(mesh_export(obj)))
    rig=bpy.data.objects.new('Classic ordered-transform action rig',None);bpy.context.collection.objects.link(rig)
    rig['contract']='Native labels are ordered operations, not an ordinary parented armature. Types 0 pivot,1 translate,2 rotate Z-X-Y,3 scale,5 face alpha. Native integer evaluation remains authoritative.'
    rig.animation_data_create()
    for ai,name in enumerate(['idle','walk','attack','hit','death']):
        data=json.loads((a.source/name/'animation.json').read_text());rows=data['frames'];action=bpy.data.actions.new(name);rig.animation_data.action=action
        active=sorted({t[0]for row in rows for t in row['transforms']})
        for label in active:rig[f't_{label}']=[0.,0.,0.]
        for row in rows:
            values=dict(row['transforms'])
            for label in active:
                rig[f't_{label}']=list(map(float,values.get(label,[128]*3 if row['types'][label]==3 else [0]*3)))
                rig.keyframe_insert(data_path=f'["t_{label}"]',frame=row['index']+1,group=f'Native transform {label}')
        for layer in action.layers:
            for strip in layer.strips:
                for bag in strip.channelbags:
                    for curve in bag.fcurves:
                        for key in curve.keyframe_points:key.interpolation='CONSTANT'
        # Preserve which operations existed, but read every value from Blender.
        export=[]
        for row in rows:
            bpy.context.scene.frame_set(row['index']+1)
            clone={k:v for k,v in row.items()if k!='transforms'}
            clone['transforms']=[[label,[round(x)for x in rig[f't_{label}']]]for label,_ in row['transforms']]
            export.append(clone)
        out=a.output/name;out.mkdir(exist_ok=True)
        result=dict(source_sequence=data['source_sequence'],custom_sequence=30000+data['source_sequence'],custom_skeleton=64000+ai,custom_frame_group=60000+ai,frames=export)
        (out/'animation.json').write_text(json.dumps(result))
        action['native_sequence']=result['custom_sequence'];action['native_frames']=json.dumps([dict(length=r['length'],active=[t[0]for t in r['transforms']])for r in rows])
        action['skeleton_types']=json.dumps(rows[0]['types']);action['skeleton_labels']=json.dumps(rows[0]['labels']);action['custom_skeleton']=result['custom_skeleton'];action['custom_frame_group']=result['custom_frame_group']
        action.use_fake_user=True
    bpy.ops.wm.save_as_mainfile(filepath=str(a.output/'roundtrip.blend'))
    print('BLENDER_GEOMETRY_AND_ACTION_ROUNDTRIP_COMPLETE')
if __name__=='__main__':main()
