"""Run shared-world regressions against a recent immutable integration target."""
import hashlib,importlib.util,json,os,re,subprocess,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
DOC=ROOT/'docs/openrune/verification'
state=json.loads((DOC/'WORKLIST.json').read_text(encoding='utf-8'))
SHA=state['forge_head'];checkout=ROOT.parent/'agent7-targets'/SHA
OUT=DOC/'independent'/('integration-'+SHA)
OUT.mkdir(parents=True,exist_ok=False)
spec=importlib.util.spec_from_file_location('native',ROOT/'scripts/proving-ground-native.py');native=importlib.util.module_from_spec(spec);spec.loader.exec_module(native)
resources=native.guard()
def git(*args):return subprocess.check_output(['git','-c','safe.directory='+ROOT.as_posix(),*args],cwd=ROOT,text=True).strip()
if not checkout.exists():git('worktree','add','--detach',str(checkout),SHA)
assert subprocess.check_output(['git','-c','safe.directory='+checkout.as_posix(),'rev-parse','HEAD'],cwd=checkout,text=True).strip()==SHA
work=ROOT/'work'/SHA
for name in ['cargo-home','target','tmp']:(work/name).mkdir(parents=True,exist_ok=True)
env=os.environ.copy();env.update(CARGO_HOME=str(work/'cargo-home'),CARGO_TARGET_DIR=str(work/'target'),TEMP=str(work/'tmp'),TMP=str(work/'tmp'),CARGO_BUILD_JOBS='1',PATH=str(native.TOOL)+os.pathsep+env['PATH'])
report={'tested_integration_sha':SHA,'source_tree':subprocess.check_output(['git','-c','safe.directory='+checkout.as_posix(),'rev-parse','HEAD^{tree}'],cwd=checkout,text=True).strip(),
        'resource_measurement':resources,'implementation_availability_is_not_integration_acceptance':True,'runs':[]}
for filter in ['potion','chemistry','combat_mixing','inventory','queues']:
    cmd=[str(native.TOOL/'cargo.exe'),'test','--offline','--locked','-j','1','-p','gielinor-world','--lib',filter]
    log=OUT/(filter+'.log');start=time.monotonic();print('REGRESSION',filter,SHA,flush=True)
    with log.open('w',encoding='utf-8',newline='\n') as f:
        rc=subprocess.run(cmd,cwd=checkout,env=env,stdout=f,stderr=subprocess.STDOUT,timeout=300,creationflags=subprocess.BELOW_NORMAL_PRIORITY_CLASS).returncode
    text=log.read_text(encoding='utf-8');counts=[int(n) for n in re.findall(r'test result: ok\. (\d+) passed;',text)]
    report['runs'].append({'filter':filter,'command':cmd,'exit_code':rc,'tests_passed':sum(counts),'elapsed_seconds':round(time.monotonic()-start,2)})
    (OUT/'manifest.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n')
    print('RESULT',filter,rc,sum(counts),flush=True)
    if rc:break
report['status']='PASS' if all(r['exit_code']==0 and r['tests_passed']>0 for r in report['runs']) else 'FAIL'
report['scope']='Shared authoritative world regression filters only; no latest-native or complete-contract acceptance claim'
report['binary_sha256']={str(p.relative_to(work)):hashlib.sha256(p.read_bytes()).hexdigest() for p in (work/'target/debug/deps').glob('*.exe')}
(OUT/'manifest.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(report),flush=True)
