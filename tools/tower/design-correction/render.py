"""Actual existing Enhanced GPU backend; original-only geometry and terrain."""
from pathlib import Path
import os,json,struct,zlib,shutil,subprocess,time
from PIL import Image,ImageDraw,ImageFont
T=Path(__file__).resolve().parent;S=T.parents[2];E=S.parent;P=S/'art/tower/cinderworks-design-correction';R=E/'cinderworks-design-correction-private'
VIEWS={
 'arrival-workshop':{'transition':(49,55,4096,1700,'player'),'mill-aisle':(40,55,15000,1250,'player'),'landing-return':(56,56,12288,1200,'player'),'overview':(48,55,15300,3400,'floor-iso')},
 'pressure-walk':{'pressure-threshold':(68,65,13500,1500,'player'),'quay-and-intake':(70,62,14500,1450,'player'),'retort-sidewall':(73,72,2500,1700,'player'),'overview':(71,68,15300,3300,'floor-iso')},
 'warden-receiver':{'arena-entry':(56,72,0,2050,'player'),'receiver-crown':(56,77,0,1250,'player'),'reverse-exit':(56,71,8192,1500,'player'),'overview':(56,73,15300,3300,'floor-iso')}}
def main():
 package=R/'study-package';package.mkdir(parents=True,exist_ok=True);old=S/'art/tower/cinderworks-environment/overlay'
 registry=(old/'registry.txt').read_text().splitlines()
 for p in old.iterdir():
  if p.suffix in ['.model','.object']:shutil.copy2(p,package/p.name)
 for i,name in enumerate(VIEWS):
  source=P/'studies'/name;mid=1231000+i;oid=64800+i;blob=(source/'architecture.model').read_bytes();model_name=f'model-{mid}.model';(package/model_name).write_bytes(blob)
  # No interaction or collision on the compound review shell. It is prohibited
  # by the existing live environment namespace guard and cannot replace the map.
  definition=bytes([7,1])+struct.pack('>I',mid)+bytes([2])+('Cinderworks '+name+' DESIGN STUDY').encode()+bytes([0,14,1,15,1,17,19,0,22,0]);object_name=f'object-{oid}.object';(package/object_name).write_bytes(definition)
  for kind,identity,filename in [('model',mid,model_name),('object',oid,object_name)]:registry.append(f'{kind} {identity} {filename} {zlib.crc32((package/filename).read_bytes()):08x}')
 (package/'registry.txt').write_text('\n'.join(registry)+'\n')
 output=P/'renders';output.mkdir(exist_ok=True);reports=json.loads((P/'render-report.json').read_text())if(P/'render-report.json').exists()else[]
 for name,views in VIEWS.items():
  for view,camera in views.items():
   for mode in (['gpu','classic']if view==next(iter(views))else['gpu']):
    x,z,yaw,distance,preset=camera;stem=name+'--'+view+'--'+mode;dest=output/(stem+'.bmp')
    if dest.with_suffix('.png').exists():continue
    env=os.environ.copy();env.update(TOWER_ASSET_PACKAGE=str(package),TOWER_ORIGINAL_ART_ONLY='1',TOWER_RENDERER=mode,TOWER_GPU_ADAPTER='1',TOWER_REVIEW_FRAMES='20')
    before=time.monotonic();run=subprocess.run([str(E/'build/enhanced/release/cinderworks-review.exe'),str(package),str(P/'studies'/name/'study.scene'),str(dest),str(x),str(z),str(yaw),str(distance),preset,'authored'],capture_output=True,text=True,env=env);(R/(stem+'.log')).write_text(run.stdout+run.stderr);run.check_returncode()
    meta=json.loads(Path(str(dest)+'.renderer.json').read_text());assert meta['mode']==mode
    if mode=='gpu':assert meta['enhanced']and'NVIDIA'in meta['stats']['adapter']
    Image.open(dest).save(dest.with_suffix('.png'));shutil.copy2(Path(str(dest)+'.camera.txt'),output/(stem+'.camera.txt'));shutil.copy2(Path(str(dest)+'.renderer.json'),output/(stem+'.renderer.json'))
    reports=[v for v in reports if(v['study'],v['view'],v['mode'])!=(name,view,mode)]
    reports.append({'study':name,'view':view,'mode':mode,'camera':camera,'native_renderer':meta,'offline_total_seconds':time.monotonic()-before,'scope':'Original-only static native scene composition; not live client navigation, dynamic state or full-client FPS'});print('DESIGN_RENDER',stem,flush=True)
 if reports:
  (P/'render-report.json').write_text(json.dumps(reports,indent=2)+'\n')
 sheet=Image.new('RGB',(1800,3*460),'#172127');d=ImageDraw.Draw(sheet);font=ImageFont.truetype('C:/Windows/Fonts/arial.ttf',20)
 for row,(name,views)in enumerate(VIEWS.items()):
  for col,view in enumerate(list(views)[:3]):
   im=Image.open(output/(name+'--'+view+'--gpu.png')).convert('RGB');im.thumbnail((595,420));x=col*600;y=row*460;sheet.paste(im,(x+(600-im.width)//2,y+35));d.text((x+8,y+5),view,font=font,fill='#e1bf87')
 sheet.save(P/'representative-contact.png')
 print('REPRESENTATIVE_NATIVE_RENDERING_COMPLETE',flush=True)
if __name__=='__main__':main()
