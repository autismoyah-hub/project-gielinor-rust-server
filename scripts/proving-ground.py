"""Independent exact-commit verification; never edits the implementation checkout."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import ctypes
import time
import re

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'docs/openrune/verification/independent'

def git(*args, root=ROOT, check=True):
    return subprocess.run(['git', '-c', 'safe.directory='+root.as_posix(), *args],
                          cwd=root, capture_output=True, text=True, encoding='utf-8', check=check)

def records():
    raw=git('show','origin/agent2/openrune-content:docs/openrune/verification/IMPLEMENTATION_QUEUE.jsonl').stdout
    return [json.loads(s) for s in raw.splitlines()]

def inspect():
    OUT.mkdir(parents=True, exist_ok=True)
    rows=records()
    queue_commit=git('rev-parse','origin/agent2/openrune-content').stdout.strip()
    snapshot={'queue_commit':queue_commit, 'contracts':rows,
              'timestamp_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'queue_sha256':hashlib.sha256(git('show',queue_commit+':docs/openrune/verification/IMPLEMENTATION_QUEUE.jsonl').stdout.encode()).hexdigest(),
              'historical_evidence_is_independent':False}
    (OUT/('contract-review-'+queue_commit+'.json')).write_text(json.dumps(snapshot,indent=2)+'\n')
    for r in rows:
        sha=r['implementation_commit_sha']
        ancestors=[]
        for dep in r['dependencies']:
            d=next((x for x in rows if x['implementation_id']==dep),None)
            if d:
                ok=git('merge-base','--is-ancestor',d['implementation_commit_sha'],sha,check=False).returncode==0
                ancestors.append(dep+':'+str(ok))
        print(r['implementation_id'],sha,r['feature_name'],','.join(ancestors))
        print(' NORMAL:',*r['expected_normal_behavior'])
        print(' NEGATIVE:',*r['expected_negative_behavior'])
        print(' TIMING:',*r['timing_and_transactions']['timing'])
        print(' LIMITS:',*r['known_limitations'])
    for folder in [Path('C:/Rust'),Path('C:/DevTools'),Path('C:/Projects/OSRSRSPS/DevTools'),Path('C:/Users/backup/.rustup'),Path('C:/Program Files/Rust'),Path('C:/tools')]:
        if folder.exists():
            for p in folder.rglob('cargo.exe'):
                print('CARGO:',p)
    print('PATH',os.environ.get('PATH'))

def run_batch(ids,independent=False):
    OUT.mkdir(parents=True,exist_ok=True)
    toolbin=Path('C:/Projects/OSRSRSPS/rust-client/work/rustup/toolchains/stable-x86_64-pc-windows-gnu/bin')
    filters={'OR-MIG-0018':'potion','OR-MIG-0006':'chemistry','OR-MIG-0010':'combat_mixing'}
    for ident in ids:
        if ident not in filters:raise ValueError('No reviewed suite mapping for '+ident+'; design a suite before claiming coverage')
        if independent and ident!='OR-MIG-0018':raise ValueError('External potion-slot scenarios are scoped only to OR-MIG-0018')
        r=next(x for x in reversed(records()) if x['implementation_id']==ident)
        sha=r['implementation_commit_sha']
        checkout=ROOT.parent/'agent7-targets'/sha
        evidence=OUT/(ident+'-'+sha[:12]+('-independent' if independent else ''))
        if evidence.exists():
            evidence=OUT/(evidence.name+'-attempt-'+str(time.time_ns()))
        evidence.mkdir(parents=True,exist_ok=True)
        try:
            git('cat-file','-e',sha+'^{commit}')
            for path in r['rust_paths']:git('cat-file','-e',sha+':'+path)
            for dep in r['dependencies']:
                if dep.startswith('ENV:'):continue
                other=next(x for x in reversed(records()) if x['implementation_id']==dep)
                git('merge-base','--is-ancestor',other['implementation_commit_sha'],sha)
            if not checkout.exists():git('worktree','add','--detach',str(checkout),sha)
        except subprocess.CalledProcessError as exc:
            blocked={'implementation_id':ident,'implementation_commit_sha':sha,'source_tree':None,
                     'outcome':'BLOCKED','independent_verification':True,'runs':[],
                     'reason':'Exact target or dependency unavailable; no substitute build: '+repr(exc.cmd)+' '+exc.stderr}
            (evidence/'manifest.json').write_text(json.dumps(blocked,indent=2)+'\n')
            print(json.dumps(blocked),flush=True);continue
        assert git('rev-parse','HEAD',root=checkout).stdout.strip()==sha
        assert not git('status','--porcelain','--untracked-files=no',root=checkout).stdout.strip()
        work=ROOT/'work'/sha
        env=os.environ.copy()
        env.update(CARGO_HOME=str(work/'cargo-home'),CARGO_TARGET_DIR=str(work/'target'),
                   TEMP=str(work/'tmp'),TMP=str(work/'tmp'),CARGO_BUILD_JOBS='1',
                   PATH=str(toolbin)+os.pathsep+env['PATH'])
        for folder in ('cargo-home','target','tmp'):(work/folder).mkdir(parents=True,exist_ok=True)
        proc=subprocess.run(['tasklist','/FO','CSV','/NH'],capture_output=True,text=True)
        active=[line for line in proc.stdout.splitlines() if line.lower().startswith(('"cargo.exe"','"rustc.exe"'))]
        class Memory(ctypes.Structure):
            _fields_=[('length',ctypes.c_ulong),('load',ctypes.c_ulong)]+[(x,ctypes.c_ulonglong) for x in ['total','avail','page','availpage','virtual','availvirtual','extended']]
        mem=Memory();mem.length=ctypes.sizeof(mem);ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(mem))
        manifest={'implementation_id':ident,'implementation_commit_sha':sha,
                  'source_tree':git('rev-parse','HEAD^{tree}',root=checkout).stdout.strip(),
                  'queue_commit':git('rev-parse','origin/agent2/openrune-content').stdout.strip(),
                  'independent_verification':True,'timestamp_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
                  'available_memory_bytes':mem.avail,'active_compilers':active,'runs':[]}
        manifest['rustc']=subprocess.run([str(toolbin/'rustc.exe'),'--version','--verbose'],env=env,capture_output=True,text=True,check=True).stdout
        if active or mem.avail<4*1024**3:
            manifest.update(outcome='BLOCKED',reason='Resource guard: active compiler or available memory below 4 GiB')
        else:
            manifests=[(p, p.read_text()) for p in checkout.glob('crates/*/Cargo.toml')]
            (evidence/'cargo-manifests.txt').write_text('\n'.join(str(p.relative_to(checkout))+'\n'+s for p,s in manifests))
            cmds=[['--version'],['test','--offline','--locked','-j','1','-p','gielinor-world','--lib',filters[ident]],
                  ['test','--offline','--locked','-j','1','-p','gielinor-server','--lib',filters[ident]]]
            if independent:
                harness=work/'independent-harness'
                harness.mkdir(exist_ok=True)
                shutil.copytree(ROOT/'verification/potion-slot/tests',harness/'tests',dirs_exist_ok=True)
                (harness/'Cargo.toml').write_text('[package]\nname="proving-ground-transactions"\nversion="0.1.0"\nedition="2021"\n[workspace]\n[dependencies]\ngielinor-world={path='+json.dumps((checkout/'crates/world').as_posix())+'}\n')
                # Generate a lock file solely for this external test crate.
                cmds=[['generate-lockfile','--offline','--manifest-path',str(harness/'Cargo.toml')],
                      ['test','--offline','--locked','-j','1','--manifest-path',str(harness/'Cargo.toml')]]
                manifest['harness_sha256']={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in (ROOT/'verification').rglob('*.rs')}
            for n,args in enumerate(cmds):
                cmd=[str(toolbin/'cargo.exe'),*args];start=time.monotonic()
                log=evidence/f'run-{n}.log'
                print(ident,'START',args,flush=True)
                try:
                    with log.open('w',encoding='utf-8') as f:
                        child=subprocess.Popen(cmd,cwd=checkout,env=env,stdout=f,stderr=subprocess.STDOUT,
                                               creationflags=subprocess.BELOW_NORMAL_PRIORITY_CLASS)
                        try:rc=child.wait(timeout=300)
                        except subprocess.TimeoutExpired:
                            subprocess.run(['taskkill','/PID',str(child.pid),'/T','/F'],capture_output=True)
                            child.wait();rc=124
                except OSError as exc:
                    log.write_text(str(exc),encoding='utf-8');rc=125
                content=log.read_text(encoding='utf-8')
                counts=[int(n) for n in re.findall(r'test result: ok\. (\d+) passed;',content)]
                manifest['runs'].append({'command':cmd,'exit_code':rc,'elapsed_seconds':round(time.monotonic()-start,2),'log':str(log.relative_to(ROOT)), 'tests_passed':sum(counts), 'zero_tests':'test' in args and not sum(counts)})
                (evidence/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
                print(ident,'END',rc,log.read_text(encoding='utf-8')[-6000:],flush=True)
                if rc:break
            manifest['outcome']='PARTIAL' if all(x['exit_code']==0 and not x['zero_tests'] for x in manifest['runs']) else 'BLOCKED'
            if any('test result: FAILED' in (ROOT/x['log']).read_text(encoding='utf-8') for x in manifest['runs']):
                manifest.update(outcome='FAILED',reason='Assertion failure; reproduce and classify before escalation')
            elif manifest['outcome']=='BLOCKED':
                manifest['reason']='Build/tool execution failed or no matching tests; inspect exact command logs before attributing a gameplay defect'
        assert git('rev-parse','HEAD',root=checkout).stdout.strip()==sha
        assert not git('status','--porcelain','--untracked-files=no',root=checkout).stdout.strip()
        manifest['test_binary_sha256']={str(p.relative_to(work)):hashlib.sha256(p.read_bytes()).hexdigest() for p in (work/'target/debug/deps').glob('*.exe')}
        (evidence/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
        print('MANIFEST',json.dumps(manifest),flush=True)

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--batch',nargs='+');p.add_argument('--independent',action='store_true');a=p.parse_args()
    if a.batch:run_batch(a.batch,a.independent)
    else:inspect()
