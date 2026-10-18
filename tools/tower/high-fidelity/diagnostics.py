"""Native pose-based contact, turntable, and fixed material/shading evidence."""
from review import *
import copy,math,statistics
sys.path.insert(0,str(HERE.parent/'asset-workbench'))
from native_codec import model
folder=ROOT/'runs/creature-final';g=json.loads((folder/'geometry-rigged.json').read_text());report={};rig=json.loads((folder/'rig.json').read_text());gait=rig['actions']['walk']['gait'];contacts=[];airborne=0;penetration=0
for tick in range(64):
    vs=json.loads((folder/'motion/walk'/('pose-%04d.json'%tick)).read_text())['vertices'];penetration=max(penetration,max(p[1]for p in vs));airborne+=max(p[1]for p in vs)<-2
    for limb in gait['limbs']:
        phase=((tick//2*2)/64+limb['phase_offset'])%1
        if phase>=gait['stance_fraction']:continue
        ids=[v for n in limb['components']for v in g['components'][n]['vertices']];height=max(vs[v][1]for v in ids);contacts.append(abs(height))
report['walk']=dict(native_frames=64,max_ground_penetration=max(0,penetration),frames_fully_airborne=airborne,mean_stance_surface_height_error=statistics.mean(contacts),max_stance_surface_height_error=max(contacts),note='Rigid group IK and native integer lattice; stance error is not a promise of final locomotion polish.')
death=json.loads((folder/'motion/death/pose-0160.json').read_text())['vertices'];lowest=max(v[1]for v in death);torso=max(death[i][1]for i in g['components']['torso_neck_sculpt']['vertices'])
surface=sorted([(max(death[i][1]for i in c['vertices']),n)for n,c in g['components'].items()],reverse=True)
report['death']=dict(final_native_tick=160,torso_clearance_from_lowest_point=lowest-torso,support_components=[n for h,n in surface if h>=lowest-1],max_ground_penetration=max(0,lowest),note='Final pose is held for 60 ticks; side contact uses the shoulder/cranial surface.')
for name in ['flat','single-material']:
    target=ROOT/'comparisons'/name;target.mkdir(parents=True,exist_ok=True);test=copy.deepcopy(g)
    if name=='flat':test['render_types']=[1]*len(test['faces'])
    else:test['colors']=[17578]*len(test['faces'])
    (target/'asset.model').write_bytes(model(test));views(target,'creature')
frames=[]
for a in range(0,360,45):
    rad=math.radians(a);camera=folder/('turn-'+str(a)+'.camera');camera.write_text(f'look {math.sin(rad)*390} -210 {-math.cos(rad)*390} 0 -60 0 900 1000 800');dest=folder/'turntable'/str(a);run([NATIVE,CACHE,dest,'scene',folder/'scene.tsv',camera],dest/'log.txt');frames.append(Image.open(dest/'scene.bmp').convert('RGB'));frames[-1].save(folder/('turn-'+str(a)+'.png'))
frames[0].save(folder/'turntable.gif',save_all=True,append_images=frames[1:],duration=200,loop=0)
(ROOT/'diagnostics.json').write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2))
