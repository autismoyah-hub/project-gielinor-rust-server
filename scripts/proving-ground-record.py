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
print('Per-level evidence events:',len(rows))
