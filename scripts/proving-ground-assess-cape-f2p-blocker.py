"""Preserve the failed expectation and describe the actual native route blocker."""
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-crafting-cape-f2p/native-001'
load=lambda n:json.loads((OUT/n).read_text(encoding='utf-8'))
m=load('produce/manifest.json');initial=load('initial-state.json');after=load('produce/after.json');log=(OUT/'produce/client.log').read_text(encoding='utf-8')
assert m['status']=='FAIL' and m['client_exit']==1 and m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged']
assert m['tested_sha']=='7d2133f23e00ad852df8726ead67493e83c901b4'
assert 'PG_CAPE_ACCOUNT kind=crafting-cape-f2p actual_member=false cape=9780' in log
assert 'actual native cape Boost menu absent' in log
options=re.findall(r'PG_CAPE_MENU press=0 option="(.*?)"',log);assert options==['Examine','Remove'],options
assert after['inventory']==initial['inventory'] and after['stats']==initial['stats'] and after['character'][:4]==initial['character'][:4]
for h in re.findall(r'PG_WIRE_TX tick=\d+ opcode=47 payload=\[(.*?)\]',log):
 b=bytes.fromhex(h.replace(',',''));assert int.from_bytes(b[:4],'big')>>16==548
names=['initial-state.json','frozen-observer/pg_observer.rs','produce/manifest.json','produce/before.json','produce/after.json','produce/client.log','produce/server.log']
report={'implementation_id':'OR-MIG-0130','tested_sha':m['tested_sha'],'status':'BLOCKED','raw_observer_status':'FAIL','classification':'Harness expected a native menu that this actual F2P client does not offer; not an implementation defect.','scope':'Actual F2P login and worn9780 observed; exact native menu has Examine and Remove only, no Boost/Teleport. No Boost operation sent; whole bag/gear/allStats/fineXP unchanged and acknowledged save. Native backend-refusal-message scenario cannot execute through this menu at this pin. World F2P refusal and typed button adapter checks are separate evidence.','next_action':'Keep exact blocked identity; continue eligible member native/other contracts. Do not construct network payloads to force the absent menu.','evidence_sha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest()for n in names}}
p=OUT/'independent-assessment.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('BLOCKED exact F2P native menu lacks Boost; no mutation or forced packet')
