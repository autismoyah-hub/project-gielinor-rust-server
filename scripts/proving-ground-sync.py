"""Consume fetched contracts, independently validate, and append scoped evidence.

Fetch origin first. This script never advances an implementation test checkout.
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys
from datetime import datetime, timezone

ROOT=Path(__file__).resolve().parents[1]
DOC=ROOT/'docs/openrune/verification'
OUT=DOC/'independent'
OUT.mkdir(exist_ok=True)
def git(*args,check=True):
    return subprocess.run(['git','-c','safe.directory='+ROOT.as_posix(),*args],cwd=ROOT,capture_output=True,text=True,encoding='utf-8',check=check)
head=git('rev-parse','origin/agent2/openrune-content').stdout.strip()
rawbytes=subprocess.check_output(['git','-c','safe.directory='+ROOT.as_posix(),'show',head+':docs/openrune/verification/IMPLEMENTATION_QUEUE.jsonl'],cwd=ROOT)
raw=rawbytes.decode('utf-8')
rows=[json.loads(s) for s in raw.splitlines()]
spec=importlib.util.spec_from_file_location('validator',ROOT/'scripts/validate-openrune-queue.py')
validator=importlib.util.module_from_spec(spec);spec.loader.exec_module(validator)
validator.validate(rows,ROOT)
snapshot=OUT/('queue-'+head+'.jsonl')
if snapshot.exists():assert [json.loads(s) for s in snapshot.read_text().splitlines()]==rows
snapshot.write_bytes(rawbytes)
for old in OUT.glob('queue-*.jsonl'):
    oldrows=[json.loads(s) for s in old.read_text().splitlines()]
    if len(oldrows)<=len(rows):assert rows[:len(oldrows)]==oldrows,'Published queue history was rewritten'
original=git('rev-parse','ba455a0').stdout.strip()
legacy=git('show',original+':docs/openrune/verification/IMPLEMENTATION_QUEUE.jsonl').stdout
original_validator=OUT/'initial-validator.py'
original_queue=OUT/'initial-queue.jsonl'
original_validator.write_text(git('show',original+':scripts/validate-openrune-queue.py').stdout,encoding='utf-8')
original_queue.write_text(legacy,encoding='utf-8')
result=subprocess.run([sys.executable,str(original_validator),str(original_queue)],capture_output=True,text=True,encoding='utf-8',check=True)
oldspec=importlib.util.spec_from_file_location('initial_validator',original_validator)
oldvalidator=importlib.util.module_from_spec(oldspec);oldspec.loader.exec_module(oldvalidator)
oldvalidator.validate([json.loads(s) for s in legacy.splitlines()],ROOT)
checks={'queue_commit':head,'initial_contract_commit':original,'supplied_ba455a6_resolves':git('cat-file','-e','ba455a6^{commit}',check=False).returncode==0,
        'initial_validator_stdout':result.stdout,'current_validator_units':len(rows),
        'initial_validator_exact_sha_git_checks':'PASS',
        'timestamp_utc':datetime.now(timezone.utc).isoformat(),'independent_verification':True,
        'claim_scope':'Queue integrity only; not V0 compilation or gameplay acceptance',
        'dependency_ancestry':[]}
for r in rows:
    for dep in r['dependencies']:
        if dep.startswith('ENV:'):continue
        d=next(x for x in rows if x['implementation_id']==dep)
        ok=git('merge-base','--is-ancestor',d['implementation_commit_sha'],r['implementation_commit_sha'],check=False).returncode==0
        checks['dependency_ancestry'].append({'feature':r['implementation_id'],'dependency':dep,'present':ok})
        assert ok,(r['implementation_id'],dep)
(OUT/('sync-'+head+'.json')).write_text(json.dumps(checks,indent=2)+'\n')
ledger=DOC/'VERIFICATION_RESULTS.jsonl'
existing=[json.loads(s) for s in ledger.read_text().splitlines()] if ledger.exists() else []
keys={x['attempt_id'] for x in existing}
def append(row):
    if row['attempt_id'] in keys:return
    with ledger.open('a',encoding='utf-8') as f:f.write(json.dumps(row)+'\n')
    existing.append(row);keys.add(row['attempt_id'])
for r in rows:
    append({'attempt_id':'contract-intake-'+r['implementation_id']+'-'+r['implementation_commit_sha'],
            'implementation_id':r['implementation_id'],'implementation_commit_sha':r['implementation_commit_sha'],
            'outcome':'NOT_ASSESSED','level':'V0-V4','independent_verification':True,
            'coverage':['Contract integrity and dependency ancestry checked; gameplay not yet assessed'],
            'evidence':[str(snapshot.relative_to(ROOT))],'queue_commit':head,'issues':[]})
for path in sorted(OUT.glob('OR-MIG-*/manifest.json')):
    m=json.loads(path.read_text());runs=m.get('runs',[])
    if 'outcome' not in m:continue  # Never classify an in-progress attempt.
    for run in runs:
        content=(ROOT/run['log']).read_text(encoding='utf-8')
        run['tests_passed']=sum(map(int,re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed;',content)))
        run['zero_tests']='test' in run['command'] and 'running 0 tests' in content
    outcome=m.get('outcome','BLOCKED')
    if any(x['zero_tests'] for x in runs):outcome='BLOCKED'
    attempt_id=path.parent.name
    previous=next((r for r in existing if r['attempt_id']==attempt_id),None)
    if previous and (previous.get('tests_passed')!=sum(x['tests_passed'] for x in runs) or previous['outcome']!=outcome):
        attempt_id+='-completed'
    append({'attempt_id':attempt_id,'implementation_id':m['implementation_id'],
            'implementation_commit_sha':m['implementation_commit_sha'],'outcome':outcome,'level':'V0/V1 partial',
            'independent_verification':True,'source_tree':m['source_tree'],
            'coverage':['Independently executed existing implementation tests' if 'independent' not in path.parent.name else 'Independent external adversarial tests against unchanged production crate'],
            'tests_passed':sum(x['tests_passed'] for x in runs),'runs':runs,
            'reason':m.get('reason','Zero matching tests is not acceptance' if any(x['zero_tests'] for x in runs) else 'V2/V3/V4 and complete V0/V1 acceptance remain open'),
            'evidence':[str(path.relative_to(ROOT))],'issues':[]})
print(json.dumps(checks,indent=2))
print('Ledger entries:',len(existing))
