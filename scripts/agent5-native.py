"""Reuse owned native persistence infrastructure with an exact maintained client."""
import argparse,hashlib,importlib.util,json,os,shutil,subprocess
from pathlib import Path
R=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('action',choices=['client-build','prepare','build','seed','produce','relogin']);p.add_argument('--target',choices=['main','contract'],default='contract');a=p.parse_args()
spec=importlib.util.spec_from_file_location('owned_native',R/'scripts/proving-ground-native.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
SHA={'main':'ec00738fbd97d9564c7711859fc509dae3d66a4e','contract':'cec5b1e125021b606371fc70488a67635b7213c9'}[a.target]
out=R/'docs/openrune/verification/independent/client-observer-build'
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
if a.action=='client-build':
 m.OUT=out;resource=m.guard();source=R.parent/'agent5-native-client';work=R/'work/current-client-build'
 for n in ['target','tmp','cargo-home']:(work/n).mkdir(parents=True,exist_ok=True)
 env=os.environ.copy();env.update(CARGO_HOME=str(work/'cargo-home'),CARGO_TARGET_DIR=str(work/'target'),TEMP=str(work/'tmp'),TMP=str(work/'tmp'),CARGO_BUILD_JOBS='1',PATH=str(m.TOOL)+os.pathsep+env['PATH'])
 cmd=[str(m.TOOL/'cargo.exe'),'build','--offline','--locked','-j','1','--example','pg_observer']
 if (out/'build.json').exists():
  previous=json.loads((out/'build.json').read_text());assert previous['status']=='FAIL','Never overwrite a successful frozen native build'
  index=1
  while (out/f'failed-attempt-{index:03d}').exists():index+=1
  history=out/f'failed-attempt-{index:03d}';history.mkdir(exist_ok=False)
  for n in ['build.json','build.log','source-manifest.json']:shutil.copyfile(out/n,history/n)
 with (out/'build.log').open('w',encoding='utf-8',newline='\n') as f:result=subprocess.run(cmd,cwd=source,env=env,stdout=f,stderr=subprocess.STDOUT,timeout=600,creationflags=subprocess.BELOW_NORMAL_PRIORITY_CLASS)
 binary=work/'target/debug/examples/pg_observer.exe'
 manifest={'status':'PASS' if result.returncode==0 else 'FAIL','exit_code':result.returncode,'client_sha':'32d8dcbbf5e527c0e5254b96b88c061fe35e8c50','resource':resource,'command':cmd,'observation_manifest_sha256':digest(out/'source-manifest.json'),'observer_sources':{'pg_observer.rs':digest(R/'verification/native-observer/pg_observer.rs')},'binaries':{'pg_observer.exe':digest(binary)} if binary.exists() else {}}
 (out/'build.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8',newline='\n')
 if result.returncode==0:
  compat=R/'work/independent-native-observer/target/debug/examples';compat.mkdir(parents=True,exist_ok=True);shutil.copyfile(binary,compat/'pg_observer.exe')
 print('CURRENT_CLIENT_BUILD',manifest['status']);raise SystemExit(result.returncode)
build=json.loads((out/'build.json').read_text());assert build['status']=='PASS'
m.SHA=SHA;m.CONTRACT_SHA='cec5b1e125021b606371fc70488a67635b7213c9';m.FEATURE='OR-MIG-0173';m.CHECKOUT=R.parent/'agent5-targets'/SHA;m.SCENARIO='spade';m.PIN='pg-observer';m.STATE='a5-spade-'+SHA[:12]+'-native-001';m.DRIVER='pg_observer-'+build['binaries']['pg_observer.exe'][:12]+'.exe';m.WORK=R/'work'/('native-spade-'+a.target)/'native-001';m.OUT=R/'docs/openrune/verification/independent'/('native-spade-'+SHA)/'native-001';m.RES=R/'work/native-spade-resources'
if a.action=='prepare':
 m.prepare();frozen=m.OUT/'frozen-observer';frozen.mkdir(exist_ok=False)
 for n in ['build.json','source-manifest.json']:shutil.copyfile(out/n,frozen/n)
 shutil.copyfile(R/'verification/native-observer/pg_observer.rs',frozen/'pg_observer.rs')
 shutil.copyfile(R/'scripts/proving-ground-native.py',m.OUT/'frozen-launcher.py');shutil.copyfile(Path(__file__),m.OUT/'scenario-launcher.py')
elif a.action=='build':m.build()
elif a.action=='seed':
 m.seed()
 # Explicit source-only fixture baseline, not a gameplay action or reward.
 import sqlite3
 with sqlite3.connect(m.dbpath()) as db:db.execute('UPDATE characters SET x=3209,z=3244,level=0,xp_rate_in_hundreds=500')
 m.save(m.OUT/'initial-state.json',m.snapshot())
else:
 assert digest(m.OUT/'frozen-launcher.py')==digest(R/'scripts/proving-ground-native.py'),'Freeze launcher across produce/relogin'
 m.phase('assemble' if a.action=='produce' else 'verify')
