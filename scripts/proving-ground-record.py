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
    if value.get('suite')=='light-conversion-world':
        reason='Independent162actual World conversion cases PASS across9lit/unlit profiles:18ordinary/packed200-life positives and144zeroHP/queued-death/member/stale/variable/count/credit/policy/delay/modal/logout/claim/descriptor refusals. Queued death established through public server combat-hit entry, then visibleHP restored without respawn; conversion correctly remains refused.18additional stale typed actions do not consume retained matching lights or emit a false synthetic success acknowledgement. No tinderbox/currentFiremaking/Crafting/XP-rate requirement for extinguish; whole inventory/Stats preserved. Packed200is a representation fixture, not a normal attainable gameplay claim. Actual cache/runtime/native bindings and sand-specific behavior remain separate.'
        add('OR-MIG-0084',value['tested_integration_sha'],'V1','IN_PROGRESS',reason,[manifest.relative_to(ROOT).as_posix()],scope='independent-light-conversion-world')
        add('SHARED-CONVERSION',value['tested_integration_sha'],'V1','IN_PROGRESS',reason+' Shared action/Rule/life regression only; no sand Empty acceptance inferred.',[manifest.relative_to(ROOT).as_posix()],kind='integration',scope='independent-generic-held-conversion',affected=['OR-MIG-0079','OR-MIG-0084'])
        continue
    if value.get('suite')=='light-source-world':
        add('OR-MIG-0084',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent18full-bag public World lighting/extinguishing roundtrips PASS for9profiles/both held directions at exact84SHA: currentFiremaking requirement, drainedCrafting0,rate0, retained duplicate tinderboxes/raw and no XP/advancement/sequence; deterministic resolver NoDraw and zero-advancement checked. Extinguish atFiremaking0uses typed conversion and emits only configured synthetic success acknowledgement.216actual World lighting member/level/tool/raw/vars/count/credit/inventory/modal/logout/delay refusals preserve full inventory/Stats.2bounded synthetic skill9queued success/failure controls independently verify18/6fineXP and SkillAdvanced9while Crafting remains unchanged; not Fletching gameplay. Actual factory/cache/menu bindings, negative extinguish/life, all-old-profile regression, native/durable/reference acceptance remain separate.',[manifest.relative_to(ROOT).as_posix()],scope='independent-light-source-world')
        continue
    if value.get('suite')=='fallible-gem-world':
        add('OR-MIG-0064',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent240public World campaigns PASS at pinned64:72three-profile/both-direction/three-rate mixed/all-success/all-crush captured batches verify exact selected17then3/9 credits, first1-compensated/repeat2 clocks, one draw only per committed selected raw, branch-specific fineXP/advancement and retained raw26.84initial invalid member/level/rate/slot/tool/either-credit/modal/logout cases never draw/publish.84late changed captured slot/tool/member/level/rate/either-credit/modal/movement/Close/new-held/logout cases stop after first draw without fallback. Native/durable and reference conflict adjudication remain separate. Clarifies the initial scoped event: this headless harness does not assert feedback-message content; native message verification remains pending.',[manifest.relative_to(ROOT).as_posix()],scope='independent-fallible-gem-world-oracle-scope')
        continue
    if value.get('suite')=='fallible-gem-bindings':
        add('OR-MIG-0064',value['tested_integration_sha'],'V1','IN_PROGRESS','Actual cache factory attests8ordinary types/Crafting/3sequences and3fallible descriptors.93missing/aliased/corrupt item/stat/sequence probes refuse. Runtime/native/batch/durability and reference conflicts remain separate.',[manifest.relative_to(ROOT).as_posix()],scope='independent-fallible-gem-bindings')
        continue
    if value.get('suite')=='immediate-regression':
        add('SHARED-WORLD',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent deterministic crafting integration regression at immediate target:576positive/24F2P/140admission/192late cases PASS. Original OR63harness required only failure:None descriptor adaptation for65API; compile failure and diagnosis preserved. This is new integration evidence at65, not historical OR63acceptance or current FORGEHEAD verification.',[manifest.relative_to(ROOT).as_posix()],kind='integration',scope='independent-immediate-shared-regression',affected=['OR-MIG-0063','OR-MIG-0065'])
        continue
    if value.get('suite') in ['immediate-bindings','immediate-admission']:
        suite=value['suite']
        reason=('Actual cache-backed startup recipe factory resolves seven exact ordinary item definitions, Crafting stat and two sequence aliases. Both immediate descriptors and limestone failure curve/rewards match independent expectations;81missing/aliased/corrupted metadata mutations refuse. Native runtime/durability remain separate.' if suite=='immediate-bindings' else 'Independent88public World admission cases PASS: both recipes/directions, membership/current level, modal/delay/logout-saving/stale actor, stale/count/variable tool/raw, inventory policy, either output credit, invalid immediate descriptor and queued/quantity wrappers refuse without inventory/Stats changes or pending production/advancement/animation. Original unsupported trusted-Catalog tool mutation oracle is preserved with diagnosis; corrected campaign passed at same SHA.')
        add('OR-MIG-0065',value['tested_integration_sha'],'V1','IN_PROGRESS',reason,[manifest.relative_to(ROOT).as_posix()],scope='independent-'+suite)
        continue
    if value.get('suite')=='immediate-tools':
        add('OR-MIG-0065',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent207656zero-XP/fallible/current-level/member/rate/controlled-byte cases PASS against a rounded integer probability oracle, including limestone certainty40and one RNG draw even at certainty; Dramen has no draw or advancement and permits rate0.60invalid possible-credit/input/tool/policy probes refuse before draw; zero-XPfailure preserves Stats/no advancement while positive-success/rate0still refuses.48actual immediate World held routes cover both directions/full bag/no menu/clock advance/stale/delayed replay. Additional World negative/configuration, actual startup/native/persistence/reference acceptance remains separate.',[manifest.relative_to(ROOT).as_posix()],scope='independent-immediate-tools')
        continue
    if value.get('suite')=='production-gems':
        add('OR-MIG-0063',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent576sixteen-profile/both-direction/three-rate/quantity cases and24nonmember F2P gem controls PASS.140invalid admission and192late captured-slot/policy/credit/lifecycle cases verify no fallback/replay, legitimate F2P continuation after membership change and exact first1/repeat2gem versus first2/repeat3shell clocks. Public World routes used; actual startup bindings/native/persistence/reference acceptance remains separate from historical manual56/57.',[manifest.relative_to(ROOT).as_posix()],scope='independent-production-gems')
        continue
    if value.get('suite')=='amethyst-counts':
        add('OR-MIG-0071',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent7776full-bag/all-selected-slot/member/level/rate/count-boundary resolver cases and36invalid counted-credit/input/tool cases PASS. Exact15/15/5/8stack outputs and XP once per raw verified; overflow leaves inventory/Stats unchanged. Registered shared batch/runtime/native/persistence/reference acceptance remains separate.',[manifest.relative_to(ROOT).as_posix()],scope='independent-amethyst-counts')
        continue
    if value.get('suite')=='fallible-gem-matrix':
        add('OR-MIG-0064',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent465408controlled-byte/profile/level1-99/boost100-120/member/100-125-500rate cases PASS against an integer endpoint/rounding oracle, including exact selected full-bag output/XP and one draw only after both states prepare.48invalid either-credit/input/tool/member/level/rate/probability cases refuse without a draw or publication. These are public resolver checks, not registered runtime/World batch/native or original RNG-stream acceptance.',[manifest.relative_to(ROOT).as_posix()],scope='independent-fallible-gem-matrix')
        continue
    if value.get('suite') in ['glass-admission','glass-bindings','pouch-bindings']:
        suite=value['suite'];ident='OR-MIG-0054' if suite=='pouch-bindings' else 'OR-MIG-0062'
        reason={'glass-admission':'Independent112targeted captured-admission refusals PASS across8profiles: missing/wrong/count input/tool, inventory policy, modal/logout, same slot/origin, invalid cycle/index. Completed1344-case campaign was not replayed. Earlier trusted-host Catalog mutation oracle failure remains explicitly out of supported scope; startup metadata refusal is separate.',
                'glass-bindings':'Independent actual-cache10definition/8profile member-skill distinction and109corrupt object/stat/sequence refusals PASS. Configured native startup/dispatch acceptance remains independently recorded.',
                'pouch-bindings':'Independent actual-cache citizen pouch/3coin binding and11invalid metadata refusals PASS; existing no_bank resolver blocks actual bank transaction with a successful coin-deposit control. Decoded nearby banker cache observations guide a future native deposit run; no native bank acceptance inferred.'}[suite]
        add(ident,value['tested_integration_sha'],'V0' if suite=='pouch-bindings' else 'V1','IN_PROGRESS',reason,[manifest.relative_to(ROOT).as_posix()],scope='independent-'+suite)
        continue
    if value.get('suite') in ['glass-matrix','pouch-matrix']:
        suite=value['suite'];ident='OR-MIG-0062' if suite=='glass-matrix' else 'OR-MIG-0054'
        reason=('Independent eight-choice actual held-route matrix PASS across1344both-direction/requested-count1through28/three-rate scenarios,72invalid admission cases and96late captured-slot/credit/member/level/tool/modal/movement/logout/Close cases. Exact first selected input then captured order, first2/subsequent3action clocks, per-step fineXP and capped count verified.' if suite=='glass-matrix' else 'Independent selected-stack pouch one/all/overflow/member/full-bag transaction matrix and actual World negative/replay/no-XP controls PASS. Configured runtime/native no_bank and lifecycle/reference remain separate.')
        add(ident,value['tested_integration_sha'],'V1','IN_PROGRESS',reason+' Complete native, configured definitions and lifecycle/reference acceptance remain open.',[str(manifest.relative_to(ROOT)).replace('\\','/')],scope='independent-'+suite)
        continue
    if value.get('suite') in ['shell-matrix','shell-bindings']:
        suite=value['suite'];reason=('Independent nine-shell matrix PASS across4536all-selected-slot/member/current-level/100-125-500rate fixtures,162invalid transactions and252actual held-route cases with two-clock first action, late guards, cancellation and no replay.' if suite=='shell-matrix' else 'Independent actual-cache19ordinary shell/chisel/output definitions and nine exact stat/animation/profile bindings PASS;199corrupt object/symbol/stat/effect cases reject the entire catalog.')
        add('OR-MIG-0057',value['tested_integration_sha'],'V1' if suite=='shell-matrix' else 'V0','IN_PROGRESS',reason+' Complete native/lifecycle/reference aggregate acceptance remains separate.',[str(manifest.relative_to(ROOT)).replace('\\','/')],scope='independent-'+suite)
        continue
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
path=OUT/'deep-shells/shells-001/independent-assessment.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS'
    for level in ['V0','V2','V3']:add('OR-MIG-0057',result['tested_sha'],level,'IN_PROGRESS',result['scope']+' Aggregate published-level acceptance remains separate.',[str(path.relative_to(ROOT)).replace('\\','/')],scope='shell-native-directions-recovery')
path=OUT/'reference-shells/comparison.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['source_documentation_subcheck']=='PASS'
    add('OR-MIG-0057',result['tested_sha'],'V4','IN_PROGRESS',result['scope']+' Native effect/cadence comparison remains separate.',[str(path.relative_to(ROOT)).replace('\\','/')],scope='shell-source-reference')
path=OUT/'shell-acceptance.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='ACCEPTED'
    for level,value in result['levels'].items():add('OR-MIG-0057',result['tested_sha'],level,value['status'],value['reason'],[str(path.relative_to(ROOT)).replace('\\','/')],scope='shell-manual-subset-acceptance-r1')
for folder,ident in [('reference-glass','OR-MIG-0062'),('reference-pouches','OR-MIG-0054'),('reference-amethyst','OR-MIG-0071'),('reference-production','OR-MIG-0063'),('reference-immediate','OR-MIG-0065')]:
    path=OUT/folder/'comparison.json'
    if path.exists():
        result=json.loads(path.read_text(encoding='utf-8'));assert result['source_documentation_subcheck']=='PASS'
        add(ident,result['tested_sha'],'V4','IN_PROGRESS',result['scope']+' Original executable unavailable; native/reference comparison remains separate.',[str(path.relative_to(ROOT)).replace('\\','/')],scope=folder+'-audit')
path=OUT/'deep-glass/choices-001/independent-assessment.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS';evidence=[path.relative_to(ROOT).as_posix()]
    add('OR-MIG-0062',result['tested_sha'],'V0','PASS','Exact5ce50server compiled and actual cache-backed startup/held registry/SkillMulti permission-resume path dispatched all8glass outputs through genuine native controls. Strict nativeClose and fresh valid controls exercised.',evidence,scope='native-glass-runtime')
    for level in ['V2','V3']:add('OR-MIG-0062',result['tested_sha'],level,'IN_PROGRESS',result['scope']+' Native labels/quantity controls/repeat cadence and interrupted captured-batch logout remain open.',evidence,scope='native-glass-eight-choices-recovery')
    add('SHARED-CLOSE',result['tested_sha'],'V2','IN_PROGRESS','Nonchemistry integration subcheck at5ce50ae: genuine native95emptyClose followed by stale actual glassResume115 does not award; fresh controls work. This does not test historicalb11260eor deferred held/worn chemistry, does not closePG123and does not complete all shared dialog obligations.',evidence,kind='integration',scope='native-glass-close',affected=['OR-MIG-0061'])
path=OUT/'deep-glass-quantities/quantities-003/independent-subchecks.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS_SUBCHECKS' and result['raw_produce_status']=='FAIL'
    for level in ['V2','V3']:add('OR-MIG-0062',result['tested_sha'],level,'IN_PROGRESS',result['scope']+' Raw producer FAIL atX remains preserved; X/All and in-flight captured-batch logout stay open.',[path.relative_to(ROOT).as_posix()],scope='native-glass-1-5-10-saved-subchecks')
path=OUT/'deep-glass-all/all-001/independent-assessment.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS'
    for level in ['V2','V3']:add('OR-MIG-0062',result['tested_sha'],level,'IN_PROGRESS',result['scope']+' Xblocked separately; in-flight captured-batch lifecycle remains open.',[path.relative_to(ROOT).as_posix()],scope='native-glass-all23-recovery')
path=OUT/'deep-glass-remaining/remaining-002/independent-blocker.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='BLOCKED'
    add('OR-MIG-0062',result['tested_sha'],'V2','IN_PROGRESS','PG-INFRA-009: Xscenario BLOCKED by observed native keyboard/script extraResume270:17/count1before intended270:18/count3. Server matches earlier actual frame; no server defect established. Native8choices/labels/1/5/10/Alland normal recovery passed in separate preserved scopes. Continue other applicable scenarios; do not promote entire V2.',[path.relative_to(ROOT).as_posix()],scope='native-glass-x-blocker')
for mode in ['interrupt','crash']:
    path=OUT/('deep-glass-'+mode)/(mode+'-001')/'independent-assessment.json'
    if path.exists():
        result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS'
        add('OR-MIG-0062',result['tested_sha'],'V3','IN_PROGRESS',result['scope']+' Whole contract acceptance remains separate; native X remains blocked.',[path.relative_to(ROOT).as_posix()],scope='native-glass-'+mode+'-recovery')
for path in (OUT/'deep-pouch').glob('pouch-*/independent-assessment.json'):
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status'] in ['PASS','PASS_SUBCHECKS']
    for level in ['V0','V2','V3']:add('OR-MIG-0054',result['tested_sha'],level,'IN_PROGRESS',result['scope']+' Aggregate applicable-level acceptance remains separate.',[path.relative_to(ROOT).as_posix()],scope='native-pouch-'+path.parent.name+'-subchecks')
for kind in ['gems','shells']:
    path=OUT/('deep-production-'+kind)/'profiles-001/independent-assessment.json'
    if path.exists():
        result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS'
        for level in ['V0','V2','V3']:add('OR-MIG-0063',result['tested_sha'],level,'IN_PROGRESS',result['scope']+' Other applicable native batch/quantity/interruption/reference obligations remain separate.',[path.relative_to(ROOT).as_posix()],scope='native-production-'+kind+'-single-choice-recovery')
production_sha='143dab04f28c3daed844da0af33d6af5edcd962d'
production_native=[OUT/('deep-production-'+kind)/'profiles-001/independent-assessment.json' for kind in ['gems','shells']]
if all(p.exists() and json.loads(p.read_text())['status']=='PASS' for p in production_native):
    add('OR-MIG-0063',production_sha,'V0','PASS','Exact target server compiled; actual startup loaded attested definitions and registered all16gem/shell recipes through the single-option native SkillMulti/permission/resume path. Every recipe dispatched through real native Use and Make115to its exact output/fineXP, with genuineClose/stale refusals; no manual-contract acceptance inherited.',[p.relative_to(ROOT).as_posix() for p in production_native],scope='production-runtime-obligations-complete')
    matrix=OUT/('integration-external-production-gems-'+production_sha)/'manifest.json'
    if matrix.exists() and json.loads(matrix.read_text())['status']=='PASS':
        add('OR-MIG-0063',production_sha,'V1','PASS','Applicable authoritative obligations satisfied:576sixteen-profile/both-direction/three-rate/quantity/cap cases verify exact selected-first full-bag outputs and fineXP, first1/repeat2gems and first2/repeat3shells.24F2Pcontrols,140admission refusals and192late slot/tool/member/level/rate/credit/modal/movement/Close/logout/other-held cases pass without fallback or future loss. Actual16registered native profiles independently match these descriptors; custom/native quantity boundaries and durable interrupted batches remain separate V2/V3obligations.',[matrix.relative_to(ROOT).as_posix(),*[p.relative_to(ROOT).as_posix() for p in production_native]],scope='production-authoritative-obligations-complete')
for kind in ['gem','shell']:
    path=OUT/('deep-production-'+kind+'-batch')/'quantities-001/independent-assessment.json'
    if path.exists():
        result=json.loads(path.read_text());assert result['status']=='PASS'
        for level in ['V2','V3']:add('OR-MIG-0063',result['tested_sha'],level,'IN_PROGRESS',result['scope']+' Multi-digit-Xboundary and in-flight interruption remain separate.',[path.relative_to(ROOT).as_posix()],scope='native-production-'+kind+'-five-quantities-recovery')
for kind,mode in [('gem','interrupt'),('shell','interrupt'),('gem','crash')]:
    path=OUT/('deep-production-'+kind+'-'+mode)/(mode+'-001/independent-assessment.json')
    if path.exists():
        result=json.loads(path.read_text());assert result['status']=='PASS'
        add('OR-MIG-0063',result['tested_sha'],'V3','IN_PROGRESS',result['scope']+' Aggregate applicable persistence acceptance remains separate.',[path.relative_to(ROOT).as_posix()],scope='native-production-'+kind+'-'+mode+'-recovery')
production_durable=[*production_native,*[OUT/('deep-production-'+kind+'-batch')/'quantities-001/independent-assessment.json' for kind in ['gem','shell']],*[OUT/('deep-production-'+kind+'-'+mode)/(mode+'-001/independent-assessment.json') for kind,mode in [('gem','interrupt'),('shell','interrupt'),('gem','crash')]]]
if all(p.exists() and json.loads(p.read_text())['status']=='PASS' for p in production_durable):
    add('OR-MIG-0063',production_sha,'V3','PASS','Applicable persistence obligations satisfied: all16registered outputs/fineXP and23-item gem/shell quantity campaigns persist across acknowledged save and real process/native restart. Partial socket disconnect retains exact committed gem3/7500fineXP and shell1/1625fineXP prefixes, with20/22raws preserved and remaining work cancelled. Separate owned abrupt termination restores the prior complete23raw/Stats state, confirmed by actual restart/native packets without replay. No mid-SQL-commit injection claimed. Native custom-Xremains a separate V2blocker.',[p.relative_to(ROOT).as_posix() for p in production_durable],scope='production-durable-obligations-complete')
path=OUT/'deep-production-gem-x/x10-001/independent-blocker.json'
if path.exists():
    result=json.loads(path.read_text());assert result['status']=='BLOCKED'
    add('OR-MIG-0063',result['tested_sha'],'V2','BLOCKED',result['scope']+' '+result['correction_needed'],[path.relative_to(ROOT).as_posix()],scope='native-production-multidigit-x-blocked')
path=OUT/'deep-production-gem-x-focus/x10-focus-001/independent-blocker.json'
if path.exists():
    result=json.loads(path.read_text());assert result['status']=='BLOCKED'
    add('OR-MIG-0063',result['tested_sha'],'V2','BLOCKED',result['scope']+' '+result['correction_needed'],[path.relative_to(ROOT).as_posix()],scope='native-production-focused-x-blocked')
glass_sha='5ce50aede9164cdee8e4008257d0298ea77289e7'
path=OUT/'deep-pouch/pouch-002/independent-blocker.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='BLOCKED'
    add('OR-MIG-0054',result['tested_sha'],'V2','BLOCKED',result['scope']+' '+result['blocker'],[path.relative_to(ROOT).as_posix()],scope='native-pouch-bank-positive-control-blocked')
glass_authoritative=[OUT/('integration-external-'+suite+'-'+glass_sha+suffix)/'manifest.json' for suite,suffix in [('glass-matrix','-1792292760698579200'),('glass-admission','-1792294219033587500'),('glass-bindings','')]]
if all(p.exists() and json.loads(p.read_text(encoding='utf-8')).get('status')=='PASS' for p in glass_authoritative):
    add('OR-MIG-0062',glass_sha,'V1','PASS','Applicable authoritative requirements satisfied:1344eight-profile/both-direction/count1-28/three-rate full-bag cases verify capped captured order, exact fineXP and2then3cadence;72invalid admission and96late member/level/tool/slot/modal/movement/Close/logout cases verify atomic stop/no fallback. Additional112admission refusals and actual-cache109metadata refusals passed. Native custom-quantity remains a separate V2blocker.',[p.relative_to(ROOT).as_posix() for p in glass_authoritative],scope='glass-authoritative-obligations-complete')
glass_durable=[OUT/folder/run/'independent-assessment.json' for folder,run in [('deep-glass','choices-001'),('deep-glass-all','all-001'),('deep-glass-interrupt','interrupt-001'),('deep-glass-crash','crash-001')]]
if all(p.exists() and json.loads(p.read_text(encoding='utf-8')).get('status')=='PASS' for p in glass_durable):
    add('OR-MIG-0062',glass_sha,'V3','PASS','Applicable persistence requirements satisfied by real acknowledged saves and process restarts: all8outputs and23-item All batch retain exact inventory/fineXP. Disconnect during captured All preserves one committed product/875fineXP plus22raws and cancels remaining work. Separate unacknowledged owned-server termination restores prior whole state. Actual restart packets independently decoded in every campaign; no replay/material loss. No mid-SQL-commit injection claimed.',[p.relative_to(ROOT).as_posix() for p in glass_durable],scope='glass-durable-obligations-complete')
path=OUT/'deep-glass-remaining/remaining-002/independent-blocker.json'
if path.exists():
    add('OR-MIG-0062',glass_sha,'V2','BLOCKED','All native8choice/label/built-in1/5/10/All and Close subchecks passed, but required offered X selection cannot be accepted: PG-INFRA-009 proves an extra keyboard/script Resume consumes the dialog before intended choice. Precise client focus/callback correction required; server defect not established. Other completed obligations remain preserved.',[path.relative_to(ROOT).as_posix()],scope='glass-native-x-required-obligation-blocked')
path=OUT/'deep-fallible-gems/native-001/independent-assessment.json'
if path.exists():
    result=json.loads(path.read_text());assert result['status']=='PASS'
    add('OR-MIG-0064',result['tested_sha'],'V0','PASS','Exact64server compiles and starts against95hash-attested immutable resources. Actual-cache factory8ordinary item definitions/Crafting/3sequences and93metadata refusals PASS. All3registered native quantity/held routes dispatch success and crush outcomes to exact output/fineXP/sequence; no inherited currentHEAD acceptance.',[path.relative_to(ROOT).as_posix(),'docs/openrune/verification/independent/integration-external-fallible-gem-bindings-'+result['tested_sha']+'/manifest.json'],scope='fallible-runtime-obligations-complete')
    for level in ['V1','V2','V3']:add('OR-MIG-0064',result['tested_sha'],level,'IN_PROGRESS',result['scope']+' All3profiles naturally witness cut and crush;12580fineXP persists and independently computed level19matches native/SQL. Additional captured-batch interruption/native custom-quantity/reference obligations remain separate.',[path.relative_to(ROOT).as_posix()],scope='native-fallible-gems-mixed-recovery')
path=OUT/'deep-fallible-gem-interrupt/interrupt-001/independent-assessment.json'
if path.exists():
    result=json.loads(path.read_text());assert result['status']=='PASS'
    add('OR-MIG-0064',result['tested_sha'],'V3','IN_PROGRESS',result['scope']+' Observed'+str(result['native_observed_products'])+'and durable'+str(result['durable_products'])+'complete credits;'+str(result['remaining_raws'])+'raws retained. '+str(result['durable_fine_xp_award'])+'fineXP persists. This is acknowledged partial-queue save/restart, not crash atomicity; full applicable persistence acceptance remains separate.',[path.relative_to(ROOT).as_posix()],scope='native-fallible-partial-recovery')
for kind in ['staff','limestone','certainty','crash']:
    path=OUT/('deep-immediate-'+kind)/('crash-001' if kind=='crash' else 'native-001')/'independent-assessment.json'
    if path.exists():
        result=json.loads(path.read_text());assert result['status']=='PASS'
        for level in (['V3'] if kind=='crash' else ['V0','V2','V3']):add('OR-MIG-0065',result['tested_sha'],level,'IN_PROGRESS',result['scope']+' Applicable aggregate acceptance remains separate.',[path.relative_to(ROOT).as_posix()],scope='native-immediate-'+kind+'-recovery')
path=OUT/'reference-sinew/comparison.json'
if path.exists():
    result=json.loads(path.read_text());assert result['source_documentation_subcheck']=='PASS'
    add('OR-MIG-0066',result['tested_sha'],'V4','IN_PROGRESS',result['scope']+' Native/headless/runtime/lifecycle evidence remains separate; source/documentation subcheck only.',[path.relative_to(ROOT).as_posix()],scope='independent-sinew-references')
path=OUT/'reference-fallible-gems/comparison.json'
if path.exists():
    result=json.loads(path.read_text());assert result['source_documentation_subcheck']=='PASS_WITH_REFERENCE_CONFLICTS'
    add('OR-MIG-0064',result['tested_sha'],'V4','IN_PROGRESS',result['scope']+' PG-REF-001: pinned Opal3.8versus Crushedgem3.7failureXP and Jade floor-only versus generic rounded success rule require explicit adjudication; not an implementation-blocking gameplay finding.',[path.relative_to(ROOT).as_posix()],scope='independent-fallible-gem-reference-conflicts')
path=OUT/'immediate-acceptance.json'
if path.exists():
    result=json.loads(path.read_text());assert result['status']=='ACCEPTED'
    for level,value in result['levels'].items():add('OR-MIG-0065',result['tested_sha'],level,value['status'],value['reason'],[path.relative_to(ROOT).as_posix()],scope='immediate-applicable-obligations-complete')
coarse=DOC/'VERIFICATION_RESULTS.jsonl';prior=[json.loads(s) for s in coarse.read_text(encoding='utf-8').splitlines()];known={r['attempt_id'] for r in prior}
for name in ['gem-acceptance.json','shell-acceptance.json','immediate-acceptance.json']:
    path=OUT/name
    if not path.exists():continue
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='ACCEPTED' and all(v['status']=='PASS' for v in result['levels'].values())
    attempt='accepted-manual-'+result['implementation_id']+'-'+result['tested_sha']+'-r'+str(result['revision'])
    if attempt in known:continue
    row={'attempt_id':attempt,'implementation_id':result['implementation_id'],'implementation_commit_sha':result['tested_sha'],'contract_revision':result['revision'],'campaign_kind':'historical','outcome':'VERIFIED','level':'V0-V4 applicable manual subset','independent_verification':True,'tests_passed':0,'coverage':[k+' PASS: '+v['reason'] for k,v in result['levels'].items()],'reason':result['acceptance_scope']+'; aggregate acceptance only, no duplicate test-execution count. '+result['exclusions'],'evidence':[path.relative_to(ROOT).as_posix()],'issues':[]}
    with coarse.open('a',encoding='utf-8',newline='\n') as f:f.write(json.dumps(row)+'\n')
print('Per-level evidence events:',len(rows))
