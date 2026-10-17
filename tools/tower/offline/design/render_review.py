"""Render isolated configurations locally through the unchanged native client library."""
import argparse,json,subprocess,sys,hashlib
from pathlib import Path
from PIL import Image
p=argparse.ArgumentParser();p.add_argument('--renderer',type=Path,required=True);p.add_argument('--cache',type=Path,required=True);p.add_argument('--configs',type=Path,required=True);p.add_argument('--private-output',type=Path,required=True);a=p.parse_args();rows=[]
for letter in ['a','b','c']:
 c=json.loads((a.configs/f'candidate-{letter}/config.json').read_text());out=a.private_output/f'candidate-{letter}';out.mkdir(parents=True,exist_ok=True)
 subprocess.run([sys.executable,str(Path(__file__).with_name('generate.py')),'--config',str(a.configs/f'candidate-{letter}/config.json'),'--output',str(out)],check=True)
 jobs=[('overview',56,56,2048,9000,'overview'),('overview-reverse',56,56,10240,9000,'overview'),('plan',56,56,0,1,'plan')]
 jobs += [(name,*pose,1800,'detail')for name,pose in c['views'].items()]
 for name,x,z,yaw,distance,preset in jobs:
  f=out/f'{name}.bmp';r=subprocess.run([str(a.renderer),str(a.cache),str(out/'candidate.scene'),str(f),str(x),str(z),str(yaw),str(distance),preset],capture_output=True,text=True);(out/f'{name}.log').write_text(r.stdout+r.stderr);r.check_returncode();png=f.with_suffix('.png');Image.open(f).save(png,optimize=True);rows.append({'candidate':letter,'name':name,'frame':str(png.resolve()),'raw_bmp':str(f.resolve()),'raw_sha256':hashlib.sha256(f.read_bytes()).hexdigest(),'png_sha256':hashlib.sha256(png.read_bytes()).hexdigest(),'camera':[x,z,yaw,distance,preset],'metadata':str(Path(str(f)+'.camera.txt').resolve())});print(letter,name,'rendered OFFLINE',flush=True)
 (a.private_output/'render-manifest.json').write_text(json.dumps({'scope':'Authentic native-library OFFLINE raster; no actors, live animation, UI, gameplay or lifecycle acceptance. Private pending media-rights clearance.','renderer_sha256':hashlib.sha256(a.renderer.read_bytes()).hexdigest(),'views':rows},indent=2))
