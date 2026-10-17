"""Check source-only publication and render the independent checkpoint manifest."""
import json
from pathlib import Path
import subprocess

ROOT=Path(__file__).resolve().parents[1]
DOC=ROOT/'docs/openrune/verification'
OUT=DOC/'independent'
# Match repository LF conventions. Text capture already normalized process output;
# remove Windows write-time newline translation and surplus terminal blank lines.
# Test messages, events and results are retained.
for path in [DOC/'VERIFICATION_RESULTS.jsonl',*OUT.rglob('*')]:
    if path.is_file() and path.suffix in ('.json','.jsonl','.txt','.log'):
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
summary={'scope':'Initial independent checkpoint; partial V0/V1 only',
         'consumed_contracts':len(latest),'latest_outcomes':{status:sum(r['outcome']==status for r in latest.values()) for status in ('VERIFIED','PARTIAL','FAILED','BLOCKED','NOT_ASSESSED')},
         'rust_test_functions_passed':sum(r.get('tests_passed',0) for r in rows if r['outcome']=='PARTIAL'),
         'historical_blocked_attempts':[r['attempt_id'] for r in rows if r['outcome']=='BLOCKED'],
         'required_open_levels':['V2','V3','V4'],'historical_FORGE_evidence_counted_as_independent':False}
(OUT/'checkpoint.json').write_text(json.dumps(summary,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(summary,indent=2))
