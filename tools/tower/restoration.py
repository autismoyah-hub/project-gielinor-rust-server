"""Assert real ClientState scene changes; captures require the native raster."""
import argparse,csv,json,pathlib,time
from tower_gateway import Gateway
p=argparse.ArgumentParser()
p.add_argument('--directory',type=pathlib.Path,required=True)
p.add_argument('--output',type=pathlib.Path,required=True)
p.add_argument('--label',required=True)
p.add_argument('--expected',required=True,help='Five expected IDs: forge,wheel,altar,hearth,foundry')
p.add_argument('--native-frames',action='store_true')
a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
expected=list(map(int,a.expected.split(',')));assert len(expected)==5
g=Gateway(a.directory,a.output/f'{a.label}-restoration-actions.ndjson')
s=g.wait(lambda s:s['ready'] and s['hp']>0,120)
positions=[(10411,114),(10445,122),(10427,113),(10423,122),(10423,142)]
end=time.monotonic()+40
while True:
    g.act('inspect');time.sleep(.2)
    try:
        rows=list(csv.DictReader((a.directory/'scene-objects.csv').open(encoding='utf8',newline='')))
        actual=[]
        for x,z in positions:
            matches=[r for r in rows if int(r['plane'])==0 and int(r['x'])==x and int(r['z'])==z and int(r['shape'])==10 and r['rejected']=='false']
            actual.append(int(matches[0]['id']) if len(matches)==1 else None)
        if actual==expected:break
    except (OSError,ValueError,KeyError):
        actual=[]
    if time.monotonic()>end:raise AssertionError(f'Restored scene mismatch: expected {expected}, actual {actual}')
    time.sleep(.3)
frames=[]
if a.native_frames:
    g.interface(35913794);g.interface(41222162)
    g.wait(lambda s:s['ready'] and 'Cinderworks' in s['journal'],30)
    shot=g.act('capture');frame=a.directory/f"frame-{shot['sequence']}.bmp"
    end=time.monotonic()+10
    while not frame.exists():
        if time.monotonic()>end:raise TimeoutError('No genuine native framebuffer')
        time.sleep(.1)
    frames.append(str(frame.resolve()))
s=g.observe()
(a.output/f'{a.label}-restoration-summary.json').write_text(json.dumps({'accepted':True,'mode':'native' if a.native_frames else 'headless','expected':expected,'actual':actual,'landmark_positions':positions,'frame_paths':frames,'state':s},indent=2),encoding='utf8')
print('Actual scene landmarks confirmed:',a.label,actual,flush=True)
