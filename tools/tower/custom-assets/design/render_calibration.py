"""Matched native comparison stages; reference frames are PRIVATE local output.

Only the explicitly original-only output subtree is suitable for publication.
"""
import argparse,json,os,subprocess,hashlib
from pathlib import Path
from PIL import Image,ImageDraw,ImageFont

def main():
 p=argparse.ArgumentParser();p.add_argument('renderer',type=Path);p.add_argument('cache',type=Path);p.add_argument('package',type=Path);p.add_argument('output',type=Path);a=p.parse_args()
 for k,v in vars(a).items():setattr(a,k,v.resolve())
 a.output.mkdir(parents=True,exist_ok=True);original=a.output/'original-only';private=a.output/'private-reference';original.mkdir(exist_ok=True);private.mkdir(exist_ok=True)
 assets={x['name']:x for x in json.loads((a.package/'catalogue.json').read_text())['assets']};report=[]
 try:font=ImageFont.truetype('C:/Windows/Fonts/segoeui.ttf',20)
 except OSError:font=ImageFont.load_default()
 refs={'arch':[(34,38,1868,10,0)],'wall':[(34,38,6132,0,1),(35,38,6132,0,1)],'wall_corner':[(34,38,6132,0,1),(35,38,6132,0,1),(34,38,6132,0,0),(34,39,6132,0,0)],'pressure_pump':[(34,38,9089,10,0)]}
 for name,placements in refs.items():
  asset=assets[name];w,l=asset['footprint']
  for local in [False,True]:
   dest=original if local else private
   lines=['S 32 32',f'L 40 38 0 {asset["object_id"]} 10 0']
   if not local:lines += [f'L {x} {z} 0 {id} {shape} {rotation}' for x,z,id,shape,rotation in placements]
   if 'scale_figure' in assets:lines.append(f'L 39 37 0 {assets["scale_figure"]["object_id"]} 10 0')
   scene=dest/f'{name}.scene';scene.write_text('\n'.join(lines)+'\n')
   for label,yaw in [('front',0),('quarter',2048),('reverse',10240)]:
    target=dest/f'{name}-{label}.bmp';env=dict(os.environ,TOWER_TARGET_HEIGHT='-160')
    for key in ['TOWER_ORIGINAL_ART_ONLY','TOWER_LEGACY_REFERENCE_ONLY','TOWER_JS5_READTHROUGH_ADDR','TOWER_ASSET_PACKAGE']:env.pop(key,None)
    if local:env['TOWER_ORIGINAL_ART_ONLY']='1'
    else:env.update(TOWER_LEGACY_REFERENCE_ONLY='1',TOWER_ASSET_PACKAGE=str(a.package))
    x=40+w/2 if local else 38;z=38+l/2;distance=850 if local else 1650
    proc=subprocess.run([str(a.renderer),str(a.package if local else a.cache),str(scene),str(target),str(x),str(z),str(yaw),str(distance),'reference','authored'],env=env,capture_output=True,text=True)
    target.with_suffix('.log').write_text(proc.stdout+proc.stderr)
    if proc.returncode:raise RuntimeError(f'{name} {local}: {proc.stderr}')
    image=Image.open(target).convert('RGB');canvas=Image.new('RGB',(image.width,image.height+64),'#15201f');canvas.paste(image,(0,64));draw=ImageDraw.Draw(canvas)
    draw.text((18,6),f'{name.replace("_"," ").title()} / {label}',font=font,fill='#e8e3d7');draw.text((18,34),'OFFLINE NATIVE RUST / '+('ORIGINAL TOWER ART ONLY' if local else 'LOCAL REFERENCE COMPARISON / DO NOT PUBLISH'),font=font,fill='#a2b8ac');canvas.save(target.with_suffix('.png'))
    report.append(dict(asset=name,view=label,original_only=local,raw_sha256=hashlib.sha256(target.read_bytes()).hexdigest(),camera=dict(x=x,z=z,yaw=yaw,distance=distance,target_height=-160,preset='reference')))
  print('CALIBRATION_RUST',name,flush=True)
 (a.output/'render-manifest.json').write_text(json.dumps(dict(renderer_sha256=hashlib.sha256(a.renderer.read_bytes()).hexdigest(),frames=report),indent=2));print('CALIBRATION_RENDER_PASS',len(report))
if __name__=='__main__':main()
