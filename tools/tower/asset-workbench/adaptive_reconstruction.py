"""Private static descriptor refinement, explicitly developed after the exam.

Bone labels partition observed geometry only. Source weighted deformation data
is not transferred and no independent weighted-motion support is claimed.
"""
import copy,math
from collections import defaultdict,Counter
from reconstruction_regions import propose
from section_reconstruction import reconstruct
def recipe(g,name):
    observed=copy.deepcopy(g);weighted=bool(g.get('bone_weights'))
    if weighted:observed['vertex_skins']=[max(row,key=lambda pair:pair[1])[0]if row else 0 for row in g['bone_weights']]
    regions=propose(observed);r,descriptors=reconstruct(observed,regions,name)
    r['preserve_authored_groups']=not weighted and bool(g.get('vertex_skins'));r['preserve_face_groups']=not weighted and g.get('face_skins')is not None
    for part,region in zip(r['components'],regions):
        ids=region['selector']['indices'];points=[g['vertices'][v]for i in ids for v in g['faces'][i]];extent=max(max(v[a]for v in points)-min(v[a]for v in points)for a in range(3));cell=max(6,extent/9);buckets=defaultdict(Counter)
        if g.get('face_skins')is not None:part['face_group']=Counter(g['face_skins'][i]for i in ids).most_common(1)[0][0]
        if region.get('source_texture_id',-1)>=0:continue
        for i in ids:
            a,b,c=[g['vertices'][v]for v in g['faces'][i]];u=[b[j]-a[j]for j in range(3)];v=[c[j]-a[j]for j in range(3)];cross=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]];area=math.sqrt(sum(x*x for x in cross));center=[(a[j]+b[j]+c[j])/3 for j in range(3)];key=tuple(math.floor(x/cell)for x in center);buckets[key][g['colors'][i]]+=area
        zones=[dict(center=[(v+.5)*cell for v in key],radius=cell*2.3,hsl=colors.most_common(1)[0][0])for key,colors in sorted(buckets.items(),key=lambda kv:-sum(kv[1].values()))[:192]];part['material_zones']=zones
    r['provenance']['method']='Post-examination private coarse sections, optional dominant-bone static partition and aggregate spatial face-color observations; no source topology or weighted frames copied'
    r['development_limitations']=['Target-derived static study, not original art or an unseen success.','Dominant source bone labels are geometric partitions, not inferred anatomy.','Weighted animation retargeting remains unsupported.','Brushes improve coarse native color placement; fine surface construction may remain wrong.']
    return r,regions,descriptors
