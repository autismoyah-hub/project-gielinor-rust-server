"""Fixed native-camera Cycles inspection, with reference-fixed light placement.

Supplementary artistic evidence only. No candidate-dependent camera fitting.
"""
import sys,json,math
from pathlib import Path
import bpy
from mathutils import Vector,Matrix
sys.path.insert(0,str(Path(__file__).resolve().parent))
from landmarks import camera as parse_camera
args=sys.argv[sys.argv.index('--')+1:];source,config_file,kind,out=args;out=Path(out);out.mkdir(parents=True,exist_ok=True)
config=json.loads(Path(config_file).read_text());case=config['cases'][kind];native=Path(source).parent/'native'
bpy.ops.wm.open_mainfile(filepath=str(Path(source).resolve()));scene=bpy.context.scene
scene.render.engine='CYCLES';scene.cycles.samples=32;scene.cycles.use_denoising=True
backend='CPU';devices=[]
try:
    prefs=bpy.context.preferences.addons['cycles'].preferences;prefs.compute_device_type='CUDA';prefs.get_devices()
    for device in prefs.devices:device.use=device.type=='CUDA';devices.append({'name':device.name,'type':device.type,'selected':device.use})
    if any(x['selected']for x in devices):scene.cycles.device='GPU';backend='CUDA'
except Exception as error:devices.append({'error':str(error)})
for o in scene.objects:
    if o.type=='MESH' and not o.get('construction_only',False):o.matrix_world=Matrix.Scale(1/128,4)@o.matrix_world
for mat in bpy.data.materials:
    rgb=mat.diffuse_color[:3];mat.use_nodes=True;node=mat.node_tree.nodes.get('Principled BSDF')
    linear=lambda x:x/12.92 if x<=.04045 else((x+.055)/1.055)**2.4
    node.inputs['Base Color'].default_value=(*(linear(x)for x in rgb),1);node.inputs['Roughness'].default_value=.78
target=Vector((0,0,case['height']/128));bounds=case['metadata']['world_bounds'];extent=max(b-a for a,b in zip(bounds[0],bounds[1]))/128
def aim(obj,point):obj.rotation_euler=(Vector(point)-obj.location).to_track_quat('-Z','Y').to_euler()
for name,location,energy,size in [('Key',(-1.1,-1.6,2),350,1.4),('Fill',(1.5,-.5,1),120,1.5),('Rim',(.3,1.4,1.6),350,1)]:
    data=bpy.data.lights.new(name,'AREA');data.energy=energy*(extent/2.5)**2;data.shape='DISK';data.size=size*extent;obj=bpy.data.objects.new(name,data);scene.collection.objects.link(obj);obj.location=target+Vector(location)*extent;aim(obj,target)
scene.world.use_nodes=True;scene.world.node_tree.nodes['Background'].inputs[0].default_value=(.13,.15,.17,1);scene.world.node_tree.nodes['Background'].inputs[1].default_value=.3
bpy.ops.mesh.primitive_plane_add(size=200,location=(0,0,-.02));floor=bpy.context.object;mat=bpy.data.materials.new('Neutral studio floor');mat.diffuse_color=(.035,.045,.05,1);floor.data.materials.append(mat)
data=bpy.data.cameras.new('Fixed native reference camera');obj=bpy.data.objects.new('Fixed native reference camera',data);scene.collection.objects.link(obj);scene.camera=obj;data.sensor_fit='HORIZONTAL';data.sensor_width=36
scene.view_settings.view_transform='AgX';scene.render.image_settings.file_format='PNG';scene.render.resolution_percentage=100
settings={}
for view in ['front','side','rear','quarter','above','gameplay']:
    text=(native/(view+'.camera')).read_text();eye,rotation,focal,width,height=parse_camera(text);position=Vector((eye[0],eye[2],-eye[1]))/128;forward=rotation[2];direction=Vector((forward[0],forward[2],-forward[1]));obj.location=position;aim(obj,position+direction)
    data.lens=focal*36/width;scene.render.resolution_x=width;scene.render.resolution_y=height;scene.render.filepath=str((out/(view+'.png')).resolve());bpy.ops.render.render(write_still=True)
    settings[view]={'native_definition':text,'position_native_units':eye.tolist(),'basis_native':rotation.tolist(),'focal_pixels':focal,'viewport':[width,height]}
    print('STUDIO_FIXED',kind,view,flush=True)
(out/'settings.json').write_text(json.dumps({'backend':backend,'devices':devices,'samples':32,'camera_source':'Frozen reference-camera files; no candidate-dependent alignment','lighting':'Fixed reference bounds and origin; three area lights','views':settings,'authority':'Supplementary Cycles/AgX; unchanged Rust renderer is final native authority'},indent=2))
bpy.ops.wm.save_as_mainfile(filepath=str((out/'matched-studio.blend').resolve()))
