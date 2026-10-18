"""Private observational geometry audit, separate from the authoring system.

Reference measurements never become authored vertices. Optional semantic regions
are coarse author-supplied observation masks, not anatomical ground truth.
"""
import json,math
from pathlib import Path
import numpy as np

def points(g):return np.asarray([[x,z,-y]for x,y,z in g['vertices']],dtype=float)
def hull2(points):
    rows=sorted(set(map(tuple,points)))
    if len(rows)<3:return rows
    def cross(a,b,c):return (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
    lo=[];hi=[]
    for p in rows:
        while len(lo)>=2 and cross(lo[-2],lo[-1],p)<=0:lo.pop()
        lo.append(p)
    for p in reversed(rows):
        while len(hi)>=2 and cross(hi[-2],hi[-1],p)<=0:hi.pop()
        hi.append(p)
    return lo[:-1]+hi[:-1]
def area(poly):return abs(sum(a[0]*b[1]-a[1]*b[0]for a,b in zip(poly,poly[1:]+poly[:1])))/2 if len(poly)>2 else 0
def sections(g,depths,z_min=0):
    vs=points(g);triangles=vs[np.asarray(g['faces'])];result=[]
    for depth in depths:
        intersects=[]
        for tri in triangles:
            if tri[:,1].min()>depth or tri[:,1].max()<depth:continue
            for a,b in zip(tri,np.roll(tri,-1,axis=0)):
                if (a[1]<=depth<b[1])or(b[1]<=depth<a[1]):
                    q=a+(b-a)*((depth-a[1])/(b[1]-a[1]))
                    if q[2]>=z_min:intersects.append((float(q[0]),float(q[2])))
        result.append({'depth':depth,'convex_section_area':area(hull2(intersects))})
    return result
def masked_bounds(g,regions):
    vs=points(g);result={}
    for name,limits in regions.items():
        selected=vs[np.all((vs>=np.array(limits)[:,0])&(vs<=np.array(limits)[:,1]),axis=1)]
        if len(selected):result[name]={'minimum':selected.min(0).tolist(),'maximum':selected.max(0).tolist(),'dimensions':np.ptp(selected,axis=0).tolist(),'centroid':selected.mean(0).tolist(),'vertices':len(selected)}
    return result
def topology(g):
    vs=points(g);fs=np.asarray(g['faces']);edges={};degenerate=0
    for a,b,c in fs:
        degenerate+=int(np.linalg.norm(np.cross(vs[b]-vs[a],vs[c]-vs[a]))<1e-8)
        for i,j in [(a,b),(b,c),(c,a)]:key=tuple(sorted((int(i),int(j))));edges[key]=edges.get(key,0)+1
    # Multiple deliberate shells, hard seams and quantized vertex duplication
    # are reported; a nonzero boundary count is not automatically a defect.
    return {'vertices':len(vs),'triangles':len(fs),'degenerate_triangles':degenerate,'boundary_edges':sum(n==1 for n in edges.values()),'nonmanifold_index_edges':sum(n>2 for n in edges.values()),'unique_positions':len(set(map(tuple,vs))),'bounds':[vs.min(0).tolist(),vs.max(0).tolist()]}
def compare(reference,candidate,spec):
    rg=json.loads(Path(reference).read_text());cg=json.loads(Path(candidate).read_text());depths=spec.get('depths',[]);low=spec.get('section_floor',0)
    rs,cs=sections(rg,depths,low),sections(cg,depths,low)
    volume=lambda rows:sum((b['depth']-a['depth'])*(a['convex_section_area']+b['convex_section_area'])/2 for a,b in zip(rows,rows[1:]))
    rb,cb=masked_bounds(rg,spec.get('regions',{})),masked_bounds(cg,spec.get('regions',{}))
    return {'reference_topology':topology(rg),'candidate_topology':topology(cg),'reference_sections':rs,'candidate_sections':cs,'envelope_volume_ratio':volume(cs)/volume(rs)if volume(rs)>0 else None,'regions':{name:{'reference':rb[name],'candidate':cb.get(name),'dimension_ratio':[b/max(1,a)for a,b in zip(rb[name]['dimensions'],cb[name]['dimensions'])]if name in cb else None,'centroid_distance':float(np.linalg.norm(np.array(rb[name]['centroid'])-cb[name]['centroid']))if name in cb else None}for name in rb},'interpretation':'Coarse convex cross-section envelope and region bounds; overlap, cavities and anatomical labels are approximations. Independent image and artistic review remain required.'}

if __name__=='__main__':
    import sys
    reference,candidate,spec,out=sys.argv[1:];Path(out).write_text(json.dumps(compare(reference,candidate,json.loads(Path(spec).read_text())),indent=2))
