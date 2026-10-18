"""Fixed-coordinate multi-view diagnostics and regression ledger.

Inputs are private rendered images. No centering, warping or per-candidate
camera/scale adjustment is performed. Metrics are evidence, not artistic approval.
"""
import json,sys,math,hashlib
from pathlib import Path
import numpy as np
from PIL import Image,ImageFilter,ImageDraw
VIEWS=['front','side','rear','quarter','above','gameplay']

def load(path):
    im=Image.open(path).convert('RGB');a=np.asarray(im);bg=a[0,0].astype(int)
    mask=np.any(a.astype(int)!=bg,axis=2)
    return im,a,mask
def dilate(mask,radius=2):
    return np.asarray(Image.fromarray(mask.astype('uint8')*255).filter(ImageFilter.MaxFilter(radius*2+1)))>0
def edge(mask):
    return dilate(mask,1)&~np.asarray(Image.fromarray(mask.astype('uint8')*255).filter(ImageFilter.MinFilter(3))).astype(bool)
def f1(a,b,radius=2):
    p=float((a&dilate(b,radius)).sum())/max(1,int(a.sum()));r=float((b&dilate(a,radius)).sum())/max(1,int(b.sum()))
    return 2*p*r/max(1e-9,p+r)
def internal_edges(a,mask):
    gray=a.mean(2);gradient=np.zeros(mask.shape,dtype=bool)
    gradient[:,1:]|=np.abs(np.diff(gray,axis=1))>10;gradient[1:]|=np.abs(np.diff(gray,axis=0))>10
    return gradient&mask&~dilate(edge(mask),2)
def material_regions(a,mask):
    # Display-space chromaticity/value regions, not source material labels.
    f=a.astype(float)/255;hi=f.max(2);lo=f.min(2);s=(hi-lo)/(hi+1e-6)
    dominant=f.argmax(2);neutral=s<.19
    codes=dominant*3+(hi>.12).astype(int)+(hi>.36).astype(int)
    codes[neutral]=9+(hi[neutral]>.12).astype(int)+(hi[neutral]>.36).astype(int)
    codes[~mask]=-1;return codes
def diagnostic(ref,candidate):
    ri,ra,rm=load(ref);ci,ca,cm=load(candidate);assert ri.size==ci.size,'camera/viewport mismatch'
    inter=rm&cm;union=rm|cm;rc,cc=material_regions(ra,rm),material_regions(ca,cm)
    agreement=float(((rc==cc)&inter).sum())/max(1,int(union.sum()))
    re,ce=edge(rm),edge(cm)
    # Multi-radius contour match avoids calling an IoU-only ranking sufficient.
    contour=sum(f1(re,ce,r) for r in [1,3,6])/3
    return {'silhouette_iou':float(inter.sum())/max(1,int(union.sum())),
      'missing_fraction':float((rm&~cm).sum())/max(1,int(rm.sum())),
      'excess_fraction':float((cm&~rm).sum())/max(1,int(rm.sum())),
      'area_ratio':float(cm.sum())/max(1,int(rm.sum())),
      'contour_f1':contour,'internal_edge_f1':f1(internal_edges(ra,rm),internal_edges(ca,cm),3),
      'display_region_agreement':agreement,'reference_pixels':int(rm.sum()),'candidate_pixels':int(cm.sum())}
def compare(reference,candidate,previous=None):
    views={v:diagnostic(Path(reference)/(v+'.png'),Path(candidate)/(v+'.png')) for v in VIEWS}
    result={'views':views,'mean_iou':sum(x['silhouette_iou'] for x in views.values())/6,'worst_iou':min(x['silhouette_iou'] for x in views.values())}
    # A material/edge diagnostic is intentionally subordinate to coherent shape.
    result['objective']=sum(.55*x['silhouette_iou']+.25*x['contour_f1']+.12*x['display_region_agreement']+.08*x['internal_edge_f1']for x in views.values())/6
    result['regressions']=[]
    if previous:
        result['deltas']={v:{k:views[v][k]-previous['views'][v][k] for k in ['silhouette_iou','contour_f1','internal_edge_f1','display_region_agreement']}for v in VIEWS}
        result['regressions']=[v for v,d in result['deltas'].items()if d['silhouette_iou']<-.015 or d['contour_f1']<-.04]
    return result
def panels(reference,previous,candidate,out):
    out=Path(out);out.mkdir(parents=True,exist_ok=True)
    for view in VIEWS:
        ims=[Image.open(Path(folder)/(view+'.png')).convert('RGB')for folder in [reference,previous,candidate]]
        # A common full frame is retained in saved evidence. Review thumbnails
        # may share one union crop, never independent alignment.
        a,b,c=ims;panel=Image.new('RGB',(a.width*3,a.height+30),'#171b20');d=ImageDraw.Draw(panel)
        for i,(im,label)in enumerate(zip(ims,['Authentic / fixed camera','Preserved previous','New candidate'])):panel.paste(im,(i*a.width,30));d.text((i*a.width+12,8),label,fill='white')
        panel.save(out/(view+'-comparison.png'));Image.blend(a,c,.5).save(out/(view+'-overlay.png'))
        _,_,rm=load(Path(reference)/(view+'.png'));_,_,cm=load(Path(candidate)/(view+'.png'))
        rgb=np.zeros((*rm.shape,3),dtype='uint8');rgb[rm&cm]=(120,135,145);rgb[rm&~cm]=(255,80,60);rgb[cm&~rm]=(40,170,255)
        Image.fromarray(rgb).save(out/(view+'-silhouette.png'))
    return out

if __name__=='__main__':
    reference,previous,candidate,out=map(Path,sys.argv[1:5]);before=compare(reference,previous);after=compare(reference,candidate,before);out.mkdir(parents=True,exist_ok=True)
    (out/'metrics.json').write_text(json.dumps({'previous':before,'candidate':after},indent=2));panels(reference,previous,candidate,out)
    print(json.dumps({'mean':after['mean_iou'],'worst':after['worst_iou'],'regressions':after['regressions']}))
