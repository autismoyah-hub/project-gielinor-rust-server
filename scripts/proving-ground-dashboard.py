"""Render the current repository dashboard and explicit machine resume lanes."""
from collections import Counter
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification'
state=json.loads((DOC/'WORKLIST.json').read_text(encoding='utf-8'))
lines=['# PROVING GROUND — current verification status','',
       'This is an ongoing verification campaign. No contract is yet fully accepted across all applicable V0–V4 levels. Historical results and current integration checks are separate.','',
       '## Intake','',f"Latest observed FORGE HEAD: `{state['forge_head']}`.",f"Exact queue SHA256: `{state['queue_sha256']}`.",
       f"Published queue: {state['records']} records / {state['implementation_units']} stable implementation IDs. Published validator: {state['published_validator_status']}.",
       'The copied queue is refreshed only from fetched immutable blobs after append-history checks. Current coverage and every contract revision are retained in [WORKLIST.json](WORKLIST.json). Synchronization observations are append-only in [SYNC_OBSERVATIONS.jsonl](SYNC_OBSERVATIONS.jsonl).','',
       '## Level coverage','']
for level in ['V0','V1','V2','V3','V4']:
    counts=Counter(x['historical_verification']['levels'][level]['status'] for x in state['items'])
    lines.append('- '+level+': '+', '.join(k+' '+str(counts.get(k,0)) for k in ['PASS','FAIL','BLOCKED','IN_PROGRESS','NOT_STARTED','NOT_APPLICABLE'])+'.')
lines+=['','PASS applies only to the named level, target and scope. IN_PROGRESS retains passing subchecks while required acceptance cases remain open. The old coarse ledger is retained; [LEVEL_RESULTS.jsonl](LEVEL_RESULTS.jsonl) is authoritative for the new level-specific program.','',
        '## Deep evidence','',
        '- Super-combat at `5a260c1b`: six genuine native inventory/menu combinations; exact item/quantity packets and authoritative two-tick transitions; six outputs and 45,000 fine XP; acknowledged save, actual process restart and native relogin without replay. [Assessment](independent/deep-combat/independent-assessment.json).',
        '- Chemistry at `e7ac3608`: native Options→Stop→Break→Confirm; selected amulet removal, five next-amulet charges and durable stop policy; actual restart/native relogin. [Assessment](independent/deep-chemistry/independent-assessment.json).',
        '- Historical worn chemistry at `e7ac3608`: separate real native equipment Options→Stop, acknowledged save, process restart and native relogin. Worn amulet, charges, stop policy, coins and XP conserved. [Assessment](independent/deep-chemistry-worn/hist-worn-002/independent-assessment.json).',
        '- Reference comparison: seven independently hashed source/pinned Wiki records for super-combat, plus chemistry control source inspection. Documented dose, timing, boost and transaction adaptations remain distinct from literal donor behavior. [Comparison](independent/deep-combat/reference-comparison.json).',
        '- Food revisions 2 (`a560034a`) and 3 (`ae3d234c`): the four independently authored tests passed separately at both exact SHAs, with 319 synthetic fixtures per target through actual `World::submit`/`World::tick`; stale claims, duplicate/replay input, full bags, remainders, malformed inputs and overheal. Initial verifier assumptions and failed source/logs are preserved. [Test source](../../../verification/ordinary-food/tests/authoritative.rs).',
        '- Food revision3 native: meat confirmed by client; cake commit confirmed by server trace and real restarted native inventory capture (2,061 packets). The six-action driver timed out at an intermediate-HP equality check; four remaining bites were unverified in that original run; fresh instrumented observer-001 completed all six actions and real restart/relogin. Passive-regeneration diagnosis and raw failure are retained. [Partial assessment](independent/deep-food/independent-assessment.json).',
        '- Item packs at `caefcf42`: atomic selected-slot/overflow helpers, server definition bindings and authoritative repeat/movement-cancellation/stale-slot scenarios passed. Native, persistence and reference acceptance remain open.',
        '- Dynamic food at `14461673`: independent action matrix passed across 1,584 base-level/HP fixtures plus live-base/stale replay and seven rollback cases. Independently hashed three protected sources and four pinned Wiki records; strawberry prose/table and watermelon donor/Wiki differences remain explicit. [Comparison](independent/reference-food-1446167370dece10751c4c332929af880259ee23/comparison.json).',
        '- Shared batch assembly at `d63c0258`: 13 independently executed existing tests and an independently authored 1,512-case full-bag/captured-slot/direction/late-failure matrix PASS. Pair/timed routing and cancellation regressions are retained; native, persistence and full reference acceptance remain open. [Independent test source](../../../verification/assembly-batch/tests/authoritative.rs).','',
        'No rendered-window, original-executable differential or crash-atomicity acceptance is claimed. Native packet evidence uses the real pinned Rust client library/binaries and actual server transport, without desktop automation. Client binary/source correspondence relies on the preserved, hash-verified baseline/driver manifest; fresh observer campaigns use an independent rebuild of 1,674 hash-verified pinned source/vendor files with observation-only logging and owned controllers.','',
        '## Integration regression','']
for path in sorted((DOC/'independent').glob('integration-*/manifest.json')):
    m=json.loads(path.read_text(encoding='utf-8'))
    if 'status' not in m:continue
    detail=', '.join(r.get('filter',m.get('suite','scenario'))+' '+str(r['tests_passed']) for r in m.get('runs',[]) if r.get('tests_passed',0)) or '0'
    note=m.get('reason','')
    if m['status']=='FAIL' and m.get('suite')=='food-regeneration':note='Rejected same-clock diagnosis; refined later-clock reproducer passed. This failed hypothesis is retained, not attributed to gameplay.'
    if m['status']=='FAIL' and m.get('suite')=='assembly-batch':note='Verifier assumed which unselected duplicate was captured; corrected selected-slot/no-fallback matrix passed. See PG-INFRA-005.'
    lines.append(f"- `{m['tested_integration_sha']}`: raw attempt {m['status']}; "+detail+' test executions passed. '+note+' [Manifest]('+str(path.relative_to(DOC)).replace('\\','/')+').')
for path in sorted((DOC/'independent').glob('native-*/independent-assessment.json')):
    m=json.loads(path.read_text(encoding='utf-8'));lines.append('- `'+m['tested_sha']+'`: '+m['scope']+' [Assessment]('+str(path.relative_to(DOC)).replace('\\','/')+').')
lines+=['','An older integration PASS is not a PASS on a newer FORGE HEAD. Change-impact lists and the last executed integration scope are recorded separately per feature.','',
        '## Corrections and constraints','',
        '- Last-charge chemistry full-bag native production and stop-on-crumble with one pair remaining plus real restart PASS. Independent 22000-case chemistry modifier matrix PASS; complete V1-V4 obligations remain separate.',
        '- Confirmed critical/dependency-blocking gameplay findings: none in completed scoped tests.',
        '- Confirmed ordinary gameplay defect PG-DEF-001: genuine Close95 leaves chemistry policy dialog live at e7ac3608; stale Stop changes saved policy. V2 FAIL; ordinary P2, not implementation-blocking. GitHub issue123.',
        '- Verifier defects: initial filter, food trace/dirty-slot assumptions and dynamic-food types/labels corrected. PG-INFRA-004 no longer blocks execution: fresh observer captured HP2->3 regeneration before Eat, all six actions and real restart/relogin. Original unrecorded missed-equality sequence remains inferred. [Backlog](CORRECTION_BACKLOG.md).',
        '- Resource deferrals used measured CPU/RAM; later builds used one job and private targets. No unrelated process was changed.',
        '- Optional third-party JSON Schema engine is unavailable; the actual published standard-library validator passes. This is not counted as a gameplay failure.',
        '- Protected source/cache, account templates, credentials, native executables and databases stay local. Published material contains synthetic tests, hashes, scoped state snapshots and logs.','',
        '## Active work and continuation','']
for task in state['resume_queue']:lines.append('- Lane '+task['lane']+' — '+task['feature']+' at `'+task['sha']+'`: '+task['action']+'.')
lines+=['','[RESUME_QUEUE.json](RESUME_QUEUE.json) contains exact targets and all outstanding feature levels. Do not infer that work runs after the active session ends.','',
        '## Current contract coverage','']
for x in state['items']:
    levels=x['historical_verification']['levels']
    lines.append('- **'+x['implementation_id']+' r'+str(x['revision'])+'** `'+x['implementation_availability']['sha'][:12]+'` — '+x['feature_name']+'; '+', '.join(k+' '+v['status'] for k,v in levels.items())+'.')
(DOC/'DASHBOARD.md').write_text('\n'.join(lines)+'\n',encoding='utf-8',newline='\n')
print('Updated dashboard for',state['implementation_units'],'features')
