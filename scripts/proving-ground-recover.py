"""Reconcile fetched FORGE records into per-level historical/integration work."""
import hashlib, importlib.util, json, subprocess, sys
from pathlib import Path
from datetime import datetime, timezone
ROOT=Path(__file__).resolve().parents[1]
DOC=ROOT/'docs/openrune/verification'
PRIVATE=ROOT/'work/recovery';PRIVATE.mkdir(parents=True,exist_ok=True)
def git(*args):return subprocess.check_output(['git','-c','safe.directory='+ROOT.as_posix(),*args],cwd=ROOT)
def save(path,value):path.write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8',newline='\n')
prior=json.loads((DOC/'WORKLIST.json').read_text(encoding='utf-8')) if (DOC/'WORKLIST.json').exists() else None
if '--fetched' not in sys.argv:
    subprocess.run(['git','-c','safe.directory='+ROOT.as_posix(),'fetch','origin','agent2/openrune-content'],cwd=ROOT,check=True)
head=git('rev-parse','origin/agent2/openrune-content').decode().strip()
q=git('show',head+':docs/openrune/verification/IMPLEMENTATION_QUEUE.jsonl')
rows=[json.loads(s) for s in q.decode().splitlines()]
latest={r['implementation_id']:r for r in rows}
for source in ['scripts/validate-openrune-queue.py','docs/openrune/verification/IMPLEMENTATION_QUEUE.schema.json']:
    (PRIVATE/Path(source).name).write_bytes(git('show',head+':'+source))
spec=importlib.util.spec_from_file_location('published',PRIVATE/'validate-openrune-queue.py');v=importlib.util.module_from_spec(spec);spec.loader.exec_module(v)
v.validate(rows,ROOT)
schema_status='NOT_EXECUTED'
try:
    import jsonschema
    schema=json.loads((PRIVATE/'IMPLEMENTATION_QUEUE.schema.json').read_text())
    for row in rows:jsonschema.Draft202012Validator(schema).validate(row)
    schema_status='PASS'
except ImportError:schema_status='UNAVAILABLE: jsonschema package; published validator PASS'
snap=DOC/'independent'/('queue-'+head+'.jsonl');snap.write_bytes(q)
copied=DOC/'IMPLEMENTATION_QUEUE.jsonl'
old_copy=[json.loads(s) for s in copied.read_text(encoding='utf-8').splitlines()]
assert rows[:len(old_copy)]==old_copy,'Published history no longer extends the retained copied queue'
copied.write_bytes(q)
historical=[json.loads(s) for s in (DOC/'VERIFICATION_RESULTS.jsonl').read_text(encoding='utf-8').splitlines()]
old={r['implementation_id'] for r in historical if r['attempt_id'].startswith('contract-intake')}
baseline_rows=[json.loads(s) for s in git('show','418d552ee5febc6bb7c93fe69e933c196efce045:docs/openrune/verification/VERIFICATION_RESULTS.jsonl').decode('utf-8').splitlines()]
baseline_ids={r['implementation_id'] for r in baseline_rows if r['attempt_id'].startswith('contract-intake')}
level_file=DOC/'LEVEL_RESULTS.jsonl'
events=[json.loads(s) for s in level_file.read_text(encoding='utf-8').splitlines()] if level_file.exists() else []
items=[]
for ident,r in latest.items():
    previous=[h for h in historical if h['implementation_id']==ident and h['implementation_commit_sha']==r['implementation_commit_sha'] and h['outcome']=='PARTIAL']
    levels={level:{'status':'IN_PROGRESS' if previous and level in ['V0','V1'] else 'NOT_STARTED','reason':'Historical partial checks retained; full level acceptance not established' if previous and level in ['V0','V1'] else 'Awaiting applicable independent campaign'} for level in ['V0','V1','V2','V3','V4']}
    for event in events:
        if event.get('implementation_id')==ident and event.get('tested_sha')==r['implementation_commit_sha'] and event.get('campaign_kind')=='historical':
            levels[event['level']]={k:event[k] for k in ['status','reason','evidence']}
    changed=git('diff','--name-only',r['implementation_commit_sha'],head,'--','crates/').decode().splitlines()
    integration={'sha':head,'status':'NOT_STARTED','levels':{k:'NOT_STARTED' for k in levels},'changed_production_paths':changed}
    prior_integration=[e for e in events if e.get('campaign_kind')=='integration' and ident in e.get('affected_implementations',[])]
    if prior_integration:
        last=prior_integration[-1]
        integration['last_executed_scope']={k:last[k] for k in ['tested_sha','level','status','reason','evidence']}
    for event in events:
        if event.get('campaign_kind')=='integration' and event.get('tested_sha')==head and ident in event.get('affected_implementations',[]):
            integration.update(status='IN_PROGRESS',reason=event['reason'],evidence=event['evidence'])
            integration['levels'][event['level']]=event['status']
    items.append({'implementation_id':ident,'revision':r['revision'],'feature_name':r['feature_name'],
                  'implementation_availability':{'status':'AVAILABLE','sha':r['implementation_commit_sha']},
                  'historical_verification':{'levels':levels,'evidence_attempts':[h['attempt_id'] for h in previous]},
                  'current_integration':integration,
                  'lane':'B' if previous else 'A','open_defects':(['PG-DEF-001 / GitHub123: historical native Close cancellation FAIL; ordinary P2, nonblocking'] if ident=='OR-MIG-0006' and any(e.get('implementation_id')==ident and e.get('status')=='FAIL' for e in events) else []),'reference_limitations':r['known_limitations']})
state={'observed_utc':datetime.now(timezone.utc).isoformat(),'forge_head':head,'queue_sha256':hashlib.sha256(q).hexdigest(),
       'published_validator_sha256':hashlib.sha256((PRIVATE/'validate-openrune-queue.py').read_bytes()).hexdigest(),
       'published_validator_status':'PASS','json_schema_status':schema_status,'records':len(rows),'implementation_units':len(latest),
       'new_contracts':sorted(set(latest)-({x['implementation_id'] for x in prior['items']} if prior else old)),
       'updated_contracts':[r['implementation_id'] for r in latest.values() if prior and any(x['implementation_id']==r['implementation_id'] and x['revision']<r['revision'] for x in prior['items'])],
       'new_since_initial_checkpoint':sorted(set(latest)-baseline_ids),'items':items,
        'resume_queue':[{'lane':'B','feature':'OR-MIG-0006','action':'Complete native negative/cancelled held and worn dialogs, last-charge production/modifiers and interrupted production persistence; retain completed positive controls/restart evidence','sha':latest['OR-MIG-0006']['implementation_commit_sha']},
                       {'lane':'C','feature':'OR-MIG-0006','action':'Inspect shared changes since the last recorded native integration campaign; select affected current-build regression scopes without inheriting older PASS results','sha':head},
                       {'lane':'A','feature':'OR-MIG-0024','action':'Expand runtime packet and genuine native ordinary/remainder food acceptance after completed revision-3 authoritative tests; item-pack native/lifecycle remains separate','sha':latest['OR-MIG-0024']['implementation_commit_sha']}]}
state['contract_revisions']=[]
for r in rows:
    levels={level:{'status':'NOT_STARTED','reason':'No complete level result at this exact contract revision'} for level in ['V0','V1','V2','V3','V4']}
    if any(h['implementation_id']==r['implementation_id'] and h['implementation_commit_sha']==r['implementation_commit_sha'] and h['outcome']=='PARTIAL' for h in historical):
        for level in ['V0','V1']:levels[level]={'status':'IN_PROGRESS','reason':'Historical partial evidence retained; no complete level acceptance inferred'}
    for e in events:
        if e.get('campaign_kind')=='historical' and e.get('implementation_id')==r['implementation_id'] and e['tested_sha']==r['implementation_commit_sha']:
            levels[e['level']]={k:e[k] for k in ['status','reason','evidence']}
    state['contract_revisions'].append({'implementation_id':r['implementation_id'],'revision':r['revision'],'implementation_sha':r['implementation_commit_sha'],'is_latest_revision':r['revision']==latest[r['implementation_id']]['revision'],'levels':levels})
food=DOC/'independent/deep-food/independent-assessment.json'
if food.exists() and json.loads(food.read_text(encoding='utf-8')).get('status')=='PARTIAL':
    item=next(x for x in state['items'] if x['implementation_id']=='OR-MIG-0024')
    item['verification_blockers']=[{'level':'V2','case_status':'BLOCKED','scope':'Remaining four bites and full six-action native client verdict',
      'reason':'Pinned food-001 driver times out after first cake action at a strict intermediate HP equality. Server commit and durable native state are verified; passive-regeneration collision is supported by an exact-SHA reproducer, but original production-session packet observation is insufficient for conclusive root cause.',
      'target_sha':item['implementation_availability']['sha'],'finding':'PG-INFRA-004','no_substituted_build':True}]
save(DOC/'WORKLIST.json',state)
sync_point=sys.argv[sys.argv.index('--sync-point')+1] if '--sync-point' in sys.argv else 'manual'
with (DOC/'SYNC_OBSERVATIONS.jsonl').open('a',encoding='utf-8',newline='\n') as f:
    f.write(json.dumps({'observed_utc':state['observed_utc'],'sync_point':sync_point,'forge_head':head,'queue_sha256':state['queue_sha256'],'records':len(rows),'units':len(latest),'validator':'PASS','new_contracts':state['new_contracts'],'updated_contracts':state['updated_contracts'],
        'production_changes_since_previous_observation':git('diff','--name-only',prior['forge_head'],head,'--','crates/').decode().splitlines() if prior else []})+'\n')
resume={'observed_forge_head':head,'queue_sha256':state['queue_sha256'],'lanes':state['resume_queue'],
        'next_historical_native':{'implementation_id':'OR-MIG-0006','target_sha':latest['OR-MIG-0006']['implementation_commit_sha'],'feature':'chemistry-worn','fresh_run_id':'hist-worn-002',
          'actions_in_order':['prepare','build','seed','produce','relogin'],'command_template':'python scripts/proving-ground-native.py ACTION --feature chemistry-worn --run-id hist-worn-002',
          'assessment_command':'python scripts/proving-ground-assess-worn.py '+latest['OR-MIG-0006']['implementation_commit_sha']+' --directory docs/openrune/verification/independent/deep-chemistry-worn/hist-worn-002',
          'reason':'Worn native checks on integration SHA8ad59b1 do not certify the earlier exact contract SHA'},
        'remaining_deep_scenarios':[
          {'implementation_id':'OR-MIG-0010','target_sha':latest['OR-MIG-0010']['implementation_commit_sha'],'cases':['native eligibility/stale selected ingredient failures','native movement/interrupt and partial production logout','native modifier conservation','applicable interrupted-transaction durable recovery']},
          {'implementation_id':'OR-MIG-0006','target_sha':latest['OR-MIG-0006']['implementation_commit_sha'],'cases':['native stale/cancelled held and worn dialogs','native last-charge production and modifier coexistence','interrupted production/logout state','finish full reference modifier comparison']},
          {'implementation_id':'OR-MIG-0024','target_sha':latest['OR-MIG-0024']['implementation_commit_sha'],'cases':['actual runtime claimed-ID packet decode','genuine native ordinary/overheal/remainder interactions','durable food and multipart recovery','source/pinned documentation comparison for current revision']},
          {'implementation_id':'OR-MIG-0023','target_sha':latest['OR-MIG-0023']['implementation_commit_sha'],'cases':['independent native pack profiles and full inventory','real save/restart interrupted repeat queue','independent recipe/reference comparison']}],
        'unverified_features':[{k:x[k] for k in ['implementation_id','revision','implementation_availability','historical_verification','current_integration']} for x in items]}
worn=DOC/'independent/deep-chemistry-worn/hist-worn-002/independent-assessment.json'
if worn.exists() and json.loads(worn.read_text(encoding='utf-8')).get('status')=='PASS':
    resume['completed_historical_native']=resume.pop('next_historical_native')
    resume['completed_historical_native'].update(status='PASS_SUBCHECKS_ONLY',evidence=str(worn.relative_to(ROOT)).replace('\\','/'))
    resume['next_native_work']={'implementation_id':'OR-MIG-0006','target_sha':latest['OR-MIG-0006']['implementation_commit_sha'],
      'cases':['cancelled and stale held/worn dialog replay','last-charge production and modifier coexistence','interrupted/partial production logout and recovery'],
      'setup':'Audit matching pinned native driver source or build an owned independent client scenario; prepare a new disposable state/run ID, never reseed or overwrite completed campaign state.',
      'existing_campaigns_requiring_no_rerun':['deep-chemistry','deep-chemistry-worn/hist-worn-002']}
for ident in ['OR-MIG-0034','OR-MIG-0035']:
    if ident in latest:
        resume['remaining_deep_scenarios'].append({'implementation_id':ident,'target_sha':latest[ident]['implementation_commit_sha'],
          'cases':['independently inspect protected source and pinned documentation','verify complete configured profile bindings and member gate','genuine native actions and durable recovery','retain partial authoritative evidence without inheriting newer-source acceptance']})
if food.exists():
    resume['native_food_observation_blocker']={'implementation_id':'OR-MIG-0024','target_sha':latest['OR-MIG-0024']['implementation_commit_sha'],
      'finding':'PG-INFRA-004','completed':'Meat client confirmation; cake server commit; real saved inventory/XP restart and independently decoded native relogin capture.',
      'next':'Use a new owned observer recording every received HP and inventory update plus script-busy state; correlate exact tick/trace and passive regeneration. Run remaining cake and pizza stages on a fresh disposable run. Preserve raw failed driver verdict; no newer source substitution.',
      'do_not_rerun':'Existing deep-food/produce and relogin are immutable evidence, and their database is post-consumption state.'}
if 'OR-MIG-0040' in latest:
    resume['remaining_deep_scenarios'].append({'implementation_id':'OR-MIG-0040','target_sha':latest['OR-MIG-0040']['implementation_commit_sha'],
      'priority':'HIGH shared assembly change','cases':['captured selected/other material slot refusal','third container-return failure atomicity','modal-open cancellation and no reward after closing','existing pair recipe regression','native full-bag four-input batter and interrupted save/restart']})
save(DOC/'RESUME_QUEUE.json',resume)
save(DOC/'independent'/('intake-'+head+'.json'),{k:value for k,value in state.items() if k!='items'})
print(json.dumps({k:value for k,value in state.items() if k not in ['items','resume_queue','contract_revisions']},indent=2))
if '--research' in sys.argv:
    base=ROOT.parent/'rust-server'
    for p in (base/'scripts').iterdir():
        if any(x in p.name for x in ['finished','combat','native','rust-login','state']):print('SCRIPT',p.name)
    for filename in ['integration-rust-login.ps1','integration-preparation-state.py']:
        p=base/'scripts'/filename
        print('\nFILE',filename,'\n',p.read_text(encoding='utf-8')[:42000])
