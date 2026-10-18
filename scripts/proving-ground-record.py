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
    if value.get('suite','').startswith(('imcando-held-','imcando-worn-','rune-altars-','rune-close-','daeyalt-','crafting-cape-','zealot-','skillcape-vendor-')):continue  # Scoped aggregate retains partial and resource-deferred attempts.
    if value.get('suite')=='light-life':continue  # Exact-target finding below retains raw FAIL/BLOCKED attempts.
    if value.get('outcome')!='PARTIAL':continue
    sha=value['implementation_commit_sha'];evidence=[str(manifest.relative_to(ROOT)).replace('\\','/')]
    add('OR-MIG-0024',sha,'V0','IN_PROGRESS','Exact-SHA production crate compiled; runtime cache/registration and packet decoder acceptance remain open.',evidence)
    add('OR-MIG-0024',sha,'V1','IN_PROGRESS','Four independent authoritative World action tests PASS across 319 fixtures: all selected slots, full bags, duplicate/stale input, overheal, remainders and malformed transactions. Runtime packet decode and complete food matrix remain open.',evidence)
for manifest in OUT.glob('integration-*/manifest.json'):
    value=json.loads(manifest.read_text(encoding='utf-8'))
    if value.get('suite','').startswith(('imcando-held-','imcando-worn-','rune-altars-','rune-close-','daeyalt-','crafting-cape-','zealot-','skillcape-vendor-')):continue  # Recorded by immutable scoped aggregate below, not legacy filter format.
    if value.get('suite') in ['member-tanning-world','member-tanning-guards']:
        continue  # Combined independent assessment includes retained partial attempt.
    if value.get('suite') in ['light-legacy-bindings','light-bindings','light-tool-regression']:
        if value.get('status')=='PASS':
            suite=value['suite']
            level='V0' if suite=='light-bindings' else 'V1'
            reason=('Actual nineteen cache item definitions/Firemaking11 produce nine light/extinguish descriptors with correct target-specific menu mapping, zero XP/ticks/animation/failure.215 independently mutated missing/aliased/malformed item/stat/op cases fail closed. Actual native startup and dispatch acceptance remain separate.' if suite=='light-bindings' else 'Exact84 shared tool regression:576 gem/shell batches,24F2P controls,140initial refusals and192late changes preserve original Crafting12, whole inventory/Stats/XP/captured order/timing. Only explicit skill12 added to independent historical fixture. No historical63/71 acceptance inherited and no chemistry gameplay executed.')
            if suite=='light-legacy-bindings':reason='Actual37legacy cache-backed Crafting tool descriptors retain skill12 with Firemaking0, full28 selected-slot output and durable tools, correctly scaled per-step125percent XP; separate light metadata prefix and World932case regression remain distinct. No chemistry gameplay or earlier contract acceptance inherited.'
            add('OR-MIG-0084',value['tested_integration_sha'],level,'IN_PROGRESS',reason,[manifest.relative_to(ROOT).as_posix()],scope='independent-'+suite)
        continue
    if value.get('suite')=='tanning-world':
        sha=value['tested_integration_sha'];log=manifest.parent/'run-1.log';text=log.read_text()
        finding=OUT/('tanning-life-finding.json' if sha=='2da0bff39acf97f0d974d8748b7ee186514b95f7' else 'tanning-life-r2-addendum.json')
        if finding.exists() and text.count('PG_TANNING_LIFE ')==16 and text.count('inventory_unchanged=false')==16:
            ev=[manifest.relative_to(ROOT).as_posix(),finding.relative_to(ROOT).as_posix()]
            add('OR-MIG-0105',sha,'V0','IN_PROGRESS','Exact-SHA World crate compiles; synthetic reached Trade3 dispatch and public World request/quantity-token flow work for Ellis3231/Guild5809. Actual-cache/startup/native runtime registration remains untested.',ev,scope='independent-tanning-world-dispatch')
            add('OR-MIG-0105',sha,'V1','FAIL','PG-DEF-005:16separate both-NPC/soft-hard/zeroHP-or-queued-death/initial-or-pending World quantity-flow refusals FAIL. Five hides convert and5/15GPspend, XPunchanged.160ordinary fullbag/minrequest-held-affordable/F2P/currentCrafting0/zeroXP/World-token-once cases PASS. Whole rawFAILretained. No private Request construction; actual Runtime/native death reachability, UI raw0clamp,106member profiles and persistence remain untested.',ev,scope='independent-tanning-life-failure')
        continue
    if value.get('suite')=='crafting-finishes-world':
        log=manifest.parent/'run-1.log';text=log.read_text()
        if all('test '+name+' ... FAILED' in text for name in ['wire_zero_hp_forward','wire_zero_hp_reverse','wire_queued_death_forward','wire_queued_death_reverse']):
            add('OR-MIG-0070',value['tested_integration_sha'],'V1','FAIL','PG-DEF-004: four isolated wiring life refusals fail across both directions/zeroHP/queued death with visibleHP restored. Selected empty orb and wire produce10973and1300fineXP at125%. Four separate manual-helm life refusals pass. Normal matrix initially failed verifier base-advancement oracle, not gameplay; raw wholeFAIL retained. Native reachability/currentHEAD/durability remain untested.',[manifest.relative_to(ROOT).as_posix(),'docs/openrune/verification/independent/crafting-finishes-life-finding.json'],scope='independent-finishes-life-failure')
        continue
    if value.get('suite')=='crafting-finishes-admission':
        log=manifest.parent/'run-1.log'
        if 'test twenty_pending_helm_changes_cancel_selected_credit ... ok' in log.read_text():
            add('OR-MIG-0070',value['tested_integration_sha'],'V1','FAIL','Twenty independent pending-helm late changes PASS across both directions: missing/variable selected raw or tool, live member/current/rate/credit refusal, modal pulse/Close/Walk preserve whole inventory/Stats. Separate initial refusal function failed an unsupported synthetic count2-refusal oracle; raw wholeFAIL retained. PG-DEF-004 wiring life failure remains unresolved; this passing subtest is not whole V1acceptance.',[manifest.relative_to(ROOT).as_posix()],scope='independent-finishes-late-subcheck')
        continue
    if value.get('suite')=='crab-matrix' and value.get('status')=='FAIL':
        log=manifest.parent/'run-1.log';text=log.read_text()
        if 'test every_roll_across_current_levels_member_and_rates_has_exact_zero_xp_failure ... ok' in text:
            assert 'profile0 mode11' in text
            add('OR-MIG-0068',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent310272controlled-byte cases PASS across two profiles/current1-99and100/120/member/three positive rates. Exact selected-only full28success or broken outputs, single draw, unchanged input snapshots, threshold70at15/241at99, exact fineXP and no advancement on failure all pass. Separate negative function failed because the verifier wrongly treated high256as invalid; shared u16 endpoint policy allows it. This is a passing subtest, not a PASS for the failed whole campaign. Preserve raw FAIL; only corrected negatives rerun separately, no repetition of completed byte matrix.',[manifest.relative_to(ROOT).as_posix(),log.relative_to(ROOT).as_posix()],scope='independent-crab-byte-domain-subcheck')
    if value.get('suite')=='pair-production-counts' and value.get('status')=='BLOCKED' and value.get('completed_case_subcheck')==728:
        add('OR-MIG-0069',value['tested_integration_sha'],'V1','IN_PROGRESS','Original count matrix hit300second bound without final result. Immutable per-profile progress establishes728complete full28count1-28cases for profiles0-12/both directions. Profiles13-17remain unaccepted; no whole-suite PASS. Preserve original wrapper timeout; rerun only remaining5profiles.',[manifest.relative_to(ROOT).as_posix()],scope='independent-pair-counts-completed-prefix')
    if value.get('status')!='PASS':continue
    if value.get('suite') in ['amethyst-boosts','amethyst-policy']:
        suite=value['suite'];reason=('Sixteen actual World current-at-required/base-one-below cases across4profiles/four rates plus8initial World i32MAXoverflow refusals PASS, retaining whole inventory/Stats/no RNG. Rate0does not award; existing resolver/late overflow evidence remains separate.' if suite=='amethyst-boosts' else 'Four unsupported multi-output failure policies refuse before RNG and eight synthetic legacy count-one success/failure branches preserve whole selected-slot/fineXP semantics. Actual amethyst recipes remain deterministic failureNone; no fallible amethyst gameplay or chemistry content claimed.')
        add('OR-MIG-0071',value['tested_integration_sha'],'V1','IN_PROGRESS',reason+' Applicable whole V1requires the aggregate of all immutable campaigns.',[manifest.relative_to(ROOT).as_posix()],scope='independent-'+suite)
        continue
    if value.get('suite')=='amethyst-bindings':
        add('OR-MIG-0071',value['tested_integration_sha'],'V0','IN_PROGRESS','Actual owned cache six item definitions/Crafting12/sequence6295symbol resolve four member raw21347/chisel1755recipes: counts15/15/5/8, levels83/85/87/89,60XPper raw, manualfirst1/cycle2and no failure.69missing/aliased/corrupt item/stat/sequence cases refuse. Exact server startup/native four-option dispatch remains open.',[manifest.relative_to(ROOT).as_posix()],scope='independent-amethyst-cache-bindings')
        continue
    if value.get('suite')=='amethyst-life':
        add('OR-MIG-0071',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent16before-first-credit life refusals PASS across4profiles/both directions/zeroHPandactual queued death with visibleHP restored without respawn. Full inventory/Stats/no RNG conserved after12ticks; separate late-death cases already complete, runtime native token/reachability remains separate.',[manifest.relative_to(ROOT).as_posix()],scope='independent-amethyst-before-credit-life')
        continue
    if value.get('suite')=='amethyst-legacy':
        add('SHARED-TOOLS',value['tested_integration_sha'],'V1','PASS','Exact71new integration regression PASS:576legacy integer gem/shell choice batches,24F2Pcontrols,140initial refusals and192late changes retain full inventory/Stats/timing/captured order. Only descriptor output_count1/failureNone compatibility fields added to the historical independent fixture; original target63campaign is not rerun or overwritten and no historical acceptance is inherited. No chemistry gameplay execution.',[manifest.relative_to(ROOT).as_posix()],kind='integration',scope='independent-amethyst-count-one-regression',affected=['OR-MIG-0056','OR-MIG-0057','OR-MIG-0063','OR-MIG-0071'])
        continue
    if value.get('suite')=='amethyst-world':
        add('OR-MIG-0071',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent288actual World choice/batch cases plus8late overflow cases PASS at exact71. Four output profiles/both directions/no-stack versus existing stack/three rates/count1-2-4-5-10-28 verify selected17then3/9/26capture, full28count15/15/5/8tips per raw, per-raw60XP, independent whole inventory/Stats per tick, first1compensation/repeat2and no RNG/repeat. Near-i32MAXstack credits exactly two whole raw results then refuses remaining9/26without partial tips or extra XP. Native/runtime/lifecycle and remaining policy/life guards remain open.',[manifest.relative_to(ROOT).as_posix()],scope='independent-amethyst-world-batches')
        continue
    if value.get('suite')=='amethyst-guards':
        add('OR-MIG-0071',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent112initial and112late World guards PASS across4profiles/both directions. Invalid option/count/member/current/rate/stale or variable selected input/tool/credit/modal/logout refuse; after one complete tips+XPcredit, captured-slot stale replacement with duplicate fallback bait, tool/policy/credit/modal/Walk/Close/logout/zeroHP/actual queued death with visibleHP restored stop remaining work and preserve whole snapshot without RNG. Runtime player token ownership and native life reachability are separate; no whole V1or contract acceptance inferred.',[manifest.relative_to(ROOT).as_posix()],scope='independent-amethyst-world-guards')
        continue
    if value.get('suite')=='crafting-finishes-refusals':
        add('OR-MIG-0070',value['tested_integration_sha'],'V1','FAIL','Independent44initial refusal cases PASS across wiring/helm/both directions: member/current/rate0andnegative/stale or variable input/tool/credit/modal/logout conserve full inventory/Stats with no RNG. Removed unsupported synthetic count2refusal expectation; no completed20late campaign replayed. Separate confirmed PG-DEF-004 wiring life failure remains unresolved, so V1remains FAIL.',[manifest.relative_to(ROOT).as_posix()],scope='independent-finishes-initial-refusals')
        continue
    if value.get('suite')=='crafting-finishes-bindings':
        add('OR-MIG-0070',value['tested_integration_sha'],'V0','IN_PROGRESS','Actual owned cache six item definitions/Crafting12 attest both manual descriptors: member empty-orb10980/wire10981 to10973/current87/104XP and chisel1755/visage12927 to ordinary uncharged12929/current52/120XP/manual1/no failure or animation.67missing/aliased/corrupt item/stat cases refuse. Runtime startup/native dispatch remain open; no level PASS inferred from factory alone.',[manifest.relative_to(ROOT).as_posix()],scope='independent-finishes-bindings')
        continue
    if value.get('suite')=='crafting-finishes-matrix':
        add('OR-MIG-0070',value['tested_integration_sha'],'V1','FAIL' if (OUT/'crafting-finishes-life-finding.json').exists() else 'IN_PROGRESS','Independent384World cases PASS across two manual profiles/four selected slots/both held directions/member/current below-at99/boosted base1/four rates. Full28whole-inventory and allStats, exact output identity/zero vars/duplicate retention, synchronous wiring versus one-tick compensated helm, independent base-level advancement and no repeat/RNG checked. Separate PG-DEF-004 life failure remains unresolved; this normal subcheck does not replace FAIL.',[manifest.relative_to(ROOT).as_posix()],scope='independent-finishes-matrix-subcheck')
        continue
    if value.get('suite') in ['pair-production-regression','current-assembly-regression'] and value['tested_integration_sha']!='fb348dda184b36ec2d503bfe0f1d5393cf8ae151':
        add('SHARED-ASSEMBLY',value['tested_integration_sha'],'V1','PASS','Independent current integration regression:16integer/precise controls,96legacy normal/timed/immediate held routes and16boosted-current-level precise transactions PASS at this exact integration SHA. Synthetic public gameplay controls only; no historical contract acceptance, chemistry content execution, native or lifecycle acceptance inferred.',[manifest.relative_to(ROOT).as_posix()],kind='integration',scope='independent-current-pair-regression',affected=['OR-MIG-0065','OR-MIG-0067','OR-MIG-0069'])
        continue
    if value.get('suite')=='pair-production-counts-tail':
        add('OR-MIG-0069',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent280remaining full28count cases for profiles13-17/both directions PASS. This only completes the tail; original728completed prefix retained without replay. Combined1008case scope requires both immutable evidence identities, and no whole PASS is assigned to the timed-out original wrapper.',[manifest.relative_to(ROOT).as_posix()],scope='independent-pair-counts-remaining-tail')
        continue
    if value.get('suite')=='pair-production-counts':
        add('OR-MIG-0069',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent1008full28World cases across18profiles/both directions/all requested1through28 PASS. Actual14two-input pairs cap14, selected13/27first then ascending0/14through12/26, exact per-credit first-free output and fractional XP/allStats, no RNG/extra credit verified each tick. This extends earlier3pair caps; native multi-pair/lifecycle remain separate.',[manifest.relative_to(ROOT).as_posix()],scope='independent-pair-production-counts')
        continue
    if value.get('suite')=='pair-production-life':
        add('OR-MIG-0069',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent72before-first-credit life cases across all18profiles/both directions PASS: zeroHP or actual queued death with visibleHP restored preserves inventory/Stats/no RNG/no future credit. This new production path is separate from the earlier immediate/precise life defects; native reachability remains distinct.',[manifest.relative_to(ROOT).as_posix()],scope='independent-pair-production-life')
        continue
    if value.get('suite')=='pair-production-regression':
        add('OR-MIG-0069',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent exact69shared regression:16integer/precise API controls,96legacy normal/timed/immediate held routes and16boosted-current-level precise transactions PASS. Synthetic gameplay controls only; no chemistry content execution, no inheritance from67.',[manifest.relative_to(ROOT).as_posix()],scope='independent-pair-production-regression')
        continue
    if value.get('suite')=='pair-production-bindings':
        add('OR-MIG-0069',value['tested_integration_sha'],'V1','IN_PROGRESS','Actual38unique cache item definitions, Crafting12, sequence7531/four spots, all18manual source factory descriptors and421missing/aliased/corrupt metadata probes PASS. Production first/repeat scheduling and quantity registration remain separate World/native scopes.',[manifest.relative_to(ROOT).as_posix()],scope='independent-pair-production-bindings')
        continue
    if value.get('suite')=='pair-production-admission':
        add('OR-MIG-0069',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent636initial refusal cases across18profiles/both directions PASS: option/count bounds, selected missing/variable/stacked pairs, member/current/rate/output credit, modal/logout/movement and invalid first/repeat ticks conserve full inventory/Stats with no future credit or RNG. Native token ownership remains separate.',[manifest.relative_to(ROOT).as_posix()],scope='independent-pair-production-admission')
        continue
    if value.get('suite') in ['pair-production','pair-production-late']:
        suite=value['suite'];reason=('Independent2592World routes across18profiles/both directions/member/current below-at99/four rates/count1-3-28PASS. Exact two captured inputs, selected-first then3/8and11/22pairs, full28caps, per-step fractional rewards, no RNG/repeat and complete inventory/Stats verified. Public action clocks sampled before Late increment validate actual first/repeat enqueue and commit trace stamps; original post-tick trace oracle failure retained. Stringing first1/repeat2andstaff first3/repeat2scheduling separately observed through compensated World clocks.' if suite=='pair-production' else 'Independent564late changes across18profiles/both directions PASS after first complete credit: stale/missing/variable captured pair with replacement duplicate, member/current/rate/credit changes, modal/Walk/Close/logout/new-held/zeroHP/actual queued death with visibleHP restored stop all remaining credits and preserve full inventory/Stats. Nonmember revocation only tested as refusal for member recipes; no invalid F2Pgate inferred. No runtime token acceptance claimed from replaying a trusted internal request.')
        add('OR-MIG-0069',value['tested_integration_sha'],'V1','IN_PROGRESS',reason+' Exact cache/startup/native/legacy and whole-contract obligations remain separate.',[manifest.relative_to(ROOT).as_posix()],scope='independent-'+suite)
        continue
    if value.get('suite')=='crab-counts':
        add('OR-MIG-0068',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent112full28World cases PASS across both profiles/directions and every requested count1through28. Selected17then ascending captured order, cap25matching raws, per-step mixed good/broken outputs,406fineXP per success at125%rather than aggregate rounding,0brokenXP/no advancement, first3/repeat3compensation and complete inventory/Stats invariants pass. Two chisels/1000coins retained; no RNG after capped work ends.',[manifest.relative_to(ROOT).as_posix()],scope='independent-crab-counts')
        continue
    if value.get('suite') in ['crab-world','crab-lifecycle']:
        suite=value['suite'];reason=('Independent48mixed/uniform captured3-item batches PASS across2profiles/both held directions/three rates. Exact selected17then3/9order/full28, single draw per credit, nominal3first/3repeat(input compensation4then3), success fineXP and zero broken XP/no SkillAdvanced event verified, including continuation after break.56late input/tool/member/level/rate/either-credit/modal/movement/Close/logout/new-held cases stop before a second draw without duplicate fallback;56invalid admissions never roll or mutate.' if suite=='crab-world' else 'Independent16production life cases PASS across both profiles and held directions. ZeroHP/actual queued death with visibleHP restored before admission or after first broken result refuses further draw/credit. First broken outcome has unchanged Crafting/no advancement, next raw retained. Dialog token identity is runtime-owned; direct replay of a trusted World Request is not a player token test. That unsupported oracle and initial moved-request compile failure are preserved. Genuine native Close/stale transport remains separate; corrected same-SHA life matrix changes no production code.')
        add('OR-MIG-0068',value['tested_integration_sha'],'V1','IN_PROGRESS',reason+' Native quantity/timing and full applicable acceptance remain separate.',[manifest.relative_to(ROOT).as_posix()],scope='independent-'+suite)
        continue
    if value.get('suite') in ['crab-bindings','crab-admission']:
        suite=value['suite'];reason=('Actual cache seven ordinary definitions/Crafting12/sequence1280and exact two member/level15/325success/0failure/40-240curve/first3descriptors PASS;79missing/aliased/corrupt item/stat/sequence cases refuse.' if suite=='crab-bindings' else 'Independent30invalid input/tool/count/variables/member/current-level/rate/either-credit/curve probes PASS before RNG and without inventory/Stats mutation. Corrected u16endpoint out-of-range65536oracle; original unsupported256refusal assertion preserved, completed310272roll subtest not repeated.')
        add('OR-MIG-0068',value['tested_integration_sha'],'V1','IN_PROGRESS',reason+' Authoritative World batch/native/lifecycle acceptance remains separate.',[manifest.relative_to(ROOT).as_posix()],scope='independent-'+suite)
        continue
    if value.get('suite')=='battlestaff-regression':
        add('OR-MIG-0067',value['tested_integration_sha'],'V1','IN_PROGRESS','Independent shared reward regression PASS:16integer/precise API controls with independent exact awards and no partial credit;96legacy normal/timed/immediate held routes across both directions, rates and zero/nonzeroXP preserve original synchronous versus queued publication/no repeat.16actual boosted-current-level precise transactions accept all4profiles below required base level and preserve exact fractional awards; rate0refuses without mutation. These are synthetic public-API controls, not chemistry content execution or native acceptance.',[manifest.relative_to(ROOT).as_posix()],scope='independent-battlestaff-regression')
        continue
    if value.get('suite') in ['battlestaff-matrix','battlestaff-bindings']:
        suite=value['suite'];reason=('Independent96precise transactions,192actual World held routes and88late changes PASS at exact67. Four profiles, both directions/full28, current-level/member/rate0-100-125-500guards, exact fractional rewards, retained duplicates, no RNG/quantity/repeat, nominal2timing and animation7531/elemental spots verified. Rate0may animate an otherwise admitted action but cannot credit; original overstrict animation oracle and compile mismatch preserved. Legacy integer/immediate regressions and native/lifecycle remain separate.' if suite=='battlestaff-matrix' else 'Actual cache nine ordinary member items/Crafting12/sequence7531/four spot aliases and exact four precise descriptors PASS.107missing/aliased/corrupt item/stat/sequence/spot cases refuse factory binding. Runtime/native/lifecycle remains separate.')
        add('OR-MIG-0067',value['tested_integration_sha'],'V1','IN_PROGRESS',reason,[manifest.relative_to(ROOT).as_posix()],scope='independent-'+suite)
        continue
    if value.get('suite') in ['sinew-matrix','sinew-bindings']:
        suite=value['suite'];reason=('Independent1248all-selected-slot/current-versus-base/member/rate resolver cases PASS with no RNG and immutable inputs.384actual World held routes cover both directions/full28/boosted10/drained9/0-100-125-500rates/exact187fineXP/manual first1queue compensation and no quantity/repeat.30late input/tool/member/level/rate/credit/modal/movement/Close/logout/zeroHP/queued-death cases preserve selected input, and overlapping new held action produces only its newly selected tail. Actual startup/native/durability remains separate.' if suite=='sinew-matrix' else 'Actual cache factory attests Knife946, Damaged monkey tail19665(alias damaged_ballista_rope), Sinew9436, Crafting12 and sequence10937. Exact member/manual1/level10/150tenthsXP/no-failure descriptor and39missing/aliased/corrupt item/stat/sequence refusals PASS. Actual native runtime/durability remains separate.')
        add('OR-MIG-0066',value['tested_integration_sha'],'V1','IN_PROGRESS',reason,[manifest.relative_to(ROOT).as_posix()],scope='independent-'+suite)
        continue
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
if path.exists() and not (OUT/'immediate-acceptance-withdrawal.json').exists():
    result=json.loads(path.read_text());assert result['status']=='ACCEPTED'
    for level,value in result['levels'].items():add('OR-MIG-0065',result['tested_sha'],level,value['status'],value['reason'],[path.relative_to(ROOT).as_posix()],scope='immediate-applicable-obligations-complete')
coarse=DOC/'VERIFICATION_RESULTS.jsonl';prior=[json.loads(s) for s in coarse.read_text(encoding='utf-8').splitlines()];known={r['attempt_id'] for r in prior}
crab_blocker=OUT/'crab-custom-input-dependency.json'
if crab_blocker.exists():
    r=json.loads(crab_blocker.read_text());attempt='independent-crab-applicable-levels-'+r['tested_sha']
    if attempt not in known:
        row={'attempt_id':attempt,'implementation_id':'OR-MIG-0068','implementation_commit_sha':r['tested_sha'],'contract_revision':1,'campaign_kind':'historical','outcome':'PARTIAL','level':'V0/V1/V3/V4PASS;V2BLOCKED','independent_verification':True,'tests_passed':0,'coverage':['Independent310272byte-domain subcheck,30pre-RNG refusals,79cache refusals and288World cases.','39normal native products/restarts, zero-XP partial save and owned unsaved-result recovery complete.'],'reason':'Required native custom input remains blocked on PG-INFRA-009; no exact68Xfailure alleged and no aggregate acceptance or duplicate test-execution count. V0/V1/V3/V4complete at pinned SHA.','evidence':[crab_blocker.relative_to(ROOT).as_posix()],'issues':['PG-INFRA-009 client dependency; historical proof remains at its own SHAs']}
        with coarse.open('a',encoding='utf-8',newline='\n') as f:f.write(json.dumps(row)+'\n')
finding=OUT/'battlestaff-life-finding.json'
if finding.exists():
    r=json.loads(finding.read_text());attempt='independent-PG-DEF-002-'+r['tested_sha']
    if attempt not in known:
        row={'attempt_id':attempt,'implementation_id':'OR-MIG-0067','implementation_commit_sha':r['tested_sha'],'contract_revision':1,'campaign_kind':'historical','outcome':'FAILED','level':'V1/V3 life refusal','independent_verification':True,'tests_passed':0,'coverage':['Four isolated authoritative death-boundary failures;8modal cancellation cases pass.','Separate V0/V2/V4 and normal/native/crash durable scopes remain recorded per-level.'],'reason':'PG-DEF-002 P2: selected water staff/orb consumed and1250fineXP awarded at zeroHP or after queued death, fresh and pending actions. Native dead-player reachability/latestHEAD unproven. No aggregate acceptance or duplicated execution count.','evidence':[finding.relative_to(ROOT).as_posix()],'issues':['PG-DEF-002 local backlog; no external issue']}
        with coarse.open('a',encoding='utf-8',newline='\n') as f:f.write(json.dumps(row)+'\n')
path=OUT/'reference-crab/comparison.json'
if path.exists():
    result=json.loads(path.read_text());assert result['source_documentation_subcheck']=='PASS'
    add('OR-MIG-0068',result['tested_sha'],'V4','IN_PROGRESS',result['scope']+' Native cadence, mixed batch and lifecycle remain separate; source recipe quest notes versus fresh-material acquisition context explicitly preserved.',[path.relative_to(ROOT).as_posix()],scope='independent-crab-references')
path=OUT/'reference-pair-production/comparison.json'
if path.exists():
    result=json.loads(path.read_text());assert result['source_documentation_subcheck']=='PASS'
    add('OR-MIG-0069',result['tested_sha'],'V4','IN_PROGRESS',result['scope']+' Source39hashes/14wool recipes/4staff facts only; actual native first-action clock comparison, exact cache/runtime/World/lifecycle remain separate.',[path.relative_to(ROOT).as_posix()],scope='independent-pair-production-references')
path=OUT/'reference-battlestaves/comparison.json'
if path.exists():
    result=json.loads(path.read_text());assert result['source_documentation_subcheck']=='PASS'
    add('OR-MIG-0067',result['tested_sha'],'V4','IN_PROGRESS',result['scope']+' Runtime/native/lifecycle and complete acceptance remain separate.',[path.relative_to(ROOT).as_posix()],scope='independent-battlestaff-references')
path=OUT/'deep-crab-quantities/quantities-001/independent-assessment.json'
if path.exists():
    result=json.loads(path.read_text());assert result['status']=='PASS'
    add('OR-MIG-0068',result['tested_sha'],'V2','IN_PROGRESS',result['scope']+' Additional15native products,6carved/9broken,9750fineXP and8raws retained. This only extends builtin quantity coverage; no custom-Xacceptance inferred.',[path.relative_to(ROOT).as_posix()],scope='crab-native-builtin-five-ten')
path=OUT/'deep-crab-carving/native-001/independent-assessment.json'
if path.exists():
    result=json.loads(path.read_text());assert result['status']=='PASS'
    ev=[path.relative_to(ROOT).as_posix(),'docs/openrune/verification/independent/integration-external-crab-bindings-'+result['tested_sha']+'/manifest.json']
    add('OR-MIG-0068',result['tested_sha'],'V0','PASS','Exact68server builds and starts against95hash-attested resources; actual cache7items/Crafting12/sequence1280and79factory refusals pass. Actual two registered native quantity choices dispatch both carved and broken outputs.',ev,scope='crab-runtime-obligations-complete')
    for level in ['V2','V3']:add('OR-MIG-0068',result['tested_sha'],level,'IN_PROGRESS',result['scope']+' Shell3success/9broken and claw6success/6broken;14625fineXP retained at Craft18. Initial concurrent reconnect startup timeout preserved; same frozen driver alone passes in immutable relogin-002with complete SQL/native continuity. Additional native quantity/interruption/crash obligations remain separate.',ev,scope='crab-native-mixed-and-reconnect')
    refs=OUT/'reference-crab/comparison.json';ref=json.loads(refs.read_text());assert ref['source_documentation_subcheck']=='PASS'
    add('OR-MIG-0068',result['tested_sha'],'V4','PASS',ref['scope']+' All24nativecredits independently show first3/repeat3receive-clock cadence and exact per-branch feedback/rewards. Wiki0XPbroken and first3explicitly override donor omitted failure/default-shortened first2. Recipe quest-note/acquisition context and lack of original random-stream/rendered/audio equivalence remain explicit. Scope is already-owned material crafting, not quest/acquisition acceptance.',ev+[refs.relative_to(ROOT).as_posix()],scope='crab-reference-obligations-complete')
path=OUT/'deep-crab-interrupt/interrupt-002/independent-assessment.json'
if path.exists():
    result=json.loads(path.read_text());assert result['status']=='PASS'
    add('OR-MIG-0068',result['tested_sha'],'V3','IN_PROGRESS',result['scope']+' Actual durable prefix is one broken shell at26,0fineXP,22raws retained. Whole native reconnect/SQL state passes; prior concurrent startup failure remains immutable. Owned abrupt termination remains a separate obligation.',[path.relative_to(ROOT).as_posix()],scope='crab-native-zero-xp-partial-recovery')
path=OUT/'deep-crab-crash/crash-001/independent-assessment.json'
if path.exists():
    result=json.loads(path.read_text());assert result['status']=='PASS'
    add('OR-MIG-0068',result['tested_sha'],'V3','IN_PROGRESS',result['scope'],[path.relative_to(ROOT).as_posix()],scope='crab-native-unsaved-recovery')
crab_sha='894646a79fa4ba7e3e8b46af6ec000dc6b796514'
crab_authoritative=[]
for suite in ['crab-world','crab-lifecycle','crab-counts','crab-admission','crab-bindings']:
    matches=[p for p in OUT.glob('integration-external-'+suite+'-'+crab_sha+'*/manifest.json') if json.loads(p.read_text()).get('status')=='PASS']
    if matches:crab_authoritative.append(matches[-1])
if len(crab_authoritative)==5:
    matrix=OUT/('integration-external-crab-matrix-'+crab_sha)/'run-1.log';assert 'test every_roll_across_current_levels_member_and_rates_has_exact_zero_xp_failure ... ok' in matrix.read_text()
    add('OR-MIG-0068',crab_sha,'V1','PASS','Applicable authoritative requirements complete:310272controlled rolls/no-input-mutation/one draw and zero-XP failure advancement;30pre-RNG refusal probes;79actual-cache metadata refusals;48mixed/uniform full-bag World batches,56admission,56late cancellation,16life and112all-count1-28cases. Complete per-step inventory/Stats, member/current/rate, captured caps/order/no fallback, first3/repeat3clocks and per-success406fineXP at125%verified. Runtime token behavior is separately tested by genuine native Close/stale frames, not replaying trusted World descriptors.',[p.relative_to(ROOT).as_posix() for p in crab_authoritative]+[matrix.relative_to(ROOT).as_posix()],scope='crab-authoritative-obligations-complete')
crab_durable=[OUT/folder/run/'independent-assessment.json' for folder,run in [('deep-crab-carving','native-001'),('deep-crab-interrupt','interrupt-002'),('deep-crab-crash','crash-001')]]
if all(p.exists() and json.loads(p.read_text()).get('status')=='PASS' for p in crab_durable):
    add('OR-MIG-0068',crab_sha,'V3','PASS','Applicable durability complete:24mixed native outputs/14625fineXP/Craft18persist through actual acknowledged save/process restart/independently decoded reconnect; consistent partial All preserves one0XPbroken product and22raws without resumed work. Separate owned unacknowledged termination restores prior complete inventory/Stats/energy on SQL and actual native restart. Stale frames, World death/cancellation and all-count caps separately pass; original startup failures preserved and sequential retry continuity proven. No mid-SQL injection claimed.',[p.relative_to(ROOT).as_posix() for p in crab_durable],scope='crab-durable-obligations-complete')
path=OUT/'deep-battlestaves/native-001/independent-assessment.json'
if path.exists():
    result=json.loads(path.read_text());assert result['status']=='PASS'
    ev=[path.relative_to(ROOT).as_posix(),'docs/openrune/verification/independent/integration-external-battlestaff-bindings-'+result['tested_sha']+'/manifest.json']
    add('OR-MIG-0067',result['tested_sha'],'V0','PASS','Exact67server build/startup succeeds against95hash-attested resources. Actual cache nine ordinary items/Crafting/sequence/four spot aliases and107factory refusals pass; real native actions dispatch all4registered pairs.',ev,scope='battlestaff-runtime-obligations-complete')
    add('OR-MIG-0067',result['tested_sha'],'V2','PASS',result['scope']+' All8native effect blocks independently identify7531and1370/1371/1372/306with2receive clocks to credit. DisplayXPfloor and exact durable fineXP agree; no fabricated sound, rendered-frame or acquisition claim. V1life defect remains separate and whole-contract acceptance withheld.',ev,scope='battlestaff-native-obligations-complete')
    add('OR-MIG-0067',result['tested_sha'],'V3','IN_PROGRESS','Eight normal outputs/47500fineXP and remaining ingredients survive actual acknowledged-save/process restart and independently decoded native reconnect. Movement/Close/stale controls pass. This positive durable scope does not accept the separate failing death interruption boundary.',ev,scope='battlestaff-normal-durable-subcheck')
    refs=OUT/'reference-battlestaves/comparison.json';ref=json.loads(refs.read_text());assert ref['source_documentation_subcheck']=='PASS'
    add('OR-MIG-0067',result['tested_sha'],'V4','PASS',ref['scope']+' Actual native sequence7531/four declared spot IDs and2clock manual credit align with unshortened source/manual Wiki2. Make-X first3/repeat2, acquisition/combat and unavailable source sound/rendered/original-executable observations explicitly outside this manual adapter. Native/headless per-award fractional rewards match separate independent calculations.',ev+[refs.relative_to(ROOT).as_posix()],scope='battlestaff-reference-obligations-complete')
path=OUT/'deep-battlestaff-crash/crash-001/independent-assessment.json'
if path.exists():
    result=json.loads(path.read_text());assert result['status']=='PASS'
    add('OR-MIG-0067',result['tested_sha'],'V3','IN_PROGRESS',result['scope']+' Whole prior inventory/Stats restored, no durable product or XP duplication. This unacknowledged-state recovery is not a mid-SQL fault or death-interruption acceptance.',[path.relative_to(ROOT).as_posix()],scope='battlestaff-unsaved-recovery-subcheck')
path=OUT/'battlestaff-life-finding.json'
if path.exists():
    result=json.loads(path.read_text());assert result['status']=='CONFIRMED_AUTHORITATIVE_BOUNDARY_FAILURE'
    add('OR-MIG-0067',result['tested_sha'],'V1','FAIL','PG-DEF-002: four isolated publicWorld lifecycle failures consume selected water-orb/staff and award1250fineXP at zeroHP or after actual queued death with visibleHP restored, both fresh action and pending completion. Modal cancellation8cases passes; remaining death profiles/directions and native reachability unproven. Ordinary P2 correction backlog, not FORGE-stop; retain all independent passing scopes and withhold aggregate acceptance.',[path.relative_to(ROOT).as_posix()],scope='battlestaff-life-refusal-required-failure')
    add('OR-MIG-0067',result['tested_sha'],'V3','FAIL','PG-DEF-002: pending precise assembly still credits after zeroHP or queued death with visibleHP restored. Normal acknowledged save/restart and unsaved-result recovery pass separately, but applicable interrupted-work cancellation cannot be accepted at this historical SHA. Requires published correction/retest identity; no newer target substituted.',[path.relative_to(ROOT).as_posix()],scope='battlestaff-life-interruption-required-failure')
path=OUT/'sinew-acceptance.json'
if path.exists():
    result=json.loads(path.read_text());assert result['status']=='ACCEPTED'
    for level,value in result['levels'].items():add('OR-MIG-0066',result['tested_sha'],level,value['status'],value['reason'],[path.relative_to(ROOT).as_posix()],scope='sinew-applicable-obligations-complete')
path=OUT/'crab-custom-input-dependency.json'
if path.exists():
    result=json.loads(path.read_text());assert result['status']=='BLOCKED'
    assert all((ROOT/p).exists() for p in result['completed_exact_target_evidence'])
    add('OR-MIG-0068',result['tested_sha'],'V2','BLOCKED','Required native custom-input obligation remains unaccepted: PG-INFRA-009client focus/keyboard continuation dependency has no attested correction. Exact68native1/Allboth profiles,5/10shell packets,39normal outputs/restarts, Close/stale/partial/crash scopes PASS. No exact68Xfailure is alleged or inferred from another server; retain its full V0/V1/V3/V4PASS and proceed to other eligible work without newer-target substitution.',[path.relative_to(ROOT).as_posix()],scope='crab-native-custom-input-dependency-blocked')
for name in ['gem-acceptance.json','shell-acceptance.json','immediate-acceptance.json','sinew-acceptance.json']:
    path=OUT/name
    if name=='immediate-acceptance.json' and (OUT/'immediate-acceptance-withdrawal.json').exists():continue
    if not path.exists():continue
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='ACCEPTED' and all(v['status']=='PASS' for v in result['levels'].values())
    attempt='accepted-manual-'+result['implementation_id']+'-'+result['tested_sha']+'-r'+str(result['revision'])
    if attempt in known:continue
    row={'attempt_id':attempt,'implementation_id':result['implementation_id'],'implementation_commit_sha':result['tested_sha'],'contract_revision':result['revision'],'campaign_kind':'historical','outcome':'VERIFIED','level':'V0-V4 applicable manual subset','independent_verification':True,'tests_passed':0,'coverage':[k+' PASS: '+v['reason'] for k,v in result['levels'].items()],'reason':result['acceptance_scope']+'; aggregate acceptance only, no duplicate test-execution count. '+result['exclusions'],'evidence':[path.relative_to(ROOT).as_posix()],'issues':[]}
    with coarse.open('a',encoding='utf-8',newline='\n') as f:f.write(json.dumps(row)+'\n')
finding=OUT/'immediate-life-finding.json'
if finding.exists():
    result=json.loads(finding.read_text());assert result['status']=='CONFIRMED_AUTHORITATIVE_BOUNDARY_FAILURE'
    for level in ['V1','V3']:add('OR-MIG-0065',result['tested_sha'],level,'FAIL','PG-DEF-003: eight separate life refusal cases fail across both immediate recipes/directions/zeroHP/actual queued death restoredHP. Full selected raw replaced; limestone300fineXP at500. Four alive controls pass, no delayed replay. Aggregate acceptance withdrawn; prior V0/V2/V4 and successful scopes retained. Native reachability/currentHEAD unproven; ordinary P2 correction.',[finding.relative_to(ROOT).as_posix(),'docs/openrune/verification/independent/immediate-acceptance-withdrawal.json'],scope='immediate-life-acceptance-correction')
    attempt='independent-PG-DEF-003-'+result['tested_sha']
    if attempt not in known:
        row={'attempt_id':attempt,'implementation_id':'OR-MIG-0065','implementation_commit_sha':result['tested_sha'],'contract_revision':1,'campaign_kind':'historical','outcome':'FAILED','level':'V1/V3 life refusal','independent_verification':True,'tests_passed':0,'coverage':['Eight individually isolated life failures, four alive controls.'],'reason':'PG-DEF-003; aggregate acceptance withdrawn append-only; preserve earlier successful scopes. No native reachability or currentHEAD inference.','evidence':[finding.relative_to(ROOT).as_posix(),'docs/openrune/verification/independent/immediate-acceptance-withdrawal.json'],'issues':['PG-DEF-003 ordinary P2 local backlog']}
        with coarse.open('a',encoding='utf-8',newline='\n') as f:f.write(json.dumps(row)+'\n')
path=OUT/'pair-authoritative-complete.json'
if path.exists():
    result=json.loads(path.read_text());assert result['status']=='PASS_APPLICABLE_V1'
    add('OR-MIG-0069',result['tested_sha'],'V1','PASS',result['scope']+' '+result['partial_campaign_boundary'],[path.relative_to(ROOT).as_posix()],scope='pair-authoritative-obligations-complete')
path=OUT/'pair-stringing-native-timing.json'
if path.exists():
    result=json.loads(path.read_text());assert result['status']=='PASS_SCOPED_TIMING_OBSERVATIONS'
    for level in ['V2','V3','V4']:add('OR-MIG-0069',result['tested_sha'],level,'IN_PROGRESS','Native14stringing credits/2800fineXP are independently observed in raw18/46and saved SQL; fourteen exact server traces first1. Client submission-to-receive clocks are1/2/4/5and include asynchronous transport/runtime latency, not asserted equal to nominal1. Producer rawFAIL/124after all14credits and stale frames is retained; final driver assertion not reached. Whole14product save acknowledged; real reconnect remains resource-deferred and unaccepted until separately assessed.',[path.relative_to(ROOT).as_posix(),'docs/openrune/verification/independent/deep-pair-stringing/native-002/produce/manifest.json'],scope='native-pair-stringing-completed-credit-subcheck')
for feature,run in [('pair-stringing','native-002'),('pair-stringing-reverse','native-001'),('pair-staves','native-001'),('pair-counts','native-001'),('pair-ten','native-001'),('pair-x','native-001'),('pair-earth-batch','native-001'),('pair-pending','native-001'),('pair-interrupt','native-001'),('pair-crash','native-001')]:
    path=OUT/('deep-'+feature)/run/'independent-assessment.json'
    if not path.exists():continue
    result=json.loads(path.read_text());assert result['status']=='PASS'
    for level in (['V3'] if feature in ['pair-interrupt','pair-crash','pair-pending'] else ['V2','V3','V4']):
        if level=='V3' and (OUT/'pair-durable-complete.json').exists():continue
        add('OR-MIG-0069',result['tested_sha'],level,'IN_PROGRESS',result['scope']+' Applicable whole-level/whole-contract obligations remain separate; no inherited PASS or chemistry execution.',[path.relative_to(ROOT).as_posix()],scope='native-'+feature+'-independent')
pair_native=[OUT/'deep-pair-stringing/native-002/independent-assessment.json',OUT/'deep-pair-staves/native-001/independent-assessment.json']
if all(p.exists() and json.loads(p.read_text()).get('status')=='PASS' for p in pair_native):
    binding=OUT/'integration-external-pair-production-bindings-fb348dda184b36ec2d503bfe0f1d5393cf8ae151/manifest.json';assert json.loads(binding.read_text())['status']=='PASS'
    add('OR-MIG-0069','fb348dda184b36ec2d503bfe0f1d5393cf8ae151','V0','PASS','Applicable startup obligations complete: exact69server build/actual startup against95hash-attested resources,38unique actual item definitions/Crafting12/sequence7531/four spots/all18factory descriptors and421metadata refusals. Real registered quantity dispatch reaches all14stringing and4staff recipes; normal two-account native reconnect independently decoded. Shared old APIs/legacy timing/boost controls128cases pass separately. Original producer timeout retained; it does not replace observed18runtime registrations with a whole-driver PASS.',[p.relative_to(ROOT).as_posix() for p in pair_native]+[binding.relative_to(ROOT).as_posix()],scope='pair-runtime-obligations-complete')
path=OUT/'reference-crafting-finishes/comparison.json'
if path.exists():
    result=json.loads(path.read_text());assert result['source_documentation_subcheck']=='PASS'
    add('OR-MIG-0070',result['tested_sha'],'V4','IN_PROGRESS',result['scope']+' Exact World/cache/startup/native/lifecycle and donor full category dispatch remain untested; excludes helmet charging and chemistry.',[path.relative_to(ROOT).as_posix()],scope='independent-crafting-finishes-reference-facts')
path=OUT/'reference-lead-smelting/comparison.json'
if path.exists():
    result=json.loads(path.read_text());assert result['source_documentation_subcheck']=='PASS_WITH_EXPLICIT_TIMING_AND_RATE_LIMITATIONS'
    add('OR-MIG-0096',result['tested_sha'],'V4','IN_PROGRESS',result['scope']+' This verifies only reference/target declaration facts. Native first/repeat/single clocks, current rate integration, World/cache/startup and persistence remain unaccepted; excludes other furnaces/modifiers.',[path.relative_to(ROOT).as_posix()],scope='independent-lead-smelting-reference-facts')
path=OUT/'pair-reference-routing-addendum.json'
if path.exists():
    result=json.loads(path.read_text());assert result['status']=='REFERENCE_QUALIFICATION_REQUIRED'
    add('OR-MIG-0069',result['tested_sha'],'V4','IN_PROGRESS','PG-REF-002: four pinned source hashes/eighteen source rows verify13AmuletStringing2routes plus Rabbit Combining1/COMBINE with no tick override. Source default first shortening/phase compensation explicitly checked. Target Rabbit1/2is Wiki-supported and World-tested; overbroad source2inheritance needs qualification. No game defect/FORGE stop alleged. Keep final V4comparison partial; original39reference artifact remains immutable.',[path.relative_to(ROOT).as_posix()],scope='independent-pair-reference-routing-qualification')
path=OUT/'pair-durable-complete.json'
if path.exists():
    result=json.loads(path.read_text());assert result['status']=='PASS_APPLICABLE_V3'
    add('OR-MIG-0069',result['tested_sha'],'V3','PASS',result['scope']+' Original producer timeout/mid-SQL/equipment/rendered/currentHEAD limitations remain explicit; no whole-contract acceptance inferred.',[path.relative_to(ROOT).as_posix()],scope='pair-durable-obligations-complete')
path=OUT/'pair-reference-complete.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS_APPLICABLE_REFERENCE_COMPARISON'
    add('OR-MIG-0069',result['tested_sha'],'V4','PASS',result['scope']+' Source13AmuletStringing2versusRabbitCombining1and Wiki-backed first1/repeat2adaptation explicit; staff manual2versusquantity3/2and per-credit fine rounding compared with native traces/effects. No original executable/rendered/audio/currentHEAD or whole-contract acceptance inferred.',[path.relative_to(ROOT).as_posix()],scope='pair-reference-obligations-complete')
path=OUT/'crafting-finishes-reference-complete.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS_APPLICABLE_REFERENCE_COMPARISON'
    add('OR-MIG-0070',result['tested_sha'],'V4','PASS',result['scope']+' Actual four native wiring traces are synchronous; manual helm descriptor/compensated World clocks and fineXP are compared without native donor timing parity. PG-DEF-004 V1failure remains separate; no whole-contract acceptance.',[path.relative_to(ROOT).as_posix()],scope='finishes-reference-obligations-complete')
path=OUT/'deep-crafting-finishes/native-001/independent-assessment.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS'
    binding=OUT/'integration-external-crafting-finishes-bindings-c7ddebe844820260252fb37df518f8c271f37243/manifest.json';assert json.loads(binding.read_text())['status']=='PASS'
    ev=[path.relative_to(ROOT).as_posix(),binding.relative_to(ROOT).as_posix()]
    add('OR-MIG-0070',result['tested_sha'],'V0','PASS','Exact70server compiles and actually starts against95immutable resource hashes; six actual cache item bindings/Crafting/two descriptors and67metadata refusals PASS. Genuine native wiring/manual-helm held routes reach both startup registrations in both directions.',ev,scope='finishes-runtime-obligations-complete')
    add('OR-MIG-0070',result['tested_sha'],'V2','PASS','Applicable native manual routes complete:four wiring/four helm products in both directions, exact raw held frames/whole inventory/44800fineXP, uncharged12929zero vars, retained chisels/duplicate visages/coins, pending helm Walk cancellation/eight stale pair refusals and no quantity115or repeat. Actual restarted native state independently decoded. No rendered/audio/original executable or native death-reachability claim.',ev,scope='finishes-native-obligations-complete')
    add('OR-MIG-0070',result['tested_sha'],'V3','IN_PROGRESS','Eight completed normal native products and44800fineXP persist through acknowledged-save/process restart/native reconnect with exact SQL inventory/Stats/remaining materials/coins/energy and fractional existing fineXP. Pending action Walk cancellation passes; separate owned unsaved-result recovery remains open. PG-DEF-004 V1failure prevents aggregate acceptance.',ev,scope='finishes-normal-durable-subcheck')
path=OUT/'crafting-finishes-disposition.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='FAILED_REQUIRES_CORRECTION'
    add('OR-MIG-0070',result['tested_sha'],'V3','PASS',result['levels']['V3']['reason']+' Exact historical storage scope only; V1FAILPG-DEF-004prevents aggregate acceptance.',[path.relative_to(ROOT).as_posix()],scope='finishes-durable-obligations-complete')
    ledger=DOC/'VERIFICATION_RESULTS.jsonl';history=[json.loads(s) for s in ledger.read_text(encoding='utf-8').splitlines()];attempt='finishes-terminal-disposition-'+result['tested_sha']
    if not any(r['attempt_id']==attempt for r in history):
        row={'attempt_id':attempt,'implementation_id':'OR-MIG-0070','implementation_commit_sha':result['tested_sha'],'contract_revision':1,'campaign_kind':'historical','outcome':'FAILED','level':'V0PASS/V1FAIL/V2PASS/V3PASS/V4PASS','independent_verification':True,'tests_passed':0,'coverage':[k+' '+v['status']+': '+v['reason'] for k,v in result['levels'].items()],'reason':result['scope'],'evidence':[path.relative_to(ROOT).as_posix()],'issues':['PG-DEF-004 ordinary P2 local correction; no external issue or FORGE stop']}
        with ledger.open('a',encoding='utf-8',newline='\n') as f:f.write(json.dumps(row)+'\n')
path=OUT/'deep-amethyst/native-001/independent-assessment.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS'
    ev=[path.relative_to(ROOT).as_posix()]
    add('OR-MIG-0071',result['tested_sha'],'V0','PASS','Exact71server compiles shared counted exchange/all startup constructors and starts against95immutable resource hashes. Six actual cache bindings/Crafting/sequence6295symbol/four counted descriptors and69refusals pass; actual native four-option dispatch/count labels reaches every recipe in both directions. Count-one shared engine regressions separately pass.',ev,scope='amethyst-runtime-obligations-complete')
    for level in ['V2','V3','V4']:add('OR-MIG-0071',result['tested_sha'],level,'IN_PROGRESS',result['scope']+' Native builtin batches/custom quantities/negative and partial/unsaved persistence/effect-clock/audio/reference scope remain separately unaccepted.',ev,scope='amethyst-native-four-choices-normal')
path=OUT/'amethyst-authoritative-complete.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS_APPLICABLE_V1'
    add('OR-MIG-0071',result['tested_sha'],'V1','PASS',result['scope']+' Native/runtime token and full contract acceptance remain separate.',[path.relative_to(ROOT).as_posix()],scope='amethyst-authoritative-obligations-complete')
path=OUT/'amethyst-durable-complete.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS_APPLICABLE_V3'
    add('OR-MIG-0071',result['tested_sha'],'V3','PASS',result['scope']+' '+result['account_scope']+' Native custom input/audio/reference and whole-contract acceptance remain separate.',[path.relative_to(ROOT).as_posix()],scope='amethyst-durable-obligations-complete')
path=OUT/'member-tanning-world-assessment.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS_SCOPED_WORLD_CASES'
    add('OR-MIG-0106',result['tested_sha'],'V1','IN_PROGRESS',result['scope'],[path.relative_to(ROOT).as_posix()],scope='member-tanning-scoped-world-assessment')
path=OUT/'amethyst-remaining-obligations.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'))
    for level in ['V2','V4']:add('OR-MIG-0071',result['tested_sha'],level,result[level],result[level+'_reason'],[path.relative_to(ROOT).as_posix()],scope='amethyst-remaining-obligations')
path=OUT/'deep-light-sources/native-002/independent-assessment.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS'
    for level,status in [('V2','PASS'),('V3','IN_PROGRESS')]:
        add('OR-MIG-0084',result['tested_sha'],level,status,result['scope']+' Separate Use/Cancel/unsaved recovery, actual cache bindings and full acceptance remain unaccepted; no donor executable/render/audio parity.',[path.relative_to(ROOT).as_posix()],scope='light-native-normal-obligations')
path=OUT/'reference-lights/comparison.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS_REFERENCE_FACTS'
    add('OR-MIG-0084',result['tested_sha'],'V4','IN_PROGRESS',result['scope']+' Reference-fact subcheck only; target-specific native/cache/feedback/lifecycle acceptance remains separate.',[path.relative_to(ROOT).as_posix()],scope='light-reference-facts')
path=OUT/'light-life-pg-def-003-addendum.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='CONFIRMED_AUTHORITATIVE_BOUNDARY_FAILURE'
    for level in ['V1','V3']:add('OR-MIG-0084',result['tested_sha'],level,'FAIL',result['scope']+' Effective-life refusal is required before fresh/pending held completion; native dead-packet reachability/currentHEAD untested.',[path.relative_to(ROOT).as_posix()],scope='light-life-pg-def-003-required-failure')
path=OUT/'light-sources-disposition.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='FAILED'
    for level,value in result['levels'].items():add('OR-MIG-0084',result['tested_sha'],level,value['status'],value['reason'],[path.relative_to(ROOT).as_posix()],scope='light-terminal-disposition')
    coarse_path=DOC/'VERIFICATION_RESULTS.jsonl';attempt='light-terminal-disposition-'+result['tested_sha']
    if not any(json.loads(s)['attempt_id']==attempt for s in coarse_path.read_text(encoding='utf-8').splitlines()):
        row={'attempt_id':attempt,'implementation_id':'OR-MIG-0084','implementation_commit_sha':result['tested_sha'],'contract_revision':1,'campaign_kind':'historical','outcome':'FAILED','level':'V0PASS/V1FAIL/V2PASS/V3FAIL/V4PASS','independent_verification':True,'tests_passed':0,'coverage':[k+' '+v['status']+': '+v['reason'] for k,v in result['levels'].items()],'reason':result['scope'],'evidence':[path.relative_to(ROOT).as_posix()],'issues':['PG-DEF-003 ordinary P2 local correction addendum; no new issue or FORGE stop']}
        with coarse_path.open('a',encoding='utf-8',newline='\n') as f:f.write(json.dumps(row)+'\n')
path=OUT/'imcando-held-world-partial.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PARTIAL'
    for level in ['V0','V1']:add('OR-MIG-0108',result['tested_sha'],level,'IN_PROGRESS',result['scope']+' '+result['native_state'],[path.relative_to(ROOT).as_posix()],scope='imcando-held-world-partial')
for folder,ident in [('reference-imcando-held','OR-MIG-0108'),('reference-imcando-worn','OR-MIG-0109'),('reference-guild-bank','OR-MIG-0110')]:
    path=OUT/folder/'comparison.json'
    if path.exists():
        result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS_REFERENCE_FACTS'
        add(ident,result['tested_sha'],'V4','IN_PROGRESS',result['scope']+' Native/cache/World/lifecycle and complete reference acceptance remain separate.',[path.relative_to(ROOT).as_posix()],scope='independent-'+folder+'-facts')
path=OUT/'imcando-held-authoritative-complete.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS_APPLICABLE_V1'
    add('OR-MIG-0108',result['tested_sha'],'V1','PASS',result['scope'],[path.relative_to(ROOT).as_posix()],scope='imcando-held-authoritative-obligations-complete')
path=OUT/'imcando-worn-authoritative-partial.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'))
    for level in ['V0','V1']:add('OR-MIG-0109',result['tested_sha'],level,'IN_PROGRESS',result['scope']+' Remaining: '+'; '.join(result['remaining']),[path.relative_to(ROOT).as_posix()],scope='imcando-worn-independent-partial')
path=OUT/'imcando-worn-authoritative-complete.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS_APPLICABLE_V1'
    add('OR-MIG-0109',result['tested_sha'],'V1','PASS',result['scope'],[path.relative_to(ROOT).as_posix()],scope='imcando-worn-applicable-authoritative-complete')
path=OUT/'imcando-held-disposition.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='ACCEPTED_HISTORICAL'
    for level,value in result['levels'].items():add('OR-MIG-0108',result['tested_sha'],level,value['status'],value['reason'],[path.relative_to(ROOT).as_posix()],scope='imcando-held-applicable-obligations-complete')
    coarse_path=DOC/'VERIFICATION_RESULTS.jsonl';attempt='imcando-held-applicable-obligations-complete-'+result['tested_sha']
    if not any(json.loads(s)['attempt_id']==attempt for s in coarse_path.read_text(encoding='utf-8').splitlines()):
        row={'attempt_id':attempt,'implementation_id':'OR-MIG-0108','implementation_commit_sha':result['tested_sha'],'contract_revision':1,'campaign_kind':'historical','outcome':'VERIFIED','level':'V0PASS/V1PASS/V2PASS/V3PASS/V4PASS','independent_verification':True,'tests_passed':0,'coverage':[k+' '+v['status']+': '+v['reason'] for k,v in result['levels'].items()],'reason':result['scope'],'evidence':[path.relative_to(ROOT).as_posix()],'issues':[]}
        with coarse_path.open('a',encoding='utf-8',newline='\n') as f:f.write(json.dumps(row)+'\n')
path=OUT/'imcando-worn-disposition.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='FAILED_REQUIRES_CORRECTION'
    for level,value in result['levels'].items():add('OR-MIG-0109',result['tested_sha'],level,value['status'],value['reason'],[path.relative_to(ROOT).as_posix()],scope='imcando-worn-native-disposition')
    coarse_path=DOC/'VERIFICATION_RESULTS.jsonl';attempt='imcando-worn-native-disposition-'+result['tested_sha']
    if not any(json.loads(s)['attempt_id']==attempt for s in coarse_path.read_text(encoding='utf-8').splitlines()):
        row={'attempt_id':attempt,'implementation_id':'OR-MIG-0109','implementation_commit_sha':result['tested_sha'],'contract_revision':1,'campaign_kind':'historical','outcome':'FAILED','level':'V0FAIL/V1PASS/V2FAIL/V3BLOCKED/V4PASS','independent_verification':True,'tests_passed':0,'coverage':[k+' '+v['status']+': '+v['reason'] for k,v in result['levels'].items()],'reason':result['scope'],'evidence':[path.relative_to(ROOT).as_posix()],'issues':['PG-DEF-006 ordinary P2local correction; no FORGE stop/newissue']}
        with coarse_path.open('a',encoding='utf-8',newline='\n')as f:f.write(json.dumps(row)+'\n')
path=OUT/'rune-altars-world-partial.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'))
    for level in ['V0','V1']:add('OR-MIG-0125',result['tested_sha'],level,'IN_PROGRESS',result['scope']+' Remaining: '+'; '.join(result['remaining']),[path.relative_to(ROOT).as_posix()],scope='rune-altars-independent-world-partial')
path=OUT/'rune-altars-stock-finding.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='CONFIRMED_IMPLEMENTATION_DEFECT'
    add('OR-MIG-0125',result['tested_sha'],'V1','FAIL',result['scope']+' '+result['correction']+' Trusted restored-state fault injection; native reachability/durability untested.',[path.relative_to(ROOT).as_posix()],scope='rune-altars-stock-metadata-finding')
path=OUT/'rune-altars-stock-correction-126.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS_SCOPED_CORRECTION'
    for level in ['V0','V1']:add('OR-MIG-0126',result['tested_sha'],level,'IN_PROGRESS',result['scope']+' Other126requirements remain unverified.',[path.relative_to(ROOT).as_posix()],scope='rune-altars-stock-correction-126')
print('Per-level evidence events:',len(rows))

path=OUT/'rune-altars-additional-world.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'))
    add('OR-MIG-0125',result['tested_sha'],'V0','IN_PROGRESS',result['scope']+' Exact server/native runtime separately attested; remaining applicable obligations open.',[path.relative_to(ROOT).as_posix()],scope='rune-altars-guards-cache-boundaries')
path=OUT/'daeyalt-world-partial.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'))
    add('OR-MIG-0126',result['tested_sha'],'V1','IN_PROGRESS',result['scope']+' Remaining: '+'; '.join(result['remaining']),[path.relative_to(ROOT).as_posix()],scope='daeyalt-independent-world-partial')
for path in sorted(OUT.glob('deep-rune-*/*/independent-assessment.json')):
    result=json.loads(path.read_text(encoding='utf-8'))
    if result['status']!='PASS_SCOPED' or result['implementation_id']!='OR-MIG-0125':continue
    for level in (['V3'] if (OUT/'rune-altars-native-complete.json').exists() else ['V2','V3']):add('OR-MIG-0125',result['tested_sha'],level,'IN_PROGRESS',result['scope']+' Remaining: '+'; '.join(result['limitations']),[path.relative_to(ROOT).as_posix()],scope='rune-native-'+result['feature']+'-'+path.parent.name)

for filename,level,scope in [('daeyalt-guards-partial.json','V1','daeyalt-initial-late-guards'),('daeyalt-bindings-partial.json','V0','daeyalt-cache-bindings'),('reference-daeyalt/comparison.json','V4','daeyalt-reference-facts')]:
    path=OUT/filename
    if path.exists():
        result=json.loads(path.read_text(encoding='utf-8'))
        add('OR-MIG-0126',result['tested_sha'],level,'IN_PROGRESS',result['scope']+' Scoped evidence only; complete applicable obligations remain open.',[path.relative_to(ROOT).as_posix()],scope=scope)

path=OUT/'rune-altars-runtime-v0.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS_APPLICABLE_V0'
    add('OR-MIG-0125',result['tested_sha'],'V0','PASS',result['scope'],[path.relative_to(ROOT).as_posix()],scope='rune-altars-runtime-v0-complete')

path=OUT/'daeyalt-kernel-partial.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'))
    add('OR-MIG-0126',result['tested_sha'],'V1','IN_PROGRESS',result['scope']+' '+result['limitations'][0],[path.relative_to(ROOT).as_posix()],scope='daeyalt-kernel-partial')

path=OUT/'daeyalt-live-and-regression-partial.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'))
    add('OR-MIG-0126',result['tested_sha'],'V1','IN_PROGRESS',result['scope']+' '+result['limitations'][0],[path.relative_to(ROOT).as_posix()],scope='daeyalt-live-and-ordinary-regression')
path=OUT/'deep-rune-mind-crash/native-001/independent-assessment.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS'
    add('OR-MIG-0125',result['tested_sha'],'V3','IN_PROGRESS',result['scope']+' Normal saved/restarted altars assessed separately; remaining native interruption/late-context/stock scopes remain open. No during-SQL-commit injection.',[path.relative_to(ROOT).as_posix()],scope='rune-unsaved-recovery')

for filename,level,scope in [('rune-altars-native-complete.json','V2','rune-altars-native-complete'),('rune-altars-reference-complete.json','V4','rune-altars-reference-complete')]:
    path=OUT/filename
    if path.exists():
        result=json.loads(path.read_text(encoding='utf-8'))
        add('OR-MIG-0125',result['tested_sha'],level,'PASS',result['scope']+' '+'; '.join(result['limitations']),[path.relative_to(ROOT).as_posix()],scope=scope)

path=OUT/'daeyalt-world-v1-complete.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS' and result['level']=='V1'
    add('OR-MIG-0126',result['tested_sha'],'V1','PASS',result['scope']+' '+result['limitations'][0],[path.relative_to(ROOT).as_posix()],scope='daeyalt-world-v1-complete')
for path in OUT.glob('deep-rune-mind-*/*/independent-assessment.json'):
    result=json.loads(path.read_text(encoding='utf-8'))
    if result.get('implementation_id')!='OR-MIG-0126' or result.get('status')!='PASS_SCOPED':continue
    for level in ['V2','V3']:
        add('OR-MIG-0126',result['tested_sha'],level,'IN_PROGRESS',result['scope']+' Separate applicable acceptance obligations remain; no native125rate claim.',[path.relative_to(ROOT).as_posix()],scope='daeyalt-native-'+result.get('feature',path.parent.parent.name)+'-'+path.parent.name)

path=OUT/'daeyalt-runtime-v0-complete.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS' and result['level']=='V0'
    add('OR-MIG-0126',result['tested_sha'],'V0','PASS',result['scope']+' '+'; '.join(result['limitations']),[path.relative_to(ROOT).as_posix()],scope='daeyalt-runtime-v0-complete')
for manifest in OUT.glob('integration-external-crafting-cape-*/manifest.json'):
    result=json.loads(manifest.read_text(encoding='utf-8'))
    if result['status']!='PASS':continue
    suite=result['suite']
    if suite not in ['crafting-cape-world','crafting-cape-bindings']:continue
    level='V0' if suite.endswith('-bindings') else 'V1'
    reason=('Fresh actual local target cape/stat/parameter definitions and malformed-binding refusals at exact130SHA; actual server/native runtime dispatch and remaining acceptance separate.' if level=='V0' else '144independent publicWorld cases:96both-cape/current0-1-50-98-99-100-104-120/member-F2P/rate1-1.25-5 combinations with three explicit Boost requests each;48late member/base/slot/identity/count/vars/catalog/modal/delay/HP/config/logout guards. WholebaggearStats/fineXP andzeroRNG checked. No native equip/boost, duration, persistence or completeV1claim; delay refusal observed while delay active.')
    add('OR-MIG-0130',result['tested_integration_sha'],level,'IN_PROGRESS',reason,[manifest.relative_to(ROOT).as_posix()],scope=suite)

path=OUT/'crafting-cape-expiry-partial.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS_SCOPED'
    add('OR-MIG-0130',result['tested_sha'],'V1','IN_PROGRESS',result['scope']+' '+'; '.join(result['limitations']),[path.relative_to(ROOT).as_posix()],scope='crafting-cape-public-timer-expiry')

path=OUT/'reference-skillcape-vendors/comparison.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS_REFERENCE_FACTS'
    add('OR-MIG-0131',result['tested_sha'],'V4','IN_PROGRESS',result['scope']+' PG-REF-003: membership provenance qualification; target native/transaction/lifecycle reference comparison remains open.',[path.relative_to(ROOT).as_posix()],scope='skillcape-vendor-reference-facts')

path=OUT/'crafting-cape-world-v1-complete.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS' and result['level']=='V1'
    add('OR-MIG-0130',result['tested_sha'],'V1','PASS',result['scope']+' '+'; '.join(result['qualification']),[path.relative_to(ROOT).as_posix()],scope='crafting-cape-world-v1-complete')

path=OUT/'reference-cooking-cape/comparison.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS_REFERENCE_FACTS'
    add('OR-MIG-0132',result['tested_sha'],'V4','IN_PROGRESS',result['scope']+' '+'; '.join(result['limitations']),[path.relative_to(ROOT).as_posix()],scope='cooking-cape-reference-facts')

path=OUT/'reference-crafting-cape/comparison.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS_REFERENCE_FACTS'
    add('OR-MIG-0130',result['tested_sha'],'V4','IN_PROGRESS',result['scope']+' Candidate mapping/native/runtime/qualified drained behavior remain separate.',[path.relative_to(ROOT).as_posix()],scope='crafting-cape-reference-facts')

path=OUT/'reference-cooking-gauntlets/comparison.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS_REFERENCE_FACTS'
    add('OR-MIG-0133',result['tested_sha'],'V4','IN_PROGRESS',result['scope']+' '+'; '.join(result['limitations']),[path.relative_to(ROOT).as_posix()],scope='cooking-gauntlets-reference-facts')

path=OUT/'crafting-cape-v1-acceptance-correction.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='IN_PROGRESS'
    add('OR-MIG-0130',result['tested_sha'],'V1','IN_PROGRESS',result['scope'],[path.relative_to(ROOT).as_posix()],scope='crafting-cape-v1-acceptance-correction')

path=OUT/'crafting-cape-v1-resolved.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS' and result['level']=='V1'
    add('OR-MIG-0130',result['tested_sha'],'V1','PASS',result['scope']+' '+result['acceptance_history']+' '+'; '.join(result['qualifications']),[path.relative_to(ROOT).as_posix()],scope='crafting-cape-v1-obligations-resolved')

path=OUT/'zealot-selected-life-finding.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'))
    add('OR-MIG-0134',result['tested_sha'],'V1','FAIL',result['scope']+' Positive16602transaction/25World controls retained; no completed V0/V2/V3/V4 claim.',[path.relative_to(ROOT).as_posix()],scope='zealot-selected-life-acceptance')
path=OUT/'crafting-cape-runtime-v0-complete.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'))
    add('OR-MIG-0130',result['tested_sha'],'V0','PASS',result['scope']+' '+'; '.join(result['qualifications']),[path.relative_to(ROOT).as_posix()],scope='crafting-cape-runtime-v0-complete')
path=OUT/'reference-zealot-burial/comparison.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'))
    add('OR-MIG-0134',result['tested_sha'],'V4','IN_PROGRESS',result['scope']+' '+'; '.join(result['limitations']),[path.relative_to(ROOT).as_posix()],scope='zealot-selected-reference-facts')
for feature,run in [('crafting-cape-boost','native-001'),('crafting-cape-trimmed','native-001'),('crafting-cape-drained','native-001'),('crafting-cape-expiry','native-001'),('crafting-cape-equipment','native-002'),('crafting-cape-f2p','native-001')]:
    path=OUT/('deep-'+feature)/run/'independent-assessment.json'
    if not path.exists():continue
    result=json.loads(path.read_text(encoding='utf-8'))
    add('OR-MIG-0130',result['tested_sha'],'V2','IN_PROGRESS',result['scope']+' Status '+result['status']+'; complete native/rendered obligations remain separate.',[path.relative_to(ROOT).as_posix()],scope=feature+'-'+run)
    if result['status']=='PASS_SCOPED':
        add('OR-MIG-0130',result['tested_sha'],'V3','IN_PROGRESS',result['scope']+' Acknowledged persistence/restart scope only; interruption/crash obligations remain separate.',[path.relative_to(ROOT).as_posix()],scope=feature+'-'+run)

path=OUT/'rune-ordinary-integration-130.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'));assert result['status']=='PASS_SCOPED'
    for ident in result['affected_implementations']:
        add(ident,result['tested_sha'],'V1','IN_PROGRESS',result['scope']+' '+'; '.join(result['limitations']),[path.relative_to(ROOT).as_posix()],kind='integration',scope='rune-ordinary-newer-integration',affected=result['affected_implementations'])

for filename,level,status in [('daeyalt-native-v2-complete.json','V2','PASS'),('daeyalt-lifecycle-v3-complete.json','V3','PASS'),('daeyalt-reference-v4-blocked.json','V4','BLOCKED')]:
    path=OUT/filename
    if path.exists():
        result=json.loads(path.read_text(encoding='utf-8'));assert result['status']==status and result['level']==level
        detail=' '.join(result['qualifications']) if status=='PASS' else result['specific_blocker']+' '+result['required_resolution']
        add('OR-MIG-0126',result['tested_sha'],level,status,result['scope']+' '+detail,[path.relative_to(ROOT).as_posix()],scope='daeyalt-'+level.lower()+'-obligations')

path=OUT/'crafting-cape-reference-v4-complete.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'))
    add('OR-MIG-0130',result['tested_sha'],'V4','PASS',result['scope']+' '+result['acceptance_scope']+' '+'; '.join(result['qualified_differences']),[path.relative_to(ROOT).as_posix()],scope='crafting-cape-qualified-reference-v4')

path=OUT/'skillcape-vendor-world-partial.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'))
    add('OR-MIG-0131',result['tested_sha'],'V1','IN_PROGRESS',result['scope']+' '+'; '.join(result['remaining']),[path.relative_to(ROOT).as_posix()],scope='skillcape-vendor-independent-transactions-dialogue')

path=OUT/'crafting-cape-native-v2-complete.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'))
    add('OR-MIG-0130',result['tested_sha'],'V2','PASS',result['scope']+' '+'; '.join(result['qualifications']),[path.relative_to(ROOT).as_posix()],scope='crafting-cape-native-v2-obligations')

path=OUT/'crafting-cape-lifecycle-v3-complete.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'))
    add('OR-MIG-0130',result['tested_sha'],'V3','PASS',result['scope']+' '+'; '.join(result['qualifications']),[path.relative_to(ROOT).as_posix()],scope='crafting-cape-lifecycle-v3-obligations')

path=OUT/'skillcape-vendor-world-extension.json'
if path.exists():
    result=json.loads(path.read_text(encoding='utf-8'))
    add('OR-MIG-0131',result['tested_sha'],'V1','IN_PROGRESS',result['scope']+' '+result['harness_history']+' '+'; '.join(result['remaining']),[path.relative_to(ROOT).as_posix()],scope='skillcape-vendor-owned-life-extension')
