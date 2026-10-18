import json,sys,html
from pathlib import Path
from PIL import Image,ImageDraw,ImageChops,ImageStat
import argparse;parser=argparse.ArgumentParser();parser.add_argument('--private',type=Path,required=True);parser.add_argument('--server',type=Path,required=True);args=parser.parse_args();Q=args.private;R=args.server;T=R/'tools/tower/asset-workbench';sys.path.insert(0,str(T));from analyze import foreground
from component_review import internal
load=lambda p:json.loads(p.read_text(encoding='utf-8'));out=Q/'review';out.mkdir(exist_ok=True);media=out/'static-development';media.mkdir(exist_ok=True);rows=[];parts=['<h1>Post-exam static partition and material development</h1><p>Private reference derivative. Not an unseen examination or original art. V2 motion evidence retains its own materials. V3 is a separate static study; weighted animation remains unsupported. Each comparison is authentic / V2 / V3.</p>']
for case in load(Q/'exam-cases.json')['cases']:
 for variant in case['variants']:
  phase=variant['phase'];ref=Q/'exam-first-attempt'/case['key']/'reference'/phase/('npc-'+str(variant['npc_id']));v2=Q/'exam-development-v2'/case['key']/phase/'candidate';v3=Q/'exam-static-development-v3'/case['key']/phase/'candidate';row=dict(case=case['key'],phase=phase,views={},v2_vertices=len(load(v2/'geometry.json')['vertices']),v3_vertices=len(load(v3/'geometry.json')['vertices']),geometry_unchanged=load(v2/'geometry.json')['vertices']==load(v3/'geometry.json')['vertices'],motion_claim=False,artistic_acceptance=False);parts.append('<h2>'+case['label']+' / '+phase+'</h2>')
  for view in ['front','side','rear','quarter','gameplay']:
   images=[Image.open(p).convert('RGB')for p in[ref/(view+'.bmp'),v2/'native'/(view+'.bmp'),v3/'native'/(view+'.bmp')]];canvas=Image.new('RGB',(1200,420),(16,21,26));d=ImageDraw.Draw(canvas);metrics=[]
   for i,im in enumerate(images):
    if i:
     union=ImageChops.lighter(foreground(images[0]),foreground(im));metrics.append(dict(foreground_union_rgb_error=sum(ImageStat.Stat(ImageChops.difference(images[0],im),mask=union).mean)/3,internal_edges=internal(images[0],im)))
    small=im.copy();small.thumbnail((400,390));canvas.paste(small,(400*i,28));d.text((400*i+8,8),['Authentic','V2 group correction','V3 static material/partition'][i],fill='white')
   row['views'][view]=dict(v2=metrics[0],v3=metrics[1]);name=case['key']+'-'+phase+'-'+view+'.png';canvas.save(media/name);parts.append('<figure><img loading="lazy" src="static-development/'+name+'"><figcaption>'+view+'</figcaption></figure>')
  row['mean_v2_foreground_rgb_error']=sum(v['v2']['foreground_union_rgb_error']for v in row['views'].values())/5;row['mean_v3_foreground_rgb_error']=sum(v['v3']['foreground_union_rgb_error']for v in row['views'].values())/5;row['color_error_improved']=row['mean_v3_foreground_rgb_error']<row['mean_v2_foreground_rgb_error'];rows.append(row)
(out/'static-development.json').write_text(json.dumps(rows,indent=2));(out/'static-development.html').write_text('<!doctype html><meta charset="utf-8"><title>Private static development</title><style>body{background:#10151a;color:#eee;font:17px system-ui;max-width:1400px;margin:auto;padding:30px}img{max-width:100%}figure{margin:20px 0}</style>'+''.join(parts),encoding='utf-8')
safe=[{k:r[k]for k in['case','phase','v2_vertices','v3_vertices','geometry_unchanged','mean_v2_foreground_rgb_error','mean_v3_foreground_rgb_error','color_error_improved','motion_claim','artistic_acceptance']}for r in rows];(R/'art/tower/advanced-mastery/static-development-summary.json').write_text(json.dumps(safe,indent=2));print('STATIC_DEVELOPMENT_REVIEW_COMPLETE',len(rows),'forms',sum(r['color_error_improved']for r in rows),'mean color improvements')
