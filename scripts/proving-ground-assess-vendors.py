"""Aggregate independent131 progress and preserve its QA compile failure."""
import hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent';SHA='35a5f39b72cab3a2655f32ae29bdde54284c9a4b'
selected=[];failed=[]
for suite in ['skillcape-vendor-matrix','skillcape-vendor-world','skillcape-vendor-hood']:
 paths=list(OUT.glob('integration-external-'+suite+'-'+SHA+'*/manifest.json'))
 good=[]
 for p in paths:
  r=json.loads(p.read_text(encoding='utf-8'));assert r['tested_integration_sha']==SHA
  (good if r['status']=='PASS' else failed).append(p)
 assert len(good)==1;selected+=good
assert len(failed)==1
failure=(failed[0].parent/'run-1.log').read_text(encoding='utf-8');assert 'unclosed delimiter'in failure and 'Running tests'not in failure
names=[(p.parent/n).relative_to(OUT).as_posix()for p in selected+failed for n in ['manifest.json','run-1.log']]
report={'implementation_id':'OR-MIG-0131','tested_sha':SHA,'status':'PASS_SCOPED','level':'V1','case_groups':{'purchase_transaction_matrix':3360,'predebit_space_money_refusals':336,'publicWorld_purchase_dialogues':2,'publicWorld_late_payment_guards':20,'publicWorld_hood_dialogue_boundaries':40},
 'scope':'3758independentcases at the original131SHA. Allcoin-slot/payment/member/base/releasedstat patterns, exact99000debit+cape+hood, firsttwofreepredebit slots and unrelatedvars conservation. Actual publicNPC dialogue/quote/payment ordering:Craftingpaysonaccept,CookingonlyafterplayerContinue;stalechoicecannotchargeagain.20latepaymentguards.40two-ackhoodcases bothNPCs/bothcapes/held-worn/currentcape-member-Close-space guard withfreehood/noXPgear/noReplay.',
 'harness_history':'Original matrix failed to compile because ownedtesthadunclosedbrace/invalidconditional syntax;zero scenarios ran. Corrected ownedtest reran against identical35a5f39SHA and passed3696cases. Originalfailure retained;not animplementationdefect.',
 'remaining':['V1 remainsIN_PROGRESS: already-owned Crafting refusal versus Cooking paidspare/freeadditionalhood, allmalformedcatalog/config/count/vars/duplicatecoin stocks, actualqueueddeath/movement/NPCgeneration/interruption contexts and broader actioncases remain.','No131actualcache/serverstartup/nativeV0/V2or durableV3acceptance;referencefacts are separate.','These are normal dependency/publicWorld tests;noFORGEdeveloper test body counted andno networknegative payloads.'],'evidence_sha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest()for n in names}}
p=OUT/'skillcape-vendor-world-partial.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS_SCOPED131 3758cases; fullV1/native/durable obligations open')
