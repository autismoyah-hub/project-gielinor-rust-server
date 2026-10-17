"""Local-only exact geometry/temporal/raster oracle and review generator."""
import json,hashlib,sys
from pathlib import Path
from PIL import Image,ImageChops,ImageDraw
root=Path(sys.argv[1]).resolve()
report={}
for kind in ['static','npc']:
    a,b=[root/f'{prefix}-{kind}'for prefix in ['native','roundtrip']]
    ga,gb=[json.loads((p/'decoded.json').read_text())for p in [a,b]]
    # Per-face priority streams override the unused global fallback value.
    diff={k:ga[k]==gb[k]for k in ga}
    if ga['priorities'] is not None and gb['priorities'] is not None:diff['priority']='unused fallback: arrays identical'if ga['priorities']==gb['priorities']else False
    frames=[]
    for p in sorted(a.glob('*.bmp')):
        q=b/p.name
        ia,ib=Image.open(p).convert('RGB'),Image.open(q).convert('RGB')
        delta=ImageChops.difference(ia,ib);channels=delta.split();mask=ImageChops.lighter(ImageChops.lighter(channels[0],channels[1]),channels[2]);changed=ia.width*ia.height-mask.histogram()[0]
        row=dict(frame=p.stem,changed_pixels=changed,total_pixels=ia.width*ia.height,pixel_exact=delta.getbbox()is None)
        if (a/(p.stem+'.json')).exists():
            va=json.loads((a/(p.stem+'.json')).read_text());vb=json.loads((b/(p.stem+'.json')).read_text())
            row['max_vertex_error_native']=max(abs(x-y)for v,w in zip(va['vertices'],vb['vertices'])for x,y in zip(v,w));row['alpha_exact']=va['alpha']==vb['alpha']
        frames.append(row)
    report[kind]=dict(vertices=len(ga['vertices']),faces=len(ga['faces']),channels=diff,frames=frames,all_pixels_exact=all(f['pixel_exact']for f in frames))
(root/'exact-comparison.json').write_text(json.dumps(report,indent=2))
print(json.dumps({k:dict(vertices=v['vertices'],faces=v['faces'],channels=v['channels'],frames=len(v['frames']),all_pixels_exact=v['all_pixels_exact'])for k,v in report.items()},indent=2))
