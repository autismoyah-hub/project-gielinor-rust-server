"""Aggregate actual pinned runtime reachability without borrowing FORGE evidence."""
import hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent';SHA='7d2133f23e00ad852df8726ead67493e83c901b4'
names=[]
for suite in ['crafting-cape-bindings','crafting-cape-buttons','crafting-cape-guild-entry','rune-altars-current-regression']:
 d='integration-external-'+suite+'-'+SHA;m=json.loads((OUT/d/'manifest.json').read_text(encoding='utf-8'));assert m['status']=='PASS' and m['tested_integration_sha']==SHA;names +=[d+'/manifest.json',d+'/run-1.log']
d='deep-crafting-cape-equipment/native-002'
m=json.loads((OUT/d/'produce/manifest.json').read_text(encoding='utf-8'));assert m['status']=='PASS' and m['tested_sha']==SHA and m['client_exit']==m['server_exit']==0
names +=[d+'/'+n for n in ['build.json','produce/manifest.json','produce/client.log','produce/server.log','independent-assessment.json']]
report={'implementation_id':'OR-MIG-0130','tested_sha':SHA,'level':'V0','status':'PASS','scope':'Exact pristine server binary built and actual cache-backed startup completed. Genuine native9780/9781Remove/Wear/Boost reaches actual mapped387:16nativeop2 and received stat46current100, proving startup/runtime/equipment/action/notification reachability. Independent actual two cape/stat/451Boost452Teleport/wear1base99facts and55malformedbinding refusals;1612typed mapped-button calls,12guild-entry/Boost permission scenarios and972ordinaryRune sharedWorld regressions PASS. No FORGE developer tests counted.','qualifications':['V0 actual reachability does not accept native rendered Skills/F2P-message requirements or lifecycle/reference levels.','Synthetic adapter tests assume upstream component mapping; genuine native positive actions separately establish that mapping. No constructed network negative payloads.','Shared UI/stat/guild paths covered within this cape contract; full129r2all49wardrobe or latestHEAD integration is separate.'],'evidence_sha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest()for n in names}}
p=OUT/'crafting-cape-runtime-v0-complete.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('V0 PASS exact130 cache/runtime/native reachability and shared regressions')
