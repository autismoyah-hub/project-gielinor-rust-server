"""Explicit allowlist export of original art; protected studies never copied."""
import argparse,json,hashlib,shutil,html,subprocess
from pathlib import Path
from PIL import Image
load=lambda p:json.loads(p.read_text(encoding='utf-8'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def run(a):
    if a.out.exists():raise ValueError('Publication output must be fresh')
    a.out.mkdir(parents=True);assets={k:Path(v)for k,v in load(a.assets).items()};assets.update({p.name:p for p in a.floor.glob('ground-*')if p.is_dir()});assets.update({p.name:p for p in(a.private/'originals-v1').iterdir()if p.is_dir()});manifest=[];sections=[]
    for key,root in sorted(assets.items()):
        r=load(root/'recipe.json');g=load(root/'geometry.json');report=load(root/'authoring-report.json')
        assert r['provenance']['kind']=='original_tower_asset'and r['provenance']['source_mesh_or_animation_reuse']is False,key
        assert not any(v>=0 for v in(g.get('face_textures')or[])),key
        dst=a.out/'assets'/key;dst.mkdir(parents=True)
        for name in ['recipe.json','geometry.json','asset.model']:shutil.copy2(root/name,dst/name)
        shutil.copytree(root/'object-package',dst/'object-package')
        actions=list(r.get('actions',{}))
        for action in actions:
            folder=dst/action;folder.mkdir()
            for path in(root/action).iterdir():
                if path.suffix in ['.bin','.json']:shutil.copy2(path,folder/path.name)
        for view in ['front','side','rear','quarter','gameplay']:
            Image.open(root/'native'/(view+'.bmp')).save(dst/(view+'.png'))
        previews=a.private/'original-boss-review-v1'/key/'previews'
        if previews.exists():
            for path in previews.glob('*.gif'):shutil.copy2(path,dst/path.name)
        row=dict(asset=key,name=r['name'],identity=report['identity'],asset_key=report['asset_key'],model_sha256=sha(dst/'asset.model'),vertices=len(g['vertices']),faces=len(g['faces']),bounds=report['geometry']['exact_decoded']['bounds'],actions=actions,original_geometry_and_choreography=True,source_commit=a.commit,generator='tools/tower/asset-workbench/author_workflow.py',recipe='assets/'+key+'/recipe.json',technical_conversion='VALIDATED',artistic_acceptance=False,gameplay_integration_tested=False)
        (dst/'manifest.json').write_text(json.dumps(row,indent=2),encoding='utf-8');manifest.append(row)
    floor=a.out/'floor';floor.mkdir();proof=load(a.floor/'floor-compatibility.json');proof['source_commit']=a.commit;proof['assets']={r['asset']:r for r in manifest};proof['anticipated_collision']=[]
    bykey={r['asset']:r for r in manifest}
    for index,p in enumerate(proof['placements']):
        if p['asset'].startswith('ground-'):continue
        lo,hi=bykey[p['asset']]['bounds'];q=p['quarter_turns']%4;points=[]
        for original_x in [lo[0],hi[0]]:
            for original_z in [lo[2],hi[2]]:
                x,z=original_x,original_z
                for turn in range(q):x,z=z,-x
                points.append([x+p['position'][0],z+p['position'][2]])
        proof['anticipated_collision'].append(dict(placement=index,asset=p['asset'],conservative_xz_aabb=[[min(v[i]for v in points)for i in range(2)],[max(v[i]for v in points)for i in range(2)]],status='PROXY_ONLY',note='Whole asset bounds overestimate arches, raised sluices and articulated creatures. Requires authored future clipping; not a validated blocker.'))
    proof['intended_anchors']={k:dict(local_xz=v['anchor'],purpose=v['story'],status='VISUAL_DESIGN_ONLY')for k,v in proof['areas'].items()};proof['future_integration_requirements']=['Allocate IDs against the destination cache and shared registry; these IDs are isolated experimental reservations.','Split into destination scene regions; implement terrain heights, roof occlusion and hand-authored clipping after design approval.','Retest passage widths with production player collision, scene lights and camera.','Resolve authored creature stance drift and floating feet before gameplay acceptance.']
    (floor/'compatibility.json').write_text(json.dumps(proof,indent=2),encoding='utf-8');shutil.copy2(a.floor/'circulation.json',floor/'circulation.json')
    for camera in a.floor.glob('*.camera'):shutil.copy2(camera,floor/camera.name)
    for view in(a.floor/'views').iterdir():Image.open(view/'scene.bmp').save(floor/(view.name+'.png'))
    rows=['light\t64\t768']
    for p in proof['placements']:rows.append('\t'.join(['owned','../assets/'+p['asset']+'/asset.model',*map(str,p['position']),str(p['quarter_turns']),'../assets/'+p['asset']+'/'+p['action']if p['action']else'-','0']))
    (floor/'scene.tsv').write_text('\n'.join(rows)+'\n',encoding='utf-8');(a.out/'manifest.json').write_text(json.dumps(dict(source_commit=a.commit,assets=manifest,cache_resources_included=False,artistic_acceptance=False),indent=2),encoding='utf-8');print('SOURCE_SAFE_ORIGINAL_ART_PACKAGED',len(manifest),'assets')
if __name__=='__main__':
    p=argparse.ArgumentParser()
    for n in ['private','assets','floor','out']:p.add_argument('--'+n,type=Path,required=True)
    p.add_argument('--commit',required=True);run(p.parse_args())
