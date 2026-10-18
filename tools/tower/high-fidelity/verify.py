"""Focused verification of saved sources, native scene and review evidence."""
from review import *
import re,hashlib,math
from PIL import ImageChops
out=REPO/'art/tower/high-fidelity-mastery';result={}
for name,folder in [('creature','creature-final'),('architecture','architecture-revision1'),('machine','machine-revision1')]:
    target=ROOT/'verification'/name;run([BLENDER,'-b',ROOT/'runs'/folder/'source.blend','--python-exit-code','1','--python',HERE/'reexport.py','--',target],target/'log.txt')
    assert(target/'asset.model').read_bytes()==(out/'assets'/name/'asset.model').read_bytes();result[name+'_source_reexport_exact']=True
    g=json.loads((out/'assets'/name/'geometry.json').read_text());assert all(math.isfinite(x)and x==round(x)for v in g['vertices']for x in v)
missing=[]
for link in re.findall(r'(?:src|href)="([^"]+)"',(out/'index.html').read_text(encoding='utf8')):
    if not link.startswith(('https:','http:','#'))and not(out/link).exists():missing.append(link)
assert not missing,missing;result['local_links_complete']=True
target=ROOT/'verification/portable-scene';run([NATIVE,CACHE,target,'scene',out/'scene/scene.tsv',out/'scene/overview.camera'],target/'log.txt')
assert ImageChops.difference(Image.open(target/'scene.bmp').convert('RGB'),Image.open(out/'scene/overview.png').convert('RGB')).getbbox()is None;result['portable_scene_pixel_exact']=True
motion=json.loads((out/'assets/creature/motion-validation.json').read_text());result['native_pose_samples']=sum(v['native_pose_samples']for v in motion.values());assert result['native_pose_samples']==409
for name,info in motion.items():
    im=Image.open(out/'assets/creature'/(name+'.gif'));duration=0
    for i in range(im.n_frames):im.seek(i);duration+=im.info.get('duration',0)
    assert duration==info['gif_ms'],(name,duration,info['gif_ms']);assert info['max_coordinate_error']==0
result['all_five_gif_durations_exact']=True
digest=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();assert digest(NATIVE)=='f6e6e12da1a80f7824c9d90a5254964fdc8d738298417c7dcd51e0793caea7cf';result['renderer_sha256']=digest(NATIVE)
hashes=json.loads((out/'FILES.sha256.json').read_text());assert all(digest(out/p)==h for p,h in hashes.items());result['file_hashes_checked']=len(hashes)
(ROOT/'validation.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2))
