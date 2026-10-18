"""Append per-level evidence; retain historical, integration and intake separately."""
import hashlib,json
from pathlib import Path
from datetime import datetime,timezone
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification';OUT=DOC/'independent'
file=DOC/'LEVEL_RESULTS.jsonl'
rows=[json.loads(s) for s in file.read_text(encoding='utf-8').splitlines()] if file.exists() else []
keys={r['attempt_id'] for r in rows}
def add(ident,sha,level,status,reason,evidence,kind='historical',scope='deep-acceptance',affected=None):
    attempt=scope+'-'+ident+'-'+sha+'-'+level
    if attempt in keys:return
    for path in evidence:assert (ROOT/path).exists(),path
    row={'attempt_id':attempt,'implementation_id':ident,'tested_sha':sha,'level':level,'status':status,
         'campaign_kind':kind,'reason':reason,'evidence':evidence,'observed_utc':datetime.now(timezone.utc).isoformat(),
         'independent_verification':True}
    if affected:row['affected_implementations']=affected
    with file.open('a',encoding='utf-8',newline='\n') as f:f.write(json.dumps(row)+'\n')
    rows.append(row);keys.add(attempt)
for ident,folder,sha in [('OR-MIG-0010','deep-combat','5a260c1bd6efa5f8a221bed7c96e6ac72fd428bc'),('OR-MIG-0006','deep-chemistry','e7ac3608e61893d80b2de6a1a2e512b3ab785505')]:
    assessment=OUT/folder/'independent-assessment.json'
    if not assessment.exists():continue
    report=json.loads(assessment.read_text(encoding='utf-8'));evidence=[str(assessment.relative_to(ROOT)).replace('\\','/')]
    add(ident,sha,'V0','PASS','Exact-SHA server compiled; full runtime startup and cache-backed definitions succeeded; native actions reached configured gameplay dispatch.',evidence)
    add(ident,sha,'V1','IN_PROGRESS','Historical headless checks and native positive transitions passed; complete independently specified authoritative negative/modifier matrix remains open.',evidence)
    add(ident,sha,'V2','IN_PROGRESS','Genuine programmatic native menu/action/packet and authoritative-state subchecks PASS. Native stale/interruption/modifier cases and any applicable rendered acceptance remain open.',evidence)
    if report.get('real_restart_and_native_relogin')=='PASS':
        add(ident,sha,'V3','IN_PROGRESS','Actual acknowledged-save, process restart, native relogin and durable inventory/XP/state checks PASS; interrupted/partial production and any applicable crash-recovery scenarios remain open.',evidence)
    if folder=='deep-chemistry' or (OUT/folder/'reference-comparison.json').exists():
        refs=evidence+([str((OUT/folder/'reference-comparison.json').relative_to(ROOT)).replace('\\','/')] if folder=='deep-combat' else [])
        add(ident,sha,'V4','IN_PROGRESS','Independent protected-source and pinned-documentation subcomparisons PASS within recorded scope; complete feature comparison remains open. No executable donor parity is claimed.',refs)
for manifest in OUT.glob('OR-MIG-0024-*-independent*/manifest.json'):
    value=json.loads(manifest.read_text(encoding='utf-8'))
    if value.get('outcome')!='PARTIAL':continue
    sha=value['implementation_commit_sha'];evidence=[str(manifest.relative_to(ROOT)).replace('\\','/')]
    add('OR-MIG-0024',sha,'V0','IN_PROGRESS','Exact-SHA production crate compiled; runtime cache/registration and packet decoder acceptance remain open.',evidence)
    add('OR-MIG-0024',sha,'V1','IN_PROGRESS','Four independent authoritative World action tests PASS across 319 fixtures: all selected slots, full bags, duplicate/stale input, overheal, remainders and malformed transactions. Runtime packet decode and complete food matrix remain open.',evidence)
for manifest in OUT.glob('integration-*/manifest.json'):
    value=json.loads(manifest.read_text(encoding='utf-8'))
    if value.get('status')!='PASS':continue
    if value.get('suite')=='gem-bindings':
        add('OR-MIG-0056',value['tested_integration_sha'],'V0','IN_PROGRESS','Independent actual-cache15item/seven-profile binding and159corrupt object/symbol/stat/effect refusals PASS; native/runtime aggregate acceptance is recorded separately.',[str(manifest.relative_to(ROOT)).replace('\\','/')],scope='independent-gem-bindings')
        continue
    if value.get('suite')=='gem-matrix':
        add('OR-MIG-0056',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent seven-profile matrix PASS across3528 all-selected-slot/current-level/member/rate fixtures and126 invalid captured-input/tool/policy/credit cases. Full-bag exact selected replacement, retained duplicate chisels/raws and fractional fineXP verified; native route, pending-action interruptions and lifecycle/reference remain separate.',[str(manifest.relative_to(ROOT)).replace('\\','/')],scope='independent-gem-matrix')
        continue
    if value.get('suite')=='prayer-cache':
        add('OR-MIG-0051',value['tested_integration_sha'],'V0','IN_PROGRESS','Independent read-only target-cache definition and effective Prayer reward preflight PASS for representative bones; exact source and source-cache decoder identities retained. Full native dispatch/registration and remaining profile acceptance remain separate.',[str(manifest.relative_to(ROOT)).replace('\\','/')],scope='prayer-loaded-policy')
        continue
    if value.get('suite')=='chemistry-matrix':
        add('OR-MIG-0006',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent 22000-combination public completion matrix and12 invalid-policy cases PASS: both input directions, five charge counts, both crumble policies, goggles coexistence, all random thresholds, full inventory and exact XP/level/slot atomicity. Native and complete action-route obligations remain separate.',[str(manifest.relative_to(ROOT)).replace('\\','/')],scope='chemistry-independent-matrix')
        continue
    if value.get('suite')=='assembly-batch':
        for level in ['V0','V1']:
            add('OR-MIG-0040',value['tested_integration_sha'],level,'IN_PROGRESS','Independent public authoritative BatchRecipe/BatchJob transaction matrix PASS across1512 full-bag/selected-slot/direction/late-failure combinations, including stale material with untouched matching duplicate, all three refused returns, live level, positive-XP invalid rate, repeated captured slot, membership and replay. Actual World held-route/pair/modal regressions passed separately; configured runtime, native, persistence and full reference acceptance remain open.',[str(manifest.relative_to(ROOT)).replace('\\','/')],scope='assembly-batch-independent')
        continue
    if value.get('suite')=='food-regeneration':
        add('OR-MIG-0024',value['tested_integration_sha'],'V1','IN_PROGRESS','Exact-SHA independent timer/food diagnosis PASS: actual queued cake commits HP5->9, followed by passive regeneration to10 without more food or XP. This supports, but does not fully prove, the native strict-equality oracle diagnosis. Earlier rejected same-clock hypothesis and failed test are retained.',[str(manifest.relative_to(ROOT)).replace('\\','/')],scope='food-regen-diagnosis')
        continue
    if value.get('suite')=='energy-food':
        add('SHARED-FOOD-ENERGY',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent food-energy authoritative queue matrix PASS: successful HP/stat/energy/remainder publication once; failed transaction energy exactly matches no-action controls. Integration scope only; associated new contract/native/persistence/reference coverage remains open.',
            [str(manifest.relative_to(ROOT)).replace('\\','/')],kind='integration',scope='independent-food-energy',affected=['OR-MIG-0024'])
        state=json.loads((DOC/'WORKLIST.json').read_text(encoding='utf-8'))
        published=next((x for x in state['items'] if x['implementation_id']=='OR-MIG-0034' and x['implementation_availability']['sha']==value['tested_integration_sha']),None)
        if published:
            for level in ['V0','V1']:
                add('OR-MIG-0034',value['tested_integration_sha'],level,'IN_PROGRESS','Same exact source SHA was independently tested as a shared-food integration checkpoint before this contract arrived. Summer-whole 10% energy/HP/boost/remainder and failed-action no-award controls PASS; papaya/half/member-gate/runtime/native/lifecycle/reference coverage remains open.',
                    [str(manifest.relative_to(ROOT)).replace('\\','/')],scope='published-food-energy-link')
        continue
    add('SHARED-WORLD',value['tested_integration_sha'],'V1','IN_PROGRESS','Shared integration regression filters PASS: '+', '.join(r['filter']+':'+str(r['tests_passed']) for r in value['runs'])+'. This is scoped regression evidence, not complete current contract acceptance.',
        [str(manifest.relative_to(ROOT)).replace('\\','/')],kind='integration',scope='shared-regression',affected=['OR-MIG-0006','OR-MIG-0010','OR-MIG-0018','OR-MIG-0019','OR-MIG-0020','OR-MIG-0022','OR-MIG-0024'])
for path in OUT.glob('native-*/independent-assessment.json'):
    result=json.loads(path.read_text(encoding='utf-8'))
    if result.get('status')!='PASS':continue
    evidence=[str(path.relative_to(ROOT)).replace('\\','/')]
    add('OR-MIG-0006',result['tested_sha'],'V0','PASS','Current pinned integration build compiled, configured real cache-backed runtime and dispatched genuine native worn Options.',evidence,kind='integration',scope='native-worn-integration',affected=['OR-MIG-0006'])
    for level in ['V2','V3']:
        add('OR-MIG-0006',result['tested_sha'],level,'IN_PROGRESS',result['scope'],evidence,kind='integration',scope='native-worn-integration',affected=['OR-MIG-0006'])
food=OUT/'deep-food/independent-assessment.json'
if food.exists():
    result=json.loads(food.read_text(encoding='utf-8'))
    evidence=[str(food.relative_to(ROOT)).replace('\\','/')]
    add('OR-MIG-0024',result['tested_sha'],'V0','PASS','Exact revision3 server compiled, real cache-backed startup and genuine native ordinary meat/cake dispatch reached production handlers.',evidence,scope='food-native-runtime')
    for level in ['V2','V3']:
        add('OR-MIG-0024',result['tested_sha'],level,'IN_PROGRESS',result['scope'],evidence,scope='food-native-partial')
for path in OUT.glob('deep-chemistry-worn/*/independent-assessment.json'):
    result=json.loads(path.read_text(encoding='utf-8'))
    if result.get('status')!='PASS' or result.get('campaign_kind')!='historical':continue
    evidence=[str(path.relative_to(ROOT)).replace('\\','/'),'docs/openrune/verification/independent/deep-chemistry/independent-assessment.json']
    for level in ['V2','V3']:
        add('OR-MIG-0006',result['tested_sha'],level,'IN_PROGRESS','Exact historical SHA now has separate genuine native held controls/break and worn Options -> Stop campaigns, each with acknowledged save, actual restart and native relogin. Equipped item, charges, stop policy, coins and XP subchecks PASS; negative/cancelled dialogs, production/modifier and interrupted-transaction acceptance remain open.',evidence,scope='historical-worn-options')
for path in OUT.glob('reference-food-*/comparison.json'):
    result=json.loads(path.read_text(encoding='utf-8'))
    if result.get('source_documentation_subcheck')=='PASS':
        add('OR-MIG-0035',result['implementation_sha'],'V4','IN_PROGRESS','Independently verified three protected-source hashes and four pinned Wiki revisions/content hashes. Strawberry prose/table conflict, deliberate watermelon donor/Wiki rounding difference, sweetcorn common policy and source-only bowl-return evidence remain explicit. Full native presentation/timing and complete integration fidelity remain open.',[str(path.relative_to(ROOT)).replace('\\','/')],scope='dynamic-food-reference')
for path in sorted(OUT.glob('OR-MIG-0035-*-independent*/manifest.json')):
    result=json.loads(path.read_text(encoding='utf-8'))
    if result.get('outcome')!='PARTIAL':continue
    evidence=[str(path.relative_to(ROOT)).replace('\\','/')]
    for level in ['V0','V1']:
        add('OR-MIG-0035',result['implementation_commit_sha'],level,'IN_PROGRESS','Independent actual World action queue checks PASS: four healing policies across 99 base levels and four HP states (1584 fixtures), plus live-base/stale replay and seven invalid-rule/transaction cases. Full bags, duplicate selected input, bowl return, no XP and rollback verified. Configured runtime/profile binding, membership, native, persistence and reference acceptance remain open.',evidence,scope='dynamic-food-independent-'+path.parent.name)
for path in OUT.glob('OR-MIG-0023-*/manifest.json'):
    result=json.loads(path.read_text(encoding='utf-8'))
    if result.get('outcome')!='PARTIAL':continue
    if result.get('scope','').startswith('Expanded authoritative'):
        add('OR-MIG-0023',result['implementation_commit_sha'],'V1','IN_PROGRESS','Four exact-SHA world tests PASS including authoritative repeat opening, movement cancellation and stale selected-slot refusal. Attested binary was replayed without rebuilding during CPU contention. Native/lifecycle/reference acceptance remains open.',[str(path.relative_to(ROOT)).replace('\\','/')],scope='pack-authoritative-queue')
    for level in ['V0','V1']:
        add('OR-MIG-0023',result['implementation_commit_sha'],level,'IN_PROGRESS','Exact-SHA world atomicity/overflow helpers and server pack-definition bindings PASS (five test functions); full repeat-queue/native/lifecycle/reference acceptance remains open.',[str(path.relative_to(ROOT)).replace('\\','/')])
for path in OUT.glob('OR-MIG-0040-*/manifest.json'):
    result=json.loads(path.read_text(encoding='utf-8'))
    if result.get('outcome')!='PARTIAL':continue
    for level in ['V0','V1']:
        add('OR-MIG-0040',result['implementation_commit_sha'],level,'IN_PROGRESS','Independently executed13 existing exact-SHA authoritative/helper tests PASS, including actual held batch routing, two processing clocks, pair/timed regression, live level/member revocation, cancellation and no repeated rewards. Whole runtime/native/lifecycle/reference acceptance remains open.',[str(path.relative_to(ROOT)).replace('\\','/')],scope='shared-assembly-regression')
for pattern,ident,scope in [('deep-chemistry-last/*/independent-assessment.json','OR-MIG-0006','native-last-charge'),('deep-food-observer/*/independent-assessment.json','OR-MIG-0024','native-food-observer'),('deep-food-negative/*/independent-assessment.json','OR-MIG-0024','native-food-negative'),('deep-combat-interrupt/*/independent-assessment.json','OR-MIG-0010','native-combat-interrupted')]:
    for path in OUT.glob(pattern):
        result=json.loads(path.read_text(encoding='utf-8'))
        if result.get('status')!='PASS':continue
        for level in ['V2','V3']:
            add(ident,result['tested_sha'],level,'IN_PROGRESS',result['scope']+' Scoped subchecks only; complete level acceptance remains open.',[str(path.relative_to(ROOT)).replace('\\','/')],scope=scope)
path=OUT/'reference-chemistry/comparison.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'))
    add('OR-MIG-0006','e7ac3608e61893d80b2de6a1a2e512b3ab785505','V4','IN_PROGRESS','Independent supported source and pinned-documentation audit PASS, with explicit RNG-order, initial-delay, cache-operation and reset-representation adaptations. Original executable unavailable. Native cancellation mismatch PG-DEF-001 prevents a full behavioral parity claim.',[str(path.relative_to(ROOT)).replace('\\','/')],scope='chemistry-reference-audit')
path=OUT/'deep-chem-close/close-001/independent-assessment.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'))
    assert result['status']=='FAIL'
    add('OR-MIG-0006',result['tested_sha'],'V2','FAIL','PG-DEF-001 / GitHub123: genuine native Close95 leaves server dialog live; stale Stop115 changes saved policy0->1. Ordinary P2, not implementation-blocking. Prior passing item-only probe is not gameplay acceptance.',[str(path.relative_to(ROOT)).replace('\\','/')],scope='chemistry-native-close-defect')
path=OUT/'reference-ordinary-food/comparison.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'))
    add('OR-MIG-0024',result['tested_sha'],'V4','IN_PROGRESS','Independent source/pinned Shark revision audit PASS for supported selected replacement, heal20, transaction ordering and explicitly different local rollback/presentation policies. Complete static profile reference, activity/combat timing and sound/presentation remain open; no Kotlin executable observation claimed.',[str(path.relative_to(ROOT)).replace('\\','/')],scope='ordinary-food-source-audit')
for path in OUT.glob('OR-MIG-0051-*/manifest.json'):
    result=json.loads(path.read_text(encoding='utf-8'))
    if result.get('outcome')!='PARTIAL':continue
    for level in ['V0','V1']:
        add('OR-MIG-0051',result['implementation_commit_sha'],level,'IN_PROGRESS','Independent execution at exact SHA: five existing Prayer-related library checks and three unchanged selected-burial World action tests PASS; production server cargo check PASS. Autotests-disabled initial harness failure retained. No new authored oracle, native, persistence or full reference acceptance inferred.',[str(path.relative_to(ROOT)).replace('\\','/')],scope='selected-burial-regression')
path=OUT/'deep-prayer/burial-001/independent-assessment.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'))
    if result['status']=='PASS':
        evidence=[str(path.relative_to(ROOT)).replace('\\','/')]
        add('OR-MIG-0051',result['tested_sha'],'V0','PASS','Exact-SHA executable built, full actual cache-backed startup and configured native Bury dispatch succeeded. Independently loaded representative bone definitions/reward policy and actual selected-slot transaction confirm the registered entrypoint.',evidence,scope='native-burial-runtime')
        for level in ['V2','V3']:add('OR-MIG-0051',result['tested_sha'],level,'IN_PROGRESS',result['scope']+' Remaining Chaos/member/modifier/interruption and applicable crash/presentation obligations stay open.',evidence,scope='native-burial-selected-restart')
path=OUT/'reference-selected-burial/comparison.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'))
    add('OR-MIG-0051',result['tested_sha'],'V4','IN_PROGRESS','Independent six-reference source/pinned-Wiki audit PASS within scope: selected slot versus local type-total policy, exact4.5/72base XP and measured cache rewards, one-clock local versus two-clock donor, stronger atomic rollback and known above-base necklace restoration gap. Complete cadence/sound/Chaos/all-modifier fidelity remains open.',[str(path.relative_to(ROOT)).replace('\\','/')],scope='selected-burial-reference')
for path in OUT.glob('OR-MIG-0056-*/manifest.json'):
    result=json.loads(path.read_text(encoding='utf-8'))
    if result.get('outcome')!='PARTIAL':continue
    for level in ['V0','V1']:add('OR-MIG-0056',result['implementation_commit_sha'],level,'IN_PROGRESS','Independent exact-SHA execution of two gem-specific authoritative action tests and one seven-profile binding/XP/member test PASS. Deferred grinding suites were not run; native/cache/lifecycle/reference and expanded independent oracle acceptance remain open.',[str(path.relative_to(ROOT)).replace('\\','/')],scope='gem-specific-regression')
path=OUT/'gem-acceptance.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='ACCEPTED'
    for level,value in result['levels'].items():
        add('OR-MIG-0056',result['tested_sha'],level,value['status'],value['reason'],[str(path.relative_to(ROOT)).replace('\\','/')],scope='gem-manual-subset-acceptance-r2')
print('Per-level evidence events:',len(rows))
