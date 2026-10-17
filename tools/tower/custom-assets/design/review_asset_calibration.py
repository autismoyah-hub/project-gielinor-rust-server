"""Build private review media and source-safe aggregate diagnostics separately."""
import json,sys,hashlib,csv,html
from pathlib import Path
from collections import Counter
from PIL import Image,ImageChops,ImageDraw
root=Path(sys.argv[1]).resolve();out=root/'review';out.mkdir(exist_ok=True)
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def metric(a,b):
    a,b=a.convert('RGB'),b.convert('RGB');delta=ImageChops.difference(a,b);c=delta.split();m=ImageChops.lighter(ImageChops.lighter(c[0],c[1]),c[2]);bg=Image.new('RGB',a.size,(16,21,26))
    def mask(im):
        q=ImageChops.difference(im,bg).split();return ImageChops.lighter(ImageChops.lighter(q[0],q[1]),q[2]).point(lambda x:255 if x else 0)
    ma,mb=mask(a),mask(b);inter=ImageChops.darker(ma,mb).histogram()[255];union=ImageChops.lighter(ma,mb).histogram()[255]
    return dict(changed_pixels=a.width*a.height-m.histogram()[0],silhouette_iou=inter/union if union else 1,meaning='same pose/camera union masks; diagnostic, not artistic approval')
def geometry(g):
    edges=Counter(tuple(sorted((a,b)))for f in g['faces']for a,b in zip(f,f[1:]+f[:1]));vs=g['vertices']
    return dict(vertices=len(vs),faces=len(g['faces']),bounds=[[min(v[i]for v in vs)for i in range(3)],[max(v[i]for v in vs)for i in range(3)]],boundary_edges=sum(n==1 for n in edges.values()),overlapping_edges=sum(n>2 for n in edges.values()),palette_colors=len(set(g['colors'])),texture_faces=sum(t>=0 for t in g.get('face_textures')or []),face_priorities=g.get('priorities') is not None,skin_labels=len(set(g.get('vertex_skins')or [])))
report={};cards=[]
def panel(kind,filename,label):
    canvas=Image.new('RGB',(1050,360),(16,21,26));draw=ImageDraw.Draw(canvas)
    for i,(prefix,title)in enumerate([('native','A: original cache'),('roundtrip','B: Blender round trip'),('independent','C: independently authored')]):
        p=root/f'{prefix}-{kind}'/filename;im=Image.open(p).convert('RGB');im.thumbnail((350,325));canvas.paste(im,(i*350,35));draw.text((i*350+10,10),title,fill='white')
    return canvas
for kind in ['static','npc']:
    report[kind]={prefix:geometry(json.loads((root/f'{prefix}-{kind}'/'decoded.json').read_text()))for prefix in ['native','roundtrip','independent']}
    report[kind]['independent_views']={}
    for view in ['front','quarter','reverse']:
        filename=f'bind-{view}.bmp';name=f'{kind}-{view}.png';panel(kind,filename,view).save(out/name);cards.append(f'<figure><img src="{name}"><figcaption>{kind}: {view}; identical per-panel native camera and lighting.</figcaption></figure>')
        report[kind]['independent_views'][view]=metric(Image.open(root/f'native-{kind}'/filename),Image.open(root/f'independent-{kind}'/filename))
    scene=root/f'compare-{kind}'/'three-way-scene.bmp';Image.open(scene).save(out/f'{kind}-single-scene.png');cards.append(f'<figure><img src="{kind}-single-scene.png"><figcaption>One native scene: A / B / C. Shared lighting; perspective varies with placement. Use panels above for exact camera comparisons.</figcaption></figure>')
human=root/'compare-static'/'authentic-human-scale.bmp';Image.open(human).save(out/'human-scale.png');cards.append('<figure><img src="human-scale.png"><figcaption>A / authentic Man 3106 / B / Man / C / Man; unanimated native reference, 202 units = 1.578125 tiles tall.</figcaption></figure>')
temporal={};motions=[]
for action in ['idle','walk','attack','hit','death','blend']:
    files=sorted((root/'native-npc').glob(f'{action}-*.bmp'));rows=[];frames=[];ticks=[]
    for p in files:
        tick=int(p.stem.split('-')[1]);a=Image.open(p);b=Image.open(root/'independent-npc'/p.name);rows.append(dict(tick=tick,**metric(a,b)))
        if tick%3==0 or p==files[-1]:frames.append(panel('npc',p.name,action));ticks.append(tick)
    durations=[max(20,(b-a)*20)for a,b in zip(ticks,ticks[1:])]+[20]
    frames[0].save(out/f'{action}.gif',save_all=True,append_images=frames[1:],duration=durations,loop=0,optimize=False)
    temporal[action]=dict(samples=rows,rendered_timestamps=len(files),gif_timestamps=ticks,gif_duration_ms=sum(durations),note='GIF is a review copy of native Rust raster frames. It is not the asset or animation implementation. Tick = 20 ms. Playback repeats for inspection; native non-looping action policy remains separately recorded.')
    motions.append(f'<figure><img src="{action}.gif"><figcaption>{action}: A / B / C, identical timestamps. The independent actions are authored equivalents, not copies of original poses. Death has a long terminal hold. Blend = attack plus walking.</figcaption></figure>')
report['temporal_independent']=temporal
report['roundtrip']=json.loads((root/'exact-comparison.json').read_text())
report['witness']={}
for name in ['static.json','walk/animation.json']:
    a=json.loads((root/'reopened-roundtrip'/name).read_text());b=json.loads((root/'mutation-witness'/name).read_text());report['witness'][name]=a!=b
assert all(report['witness'].values()),'Blender mutation witness failed'
assert all(v['all_pixels_exact']for v in report['roundtrip'].values()),'Round-trip comparison failed'
report['source_files']={str(p.relative_to(root.parent)):digest(p)for p in (root.parent/'reference-final').rglob('*')if p.is_file()}
report['build_sha256']=digest(root.parents[1]/'build/client/release/asset_calibration.exe')
report['status']={'stage1':'PASS for selected untextured wall; three native views pixel exact','stage2':'Independent static study rendered; dimensional and visual comparison supplied; artistic approval not asserted','stage3':'PASS selected untextured classic NPC geometry/material/skin/depth channels','stage4':'PASS sampled original and re-encoded classic animations, plus native masked blend; not general cached-skeletal conversion','stage5':'Independent model and five authored actions render natively; likeness and choreography differ; artistic approval not asserted','floor_production':'SUSPENDED; no promotion or map change'}
(root/'diagnostics.json').write_text(json.dumps(report,indent=2))
safe=dict(status=report['status'],geometry={k:{p:report[k][p]for p in ['native','roundtrip','independent']}for k in ['static','npc']},roundtrip={k:dict(frames=len(v['frames']),all_pixels_exact=v['all_pixels_exact'],channels=v['channels'])for k,v in report['roundtrip'].items()},witness=report['witness'],native_binary_sha256=report['build_sha256'],independent_mask_metrics={k:report[k]['independent_views']for k in ['static','npc']},animation_samples={k:v['rendered_timestamps']for k,v in temporal.items()})
(root/'SOURCE_SAFE_SUMMARY.json').write_text(json.dumps(safe,indent=2))
body=''.join(cards)+''.join(motions)
(out/'index.html').write_text('<!doctype html><meta charset="utf-8"><title>ASCENT controlled asset calibration</title><style>body{background:#10151a;color:#e6e8e8;font:17px system-ui;max-width:1250px;margin:40px auto;padding:20px}h1{font-size:32px}img{max-width:100%;display:block}figure{margin:26px 0;background:#182128;padding:12px}figcaption{padding:12px;color:#c5cecf}a{color:#87c9ff}p{line-height:1.6}pre{white-space:pre-wrap}</style><h1>Static object and animated NPC calibration</h1><p>PRIVATE REFERENCE REVIEW. A is the genuine cache asset. B is exported from a saved Blender file reopened in a fresh process, then re-encoded under custom identifiers. C uses independently authored geometry and classic group actions. No extracted original or direct reconstruction is published as Tower art.</p><p><strong>Technical conversion passes for these selected untextured classic assets. Artistic approval is not asserted.</strong> The independent model has visible anatomy, equipment and shading differences; its actions have matching durations but different choreography. Floor production remains suspended.</p><p><a href="../diagnostics.json">Full local diagnostics</a> · <a href="../exact-comparison.json">Exact round-trip oracle</a> · <a href="../SOURCE_SAFE_SUMMARY.json">Source-safe summary</a></p>'+body,encoding='utf8')
print('PRIVATE_REVIEW_AND_SOURCE_SAFE_SUMMARY_COMPLETE',out/'index.html')
