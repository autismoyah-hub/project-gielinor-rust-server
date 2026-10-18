import sys,json,subprocess
from pathlib import Path
from PIL import Image
HERE=Path(__file__).resolve().parent;REPO=HERE.parents[2];EXP=REPO.parent;ROOT=EXP/'static-production'
NATIVE=EXP/'astra-benchmark/tools/asset_workbench.exe';CACHE=EXP.parents[1]/'resource-staging/server/work/integration/a-kotlin/.data/cache/js5';BLENDER=Path(r'C:\Program Files\Blender Foundation\Blender 5.2\blender.exe');REFERENCE=EXP/'astra-benchmark/private/reference/object-2097'
def run(args,log):
    p=subprocess.run(list(map(str,args)),capture_output=True,text=True);log.parent.mkdir(parents=True,exist_ok=True);log.write_text(p.stdout+'\n'+p.stderr,encoding='utf8')
    if p.returncode:raise RuntimeError(str(log)+'\n'+p.stdout[-700:]+p.stderr[-700:])
def views(folder,kind):
    if kind=='anvil':
        run([NATIVE,CACHE,folder/'matched','render',folder/'asset.model',REFERENCE/'camera.txt',64,768],folder/'matched/log.txt')
        for v in ['front','side','quarter','rear','gameplay']:Image.open(folder/'matched'/(v+'.bmp')).save(folder/(v+'.png'))
    h,d={'anvil':(32,180),'workstation':(73,370),'machine':(90,400),'creature':(65,390)}[kind]
    scene=folder/'scene.tsv';scene.write_text('light\t64\t768\nowned\tasset.model\t0\t0\t0\t0\t-\t0\n')
    cams={'front':f'look 0 {-h-20} {-d} 0 {-h} 0 900 1000 800','side':f'look {d} {-h-20} 0 0 {-h} 0 900 1000 800','quarter':f'look {d*.64} {-h-d*.42} {-d*.78} 0 {-h} 0 900 1000 800','rear':f'look {-d*.64} {-h-d*.42} {d*.78} 0 {-h} 0 900 1000 800','gameplay':f'player 0 {-h} 0 2000 14500 {d}'}
    for name,c in cams.items():
        camera=folder/(name+'.camera');camera.write_text(c);dest=folder/'native'/name;run([NATIVE,CACHE,dest,'scene',scene,camera],dest/'log.txt');Image.open(dest/'scene.bmp').save(folder/(('close-'if kind=='anvil'else'')+name+'.png'))
if __name__=='__main__':
    version=sys.argv[1]
    for kind in (sys.argv[2:]or ['anvil','workstation','machine','creature']):
        out=ROOT/'runs'/(kind+'-'+version)
        if not(out/'source.blend').exists():run([BLENDER,'-b','--factory-startup','--python-exit-code','1','--python',HERE/'designs.py','--',kind,version,out],ROOT/'logs'/(kind+'-'+version+'.log'))
        views(out,kind);print('STATIC_NATIVE',kind,version,flush=True)
