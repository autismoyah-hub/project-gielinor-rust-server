"""Compare explicit semantic proxies at matching native action timestamps.

Only decoded native poses/images are measured. Contact is a geometric proxy;
without root travel or collision state it is not a locomotion correctness proof.
"""
import argparse,json,math
from pathlib import Path
from PIL import Image,ImageDraw
from motion import landmarks,selectors,compare_motion

def load(p):return json.loads(p.read_text())
def poses(folder,times=None):
    return [(int(p.stem.split('-')[1]),load(p)['vertices'])for p in sorted(folder.glob('pose-*.json'))if times is None or int(p.stem.split('-')[1])in times]
def review(reference,candidates,bindings,out):
    out.mkdir(parents=True,exist_ok=True);summaries=[]
    for action,folder in reference['actions'].items():
        ref=Path(folder);g=load(Path(reference['geometry']));ap=poses(ref);times={t for t,_ in ap};ann_a={k:v['reference']for k,v in bindings.items()};la=landmarks(g,ap,ann_a);ticks=sorted(times);selected=sorted({ticks[round(i*(len(ticks)-1)/6)]for i in range(7)})
        sheet=Image.new('RGB',(280*len(selected),284*(len(candidates)+1)),(16,21,26));draw=ImageDraw.Draw(sheet);gif_frames=[]
        sources=[('A authentic',ref)]+[(name,Path(root)/'motion'/action)for name,root in candidates.items()]
        for row,(label,folder)in enumerate(sources):
            for col,tick in enumerate(selected):
                p=folder/f'pose-{tick:04}.bmp'
                if not p.exists():raise ValueError('Missing matched native image '+str(p))
                im=Image.open(p);im.thumbnail((280,260));sheet.paste(im,(col*280,row*284+24));draw.text((col*280+8,row*284+5),f'{label} | tick {tick}',fill='white')
        sheet.save(out/(action+'-contact-sheet.png'))
        for tick in ticks:
            frame=Image.new('RGB',(350*len(sources),350),(16,21,26));d=ImageDraw.Draw(frame)
            for col,(label,folder)in enumerate(sources):
                im=Image.open(folder/f'pose-{tick:04}.bmp');im.thumbnail((350,325));frame.paste(im,(350*col,25));d.text((350*col+8,5),f'{label} | tick {tick}',fill='white')
            gif_frames.append(frame)
        gif_frames[0].save(out/(action+'-synchronized.gif'),save_all=True,append_images=gif_frames[1:],duration=[max(20,(ticks[i+1]-t)*20)if i+1<len(ticks)else 500 for i,t in enumerate(ticks)],loop=0)
        for name,root in candidates.items():
            root=Path(root);cg=load(root/'geometry.json');bp=poses(root/'motion'/action,times)
            if {t for t,_ in bp}!=times:raise ValueError('Incomplete matched action coverage')
            lb=landmarks(cg,bp,{k:v['candidate']for k,v in bindings.items()});result=compare_motion(la,lb,out/(action+'-'+name+'-trajectories.json'));feet={}
            for foot in [k for k in bindings if 'foot'in k]:
                ix=selectors(cg,bindings[foot]['candidate']);ys=[max(v[i][1]for i in ix)for _,v in bp];feet[foot]=dict(lowest_surface_y_range=[min(ys),max(ys)],ground_proxy='native Y down; bind ground Y=0',max_penetration_native=max(0,max(ys)),max_lift_native=max(0,-min(ys)))
            summaries.append(dict(action=action,candidate=name,matched_native_timestamps=len(times),mean_proxy_arc_error_native=sum(v['mean_arc_error']for v in result.values())/len(result),regions={k:dict(mean_arc_error_native=v['mean_arc_error'],max_arc_error_native=v['max_arc_error'])for k,v in result.items()},foot_surface_proxies=feet))
    (out/'summary.json').write_text(json.dumps(dict(results=summaries,interpretation='Curated component centroids are proxies, not inferred joints. Equal timestamp and camera; no time warp. Arc errors remove initial bind offset only. Artistic acceptance remains false.'),indent=2))
    print('MATCHED_NATIVE_MOTION_REVIEW_COMPLETE',len(summaries))
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('manifest',type=Path);p.add_argument('bindings',type=Path);p.add_argument('out',type=Path);a=p.parse_args();m=load(a.manifest);review(m['reference'],m['candidates'],load(a.bindings),a.out)
