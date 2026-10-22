"""Record current-main observations separately from immutable historical acceptance."""
import csv,hashlib,json,sqlite3
from pathlib import Path
R=Path(__file__).resolve().parents[1];D=R/'docs/openrune/agent5';I=R/'docs/openrune/verification/independent';OLD=R.parent/'agent7-verification';MAIN='ec00738fbd97d9564c7711859fc509dae3d66a4e'
def load(p):return json.loads(p.read_text(encoding='utf-8'))
def save(p,x):p.write_text(json.dumps(x,indent=2)+'\n',encoding='utf-8',newline='\n')
def hash(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def evidence(paths):return {str(p.relative_to(R)).replace('\\','/'):hash(p) for p in paths}
recovery=load(D/'history-recovery.json')
contract=load(D/'OR-MIG-0173-contract.json');donor=R.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main'
refs=[]
db=sqlite3.connect('file:'+str(R.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').replace('\\','/')+'?mode=ro',uri=True)
for ref in contract['source_references']:
 if ref['kind']=='openrune':raw=(donor/ref['reference']).read_bytes();revision=ref['revision']
 else:
  revision,wikitext=db.execute('select revision_id,wikitext from pages where title=? COLLATE BINARY',(ref['reference'],)).fetchone();raw=wikitext.encode('utf-8')
 actual=hashlib.sha256(raw).hexdigest();assert actual==ref['sha256'],ref['reference'];refs.append({**ref,'actual_sha256':actual,'actual_revision':revision,'independently_read':True})
db.close()
save(D/'spade-reference-identities.json',{'implementation_sha':contract['implementation_commit_sha'],'status':'HASHES_VERIFIED','references':refs,'source_read_scope':'SpadeScript,Module,Hook read directly. Quest gamevals hash verified with digspade entry inspected separately. Wiki full wikitext loaded for hash; generic Dig/tool context distinct from exact tick/audio identifiers.','limitations':['Source configured241/upstream commit unavailable. Target239 IDs require actual cache checks.','Synchronous test hook callbacks do not establish real suspendable quest/boss consumers.','Native sound1470/animation/reset/chat and persistence acceptance remain open.']})
attempts=[]
for p in sorted(I.glob('integration-external-*/manifest.json')):
 m=load(p)
 if m.get('status')=='IN_PROGRESS':continue
 attempts.append({'path':p,'manifest':m})
search=next(a for a in attempts if a['manifest']['suite']=='empty-search-reach' and a['manifest']['status']=='PASS')
life=[a for a in attempts if a['manifest']['suite']=='immediate-life'];corrected=next(a for a in life if a['manifest']['test_identity']['status']=='PASS')
assert corrected['manifest']['status']=='FAIL'
initial=next(a for a in life if a['manifest']['test_identity']['status']=='NOT_PROVEN')
save(initial['path'].parent/'independent-diagnosis.json',{'status':'HARNESS_API_MISMATCH','scope':'No executable/test ran. Consolidated Recipe adds skill and output_count. Original Agent7 Crafting12/one-output semantics are made explicit in Agent5 fixture; no gameplay expectation changes.','evidence_sha256':evidence([initial['path'],initial['path'].parent/'run-1.log'])})
save(corrected['path'].parent/'independent-finding.json',{'finding_id':'PG-DEF-003','status':'CONFIRMED_CURRENT_MAIN','tested_sha':MAIN,'severity':'P2','implementation_owner':'FORGE','reproduction':'Fixture installs original immediate battlestaff/limestone recipes at Crafting99/rate500. Full bag retains duplicate tools and raw items. For each item direction set HP0, or queue an actual lethal hit, observe DeathRequested and restore visible HP without respawn. Submit exact selected raw slot17 via public prepare/finish held-use boundary.','expected':'No item transaction, XP award, RNG or later replay while dead or death-pending. Living selected-slot controls produce exactly one output.','actual':'All8 refusal scenarios fail conservation; selected raw changes into output. Limestone gives300 fineXP on success or75 on break. Living control test passes four recipe/direction combinations.','qualification':'World authority component, not native client reachability or durable persistence acceptance. New main confirmation of historical PG-DEF-003; no new global root cause or isolated-source fix claimed.','correction':'Apply authoritative alive/death-pending gates before immediate item/XP transaction, publish separately pinned correction and retain living/stale-selected regression controls. Ordinary finding: no implementation stop.','evidence_sha256':evidence([corrected['path'],corrected['path'].parent/'run-1.log',corrected['path'].parent/'harness/life.rs'])})
matrix=R/'OPENRUNE_VERIFICATION_MATRIX.csv'
with matrix.open(newline='',encoding='utf-8') as f:reader=csv.DictReader(f);fields=reader.fieldnames;rows=list(reader)
for row in rows:
 if row['implementation_id']=='OR-MIG-0161':row['current_main_acceptance']='PARTIAL: independent21-case World reach/cancel PASS; current native/persistence/reference acceptance not inherited'
 if row['implementation_id']=='OR-MIG-0065':row['current_main_acceptance']='FAIL_SCOPED: PG-DEF-003 eight life refusals fail; living controls pass'
 if row['implementation_id']=='OR-MIG-0173':
  row['historical_V0']='PARTIAL';row['historical_V1']='FAIL';row['historical_V2']='NOT_STARTED';row['historical_V3']='NOT_STARTED';row['historical_V4']='PARTIAL'
  row['current_main_acceptance']='FAIL_SCOPED: normal/cache/cancel suite PASS; unresolved-death Dig starts/completes after visibleHP restore'
  row['historical_evidence']+=';docs/openrune/agent5/LEVEL_RESULTS.jsonl'
  row['remaining']='Fix/retest authoritative unresolved-death gates; actual executable/native/audio/persistence/source phase equivalence remain open'
with matrix.open('w',newline='',encoding='utf-8') as f:writer=csv.DictWriter(f,fieldnames=fields);writer.writeheader();writer.writerows(rows)
history=str(OLD/'docs/openrune/verification').replace('\\','/')
(R/'OPENRUNE_VERIFICATION_DEFECTS.md').write_text(f'''# Independent verification findings

Current maintained-main target: `{MAIN}`. Original Agent7 findings remain in [the preserved correction backlog]({history}/CORRECTION_BACKLOG.md). No historical failure is deleted or silently relabelled fixed.

## PG-DEF-003 — confirmed on maintained main

Eight independent immediate battlestaff/limestone life-refusal scenarios still change inventory at zero HP or after an actual unresolved lethal hit. Applicable limestone XP is also awarded. The living control test passes all four recipe/direction combinations. Expected behavior is refusal before inventory/XP publication. See [exact reproduction and evidence]({corrected['path'].parent.relative_to(R).as_posix()}/independent-finding.json).

The first current-main fixture did not compile because Recipe gained `skill` and `output_count`. Its unchanged raw failure is preserved and classified as a harness API mismatch. The corrected fixture states Crafting12 and one output explicitly; refusal expectations are unchanged. No production implementation was edited.

## Historical findings requiring distinct current-main checks

PG-DEF-001: Close/chemistry policy epoch; independent chemistry remains user-deferred. PG-DEF-002/004/005: dead-actor assembly/pair/tanning paths. PG-DEF-006: worn hammer Swap operation routing. PG-DEF-007: variable rune-stock normalization. PG-DEF-008: native Toad Eat6 versus server5. Their original affected SHAs, reproductions and native qualifications remain in Agent7 evidence. None is claimed repaired by consolidation without a relevant current-main retest.

Existing harness findings include empty/stale test selection, mistaken synchronous submit refusal, native UI/X-input focus, startup contention, incomplete fixture population and timer overlay oracle errors. All4,729 indexed Agent7 evidence hashes matched during recovery. Native vendor population and Runecrafting boost dependencies remain separate from gameplay defects.
''',encoding='utf-8',newline='\n')
(R/'OPENRUNE_NATIVE_ACCEPTANCE.md').write_text(f'''# Native acceptance

Original contract stage definitions are retained: V2 is native acceptance and V4 is reference comparison for these historical contracts. The Agent5 handoff's summary numbering does not overwrite them.

The six historically accepted feature IDs are OR56,57,66,108,130,161. Their native/client/server identities and limitations remain in [Agent7's append-only results]({history}/LEVEL_RESULTS.jsonl). OR161 was accepted at `b06e3079d14f44f6f9ce25674f2e983d863af177`, including three native Search families, actual moving cancellation, acknowledged SQL save/process restart and quiet reconnect. That does not establish native acceptance against consolidated `{MAIN}`.

Agent5 has not yet run a maintained-current-client native campaign. Current local client HEAD was observed at `32d8dcbbf5e527c0e5254b96b88c061fe35e8c50`; its working tree has unrelated edits and is preserved. No current-main rendered/audio/SQL claim is inferred from the new headless checks. Existing maintained-main consolidation native reports are implementation/integration-owner evidence, not Agent5 independent acceptance.

Native work must use an isolated exact client/server source pair, one owned campaign at a time, ephemeral local game address and disposable account/DB, while respecting rendering/performance sessions. Never use43594. No desktop Computer Use, constructed negative wire actions, shared writable account or replacement actor population is used by this handoff.
''',encoding='utf-8',newline='\n')
body=f'''# Independent verification progress

Maintained-main baseline `{MAIN}`; FORGE observed `7421b89d5e458a7022ff7c5afc51ea79b02406fe`. Independent branch `agent5/openrune-verification` and isolated workspace preserve Agent7, ASCENT, Tower and modeling projects. Queue validator independently passed185 records/173 IDs. Historical452 level rows and4,729 indexed evidence hashes recovered without mismatches; six accepted feature IDs retained at their historical pins.

Consolidation changes shared simulation/runtime/storage, so direct old adapter identity alone cannot confer current-main acceptance. [History and exact changed paths](docs/openrune/agent5/history-recovery.json) and [per-contract matrix](OPENRUNE_VERIFICATION_MATRIX.csv) keep identities separate. Every contract's original V0–V4 definitions remain authoritative.

Current-main independent Search reach checks PASS: seven immediate messages, seven genuinely unreachable targets and seven moving approaches cancelled by ordinary Walk; no RNG/item/XP/animation/sound/repeat. This is scoped regression, not renewed full native/persistence acceptance.

Current-main PG-DEF-003 persists: eight authoritative life-refusal cases FAIL; four living controls PASS. Initial recipe API compilation failure retained separately; no implementation patch or expected-result weakening.

OR173 exact `cec5b1e125021b606371fc70488a67635b7213c9`: independent normal/cache/cancellation suite PASS at original pin and maintained main:112 positives,24 initial refusals,3 cancels,2 first-claim/veto fixtures and6 cache contradictions. New life tail FAIL at both pins: visibleHP restoration after actual DeathRequested permits Dig before admission and completion; both zeroHP controls PASS. V1 remains FAIL. Native callback consumers, asynchronous phase equivalence, audio and durable lifecycle remain separate obligations. Source-script/module/hook and quest-gamevals plus Wiki identities matched.

Chemistry/potions/Herblore remain deferred. Historical Agent7 local checkpoint `0936f81` was not pushed because automatic approval review blocked that301-file egress; its evidence is preserved locally and was read for recovery, not republished as part of this handoff.

Next: finish exact173 tests, run relevant current-main regression, continue unfinished eligible contracts and shared engine checks. No background continuation is promised.
'''
for a in attempts:
 m=a['manifest'];body+='\n- '+m['suite']+' at `'+m['tested_integration_sha']+'`: '+m['status']+'; test identity '+m['test_identity']['status']+'; [manifest]('+a['path'].relative_to(R).as_posix()+').\n'
(R/'OPENRUNE_VERIFICATION_PROGRESS.md').write_text(body,encoding='utf-8',newline='\n')
tails=[a for a in attempts if a['manifest']['suite']=='spade-life-tail']
if len(tails)==2:
 assert all(a['manifest']['status']=='FAIL' and a['manifest']['test_identity']['status']=='PASS' for a in tails)
 paths=[a['path'] for a in tails]+[a['path'].parent/'run-1.log' for a in tails]
 finding={'finding_id':'PG-DEF-003-SPADE','parent_finding':'PG-DEF-003','severity':'P2','status':'CONFIRMED_PINNED_AND_MAIN','tested_shas':[a['manifest']['tested_integration_sha'] for a in tails],'reproduction':'Admit a real player with spade952 slot7/HP99. Queue a150-damage typeless World hit and observe DeathRequested with HP0. Restore only visible HP99, without respawn. Submit Dig before admission, or restore HP during the already admitted two-tick Dig.','expected':'No Dig animation/sound/admission after unresolved death; no completed generic message/hook during that death state.','actual':'Both independently executed pins emit one generic message in each restoredHP case; the pre-admission case also emits a new Dig animation. Both HP0 controls suppress completion. No item/XP reward is invented by generic Dig.','correction':'Use authoritative life/death-pending state before spade admission and completion/hook dispatch. Retain original failures; publish and retest separately pinned correction.','scope':'World authority, not native gameplay reachability. Synchronous independent callbacks are fixture-only; no actual quest/boss consumer accepted. Ordinary backlog finding; no FORGE stop.','evidence_sha256':evidence(paths)}
 save(D/'spade-life-finding.json',finding)
 with (R/'OPENRUNE_VERIFICATION_DEFECTS.md').open('a',encoding='utf-8',newline='\n') as f:f.write('\n## PG-DEF-003 extension — spade unresolved death\n\nIndependent four-case lifecycle tails at both exact173 and maintained main have2PASS/2FAIL. Visible HP restoration after actual DeathRequested allows new Dig or delayed completion. Generic Dig gives no item/XP, but the authoritative lifecycle refusal is absent. [Exact finding and correction](docs/openrune/agent5/spade-life-finding.json). Normal positive/cache/cancellation suites remain valid; complete V1 acceptance is withheld.\n')
 levels=[]
 for level,status,reason in [('V0','PARTIAL','Five independently hashed source/Wiki identities plus actual native952/830/1470/default binding and six factory contradictions pass; actual executable/runtime/native dispatch remains open.'),('V1','FAIL','112normal/24initial/3cancel/2hook/6cache cases PASS. Two actual unresolved-death visibleHP-restored cases FAIL; two zeroHP controls PASS at exact contract SHA.'),('V2','NOT_STARTED','No Agent5 pinned native gameplay acceptance yet.'),('V3','NOT_STARTED','No Agent5 actual spade SQL/process restart/native reconnect campaign yet.'),('V4','PARTIAL','Source ordinary Dig/two ticks/first claim/veto/default compared; configured241 async/phase/native parity remains qualified and open.')]:
  levels.append({'implementation_id':'OR-MIG-0173','tested_sha':contract['implementation_commit_sha'],'level':level,'status':status,'independent_verification':True,'stage_definitions':'Original OR173 contract definitions retained, not renumbered','reason':reason,'evidence':['docs/openrune/agent5/spade-life-finding.json']+[a['path'].relative_to(R).as_posix() for a in attempts if a['manifest']['suite'].startswith('spade')]})
 (D/'LEVEL_RESULTS.jsonl').write_text(''.join(json.dumps(x)+'\n' for x in levels),encoding='utf-8',newline='\n')
print('Recovery, matrix, findings, native boundaries and running progress reconciled.')
