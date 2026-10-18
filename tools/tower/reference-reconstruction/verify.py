"""Preservation, static source reproduction, codec and native pixel checks."""
from pathlib import Path
import sys,json,hashlib,subprocess,datetime
from PIL import Image,ImageChops
T=Path(__file__).resolve().parent
sys.path[:0]=[str(T),str(T.parent/'constrained')]
from run import R,E,BL,NATIVE,REF,execute,views
from evaluate import SELECTED
import geometry_audit
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def git(p,*args):return subprocess.check_output(['git',*args],cwd=p,text=True).strip()

baseline=json.loads((R/'baseline.json').read_text());p=R/'candidates'/SELECTED;out=R/('validation-'+SELECTED[:2]);out.mkdir(exist_ok=True)
assert all(sha(R/name)==digest for name,digest in baseline['preserved_files'].items())
assert sha(NATIVE)==baseline['native_workbench_sha256']
states={name:{'head':git(E/name,'rev-parse','HEAD'),'status':git(E/name,'status','--porcelain')}for name in baseline['other_workspaces']}
assert states==baseline['other_workspaces'],states
source_export=out/'saved-source-reexport'
execute([BL,'-b',p/'source.blend','--python-exit-code','1','--python',T.parent/'constrained/reexport.py','--',source_export],out/'source-reexport.log')
assert sha(p/'asset.model')==sha(source_export/'asset.model')
fresh=out/'fresh-build'
execute([BL,'-b','--factory-startup','--python-exit-code','1','--python',T/'build.py','--',R/'constraints.json',fresh,SELECTED[:2]],out/'fresh-build.log')
assert sha(p/'asset.model')==sha(fresh/'asset.model')
views(fresh/'asset.model',fresh/'native')
pixels={}
for name in ['front','side','rear','quarter','above','gameplay']:
    pixels[name]=ImageChops.difference(Image.open(p/'native'/(name+'.png')).convert('RGB'),Image.open(fresh/'native'/(name+'.png')).convert('RGB')).getbbox()is None
for name in ['front','left','rear','quarter']:
    pixels['reference/'+name]=ImageChops.difference(Image.open(p/'native/reference'/(name+'.png')).convert('RGB'),Image.open(fresh/'native/reference'/(name+'.png')).convert('RGB')).getbbox()is None
assert all(pixels.values())
g=json.loads((p/'geometry.json').read_text());d=json.loads((p/'native/reference/front/decoded.json').read_text())
decoded={k:g[k]==d[k]for k in ['vertices','faces','colors','render_types']};assert all(decoded.values())
assert g.get('vertex_skins')is None and g.get('face_skins')is None
conversion=json.loads((p/'conversion.json').read_text());assert conversion['texture_image_samples']==0
top=geometry_audit.topology(g);assert top['degenerate_triangles']==0
components=json.loads((p/'components.json').read_text());names=[x['name']for x in components]
counts={'toes':sum('_toe_'in x for x in names),'claws':sum('_claw_'in x for x in names),'dorsal_plates':sum(x.startswith('dorsal_shingle_')for x in names),'shoulder_plates':sum(x.startswith('shoulder_shingle_')for x in names)}
assert counts=={'toes':12,'claws':12,'dorsal_plates':5,'shoulder_plates':4}
report={'finished_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'selected':SELECTED,'model_sha256':sha(p/'asset.model'),'blend_sha256':sha(p/'source.blend'),
        'preserved_baseline_files':len(baseline['preserved_files']),'all_baseline_hashes_unchanged':True,'other_workspaces_unchanged':states,
        'native_workbench_sha256':sha(NATIVE),'camera_adapter_sha256':sha(REF),'saved_source_reexport_byte_exact':True,'fresh_generation_byte_exact':True,'native_decode_exact':decoded,'fresh_native_pixel_exact':pixels,
        'static_only_no_armature_or_actions':True,'no_vertex_or_face_skins':True,'source_image_texture_samples':0,'semantic_component_counts':counts,'topology':top,'conversion':conversion,
        'interpretation':'Technical reproduction and codec integrity pass. Separate shells and quantized seams are reported, not asserted watertight. These checks do not establish artistic fidelity or approval.'}
(R/'verification.json').write_text(json.dumps(report,indent=2)+'\n');print('VERIFIED',counts,'fresh build and 10 native views exact',flush=True)
(out/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
