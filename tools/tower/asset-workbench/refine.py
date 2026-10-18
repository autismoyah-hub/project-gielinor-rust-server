"""Conservative descriptor-driven local edit proposals; no artistic approval."""
import json,math,argparse
from pathlib import Path
def rotation_between(a,b,limit_degrees=10):
    if sum(x*y for x,y in zip(a,b))<0:a=[-x for x in a]
    cross=[b[1]*a[2]-b[2]*a[1],b[2]*a[0]-b[0]*a[2],b[0]*a[1]-b[1]*a[0]];norm=math.sqrt(sum(x*x for x in cross));angle=min(math.radians(limit_degrees),math.acos(max(-1,min(1,sum(x*y for x,y in zip(a,b))))))
    if norm<1e-8:return [[1,0,0],[0,1,0],[0,0,1]]
    x,y,z=[v/norm for v in cross];c,s,t=math.cos(angle),math.sin(angle),1-math.cos(angle)
    return [[t*x*x+c,t*x*y-s*z,t*x*z+s*y],[t*x*y+s*z,t*y*y+c,t*y*z-s*x],[t*x*z-s*y,t*y*z+s*x,t*z*z+c]]
def propose(diagnostics,bindings,geometry):
    ops=[]
    for name,record in diagnostics.items():
        policy=bindings[name];a,b=record.get('reference'),record.get('candidate')
        if not a or not b:continue
        names=policy['candidate'].get('components',[])
        if not names:continue
        if policy.get('auto_orientation'):
            target=record['reference']['principal_axes'][0]['axis'];current=record['candidate']['principal_axes'][0]['axis'];anchor=policy.get('anchor_components',names);ids=sorted({i for n in anchor for i in geometry['components'][n]['vertices']});pivot=[sum(geometry['vertices'][i][a]for i in ids)/len(ids)for a in range(3)];ops.append(dict(op='rotate',components=names,matrix=rotation_between(target,current),pivot=pivot,reason='component plane-normal discrepancy; bounded 10-degree edit around supplied attachment proxy'))
        elif policy.get('auto_affine'):
            scale=[max(.9,min(1.1,a['extents'][i]/b['extents'][i]))if b['extents'][i]else 1 for i in range(3)];center=[(b['bounds'][0][i]+b['bounds'][1][i])/2 for i in range(3)];target=[(a['bounds'][0][i]+a['bounds'][1][i])/2 for i in range(3)];translation=[max(-3,min(3,target[i]-center[i]))for i in range(3)];ops.append(dict(op='affine',components=names,scale=scale,pivot=center,translation=translation,reason='explicit curator-approved component bounds; bounded 10-percent / three-unit change'))
        if policy.get('auto_height_palette'):
            bands=a['height_band_palettes'];fallback=max(a['palette'],key=a['palette'].get);colors=[int(max(v,key=v.get))if v else int(fallback)for v in bands];ops.append(dict(op='height_palette',components=policy.get('palette_components',names),colors=colors,bounds_y=[b['bounds'][0][1],b['bounds'][1][1]],reason='coarse material-region descriptor; native HSL values, no original face assignments copied'))
    return dict(schema=1,operations=ops,acceptance='Proposal only; compare native views and reject regressions. Semantic annotation coverage must be reviewed.')
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('diagnostics',type=Path);p.add_argument('bindings',type=Path);p.add_argument('geometry',type=Path);p.add_argument('output',type=Path);a=p.parse_args();a.output.write_text(json.dumps(propose(json.loads(a.diagnostics.read_text()),json.loads(a.bindings.read_text()),json.loads(a.geometry.read_text())),indent=2));print('LOCAL_REFINEMENT_PROPOSAL_COMPLETE')
