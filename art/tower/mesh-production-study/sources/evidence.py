"""Native compatibility experiments, pose proof and scene renders."""
import sys,json,copy,struct,shutil
from pathlib import Path
from PIL import Image
from run import ROOT,REPO,HERE,NATIVE,CACHE,BLENDER,run,views
sys.path.insert(0,str(HERE.parent/'asset-workbench'))
from native_codec import model
from kinematics import trace,frame_at

views(ROOT/'runs/creature-control','creature')
views(ROOT/'runs/creature-union','creature')
previous=ROOT/'runs/previous-best';previous.mkdir(exist_ok=True)
shutil.copy2(REPO/'art/tower/astra-benchmark/assets/D-sootsnout/asset.model',previous/'asset.model');views(previous,'creature')
for name in ['material-only','coarse-UV-bake','subdivided-UV-bake']:views(ROOT/'compatibility-v2'/name,'machine')
g=json.loads((ROOT/'runs/creature-final/geometry.json').read_text());flat=copy.deepcopy(g);flat['render_types']=[1]*len(g['faces']);out=ROOT/'compatibility-v2/flat-shading';out.mkdir(exist_ok=True);(out/'asset.model').write_bytes(model(flat));views(out,'creature')
folder=ROOT/'runs/creature-final'
run([NATIVE,CACHE,folder/'native-motion','render-own',folder/'asset.model',64,768],folder/'native-motion/render.log')
run([NATIVE,CACHE,folder/'motion','play',folder/'asset.model',folder/'native-motion/camera.txt',64,768,folder/'idle'],folder/'motion/render.log')
data=json.loads((folder/'idle/animation.json').read_text());duration=sum(f['length']for f in data['frames']);worst=0;count=0
for p in sorted((folder/'motion').glob('pose-*.json')):
    tick=int(p.stem.split('-')[1]);_,frame=frame_at(data['frames'],tick%duration);predicted,_,=trace(g,frame);actual=json.loads(p.read_text())['vertices'];f32=lambda x:struct.unpack('<f',struct.pack('<f',x))[0]
    assert len(predicted)==len(actual)
    err=max(abs(f32(x)-f32(y))for a,b in zip(predicted,actual)for x,y in zip(a,b));worst=max(worst,err);count+=1
assert worst==0
(folder/'motion-validation.json').write_text(json.dumps(dict(native_samples=count,max_coordinate_error=worst,duration_ticks=duration,proof_scope='Head/jaw classic rig only; no locomotion acceptance'),indent=2))
motion_scene=folder/'motion-scene.tsv';motion_scene.write_text('light\t64\t768\nowned\tasset.model\t0\t0\t0\t0\tidle\t0\n');frames=[]
for tick in range(0,60,2):
    dest=folder/'close-motion'/str(tick);run([NATIVE,CACHE,dest,'scene',motion_scene,folder/'quarter.camera',tick],dest/'render.log');frames.append(Image.open(dest/'scene.bmp').convert('RGB'))
frames[0].save(folder/'idle.gif',save_all=True,append_images=frames[1:],duration=40,loop=0)
print('RIG_PROOF',count,worst,flush=True)
ground=ROOT/'scene/ground'
run([BLENDER,'-b','--factory-startup','--python-exit-code','1','--python',HERE/'scene_floor.py','--',ground],ROOT/'logs/ground.log')
scene=ROOT/'scene/scene.tsv';rows=['light\t64\t768']
def add(kind,source,x,z,q=0,y=0):rows.append('\t'.join(map(str,[kind,source,x,y,z,q,'-',0])))
add('owned',ground/'asset.model',0,0)
add('owned',ROOT/'runs/architecture-final/asset.model',0,180)
add('owned',ROOT/'runs/machine-final/asset.model',-160,-35)
add('owned',ROOT/'runs/creature-final/asset.model',130,-80)
old=REPO/'art/tower/advanced-mastery/assets'
for x in [-255,255]:add('owned',old/'wall-bay/asset.model',x,225)
add('owned',old/'stock-rack/asset.model',-280,130)
add('owned',old/'supply-barrel/asset.model',265,150)
add('object',2097,220,30)
scene.write_text('\n'.join(rows)+'\n')
cams={'overview':'look 790 -880 -1050 0 -65 20 1000 1200 900','player-west':'player -90 -65 -15 1850 15300 380','player-east':'player 70 -65 -30 1900 1200 380','creature-detail':'look 390 -220 -440 130 -60 -80 850 1000 800'}
for name,cam in cams.items():
    path=scene.parent/(name+'.camera');path.write_text(cam);dest=scene.parent/'views'/name;run([NATIVE,CACHE,dest,'scene',scene,path],dest/'render.log');Image.open(dest/'scene.bmp').save(scene.parent/(name+'.png'))
print('SCENE_COMPLETE')
