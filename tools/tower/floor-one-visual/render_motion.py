"""Every native tick of original creature actions inside the authored floor."""
import argparse,json,subprocess,hashlib
from pathlib import Path
from PIL import Image
def run(a):
    if a.out.exists():raise ValueError('Choose a new motion output')
    a.out.mkdir(parents=True);rows=[s.split('\t')for s in(a.floor/'scene.tsv').read_text().splitlines()];actor=next(i for i,r in enumerate(rows)if len(r)>1 and Path(r[1])==a.actor/'asset.model');recipe=json.loads((a.actor/'recipe.json').read_text());results=[]
    for action,spec in recipe['actions'].items():
        active=[r[:]for r in rows];active[actor][6]=str(a.actor/action);speed=0
        if action=='walk':
            gait=recipe['locomotion'];speed=-gait['stride']/(gait['stance_fraction']*spec['duration_ticks'])
        active[actor][7]=str(speed);manifest=a.out/(action+'.tsv');manifest.write_text('\n'.join('\t'.join(r)for r in active)+'\n');duration=sum(f['length']for f in json.loads((a.actor/action/'animation.json').read_text())['frames']);ticks=list(range(duration+1));frames=[];durations=[];last=None
        for tick in ticks:
            out=a.out/action/f'tick-{tick:04}';subprocess.run([str(a.native),str(a.cache),str(out),'scene',str(manifest),str(a.floor/'foundry-a.camera'),str(tick)],check=True);path=out/'scene.bmp';digest=hashlib.sha256(path.read_bytes()).hexdigest()
            if digest==last:durations[-1]+=20
            else:frames.append(Image.open(path).convert('RGB'));durations.append(20);last=digest
        frames[0].save(a.out/(action+'.gif'),save_all=True,append_images=frames[1:],duration=durations,loop=0)
        results.append(dict(action=action,ticks=ticks,native_samples=len(ticks),preview_states=len(frames),preview_duration_ms=sum(durations),world_travel_z_per_tick=speed,limitation='Native authored sequence; known contact defects retained. Preview combines identical image holds without changing time.'))
        (a.out/'results.json').write_text(json.dumps(dict(actions=results,authentic_geometry_used=False,gameplay_integration_tested=False),indent=2));print('FLOOR_ACTION_COMPLETE',action,flush=True)
if __name__=='__main__':
    p=argparse.ArgumentParser()
    for name in ['floor','actor','out','native','cache']:p.add_argument('--'+name,type=Path,required=True)
    a=p.parse_args();a.actor=a.actor.resolve();run(a)
