"""Native-pose contact, support, action and deformation diagnostics.

All thresholds are stated native-unit review heuristics. Support uses a geometric
body proxy, not inferred mass or a physics simulation. AABB clipping is potential
intersection only. Original animation tick sampling remains unchanged.
"""
import json,math,argparse,csv,statistics
from pathlib import Path
from PIL import Image,ImageChops,ImageDraw
from analyze import foreground
from kinematics import frame_at

def load(p):return json.loads(p.read_text())
def center(vs,ids):return [sum(vs[i][a]for i in ids)/len(ids)for a in range(3)]
def hull(points):
    pts=sorted(set(map(tuple,points)))
    def cross(o,a,b):return(a[0]-o[0])*(b[1]-o[1])-(a[1]-o[1])*(b[0]-o[0])
    lo=[];hi=[]
    for p in pts:
        while len(lo)>1 and cross(lo[-2],lo[-1],p)<=0:lo.pop()
        lo.append(p)
    for p in reversed(pts):
        while len(hi)>1 and cross(hi[-2],hi[-1],p)<=0:hi.pop()
        hi.append(p)
    return lo[:-1]+hi[:-1]
def inside(point,poly):
    if len(poly)<3:return False
    signs=[]
    for a,b in zip(poly,poly[1:]+poly[:1]):signs.append((b[0]-a[0])*(point[1]-a[1])-(b[1]-a[1])*(point[0]-a[0]))
    return min(signs)>=-1e-5 or max(signs)<=1e-5

def diagnose(root,out,contact_override=None):
    out.mkdir(parents=True,exist_ok=True);g=load(root/'geometry.json');r=load(root/'recipe.json');summary={};contacts=contact_override or r.get('locomotion');ground=0;tolerance=3;all_motion={}
    for action in r.get('actions',{}):
        folder=root/'motion'/action
        if not folder.exists():continue
        spec=r['actions'][action];files=sorted(folder.glob('pose-*.json'));rows=[(int(p.stem.split('-')[1]),load(p)['vertices'])for p in files];all_motion[action]=rows;first=rows[0][1];last=rows[-1][1];pose_rms=[math.sqrt(sum(math.dist(v,w)**2 for v,w in zip(first,vs))/len(vs))for _,vs in rows];peak=max(range(len(rows)),key=pose_rms.__getitem__);a=dict(native_samples=len(rows),largest_pose_displacement_tick=rows[peak][0],largest_pose_rms_native=pose_rms[peak],end_to_start_rms_native=math.sqrt(sum(math.dist(v,w)**2 for v,w in zip(first,last))/len(last)),end_bounds=[[min(v[i]for v in last)for i in range(3)],[max(v[i]for v in last)for i in range(3)]],declared_events=spec.get('events',{}))
        contact_rows=[];window_anchors={};slips=[];penetrations=[];unsupported=0;outside_support=0;support_checked=0;foot_floating=0
        if contacts and action=='walk':
            duration=spec['duration_ticks'];stance=contacts['stance_fraction'];speed=contacts['stride']/(stance*duration);body_names=[n for n in ['abdomen','thorax','torso']if n in g['components']];body_ids=sorted({v for n in body_names for v in g['components'][n]['vertices']});soles={}
            for limb in contacts['limbs']:
                ids=[v for n in limb['components']for v in g['components'][n]['vertices']];y=max(g['vertices'][i][1]for i in ids);soles[limb['name']]=[i for i in ids if g['vertices'][i][1]>=y-1]
            animation=load(root/action/'animation.json');total=sum(f['length']for f in animation['frames'])
            for tick,vs in rows:
                sample=tick%total if animation['policy'].get('loop_frames',-1)>0 else tick;index,_=frame_at(animation['frames'],sample);effective_tick=sum(f['length']for f in animation['frames'][:index]);cycle=tick//total if animation['policy'].get('loop_frames',-1)>0 else 0
                supports=[];active_heights=[]
                for limb in contacts['limbs']:
                    raw=cycle+effective_tick/duration+limb.get('phase_offset',0);phase=raw%1;active=phase<stance;ids=soles[limb['name']];point=center(vs,ids);point[2]-=speed*tick;lowest=max(vs[i][1]for i in ids);penetrations.append(max(0,lowest));slip=0
                    if active:
                        key=(limb['name'],math.floor(raw));window_anchors.setdefault(key,point);anchor=window_anchors[key];slip=math.dist([point[0],point[2]],[anchor[0],anchor[2]]);slips.append(slip);active_heights.append(lowest)
                        if lowest<-tolerance:foot_floating+=1
                        elif contacts.get('support_polygon_mode')=='sole_footprints':supports += [[vs[i][0],vs[i][2]-speed*tick]for i in ids if vs[i][1]>=lowest-tolerance]
                        else:supports.append([point[0],point[2]])
                    contact_rows.append(dict(tick=tick,limb=limb['name'],stance=active,world_x=point[0],world_y=point[1],world_z=point[2],lowest_surface_y=lowest,stance_drift_native=slip))
                if active_heights and max(active_heights)<-tolerance:unsupported+=1
                if body_ids:
                    bc=center(vs,body_ids);bc[2]-=speed*tick;support_checked+=1
                    if not inside([bc[0],bc[2]],hull(supports)):outside_support+=1
            a['contact']=dict(phase_source='Actual exported native frame and clock boundary rules',support_polygon_mode=contacts.get('support_polygon_mode','legacy_foot_centroids'),assumed_forward_travel_native_per_tick=-speed,stance_samples=len(slips),mean_stance_drift_native=sum(slips)/len(slips)if slips else None,max_stance_drift_native=max(slips,default=0),max_penetration_native=max(penetrations,default=0),floating_stance_foot_samples=foot_floating,fully_unsupported_frames=unsupported,body_projection_outside_support_frames=outside_support,support_frames_checked=support_checked,quantization_review_tolerance_native=tolerance)
            with (out/(action+'-contacts.csv')).open('w',newline='')as f:w=csv.DictWriter(f,fieldnames=list(contact_rows[0]));w.writeheader();w.writerows(contact_rows)
        # Rigid component edge-length changes expose collapse/shearing without
        # pretending a bounding-box overlap proves triangle intersection.
        errors=[]
        for component in g['components'].values():
            edges={tuple(sorted([f[i],f[(i+1)%3]]))for fi in component['faces']for f in [g['faces'][fi]]for i in range(3)}
            for tick,vs in rows[::max(1,len(rows)//24)]:
                errors += [abs(math.dist(vs[i],vs[j])-math.dist(g['vertices'][i],g['vertices'][j]))for i,j in edges]
        a['rigid_component_edge_error_native']=dict(max=max(errors,default=0),mean=sum(errors)/len(errors)if errors else 0,interpretation='Integer rotation truncation is expected; this measures edge preservation, not biological plausibility')
        if action=='attack':
            component='head'if'head'in g['components']else next(iter(g['components']));ids=g['components'][component]['vertices'];start=center(first,ids);traj=[center(vs,ids)for _,vs in rows];a['attack_facing']=dict(expected_forward_axis='-Z',maximum_forward_excursion_native=max(start[2]-v[2]for v in traj),end_head_recovery_error_native=math.dist(start,traj[-1]),marker=component+' centroid proxy; no impact inference')
        if action=='death':
            a['terminal_state']=dict(persistent_corpse=not spec.get('terminal_zero',False),native_hold_ticks=spec.get('terminal_pose_hold',0),end_ground_penetration_native=max(0,max(v[1]for v in last)),noncollapsed_final_pose=max(math.dist(v,last[0])for v in last)>2)
        ticks=sorted({rows[0][0],rows[-1][0],rows[peak][0],*[rows[round(i*(len(rows)-1)/6)][0]for i in range(7)]});sheet=Image.new('RGB',(280*len(ticks),284),(16,21,26));draw=ImageDraw.Draw(sheet)
        for col,tick in enumerate(ticks):im=Image.open(folder/f'pose-{tick:04}.bmp');im.thumbnail((280,260));sheet.paste(im,(col*280,24));draw.text((col*280+8,6),f'{action} | native tick {tick}',fill='white')
        sheet.save(out/(action+'-contact-sheet.png'));summary[action]=a
    transitions=[]
    for before,after in [('idle','walk'),('walk','attack'),('attack','idle'),('hit','idle')]:
        if before in all_motion and after in all_motion:
            a=all_motion[before][-1][1];b=all_motion[after][0][1];transitions.append(dict(before=before,after=after,raw_vertex_rms_native=math.sqrt(sum(math.dist(v,w)**2 for v,w in zip(a,b))/len(a))))
    result=dict(actions=summary,transitions=transitions,limitations='Contact and support are explicit geometric proxies. No inferred mass, physical collision, automatic artistic approval or time warp. World travel is stated, not fitted to minimize slip.',artistic_acceptance=False);(out/'motion-diagnostics.json').write_text(json.dumps(result,indent=2));return result
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('root',type=Path);p.add_argument('out',type=Path);p.add_argument('--contacts',type=Path);a=p.parse_args();diagnose(a.root,a.out,load(a.contacts)if a.contacts else None);print('NATIVE_CONTACT_ACTION_DIAGNOSTICS_COMPLETE')
