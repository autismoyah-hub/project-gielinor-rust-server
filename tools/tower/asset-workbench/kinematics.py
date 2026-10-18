"""Classic integer transform tracer and explicit joint-to-group compiler.

This produces geometry/operation diagnostics, never pixels. Its results must be
validated against native Rust deformation before being used as an authoring preview.
"""
import math,copy
def div(a,b):return int(a/b)
def rotate(v,angles,pivot):
    v=[int(v[i])-pivot[i]for i in range(3)]
    for axis in [2,0,1]:
        angle=(int(angles[axis])&255)*8
        if not angle:continue
        r=angle*0.0030679615;s,c=int(math.sin(r)*65536),int(math.cos(r)*65536);a,b,invert=(0,1,False)if axis==2 else (1,2,True)if axis==0 else (0,2,False);x,y=v[a],v[b]
        if invert:v[a]=(c*x-s*y)>>16;v[b]=(s*x+c*y)>>16
        else:v[a]=(s*y+c*x)>>16;v[b]=(c*y-s*x)>>16
    return [v[i]+pivot[i]for i in range(3)]
def trace(g,frame):
    vs=[list(map(int,v))for v in g['vertices']];skins=g.get('vertex_skins');pivot=[0]*3;records=[]
    if skins is None:return vs,records
    groups={n:[i for i,v in enumerate(skins)if v==n]for n in set(skins)}
    for label,value in frame['transforms']:
        kind=frame['types'][label];ids=[i for group in frame['labels'][label]for i in groups.get(group,[])]
        if kind==0:pivot=[value[a]+(div(sum(vs[i][a]for i in ids),len(ids))if ids else 0)for a in range(3)]
        elif kind==1:
            for i in ids:vs[i]=[vs[i][a]+value[a]for a in range(3)]
        elif kind==2:
            for i in ids:vs[i]=rotate(vs[i],value,pivot)
        elif kind==3:
            for i in ids:vs[i]=[div((vs[i][a]-pivot[a])*value[a],128)+pivot[a]for a in range(3)]
        elif kind!=5:raise ValueError('unsupported classic transform type '+str(kind))
        records.append(dict(label=label,type=kind,pivot=list(pivot),affected_vertices=len(ids),values=value))
    return vs,records
def frame_at(rows,tick):
    elapsed=tick
    for i,row in enumerate(rows):
        if elapsed<=row['length']:return i,row
        elapsed-=row['length']
    return len(rows)-1,rows[-1]
def two_bone_yz(hip,target,lengths):
    """Planar IK; diagnostic reach flag is explicit, not hidden limb stretching."""
    y,z=target[1]-hip[1],target[2]-hip[2];a,b=lengths;raw=math.hypot(y,z);d=max(abs(a-b)+1e-5,min(a+b-1e-5,raw));alpha=math.atan2(z,y);offset=math.acos(max(-1,min(1,(d*d+a*a-b*b)/(2*d*a))));knee=math.pi-math.acos(max(-1,min(1,(a*a+b*b-d*d)/(2*a*b))));return dict(hip_native=round((alpha-offset)*128/math.pi),knee_native=round(knee*128/math.pi),reachable=abs(a-b)<=raw<=a+b,unclamped_distance=raw)
def compile_joints(g,rig,channels,translation=(0,0,0)):
    """Compensate native dynamic-centroid pivots deliberately for authored joints.

    Joint origins are control points transformed by their already-applied parents.
    Vertex ownership comes from explicit authored group lists, not guessed proximity.
    """
    labels=sorted(set(g['vertex_skins']or []));types=[0,1];groups=[[],labels];transforms=[[0,[0,0,0]],[1,list(translation)]];vs=[[v[i]+translation[i]for i in range(3)]for v in g['vertices']];points={j['name']:[j['origin'][i]+translation[i]for i in range(3)]for j in rig};skins=g['vertex_skins'];byname={j['name']:j for j in rig}
    def descendant(name,parent):
        while name is not None:
            if name==parent:return True
            name=byname[name].get('parent')
        return False
    for joint in rig:
        owned=joint['groups'];indices=[i for i,label in enumerate(skins)if label in owned];centroid=[div(sum(vs[i][a]for i in indices),len(indices))if indices else 0 for a in range(3)];target=list(map(round,points[joint['name']]));offset=[target[a]-centroid[a]for a in range(3)];angles=list(map(round,channels.get(joint['name'],[0,0,0])));index=len(types);types.extend([0,2]);groups.extend([owned,owned]);transforms.extend([[index,offset],[index+1,angles]])
        for i in indices:vs[i]=rotate(vs[i],angles,target)
        for name,point in points.items():
            if name!=joint['name']and descendant(name,joint['name']):points[name]=rotate(point,angles,target)
    return dict(types=types,labels=groups,transforms=transforms),vs,points
