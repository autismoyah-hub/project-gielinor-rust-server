"""Retain positive controls, QA failures and exact132 life-boundary finding."""
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent';SHA='a2c5ceaae693a73c64afa8b4558e05b12bb6e100'
world=list(OUT.glob('integration-external-cooking-cape-world-'+SHA+'*/manifest.json'));assert len(world)==2
compiled=next(p for p in world if json.loads(p.read_text(encoding='utf-8'))['test_binary_sha256']);empty=next(p for p in world if p!=compiled)
assert json.loads(empty.read_text(encoding='utf-8'))['status']=='FAIL'and 'could not compile'in(empty.parent/'run-1.log').read_text(encoding='utf-8')
assert 'PG_COOK_CAPE108'in(compiled.parent/'run-1.log').read_text(encoding='utf-8')
tail=OUT/('integration-external-cooking-cape-guards-tail-'+SHA);life=OUT/('integration-external-cooking-cape-life-'+SHA)
for d in [tail,life]:assert json.loads((d/'manifest.json').read_text(encoding='utf-8'))['status']=='PASS'
log=(life/'run-1.log').read_text(encoding='utf-8')
rows=[dict(zip(['cape','surface','slot','phase','hp','fine_delta','cooked','calls','violation'],v))for v in re.findall(r'PG_COOK_CAPE_LIFE cape=(\d+) surface=(\d+) slot=(\d+) phase=(\d+) hp=(\d+) fineXPdelta=(\d+) cooked=(true|false) calls=(\d+) violation=(true|false)',log)]
assert len(rows)==72 and all(r['fine_delta']=='375'and r['cooked']=='true'and r['calls']=='0'and r['violation']=='true'for r in rows)
names=[(p.parent/n).relative_to(OUT).as_posix()for p in world for n in ['manifest.json','run-1.log']]+[(d/n).relative_to(OUT).as_posix()for d in [tail,life]for n in ['manifest.json','run-1.log']]
r={'implementation_id':'OR-MIG-0132','tested_sha':SHA,'level':'V1','status':'FAIL','finding':'PG-DEF-003 extension: Cooking-cape pending action life guards', 'priority':'P2 ordinary correction backlog; no FORGE stop/external issue',
 'scope':'72independent publicWorld scenarios at exact132confirm cape cooking still publishes selected2132->2142and+375fineCookingXP at125rate whileHP0oractuallethalDeathRequested remainsunresolved,includingvisibleHP10restorationwithoutrespawn. Both9801/9802,fire26185/range26181,selected0/13/27, sixbefore/duringlifeboundaries;four-tickqueuebarrier independentlyassertsnoawardbeforelatelethalhit. ValidcapeavoidsRNGbutdoesnotblockdeadcompletion. DiagnosticRustPASSmeansobservationscompleted,not acceptancePASS.',
 'passing_scope':'108publicWorldpositivecases:bothcapes/fire-range/selected0-13-27/rates100-125-500/current1-50-99/fullbag,strictselected2tickoutput/exactXP/wholegear/zeroRNG.120latecape/equip/remove/member/wrongslot/count/vars/catalog/lowlevel/staleslot/raw/bothoutputpolicy/rate/modal/location/ClosecontrolsPASS. No positivecampaignreplay.',
 'harness_history':['Firstattemptfailedcompilation onownedset_modal/remove names;zero scenariosexecuted. Correctedset_modal_open/locations.delete againstsameSHA.','CorrectedWorldattemptpassed108positives thenfirstguardfailed:ownedexpectationassumed0/0chancewithdraw0wouldburn. Existingformulaactuallyhas1/256minimum,so ordinarydraw0successwasvalid. Tailchangedonlydrawto255andranfailed+119unexecutedguards;all120PASS. Originalfailuresretained.'],
 'actionable_correction':'Apply existing common alive/death-pending checks at cooking admission and actual completion, before cape/chance/transaction/XP publication. A visible HP restore alone must not bypass queued death. Publish separately pinned correction; retain historical132FAIL and rerun unchanged72life scenarios plus228controls at the newSHA.',
 'qualifications':['Controlledrecipefixtureuseslow0/high0,min1/256chance,30XPandtwo-tickpositive/four-ticklifebarrier. It isolates authoritativeperkeffects;not anattestationofactualregisteredfoodcurve/recipebindings. Rawoomlie2337recipe remainsunported andwasnotexecuted.','Lateequipment/member/catalog mutations are explicitpublicWorldfixturefaultinjection;no nativeUIoractualstorageclaim.','V0runtime/cachebinding, V2native, V3durability andfullV4 remainseparate. No chemistry/potion scenario executed.'],
 'observations':rows,'evidence_sha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest()for n in names}}
p=OUT/'cooking-cape-life-finding.json';assert not p.exists();p.write_text(json.dumps(r,indent=2)+'\n',encoding='utf-8',newline='\n');print('132 V1FAIL72lifeviolations;228positive/guardcontrols preserved')
