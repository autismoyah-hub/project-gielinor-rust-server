from run import *
import hashlib,re,math
from PIL import ImageChops
out=REPO/'art/tower/static-production-benchmark';report={};selected={'anvil':'final2','workstation':'final2','machine':'final','creature':'final2'}
for name,version in selected.items():
    dest=ROOT/'verification'/name;run([BLENDER,'-b',out/'assets'/name/'source.blend','--python-exit-code','1','--python',HERE/'reexport.py','--',dest],dest/'log.txt')
    assert(dest/'asset.model').read_bytes()==(out/'assets'/name/'asset.model').read_bytes();g=json.loads((out/'assets'/name/'geometry.json').read_text());assert g['vertex_skins']is None;assert all(math.isfinite(x)and x==round(x)for v in g['vertices']for x in v);report[name+'_source_reexport_exact_static']=True
assert not list(out.rglob('*.bin'));report['no_rig_or_animation_payloads']=True
missing=[]
for link in re.findall(r'(?:href|src)="([^"]+)"',(out/'index.html').read_text(encoding='utf8')):
    if not link.startswith(('http:','https:','#'))and not(out/link).exists():missing.append(link)
assert not missing,missing;report['local_links_complete']=True
dest=ROOT/'verification/portable-scene';run([NATIVE,CACHE,dest,'scene',out/'scene/scene.tsv',out/'scene/overview.camera'],dest/'log.txt')
assert ImageChops.difference(Image.open(dest/'scene.bmp').convert('RGB'),Image.open(out/'scene/overview.png').convert('RGB')).getbbox()is None;report['portable_scene_pixel_exact']=True
for view in ['front','side','rear','quarter']:
    a=Image.open(out/'reference-comparison'/('reference-'+view+'.png')).convert('RGB');b=Image.open(out/'assets/anvil'/(view+'.png')).convert('RGB');overlay=Image.open(out/'reference-comparison'/('overlay-'+view+'.png')).convert('RGB');assert ImageChops.difference(Image.blend(a,b,.5),overlay).getbbox()is None
report['matched_anvil_overlays_exact']=True
digest=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();report['renderer_sha256']=digest(NATIVE);assert report['renderer_sha256']=='f6e6e12da1a80f7824c9d90a5254964fdc8d738298417c7dcd51e0793caea7cf'
hashes=json.loads((out/'FILES.sha256.json').read_text());assert all(digest(out/p)==h for p,h in hashes.items());report['file_hashes_checked']=len(hashes)
(ROOT/'validation.json').write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2))
