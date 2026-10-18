"""Dedicated static reference/measurement project, prepared before mesh work."""
import bpy,json,math,sys
from pathlib import Path
from mathutils import Vector,Matrix
T=Path(__file__).resolve().parent;sys.path.insert(0,str(T.parent/'mesh-study'))
from mesh_tools import ellipsoid,material
R=T.parents[3]/'ashhorn-reference-private';C=json.loads((R/'constraints.json').read_text())
bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
scene=bpy.context.scene;scene.unit_settings.system='METRIC';scene.unit_settings.scale_length=1/128
def collection(name):
    c=bpy.data.collections.new(name);scene.collection.children.link(c);return c
landmarks=collection('01 Named anatomical estimates — uncertainties in constraints.json')
guides=collection('02 Original image guides — not NPC geometry')
framework=collection('03 Static measured framework — excluded from export')
cameras=collection('04 Reference-calibrated and orthographic working cameras')
for name,position in C['landmarks'].items():
    obj=bpy.data.objects.new(name,None);landmarks.objects.link(obj);obj.location=position;obj.empty_display_type='SPHERE';obj.empty_display_size=1.7;obj.show_in_front=True;obj['measurement_status']='Manual estimate + bounded multiview adjustment, not exact';obj['canonical_image_sha256']=json.loads((R/'references/preparation.json').read_text())['canonical_sha256']
chains=C['framework']['forelimbs']+C['framework']['hindlimbs']+[['nose','horn_root_pos','horn_tip_pos'],['nose','horn_root_neg','horn_break_neg'],['withers','spine_mid','spine_lumbar','tail_root'],C['framework']['tail_centerline']]
for idx,chain in enumerate(chains):
    data=bpy.data.curves.new('Measurement chain','CURVE');data.dimensions='3D';data.bevel_depth=.5;data.bevel_resolution=0;poly=data.splines.new('POLY');poly.points.add(len(chain)-1)
    for p,name in zip(poly.points,chain):p.co=(*C['landmarks'][name],1)
    obj=bpy.data.objects.new('Static chain '+str(idx),data);framework.objects.link(obj);obj.hide_render=True;obj['construction_only']=True
mat=material('Measurement cage only',(.12,.5,.46))
for name,c,r in [('Ribcage',C['framework']['ribcage_center'],C['framework']['ribcage_radii']),('Pelvis',C['framework']['pelvis_center'],C['framework']['pelvis_radii']),('Skull envelope',[0,-109,124],[25,34,20])]:
    o=ellipsoid('CAGE '+name,c,r,mat,segments=12,rings=8);o['construction_only']=True;o.hide_render=True;o.display_type='WIRE'
    for col in list(o.users_collection):col.objects.unlink(o)
    framework.objects.link(o)
rig=[]
for name,c in C['cameras'].items():
    eye=Vector(c['eye_author']);target=Vector(c['target_author']);forward=(target-eye).normalized();right=Vector(c['basis_author'][0]);up=Vector(c['basis_author'][1])
    data=bpy.data.cameras.new('Matched illustration '+name);data.sensor_fit='HORIZONTAL';data.sensor_width=36;data.lens=c['focal']*36/c['viewport'][0];data.shift_x=-c['offset_pixels'][0]/c['viewport'][0];data.shift_y=c['offset_pixels'][1]/c['viewport'][0];data.clip_end=10000
    obj=bpy.data.objects.new('Matched illustration '+name,data);cameras.objects.link(obj);obj.location=eye;obj.rotation_euler=forward.to_track_quat('-Z','Y').to_euler();obj['calibration_json']=json.dumps(c)
    image=bpy.data.images.load(str((R/'references'/(name+'-panel.png')).resolve()));image.pack();guide=bpy.data.objects.new('Original guide '+name,None);guides.objects.link(guide);guide.empty_display_type='IMAGE';guide.data=image;guide.empty_image_depth='BACK';guide.color[3]=.45;guide.hide_render=True
    plane_distance=(target-eye).length+260;guide.location=eye+forward*plane_distance+right*(-c['offset_pixels'][0])*plane_distance/c['focal']+up*c['offset_pixels'][1]*plane_distance/c['focal'];guide.rotation_euler=obj.rotation_euler;guide.empty_display_size=c['viewport'][0]*plane_distance/c['focal'];guide['original_pixels_unmodified']=True
    rig.append({'view':name,'camera_position':list(obj.location),'camera_rotation_euler':list(obj.rotation_euler),'lens_mm':data.lens,'shift':[data.shift_x,data.shift_y],'reference_position':list(guide.location),'reference_width_native':guide.empty_display_size,'image_size':list(image.size),'projection_assumption':'Estimated uniform perspective; image itself was never stretched'})
    if name=='front':scene.camera=obj;scene.render.resolution_x=c['viewport'][0];scene.render.resolution_y=c['viewport'][1]
for name,eye,scale in [('Front',(0,-600,88),210),('Left',(600,0,88),220),('Rear',(0,600,88),210)]:
    data=bpy.data.cameras.new('Orthographic measurement '+name);data.type='ORTHO';data.ortho_scale=scale;data.clip_end=10000;obj=bpy.data.objects.new('Orthographic measurement '+name,data);cameras.objects.link(obj);obj.location=eye;obj.rotation_euler=(Vector((0,0,88))-obj.location).to_track_quat('-Z','Y').to_euler()
text=bpy.data.texts.new('Canonical constraints and uncertainty');text.write(json.dumps(C,indent=2))
scene['reference_target']='Existing Ashhorn turnaround-v2; no generated replacement';scene['native_units_per_tile']=128
scene.render.resolution_percentage=100
bpy.ops.wm.save_as_mainfile(filepath=str((R/'reference-workspace.blend').resolve()))
(R/'reference-arrangement.json').write_text(json.dumps(rig,indent=2)+'\n')
print('REFERENCE_WORKSPACE_READY',len(C['landmarks']),'named landmarks;',len(rig),'calibrated guides;3 orthographic working cameras; no animation rig.')
