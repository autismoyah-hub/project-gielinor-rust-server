"""Verify original meshes, preservation, decode and saved-source export."""
from pathlib import Path
import json,hashlib,subprocess,sys,datetime
import numpy as np
T=Path(__file__).resolve().parent;E=T.parents[3];R=E/'concept-ab-private'
sys.path[:0]=[str(T),str(T.parent/'constrained')]
from run import CFG,CASE,BL,execute
from compare import render
import geometry_audit
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
baseline=json.loads((R/'baseline.json').read_text())
assert sha(R/'design.json')==baseline['frozen_design_sha256']
assert all(sha(E/p)==h for p,h in baseline['files'].items())
frozen=json.loads((R/'text-freeze.json').read_text())
assert all(sha(R/'text/final'/p)==h for p,h in frozen['files'].items())
verification={'frozen_design_sha256':sha(R/'design.json'),'baseline_files_unchanged':True,'text_final_unchanged_since_preconcept_freeze':True,'renderer_unchanged':sha(CFG['renderer'])==baseline['renderer_sha256'],'models':{}}
execute([CFG['renderer'],CFG['cache'],R/'fresh-reference','inspect-static','npc:655','npc:2790','npc:106','item:1323'],R/'fresh-reference.log')
old=json.loads((E/'constrained-private/config.json').read_text())
for case in old['cases'].values():
    new=json.loads((R/'fresh-reference'/case['selector']/'metadata.json').read_text())
    assert all(new[k]==case['metadata'][k] for k in ['sources','definition_sha256','revision','render_scale','ambient','contrast'])
verification['original_cache_target_identities_unchanged']=True
for method in ['text','concept']:
    p=R/method/'final'
    if not (p/'source-reexport').exists():execute([BL,'-b',p/'source.blend','--python-exit-code','1','--python',T.parent/'constrained/reexport.py','--',p/'source-reexport'],R/(method+'-reexport.log'))
    assert sha(p/'asset.model')==sha(p/'source-reexport/asset.model')
    camera=R/'decode-camera.txt';camera.write_text('0 -84 0 420')
    execute([CFG['renderer'],CFG['cache'],p/'decode','render',p/'asset.model',camera,72,850],R/(method+'-decode.log'))
    g=json.loads((p/'geometry.json').read_text());d=json.loads((p/'decode/decoded.json').read_text())
    exact={k:g[k]==d[k] for k in ['vertices','faces','colors','render_types']};assert all(exact.values())
    components=json.loads((p/'components.json').read_text());names=[x['name'] for x in components]
    claws=[x for x in names if 'claw_'in x];toes=[x for x in names if 'toe_'in x]
    shields=[x for x in names if x.startswith('dorsal_plate_')or x.startswith('basalt_dorsal_shield_')]
    assert len(claws)==12 and len(toes)==12 and len(shields)==5
    top=geometry_audit.topology(g);assert top['degenerate_triangles']==0
    dims=(np.array(top['bounds'][1])-np.array(top['bounds'][0])).tolist()
    verification['models'][method]={'source_reexport_byte_exact':True,'native_decode_exact':exact,'model_sha256':sha(p/'asset.model'),'blend_sha256':sha(p/'source.blend'),'topology':top,'dimensions_xyz_authoring':dims,'explicit_claws':len(claws),'explicit_toes':len(toes),'primary_dorsal_shields':len(shields),'static_only':True,'conversion':json.loads((p/'conversion.json').read_text())}
    print('VERIFIED',method,top['triangles'],dims,flush=True)
verification['finished_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat()
(R/'verification.json').write_text(json.dumps(verification,indent=2)+'\n')
