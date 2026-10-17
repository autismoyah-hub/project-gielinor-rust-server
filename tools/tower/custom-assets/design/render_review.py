"""Reproducible original-only Rust review frames and catalogue thumbnails.

PNG headers identify provenance; the underlying unedited Rust BMP and camera
sidecar are retained. No cache assets, Blender previews or AI images are used.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess
import time
from PIL import Image,ImageDraw,ImageFont

def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    p=argparse.ArgumentParser();p.add_argument('renderer',type=Path);p.add_argument('package',type=Path);p.add_argument('scene',type=Path);p.add_argument('output',type=Path);p.add_argument('--catalogue',action='store_true');p.add_argument('--design',type=Path);a=p.parse_args()
    a.output.mkdir(parents=True,exist_ok=True);a.renderer=a.renderer.resolve();a.package=a.package.resolve();a.scene=a.scene.resolve()
    env=dict(os.environ,TOWER_ORIGINAL_ART_ONLY='1');env.pop('TOWER_JS5_READTHROUGH_ADDR',None);env.pop('TOWER_ASSET_PACKAGE',None)
    env.pop('TOWER_LEGACY_REFERENCE_ONLY',None);env.pop('TOWER_TARGET_HEIGHT',None)
    catalogue=json.loads((a.package/'catalogue.json').read_text())
    results=dict(origin='OFFLINE Rust native library; entirely original geometry; no original cache source',renderer_sha256=sha(a.renderer),scene_sha256=sha(a.scene),registry_sha256=sha(a.package/'registry.txt'),frames=[])
    try:font=ImageFont.truetype('C:/Windows/Fonts/segoeui.ttf',21)
    except OSError:font=ImageFont.load_default()

    def render(name,title,x,z,yaw,distance,preset,scene=None,folder=None):
        destination=a.output/(folder or 'views');destination.mkdir(exist_ok=True)
        target=destination/(name+'.bmp')
        args=[str(a.renderer),str(a.package),str(scene or a.scene),str(target.resolve()),str(x),str(z),str(yaw),str(distance),preset,'authored']
        started=time.monotonic();process=subprocess.run(args,env=env,capture_output=True,text=True)
        target.with_suffix('.log').write_text(process.stdout+process.stderr,encoding='utf-8')
        if process.returncode:raise RuntimeError(f'{name}: {process.stderr}')
        raw=Image.open(target).convert('RGB');image=Image.new('RGB',(raw.width,raw.height+64),'#111a1d');image.paste(raw,(0,64));draw=ImageDraw.Draw(image)
        draw.text((20,7),title,fill='#e5e8df',font=font);draw.text((20,34),'OFFLINE RUST  /  ORIGINAL ART  /  VISUAL PROPOSAL',fill='#8fb9b2',font=font)
        png=target.with_suffix('.png');image.save(png,optimize=True)
        results['frames'].append(dict(name=name,title=title,file=str(png.relative_to(a.output)),raw=str(target.relative_to(a.output)),camera=dict(x=x,z=z,yaw=yaw,distance=distance,preset=preset),seconds=round(time.monotonic()-started,3),raw_sha256=sha(target),png_sha256=sha(png),scene_sha256=sha(scene or a.scene)))
        (a.output/'render-manifest.json').write_text(json.dumps(results,indent=2)+'\n',encoding='utf-8')
        print('RUST_FRAME',name,flush=True)

    render('whole-isometric','The Fallen Cinderworks — complete floor',67,81,2048,11500,'floor-iso')
    render('whole-reverse','The Fallen Cinderworks — reverse isometric',67,81,10240,11500,'floor-iso')
    render('whole-plan','The Fallen Cinderworks — architectural plan',67,81,0,64,'floor-plan')
    areas=[('arrival','The Broken Processional',67,38,2100,67,38,0,1800),('sanctuary','The Fourfold Refuge',66,57,2400,66,57,2048,1800),
           ('workshop','The Brass Service Hall',41,67,2600,34,70,0,1900),('quarry','The Fractured Haulage Court',36,86,2400,33,88,0,1900),
           ('waterworks','The Headgate and Wheel',91,66,2800,92,65,4096,1900),('garden','The Breached Root Cistern',101,88,2500,100,88,1024,2100),
           ('vents','The Cinder Lung',82,92,2600,82,92,2048,1900),('narthex','The Last Cooling Gallery',63,90,2100,63,90,0,1900),
           ('foundry','The Bound Crucible',62,110,3300,62,115,0,3000),('ascent','The Upper Bearing',64,127,2200,63,128,10240,1900)]
    if a.design:
        design=json.loads(a.design.read_text());areas=[]
        for area in design['areas']:
            key=area['key'];x,z=area['center'];px,pz=x,z;yaw=0;distance=1800;overview_distance=2200
            if key=='waterworks':yaw=4096;distance=2100
            if key=='garden':yaw=2048;distance=2000
            if key=='foundry':pz=112;distance=2700;overview_distance=3100
            if key=='ascent':yaw=10240;distance=1900
            if key=='quarry':yaw=2048
            areas.append((key,area['title'],x,z,overview_distance,px,pz,yaw,distance))
    for key,title,x,z,d,px,pz,yaw,pd in areas:
        render(key+'-overview',title+' — room composition',x,z,2048,d,'room')
        render(key+'-reverse',title+' — reverse inspection',x,z,10240,d,'room')
        render(key+'-player',title+' — close player-oriented view',px,pz,yaw,pd,'player')
    if a.catalogue:
        stage=a.output/'asset-scenes';stage.mkdir(exist_ok=True)
        floor=next(asset for asset in catalogue['assets'] if asset['name']=='paving')['object_id']
        for asset in catalogue['assets']:
            w,l=asset['footprint'];x,z=30,30
            # Sector objects preserve local relative masonry geometry; their
            # thumbnails deliberately show the whole authored cutaway sector.
            scene=stage/(asset['name']+'.scene')
            rows=['S 96 112',f'L {x} {z} 0 {asset["object_id"]} 10 0']
            if not asset['name'].startswith('curtain_'):
                for fx in range(x-2,x+w+2):
                    for fz in range(z-2,z+l+2):rows.append(f'L {fx} {fz} 0 {floor} 10 0')
            scene.write_text('\n'.join(rows)+'\n',encoding='utf-8')
            height=asset['bounds'][2][1]-asset['bounds'][2][0]
            distance=max(1400,max(w,l,height)*280)
            render(asset['name'],asset['name'].replace('_',' ').title(),x+w/2,z+l/2,2048,round(distance),'detail',scene=scene,folder='assets')
    print('ORIGINAL_REVIEW_RENDERED',len(results['frames']),'genuine Rust frames',flush=True)

if __name__=='__main__':main()
