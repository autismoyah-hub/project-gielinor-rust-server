"""Assess exact134 diagnostic output, preserving the original failed run."""
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent';SHA='a1658fd5c4d4a6869063a75859899d381742d271'
dirs={s:OUT/('integration-external-'+s+'-'+SHA)for s in ['zealot-selected-matrix','zealot-selected-world','zealot-selected-world-tail','zealot-selected-life']}
for s,p in dirs.items():
 m=json.loads((p/'manifest.json').read_text(encoding='utf-8'));assert m['tested_integration_sha']==SHA and m['status']==('FAIL' if s=='zealot-selected-world' else 'PASS')
log=(dirs['zealot-selected-life']/'run-1.log').read_text(encoding='utf-8')
rows=[dict(zip(['slot','phase','draw','hp','fine_delta','retained','calls','violation'],v))for v in re.findall(r'PG_ZEALOT_LIFE slot=(\d+) phase=(\d+) draw=(\d+) hp=(\d+) fineXPdelta=(\d+) selectedRetained=(true|false) calls=(\d+) violation=(true|false)',log)]
assert len(rows)==36 and all(r['fine_delta']=='56' and r['calls']=='1' and r['violation']=='true' for r in rows)
assert all(r['retained']==('true' if r['draw']=='79' else 'false')for r in rows)
names=[p.relative_to(OUT).as_posix()+'/'+n for p in dirs.values()for n in ['manifest.json','run-1.log']]+['reference-zealot-burial/comparison.json']
result={'implementation_id':'OR-MIG-0134','tested_sha':SHA,'status':'FAIL','finding':'PG-DEF-003 extension: selected burial life guards','priority':'P2 ordinary correction backlog; no implementation stop or external issue',
 'scope':'36 independent publicWorld diagnostics confirm ordinary selected burial awards56fineXP at125rate and draws robe RNG while zeroHP or authoritativeDeathRequested remains unresolved. Draw75 consumes selected bone,draw79 preserves it; both award normalXP. Three selectedslots0/7/27 and six boundaries: initialzeroHP,latezeroHP,actualdeathbeforeadmission,actualdeathbeforeadmissionthenvisibleHPrestore,actualdeathafteradmission,actualdeathafteradmissionthenvisibleHPrestore. Actualdeath variants assert WorldEvent::DeathRequested from a queued lethalHit, with no respawn. Diagnostic Rust test PASS means all observations completed; every scenario is acceptanceFAIL.',
 'reproduction':['Create member actor HP10 with ordinary526 in all28slots and ordinary25438 worn0;rate125.','Queue authoritative lethalHit20 and tick;assertDeathRequested. OptionallyrestorevisibleHP10withoutrespawn.','Submit BuryItemSlot with exactselectedbone526, tick through admission/completion using existing World RNG returning75or79.','Observe finePrayerXP+56 and onebelow80draw. At75selectedboneconsumed;79retained. Expectedlife guard refuses beforeXP/item/RNG while dead/deathpending.'],
 'actionable_correction':'At the original target or a separately contracted correction, apply existing common alive/death-pending checks consistently at selected burial admission, confirmation resume/animation and completion before RNG/transaction publication. Do not change legacy reference/altar behavior implicitly. Re-run all36historical scenarios against the new exact correction SHA plus live controls; retain originalFAIL.',
 'passing_scope':{'transaction_cases':16602,'world_normal_cases':21,'world_confirmation_cases':4,'notes':'Original mixedWorld run completed first7normalcases before the slot0lateHPfailure. Tail executes only14remainingnormalcases at slots7/27. Fourconfirmation cases passed in the original separate Rust test. No completed immutable gameplay replay.'},
 'remaining':'V0runtime/cache/native and V2/V3durability remain open; positive subchecks do not acceptV1. OriginalrawFAIL and exactpristinesource retained.',
 'observations':rows,'evidence_sha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest()for n in names}}
p=OUT/'zealot-selected-life-finding.json';assert not p.exists();p.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8',newline='\n');print('V1 FAIL PG-DEF-003 extension:36of36 verified life violations;16602transaction/25World controls retained')
