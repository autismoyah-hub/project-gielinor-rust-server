"""Local study runner. No changes to renderer, cache or live game."""
import sys,subprocess,json,time
from pathlib import Path
from PIL import Image
HERE=Path(__file__).resolve().parent
REPO=HERE.parents[2];EXP=REPO.parent;ROOT=EXP/'mesh-production'
NATIVE=EXP/'astra-benchmark/tools/asset_workbench.exe'
CACHE=EXP.parents[1]/'resource-staging/server/work/integration/a-kotlin/.data/cache/js5'
BLENDER=Path(r'C:\Program Files\Blender Foundation\Blender 5.2\blender.exe')

def run(cmd,log):
    start=time.time();p=subprocess.run(list(map(str,cmd)),capture_output=True,text=True);log.parent.mkdir(parents=True,exist_ok=True);log.write_text(p.stdout+'\n'+p.stderr,encoding='utf8')
    if p.returncode:raise RuntimeError(str(log)+': '+p.stderr[-500:]+p.stdout[-500:])
    return time.time()-start

def views(folder,kind):
    height,dist={'architecture':(165,620),'machine':(85,370),'creature':(60,340)}[kind]
    scene=folder/'scene.tsv';scene.write_text('light\t64\t768\nowned\tasset.model\t0\t0\t0\t0\t-\t0\n')
    cams={
        'front':f'look 0 {-height-20} {-dist} 0 {-height} 0 900 1000 800',
        'side':f'look {dist} {-height-20} 0 0 {-height} 0 900 1000 800',
        'quarter':f'look {dist*.64} {-height-dist*.42} {-dist*.78} 0 {-height} 0 900 1000 800',
        'rear':f'look {-dist*.64} {-height-dist*.42} {dist*.78} 0 {-height} 0 900 1000 800',
        'gameplay':f'player 0 {-height} 0 2000 14500 {dist}'}
    for name,c in cams.items():
        cam=folder/(name+'.camera');cam.write_text(c);target=folder/'views'/name
        run([NATIVE,CACHE,target,'scene',scene,cam],target/'render.log')
        Image.open(target/'scene.bmp').save(folder/(name+'.png'))

if __name__=='__main__':
    for kind in ['architecture','machine','creature']:
        ver=sys.argv[1];folder=ROOT/'runs'/(kind+'-'+ver)
        if not (folder/'asset.model').exists():
            secs=run([BLENDER,'-b','--factory-startup','--python-exit-code','1','--python',HERE/'author.py','--',kind,ver,folder],ROOT/'logs'/(kind+'-'+ver+'.log'));print(kind,ver,'build seconds',round(secs,2),flush=True)
        views(folder,kind);print(kind,ver,'native views complete',flush=True)
