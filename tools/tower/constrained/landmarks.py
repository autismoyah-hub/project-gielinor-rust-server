"""Fixed-camera surface landmarks and opaque-geometry occlusion probes.

Anchors are author-supplied observations, not automatic anatomical truth. Closest
surface vertices are diagnostics only; the generator never imports these arrays.
"""
import math,json
from pathlib import Path
import numpy as np
from geometry_audit import points

def camera(text):
    f=text.split()
    if f[0]=='look':
        v=list(map(float,f[1:]));eye=np.array(v[:3]);forward=np.array(v[3:6])-eye;forward/=np.linalg.norm(forward);right=np.cross(forward,[0,-1,0]);right/=np.linalg.norm(right);up=np.cross(right,forward)
        return eye,np.stack([right,-up,forward]),v[6],int(v[7]),int(v[8])
    focus=np.array(list(map(int,f[1:4])));pitch,yaw,distance=map(int,f[4:]);z=320*distance//256;y=x=0
    trig=lambda angle:(int(math.sin((angle&16383)*math.tau/16384)*65536),int(math.cos((angle&16383)*math.tau/16384)*65536))
    sin,cos=trig(-pitch);y=(-z*sin)>>16;z=(z*cos)>>16
    sin,cos=trig(-yaw);x=(sin*z)>>16;z=(cos*z)>>16;eye=focus-np.array([x,y,z])
    sp,cp=math.sin(pitch*math.tau/16384),math.cos(pitch*math.tau/16384);sy,cy=math.sin(yaw*math.tau/16384),math.cos(yaw*math.tau/16384)
    return eye,np.array([[cy,0,sy],[sy*sp,cp,-cy*sp],[-sy*cp,sp,cy*cp]]),308,765,503

def project(author_point,cam):
    x,depth,height=author_point;eye,m,f,w,h=cam;q=m@(np.array([x,-height,depth])-eye)
    return [int(w//2+q[0]*f/q[2]),int(h//2+q[1]*f/q[2])] if q[2]>=50 else None

def visible(author_point,g,cam,tolerance=2):
    eye=cam[0];world=np.array([author_point[0],-author_point[2],author_point[1]]);direction=world-eye;distance=np.linalg.norm(direction)
    if distance<1e-8:return True
    direction/=distance;tri=np.asarray(g['vertices'])[np.asarray(g['faces'])];e1=tri[:,1]-tri[:,0];e2=tri[:,2]-tri[:,0];h=np.cross(direction,e2);det=np.sum(e1*h,axis=1);ok=abs(det)>1e-8;inv=np.zeros(len(det));inv[ok]=1/det[ok];s=eye-tri[:,0];u=inv*np.sum(s*h,axis=1);q=np.cross(s,e1);v=inv*np.sum(direction*q,axis=1);t=inv*np.sum(e2*q,axis=1);hits=t[ok&(u>=0)&(v>=0)&(u+v<=1)&(t>0)]
    return bool(not len(hits) or float(hits.min())>=distance-tolerance)

def evaluate(reference,candidate,anchors,cameras):
    rg=json.loads(Path(reference).read_text());cg=json.loads(Path(candidate).read_text());rp,cp=points(rg),points(cg);result={}
    for name,target in anchors.items():
        anchor=np.array(target);r=rp[np.linalg.norm(rp-anchor,axis=1).argmin()];c=cp[np.linalg.norm(cp-anchor,axis=1).argmin()];row={'observed_anchor':target,'reference_surface_probe':r.tolist(),'candidate_surface_probe':c.tolist(),'surface_distance':float(np.linalg.norm(c-r)),'views':{}}
        for view,text in cameras.items():
            cam=camera(text);rxy,cxy=project(r,cam),project(c,cam);rv,cv=visible(r,rg,cam),visible(c,cg,cam)
            row['views'][view]={'reference_pixel':rxy,'candidate_pixel':cxy,'pixel_distance':float(np.linalg.norm(np.array(rxy)-cxy))if rxy and cxy else None,'reference_visible':rv,'candidate_visible':cv,'occlusion_mismatch':rv!=cv}
        result[name]=row
    return {'landmarks':result,'method':'Manually named fixed 3D anchors, nearest surface vertex probes, exact fixed camera transforms and opaque double-sided ray tests; not a fitted mesh or verified anatomical correspondences.'}

def overlay(images,result,out):
    from PIL import Image,ImageDraw
    out=Path(out);out.mkdir(parents=True,exist_ok=True)
    views=next(iter(result['landmarks'].values()))['views']
    for view in views:
        im=Image.open(Path(images)/(view+'.png')).convert('RGB');draw=ImageDraw.Draw(im)
        for name,row in result['landmarks'].items():
            item=row['views'][view]
            for field,color in [('reference_pixel','#ff7050'),('candidate_pixel','#50cfff')]:
                p=item[field]
                if p:draw.ellipse((p[0]-3,p[1]-3,p[0]+3,p[1]+3),outline=color,width=2)
            if item['candidate_pixel']:draw.text(tuple(x+5 for x in item['candidate_pixel']),name,fill='#50cfff')
        im.save(out/(view+'-landmarks.png'))

if __name__=='__main__':
    import sys
    reference,candidate,spec,out=sys.argv[1:];spec=json.loads(Path(spec).read_text());Path(out).write_text(json.dumps(evaluate(reference,candidate,spec['anchors'],spec['cameras']),indent=2))
