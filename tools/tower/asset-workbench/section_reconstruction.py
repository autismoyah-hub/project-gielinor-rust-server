"""Private descriptor reconstruction on an independently authored ring topology.

Curated regions become coarse landmark sections. No source vertex, face or
animation array is copied into candidate topology. The resulting reference
derivative and recipe must remain private; original designs use authored inputs.
"""
import math,json,argparse
from pathlib import Path
from component_review import selected
from templates import part
from construction import asset

def observe(g,selector,axis=1,bins=6):
    if not 3<=bins<=12:raise ValueError('Use 3..12 coarse semantic sections')
    faces=selected(g,selector);ids=sorted({i for f in faces for i in g['faces'][f]});points=[g['vertices'][i]for i in ids]
    if not points:raise ValueError('Empty curated region')
    lo=min(v[axis]for v in points);hi=max(v[axis]for v in points)
    if hi-lo<2:raise ValueError('Region has insufficient longitudinal extent')
    other=[i for i in range(3)if i!=axis];rows=[]
    for i in range(bins):
        t=lo+(hi-lo)*i/(bins-1);radius=(hi-lo)/(bins-1)*.6;near=[v for v in points if abs(v[axis]-t)<=radius]
        if not near:
            delta=min(abs(v[axis]-t)for v in points);near=[v for v in points if abs(v[axis]-t)<=delta+1]
        bounds=[[min(v[a]for v in near)for a in other],[max(v[a]for v in near)for a in other]];center=[(bounds[0][a]+bounds[1][a])*.5 for a in range(2)];radii=[max(1,(bounds[1][a]-bounds[0][a])*.5)for a in range(2)];rows.append(dict(longitudinal=t,center=center,radii=radii))
    palette={}
    for f in faces:
        if(g.get('face_textures')or[-1]*len(g['faces']))[f]<0:palette[g['colors'][f]]=palette.get(g['colors'][f],0)+1
    return dict(axis=axis,transverse_axes=other,sections=rows,dominant_native_hsl=max(palette,key=palette.get)if palette else 10282,source_region_faces=len(faces),topology_reused=False)

def component(name,descriptor,material='organic',group=0,sides=8):
    axis=descriptor['axis'];u,v=descriptor['transverse_axes'];basis=[[int(i==u)for i in range(3)],[int(i==axis)for i in range(3)],[int(i==v)for i in range(3)]]
    det=basis[0][0]*(basis[1][1]*basis[2][2]-basis[1][2]*basis[2][1])-basis[0][1]*(basis[1][0]*basis[2][2]-basis[1][2]*basis[2][0])+basis[0][2]*(basis[1][0]*basis[2][1]-basis[1][1]*basis[2][0]);sign=1 if det>0 else-1;basis[0]=[x*sign for x in basis[0]]
    rows=[[r['center'][0]*sign,r['longitudinal'],r['center'][1],r['radii'][0],r['radii'][1]]for r in descriptor['sections']]
    return part(name,'section_mesh',material,group,role=name,sections=rows,sides=sides,parent_socket=dict(basis=basis),parameters=dict(construction='independent consistent-ring topology from curated coarse landmarks',source_topology_reused=False))

def reconstruct(g,regions,name='Private semantic section study'):
    c=[];descriptors={};materials={}
    for region in regions:
        key=region['name'];d=observe(g,region['selector'],region.get('axis',1),region.get('bins',6));descriptors[key]=d;p=component(key,d,key,region.get('group',0),region.get('sides',8));c.append(p);materials[key]=dict(hsl=d['dominant_native_hsl'],render_type=0,alpha=region.get('alpha',0),priority=region.get('priority',0))
        if region.get('source_texture_id',-1)>=0:
            materials[key]['native_texture_id']=region['source_texture_id'];axis=d['axis'];u=max(d['transverse_axes'],key=lambda a:max(v[a]for v in g['vertices'])-min(v[a]for v in g['vertices']));p['native_uv_projection']=dict(axes=[u,axis])
    r=asset(name,c);r['materials'].update(materials);r['geometry_validation']=dict(remove_zero_area_faces=True,preserve_vertex_order=True);r['provenance']=dict(kind='private_reference_reconstruction',method='Curated coarse cross-section observations on independently authored ring topology; not a cache round trip',source_topology_reused=False,protected_derivative=True,artistic_acceptance=False);return r,descriptors

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('geometry',type=Path);p.add_argument('regions',type=Path);p.add_argument('out',type=Path);a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True);r,d=reconstruct(json.loads(a.geometry.read_text()),json.loads(a.regions.read_text()));(a.out/'recipe.json').write_text(json.dumps(r,indent=2));(a.out/'landmark-observations.json').write_text(json.dumps(d,indent=2));print('PRIVATE_SECTION_RECONSTRUCTION_RECIPE_COMPLETE')
