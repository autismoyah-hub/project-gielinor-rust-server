"""Temporal native-pose QA with explicit landmarks, limits and collision proxies.

Capsules are declared conservative review volumes, not triangle intersections
or a replacement for native animation. No timestamp fitting is performed.
"""
import argparse,json,math
from pathlib import Path
from kinematics import frame_at
from rig_contracts import validate

def dot(a,b):return sum(x*y for x,y in zip(a,b))
def sub(a,b):return[x-y for x,y in zip(a,b)]
def segment_distance(p1,q1,p2,q2):
    d1=sub(q1,p1);d2=sub(q2,p2);r=sub(p1,p2);a=dot(d1,d1);e=dot(d2,d2);f=dot(d2,r);clamp=lambda x:max(0,min(1,x))
    if a<1e-9 and e<1e-9:return math.dist(p1,p2)
    if a<1e-9:s=0;t=clamp(f/e)
    else:
        c=dot(d1,r)
        if e<1e-9:t=0;s=clamp(-c/a)
        else:
            b=dot(d1,d2);den=a*e-b*b;s=clamp((b*f-c*e)/den)if abs(den)>1e-9 else 0;t=(b*s+f)/e
            if t<0:t=0;s=clamp(-c/a)
            elif t>1:t=1;s=clamp((b-c)/a)
    return math.dist([p1[i]+d1[i]*s for i in range(3)],[p2[i]+d2[i]*t for i in range(3)])

def review(root,manifest,out):
    out.mkdir(parents=True,exist_ok=True);load=lambda p:json.loads(p.read_text());g=load(root/'geometry.json');recipe=load(root/'recipe.json');landmarks={}
    for name,spec in manifest['landmarks'].items():
        ids=spec.get('vertices',sorted({v for c in spec.get('components',[])for v in g['components'][c]['vertices']}))
        if not ids:raise ValueError('Empty explicit motion landmark '+name)
        landmarks[name]=ids
    def point(vs,name):return[sum(vs[i][a]for i in landmarks[name])/len(landmarks[name])for a in range(3)]
    results={}
    for folder in sorted((root/'motion').iterdir()):
        if not folder.is_dir():continue
        poses=[(int(p.stem.split('-')[1]),load(p)['vertices'])for p in sorted(folder.glob('pose-*.json'))]
        if not poses:continue
        animation=load(root/folder.name/'animation.json');duration=sum(f['length']for f in animation['frames']);trajectories={name:[]for name in landmarks};collisions=[];limits=[];rms=[];previous=None
        for tick,vs in poses:
            p={name:point(vs,name)for name in landmarks}
            for name,value in p.items():trajectories[name].append(dict(tick=tick,position=value))
            for pair in manifest.get('collision_pairs',[]):
                a,b=pair['a'],pair['b'];distance=segment_distance(p[a[0]],p[a[1]],p[b[0]],p[b[1]]);overlap=pair['radius_a']+pair['radius_b']-distance
                if overlap>pair.get('tolerance',2):collisions.append(dict(tick=tick,pair=pair['name'],proxy_overlap_native=overlap,finding='Declared capsule review volumes overlap; inspect native images before declaring a mesh collision.'))
            sample=tick%duration if animation['policy'].get('loop_frames',-1)>0 else tick;_,frame=frame_at(animation['frames'],sample);values=dict(frame['transforms']);channels={j['name']:values[3+2*i]for i,j in enumerate(recipe.get('rig',[]))};limits+= [dict(tick=tick,**v)for v in validate(recipe,channels)['limit_violations']]
            if previous is not None:rms.append(dict(tick=tick,vertex_step_rms_native=math.sqrt(sum(math.dist(v,w)**2 for v,w in zip(vs,previous))/len(vs))))
            previous=vs
        motions={}
        for name,rows in trajectories.items():
            start=rows[0]['position'];distances=[math.dist(start,p['position'])for p in rows];peak=max(range(len(rows)),key=distances.__getitem__);arc=sum(math.dist(a['position'],b['position'])for a,b in zip(rows,rows[1:]));motions[name]=dict(peak_tick=rows[peak]['tick'],maximum_displacement_native=distances[peak],arc_length_native=arc,recovery_error_native=distances[-1],trajectory=rows)
        results[folder.name]=dict(native_samples=len(poses),duration_ticks=duration,frame_cadence=[r['length']for r in animation['frames']],landmarks=motions,static_steps=[r['tick']for r in rms if r['vertex_step_rms_native']==0],step_rms=rms,joint_limit_violations=limits,collision_proxy_observations=collisions,events=recipe['actions'][folder.name].get('events',{}),terminal_hold_is_expected=bool(recipe['actions'][folder.name].get('terminal_pose_hold')),artistic_acceptance=False)
    report=dict(actions=results,method='Actual native vertices at original timestamps; explicit landmark/attachment/capsule declarations; no time warp, mass inference or triangle collision claim',artistic_acceptance=False);(out/'advanced-motion.json').write_text(json.dumps(report,indent=2));return report

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('root',type=Path);p.add_argument('manifest',type=Path);p.add_argument('out',type=Path);a=p.parse_args();review(a.root,json.loads(a.manifest.read_text()),a.out);print('ADVANCED_NATIVE_MOTION_REVIEW_COMPLETE')
