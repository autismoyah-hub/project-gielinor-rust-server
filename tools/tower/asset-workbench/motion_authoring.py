"""Contact-aware authoring on the existing classic world-axis joint compiler.

These functions generate authoring controls, not a new runtime. Final controls
are baked into real Blender action curves and validated by native playback.
"""
import math,copy
from kinematics import rotate,compile_joints

def ancestors(rig,name):
    by={j['name']:j for j in rig};result=[]
    while name is not None:result.append(by[name]);name=by[name].get('parent')
    return list(reversed(result))

def forward(point,chain,channels,translation):
    p=[round(point[i]+translation[i])for i in range(3)];pivots=[]
    for j in chain:
        pivot=[round(j['origin'][i]+translation[i])for i in range(3)]
        for q,angles in pivots:pivot=rotate(pivot,angles,q)
        angles=list(map(round,channels.get(j['name'],[0,0,0])));p=rotate(p,angles,pivot);pivots.append((pivot,angles))
    return p

def matmul(a,b):return [[sum(a[i][k]*b[k][j]for k in range(3))for j in range(3)]for i in range(3)]
def matrix(angles):
    x,y,z=[v*math.pi/128 for v in angles];cx,sx,cy,sy,cz,sz=math.cos(x),math.sin(x),math.cos(y),math.sin(y),math.cos(z),math.sin(z)
    return matmul([[cy,0,sy],[0,1,0],[-sy,0,cy]],matmul([[1,0,0],[0,cx,-sx],[0,sx,cx]],[[cz,sz,0],[-sz,cz,0],[0,0,1]]))
def inverse_angles(chain,channels):
    p=[[1.,0.,0.],[0.,1.,0.],[0.,0.,1.]]
    for j in chain:p=matmul(matrix(channels.get(j['name'],[0,0,0])),p)
    return angles_from_matrix([[p[j][i]for j in range(3)]for i in range(3)])

def angles_from_matrix(a):
    x=math.asin(max(-1,min(1,-a[1][2])));y=math.atan2(a[0][2],a[2][2]);z=math.atan2(-a[1][0],a[1][1]);return [round(v*128/math.pi)for v in [x,y,z]]

def normalize(v):
    length=math.sqrt(sum(x*x for x in v));return [x/max(length,1e-9)for x in v]

def cross(a,b):return [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]

def align_rotation(a,b):
    a,b=normalize(a),normalize(b);v=cross(a,b);c=max(-1,min(1,sum(x*y for x,y in zip(a,b))));s=math.sqrt(sum(x*x for x in v))
    if s<1e-8:
        if c>0:return [[1.,0.,0.],[0.,1.,0.],[0.,0.,1.]]
        u=normalize(cross(a,[1,0,0]if abs(a[0])<.8 else[0,1,0]));return [[2*u[i]*u[j]-(1 if i==j else 0)for j in range(3)]for i in range(3)]
    k=[[0,-v[2],v[1]],[v[2],0,-v[0]],[-v[1],v[0],0]];kk=matmul(k,k);return [[(1 if i==j else 0)+k[i][j]+kk[i][j]*(1-c)/(s*s)for j in range(3)]for i in range(3)]

def solve_limb(rig,channels,limb,target,translation):
    name=limb['name'];hip=name+'_hip';knee=name+'_knee';chain=ancestors(rig,knee);controls=[(joint,axis)for joint in [hip,knee]for axis in range(3)];result=copy.deepcopy(channels)
    for joint,axis in controls:result.setdefault(joint,[0,0,0])
    # Analytic two-bone knee placement uses the authored bend preference; a
    # small native-integer refinement follows instead of guessing from zero.
    prefix=ancestors(rig,hip)[:-1];h=forward(limb['hip'],prefix,result,translation);k=forward(limb['knee'],prefix,result,translation);tip=forward(limb['tip'],prefix,result,translation)
    a,b=math.dist(h,k),math.dist(k,tip);vector=[target[i]-h[i]for i in range(3)];distance=math.sqrt(sum(x*x for x in vector));direction=normalize(vector);d=max(abs(a-b)+1e-4,min(a+b-1e-4,distance));along=(a*a-b*b+d*d)/(2*d);height=math.sqrt(max(0,a*a-along*along));bend=[k[i]-h[i]for i in range(3)];dot=sum(bend[i]*direction[i]for i in range(3));bend=normalize([bend[i]-dot*direction[i]for i in range(3)])
    if sum(x*x for x in bend)<.5:bend=normalize(cross(direction,[0,0,1]if abs(direction[2])<.8 else[1,0,0]))
    desired=[h[i]+direction[i]*along+bend[i]*height for i in range(3)];result[hip]=angles_from_matrix(align_rotation([k[i]-h[i]for i in range(3)],[desired[i]-h[i]for i in range(3)]));actual_k=forward(limb['knee'],ancestors(rig,hip),result,translation);actual_tip=forward(limb['tip'],ancestors(rig,hip),result,translation);result[knee]=angles_from_matrix(align_rotation([actual_tip[i]-actual_k[i]for i in range(3)],[target[i]-actual_k[i]for i in range(3)]))
    def cost():
        p=forward(limb['tip'],chain,result,translation)
        return sum((p[i]-target[i])**2 for i in range(3))+.00001*sum(result[j][a]**2 for j,a in controls)
    for step in [3,2,1]:
        for repeat in range(3):
            changed=False
            for joint,axis in controls:
                best=cost();current=result[joint][axis];chosen=current
                for v in [max(-120,current-step),min(120,current+step)]:
                    result[joint][axis]=v;value=cost()
                    if value<best:best=value;chosen=v
                result[joint][axis]=chosen;changed|=chosen!=current
            if not changed:break
    ankle=name+'_ankle'
    if any(j['name']==ankle for j in rig):result[ankle]=inverse_angles(chain,result)
    return result,math.dist(forward(limb['tip'],chain,result,translation),target)

def gait_controls(g,rig,channels,translation,spec,tick):
    gait=spec['gait'];duration=spec['duration_ticks'];translation=list(translation);translation[1]+=gait.get('crouch',3);channels=copy.deepcopy(channels);targets=[];active=[]
    for limb in gait['limbs']:
        phase=(tick/duration+limb.get('phase_offset',0))%1;stance=gait.get('stance_fraction',.6);stride=gait['stride'];target=list(limb['tip'])
        if phase<stance:target[2]+=stride*(phase/stance-.5);lift=0;active+=limb['components']
        else:
            u=(phase-stance)/(1-stance);target[2]+=stride*(.5-(u*u*(3-2*u)));lift=gait['lift']*math.sin(math.pi*u)
        target[1]-=lift;target=list(map(round,target));channels,error=solve_limb(rig,channels,limb,target,translation);targets.append(dict(limb=limb['name'],target=target,endpoint_error_native=error,stance=phase<stance))
    frame,posed,origins=compile_joints(g,rig,channels,translation)
    # Optional support-surface correction measures the actual authored pad mesh,
    # not just the abstract ankle point. It is opt-in to preserve earlier recipes.
    for _ in range(gait.get('support_surface_iterations',0)):
        for limb,record in zip(gait['limbs'],targets):
            if not record['stance']:continue
            ids=[v for name in limb['components']for v in g['components'][name]['vertices']];height=max(posed[v][1]for v in ids)
            if abs(height)<=1:continue
            record['target'][1]-=height;channels,error=solve_limb(rig,channels,limb,record['target'],translation);record['endpoint_error_native']=error
            frame,posed,origins=compile_joints(g,rig,channels,translation)
    if active:
        ids=[v for name in active for v in g['components'][name]['vertices']];translation[1]-=max(posed[v][1]for v in ids)
    return channels,translation,targets
