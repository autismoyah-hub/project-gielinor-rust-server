"""Owned section topology, fitted shell surfaces and constrained cage deformation.

Inputs are authored semantic dimensions, never cache topology. Native integer
quantization is the final step. Existing primitives opt in explicitly.
"""
import math

def sections(rows,profiles=None,sides=8,hem=None):
    if len(rows)<2 or not 3<=sides<=32:raise ValueError('Section topology needs >=2 rings and 3..32 sides')
    default=[[math.cos(i*math.tau/sides),math.sin(i*math.tau/sides)]for i in range(sides)]
    profiles=profiles or[default]*len(rows)
    if len(profiles)!=len(rows)or any(len(p)!=sides for p in profiles):raise ValueError('Section vertex correspondence must remain consistent')
    hem=hem or[[0]*sides for _ in rows]
    if len(hem)!=len(rows)or any(len(p)!=sides for p in hem):raise ValueError('Invalid per-ring hem controls')
    v=[];f=[]
    for ring,(x,y,z,rx,rz)in enumerate(rows):
        if min(rx,rz)<=0:raise ValueError('Section radii must be positive')
        for i,(u,w)in enumerate(profiles[ring]):v.append([round(x+u*rx),round(y+hem[ring][i]),round(z+w*rz)])
    for k in range(len(rows)-1):
        for i in range(sides):a=k*sides+i;b=k*sides+(i+1)%sides;c=b+sides;d=a+sides;f.extend([[a,b,c],[a,c,d]])
    # A center vertex avoids invalid triangle fans on asymmetric star-shaped caps.
    for ring,reverse in [(0,True),(len(rows)-1,False)]:
        ids=list(range(ring*sides,(ring+1)*sides));center=len(v);v.append([round(sum(v[i][a]for i in ids)/sides)for a in range(3)])
        for i in range(sides):tri=[center,ids[i],ids[(i+1)%sides]];f.append(list(reversed(tri))if reverse else tri)
    return v,f

def shell(rows,thickness=3,sides=8,start=0,end=math.tau):
    if thickness<=0 or len(rows)<2 or not 2<=sides<=32 or not 0<end-start<=math.tau+1e-7:raise ValueError('Shell dimensions')
    closed=abs(end-start-math.tau)<1e-6;n=sides if closed else sides+1;v=[];f=[]
    for inner in [False,True]:
        for x,y,z,rx,rz in rows:
            if min(rx,rz)<=thickness:raise ValueError('Shell thickness exceeds radius')
            for i in range(n):a=start+(end-start)*i/sides;v.append([round(x+(rx-(thickness if inner else 0))*math.cos(a)),round(y),round(z+(rz-(thickness if inner else 0))*math.sin(a))])
    layer=n*len(rows)
    def quad(a,b,c,d,reverse=False):
        tris=[[a,b,c],[a,c,d]];f.extend([list(reversed(t))for t in tris]if reverse else tris)
    for k in range(len(rows)-1):
        for i in range(sides):a=k*n+i;b=k*n+(i+1)%n;quad(a,b,b+n,a+n);quad(a+layer,b+layer,b+n+layer,a+n+layer,True)
    for k,rev in [(0,False),(len(rows)-1,True)]:
        for i in range(sides):a=k*n+i;b=k*n+(i+1)%n;quad(a,a+layer,b+layer,b,rev)
    if not closed:
        for i,rev in [(0,False),(n-1,True)]:
            for k in range(len(rows)-1):a=k*n+i;quad(a,a+n,a+n+layer,a+layer,rev)
    return v,f

def gear(center,radius,root_radius,teeth,depth,bore=0):
    if not 4<=teeth<=32 or not 0<root_radius<radius or depth<=0:raise ValueError('Gear dimensions')
    points=[]
    for i in range(teeth):
        for phase,r in [(0,root_radius),(.22,radius),(.66,radius),(.9,root_radius)]:
            a=(i+phase)*math.tau/teeth;points.append([r*math.cos(a),r*math.sin(a)])
    n=len(points);cx,cy,cz=center;v=[[round(cx+x),round(cy+y),round(cz+z)]for z in [-depth/2,depth/2]for x,y in points];f=[]
    # Circular bore and star-shaped gear use corresponding outer/inner rings.
    if bore:
        if not 0<bore<root_radius:raise ValueError('Gear bore')
        v += [[round(cx+bore*math.cos(math.atan2(y,x))),round(cy+bore*math.sin(math.atan2(y,x))),round(cz+z)]for z in [-depth/2,depth/2]for x,y in points]
        for i in range(n):
            j=(i+1)%n
            for a,b,c,d in [(i,j,2*n+j,2*n+i),(n+i,3*n+i,3*n+j,n+j),(i,n+i,n+j,j),(2*n+i,2*n+j,3*n+j,3*n+i)]:f.extend([[a,b,c],[a,c,d]])
    else:
        v.extend([[round(cx),round(cy),round(cz-depth/2)],[round(cx),round(cy),round(cz+depth/2)]])
        for i in range(n):j=(i+1)%n;f.extend([[i,j,n+j],[i,n+j,n+i],[2*n,j,i],[2*n+1,n+i,n+j]])
    return v,f

def cage(vertices,spec):
    """Trilinear eight-corner authored warp; reject local foldover before rounding."""
    lo,hi=spec['bounds'];offsets=spec['offsets']
    if len(offsets)!=8 or any(len(x)!=3 for x in offsets)or any(hi[i]<=lo[i]for i in range(3)):raise ValueError('Cage requires 8 finite corners and positive bounds')
    if any(not math.isfinite(c)for p in offsets for c in p):raise ValueError('Non-finite cage')
    def mapped(p):
        t=[(p[i]-lo[i])/(hi[i]-lo[i])for i in range(3)];d=[0.,0.,0.]
        for k,o in enumerate(offsets):
            w=math.prod(t[i]if(k>>i)&1 else 1-t[i]for i in range(3))
            for i in range(3):d[i]+=o[i]*w
        return[p[i]+d[i]for i in range(3)]
    for x in [0,.5,1]:
        for y in [0,.5,1]:
            for z in [0,.5,1]:
                p=[lo[i]+[x,y,z][i]*(hi[i]-lo[i])for i in range(3)];base=mapped(p);columns=[]
                for i in range(3):q=list(p);q[i]+=.01;columns.append([(v-base[a])/.01 for a,v in enumerate(mapped(q))])
                a,b,c=columns;det=a[0]*(b[1]*c[2]-b[2]*c[1])-b[0]*(a[1]*c[2]-a[2]*c[1])+c[0]*(a[1]*b[2]-a[2]*b[1])
                if det<=.05:raise ValueError('Authored deformation folds or collapses the cage')
    return[[round(c)for c in mapped(p)]for p in vertices]

def topology(vertices,faces):
    from collections import Counter
    edges=Counter();directions=Counter()
    for face in faces:
        for a,b in zip(face,face[1:]+face[:1]):edges[tuple(sorted([a,b]))]+=1;directions[(a,b)]+=1
    return dict(boundary_edges=sum(v==1 for v in edges.values()),nonmanifold_edges=sum(v>2 for v in edges.values()),inconsistently_wound_edges=sum(v==2 and(directions[(a,b)]!=1 or directions[(b,a)]!=1)for(a,b),v in edges.items()),vertices=len(vertices),faces=len(faces))
