"""Run independent public-API scenarios at an explicit integration commit."""
import argparse,hashlib,importlib.util,json,os,re,shutil,subprocess,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('sha');p.add_argument('--suite',choices=['crafting-cape-death','crafting-cape-interrupt','crafting-cape-expiry-observe','crafting-cape-expiry','crafting-cape-bindings','crafting-cape-world','daeyalt-member-transition','daeyalt-boundaries-tail','daeyalt-boundaries','rune-close-ordering','daeyalt-context','daeyalt-ordinary-regression','daeyalt-live-guards','daeyalt-kernel-config','daeyalt-kernel','daeyalt-bindings','daeyalt-guards-tail','daeyalt-guards','daeyalt-world','rune-altars-boundaries','rune-altars-bindings','rune-altars-stock','rune-altars-guards','rune-altars-world','imcando-worn-duplicates','imcando-worn-interrupt','imcando-worn-listeners','imcando-worn-bindings','imcando-worn-refusals','imcando-worn-world','imcando-held-interrupt','imcando-held-refusals','imcando-held-bindings','imcando-held-world','light-life','light-legacy-bindings','light-tool-regression','light-bindings','member-tanning-guards','member-tanning-world','tanning-world','amethyst-policy','amethyst-boosts','amethyst-life','amethyst-legacy','amethyst-bindings','amethyst-guards','amethyst-world','crafting-finishes-refusals','crafting-finishes-admission','crafting-finishes-bindings','crafting-finishes-matrix','crafting-finishes-world','current-assembly-regression','energy-food','food-regeneration','assembly-batch','chemistry-matrix','prayer-cache','gem-matrix','gem-bindings','shell-matrix','shell-bindings','pouch-matrix','glass-matrix','glass-bindings','pouch-bindings','glass-admission','fallible-gem-matrix','amethyst-counts','production-gems','immediate-tools','immediate-bindings','immediate-admission','immediate-life','fallible-gem-bindings','immediate-regression','fallible-gem-world','light-source-world','light-conversion-world','sinew-matrix','sinew-bindings','battlestaff-matrix','battlestaff-bindings','battlestaff-regression','battlestaff-lifecycle','crab-matrix','crab-bindings','crab-admission','crab-world','crab-lifecycle','crab-counts','pair-production','pair-production-late','pair-production-admission','pair-production-regression','pair-production-bindings','pair-production-life','pair-production-counts','pair-production-counts-tail'],required=True);a=p.parse_args();assert re.fullmatch('[0-9a-f]{40}',a.sha)
policy=ROOT/'docs/openrune/verification/USER_DEFERRALS.json'
if policy.exists() and json.loads(policy.read_text(encoding='utf-8'))['status']=='ACTIVE_UNTIL_USER_RESUMES':assert a.suite!='chemistry-matrix','User deferred chemistry tests; select eligible unrelated suite'
out=ROOT/'docs/openrune/verification/independent'/('integration-external-'+a.suite+'-'+a.sha)
if out.exists():out=out.parent/(out.name+'-'+str(time.time_ns()))
out.mkdir(parents=True)
report={'tested_integration_sha':a.sha,'suite':a.suite,'campaign_kind':'integration','independent_verification':True,'status':'IN_PROGRESS'}
spec=importlib.util.spec_from_file_location('capacity',ROOT/'scripts/proving-ground-native.py');capacity=importlib.util.module_from_spec(spec);spec.loader.exec_module(capacity);capacity.OUT=out
try:report['resource']=capacity.guard()
except (AssertionError,subprocess.CalledProcessError) as exc:
    report.update(status='BLOCKED',reason='Measured-resource deferral; no substitute target: '+str(exc));(out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report));raise SystemExit(0)
checkout=ROOT.parent/'agent7-targets'/a.sha
if not checkout.exists():subprocess.run(['git','-c','safe.directory='+ROOT.as_posix(),'worktree','add','--detach',str(checkout),a.sha],cwd=ROOT,check=True)
def git(*args):return subprocess.check_output(['git','-c','safe.directory='+checkout.as_posix(),*args],cwd=checkout,text=True).strip()
assert git('rev-parse','HEAD')==a.sha and not git('status','--porcelain','--untracked-files=no')
report['source_tree']=git('rev-parse','HEAD^{tree}')
work=ROOT/'work'/a.sha
for name in ['target','tmp','cargo-home','external-harness']:(work/name).mkdir(parents=True,exist_ok=True)
harness=work/('external-harness-'+str(time.time_ns())[-10:]);shutil.copytree(ROOT/'verification'/a.suite/'tests',harness/'tests',dirs_exist_ok=True);shutil.copytree(harness/'tests',out/'harness')
(harness/'Cargo.toml').write_text('[package]\nname="proving-ground-energy"\nversion="0.1.0"\nedition="2021"\n[workspace]\n[dependencies]\ngielinor-world={path='+json.dumps((checkout/'crates/world').as_posix())+'}\n',encoding='utf-8')
env=os.environ.copy();env.update(CARGO_HOME=str(work/'cargo-home'),CARGO_TARGET_DIR=str(work/'target'),TMP=str(work/'tmp'),TEMP=str(work/'tmp'),PATH=str(capacity.TOOL)+os.pathsep+env['PATH'])
if a.suite in ['crafting-cape-bindings','daeyalt-bindings','rune-altars-bindings','imcando-worn-bindings','imcando-held-bindings','light-legacy-bindings','light-bindings','amethyst-bindings','crafting-finishes-bindings','prayer-cache','gem-bindings','shell-bindings','glass-bindings','pouch-bindings','immediate-bindings','fallible-gem-bindings','sinew-bindings','battlestaff-bindings','crab-bindings','pair-production-bindings']:
    with (harness/'Cargo.toml').open('a',encoding='utf-8') as f:f.write('gielinor-cache={path='+json.dumps((checkout/'crates/cache').as_posix())+'}\n')
    env.update(PG_GAME_CACHE=str(ROOT/'work/native-combat/resources/cache/game'),PG_SYMBOLS=str(ROOT/'work/native-combat/resources/symbols'))
    if a.suite in ['crafting-cape-bindings','daeyalt-bindings','rune-altars-bindings','imcando-worn-bindings','imcando-held-bindings','light-legacy-bindings','light-bindings','amethyst-bindings','crafting-finishes-bindings','gem-bindings','shell-bindings','glass-bindings','pouch-bindings','immediate-bindings','fallible-gem-bindings','sinew-bindings','battlestaff-bindings','crab-bindings','pair-production-bindings']:
        with (harness/'Cargo.toml').open('a',encoding='utf-8') as f:f.write('gielinor-server={path='+json.dumps((checkout/'crates/server').as_posix())+'}\n')
report['runs']=[]
for i,args in enumerate([['generate-lockfile','--offline'],['test','--offline','--locked','-j','1']]):
    cmd=[str(capacity.TOOL/'cargo.exe'),*args,'--manifest-path',str(harness/'Cargo.toml')];log=out/f'run-{i}.log'
    if a.suite in ['crafting-cape-death','crafting-cape-interrupt','crafting-cape-expiry-observe','crafting-cape-expiry','crafting-cape-bindings','crafting-cape-world','daeyalt-member-transition','daeyalt-boundaries-tail','daeyalt-boundaries','rune-close-ordering','daeyalt-context','daeyalt-ordinary-regression','daeyalt-live-guards','daeyalt-kernel-config','daeyalt-kernel','daeyalt-bindings','daeyalt-guards-tail','daeyalt-guards','daeyalt-world','rune-altars-boundaries','rune-altars-bindings','rune-altars-stock','rune-altars-guards','rune-altars-world','imcando-worn-duplicates','imcando-worn-interrupt','imcando-worn-listeners','imcando-worn-refusals','imcando-worn-world','imcando-held-interrupt','imcando-held-refusals','imcando-held-world','light-life','tanning-world','amethyst-bindings','crafting-finishes-bindings','crafting-finishes-world','prayer-cache','gem-bindings','shell-bindings','glass-bindings','pouch-bindings','immediate-bindings','fallible-gem-bindings','sinew-bindings','battlestaff-bindings','crab-bindings','pair-production','immediate-life','pair-production-counts','pair-production-counts-tail'] and args[0]=='test':cmd+=['--','--nocapture']
    print('INDEPENDENT_INTEGRATION',a.sha,args,flush=True)
    # Measured4480World pouch and2592World pair matrices require the600second bounded budget; retain prior timeout evidence.
    with log.open('w',encoding='utf-8',newline='\n') as f:rc=subprocess.run(cmd,cwd=checkout,env=env,stdout=f,stderr=subprocess.STDOUT,timeout=600 if a.suite in ['pouch-matrix','pair-production'] else 300,creationflags=subprocess.BELOW_NORMAL_PRIORITY_CLASS).returncode
    text=log.read_text(encoding='utf-8');count=sum(map(int,re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed;',text)))
    report['runs'].append({'command':cmd,'exit_code':rc,'tests_passed':count,'log':str(log.relative_to(ROOT))})
    if rc:break
report['status']='PASS' if all(x['exit_code']==0 for x in report['runs']) and report['runs'][-1]['tests_passed']>0 else 'FAIL'
report['scope']='Independent authoritative '+a.suite+' scenarios at explicit SHA; scenario evidence only, not complete contract acceptance'
report['harness_sha256']={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in (out/'harness').glob('*.rs')}
report['test_binary_sha256']={str(p.relative_to(work)):hashlib.sha256(p.read_bytes()).hexdigest() for p in (work/'target/debug/deps').glob('*.exe')}
assert git('rev-parse','HEAD')==a.sha and not git('status','--porcelain','--untracked-files=no')
(out/'manifest.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report),flush=True)
