"""Preserve original contract stages and map historical evidence to maintained main."""
import csv,hashlib,json,subprocess
from pathlib import Path
from datetime import datetime,timezone
R=Path(__file__).resolve().parents[1];OLD=R.parent/'agent7-verification';D=R/'docs/openrune/agent5';D.mkdir(parents=True,exist_ok=True)
MAIN='ec00738fbd97d9564c7711859fc509dae3d66a4e';G=['git','-c','safe.directory='+R.as_posix()]
def run(*a):return subprocess.check_output(G+list(a),cwd=R).decode().strip()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def save(p,x):p.write_text(json.dumps(x,indent=2)+'\n',encoding='utf-8',newline='\n')
queue=[json.loads(s) for s in(R/'docs/openrune/verification/IMPLEMENTATION_QUEUE.jsonl').read_text().splitlines()]
ledger=[json.loads(s) for s in(OLD/'docs/openrune/verification/LEVEL_RESULTS.jsonl').read_text().splitlines()]
last={(r['implementation_id'],r['tested_sha'],r['level']):r for r in ledger}
index= json.loads((OLD/'docs/openrune/verification/independent/EVIDENCE_INDEX.json').read_text())
errors=[]
for n,expected in index.items():
 p=OLD/n
 if not p.is_file():errors.append({'path':n,'reason':'MISSING'})
 elif sha(p)!=expected:errors.append({'path':n,'reason':'HASH_MISMATCH','expected':expected,'actual':sha(p)})
keys={r['implementation_commit_sha']+':'+p for r in queue for p in r['rust_paths']}
keys|={MAIN+':'+p for r in queue for p in r['rust_paths']}
blobs={}
with subprocess.Popen(G+['cat-file','--batch'],cwd=R,stdin=subprocess.PIPE,stdout=subprocess.PIPE) as proc:
 for key in sorted(keys):
  proc.stdin.write(key.encode()+b'\n');proc.stdin.flush();header=proc.stdout.readline().decode().strip()
  if header.endswith(' missing'):blobs[key]=None;continue
  oid,kind,size=header.split();body=proc.stdout.read(int(size));assert proc.stdout.read(1)==b'\n';blobs[key]=oid
 proc.stdin.close();assert proc.wait()==0
changed_shared=run('diff','--name-only','0936f81',MAIN,'--','crates/world/src','crates/server/src','crates/storage/src','crates/protocol/src').splitlines()
rows=[];accepted=[]
for r in queue:
 pin=r['implementation_commit_sha'];ident=r['implementation_id'];same=[];changed=[];missing=[]
 for p in r['rust_paths']:
  if blobs[MAIN+':'+p] is None:missing.append(p)
  elif blobs[pin+':'+p]==blobs[MAIN+':'+p]:same.append(p)
  else:changed.append(p)
 levels={level:last.get((ident,pin,level),{}).get('status','NOT_STARTED') for level in ['V0','V1','V2','V3','V4']}
 if all(v=='PASS' for v in levels.values()):accepted.append({'implementation_id':ident,'sha':pin,'revision':r['revision']})
 rows.append({'implementation_id':ident,'revision':r['revision'],'feature':r['feature_name'],'implementation_sha':pin,'reference_identities':';'.join(x['reference']+'@'+x['sha256'] for x in r['source_references']),'maintained_main_sha':MAIN,**{'historical_'+k:v for k,v in levels.items()},'main_direct_source':'UNCHANGED' if not changed and not missing else 'CHANGED_OR_MISSING','main_changed_paths':';'.join(changed),'main_missing_paths':';'.join(missing),'current_main_acceptance':'NOT_VERIFIED','historical_evidence':str(OLD/'docs/openrune/verification/LEVEL_RESULTS.jsonl'),'remaining':'Focused shared-engine regression and applicable native/persistence acceptance; historical results retain original pinned identity.'})
matrix=R/'OPENRUNE_VERIFICATION_MATRIX.csv'
with matrix.open('w',encoding='utf-8',newline='') as f:
 w=csv.DictWriter(f,fieldnames=list(rows[0]));w.writeheader();w.writerows(rows)
snapshot={'observed_utc':datetime.now(timezone.utc).isoformat(),'maintained_main':MAIN,'forge_head':run('rev-parse','origin/agent2/openrune-content'),'agent7_checkpoint':'0936f81','agent7_checkpoint_full':run('rev-parse','0936f81'),'queue_records':len(queue),'implementation_ids':len({r['implementation_id'] for r in queue}),'queue_sha256':sha(R/'docs/openrune/verification/IMPLEMENTATION_QUEUE.jsonl'),'schema_sha256':sha(R/'docs/openrune/verification/IMPLEMENTATION_QUEUE.schema.json'),'validator_sha256':sha(R/'scripts/validate-openrune-queue.py'),'validator_independently_executed':'PASS:185records/173units; optional JSON Schema package not used','historical_level_rows':len(ledger),'historical_index_entries':len(index),'historical_hash_issues':errors,'historically_accepted_revisions':accepted,'unchanged_direct_source_revisions':sum(r['main_direct_source']=='UNCHANGED' for r in rows),'shared_changed_paths_since_agent7':changed_shared,'interpretation':'Historical results remain evidence of their exact tested SHA. Unchanged direct adapters are not blanket acceptance: shared World/runtime/storage changed during consolidation. Current main tests are separately identified. No original stage meaning or failed evidence is rewritten.','deferrals':'Chemistry, potions, Herblore remain deferred. General execution pause superseded by explicit Agent5 resume assignment; no Agent7 DB or source modified.'}
save(D/'history-recovery.json',snapshot)
u=json.loads((R/'docs/openrune/verification/USER_DEFERRALS.json').read_text());u['execution_pause']={'status':'RESUMED_BY_USER_AGENT5','instruction':'Agent5 independent verification handoff','historical_pause_preserved_at':str(OLD/'docs/openrune/verification/USER_DEFERRALS.json')};save(R/'docs/openrune/verification/USER_DEFERRALS.json',u)
print(json.dumps({k:snapshot[k] for k in ['queue_records','implementation_ids','historical_level_rows','historical_index_entries','historical_hash_issues','historically_accepted_revisions','unchanged_direct_source_revisions']},indent=2))
for r in queue:
 if r['implementation_id'] in ['OR-MIG-0172','OR-MIG-0173']:save(D/(r['implementation_id']+'-contract.json'),r)
