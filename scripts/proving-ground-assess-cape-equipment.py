"""Read-only independent assessment of the immutable native equipment campaign."""
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-crafting-cape-equipment/native-002'
load=lambda n:json.loads((OUT/n).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json')
assert load('produce/before.json')==initial and load('relogin/before.json')==after
assert initial['stats'][12]==[12,99,99,130344311] and after['stats'][12]==[12,100,99,130344311] and post['stats'][12]==[12,99,99,130344311]
for image in [initial,after,post]:
 assert image['inventory']==initial['inventory'] and image['character'][:4]==initial['character'][:4] and image['integrity_check']=='ok'
 assert image['xp_rate_hundreds']==500
 for a,b in zip(initial['stats'],image['stats']):assert b==a if a[0]!=12 else b[:1]+b[2:]==a[:1]+a[2:]
for phase in ['produce','relogin']:
 m=load(phase+'/manifest.json');assert m['status']=='PASS' and m['tested_sha']==m['implementation_sha']=='7d2133f23e00ad852df8726ead67493e83c901b4'
 assert m['client_exit']==m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown']
 assert m['server_sha256']==load('build.json')['binary_sha256'] and m['client_sha256']==load('frozen-observer/build.json')['binaries']['pg_observer.exe']
text=(OUT/'produce/client.log').read_text(encoding='utf-8');relog=(OUT/'relogin/client.log').read_text(encoding='utf-8')
assert 'PG_CASE_PASS native cape actual Remove/Wear' in text
actions=[]
for t,h in re.findall(r'PG_WIRE_TX tick=(\d+) opcode=47 payload=\[(.*?)\]',text):
 b=bytes.fromhex(h.replace(',',''));assert len(b)==9
 component=int.from_bytes(b[:4],'big');sub=int.from_bytes(b[4:6],'big');obj=int.from_bytes(b[6:8],'big');op=b[8]
 if component>>16!=548:actions.append([int(t),component,sub,obj,op])
assert [(c,s,o,p)for _,c,s,o,p in actions]==[(25362448,65535,65535,1),(9764864,0,9781,3),(25362448,65535,65535,2),(25362448,65535,65535,1),(9764864,1,9780,3),(25362448,65535,65535,2)],actions
samples=[]
for t,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',text):
 b=bytes.fromhex(h.replace(',',''))
 if int(op)==46 and ((b[2]-128)&255)==12:
  v=[int(t),b[1],int.from_bytes(b[5:7]+b[3:5],'big')]
  assert v[2]==13034431
  if not samples or samples[-1][1:]!=v[1:]:samples.append(v)
assert [v[1]for v in samples]==[99,100,99,100],samples
assert 'PG_CAPE_EQUIPMENT trimmedWear actual no passive current boost' in text and 'PG_CAPE_EQUIPMENT_EXPIRED actual99' in text
names=['initial-state.json','build.json','frozen-observer/build.json','frozen-observer/pg_observer.rs']+[p+'/'+n for p in ['produce','relogin']for n in ['manifest.json','before.json','after.json','client.log','server.log']]
report={'implementation_id':'OR-MIG-0130','tested_sha':'7d2133f23e00ad852df8726ead67493e83c901b4','status':'PASS_SCOPED','native_actions':actions,'crafting_stat_samples':samples,
 'scope':'Six genuine native inventory/equipment actions: Remove normal, Wear trimmed, Boost trimmed, Remove trimmed, Wear normal, Boost normal. Received Crafting99/100/99/100 with constantXP plus whole SQL bag/gear/base/fineXP conservation across acknowledged save and real server restart. Frozen native observer also checks received intermediate inventory/gear changes and no passive boost on Wear. No successful action replay.',
 'limitations':['Three private BMPs are identical scene-only c129cb92a4c7cababfb78189c0597b7c97d0973417271fa5181b6a7f40bc9b0e. Inspected pixels do not display Skills/equipment overlay; rendered Skills100 obligation remains open for a same-account observation continuation.','Normal expiry follows existing shared regeneration phase rather than a new activation-relative timer. No unsaved crash or donor executable parity claim.'],
 'evidence_sha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest()for n in names}}
path=OUT/'independent-assessment.json';assert not path.exists();path.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS_SCOPED actual equipment sequence; rendered Skills observation remains OPEN')
