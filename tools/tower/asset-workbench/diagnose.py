"""Component-linked diagnostics using explicit curator/author selectors."""
import argparse,json,math
from pathlib import Path
from collections import Counter,defaultdict
from motion import selectors
from analyze import bound
def principal_axes(points):
    center=[sum(v[i]for v in points)/len(points)for i in range(3)];a=[[sum((v[i]-center[i])*(v[j]-center[j])for v in points)/len(points)for j in range(3)]for i in range(3)];vectors=[[float(i==j)for j in range(3)]for i in range(3)]
    for _ in range(24):
        p,q=max([(0,1),(0,2),(1,2)],key=lambda ij:abs(a[ij[0]][ij[1]]))
        if abs(a[p][q])<1e-8:break
        angle=.5*math.atan2(2*a[p][q],a[q][q]-a[p][p]);c,s=math.cos(angle),math.sin(angle)
        for k in range(3):x,y=a[k][p],a[k][q];a[k][p]=c*x-s*y;a[k][q]=s*x+c*y
        for k in range(3):x,y=a[p][k],a[q][k];a[p][k]=c*x-s*y;a[q][k]=s*x+c*y
        for k in range(3):x,y=vectors[k][p],vectors[k][q];vectors[k][p]=c*x-s*y;vectors[k][q]=s*x+c*y
    return sorted([dict(variance=max(0,a[i][i]),axis=[vectors[k][i]for k in range(3)])for i in range(3)],key=lambda v:v['variance'])
def describe(g,indices):
    if not indices:return None
    points=[g['vertices'][i]for i in indices];b=bound(points);ext=[b[1][i]-b[0][i]for i in range(3)];members=set(indices);faces=[i for i,f in enumerate(g['faces'])if sum(v in members for v in f)>=2];hist=Counter();bands=[Counter()for _ in range(4)]
    for i in faces:
        if g.get('face_textures')is not None and g['face_textures'][i]>=0:continue
        c=g['colors'][i];hist[c]+=1;y=sum(g['vertices'][v][1]for v in g['faces'][i])/3;band=min(3,max(0,int((y-b[0][1])/max(1,ext[1])*4)));bands[band][c]+=1
    return dict(bounds=b,extents=ext,centroid=[sum(v[i]for v in points)/len(points)for i in range(3)],vertices=len(indices),palette=dict(hist),height_band_palettes=[dict(h)for h in bands],principal_axes=principal_axes(points),selector_note='Bounds, PCA axes and centroids of explicitly selected vertices; not automatically inferred anatomical joints')
def diagnose(ref,candidate,bindings):
    result={}
    for name,record in bindings.items():
        a=describe(ref,selectors(ref,record['reference']));b=describe(candidate,selectors(candidate,record['candidate']))
        if not a or not b:result[name]=dict(status='UNRESOLVED selector empty');continue
        messages=[]
        normal_a=a['principal_axes'][0]['axis'];normal_b=b['principal_axes'][0]['axis'];angle=math.degrees(math.acos(min(1,abs(sum(x*y for x,y in zip(normal_a,normal_b))))))
        if angle>15 and a['principal_axes'][0]['variance']<a['principal_axes'][1]['variance']*.3:messages.append(dict(kind='equipment orientation',component=name,parameter=record.get('parameter',name)+'.socket_basis',plane_normal_angle_degrees=angle,advice='Inspect the socket plane before changing width/depth; a rotated thin component can have very different axis-aligned extents. PCA axes are geometric estimates.'))
        for i,axis in enumerate(['width','height','depth']):
            if a['extents'][i]:
                ratio=b['extents'][i]/a['extents'][i]
                if abs(ratio-1)>.08:messages.append(dict(kind='geometry',component=name,parameter=record.get('parameter',name)+'.'+axis,direction='too large'if ratio>1 else 'too small',relative_difference=ratio-1,advice='Inspect component selector coverage, then adjust this component; retain shared camera and overall world scale.'))
        if len(b['palette'])<len(a['palette'])/2:messages.append(dict(kind='material placement',component=name,parameter=record.get('parameter',name)+'.material_regions',advice='Candidate omits reference palette variation; inspect height-band placement and native normals rather than changing global lighting.'))
        drift=[b['centroid'][i]-a['centroid'][i]for i in range(3)]
        if max(map(abs,drift))>8:messages.append(dict(kind='attachment/origin',component=name,parameter=record.get('parameter',name)+'.origin',delta_native=drift,advice='Inspect socket/joint origin and component coverage; centroid is a geometric proxy, not a joint.'))
        result[name]=dict(reference=a,candidate=b,annotation_evidence=record.get('evidence','Explicit supplied selectors; semantic confidence not provided'),messages=messages,components=record['candidate'].get('components',[]))
    return result
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('reference',type=Path);p.add_argument('candidate',type=Path);p.add_argument('bindings',type=Path);p.add_argument('out',type=Path);a=p.parse_args();r=diagnose(json.loads(a.reference.read_text()),json.loads(a.candidate.read_text()),json.loads(a.bindings.read_text()));a.out.write_text(json.dumps(r,indent=2));print('COMPONENT_DIAGNOSTICS_COMPLETE',len(r))
