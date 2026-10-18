"""Strict matching-timestamp coverage and native image/coordinate comparison."""
import json,sys,struct
from pathlib import Path
from PIL import Image,ImageChops
def verify(root):
    rows=[];f32=lambda x:struct.unpack('<f',struct.pack('<f',x))[0]
    for action in sorted((root/'reference').glob('*/*/animation.json')):
        asset=action.parent.parent.name;name=action.parent.name;converted=root/'roundtrip'/asset/'actions'/name/'native';samples=[]
        for p in sorted(action.parent.glob('pose-*.bmp')):
            q=converted/p.name
            if not q.exists():raise AssertionError('Missing matching pose '+str(q))
            pixels=ImageChops.difference(Image.open(p).convert('RGB'),Image.open(q).convert('RGB')).getbbox()is None;a=json.loads(p.with_suffix('.json').read_text());b=json.loads(q.with_suffix('.json').read_text());error=max(abs(f32(x)-f32(y))for v,w in zip(a['vertices'],b['vertices'])for x,y in zip(v,w));samples.append(dict(tick=int(p.stem.split('-')[1]),pixels_exact=pixels,max_coordinate_error_native=error,alpha_exact=a.get('alpha')==b.get('alpha')))
        row=dict(asset=asset,action=name,source_sequence=json.loads(action.read_text())['source_sequence'],samples=samples,all_exact=bool(samples)and all(s['pixels_exact']and s['max_coordinate_error_native']==0 and s['alpha_exact']for s in samples));rows.append(row)
    if not rows or not all(r['all_exact']for r in rows):raise AssertionError('Animation preservation failed')
    (root/'roundtrip/verified-animation-results.json').write_text(json.dumps(rows,indent=2));print('STRICT_NATIVE_ANIMATION_PRESERVATION_PASS',len(rows),'actions',sum(len(r['samples'])for r in rows),'matched timestamps');return rows
if __name__=='__main__':verify(Path(sys.argv[1]))
