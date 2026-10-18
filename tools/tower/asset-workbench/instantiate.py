"""Make an independent archetype recipe from low-dimensional analysis descriptors.

No original vertex/face arrays are accepted. Descriptor-driven held-out fitting
is reported as such, not blind class prediction or artistic acceptance.
"""
import argparse,json,copy
from pathlib import Path
from templates import pillar,anvil,shield,sword,wheel,barrel,crate,quadruped,recipe,part
def make(kind,report):
    surface=report.get('submitted_surface',report);exact=surface['exact_decoded'];calc=surface['calculated'];b=surface.get('world_bounds',report['source']['world_bounds']);x,y,z=[b[1][i]-b[0][i]for i in range(3)];center=[sum(v)/2 for v in zip(*b)]
    if kind=='pillar':
        r=pillar(x,y,z);r['components'][0]['sections']=[[center[0],b[0][1]+row['height_fraction']*y,center[2],max(2,row['width']/2),max(2,row['depth']/2)]for row in calc['height_profile']];r['components'][0]['outline']=[[-1,-.7],[-.7,-1],[.7,-1],[1,-.7],[1,.7],[.7,1],[-.7,1],[-1,.7]];r['components']=r['components'][:1]
    elif kind=='anvil':
        # Industrial objects can be authored along either horizontal axis.
        # This correction was motivated by frozen-v1 failure on object:2097;
        # that asset is development data for this version, not a new holdout.
        r=anvil(max(x,z),y,min(x,z));basis=[[1,0,0],[0,1,0],[0,0,1]]if x>=z else [[0,0,1],[0,1,0],[-1,0,0]]
        for c in r['components']:c['parent_socket']=dict(origin=[center[0],center[1]+y/2,center[2]],basis=basis)
    elif kind=='shield':
        # Worn/inventory coordinate frames are different: orient from dimensions,
        # with vertical Y retained for worn geometry and Z for inventory geometry.
        normal=min(range(3),key=lambda i:[x,y,z][i]);axes=[i for i in range(3)if i!=normal];vertical=1 if 1 in axes else 2;horizontal=next(i for i in axes if i!=vertical);r=shield([x,y,z][horizontal],[x,y,z][vertical],max(4,[x,y,z][normal]*.5));basis=[[float(i==horizontal)for i in range(3)],[float(i==vertical)for i in range(3)],[float(i==normal)for i in range(3)]]
        det=(basis[0][0]*(basis[1][1]*basis[2][2]-basis[1][2]*basis[2][1])-basis[0][1]*(basis[1][0]*basis[2][2]-basis[1][2]*basis[2][0])+basis[0][2]*(basis[1][0]*basis[2][1]-basis[1][1]*basis[2][0]))
        if det<0:basis[0]=[-v for v in basis[0]]
        for c in r['components']:c['parent_socket']=dict(origin=center,basis=basis)
    elif kind=='sword':
        # A sword's longest axis is its blade, not necessarily native Y (worn
        # and inventory factories deliberately use different coordinate frames).
        dims=[x,y,z];long=max(range(3),key=dims.__getitem__);thin=min(range(3),key=dims.__getitem__);wide=next(i for i in range(3)if i not in [long,thin]);length=dims[long];r=sword(length,max(5,dims[wide]/2.2),curve=0)
        basis=[[float(i==wide)for i in range(3)],[float(i==long)for i in range(3)],[float(i==thin)for i in range(3)]]
        from primitives import socket
        try:socket([[0,0,0]],dict(basis=basis))
        except ValueError:basis[0]=[-v for v in basis[0]]
        origin=[center[i]+length/2*basis[1][i]for i in range(3)]
        for c in r['components']:c['parent_socket']=dict(origin=origin,basis=basis)
    elif kind=='wheel':r=wheel(x,y,z)
    elif kind=='barrel':r=barrel(x,y,z)
    elif kind=='crate':r=crate(x,y,z)
    elif kind=='quadruped':r=quadruped(x,y,z)
    elif kind=='wall':r=recipe('Independent wall-panel study',[part('panel','box','stone',center=center,size=[max(2,x),max(2,y),max(2,z)])])
    else:raise ValueError('Unsupported independent archetype '+kind)
    palette=calc.get('untextured_area_palette')or calc.get('untextured_palette')or exact['colors'];colors=[int(c)for c in sorted(palette,key=lambda c:-palette[c])]
    # Region roles are deliberately coarse; lack of painting remains measurable.
    if colors:
        for name,material in r['materials'].items():material['hsl']=colors[0 if name not in ['edge','iron_edge','wood_dark','stone_shadow','skin_shadow']else min(1,len(colors)-1)]
    r['provenance']=dict(kind='independent_reference_study',method='Owned archetype plus bounds/profile/palette descriptors; no source vertex or face arrays',reference_selector=report['source']['selector'],reference_definition_sha256=report['source']['definition_sha256'],artistic_acceptance=False)
    return r
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('kind');p.add_argument('analysis',type=Path);p.add_argument('output',type=Path);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_text(json.dumps(make(a.kind,json.loads(a.analysis.read_text())),indent=2));print('INDEPENDENT_ARCHETYPE_RECIPE_COMPLETE',a.kind)
