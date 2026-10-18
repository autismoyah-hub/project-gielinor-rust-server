"""Sequential, measured Blender/native runs with preserved source snapshots."""
import json,sys,time,hashlib,subprocess,datetime,shutil
from pathlib import Path
T=Path(__file__).resolve().parent;E=T.parents[3];R=E/'concept-ab-private';TOOLS=T.parent
sys.path.insert(0,str(TOOLS/'reassessment'))
from compare import render
BL='C:/Program Files/Blender Foundation/Blender 5.2/blender.exe'
CFG={'renderer':str(E/'astra-benchmark/tools/asset_workbench.exe'),'cache':str(E.parents[1]/'resource-staging/server/work/integration/a-kotlin/.data/cache/js5')}
CASE={'height':84,'distance':420,'ambient':72,'contrast':850}
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def execute(cmd,log):
    start=time.monotonic();p=subprocess.run(list(map(str,cmd)),capture_output=True,text=True);Path(log).write_text(p.stdout+'\n'+p.stderr,encoding='utf8')
    if p.returncode:raise RuntimeError(str(log)+'\n'+p.stdout[-500:]+p.stderr[-1000:])
    return time.monotonic()-start
def main():
    method,stage,label,reason=sys.argv[1:];out=R/method/label
    if out.exists():raise SystemExit('No overwrites: '+str(out))
    out.parent.mkdir(exist_ok=True);recipe=R/method/(label+'.json')
    start=datetime.datetime.now(datetime.timezone.utc).isoformat()
    builder=T/('method_text.py' if method=='text' else 'method_concept.py')
    subprocess.run([sys.executable,str(builder),str(R/'design.json'),str(recipe),stage],check=True)
    started=time.monotonic()
    authoring=[T/'build_concept.py','--',recipe,out] if method=='concept' and int(stage)>=4 else [TOOLS/'constrained/surface.py','--','-',recipe,out]
    seconds=execute([BL,'-b','--factory-startup','--python-exit-code','1','--python',*authoring],R/method/(label+'-blender.log'))
    print('BLENDER',method,label,round(seconds,2),flush=True)
    native_start=time.monotonic();render(CFG,CASE,out/'asset.model',out/'native');native_seconds=time.monotonic()-native_start
    shutil.copy2(builder,out/'generation-snapshot.py')
    row={'method':method,'stage':int(stage),'candidate':label,'started_utc':start,'finished_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'blender_execution_seconds':seconds,'native_execution_seconds':native_seconds,'wall_generation_seconds':time.monotonic()-started,'reason':reason,'design_sha256':sha(R/'design.json'),'builder_sha256':sha(builder),'model_sha256':sha(out/'asset.model'),'blend_sha256':sha(out/'source.blend'),'native_views':6,'model_vertices':len(json.loads((out/'geometry.json').read_text())['vertices']),'model_triangles':len(json.loads((out/'geometry.json').read_text())['faces'])}
    (out/'iteration.json').write_text(json.dumps(row,indent=2)+'\n')
    with (R/'iterations.jsonl').open('a') as f:f.write(json.dumps(row)+'\n')
    print('NATIVE',method,label,round(native_seconds,2),'seconds',row['model_triangles'],'triangles',flush=True)
if __name__=='__main__':main()
