"""Capture genuine native views at every major space; yaw uses 16384 units/turn."""
import argparse, pathlib, json, time
from tower_gateway import Gateway
p=argparse.ArgumentParser()
p.add_argument('--directory',type=pathlib.Path,required=True)
p.add_argument('--output',type=pathlib.Path,required=True)
p.add_argument('--source-label',required=True)
a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
g=Gateway(a.directory,a.output/'visual-walkthrough-actions.ndjson')
g.wait(lambda s:s.get('ready'),120);g.act('close')
points=[('arrival',10430,124),('sanctuary',10424,120),('plaque and reward',10424,116),('workshop',10406,118),('moss supply court',10408,104),('southern processional hall',10424,104),('poultry supply court',10440,104),('waterworks',10439,116),('grove',10408,136),('foundry vestibule',10424,130),('Ash Warden chamber',10423,136),('Ash Imp vent gallery',10439,136)]
rows=[]
for name,x,z in points:
    end=time.monotonic()+70;retry=0
    while True:
        s=g.observe()
        if s['ready'] and s['position'][1:]==[x,z]:break
        if time.monotonic()>end:raise TimeoutError(f'Native walkthrough route failed: {name}')
        if time.monotonic()>=retry:g.act(f'walk {x} {z}');retry=time.monotonic()+4
        time.sleep(.2)
    tick=s['tick'];g.wait(lambda s:s['tick']>tick+2,8)
    for yaw in [0,4096,8192,12288]:
        g.act(f'camera 2048 {yaw}');time.sleep(.35)
        s=g.act('capture');frame=a.directory/f"frame-{s['sequence']}.bmp"
        end=time.monotonic()+10
        while not frame.exists():
            if time.monotonic()>end:raise TimeoutError('No genuine native capture; headless observations are insufficient')
            time.sleep(.1)
        rows.append({'area':name,'position':s['position'],'pitch':2048,'yaw':yaw,'frame':str(frame.resolve()),'sequence':s['sequence']})
    (a.output/'visual-walkthrough-summary.json').write_text(json.dumps({'source':a.source_label,'mode':'native','completed':False,'views':rows},indent=2))
    print(name,flush=True)
g.act('walk 10424 120');g.wait(lambda s:s['position'][1:]==[10424,120],60);g.act('inspect')
(a.output/'visual-walkthrough-summary.json').write_text(json.dumps({'source':a.source_label,'mode':'native','completed':True,'views':rows},indent=2))
