"""Check source-only publication and render the independent checkpoint manifest."""
import json
import hashlib
import py_compile
from pathlib import Path
import subprocess

ROOT=Path(__file__).resolve().parents[1]
DOC=ROOT/'docs/openrune/verification'
OUT=DOC/'independent'
for source in (ROOT/'scripts').glob('proving-ground*.py'):py_compile.compile(str(source),doraise=True)
# Match repository LF conventions. Text capture already normalized process output;
# remove Windows write-time newline translation and surplus terminal blank lines.
# Test messages, events and results are retained.
for path in [DOC/'VERIFICATION_RESULTS.jsonl',*OUT.rglob('*')]:
    if path.is_file() and path.suffix in ('.json','.jsonl','.txt','.log'):
        if path.name.startswith('queue-') and path.suffix=='.jsonl':continue  # Exact fetched blob identity.
        raw=path.read_bytes()
        path.write_bytes(raw.replace(b'\r\n',b'\n').rstrip(b'\n')+b'\n')
rows=[json.loads(s) for s in (DOC/'VERIFICATION_RESULTS.jsonl').read_text(encoding='utf-8').splitlines()]
assert len({r['attempt_id'] for r in rows})==len(rows)
latest={}
for row in rows:
    assert row['outcome'] in ('VERIFIED','PARTIAL','FAILED','BLOCKED','NOT_ASSESSED')
    assert row['independent_verification'] is True
    for path in row['evidence']:assert (ROOT/path).exists(),path
    latest[row['implementation_id']]=row
for path in OUT.glob('OR-MIG-*/manifest.json'):
    m=json.loads(path.read_text(encoding='utf-8'))
    if not m.get('source_tree'):continue
    checkout=ROOT.parent/'agent7-targets'/m['implementation_commit_sha']
    def git(*args):return subprocess.check_output(['git','-c','safe.directory='+checkout.as_posix(),*args],cwd=checkout,text=True).strip()
    assert git('rev-parse','HEAD')==m['implementation_commit_sha']
    assert git('rev-parse','HEAD^{tree}')==m['source_tree']
    assert not git('status','--porcelain','--untracked-files=no')
for path in OUT.rglob('build.json'):
    m=json.loads(path.read_text(encoding='utf-8'))
    target=m.get('tested_sha',m.get('implementation_sha'))
    if not target or not m.get('source_tree'):continue
    checkout=ROOT.parent/'agent7-targets'/target
    args=['git','-c','safe.directory='+checkout.as_posix()]
    assert subprocess.check_output([*args,'rev-parse','HEAD^{tree}'],cwd=checkout,text=True).strip()==m['source_tree']
    assert not subprocess.check_output([*args,'status','--porcelain','--untracked-files=no'],cwd=checkout,text=True).strip()
for path in OUT.glob('integration-*/manifest.json'):
    m=json.loads(path.read_text(encoding='utf-8'))
    if not m.get('source_tree'):continue
    checkout=ROOT.parent/'agent7-targets'/m['tested_integration_sha'];args=['git','-c','safe.directory='+checkout.as_posix()]
    assert subprocess.check_output([*args,'rev-parse','HEAD^{tree}'],cwd=checkout,text=True).strip()==m['source_tree']
    assert not subprocess.check_output([*args,'status','--porcelain','--untracked-files=no'],cwd=checkout,text=True).strip()
summary={'scope':'Coarse historical ledger summary only; use LEVEL_RESULTS.jsonl and WORKLIST.json for current V0-V4 and integration scope',
         'consumed_contracts':len(latest),'latest_outcomes':{status:sum(r['outcome']==status for r in latest.values()) for status in ('VERIFIED','PARTIAL','FAILED','BLOCKED','NOT_ASSESSED')},
         'rust_test_executions_passed':sum(r.get('tests_passed',0) for r in rows if r['outcome']=='PARTIAL'),
         'historical_blocked_attempts':[r['attempt_id'] for r in rows if r['outcome']=='BLOCKED'],
         'required_open_levels':['V2','V3','V4'],'historical_FORGE_evidence_counted_as_independent':False}
(OUT/'checkpoint.json').write_text(json.dumps(summary,indent=2)+'\n',encoding='utf-8',newline='\n')
state_file=DOC/'WORKLIST.json'
if state_file.exists():
    state=json.loads(state_file.read_text(encoding='utf-8'))
    snapshot=OUT/('queue-'+state['forge_head']+'.jsonl')
    assert hashlib.sha256(snapshot.read_bytes()).hexdigest()==state['queue_sha256']
    assert snapshot.read_bytes()==(DOC/'IMPLEMENTATION_QUEUE.jsonl').read_bytes()
    allowed={'NOT_STARTED','IN_PROGRESS','PASS','FAIL','BLOCKED','NOT_APPLICABLE'}
    assert len(state['contract_revisions'])==state['records']
    for row in state['contract_revisions']:
        assert set(row['levels'])=={'V0','V1','V2','V3','V4'}
        for level in row['levels'].values():assert level['status'] in allowed and level['reason'].strip()
    public_suffixes={'.json','.jsonl','.txt','.log','.err','.rs'}
    hashes={str(p.relative_to(ROOT)).replace('\\','/'):hashlib.sha256(p.read_bytes()).hexdigest() for p in OUT.rglob('*') if p.is_file() and p.suffix in public_suffixes and p.name!='EVIDENCE_INDEX.json'}
    (OUT/'EVIDENCE_INDEX.json').write_text(json.dumps(hashes,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(summary,indent=2))
