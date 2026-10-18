"""Reusable Cycles studio: GPU selection, three-point light, clay/topology views.

Presentation-only scene transforms. Never used as native acceptance evidence.
"""
import bpy,sys,math,json
from pathlib import Path
from mathutils import Vector,Matrix
args=sys.argv[sys.argv.index('--')+1:];out=Path(args[0]).resolve();mode=args[1]if len(args)>1 else'beauty';out.mkdir(parents=True,exist_ok=True)
scene=bpy.context.scene;scene.render.engine='CYCLES';scene.cycles.samples=32;scene.cycles.use_denoising=True
prefs=bpy.context.preferences.addons['cycles'].preferences;devices=[];selected='CPU'
try:
    prefs.compute_device_type='CUDA';prefs.get_devices()
    for d in prefs.devices:
        d.use=d.type=='CUDA';devices.append(dict(name=d.name,type=d.type,selected=d.use))
    if any(d['selected']for d in devices):scene.cycles.device='GPU';selected='CUDA'
except Exception as e:devices.append(dict(error=str(e)))
for o in list(scene.objects):
    if o.type=='MESH':
        o.matrix_world=Matrix.Scale(1/128,4)@o.matrix_world
        for p in o.data.polygons:
            p.use_smooth=not bool(o.data.materials[p.material_index].get('native_flat',False))
for m in bpy.data.materials:
    rgb=list(m.diffuse_color[:3]);m.use_nodes=True;bs=m.node_tree.nodes.get('Principled BSDF');linear=lambda x:x/12.92 if x<=.04045 else((x+.055)/1.055)**2.4
    bs.inputs['Base Color'].default_value=(*(linear(x)for x in rgb),1);bs.inputs['Roughness'].default_value=.75
    if mode=='clay':bs.inputs['Base Color'].default_value=(.32,.34,.35,1)
    if mode=='topology':
        wire=m.node_tree.nodes.new('ShaderNodeWireframe');wire.inputs['Size'].default_value=.005;mix=m.node_tree.nodes.new('ShaderNodeMixRGB');mix.inputs[1].default_value=(.4,.44,.42,1);mix.inputs[2].default_value=(.025,.035,.035,1);m.node_tree.links.new(wire.outputs[0],mix.inputs[0]);m.node_tree.links.new(mix.outputs[0],bs.inputs['Base Color'])
vs=[o.matrix_world@v.co for o in scene.objects if o.type=='MESH'for v in o.data.vertices];lo=Vector(tuple(min(v[i]for v in vs)for i in range(3)));hi=Vector(tuple(max(v[i]for v in vs)for i in range(3)));target=(lo+hi)/2;extent=max(hi-lo)
def aim(o,p):o.rotation_euler=(Vector(p)-o.location).to_track_quat('-Z','Y').to_euler()
def area(name,location,power,size):
    data=bpy.data.lights.new(name,'AREA');data.energy=power*(extent/2.5)**2;data.shape='DISK';data.size=size;o=bpy.data.objects.new(name,data);scene.collection.objects.link(o);o.location=target+Vector(location)*extent;aim(o,target)
area('Key',(-1.1,-1.6,2),350,extent*1.4);area('Fill',(1.5,-.5,1),120,extent*1.5);area('Rim',(.3,1.4,1.6),350,extent)
scene.world.use_nodes=True;scene.world.node_tree.nodes['Background'].inputs[0].default_value=(.13,.15,.17,1);scene.world.node_tree.nodes['Background'].inputs[1].default_value=.3
bpy.ops.mesh.primitive_plane_add(size=200,location=(0,0,max(-.02,lo.z-.02)));floor=bpy.context.object;fm=bpy.data.materials.new('Neutral studio floor');fm.use_nodes=True;fm.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value=(.035,.045,.05,1);fm.node_tree.nodes['Principled BSDF'].inputs['Roughness'].default_value=.8;floor.data.materials.append(fm)
data=bpy.data.cameras.new('Studio camera');camera=bpy.data.objects.new('Studio camera',data);scene.collection.objects.link(camera);scene.camera=camera;data.lens=52;camera.location=target+Vector((1,-1.5,.9))*extent*1.02;aim(camera,target)
scene.render.resolution_x=1100;scene.render.resolution_y=850;scene.render.resolution_percentage=100;scene.view_settings.view_transform='AgX';scene.render.image_settings.file_format='PNG';scene.render.filepath=str(out/(mode+'.png'))
if mode in ['front','side','rear']:
    data.type='ORTHO';data.ortho_scale=extent*1.35;camera.location=target+Vector({'front':(0,-5,0),'side':(5,0,0),'rear':(0,5,0)}[mode])*extent;aim(camera,target)
if mode.startswith('turn-'):
    angle=math.radians(int(mode.split('-')[1]));camera.location=target+Vector((math.sin(angle)*1.8,-math.cos(angle)*1.8,.9))*extent;aim(camera,target)
bpy.ops.render.render(write_still=True)
(out/(mode+'-device.json')).write_text(json.dumps(dict(engine='Cycles',backend=selected,devices=devices,samples=scene.cycles.samples,source_scale='Presentation normalized to 128 units/m',native_acceptance=False),indent=2))
bpy.ops.wm.save_as_mainfile(filepath=str(out/(mode+'-studio.blend')))
