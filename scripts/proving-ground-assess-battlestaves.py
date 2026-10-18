"""Independent durable inventory/Stats and decoded native effect oracles for OR67."""
import argparse,ast,hashlib,json,re
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--crash',action='store_true');a=p.parse_args()
ROOT=Path(__file__).resolve().parents[1];SHA='e3ced82211ee438fd6a74a110d435e6746295a16'
OUT=ROOT/'docs/openrune/verification/independent'/('deep-battlestaff-crash/crash-001' if a.crash else 'deep-battlestaves/native-001')
load=lambda name:json.loads((OUT/name).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json')
profiles=[(571,1395,5000,1370),(575,1399,5625,1371),(569,1393,6250,1372),(573,1397,6875,306)]
actions=[];expected=[r.copy() for r in initial['inventory']];award=0
for i,(orb,out,xp,spot) in enumerate(profiles):
 for direction in range(2):
  if a.crash and actions:continue
  staff_slot=4+i*4+direction;orb_slot=6+i*4+direction
  assert next(r for r in expected if r[:2]==[93,staff_slot])[2:]==[1391,1,0]
  assert next(r for r in expected if r[:2]==[93,orb_slot])[2:]==[orb,1,0]
  expected=[r for r in expected if r[:2] not in [[93,staff_slot],[93,orb_slot]]]
  free=next(s for s in range(28) if not any(r[:2]==[93,s] for r in expected))
  expected.append([93,free,out,1,0]);expected.sort();award+=xp
  actions.append({'staff_slot':staff_slot,'orb_slot':orb_slot,'orb':orb,'output':out,'output_slot':free,'fine_award':xp,'spot':spot,'direction':direction,'fine_total':130344311+award})
assert award==(5000 if a.crash else 47500)
durable=initial['inventory'] if a.crash else expected
assert after['inventory']==post['inventory']==durable and load('relogin/before.json')==after
for x,y,z in zip(initial['stats'],after['stats'],post['stats']):
 assert x[0]==y[0]==z[0] and y==z
 if a.crash or x[0]!=12:assert x==y
 else:assert y==[12,99,99,x[3]+award]
for phase in ['produce','relogin']:
 m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS' and m['resource_hashes_unchanged'] and m['client_exit']==0
 assert load(phase+'/after.json')['integrity_check']=='ok'
 if a.crash and phase=='produce':assert m['server_exit']!=0 and m['intentional_crash_injected'] and 'battlestaff' in m['fault_scope'] and not m['save_acknowledgement'] and not m['graceful_shutdown']
 else:assert m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and not m['forced_shutdown']
log=(OUT/'produce/client.log').read_text(encoding='utf-8')
if a.crash:assert 'PG_CRASH_READY native battlestaff' in log and 'PG_EXPECTED_CRASH_DISCONNECT' in log
else:assert 'PG_CASE_PASS native battlestaff pending Walk cancellation' in log and 'PG_CASE_PASS native battlestaff eight manual actions' in log
uses=re.findall(r'PG_WIRE_TX tick=(\d+) opcode=27 payload=\[(.*?)\]',log);assert len(uses)==len(actions)+(0 if a.crash else 1)
for i,(_,hexes) in enumerate(uses):
 b=bytes.fromhex(hexes.replace(',',''));assert len(b)==16
 beadd=lambda v:(v[0]<<8)|((v[1]-128)&255);leadd=lambda v:((v[0]-128)&255)|(v[1]<<8)
 pair=(beadd(b[4:6]),beadd(b[10:12]),int.from_bytes(b[8:10],'little'),leadd(b[6:8]))
 j=i if a.crash else i-1
 if j<0:assert pair==(0,1391,22,571)
 else:
  r=actions[j];assert pair==((r['staff_slot'],1391,r['orb_slot'],r['orb']) if r['direction']==0 else (r['orb_slot'],r['orb'],r['staff_slot'],1391))
assert len(re.findall('PG_REPLAY label="native stale selected battlestaff pair',log))==len(actions)
assert not re.search(r'PG_WIRE_TX tick=\d+ opcode=115 ',log)
effects=[];seen=set();sequence=None;spots=None;xp_seen=set()
for line in log.splitlines():
 s=re.fullmatch(r'PG_SEQUENCE tick=(\d+) sequence=7531 delay=(\d+)',line)
 if s:sequence=(int(s[1]),int(s[2]))
 s=re.fullmatch(r'PG_SPOTS tick=(\d+) decoded_slot_id_height_delay=(\[.*\])',line)
 if s:spots=(int(s[1]),ast.literal_eval(s[2]))
 rx=re.fullmatch(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*)\]',line)
 if not rx:continue
 tick,op=int(rx[1]),int(rx[2]);b=bytes.fromhex(rx[3].replace(',',''))
 if op==46 and (b[2]-128)&255==12:xp_seen.add(int.from_bytes(b[5:7]+b[3:5],'big'))
 if op!=18 or int.from_bytes(b[4:6],'big')!=93:continue
 pos=6
 while pos<len(b):
  slot=b[pos];pos+=1
  if slot>=128:slot=((slot<<8)|b[pos])-32768;pos+=1
  item=int.from_bytes(b[pos:pos+2],'big')-1;pos+=2;count=0
  if item>=0:
   count=b[pos];pos+=1
   if count==255:count=int.from_bytes(b[pos:pos+4],'big');pos+=4
  if item in [r[1] for r in profiles] and slot not in seen:
   r=actions[len(effects)];assert (slot,item,count)==(r['output_slot'],r['output'],1)
   assert sequence is not None and sequence[1]==0 and tick-sequence[0]==2
   assert spots is not None and spots[0]==sequence[0] and (0,r['spot'],0,0) in spots[1]
   effects.append({'output':item,'slot':slot,'sequence_tick':sequence[0],'credit_tick':tick,'sequence':7531,'spot':r['spot']});seen.add(slot)
 assert pos==len(b)
assert len(effects)==len(actions)
for r in actions:assert r['fine_total']//10 in xp_seen
invs={};stats={}
for op,hexes in re.findall(r'PG_RX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',(OUT/'relogin/client.log').read_text(encoding='utf-8')):
 op=int(op);b=bytes.fromhex(hexes.replace(',',''))
 if op==46:stats[(b[2]-128)&255]=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'))
 if op!=22:continue
 inv=int.from_bytes(b[4:6],'big');size=int.from_bytes(b[6:8],'big');pos=8;items=[]
 for slot in range(size):
  count=(128-b[pos])&255;pos+=1
  if count==255:c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
  item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2
  if item>=0:items.append([inv,slot,item,count,0])
 assert pos==len(b);invs[inv]=items
assert invs[93]==durable
for skill,current,base,fine in post['stats']:assert stats[skill]==(current,fine//10)
report={'implementation_id':'OR-MIG-0067','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','crash':a.crash,'actions':actions,'durable_outputs':0 if a.crash else 8,'observed_fine_award':award,'effects':effects,'scope':'Independent selected two-input transaction oracle, raw held-Use/inventory/Stats and decoded sequence/spot events. Stale selected requests cannot substitute retained duplicates; no Make-X or automatic repeat. Complete durable inventory/Stats verified by SQL and independently decoded actual server restart/native reconnect. '+('One unsaved observed product restores prior state after termination of only the owned disposable server, with no save acknowledgement.' if a.crash else 'Eight manual actions, both directions for all four profiles, exact47500fineXP, retained orb/staff duplicates and coins, plus actual pending Walk cancellation.'),'limitations':['Native rate500; modifier/current-level/boost/refusal and legacy API cases are separate public-API campaigns.','Decoded effects/two receive clocks do not establish rendered frames or audio.','Recovery fault is unacknowledged state rollback, not mid-SQL commit.'],'evidence_sha256':{p:hashlib.sha256((OUT/p).read_bytes()).hexdigest() for p in ['produce/client.log','produce/manifest.json','relogin/client.log','relogin/manifest.json']}}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
