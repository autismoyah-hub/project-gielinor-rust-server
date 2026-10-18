"""Extract untouched illustration pixels and estimated silhouette constraints.

No image generation or redesign. Background/label removal is a transparent
modeling-copy operation; all source RGB pixels are retained beneath alpha.
"""
from pathlib import Path
import json,hashlib
import numpy as np
from PIL import Image,ImageFilter,ImageDraw
from collections import deque
T=Path(__file__).resolve().parent;R=T.parents[3]/'ashhorn-reference-private';D=R/'references';D.mkdir(exist_ok=True)
raw=Image.open(R/'baseline/original-art/turnaround-v2.png').convert('RGB')
panels={'front':([0,0,584,512],[78,7,470,501],493),'left':([584,0,1536,512],[30,17,941,502],490),'rear':([0,512,584,1024],[78,7,474,482],470),'quarter':([584,512,1536,1024],[125,12,943,491],475)}
manifest={'canonical_sha256':hashlib.sha256((R/'baseline/original-art/turnaround-v2.png').read_bytes()).hexdigest(),'highest_available_original_resolution':list(raw.size),'operation':'Exact pixel crops; robust quadratic background estimate, silhouette connected-component extraction and alpha removal of disconnected labels/dividers. No RGB repainting, stretching, generative repair or upscaling.','accuracy':'Silhouette masks are estimates requiring visual inspection, not exact ground truth. Antialiased edges and contact shadows carry uncertainty.','panels':{}}
def largest(a):
    visited=np.zeros_like(a);best=[];h,w=a.shape
    for y,x in zip(*np.where(a & ~visited)):
        if visited[y,x]:continue
        q=deque([(y,x)]);visited[y,x]=1;region=[]
        while q:
            cy,cx=q.popleft();region.append((cy,cx))
            for dy,dx in [(0,1),(0,-1),(1,0),(-1,0)]:
                ny,nx=cy+dy,cx+dx
                if 0<=ny<h and 0<=nx<w and a[ny,nx] and not visited[ny,nx]:visited[ny,nx]=1;q.append((ny,nx))
        if len(region)>len(best):best=region
    result=np.zeros_like(a)
    for y,x in best:result[y,x]=True
    return result
for name,(panel,crop,ground) in panels.items():
    image=raw.crop(panel);rgb=np.array(image).astype(float);h,w=rgb.shape[:2];yy,xx=np.mgrid[:h,:w];x=xx/max(1,w-1);y=yy/max(1,h-1);basis=np.stack([np.ones_like(x),x,y,x*x,x*y,y*y],-1)
    r,g,b=rgb.transpose(2,0,1);sample=(r-g>2)&(r-g<12)&(g-b>3)&(g-b<17)&(g>115)&(g<185)&(yy<ground-4)
    sample &= (xx%3==0)&(yy%3==0)
    pts=basis[sample];values=rgb[sample];keep=np.ones(len(values),bool)
    for _ in range(5):
        coef=np.linalg.lstsq(pts[keep],values[keep],rcond=None)[0];err=np.linalg.norm(pts@coef-values,axis=1);keep=err<max(8,np.quantile(err,.78))
    bg=basis@coef;difference=np.linalg.norm(rgb-bg,axis=2)
    candidate=(difference>23)&(yy<ground+1)
    allowed=np.zeros((h,w),bool);allowed[crop[1]:min(crop[3],ground+1),crop[0]:crop[2]]=True;candidate &= allowed
    m=Image.fromarray(candidate.astype('uint8')*255).filter(ImageFilter.MaxFilter(3)).filter(ImageFilter.MinFilter(3));candidate=largest(np.array(m)>0)
    # Fill enclosed sampling holes while keeping the four-leg negative spaces.
    padded=Image.new('L',(w+2,h+2));padded.paste(Image.fromarray(candidate.astype('uint8')*255),(1,1));ImageDraw.floodfill(padded,(0,0),128)
    flood=np.array(padded)[1:-1,1:-1];holes=flood==0;visited=np.zeros_like(holes);filled=candidate.copy()
    for sy,sx in zip(*np.where(holes)):
        if visited[sy,sx]:continue
        q=deque([(sy,sx)]);visited[sy,sx]=True;region=[]
        while q:
            cy,cx=q.popleft();region.append((cy,cx))
            for dy,dx in [(0,1),(0,-1),(1,0),(-1,0)]:
                ny,nx=cy+dy,cx+dx
                if 0<=ny<h and 0<=nx<w and holes[ny,nx] and not visited[ny,nx]:visited[ny,nx]=True;q.append((ny,nx))
        if len(region)<=300:
            for cy,cx in region:filled[cy,cx]=True
    # Explicitly exclude the long illustrated floor/contact-shadow line. These
    # conservative paw bands affect only the last18px and are recorded masks,
    # not repainted pixels or a claim of exact segmentation.
    paw_bands={'front':[(82,191),(350,465)],'left':[(90,205),(224,337),(475,588),(630,761)],'rear':[(163,246),(310,389)],'quarter':[(128,216),(222,333),(509,612),(680,794)]}[name]
    foot_area=np.zeros(w,bool)
    for lo,hi in paw_bands:foot_area[lo:hi]=True
    filled[(yy>ground-18)&~foot_area[None,:]]=False
    fullmask=Image.fromarray(filled.astype('uint8')*255);fullmask.save(D/(name+'-panel-mask.png'))
    result=image.convert('RGBA');result.putalpha(fullmask);result.save(D/(name+'-panel.png'));result.crop(crop).save(D/(name+'.png'));image.crop(crop).save(D/(name+'-original-crop.png'))
    fullmask.crop(crop).save(D/(name+'-mask.png'))
    manifest['panels'][name]={'original_panel':panel,'modeling_crop':crop,'size':[crop[2]-crop[0],crop[3]-crop[1]],'ground_y_panel_pixels':ground,'silhouette_pixels':int(filled.sum()),'background_fit_samples':int(keep.sum()),'mask_threshold_rgb_distance':23,'rgb_pixels_unchanged':True}
(D/'preparation.json').write_text(json.dumps(manifest,indent=2)+'\n')
print('Four existing illustration copies prepared without new generated pixels.')
