"""Original-only native close comparisons; not a gameplay-camera substitute."""
import argparse,json,os,subprocess,hashlib
from pathlib import Path
from PIL import Image,ImageDraw

def main():
 p=argparse.ArgumentParser();p.add_argument('renderer',type=Path);p.add_argument('before',type=Path);p.add_argument('after',type=Path);p.add_argument('output',type=Path);a=p.parse_args()
 for key,value in vars(a).items():setattr(a,key,value.resolve())
 a.output.mkdir(parents=True,exist_ok=True);env=dict(os.environ,TOWER_ORIGINAL_ART_ONLY='1',TOWER_TARGET_HEIGHT='-160')
 for key in ['TOWER_LEGACY_REFERENCE_ONLY','TOWER_ASSET_PACKAGE','TOWER_JS5_READTHROUGH_ADDR']:env.pop(key,None)
 report=[]
 for family in ['rack','rock']:
  for label,package in [('before',a.before),('after',a.after)]:
   name=family+'-'+label;scene=a.output/(name+'.scene')
   text='S 32 32\n'
   if family=='rack':text+=f'L 36 38 0 63005 10 0\nL 36 {37 if label=="before" else 38} 0 63045 10 0\n';point=[37,38.5]
   else:text+='L 36 38 0 63073 10 0\n';point=[37.5,39.5]
   scene.write_text(text);bmp=a.output/(name+'.bmp');proc=subprocess.run([str(a.renderer),str(package),str(scene),str(bmp),*map(str,point),'2048','850','reference','authored'],env=env,capture_output=True,text=True)
   bmp.with_suffix('.log').write_text(proc.stdout+proc.stderr)
   if proc.returncode:raise RuntimeError(proc.stderr)
   im=Image.open(bmp).convert('RGB');canvas=Image.new('RGB',(im.width,im.height+36),'#15201f');canvas.paste(im,(0,36));ImageDraw.Draw(canvas).text((12,10),f'{family.upper()} {label.upper()} / ORIGINAL ART / NATIVE RUST CLOSE INSPECTION',fill='white');canvas.save(bmp.with_suffix('.png'))
   report.append(dict(name=name,original_only=True,raw_sha256=hashlib.sha256(bmp.read_bytes()).hexdigest(),scene_sha256=hashlib.sha256(scene.read_bytes()).hexdigest(),registry_sha256=hashlib.sha256((package/'registry.txt').read_bytes()).hexdigest(),camera=Path(str(bmp)+'.camera.txt').read_text()))
 (a.output/'manifest.json').write_text(json.dumps(report,indent=2));print('ORIGINAL_ASSET_COMPARISONS_PASS',len(report))
if __name__=='__main__':main()
