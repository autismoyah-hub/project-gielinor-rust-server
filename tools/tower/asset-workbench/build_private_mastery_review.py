import json,sys,hashlib,html,os
from pathlib import Path
from PIL import Image,ImageDraw,ImageChops,ImageStat
import argparse;parser=argparse.ArgumentParser();parser.add_argument('--private',type=Path,required=True);args=parser.parse_args();Q=args.private;T=Path(__file__).resolve().parent;sys.path.insert(0,str(T));from analyze import foreground
from component_review import internal
load=lambda p:json.loads(p.read_text(encoding='utf-8'))
if Q.resolve().is_relative_to(T.resolve().parents[2])or not(Q/'exam-freeze.json').is_file():raise ValueError('Use the private sealed evidence root outside source and cache')
out=Q/'review';out.mkdir(exist_ok=True);media=out/'private-media';media.mkdir(exist_ok=True);parts=['<h1>ASCENT · private reconstruction review</h1><p>Protected local evidence. Do not publish this gallery or its media. Frozen failures remain unchanged. Later results are development retests, never unseen successes.</p>'];audit=[]
def url(p):return os.path.relpath(p,out).replace('\\','/')
for case in load(Q/'exam-cases.json')['cases']:
 parts.append('<h2>'+html.escape(case['label'])+'</h2>')
 for variant in case['variants']:
  phase=variant['phase'];old=Q/'exam-first-attempt'/case['key'];new=Q/'exam-development-v2'/case['key']/phase;ref=old/'reference'/phase/('npc-'+str(variant['npc_id']));candidate=new/'candidate';row=dict(case=case['key'],phase=phase,surface={},actions=[]);parts.append('<h3>'+html.escape(phase)+' · NPC '+str(variant['npc_id'])+'</h3><p>Coarse data-assisted geometry; facial and material construction remain under review. First-attempt classic animation failed because exported groups were dropped. The separate retest corrects group ownership without changing geometry.</p><div class="grid">')
  for view in ['front','side','rear','quarter','gameplay']:
   a=Image.open(ref/(view+'.bmp')).convert('RGB');b=Image.open(candidate/'native'/(view+'.bmp')).convert('RGB');mask=ImageChops.lighter(foreground(a),foreground(b));row['surface'][view]=dict(foreground_union_rgb_error=sum(ImageStat.Stat(ImageChops.difference(a,b),mask=mask).mean)/3,internal_edges=internal(a,b));canvas=Image.new('RGB',(1000,510),(16,21,26));d=ImageDraw.Draw(canvas)
   for col,(im,label)in enumerate([(a,'Authentic'),(b,'Coarse reconstruction / development retest')]):im.thumbnail((500,475));canvas.paste(im,(col*500,30));d.text((col*500+8,8),label,fill='white')
   p=media/(case['key']+'-'+phase+'-'+view+'.png');canvas.save(p);parts.append('<figure><a href="'+url(p)+'"><img loading="lazy" src="'+url(p)+'"></a><figcaption>'+view+'</figcaption></figure>')
  parts.append('</div>');done={};meta=load(ref/'metadata.json')
  for role,seq in {**meta['sequences'],**variant.get('additional_sequences',{})}.items():
   if seq in done:parts.append('<p>'+role+' uses the same source sequence as '+done[seq]+'.</p>');continue
   source=old/'reference-motion'/phase/role/('npc-'+str(variant['npc_id']))/('selected-'+str(seq));target=new/'motion'/role;cached=load(source/'animation.json').get('cached_animation')is not None;frames=[];durations=[];last=None;sf=None;cf=None;sm=cm=0
   for path in sorted(source.glob('pose-*.json')):
    tick=int(path.stem.split('-')[1]);sv=load(path)['vertices'];sf=sf or sv;sm+=sv!=sf
    if cached:cp=candidate/'native/gameplay.bmp'
    else:
     cp=target/(path.stem+'.bmp');cv=load(target/path.name)['vertices'];cf=cf or cv;cm+=cv!=cf
    ap=source/(path.stem+'.bmp');digest=hashlib.sha256(ap.read_bytes()+cp.read_bytes()).hexdigest()
    if digest==last:durations[-1]+=20;continue
    canvas=Image.new('RGB',(768,280),(16,21,26));d=ImageDraw.Draw(canvas)
    for col,(p,label)in enumerate([(ap,'Authentic native motion'),(cp,'STATIC ONLY / weighted retarget unsupported'if cached else'New topology / source classic controls')]):im=Image.open(p).convert('RGB');im.thumbnail((384,252));canvas.paste(im,(col*384,25));d.text((col*384+5,6),label,fill='white')
    frames.append(canvas.quantize(colors=128));durations.append(20);last=digest
   output=media/(case['key']+'-'+phase+'-'+role+'.gif');frames[0].save(output,save_all=True,append_images=frames[1:],duration=durations,loop=0);status='WEIGHTED_RETARGET_UNSUPPORTED'if cached else'FAIL_SOURCE_MOVES_CANDIDATE_STATIC'if sm and not cm else'COMPLETE_CLASSIC_NATIVE_MOTION';row['actions'].append(dict(role=role,sequence=seq,samples=sum(durations)//20,reference_moving_samples=sm,candidate_moving_samples=cm,status=status));parts.append('<details><summary>'+html.escape(role)+' · '+status+'</summary><img loading="lazy" src="'+url(output)+'"><p>All native ticks retained. Identical image holds compressed using their original durations; no timing changes. Source controls reused privately; no independent choreography claim.</p></details>');done[seq]=role
  audit.append(row);(out/'retest-review-diagnostics.json').write_text(json.dumps(audit,indent=2));print('PRIVATE_REVIEW_FORM_COMPLETE',case['key'],phase,flush=True)
parts.append('<p>Original art and full floor: <a href="../../rust-server/art/tower/advanced-mastery/index.html">source-safe gallery</a>.</p>');(out/'index.html').write_text('<!doctype html><meta charset="utf-8"><title>ASCENT private reconstruction review</title><style>body{background:#10151a;color:#eee;font:16px system-ui;max-width:1500px;margin:auto;padding:35px}a{color:#e8c58d}.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(380px,1fr));gap:15px}img{max-width:100%}figure{margin:0}details{padding:12px;border:1px solid #50606a;margin:10px 0}</style>'+''.join(parts),encoding='utf-8')
