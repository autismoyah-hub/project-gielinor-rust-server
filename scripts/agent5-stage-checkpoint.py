"""Select closed source-only verification evidence; never collect runtime resources."""
import hashlib,json,py_compile,subprocess
from pathlib import Path
R=Path(__file__).resolve().parents[1];D=R/'docs/openrune/agent5'
selected=[R/n for n in ['OPENRUNE_VERIFICATION_MATRIX.csv','OPENRUNE_VERIFICATION_DEFECTS.md','OPENRUNE_NATIVE_ACCEPTANCE.md','OPENRUNE_VERIFICATION_PROGRESS.md']]
selected+=list(D.glob('*.json'))+list(D.glob('*.jsonl'))
selected+=list((R/'scripts').glob('agent5-*.py'))+[R/'scripts/proving-ground-integration-external.py',R/'scripts/proving-ground-native.py']
for folder in ['empty-search-reach','immediate-life','spade-independent','spade-life-tail','native-observer']:selected+=list((R/'verification'/folder).rglob('*.rs'))
selected.append(R/'docs/openrune/verification/USER_DEFERRALS.json')
for manifest in (R/'docs/openrune/verification/independent').glob('integration-external-*/manifest.json'):
 if json.loads(manifest.read_text())['status']=='IN_PROGRESS':continue
 selected += [p for p in manifest.parent.rglob('*') if p.is_file() and p.suffix in {'.json','.log','.rs','.err','.txt'}]
index=D/'EVIDENCE_INDEX.json';selected=sorted(set(p for p in selected if p!=index))
assert all(p.is_file() for p in selected)
for p in selected:
 if p.suffix in {'.log','.err'}:
  b=p.read_bytes().replace(b'\r\n',b'\n').rstrip();p.write_bytes(b+b'\n' if b else b'')
 elif p.suffix=='.csv':p.write_bytes(p.read_bytes().replace(b'\r\n',b'\n'))
for p in selected:
 if p.suffix!='.json':continue
 obj=json.loads(p.read_text());hashes=obj.get('evidence_sha256') if isinstance(obj,dict) else None
 if isinstance(hashes,dict):
  for name in hashes:hashes[name]=hashlib.sha256((R/name).read_bytes()).hexdigest()
  p.write_text(json.dumps(obj,indent=2)+'\n',encoding='utf-8',newline='\n')
for p in selected:
 assert p.suffix in {'.csv','.md','.json','.jsonl','.py','.rs','.log','.err','.txt'},p
 if p.suffix=='.py':py_compile.compile(str(p),doraise=True)
index.write_text(json.dumps({p.relative_to(R).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in selected},indent=2)+'\n',encoding='utf-8',newline='\n')
selected.append(index)
(R/'work/agent5-stage-paths.nul').write_bytes(b'\0'.join(p.relative_to(R).as_posix().encode() for p in selected)+b'\0')
print('Selected',len(selected),'source/evidence files; no DB,binary,cache,key,password,capture or image assets.')
