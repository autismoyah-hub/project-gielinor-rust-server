"""Focused source/review checks; does not touch authoritative renderer tests."""
from run import *
import hashlib,re
from PIL import Image,ImageChops
out=REPO/'art/tower/mesh-production-study';checks={}
for kind in ['architecture','machine','creature']:
    folder=ROOT/'runs'/(kind+'-final');target=ROOT/'reopen'/kind
    run([BLENDER,'-b',folder/'source.blend','--python-exit-code','1','--python',HERE/'reopen.py','--',target],target/'reopen.log')
    assert (folder/'asset.model').read_bytes()==(target/'asset.model').read_bytes(),kind
    checks[kind+'_saved_blend_reexport_exact']=True
dest=ROOT/'portable-scene-verification'
run([NATIVE,CACHE,dest,'scene',out/'scene/scene.tsv',out/'scene/overview.camera'],dest/'render.log')
assert ImageChops.difference(Image.open(dest/'scene.bmp').convert('RGB'),Image.open(out/'scene/overview.png').convert('RGB')).getbbox()is None
checks['portable_scene_pixel_exact']=True
missing=[]
for p in re.findall(r'(?:href|src)="([^"]+)"',(out/'index.html').read_text(encoding='utf8')):
    if not p.startswith(('http','#'))and not(out/p).exists():missing.append(p)
assert not missing,missing
checks['local_links_complete']=True
im=Image.open(out/'assets/creature-final/idle.gif');duration=0
for i in range(im.n_frames):im.seek(i);duration+=im.info.get('duration',0)
assert duration==1200,duration
checks['motion_gif_duration_ms']=duration
checks['native_pose_proof']=json.loads((out/'assets/creature-final/motion-validation.json').read_text())
checks['renderer_sha256']=hashlib.sha256(NATIVE.read_bytes()).hexdigest()
assert checks['renderer_sha256']=='f6e6e12da1a80f7824c9d90a5254964fdc8d738298417c7dcd51e0793caea7cf'
hashes=json.loads((out/'FILES.sha256.json').read_text())
assert all(hashlib.sha256((out/p).read_bytes()).hexdigest()==h for p,h in hashes.items())
checks['published_file_hashes_valid']=len(hashes)
(ROOT/'validation.json').write_text(json.dumps(checks,indent=2));print(json.dumps(checks,indent=2))
