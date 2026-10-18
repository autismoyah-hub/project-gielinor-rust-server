"""Matched static evidence runner. Config and protected inputs stay private.

Usage: python compare.py private-config.json render VERSION [ASSET...]
Config specifies frozen renderer, cache, per-case reference/history/roundtrip,
light, camera, and authored version directories. No fitting or auto alignment.
"""
import sys,json,subprocess
from pathlib import Path
from PIL import Image,ImageChops,ImageDraw

def run(args,log):
    p=subprocess.run(list(map(str,args)),capture_output=True,text=True);log=Path(log);log.parent.mkdir(parents=True,exist_ok=True);log.write_text(p.stdout+'\n'+p.stderr,encoding='utf8')
    if p.returncode:raise RuntimeError(str(log)+'\n'+p.stdout[-500:]+p.stderr[-600:])

def cameras(height,distance):
    h,d=height,distance
    return dict(front=f'look 0 {-h} {-d} 0 {-h} 0 1000 1000 900',side=f'look {d} {-h} 0 0 {-h} 0 1000 1000 900',rear=f'look 0 {-h} {d} 0 {-h} 0 1000 1000 900',quarter=f'look {d*.67} {-h-d*.3} {-d*.74} 0 {-h} 0 1000 1000 900',above=f'look {d*.35} {-h-d*.9} {-d*.55} 0 {-h} 0 1000 1000 900',gameplay=f'player 0 {round(-h)} 0 2000 14500 {round(d*2.4)}')

def render(config,case,model,out):
    out=Path(out);out.mkdir(parents=True,exist_ok=True)
    manifest=out/'scene.tsv';manifest.write_text(f"light\t{case['ambient']}\t{case['contrast']}\nowned\t{Path(model).resolve().as_posix()}\t0\t0\t0\t0\t-\t0\n")
    for name,c in cameras(case['height'],case['distance']).items():
        cam=out/(name+'.camera');cam.write_text(c);dest=out/name
        run([config['renderer'],config['cache'],dest,'scene',manifest,cam],dest/'log.txt');Image.open(dest/'scene.bmp').save(out/(name+'.png'))

def metrics(a,b):
    # Fixed image coordinates and background. No silhouette centering or scaling.
    def mask(im):return ImageChops.difference(im.convert('RGB'),Image.new('RGB',im.size,im.getpixel((0,0)))).convert('L').point(lambda x:255 if x else 0)
    ma,mb=mask(a),mask(b);inter=sum(x>0 for x in ImageChops.darker(ma,mb).getdata());union=sum(x>0 for x in ImageChops.lighter(ma,mb).getdata())
    return dict(silhouette_iou=inter/max(1,union),reference_pixels=sum(x>0 for x in ma.getdata()),candidate_pixels=sum(x>0 for x in mb.getdata()))

def evidence(root,version,kind):
    case=root/kind;new=root/'runs'/f'{kind}-{version}'/'native';dest=case/'comparisons'/version;dest.mkdir(parents=True,exist_ok=True);data={}
    for view in ['front','side','rear','quarter','above','gameplay']:
        a,b,c=[Image.open(p/(view+'.png')).convert('RGB')for p in [case/'authentic',case/'historical',new]]
        data[view]=dict(historical=metrics(a,b),new=metrics(a,c));Image.blend(a,c,.5).save(dest/(view+'-overlay.png'))
        # All panels receive exactly the same crop, derived from their union.
        bounds=[]
        for im in [a,b,c]:bounds.append(ImageChops.difference(im,Image.new('RGB',im.size,im.getpixel((0,0)))).getbbox())
        box=(max(0,min(q[0]for q in bounds)-15),max(0,min(q[1]for q in bounds)-15),min(a.width,max(q[2]for q in bounds)+15),min(a.height,max(q[3]for q in bounds)+15))
        panel=Image.new('RGB',((box[2]-box[0])*3,box[3]-box[1]+36),'#151a20');draw=ImageDraw.Draw(panel)
        for i,(im,label)in enumerate(zip([a,b,c],['Authentic','Historical','New independent'])):
            panel.paste(im.crop(box),(i*(box[2]-box[0]),36));draw.text((i*(box[2]-box[0])+8,10),label,fill='white')
        panel.save(dest/(view+'-comparison.png'))
    (dest/'metrics.json').write_text(json.dumps(data,indent=2));return data

if __name__=='__main__':
    c=json.loads(Path(sys.argv[1]).read_text());action=sys.argv[2];version=sys.argv[3];root=Path(c['root'])
    for name in sys.argv[4:]or c['cases']:
        case=c['cases'][name]
        if action=='reference':
            for field in ['authentic','historical','roundtrip']:render(c,case,case[field],root/name/field)
        elif action=='render':
            out=root/'runs'/f'{name}-{version}';render(c,case,out/'asset.model',out/'native');evidence(root,version,name)
        print('MATCHED_STATIC',name,action,version,flush=True)
