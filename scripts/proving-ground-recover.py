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
for item in items:
    if item['implementation_id']=='OR-MIG-0065' and (DOC/'independent/immediate-life-finding.json').exists():item['open_defects'].append('PG-DEF-003: exact65 immediate life refusal V1/V3FAIL; prior aggregate acceptance withdrawn; ordinary P2, native reachability/currentHEAD unproven')
    if item['implementation_id']=='OR-MIG-0067' and (DOC/'independent/battlestaff-life-finding.json').exists():item['open_defects'].append('PG-DEF-002: exact67authoritative life boundary V1/V3FAIL; ordinary P2, nonblocking, native dead-player reachability/currentHEAD unproven')
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
policy_path=DOC/'USER_DEFERRALS.json'
policy=json.loads(policy_path.read_text(encoding='utf-8')) if policy_path.exists() else None
deferred=set(policy['implementation_ids']) if policy and policy['status']=='ACTIVE_UNTIL_USER_RESUMES' else set()
if deferred:
    state['user_deferrals']=policy
    for item in items:item['execution_disposition']='DEFERRED_BY_USER' if item['implementation_id'] in deferred else 'ELIGIBLE'
    state['resume_queue']=[{'lane':'B','feature':'OR-MIG-0051','action':'Complete native selected burial, stale claimed slot, full inventory, real restart and supported reference comparison; chemistry/Herblore/potions deferred by user','sha':latest['OR-MIG-0051']['implementation_commit_sha']},{'lane':'B','feature':'OR-MIG-0023','action':'Continue native pack profiles and durable interrupted repeat work','sha':latest['OR-MIG-0023']['implementation_commit_sha']},{'lane':'A','feature':'OR-MIG-0054','action':'Initial exact-SHA coin-pouch quantity/overflow/selected stack verification','sha':latest['OR-MIG-0054']['implementation_commit_sha']}]
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
observer=DOC/'independent/deep-food-observer/observer-001/independent-assessment.json'
if observer.exists() and json.loads(observer.read_text(encoding='utf-8')).get('status')=='PASS':
    item=next(x for x in items if x['implementation_id']=='OR-MIG-0024');item['verification_blockers']=[];item['resolved_observation_blocker']='PG-INFRA-004: independent fresh six-action observer and real restart/relogin PASS; original missed equality remains inferred'
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
if deferred:
    resume['user_deferrals']=policy
    resume['remaining_deep_scenarios']=[x for x in resume['remaining_deep_scenarios'] if x['implementation_id'] not in deferred]
    resume['deferred_native_work']=resume.pop('next_native_work',None)
    next_ident='OR-MIG-0062' if (DOC/'independent/shell-acceptance.json').exists() and 'OR-MIG-0062' in latest else 'OR-MIG-0057'
    resume['next_native_work']={'implementation_id':next_ident,'target_sha':latest[next_ident]['implementation_commit_sha'],'feature':'glassblowing' if next_ident=='OR-MIG-0062' else 'shell-carving','actions_in_order':['complete independent authoritative scenarios','prepare fresh disposable native run','produce','relogin','independent assessment'],'reason':'User deferred chemistry. Preserve accepted historical manual gem/shell states. New shared glass choice/batch changes require their own exact-SHA tests; no acceptance inheritance.'}
    if (DOC/'independent/deep-glass-all/all-001/independent-assessment.json').exists():
        resume['next_native_work']={'implementation_id':'OR-MIG-0062','target_sha':'5ce50aede9164cdee8e4008257d0298ea77289e7','feature':'glass-interrupt','fresh_run_id':'interrupt-001','actions_in_order':['prepare/build only if absent','seed only if new database absent','produce once','relogin same state','independent assessment','then fresh glass-crash/crash-001','then native coin-pouch operations/deposit refusal at7fc1306b'],'reason':'Eight choices,1/5/10and All23 plus normal recovery completed. Do not repeat their immutable produce phases. Xblocked byPG-INFRA-009; continue independent batch lifecycle obligations.'}
    if (DOC/'independent/deep-glass-interrupt/interrupt-001/independent-assessment.json').exists():
        resume['next_native_work']={'implementation_id':'OR-MIG-0062','target_sha':'5ce50aede9164cdee8e4008257d0298ea77289e7','feature':'glass-crash','fresh_run_id':'crash-001','actions_in_order':['reuse prepared build/seed only if present','produce once','relogin same state','independent assessment','then native coin-pouch operations/deposit refusal at7fc1306b'],'reason':'Native partial All disconnect/restart independently passed with one committed product,875fineXP and22remaining raws; no replay. Preserve immutable state. X remains separately blocked.'}
    if (DOC/'independent/deep-glass-crash/crash-001/independent-assessment.json').exists():
        resume['next_native_work']={'implementation_id':'OR-MIG-0054','target_sha':'7fc1306bb5c01540038ce5ff175b577294ade432','feature':'pouch','fresh_run_id':'pouch-001','actions_in_order':['reuse prepared build/seed only if present','produce once','relogin same state','independent assessment'],'reason':'Glass disconnect and abrupt unacknowledged termination/restart scenarios independently passed. Native X remains scoped BLOCKED; continue eligible pouch operations and no_bank restriction.'}
    if (DOC/'independent/deep-pouch/pouch-001/independent-assessment.json').exists():
        resume['next_native_work']={'implementation_id':'OR-MIG-0054','target_sha':'7fc1306bb5c01540038ce5ff175b577294ade432','feature':'pouch','fresh_run_id':'pouch-002','actions_in_order':['require corrected observer build PASS','prepare/build/seed only when absent','produce once','relogin same state','independent assessment'],'reason':'Preserve pouch-001 native opening/stale/restart PASS_SUBCHECKS and raw bank-controller FAIL. PG-INFRA-010 immediate-packet assumption corrected via readonly post-send capture; fresh bank setup at same exact server SHA required.'}
    if (DOC/'independent/deep-pouch/pouch-002/independent-assessment.json').exists() or (DOC/'independent/deep-pouch/pouch-002/independent-blocker.json').exists():
        resume['next_native_work']={'implementation_id':'OR-MIG-0071','target_sha':'bc277dc3e7e15088323a4f048f57dc43185fb7c0','feature':'amethyst-counts','actions_in_order':['run prepared authoritative counted-output matrix','independently assess exact results','continue shared production upgrades OR63/69 at their own SHAs'],'reason':'Preserve completed pouch campaigns and all immutable glass states. Continue highest-risk eligible shared counted-credit/overflow and batch regressions; chemistry remains deferred.'}
    amethyst_matrix=DOC/'independent/integration-external-amethyst-counts-bc277dc3e7e15088323a4f048f57dc43185fb7c0/manifest.json'
    if amethyst_matrix.exists() and json.loads(amethyst_matrix.read_text())['status']=='PASS':
        resume['next_native_work']={'implementation_id':'OR-MIG-0063','target_sha':'143dab04f28c3daed844da0af33d6af5edcd962d','feature':'production-gems','actions_in_order':['inspect existing exact-SHA matrix manifests before launching','poll current owned campaign if running; never repeat PASS','complete actual-cache startup bindings and fresh native single-choice quantity scenarios','save/relogin/interrupted captured-batch and reference acceptance'],'reason':'OR71counted-credit7776+36cases and12pinned references passed within scope. Preserve all immutable manual56/57and glass/pouch results. New16-profile quantity upgrade needs independent acceptance; no inheritance to newer SHAs.'}
    production_crash=DOC/'independent/deep-production-gem-crash/crash-001/independent-assessment.json'
    if production_crash.exists() and json.loads(production_crash.read_text())['status']=='PASS':
        resume['next_native_work']={'implementation_id':'OR-MIG-0065','target_sha':'4f4aacb2929e8ad775d091527e2d80070100ee69','feature':'immediate-tools','actions_in_order':['retain completed207656resolver/60refusal/48World and12reference PASS scopes','execute remaining independent World admission/configuration cases only','attest actual-cache definitions/startup','prepare fresh native knife/branch and limestone scenarios at exact SHA','save/relogin/stale replay and aggregate applicable acceptance'],'reason':'OR63V0/V1/V3complete from16native recipes, both23product quantity campaigns, actual partial disconnects and owned-crash/restart. V2still BLOCKED by PG-INFRA-009even with real input prompt; preserve failures, do not force native quantities or substitute newer server. Continue eligible immediate content.'}
    immediate_acceptance=DOC/'independent/immediate-acceptance.json'
    if immediate_acceptance.exists() and not (DOC/'independent/immediate-acceptance-withdrawal.json').exists() and json.loads(immediate_acceptance.read_text())['status']=='ACCEPTED':
        resume['next_native_work']={'implementation_id':'OR-MIG-0064','target_sha':'0c489b46b552ad012e630810c1218dbaeb437a51','feature':'fallible-gems','actions_in_order':['preserve accepted OR65 and all immutable native/headless states','retain465408resolver/48refusal and93actual-cache probes; do not repeat','execute authoritative World mixed outcome/captured batch/late-stale scenarios','fresh native three-profile quantity/animation/mixed rewards; save/relogin and interrupted captured batch','retain PG-REF-001 reference conflicts and inherited custom-X dependency as separate unresolved obligations','continue OR66 manual tool crafting when this scope is blocked'],'reason':'OR65applicable V0-V4 completed at4f, including27normal native actions/restarts, explicit unsaved-result owned-crash rollback, authoritative admission/curve/zero-XP and deterministic queued integration regression. OR64fallible shared engine remains high risk and partly unverified; no historical acceptance inheritance or moving-HEAD substitution.'}
    fallible_partial=DOC/'independent/deep-fallible-gem-interrupt/interrupt-001/independent-assessment.json'
    if fallible_partial.exists() and json.loads(fallible_partial.read_text())['status']=='PASS':
        resume['next_native_work']={'implementation_id':'OR-MIG-0066','target_sha':'5582c276d1657a0142151936e5a9fec022d7c938','feature':'sinew','actions_in_order':['preserve OR64normal21native outputs/restart and partial saved2/940fineXP/21raw state; no repeat','retain6pinned sinew reference PASS subcheck','run independent authoritative manual World and resolver cases at5582','attest actual-cache three items/stat/sequence and startup','fresh disposable native both-direction/manual/no-quantity/interruption cases; actual save/relogin and applicable recovery','aggregate only satisfied V0-V4 obligations; continue next eligible contract'],'reason':'OR64V0PASS; controlled resolver/240World/93cache probes and21mixed native results plus genuine partial-batch recovery complete within scope. Required custom quantity andPG-REF-001adjudication remain unresolved; do not wait or force a whole-contract PASS. OR65earlier acceptance is withdrawn by PG-DEF-003life refusal failures; earlier successful evidence remains intact. New OR84world/life and generic stat regression scopes passed independently; current integration remains distinct.'}
    sinew_acceptance=DOC/'independent/sinew-acceptance.json'
    if sinew_acceptance.exists() and json.loads(sinew_acceptance.read_text())['status']=='ACCEPTED':
        resume['next_native_work']={'implementation_id':'OR-MIG-0067','target_sha':'e3ced82211ee438fd6a74a110d435e6746295a16','feature':'battlestaves','actions_in_order':['preserve OR66applicable V0-V4 acceptance and both completed native databases','retain OR67passed96resolver/192World/88late and107cache refusals plus11reference hashes','run legacy integer/immediate assembly regressions at exact67without chemistry','fresh native four-profile both-direction/manual animation/spot/output/XP cases','actual save/relogin, interruption and applicable recovery','aggregate only satisfied obligations, then immediately continue next eligible contract'],'reason':'OR66manual applicable V0-V4completed at5582with native movement cancellation, four selected products/3000fineXP, stale refusal, acknowledged save/actual restart and separate unsaved-result recovery/native restart. OR67shared precise reward API is next higher-risk unfinished content. Historical acceptance never inherits to moving FORGE HEAD.'}
    battlestaff_normal=DOC/'independent/deep-battlestaves/native-001/independent-assessment.json'
    if battlestaff_normal.exists() and json.loads(battlestaff_normal.read_text())['status']=='PASS':
        resume['next_native_work']={'implementation_id':'OR-MIG-0068','target_sha':'894646a79fa4ba7e3e8b46af6ec000dc6b796514','feature':'crab-carving','actions_in_order':['preserve OR66accepted and OR67eight native products/47500fineXP plus unsaved recovery and all databases','retain OR68completed310272roll subcheck, corrected30refusals,79cache refusals and13reference hashes','execute authoritative mixed success/broken batches and live/token/death cancellation at exact68','fresh native both-profile/both-direction/quantity/zero-XP failure and3tick effects','actual acknowledged save/relogin and partial/crash recovery','aggregate only satisfied obligations; continue next eligible contract'],'reason':'OR67V0/V2/V4PASS; PG-DEF-002at exact67makes V1/V3FAIL despite passing normal/native/crash persistence scopes. Ordinary P2 correction recorded locally; do not stop FORGE or silently replace historical target. Continue unrelated OR68staged outcomes/zero-XP failure and explicit Wiki cadence adaptation.'}
    crab_blocker=DOC/'independent/crab-custom-input-dependency.json'
    if crab_blocker.exists():
        resume['next_native_work']={'implementation_id':'OR-MIG-0069','target_sha':'fb348dda184b36ec2d503bfe0f1d5393cf8ae151','feature':'pair-production','actions_in_order':['preserve OR68V0/V1/V3/V4PASS and all completed disposable databases; V2custom input BLOCKED on PG-INFRA-009','retain39pinned reference facts for14stringing/4staff recipes','run corrected2592World pair/count/member/current/rate/trace-clock matrix at exact69; initial post-tick oracle failure and resource deferral remain immutable','run564late changes and legacy integer/immediate regressions; actual-cache/startup attestation','fresh native18recipe choices, builtin quantities/Close/stale frames and exact cadence; one owned native client at a time','save/relogin/partial/crash and aggregate only satisfied levels; proceed immediately to next eligible scenario'],'reason':'OR68independent310272rolls/30refusals/79cache refusals/288Worldcases and39normal native outputs/restarts plus0XPpartial/owned-crash recovery complete. Custom input lacks an attested client correction; no exact68Xfailure invented or newer server substituted. OR69changes actual startup/shared captured-pair scheduling and remains high risk.'}
    pair_normal=DOC/'independent/deep-pair-stringing/native-002/independent-assessment.json'
    if pair_normal.exists() and json.loads(pair_normal.read_text())['status']=='PASS':
        resume['next_native_work']={'implementation_id':'OR-MIG-0069','target_sha':'fb348dda184b36ec2d503bfe0f1d5393cf8ae151','feature':'pair-staves','run_id':'native-001','actions_in_order':['preserve OR69V1PASS4872World/128shared/421cache and all immutable original timeout evidence; do not repeat','preserve14native stringing products/2800fineXP and actual native restart; producerFAIL/124remains separate','pair-staves/native-001is already prepared,built,seeded with frozen626194f81f77client; produce once if no produce phase, then relogin and assess','after staff scope select first unfinished native builtin1/5/10/All batch, opposite stringing directions, partial save/crash case','pending new batch/reverse observer source requires build before fresh preparation; never overwrite old native state','retain native custom input dependency unless actual corrected client behavior is independently proven; aggregate only satisfied levels and continue'],'reason':'OR69applicable V1complete; real14stringing credit and normal durable subchecks PASS independently despite retained raw producer timeout. Staff and multi-pair native/persistence remain unfinished. OR65acceptance withdrawn PG-DEF-003; OR66accepted; OR68V2blocked PG-INFRA-009. Chemistry stays deferred.'}
    if pair_normal.exists():
        for feature in ['pair-staves','pair-counts','pair-interrupt','pair-crash','pair-earth-batch','pair-pending','pair-ten','pair-x','pair-stringing-reverse']:
            run=DOC/'independent'/('deep-'+feature)/'native-001'
            if (run/'independent-assessment.json').exists():continue
            phases={name:(json.loads((run/name/'manifest.json').read_text()).get('status') if (run/name/'manifest.json').exists() else 'NOT_STARTED') for name in ['produce','relogin']}
            resume['next_native_work']={'implementation_id':'OR-MIG-0069','target_sha':'fb348dda184b36ec2d503bfe0f1d5393cf8ae151','feature':feature,'run_id':'native-001','phase_status':phases,'actions_in_order':['preserve completed4872World/128shared/421cache and all native14stringing/8staff/13batch evidence; no replay','inspect this exact run manifests; wait any IN_PROGRESSowned session rather than launching another client','prepare/build/seed only missing fresh steps; never reseed an existing DB or rerun a completed produce phase','execute only the next missing produce,relogin,independent-assessment step; diagnose failures at this pin and keep originals','after this scoped result immediately select next unfinished native persistence/quantity scenario','remaining opposite stringing directions, staff multi-pair/fractional-save and custom-input dependency stay separate; aggregate only satisfied levels'],'reason':'OR69V0/V1PASS;18registered recipes and normal/native13pair batch plus restarts verified. Required interrupted/crash/remaining quantities still unfinished. Exact historical target only; chemistry stays deferred.'}
            break
    blocker=DOC/'independent/deep-glass-remaining/remaining-002/independent-blocker.json'
    reverse_done=DOC/'independent/deep-pair-stringing-reverse/native-001/independent-assessment.json'
    if reverse_done.exists():
        resume['next_native_work']={'implementation_id':'OR-MIG-0070','target_sha':'c7ddebe844820260252fb37df518f8c271f37243','feature':'crafting-finishes','actions_in_order':['preserve every completed OR69campaign/database; V0/V1/V3PASS, V2/V4remain partial','retain OR70corrected384World positives/refusals plus four confirmed wiring life failures and four passing helm life controls','execute OR70actual-cache factory/metadata refusal checks, then genuine native/manual normal and persistence scopes','record PG-DEF-004 ordinary correction locally; no FORGE stop or currentHEAD/native reachability inference','continue next eligible nonchemistry scenario after each batch'],'reason':'All prepared OR69native batches and opposite directions completed. Multi-digit X dependency and qualified reference comparison remain separate; OR70is eligible, with V1FAIL and unfinished cache/native/durability obligations.'}
    finishes_done=DOC/'independent/deep-crafting-finishes/native-001/independent-assessment.json'
    if finishes_done.exists():
        resume['next_native_work']={'implementation_id':'OR-MIG-0070','target_sha':'c7ddebe844820260252fb37df518f8c271f37243','feature':'crafting-finishes-crash','run_id':'native-001','actions_in_order':['preserve completed OR70eight native products/44800fineXP/actual reconnect and all immutable cases; V0/V2/V4PASS, V1FAILPG-DEF-004, V3partial','fresh crash scenario already prepared/built/seeded/frozen; inspect no produce phase before a single launch','resource guard refused prior launch before phase creation; retry produce only when original6GiB/70percent guard permits','after observed unsaved result/owned recovery injection, restart same frozen driver and independently assess full prior state/no replay','record ordinary wiring life correction locally and continue next eligible contract; chemistry deferred'],'reason':'Normal native state/reconnect and source comparison complete. Separate unsaved-result recovery remains an explicit resource-deferred test, not accepted or substituted at newerHEAD.'}
    finishes_disposition=DOC/'independent/crafting-finishes-disposition.json'
    if finishes_disposition.exists():
        feature='amethyst-batch' if (DOC/'independent/deep-amethyst/native-001/independent-assessment.json').exists() else 'amethyst'
        run=DOC/'independent'/('deep-'+feature)/'native-001'
        phases={name:(json.loads((run/name/'manifest.json').read_text()).get('status') if (run/name/'manifest.json').exists() else 'NOT_STARTED') for name in ['produce','relogin']}
        resume['next_native_work']={'implementation_id':'OR-MIG-0071','target_sha':'bc277dc3e7e15088323a4f048f57dc43185fb7c0','feature':feature,'run_id':'native-001','phase_status':phases,'actions_in_order':['OR70terminalFAILED:V0/V2/V3/V4PASS,V1FAILPG-DEF-004; preserve all campaigns and continue unrelated work','retain OR71resolver/World/guard/life/cache scopes and active count-one legacy result; no successful target replay','complete only missing native normal produce/relogin/assessment; wait any IN_PROGRESSowned session','next fresh amethyst-batch1/5/10/All7requires successfully built current observer before prepare; preserve frozen first campaign driver','after builtin batches/restarts execute remaining partial/unsaved recovery/effect/reference obligations; native custom-input dependencies remain explicit','aggregate only satisfied levels and keep chemistry deferred'],'reason':'Independent exact71counted-output path is the next eligible contract; no historical63acceptance or currentHEADinheritance. Existing queue and functioning runners reused.'}
    if (DOC/'independent/amethyst-durable-complete.json').exists():
        resume['next_native_work']={'implementation_id':'OR-MIG-0084','target_sha':'ad739676dd01f5be809cdfe5ecf4f1e6ea77238a','feature':'light-crash','run_id':'native-001','actions_in_order':['preserve OR71V0/V1/V3PASS and all normal/builtin/partial/recovery identities; custom input/audio unaccepted','preserve OR84native-00227actions/27stale refusals and real saved reconnect; native-001unknown scenario is a retained harness failure before lighting','finish actual nineteen-item/215refusal bindings and37legacyfactorycontrols; inspect current owned session before launch','execute missing exact84shared legacy World regression932cases','fresh light-crash requires successfully built observer matching controller, new disposable DB and frozen identity; do not reprepare existing campaign','complete Use/Cancel zero-target/no-mutation, unsaved lighting rollback and actual native restart; aggregate only satisfied V0–V4','continue next eligible nonchemistry contract immediately after disposition'],'reason':'OR71applicable durable obligations complete; dependency and reference limitations recorded. OR84manual routes have no custom quantity dependency and are eligible for closure; no successful campaign replay.'}
    if (DOC/'independent/light-sources-disposition.json').exists():
        resume['next_native_work']={'implementation_id':'OR-MIG-0108','target_sha':'7d4e09b6fdad766312ef55238e61c2730ea0b529','feature':'imcando-held','actions_in_order':['preserve OR84terminalFAILED V0/V2/V4PASS,V1/V3FAILPG-DEF-003 and all completed native/World evidence','run new independent exact108World selected/full/sparse/member/F2P/metadata/life/delay boundaries first','verify actual cache two forms/Swap op and startup before new genuine native held menus','native member/F2P UI, stale/session/save/recovery and pinned reference comparisons remain unexecuted; do not infer from headless membership policy','continue unrelated highest-priority eligible scenario after every batch; chemistry stays deferred'],'reason':'Exact84all levels have terminal disposition;72life failures are ordinary correction backlog, no FORGE stop.108manual held conversions have no quantity-input dependency and remain eligible;109worn and110Guild bank are independently untested.'}
    if blocker.exists():resume['blocked_scenarios']=[{'implementation_id':'OR-MIG-0062','target_sha':'5ce50aede9164cdee8e4008257d0298ea77289e7','scenario':'native X','finding':'PG-INFRA-009','reason':'Observed extra native keyboard/script continuation consumes dialog before intended choice. Requires correct focused-input dispatch or pinned client correction; no server defect established.','evidence':blocker.relative_to(ROOT).as_posix()}]
    bank_blocker=DOC/'independent/deep-pouch/pouch-002/independent-blocker.json'
    if bank_blocker.exists():resume.setdefault('blocked_scenarios',[]).append({'implementation_id':'OR-MIG-0054','target_sha':'7fc1306bb5c01540038ce5ff175b577294ade432','scenario':'native no_bank positive control','finding':'PG-INFRA-010','reason':'Actual Deposit-All menu offered but no normal interface47deposit submitted during60seconds; bank restriction not reached. Exact dispatch/content/mask cause unproven. Preserve opening/restart subchecks and continue other eligible work.','evidence':bank_blocker.relative_to(ROOT).as_posix()})
    production_x=DOC/'independent/deep-production-gem-x-focus/x10-focus-001/independent-blocker.json'
    if production_x.exists():resume.setdefault('blocked_scenarios',[]).append({'implementation_id':'OR-MIG-0063','target_sha':'143dab04f28c3daed844da0af33d6af5edcd962d','scenario':'native multi-digit X10with observed input prompt','finding':'PG-INFRA-009','reason':'Separate pointer frames and actual input prompt162:43do not prevent underlying270:15/count1while typing10. Native key/event routing versus observer/ordinary client loop remains a dependency; no server defect established. Preserve all passing X3/built-in/recovery scopes.','evidence':production_x.relative_to(ROOT).as_posix()})
    if crab_blocker.exists():resume.setdefault('blocked_scenarios',[]).append({'implementation_id':'OR-MIG-0068','target_sha':'894646a79fa4ba7e3e8b46af6ec000dc6b796514','scenario':'native custom-X','finding':'PG-INFRA-009','reason':'Required remaining custom input depends on unchanged pinned client focus/keyboard continuation. Exact68builtins1/5/10/All and39normal outputs/restarts pass; no exact68Xfailure is claimed. Preserve full V0/V1/V3/V4PASS, never force input or substitute newer server.','evidence':crab_blocker.relative_to(ROOT).as_posix()})
    resume.pop('native_food_observation_blocker',None)
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
