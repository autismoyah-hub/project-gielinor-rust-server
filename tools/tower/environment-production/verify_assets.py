"""Saved-source conversion, fresh authoring and independent native codec checks."""
from pathlib import Path
import os,json,subprocess,hashlib,time
T=Path(__file__).resolve().parent;S=T.parents[2];E=S.parent;R=E/'cinderworks-environment-private';BL='C:/Program Files/Blender Foundation/Blender 5.2/blender.exe'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def execute(args,log,env=None):
    p=subprocess.run(list(map(str,args)),capture_output=True,text=True,env=env);log.parent.mkdir(parents=True,exist_ok=True);log.write_text(p.stdout+p.stderr);p.check_returncode()
def normalized(g):
    result=[]
    for i,f in enumerate(g['faces']):
        f=tuple(f);f=min(f[k:]+f[:k]for k in range(3));result.append((f,g['colors'][i],g['render_types'][i]))
    return g['vertices'],sorted(result)
if __name__=='__main__':
    out=R/'asset-validation';out.mkdir(exist_ok=True);catalog=json.loads((R/'package-v1/catalog.json').read_text());checks={}
    for name,asset in catalog.items():
        source=R/asset['source'];target=out/'reexport'/name
        execute([BL,'-b',source/'source.blend','--python-exit-code','1','--python',T.parent/'constrained/reexport.py','--',target],out/'logs'/(name+'-reexport.log'))
        assert sha(source/'asset.model')==sha(target/'asset.model'),(name,'saved source differs')
        decode=out/'decode'/name;cam=out/'decode.camera';cam.write_text('0 -84 0 420')
        execute([E/'astra-benchmark/tools/asset_workbench.exe',E/'live-client-cache-v1',decode,'render',source/'asset.model',cam,72,850],out/'logs'/(name+'-decode.log'))
        g=json.loads((source/'geometry.json').read_text());d=json.loads((decode/'decoded.json').read_text());fields={k:g[k]==d[k]for k in ['vertices','faces','colors','render_types']};assert all(fields.values())
        conversion=json.loads((source/'conversion.json').read_text());assert conversion['texture_image_samples']==0
        checks[name]={'saved_source_byte_exact':True,'decode_exact':fields,'model_sha256':sha(source/'asset.model'),'source_blend_sha256':sha(source/'source.blend'),'triangles':len(g['faces']),'source_conversion':conversion,'static_no_skin_labels':g['vertex_skins']is None};print('ASSET_SOURCE_AND_CODEC',name,flush=True)
    env=os.environ.copy();env['TOWER_CINDERWORKS_ASSET_OUTPUT']=str(out/'fresh')
    first=[name for name,a in catalog.items()if a['source'].startswith('kit-v1/')]
    for label,names in [('v1',first),('v2',['refined-fit']),('v3',['utility-clearance']),('v4',['finishing'])]:execute([BL,'-b','--factory-startup','--python-exit-code','1','--python',T/'kit.py','--',*names],out/'logs'/('fresh-'+label+'.log'),env)
    for name,asset in catalog.items():
        fresh=out/'fresh'/asset['source'];original=R/asset['source'];a=json.loads((original/'geometry.json').read_text());b=json.loads((fresh/'geometry.json').read_text());assert normalized(a)==normalized(b),(name,'fresh geometry/material difference')
        checks[name]['fresh_surface_and_materials_exact']=True;checks[name]['fresh_native_bytes_exact']=sha(fresh/'asset.model')==sha(original/'asset.model')
    report={'assets':checks,'all_saved_source_exports_exact':True,'all_native_decodes_exact':True,'all_fresh_oriented_surfaces_and_materials_exact':True,'all_fresh_native_bytes_exact':all(a['fresh_native_bytes_exact']for a in checks.values()),'note':'Fresh oriented surface/material equality is checked separately from byte equality; any Blender tessellation ordering difference is exposed rather than hidden.'}
    (R/'asset-verification.json').write_text(json.dumps(report,indent=2)+'\n');print('ASSET_GENERATION_VERIFIED',len(checks),'byte_exact',report['all_fresh_native_bytes_exact'],flush=True)
