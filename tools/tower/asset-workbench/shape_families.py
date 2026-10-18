"""Owned curved, faceted and sector-shell geometry in native units."""
import math

def cross(a,b):return [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]
def unit(v):
    d=math.sqrt(sum(x*x for x in v))
    if d<1e-8:raise ValueError('Degenerate sweep direction')
    return [x/d for x in v]

def sweep(sections,sides=6,normal=(0,0,1),diamond=False):
    if len(sections)<2 or not 3<=sides<=16:raise ValueError('Sweep sections/sides')
    vertices=[];faces=[];previous=None
    for i,row in enumerate(sections):
        p=row[:3];w,h=row[3:5]
        if min(w,h)<=0:raise ValueError('Sweep radii must remain positive')
        lo=sections[max(0,i-1)][:3];hi=sections[min(len(sections)-1,i+1)][:3];t=unit([hi[j]-lo[j]for j in range(3)]);n=[normal[j]-sum(normal[k]*t[k]for k in range(3))*t[j]for j in range(3)]
        if sum(x*x for x in n)<1e-6:n=cross(t,[1,0,0]if abs(t[0])<.8 else[0,1,0])
        n=unit(n);u=unit(cross(t,n))
        if previous and sum(u[j]*previous[j]for j in range(3))<0:u=[-x for x in u];n=[-x for x in n]
        previous=u
        for k in range(sides):
            angle=math.tau*k/sides;vertices.append([round(p[j]+w*math.cos(angle)*u[j]+h*math.sin(angle)*n[j])for j in range(3)])
    for ring in range(len(sections)-1):
        for j in range(sides):
            a=ring*sides+j;b=ring*sides+(j+1)%sides;c=b+sides;d=a+sides;faces.extend([[a,b,c],[a,c,d]])
    for offset,rev in [(0,True),((len(sections)-1)*sides,False)]:
        for j in range(1,sides-1):faces.append([offset,offset+j+1,offset+j]if rev else[offset,offset+j,offset+j+1])
    return vertices,faces

def stave(sections,start_angle,end_angle,thickness=4):
    if thickness<=0 or end_angle<=start_angle:raise ValueError('Stave thickness/angular interval')
    vs=[];fs=[]
    for y,rx,rz in sections:
        if min(rx,rz)<=thickness:raise ValueError('Stave wall exceeds radius')
        for a,r in [(start_angle,0),(end_angle,0),(end_angle,thickness),(start_angle,thickness)]:vs.append([round((rx-r)*math.cos(a)),round(y),round((rz-r)*math.sin(a))])
    for k in range(len(sections)-1):
        for i in range(4):a=k*4+i;b=k*4+(i+1)%4;c=b+4;d=a+4;fs.extend([[a,b,c],[a,c,d]])
    fs.extend([[0,2,1],[0,3,2]]);o=(len(sections)-1)*4;fs.extend([[o,o+1,o+2],[o,o+2,o+3]])
    return vs,fs

def voussoir(center,inner,outer,start,end,depth):
    if not 0<inner<outer or depth<=0:raise ValueError('Arch wedge dimensions')
    cx,cy,cz=center;vs=[]
    for z in [-depth/2,depth/2]:
        for a,r in [(start,inner),(end,inner),(end,outer),(start,outer)]:vs.append([round(cx+math.cos(a)*r),round(cy-math.sin(a)*r),round(cz+z)])
    return vs,[[0,2,1],[0,3,2],[4,5,6],[4,6,7],[0,1,5],[0,5,4],[1,2,6],[1,6,5],[2,3,7],[2,7,6],[3,0,4],[3,4,7]]

def chamfer_block(center,size,bevel=3,wear=0,seed=0):
    from primitives import loft
    x,y,z=center;w,h,d=size;rx,rz=w/2,d/2;b=min(bevel,rx*.35,rz*.35,h*.2)
    if min(size)<=0:raise ValueError('Stone block dimensions')
    outline=[[-1,-1+b/rz],[-1+b/rx,-1],[1-b/rx,-1],[1,-1+b/rz],[1,1-b/rz],[1-b/rx,1],[-1+b/rx,1],[-1,1-b/rz]]
    sections=[[x,y+h/2,z,max(1,rx-b),max(1,rz-b)],[x,y+h/2-b,z,rx,rz],[x,y-h/2+b,z,rx,rz],[x,y-h/2,z,max(1,rx-b),max(1,rz-b)]];vs,fs=loft(sections,outline=outline)
    # One controlled corner fracture, no global random displacement/noise.
    if wear:
        corner=(seed*3)%8
        for ring in [2,3]:
            i=ring*8+corner;vs[i][0]=round(vs[i][0]+(x-vs[i][0])*wear);vs[i][2]=round(vs[i][2]+(z-vs[i][2])*wear)
    return vs,fs
