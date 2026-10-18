"""Validate transform tracing and measure annotated native landmark trajectories."""
import json,argparse,math,csv,re,struct
from pathlib import Path
from kinematics import trace,frame_at
def selectors(g,spec):
    if 'indices'in spec:return spec['indices']
    if 'skin_labels'in spec:return [i for i,n in enumerate(g.get('vertex_skins')or [])if n in spec['skin_labels']]
    if 'components'in spec:return sorted({i for name in spec['components']for i in g['components'][name]['vertices']})
    raise ValueError('Explicit landmark selector required')
def centroid(vs,indices):return [sum(vs[i][a]for i in indices)/len(indices)for a in range(3)]if indices else None
def validate(root,out):
    rows=[]
    for action in sorted(root.glob('*/*/animation.json')):
        g=json.loads((action.parent.parent/'geometry.json').read_text());data=json.loads(action.read_text())
        if data.get('cached_animation')is not None:rows.append(dict(action=str(action.parent.relative_to(root)),status='SKIP cached weighted path; no classic tracer claim'));continue
        checked=0;worst=0
        for p in action.parent.glob('pose-*.json'):
            tick=int(p.stem.split('-')[1]);policy=(action.parent/'sequence.txt').read_text();loop=int(re.search(r'loop_frames: (-?\d+)',policy).group(1));duration=sum(r['length']for r in data['frames']);sample=tick%duration if loop>0 and duration else tick;_,frame=frame_at(data['frames'],sample);actual=json.loads(p.read_text())['vertices'];predicted,_=trace(g,frame);scale=json.loads((action.parent.parent/'metadata.json').read_text())['render_scale'];predicted=[[v[a]*scale[a]/128 for a in range(3)]for v in predicted]
            f32=lambda x:struct.unpack('<f',struct.pack('<f',x))[0]
            error=max(abs(f32(x)-f32(y))for v,w in zip(predicted,actual)for x,y in zip(v,w));worst=max(worst,error);checked+=1
        rows.append(dict(action=str(action.parent.relative_to(root)),native_samples=checked,max_coordinate_error_native=worst,status='PASS'if worst==0 else 'FAIL'))
    out.write_text(json.dumps(rows,indent=2));print('CLASSIC_TRACER_NATIVE_VALIDATION',len(rows),'failures',sum(r['status']=='FAIL'for r in rows));return rows
def landmarks(g,poses,annotation):
    ids={name:selectors(g,spec)for name,spec in annotation.items()};result=[]
    for tick,vertices in poses:
        result.append(dict(tick=tick,landmarks={name:centroid(vertices,ix)for name,ix in ids.items()}))
    return result
def compare_motion(a,b,out):
    """Input: explicit landmarks per timestamp. No automatic joint inference."""
    aa={r['tick']:r['landmarks']for r in a};bb={r['tick']:r['landmarks']for r in b};result={}
    for name in sorted(set(a[0]['landmarks'])&set(b[0]['landmarks'])):
        common=[t for t in sorted(set(aa)&set(bb))if aa[t][name]is not None and bb[t][name]is not None]
        if not common:continue
        origin_a,origin_b=aa[common[0]][name],bb[common[0]][name];samples=[]
        for tick in common:
            va,vb=aa[tick][name],bb[tick][name];absolute=math.dist(va,vb);arc=math.dist([va[i]-origin_a[i]for i in range(3)],[vb[i]-origin_b[i]for i in range(3)]);samples.append(dict(tick=tick,absolute_displacement=absolute,bind_offset_removed_arc_error=arc,reference=va,candidate=vb))
        result[name]=dict(samples=samples,max_arc_error=max(x['bind_offset_removed_arc_error']for x in samples),mean_arc_error=sum(x['bind_offset_removed_arc_error']for x in samples)/len(samples),reference_extents=[max(aa[t][name][i]for t in common)-min(aa[t][name][i]for t in common)for i in range(3)],candidate_extents=[max(bb[t][name][i]for t in common)-min(bb[t][name][i]for t in common)for i in range(3)])
    out.write_text(json.dumps(result,indent=2));return result
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('reference_root',type=Path);p.add_argument('output',type=Path);a=p.parse_args();validate(a.reference_root,a.output)
