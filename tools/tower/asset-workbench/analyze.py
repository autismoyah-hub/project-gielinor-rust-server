"""Source-safe numeric analysis; geometry and image input remain private.

Exact decoded fields, calculated descriptors and curator annotations are kept
separate. A component centroid is not silently promoted to an anatomical joint.
"""
import argparse,json,math,hashlib,statistics
from pathlib import Path
from collections import Counter,defaultdict
from PIL import Image,ImageChops,ImageDraw
def quantiles(xs):
    xs=sorted(xs)
    return [xs[round((len(xs)-1)*p)]for p in [0,.1,.25,.5,.75,.9,1]]if xs else []
def bound(vs):return [[min(v[i]for v in vs)for i in range(3)],[max(v[i]for v in vs)for i in range(3)]]if vs else None
def graph_components(vertices,faces,weld=False):
    parent=list(range(len(vertices)))
    def find(i):
        while parent[i]!=i:parent[i]=parent[parent[i]];i=parent[i]
        return i
    def union(a,b):parent[find(a)]=find(b)
    if weld:
        seen={}
        for i,v in enumerate(vertices):
            t=tuple(v)
            if t in seen:union(i,seen[t])
            else:seen[t]=i
    for f in faces:union(f[0],f[1]);union(f[1],f[2])
    return sorted(Counter(find(i)for i in range(len(vertices))).values(),reverse=True)
def geometry(g):
    vs,fs=g['vertices'],g['faces'];bounds=bound(vs);ext=[bounds[1][i]-bounds[0][i]for i in range(3)];edges=defaultdict(list);areas=[];normals=[];centroids=[]
    for i,f in enumerate(fs):
        a,b,c=[vs[j]for j in f];u=[b[k]-a[k]for k in range(3)];v=[c[k]-a[k]for k in range(3)];n=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]];size=math.sqrt(sum(x*x for x in n));areas.append(size/2);normals.append([x/size for x in n]if size else [0]*3);centroids.append([sum(vs[j][k]for j in f)/3 for k in range(3)])
        for a,b in zip(f,f[1:]+f[:1]):edges[tuple(sorted((a,b)))].append(i)
    dihedrals=[]
    for neighbors in edges.values():
        if len(neighbors)==2:dihedrals.append(math.degrees(math.acos(max(-1,min(1,sum(normals[neighbors[0]][i]*normals[neighbors[1]][i]for i in range(3)))))))
    points={tuple(v)for v in vs};symmetry={}
    for name,x in [('origin',0),('bounds_center',(bounds[0][0]+bounds[1][0])/2)]:symmetry[name]=sum(tuple([2*x-v[0],v[1],v[2]])in points for v in vs)/len(vs)
    palette=Counter(g['colors']);area_palette=defaultdict(float)
    for c,a in zip(g['colors'],areas):area_palette[c]+=a
    total=sum(areas);entropy=-sum((n/len(fs))*math.log2(n/len(fs))for n in palette.values())
    groups={}
    if g.get('vertex_skins')is not None:
        for label in sorted(set(g['vertex_skins'])):
            selected=[v for v,k in zip(vs,g['vertex_skins'])if k==label];groups[str(label)]=dict(vertices=len(selected),bounds=bound(selected),centroid=[sum(v[i]for v in selected)/len(selected)for i in range(3)])
    # Cross sections intersect triangle edges. Nearest-vertex bands alias narrow
    # collars into wide plates when adjacent rings have very different radii.
    profile=[]
    for i in range(33):
        y=bounds[0][1]+ext[1]*i/32;points=[]
        for f in fs:
            for j in range(3):
                a,b=vs[f[j]],vs[f[(j+1)%3]]
                if abs(a[1]-y)<1e-6:points.append(a)
                if (a[1]<y<b[1])or(b[1]<y<a[1]):
                    t=(y-a[1])/(b[1]-a[1]);points.append([a[k]+t*(b[k]-a[k])for k in range(3)])
        bb=bound(points)if points else [[0,0,0],[0,0,0]];profile.append(dict(height_fraction=i/32,width=bb[1][0]-bb[0][0],depth=bb[1][2]-bb[0][2],method='triangle-plane axis-aligned envelope; not semantic rings'))
    untextured=Counter(c for i,c in enumerate(g['colors'])if g.get('face_textures')is None or g['face_textures'][i]<0)
    return dict(exact_decoded=dict(vertices=len(vs),faces=len(fs),bounds=bounds,colors=dict(palette),render_types=dict(Counter(g.get('render_types')or [0]*len(fs))),texture_faces=sum(t>=0 for t in g.get('face_textures')or []),texture_types=dict(Counter(g.get('texture_types')or [])),alpha_present=g.get('alpha')is not None,priorities_present=g.get('priorities')is not None,face_depth_present=g.get('face_flags')is not None,skin_labels=groups),calculated=dict(extent_native=ext,extent_tiles=[x/128 for x in ext],width_height=ext[0]/ext[1]if ext[1]else None,depth_height=ext[2]/ext[1]if ext[1]else None,index_components=graph_components(vs,fs),coordinate_weld_components=graph_components(vs,fs,True),boundary_edges=sum(len(x)==1 for x in edges.values()),nonmanifold_edges=sum(len(x)>2 for x in edges.values()),zero_area_triangles=sum(x==0 for x in areas),triangle_area_quantiles=quantiles(areas),dihedral_degrees_quantiles=quantiles(dihedrals),surface_area_native_squared=total,surface_centroid=[sum(c[i]*a for c,a in zip(centroids,areas))/total for i in range(3)]if total else None,exact_mirror_coverage=symmetry,face_palette_entropy_bits=entropy,area_weighted_palette=dict(area_palette),untextured_palette=dict(untextured),untextured_area_palette={str(c):sum(a for j,a in enumerate(areas)if g['colors'][j]==c and (g.get('face_textures')is None or g['face_textures'][j]<0))for c in untextured},height_profile=profile,lightness_face_quantiles=quantiles([c&127 for c in g['colors']])),not_inferred=['semantic components','anatomical joints','material names','deliberate versus accidental simplification','artist intention','physical mass'])
def foreground(im):
    im=im.convert('RGB');d=ImageChops.difference(im,Image.new('RGB',im.size,(16,21,26)));a,b,c=d.split();return ImageChops.lighter(ImageChops.lighter(a,b),c).point(lambda v:255 if v else 0)
def mask_metrics(a,b):
    a,b=foreground(a),foreground(b);intersection=ImageChops.darker(a,b).histogram()[255];union=ImageChops.lighter(a,b).histogram()[255];ba,bb=a.getbbox(),b.getbbox();bands=[]
    if ba and bb:
        for fraction in [.15,.3,.5,.7,.9]:
            y=round(ba[1]+(ba[3]-ba[1]-1)*fraction);aa=a.crop((0,y,a.width,y+1)).getbbox();bc=b.crop((0,y,b.width,y+1)).getbbox();bands.append(dict(reference_height_fraction=fraction,row=y,reference_span=list(aa[::2])if aa else None,candidate_span=list(bc[::2])if bc else None,delta_width_px=(bc[2]-bc[0]if bc else 0)-(aa[2]-aa[0]if aa else 0)))
    return dict(iou=intersection/union if union else 1,reference_bounds=ba,candidate_bounds=bb,width_delta_px=(bb[2]-bb[0])-(ba[2]-ba[0])if ba and bb else None,height_delta_px=(bb[3]-bb[1])-(ba[3]-ba[1])if ba and bb else None,profile_bands=bands,alignment='raw matched camera; not fitted or rescaled to improve overlap')
def surface_geometry(g,indices):
    """Native-submitted face subset for descriptors, never a replacement asset."""
    used=sorted({i for n in indices for i in g['faces'][n]});remap={old:new for new,old in enumerate(used)};h=dict(g)
    h['vertices']=[g['vertices'][i]for i in used];h['faces']=[[remap[i]for i in g['faces'][n]]for n in indices]
    for key in ['colors','render_types','priorities','alpha','face_skins','face_textures','texture_coords','face_flags']:
        if g.get(key)is not None:h[key]=[g[key][i]for i in indices]
    for key in ['vertex_skins','bone_weights']:
        if g.get(key)is not None:h[key]=[g[key][i]for i in used]
    return h

def library(root,out):
    out.mkdir(parents=True,exist_ok=True);catalog=[]
    for p in sorted(root.glob('*/geometry.json')):
        g=json.loads(p.read_text());meta=json.loads((p.parent/'metadata.json').read_text());report=geometry(g);report['source']=meta;report['schema']=1
        surface=p.parent/'submitted-surface.json'
        if surface.exists():
            submitted=json.loads(surface.read_text());report['submitted_surface']=geometry(surface_geometry(g,submitted['face_indices']));report['submitted_surface']['world_bounds']=submitted['world_bounds'];report['submitted_surface']['basis']=submitted['basis']
        (out/(p.parent.name+'.json')).write_text(json.dumps(report,indent=2));catalog.append(dict(selector=meta['selector'],name=meta['name'],model_ids=meta['model_ids'],definition_sha256=meta['definition_sha256'],source_payload_hashes=[v['payload_sha256']for v in meta['sources']],vertices=len(g['vertices']),faces=len(g['faces']),textures=report['exact_decoded']['texture_faces'],sequences=meta['sequences'],bounds=report['exact_decoded']['bounds'],report=p.parent.name+'.json'))
    (out/'catalog.json').write_text(json.dumps(catalog,indent=2));print('REFERENCE_ANALYSIS_COMPLETE',len(catalog))
def compare(ref,candidate,out):
    out.mkdir(parents=True,exist_ok=True);rows={}
    for view in ['front','side','rear','quarter','gameplay']:
        a,b=Image.open(ref/(view+'.bmp')).convert('RGB'),Image.open(candidate/(view+'.bmp')).convert('RGB');metrics=mask_metrics(a,b);delta=ImageChops.difference(a,b);c=delta.split();mask=ImageChops.lighter(ImageChops.lighter(c[0],c[1]),c[2]);metrics['changed_pixels']=a.width*a.height-mask.histogram()[0];metrics['mean_rgb_absolute_error']=sum(i*n for i,n in enumerate(delta.histogram()))/(a.width*a.height*3) # replaced below with channel-local intensity
        metrics['mean_rgb_absolute_error']=sum(sum(i*n for i,n in enumerate(ch.histogram()))for ch in delta.split())/(a.width*a.height*3)
        canvas=Image.new('RGB',(a.width*3,a.height));canvas.paste(a,(0,0));canvas.paste(b,(a.width,0));canvas.paste(delta,(a.width*2,0));canvas.save(out/(view+'.png'));foreground(a).save(out/(view+'-reference-mask.png'));foreground(b).save(out/(view+'-candidate-mask.png'));rows[view]=metrics
    (out/'comparison.json').write_text(json.dumps(rows,indent=2));print('NATIVE_VIEW_DIAGNOSTICS_COMPLETE',out)
if __name__=='__main__':
    p=argparse.ArgumentParser();sub=p.add_subparsers(dest='command',required=True);q=sub.add_parser('library');q.add_argument('root',type=Path);q.add_argument('out',type=Path);q=sub.add_parser('compare');q.add_argument('reference',type=Path);q.add_argument('candidate',type=Path);q.add_argument('out',type=Path);a=p.parse_args();library(a.root,a.out)if a.command=='library'else compare(a.reference,a.candidate,a.out)
