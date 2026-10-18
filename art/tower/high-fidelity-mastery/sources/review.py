"""Standard native views for sustained refinement; unchanged frozen renderer."""
import sys,subprocess,json
from pathlib import Path
from PIL import Image
HERE=Path(__file__).resolve().parent;REPO=HERE.parents[2];EXP=REPO.parent;ROOT=EXP/'high-fidelity'
NATIVE=EXP/'astra-benchmark/tools/asset_workbench.exe';CACHE=EXP.parents[1]/'resource-staging/server/work/integration/a-kotlin/.data/cache/js5';BLENDER=Path(r'C:\Program Files\Blender Foundation\Blender 5.2\blender.exe')
def run(cmd,log):
    p=subprocess.run(list(map(str,cmd)),capture_output=True,text=True);log.parent.mkdir(parents=True,exist_ok=True);log.write_text(p.stdout+'\n'+p.stderr,encoding='utf8')
    if p.returncode:raise RuntimeError(str(log)+'\n'+p.stdout[-800:]+p.stderr[-800:])
def views(folder,kind):
    h,d={'architecture':(165,620),'machine':(85,370),'creature':(60,340)}[kind]
    scene=folder/'scene.tsv';scene.write_text('light\t64\t768\nowned\tasset.model\t0\t0\t0\t0\t-\t0\n')
    cams={'front':f'look 0 {-h-20} {-d} 0 {-h} 0 900 1000 800','side':f'look {d} {-h-20} 0 0 {-h} 0 900 1000 800','quarter':f'look {d*.64} {-h-d*.42} {-d*.78} 0 {-h} 0 900 1000 800','rear':f'look {-d*.64} {-h-d*.42} {d*.78} 0 {-h} 0 900 1000 800','gameplay':f'player 0 {-h} 0 2000 14500 {d}','head-close':f'look 170 -140 -280 0 -77 -84 1000 1100 800'}
    for name,cam in cams.items():
        if name=='head-close'and kind!='creature':continue
        camera=folder/(name+'.camera');camera.write_text(cam);target=folder/'native'/name;run([NATIVE,CACHE,target,'scene',scene,camera],target/'render.log');Image.open(target/'scene.bmp').save(folder/(name+'.png'))
if __name__=='__main__':
    kind,version=sys.argv[1:3];folder=ROOT/'runs'/(kind+'-'+version)
    if not(folder/'source.blend').exists():run([BLENDER,'-b','--factory-startup','--python-exit-code','1','--python',HERE/'refine.py','--',kind,version,folder],ROOT/'logs'/(kind+'-'+version+'.log'))
    views(folder,kind);print('NATIVE_REVIEW',kind,version)
