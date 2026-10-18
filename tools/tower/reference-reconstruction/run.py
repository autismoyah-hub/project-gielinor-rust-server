"""Preserved native standard views and frozen calibrated drawing views."""
from pathlib import Path
import sys,json,subprocess,time,datetime,hashlib
from PIL import Image,ImageChops,ImageDraw
T=Path(__file__).resolve().parent;E=T.parents[3];R=E/'ashhorn-reference-private'
BL='C:/Program Files/Blender Foundation/Blender 5.2/blender.exe';CACHE=E.parents[1]/'resource-staging/server/work/integration/a-kotlin/.data/cache/js5';NATIVE=E/'astra-benchmark/tools/asset_workbench.exe';REF=E/'build/concept-ab/release/reference_mesh_view.exe'
def execute(args,log):
    before=time.monotonic();p=subprocess.run(list(map(str,args)),capture_output=True,text=True);log=Path(log);log.parent.mkdir(parents=True,exist_ok=True);log.write_text(p.stdout+'\n'+p.stderr,encoding='utf8')
    if p.returncode:raise RuntimeError(str(log)+'\n'+p.stdout[-600:]+p.stderr[-1200:])
    return time.monotonic()-before
def views(model,out):
    out=Path(out);out.mkdir(parents=True,exist_ok=True);scene=out/'scene.tsv';scene.write_text('light\t72\t850\nowned\t'+Path(model).resolve().as_posix()+'\t0\t0\t0\t0\t-\t0\n')
    cams={'front':'look 0 -88 -415 0 -88 35 1000 1000 900','side':'look 450 -88 35 0 -88 35 1000 1000 900','rear':'look 0 -88 485 0 -88 35 1000 1000 900','quarter':'look 301.5 -223 -298 0 -88 35 1000 1000 900','above':'look 157.5 -493 -212.5 0 -88 35 1000 1000 900','gameplay':'player 0 -88 35 2000 14500 600'}
    for v,text in cams.items():
        camera=out/(v+'.camera');camera.write_text(text);dest=out/v;execute([NATIVE,CACHE,dest,'scene',scene,camera],dest/'log.txt');Image.open(dest/'scene.bmp').save(out/(v+'.png'))
    C=json.loads((R/'constraints.json').read_text())
    for v,c in C['cameras'].items():
        dest=out/'reference'/v;dest.mkdir(parents=True,exist_ok=True);cam=dest/'camera.txt';cam.write_text(c['native_camera']);execute([REF,CACHE,model,dest,cam],dest/'log.txt');Image.open(dest/'native.bmp').save(out/'reference'/(v+'.png'))
def contact(root):
    files=[root/'native/reference'/f'{v}.png'for v in ['front','left','rear','quarter']];canvas=Image.new('RGB',(1300,820),'#182329');draw=ImageDraw.Draw(canvas)
    for i,p in enumerate(files):
        im=Image.open(p).convert('RGB');im.thumbnail((625,375));x=(i%2)*650;y=(i//2)*410;canvas.paste(im,(x,y+30));draw.text((x+10,y+5),p.stem,fill='white')
    canvas.save(root/'reference-contact.png')
if __name__=='__main__':
    phase,label=sys.argv[1:];out=R/'candidates'/label
    if out.exists():raise SystemExit('Preserved candidate already exists')
    started=datetime.datetime.now(datetime.timezone.utc).isoformat();seconds=execute([BL,'-b','--factory-startup','--python-exit-code','1','--python',T/'build.py','--',R/'constraints.json',out,phase],R/'logs'/(label+'-blender.log'))
    print('BUILT',label,round(seconds,2),flush=True);before=time.monotonic();views(out/'asset.model',out/'native');contact(out);g=json.loads((out/'geometry.json').read_text())
    record={'phase':int(phase),'label':label,'started_utc':started,'finished_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'blender_seconds':seconds,'native_seconds':time.monotonic()-before,'vertices':len(g['vertices']),'triangles':len(g['faces']),'native_model_sha256':hashlib.sha256((out/'asset.model').read_bytes()).hexdigest(),'constraint_sha256':hashlib.sha256((R/'constraints.json').read_bytes()).hexdigest()}
    (out/'iteration.json').write_text(json.dumps(record,indent=2)+'\n');print('RENDERED',label,record['triangles'],'triangles',flush=True)
