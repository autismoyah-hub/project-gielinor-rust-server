"""Genuine map scenes and original sample under explicitly matched native conditions.

Reference images/placements stay in the local reference directory. Only the
original-only custom output may be published. No live client is launched.
"""
import argparse,json,os,subprocess,hashlib,csv,collections
from pathlib import Path
from PIL import Image,ImageDraw,ImageFont

def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('renderer',type=Path);p.add_argument('cache',type=Path);p.add_argument('package',type=Path);p.add_argument('root',type=Path);p.add_argument('--candidate',type=Path,required=True);p.add_argument('--baseline',type=Path,required=True);p.add_argument('--baseline-package',type=Path);p.add_argument('--previous-candidate',type=Path);p.add_argument('--pitch',type=int,default=2048);a=p.parse_args()
 for key,value in vars(a).items():
  if isinstance(value,Path):setattr(a,key,value.resolve())
 a.baseline_package=a.baseline_package or a.package
 targets=json.loads((a.candidate/'candidate-manifest.json').read_text())['camera_targets']
 ref=a.root/'references';ref.mkdir(parents=True,exist_ok=True);custom=a.candidate/'views';custom.mkdir(exist_ok=True);records=[]
 for name,(x,z) in dict(industrial=(2896,10176),dungeon=(2880,9792),river=(3232,3240)).items():
  path=ref/(name+'.scene');expected=f'N {x} {z} 0'
  if path.exists() and path.read_text().strip()!=expected:raise ValueError(f'Existing reference source differs: {path}')
  if not path.exists():path.write_text(expected+'\n')
 try:font=ImageFont.truetype('C:/Windows/Fonts/segoeui.ttf',17)
 except OSError:font=ImageFont.load_default()
 def render(name,scene,point,yaw=0,original=True,preset='matched',distance=0,pitch=None,package=None):
  pitch=a.pitch if pitch is None else pitch;package=package or (a.baseline_package if name.startswith('baseline-') else a.package)
  destination=custom if original else ref;bmp=destination/f'{name}.bmp';env=dict(os.environ,TOWER_MATCHED_PITCH=str(pitch))
  for key in ['TOWER_ORIGINAL_ART_ONLY','TOWER_LEGACY_REFERENCE_ONLY','TOWER_ASSET_PACKAGE','TOWER_JS5_READTHROUGH_ADDR','TOWER_TARGET_HEIGHT']:env.pop(key,None)
  env['TOWER_ORIGINAL_ART_ONLY' if original else 'TOWER_LEGACY_REFERENCE_ONLY']='1'
  cmd=[str(a.renderer),str(package if original else a.cache),str(scene),str(bmp),str(point[0]),str(point[1]),str(yaw),str(distance),preset,'authored']
  proc=subprocess.run(cmd,env=env,capture_output=True,text=True);bmp.with_suffix('.log').write_text(proc.stdout+proc.stderr)
  if proc.returncode:raise RuntimeError(f'{name}: {proc.stderr}')
  im=Image.open(bmp).convert('RGB');canvas=Image.new('RGB',(im.width,im.height+57),'#15201f');canvas.paste(im,(0,57));draw=ImageDraw.Draw(canvas)
  draw.text((13,5),name.replace('-',' ').title(),font=font,fill='#e5e0d6');draw.text((13,30),'OFFLINE RUST / '+('ORIGINAL TOWER ART' if original else 'LOCAL AUTHENTIC CACHE SCENE / DO NOT PUBLISH'),font=font,fill='#a9c1b7');canvas.save(bmp.with_suffix('.png'))
  background=sum(pixel==(16,21,26) for pixel in im.getdata())/(im.width*im.height)
  records.append(dict(name=name,original_only=original,source_scene=str(scene),scene_sha256=sha(scene),package_registry_sha256=sha(package/'registry.txt') if original else None,renderer_sha256=sha(a.renderer),raw_sha256=sha(bmp),png_sha256=sha(bmp.with_suffix('.png')),camera_sidecar=bmp.name+'.camera.txt',camera_sidecar_sha256=sha(Path(str(bmp)+'.camera.txt')),native_camera=Path(str(bmp)+'.camera.txt').read_text(),point=point,yaw=yaw,pitch=pitch,orbit_override=distance,preset=preset,background_pixel_fraction=round(background,6),log=proc.stdout))
  print('FIDELITY_FRAME',name,flush=True)
 for name,point in [('industrial',[2871,10205]),('dungeon',[2854,9809]),('river',[3232,3240])]:
  for label,yaw in [('front',0),('quarter',2048),('reverse',10240)]:render(f'{name}-{label}',ref/f'{name}.scene',point,yaw,False)
 for label,yaw in [('front',0),('quarter',2048),('reverse',10240)]:
  render('baseline-'+label,a.baseline,[62,55],yaw)
  render('suite-'+label,a.candidate/'candidate.scene',targets['suite'],yaw)
 for name,point in [('sanctuary',targets['sanctuary']),('workshop',targets['workshop'])]:
  for label,yaw in [('front',0),('quarter',2048)]:render(name+'-'+label,a.candidate/'candidate.scene',point,yaw)
 for variant in ['uniform-material','flat-height']:render('control-'+variant,a.candidate/f'{variant}.scene',targets['suite'],2048)
 render('baseline-old-camera',a.baseline,[62,55],0,True,'player',1800)
 render('suite-pitch30',a.candidate/'candidate.scene',targets['suite'],2048,pitch=1365)
 if a.previous_candidate:
  previous=json.loads((a.previous_candidate/'candidate-manifest.json').read_text())['camera_targets']['suite']
  render('previous-suite-quarter',a.previous_candidate/'candidate.scene',previous,2048,package=a.baseline_package)
 # Scene-level source inventory is metadata only. Raw placements stay local.
 inventories=[]
 for name in ['industrial','dungeon','river']:
  raw=(ref/f'{name}-front.bmp.inventory.txt').read_text();meta=raw.split('object_id\t')[0]
  ids=[]
  for line in raw.split('object_id\t')[1].splitlines()[1:]:
   if not line:continue
   fields=line.split('\t');ids.append(dict(object_id=int(fields[0]),placements=int(fields[1]),name=fields[2],model_ids=json.loads(fields[3])))
  inventories.append(dict(scene=name,source_metadata=meta,objects=ids))
 manifest=dict(status='Scene-level calibration for human artistic review; no full-floor promotion',native_shared_configuration=dict(viewport=[765,503],fov=[256,205],zoom=[256,320],focal=308,pitch=a.pitch,orbit=600+3*(a.pitch//8),follow_height=50,brightness=.8,far_clip=5250,roof_max_plane=0,actors=False,ui=False,skybox=False),reference_inventory=inventories,frames=records)
 (a.candidate/'fidelity-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print('SCENE_FIDELITY_RENDER_PASS',len(records))
if __name__=='__main__':main()
