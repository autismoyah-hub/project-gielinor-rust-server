"""LOCAL ONLY. Native cache study frames contain protected reference art.

Never publish output directories. Inputs and output resolve absolutely.
"""
import argparse,json,os,subprocess,hashlib
from pathlib import Path
from PIL import Image,ImageDraw

def main():
 p=argparse.ArgumentParser();p.add_argument('renderer',type=Path);p.add_argument('cache',type=Path);p.add_argument('references',type=Path);a=p.parse_args()
 a.renderer=a.renderer.resolve();a.cache=a.cache.resolve();a.references=a.references.resolve();out=a.references/'rust';out.mkdir(exist_ok=True)
 env=dict(os.environ,TOWER_LEGACY_REFERENCE_ONLY='1')
 for key in ['TOWER_ORIGINAL_ART_ONLY','TOWER_JS5_READTHROUGH_ADDR','TOWER_ASSET_PACKAGE']:env.pop(key,None)
 report=[]
 for path in sorted(a.references.glob('object-*.json')):
  d=json.loads(path.read_text());id=d['object_id'];scene=out/f'{id}.scene';scene.write_text(f'S 32 32\nL 38 38 0 {id} {d["shape"]} 0\n')
  w,l=d['definition_tiles'];h=max(-v[1] for v in d['geometry']['vertices'])
  distance=max(650,h*1.8,w*100,l*100)
  for label,yaw in [('front',0),('quarter',2048),('reverse',10240)]:
   bmp=out/f'{id}-{label}.bmp';cmd=[str(a.renderer),str(a.cache),str(scene),str(bmp),str(38+w/2),str(38+l/2),str(yaw),str(distance),'reference','authored']
   proc=subprocess.run(cmd,env=env,capture_output=True,text=True);bmp.with_suffix('.log').write_text(proc.stdout+proc.stderr)
   if proc.returncode:raise RuntimeError(f'{id}: {proc.stderr}')
   img=Image.open(bmp).convert('RGB');canvas=Image.new('RGB',(img.width,img.height+36),'#162020');canvas.paste(img,(0,36));ImageDraw.Draw(canvas).text((8,8),f'LOCAL OSRS 239.1 | object {id} | {d["name"]} | {label}',fill='white');canvas.save(bmp.with_suffix('.png'))
   report.append(dict(object_id=id,view=label,sha256=hashlib.sha256(bmp.read_bytes()).hexdigest()))
  print('NATIVE_REFERENCE',id,flush=True)
 (out/'render-report.json').write_text(json.dumps(report,indent=2))
 files=sorted(out.glob('*-quarter.png'))
 sheet=Image.new('RGB',(5*383,((len(files)+4)//5)*280),'#162020')
 for i,path in enumerate(files):
  im=Image.open(path);im.thumbnail((383,270));sheet.paste(im,((i%5)*383,(i//5)*280))
 sheet.save(out/'contact-sheet.png');print('NATIVE_REFERENCE_RENDER_PASS',len(report))
if __name__=='__main__':main()
