"""Common-camera observation of original-only meshes, with explicit limitations."""
from pathlib import Path
import sys,json,hashlib,copy
import numpy as np
from PIL import Image,ImageDraw
T=Path(__file__).resolve().parent
sys.path[:0]=[str(T),str(T.parent/'asset-workbench')]
from run import R,REF,CACHE,execute,views
from native_codec import model

SELECTED='18-jaw-sides-and-orbital-rims'
COMPARISON=R/'comparison-18'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def mask(path):
    a=np.asarray(Image.open(path).convert('RGB'));return np.max(abs(a.astype(int)-np.array([15,21,26])),axis=2)>5
def boundary(m):
    inner=m.copy()
    for dy,dx in [(-1,0),(1,0),(0,-1),(0,1)]:inner&=np.roll(m,(dy,dx),(0,1))
    return m&~inner
def symmetric_contour(a,b):
    pa=np.argwhere(boundary(a));pb=np.argwhere(boundary(b))
    def distance(p,q):
        values=[]
        for i in range(0,len(p),128):values.extend(np.sqrt(((p[i:i+128,None,:]-q[None,:,:])**2).sum(2).min(1)).tolist())
        return values
    d=distance(pa,pb)+distance(pb,pa)
    return {'mean_pixels':float(np.mean(d)),'p95_pixels':float(np.percentile(d,95))}
def closeups(asset,out):
    out.mkdir(parents=True,exist_ok=True)
    cameras={'face-front':'look 0 -125 -425 0 -125 -120 1600 900 700 0 0',
             'face-quarter':'look 205 -170 -345 0 -125 -120 1450 900 700 0 0',
             'shoulder':'look 230 -215 -170 30 -115 -40 1200 900 700 0 0',
             'forepaw':'look 160 -110 -260 55 -13 -65 1450 900 700 0 0',
             'hock':'look 340 -105 90 38 -45 80 1400 900 700 0 0'}
    for name,c in cameras.items():
        dest=out/name;dest.mkdir(exist_ok=True);cp=dest/'camera.txt';cp.write_text(c)
        execute([REF,CACHE,asset,dest,cp],dest/'log.txt');Image.open(dest/'native.bmp').save(out/(name+'.png'))

if __name__=='__main__':
    selected=R/'candidates'/SELECTED;comparison=COMPARISON;comparison.mkdir(exist_ok=True)
    stats={'selected':SELECTED,'method':'Frozen v1 perspective cameras, original illustration masks and native actor pixels; no independent candidate alignment or image stretching.',
           'limitations':['Illustration poses and perspective are inconsistent; rear and three-quarter alignment remain approximate.',
                         'Estimated reference masks include manual contact-plane cleanup. IoU and contour distance measure projected outline, not artistic fidelity.',
                         'Calibrated side camera has a low elevation and does not reproduce every painted top surface. Standard elevated cameras are also provided.'],
           'models':{}}
    for label,source in [('previous',R/'baseline/previous'),('reconstruction',selected)]:
        out=comparison/label
        if label=='previous':views(source/'asset.model',out/'native')
        else:out.mkdir(parents=True,exist_ok=True)
        native=(out/'native')if label=='previous'else source/'native'
        closeups(source/'asset.model',out/'closeups')
        g=json.loads((source/'geometry.json').read_text());clay=copy.deepcopy(g);clay['colors']=[54]*len(g['faces']);clay_path=out/'clay.model';clay_path.write_bytes(model(clay))
        views(clay_path,out/'clay')
        assert clay['vertices']==g['vertices'] and clay['faces']==g['faces'] and clay['render_types']==g['render_types']
        metrics={}
        for view in ['front','left','rear','quarter']:
            reference=np.asarray(Image.open(R/'references'/(view+'-panel.png')).convert('RGBA'))[:,:,3]>200
            rendered=mask(native/'reference'/(view+'.png'))
            metrics[view]={'silhouette_iou':float((reference&rendered).sum()/(reference|rendered).sum()),'contour':symmetric_contour(reference,rendered)}
            canvas=Image.new('RGB',(reference.shape[1],reference.shape[0]),'#0f151a');a=np.asarray(canvas).copy()
            a[reference]=[66,79,83];a[boundary(reference)]=[70,216,200];a[boundary(rendered)]=[248,169,85]
            Image.fromarray(a).save(out/(view+'-contours.png'))
        stats['models'][label]={'model_sha256':sha(source/'asset.model'),'triangles':len(g['faces']),'views':metrics,'clay_geometry_and_shading_flags_identical':True}
        print('OBSERVED',label,metrics,flush=True)
    (R/'evaluation.json').write_text(json.dumps(stats,indent=2)+'\n')
    (comparison/'evaluation.json').write_text(json.dumps(stats,indent=2)+'\n')
