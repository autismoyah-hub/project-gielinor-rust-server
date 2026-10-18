"""Explicit descriptor-driven independent authoring, with construction semantics.

The envelope is an authoring dimension constraint, not an image registration.
No reference vertex/face arrays or frame transforms are read.
"""
import json,argparse,copy,math
from pathlib import Path
import construction as C
from creature_families import mammal,bat
from primitives import make,socket
from templates import humanoid,wheel,pillar
from humanoid_families import small_humanoid

BUILDERS=dict(barrel=C.barrel,crate=C.crate,shield=C.shield,scimitar=C.scimitar,anvil=C.anvil,arch=C.arch,masonry_pier=C.masonry,chest=C.chest,bench=C.bench,pickaxe=C.pickaxe,hammer=C.hammer)

def vertices(spec):
    if spec['type']=='profile':return socket(socket([[x,y,z]for z in [-spec['depth']/2,spec['depth']/2]for x,y in spec['profile']],spec.get('socket',{})),spec.get('parent_socket',{}))
    return make(spec)[0]

def build(kind,report):
    surface=report.get('high_opacity_surface',report.get('submitted_surface',report))if kind=='bat'else report.get('submitted_surface',report);b=surface.get('world_bounds',report['source']['world_bounds'])
    if report.get('curated_regions',{}).get('anatomy'):b=report['curated_regions']['anatomy']['world_bounds']
    extent=[b[1][i]-b[0][i]for i in range(3)];x,y,z=extent;basis=[[1,0,0],[0,1,0],[0,0,1]]
    if kind in ['scimitar','pickaxe','hammer','shield']:
        normal=min(range(3),key=extent.__getitem__);long=max(range(3),key=extent.__getitem__);wide=next(i for i in range(3)if i not in [normal,long]);basis=[[int(i==wide)for i in range(3)],[int(i==long)for i in range(3)],[int(i==normal)for i in range(3)]]
        try:socket([[0,0,0]],dict(basis=basis))
        except ValueError:basis[0]=[-v for v in basis[0]]
        if kind=='scimitar':r=C.scimitar(extent[long],extent[wide]*.55,extent[wide]*.6)
        elif kind=='shield':r=C.shield(extent[wide],extent[long],max(4,extent[normal]*.55))
        elif kind=='pickaxe':r=C.pickaxe(extent[long],extent[wide])
        else:r=C.hammer(extent[long],extent[wide])
    elif kind=='anvil':r=C.anvil(max(x,z),y,min(x,z));basis=[[1,0,0],[0,1,0],[0,0,1]]if x>=z else[[0,0,1],[0,1,0],[-1,0,0]]
    elif kind=='arch':r=C.arch(x,y,z)
    elif kind=='masonry_pier':r=C.masonry(x,y,z,rows=4)
    elif kind=='wolf':r=mammal(x,y,z,'wolf')
    elif kind=='bear':r=mammal(x,y,z,'bear')
    elif kind=='bat':r=bat(x,y,z)
    elif kind in ['goblin','imp']:r=small_humanoid()
    elif kind=='pillar':r=pillar(x,y,z)
    elif kind=='wheel':r=wheel(x,y,z)
    elif kind in ['barrel','crate','chest','bench']:r=BUILDERS[kind](x,y,z)
    else:raise ValueError('Unsupported semantic family '+kind)
    # Palette matching is explicit nearest native-HSL family matching, not a claim
    # that a material identity can be decoded from an unlabeled face color.
    palette=surface['calculated'].get('untextured_area_palette')or surface['exact_decoded']['colors'];codes=list(map(int,palette))
    for mat in r['materials'].values():
        if not mat.get('family')or not codes:continue
        base=mat['hsl'];h,s,l=base>>10,(base>>7)&7,base&127
        def distance(code):
            hh,ss,ll=code>>10,(code>>7)&7,code&127;dh=min(abs(h-hh),64-abs(h-hh));return dh*(.25 if s<2 else 1)+abs(s-ss)*4+abs(l-ll)*.18
        candidates=codes
        if mat['family']=='wood':
            warm=[c for c in codes if ((c>>7)&7)>=2 and min(abs((c>>10)-8),64-abs((c>>10)-8))<=12]
            if warm:candidates=warm
        if mat['family']=='organic'and mat.get('subtype')!='keratin':target=max(codes,key=lambda c:float(palette[str(c)]))
        else:target=min(candidates,key=distance)
        if mat['family']=='bronze':target=max((c for c in codes if ((c>>7)&7)>=3),key=lambda c:c&127,default=target)
        if mat['family']=='wood'and report.get('texture_statistics'):target=report['texture_statistics'][0]['nearest_unlit_native_palette_code']
        old=mat['hsl'];mat['hsl']=target
        for key in ['shadow','end','edge']:mat[key]=(target&65408)|max(2,min(126,(target&127)+(mat[key]&127)-(old&127)))
    for p in r['components']:
        old=p.get('parent_socket',{});origin=socket([old.get('origin',[0,0,0])],dict(basis=basis))[0];oldbasis=old.get('basis',[[1,0,0],[0,1,0],[0,0,1]]);newbasis=[[sum(row[k]*basis[k][j]for k in range(3))for j in range(3)]for row in oldbasis];p['parent_socket']=dict(origin=origin,basis=newbasis)
    pts=[v for p in r['components']for v in vertices(p)];lo=[min(v[i]for v in pts)for i in range(3)];hi=[max(v[i]for v in pts)for i in range(3)];scale=[extent[i]/max(1,hi[i]-lo[i])for i in range(3)];offset=[b[0][i]-lo[i]*scale[i]for i in range(3)]
    for p in r['components']:p['asset_transform']=dict(scale=scale,offset=offset)
    def transform(point):return [round(point[i]*scale[i]+offset[i])for i in range(3)]
    if r.get('rig'):
        if basis!=[[1,0,0],[0,1,0],[0,0,1]]:raise ValueError('Animated semantic family must use native axes')
        for j in r['rig']:j['origin']=transform(j['origin'])
        for a in r['actions'].values():
            for key in a['keys']:key['translation']=[round(key.get('translation',[0,0,0])[i]*scale[i])for i in range(3)]
            if a.get('gait'):
                gait=a['gait'];gait['stride']*=scale[2];gait['lift']*=scale[1]
                for limb in gait['limbs']:
                    for key in ['hip','knee','tip']:limb[key]=transform(limb[key])
    r['provenance']=dict(kind='independent_reference_development',reference_selector=report['source']['selector'],definition_sha256=report['source']['definition_sha256'],source_mesh_arrays_read=False,method='Explicit semantic family, native family palette, and owned dimension-constrained construction; no image registration',artistic_acceptance=False);return r
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('kind');p.add_argument('analysis',type=Path);p.add_argument('out',type=Path);a=p.parse_args();a.out.parent.mkdir(parents=True,exist_ok=True);a.out.write_text(json.dumps(build(a.kind,json.loads(a.analysis.read_text())),indent=2));print('SEMANTIC_OWNED_RECIPE_COMPLETE',a.kind)
