"""Curate original-only assets; private cache-backed evidence stays outside Git."""
import json,secrets,shutil,hashlib,sys,datetime
from pathlib import Path
T=Path(__file__).resolve().parent;E=T.parents[3];R=E/'concept-ab-private';S=T.parents[2];P=S/'art/tower/concept-ab'
sys.path[:0]=[str(T),str(T.parent/'asset-workbench')]
from run import CFG,BL,execute
from registry import object_package
from PIL import Image
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
P.mkdir(parents=True,exist_ok=True)
if not (R/'blind-mapping.json').exists():
    choices=['text','concept'];secrets.SystemRandom().shuffle(choices)
    (R/'blind-mapping.json').write_text(json.dumps(dict(zip(['A','B'],choices)),indent=2)+'\n')
mapping=json.loads((R/'blind-mapping.json').read_text());bounds=json.loads((R/'fresh-reference/original-id-bounds.json').read_text());registry={'scope':'Isolated static NPC mesh benchmark; no live registry/cache writes or NPC-definition installation','source_bounds':bounds,'entries':{}}
for i,(label,method) in enumerate(mapping.items()):
    source=R/method/'final';dest=P/'models'/label;dest.mkdir(parents=True,exist_ok=True)
    for name in ['asset.model','source.blend','geometry.json','conversion.json','components.json','recipe.json']:
        shutil.copy2(source/name,dest/name)
    native=dest/'native';native.mkdir(exist_ok=True)
    for view in ['front','side','rear','quarter','above','gameplay']:
        shutil.copy2(source/'native'/(view+'.png'),native/(view+'.png'));shutil.copy2(source/'native'/(view+'.camera'),native/(view+'.camera'))
    # Preserve the distant history camera, add a matched ordinary closer orbit.
    shutil.copy2(native/'gameplay.png',native/'gameplay-distant.png');shutil.copy2(native/'gameplay.camera',native/'gameplay-distant.camera')
    (native/'scene.tsv').write_text('light\t72\t850\nowned\t../asset.model\t0\t0\t0\t0\t-\t0\n')
    (native/'gameplay.camera').write_text('player 0 -84 0 2000 14500 600')
    runtime=R/'comparison'/label;runtime.mkdir(parents=True,exist_ok=True)
    execute([CFG['renderer'],CFG['cache'],runtime/'gameplay','scene',native/'scene.tsv',native/'gameplay.camera'],runtime/'gameplay.log')
    Image.open(runtime/'gameplay/scene.bmp').save(native/'gameplay.png')
    for view,camera in {'face':'look 150 -160 -315 0 -116 -109 1000 1000 800','armor':'look 220 -240 -160 0 -115 -15 1000 1000 800'}.items():
        cp=runtime/(view+'.camera');cp.write_text(camera)
        execute([CFG['renderer'],CFG['cache'],runtime/view,'scene',native/'scene.tsv',cp],runtime/(view+'.log'))
        Image.open(runtime/view/'scene.bmp').save(native/(view+'.png'));shutil.copy2(cp,native/(view+'.camera'))
    ids={'model_id':1210800+i,'object_id':64680+i,'npc_id':None,'role':'Static original NPC mesh; diagnostic object proxy only','installed':False}
    registry['entries'][label]=ids
    object_package(dest/'package',[json.loads((dest/'geometry.json').read_text())],{'shape':10,'size':[3,3],'non_flat_shading':True,'scale':[128]*3},[ids['model_id']],ids['object_id'],bounds)
    execute([CFG['renderer'],CFG['cache'],runtime/'package','package',dest/'package',ids['object_id'],10,R/'decode-camera.txt',72,850],runtime/'package.log')
    decoded=json.loads((runtime/'package/decoded.json').read_text());g=json.loads((dest/'geometry.json').read_text())
    assert all(g[k]==decoded[k]for k in ['vertices','faces','colors','render_types'])
    ids['native_package_decode_exact']=True
    # First, intermediate and final checkpoints are all original, with no runtime palette/log copies.
    for candidate in ['blockout','refined','final']:
        old=R/method/candidate;history=P/'history'/label/candidate;history.mkdir(parents=True,exist_ok=True)
        for name in ['source.blend','asset.model','recipe.json','iteration.json','generation-snapshot.py']:shutil.copy2(old/name,history/name)
        for view in ['front','side','rear','quarter','above','gameplay']:shutil.copy2(old/'native'/(view+'.png'),history/(view+'.png'))
shutil.copy2(R/'design.json',P/'design.json');shutil.copy2(R/'design.sha256',P/'design.sha256')
original=P/'technical/original-concept';original.mkdir(parents=True,exist_ok=True)
for name in ['hero.png','turnaround-v1.png','turnaround-v2.png','front.png','left.png','rear.png','quarter.png','consistency.json','hero-prompt.txt','turnaround-prompt.txt','toe-correction-prompt.txt','generation-start.json']:shutil.copy2(R/'concept'/name,original/name)
(P/'registry.json').write_text(json.dumps(registry,indent=2)+'\n')
(R/'package-validation.json').write_text(json.dumps({'entries':registry['entries'],'source_safe_original_assets_only':True,'cache_geometry_and_environment_images_excluded_from_publication':True,'created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()},indent=2)+'\n')
print('Original-only package curated; both static registry proxies decode exactly.')
