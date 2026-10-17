"""Normal client gateway actions and genuine native raster captures only."""
import csv
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys
import time

root=Path(os.environ.get('TOWER_EXPERIMENT_ROOT',Path(__file__).resolve().parent)).resolve()
sys.path.insert(0,str(root/'rust-server/tools/tower'))
from tower_gateway import Gateway
directory=root/(sys.argv[1] if len(sys.argv)>1 else 'native-session')
out=root/'evidence/native'
out.mkdir(exist_ok=True)
g=Gateway(directory,out/'actions.ndjson')
results={'mode':'genuine native client with authoritative isolated server','completed':False,'views':[],'routes':[]}

def save(): (out/'validation.json').write_text(json.dumps(results,indent=2),encoding='utf-8')

def observe(): return g.observe()

def walk(x,z,label):
    start=observe(); g.act(f'walk {x} {z}')
    trace=[]; end=time.monotonic()+65
    while time.monotonic()<end:
        s=observe()
        if not trace or s['position']!=trace[-1]['position']:
            trace.append({'tick':s['tick'],'position':s['position']})
        if s['ready'] and s['position'][1:]==[x,z]:
            results['routes'].append({'label':label,'from':start['position'],'to':[x,z],'arrived':True,'trace':trace});save()
            print('WALK_PASS',label,flush=True);return s
        time.sleep(.12)
    results['routes'].append({'label':label,'arrived':False,'trace':trace});save()
    raise TimeoutError(f'Failed normal route {label}: last {s["position"]}')

def capture(label,pitch,yaw):
    g.act(f'camera {pitch} {yaw}');time.sleep(.8)
    s=g.act('capture'); source=directory/f'frame-{s["sequence"]}.bmp'
    end=time.monotonic()+20
    while not source.exists():
        if time.monotonic()>end: raise TimeoutError('No genuine native frame')
        time.sleep(.1)
    target=out/f'{label}.bmp';shutil.copyfile(source,target)
    results['views'].append(dict(label=label,position=s['position'],pitch=pitch,yaw=yaw,tick=s['tick'],frame=target.name,sha256=hashlib.sha256(target.read_bytes()).hexdigest()))
    save(); print('CAPTURE',label,flush=True)

def inspect(label):
    g.act('inspect');time.sleep(.6)
    source=directory/'scene-objects.csv';shutil.copyfile(source,out/f'{label}-scene-objects.csv')
    rows=list(csv.DictReader(source.open()))
    arches=[r for r in rows if r['id']=='65000' and r['rejected']=='false']
    assert {(r['x'],r['z'],r['rotation']) for r in arches}=={('10422','104','0'),('10428','102','1')},arches
    footings=[r for r in rows if r['id']=='65001' and r['rejected']=='false']
    assert len(footings)==4,footings
    assert any(r['id']=='7016' and r['x']=='10420' and r['z']=='104' and r['rejected']=='false' for r in rows)
    results[label+'_scene']={'arches':arches,'footings':footings,'original_pillar_present':True};save()
    print('SCENE_PASS',label,flush=True)

if __name__=='__main__':
    deadline=time.monotonic()+600
    while not (directory/'state.json').exists():
        if time.monotonic()>deadline:raise TimeoutError('Native startup did not create gateway observation')
        time.sleep(.5)
    g.wait(lambda s:s.get('ready'),180)
    g.act('close')
    time.sleep(3)
    walk(10424,107,'approach main arch from north')
    capture('main-front',2048,0)
    walk(10424,102,'walk south through right central opening')
    capture('main-rear',2048,8192)
    walk(10423,102,'align left central opening')
    walk(10423,106,'walk north through left central opening')
    capture('main-three-quarter',2048,2048)
    walk(10422,102,'approach left stone column')
    start=observe();g.act('walk 10422 104')
    trace=[];until=time.monotonic()+6
    while time.monotonic()<until:
        s=observe();trace.append(s['position']);assert s['position'][1:]!=[10422,104],'Player entered solid column'
        time.sleep(.2)
    results['blocked_column']={'requested':[10422,104],'observed':trace,'never_entered':True};save()
    walk(10427,103,'approach rotated arch west side')
    walk(10429,103,'cross rotated central opening')
    capture('rotated-arch',2048,4096)
    walk(10424,105,'return between the two arches')
    capture('both-arches',2048,12288)
    inspect('initial')
    results['completed']=True;save()
    print('NATIVE_INITIAL_VALIDATION_PASS',flush=True)
