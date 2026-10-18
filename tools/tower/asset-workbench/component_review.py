"""Native occlusion-aware semantic masks and internal/material diagnostics.

Bindings are curated. No automatic claim of anatomical or material identity is
made from colors. Masks use the native client rasterizer with original geometry,
priority and alpha; only mask colors/textures are replaced for identification.
"""
import argparse,json,subprocess,math
from pathlib import Path
from PIL import Image,ImageChops,ImageFilter,ImageStat
from analyze import foreground

def selected(g,spec):
    result=[]
    for i,f in enumerate(g['faces']):
        yes=True
        if 'indices'in spec:yes&=i in spec['indices']
        if 'colors'in spec:yes&=g['colors'][i]in spec['colors']
        if 'textures'in spec:yes&=(g.get('face_textures')or[-1]*len(g['faces']))[i]in spec['textures']
        if 'skin_labels'in spec:yes&=any((g.get('vertex_skins')or[])[v]in spec['skin_labels']for v in f)
        if 'centroid_bounds'in spec:
            c=[sum(g['vertices'][v][a]for v in f)/3 for a in range(3)];lo,hi=spec['centroid_bounds'];yes&=all(lo[a]<=c[a]<=hi[a]for a in range(3))
        if 'components'in spec:yes&=any(i in g['components'][n]['faces']for n in spec['components'])
        if yes:result.append(i)
    return result

def binmask(path):return Image.open(path).convert('L').point(lambda x:255 if x>=160 else 0)
def metrics(a,b):
    inter=ImageChops.darker(a,b).histogram()[255];union=ImageChops.lighter(a,b).histogram()[255];return dict(iou=inter/union if union else 1,reference_bounds=a.getbbox(),candidate_bounds=b.getbbox())
def edges(im):
    mask=foreground(im).filter(ImageFilter.MinFilter(5));edge=im.convert('L').filter(ImageFilter.FIND_EDGES).point(lambda v:255 if v>18 else 0);return ImageChops.darker(edge,mask)
def internal(a,b):
    aa,bb=edges(a),edges(b);da,db=aa.filter(ImageFilter.MaxFilter(5)),bb.filter(ImageFilter.MaxFilter(5));ac,bc=aa.histogram()[255],bb.histogram()[255];precision=ImageChops.darker(bb,da).histogram()[255]/bc if bc else 0;recall=ImageChops.darker(aa,db).histogram()[255]/ac if ac else 0
    return dict(reference_edge_pixels=ac,candidate_edge_pixels=bc,two_pixel_tolerance_precision=precision,two_pixel_tolerance_recall=recall,interpretation='Image edge agreement is affected by texture, shading and geometry; not a topology or artistic score')

def review(a):
    data=json.loads(a.manifest.read_text());a.out.mkdir(parents=True,exist_ok=True);regions={};whole={}
    for name,binding in data['regions'].items():
        root=a.out/name;root.mkdir(exist_ok=True);counts={}
        for side in ['reference','candidate']:
            source=data[side];g=json.loads(Path(source['geometry']).read_text());ids=selected(g,binding[side]);counts[side]=len(ids);selection=root/(side+'-faces.txt');selection.write_text(' '.join(map(str,ids)));command=[str(a.native),str(a.cache),str(root/side),'mask',source['model'],data['reference']['camera'],str(selection),*map(str,source.get('scale',[128,128,128]))];subprocess.run(command,check=True)
        views={}
        for view in ['front','side','rear','quarter','gameplay']:
            am,bm=[binmask(root/side/(view+'.bmp'))for side in ['reference','candidate']];m=metrics(am,bm);colors={}
            for side,mask in [('reference',am),('candidate',bm)]:
                im=Image.open(Path(data[side]['images'])/(view+'.bmp')).convert('RGB');colors[side]=ImageStat.Stat(im,mask=mask).mean if mask.getbbox()else None
            m['region_mean_native_rgb']=colors
            if colors['reference']and colors['candidate']:m['region_mean_rgb_distance']=math.dist(colors['reference'],colors['candidate'])
            canvas=Image.new('RGB',(am.width*2,am.height),(16,21,26));canvas.paste(am.convert('RGB'),(0,0));canvas.paste(bm.convert('RGB'),(am.width,0));canvas.save(root/(view+'-mask-pair.png'));views[view]=m
        regions[name]=dict(face_counts=counts,views=views,parameters=binding.get('parameters',[]),curated_interpretation=binding.get('finding',''),worst_silhouette_view=min(views,key=lambda v:views[v]['iou']),artistic_acceptance=False)
    for view in ['front','side','rear','quarter','gameplay']:
        aimg,bimg=[Image.open(Path(data[s]['images'])/(view+'.bmp')).convert('RGB')for s in ['reference','candidate']];union=ImageChops.lighter(foreground(aimg),foreground(bimg));delta=ImageChops.difference(aimg,bimg);whole[view]=dict(mean_rgb_error_in_foreground_union=sum(ImageStat.Stat(delta,mask=union).mean)/3,internal_structure=internal(aimg,bimg))
    result=dict(regions=regions,whole=whole,binding_provenance=data.get('provenance'),artistic_acceptance=False);(a.out/'component-diagnostics.json').write_text(json.dumps(result,indent=2));print('NATIVE_COMPONENT_MATERIAL_DIAGNOSTICS_COMPLETE');return result
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('manifest',type=Path);p.add_argument('--native',type=Path,required=True);p.add_argument('--cache',type=Path,required=True);p.add_argument('--out',type=Path,required=True);review(p.parse_args())
