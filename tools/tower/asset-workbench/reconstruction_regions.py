"""Private, data-assisted section studies with explicit provenance.

Region proposals use native animation labels/material IDs. They are geometric
partitions, not inferred anatomy. Curators can replace them before authoring.
This is coarse descriptor reconstruction, never an original asset generator.
"""
import argparse,json
from collections import defaultdict,Counter
from pathlib import Path
from section_reconstruction import reconstruct

def propose(g):
    buckets=defaultdict(list);skins=g.get('vertex_skins');textures=g.get('face_textures')or[-1]*len(g['faces'])
    for i,f in enumerate(g['faces']):
        a,b,c=[g['vertices'][v]for v in f];u=[b[j]-a[j]for j in range(3)];v=[c[j]-a[j]for j in range(3)];cross=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
        if sum(x*x for x in cross)==0 or(g.get('alpha')or[0]*len(g['faces']))[i]==255:continue
        label=Counter(skins[v]for v in f).most_common(1)[0][0]if skins else-1
        key=(label,g['colors'][i]if not skins else-1,textures[i]);buckets[key].append(i)
    regions=[]
    for n,((label,color,texture),ids)in enumerate(sorted(buckets.items())):
        points=[g['vertices'][v]for i in ids for v in g['faces'][i]];extent=[max(v[a]for v in points)-min(v[a]for v in points)for a in range(3)];axis=max(range(3),key=extent.__getitem__)
        if extent[axis]<2:continue
        regions.append(dict(name=f'region_{n}_group_{label}',selector=dict(indices=ids),axis=axis,bins=6 if len(ids)>8 else 3,sides=8 if len(ids)>12 else 4,group=label if 0<=label<=255 else 0,source_texture_id=texture,alpha=Counter((g.get('alpha')or[0]*len(g['faces']))[i]for i in ids).most_common(1)[0][0],priority=Counter((g.get('priorities')or[g.get('priority',0)]*len(g['faces']))[i]for i in ids).most_common(1)[0][0],annotation='Native group/material partition, not automatic semantic recognition; zero-area/fully invisible faces omitted'))
    return regions

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('geometry',type=Path);p.add_argument('out',type=Path);a=p.parse_args();g=json.loads(a.geometry.read_text());regions=propose(g);r,d=reconstruct(g,regions);a.out.mkdir(parents=True,exist_ok=True);(a.out/'regions.json').write_text(json.dumps(regions,indent=2));(a.out/'landmark-observations.json').write_text(json.dumps(d,indent=2));(a.out/'recipe.json').write_text(json.dumps(r,indent=2));print('PRIVATE_REGION_RECONSTRUCTION_RECIPE_COMPLETE',len(regions))
