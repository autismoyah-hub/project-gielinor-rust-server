"""Verify owned abrupt termination restores prior whole immediate-crafting state."""
import json,re,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='ad739676dd01f5be809cdfe5ecf4f1e6ea77238a';OUT=ROOT/'docs/openrune/verification/independent/deep-light-crash/native-001';load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');m=load('produce/manifest.json');restart=load('relogin/manifest.json')
assert m['tested_sha']==m['implementation_sha']==restart['tested_sha']==restart['implementation_sha']==SHA
assert m['status']=='PASS' and m['client_exit']==0 and m['server_exit']!=0 and m['intentional_crash_injected'] and 'unsaved native candle lighting' in m['fault_scope']
assert not m['save_acknowledgement'] and not m['graceful_shutdown'] and m['resource_hashes_unchanged']
assert restart['status']=='PASS' and restart['client_exit']==restart['server_exit']==0 and restart['save_acknowledgement'] and restart['graceful_shutdown'] and restart['resource_hashes_unchanged'] and not restart['forced_shutdown']
for state in [after,post]:assert state['integrity_check']=='ok' and state['inventory']==initial['inventory'] and state['stats']==initial['stats'] and state['character'][3]==initial['character'][3]
assert load('relogin/before.json')==after
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'PG_CRASH_READY native unsaved candle lit only selected slot4' in log and 'PG_EXPECTED_CRASH_DISCONNECT' in log
output=33
assert 'PG_LIGHT_CANCELLED real native selected Use/Cancel zero target packet zero mutation' in log
prefix=log.split('PG_LIGHT_CANCELLED',1)[0];assert not re.search(r'PG_WIRE_TX tick=\d+ opcode=27 ',prefix)
wire=re.findall(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log)
assert sum(int(op)==27 for op,h in wire)==1 and not any(int(op)==115 for op,h in wire)
observed_output=False
for op,hexes in re.findall(r'PG_RX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log):
 b=bytes.fromhex(hexes.replace(',',''));op=int(op)
 if op!=18 or int.from_bytes(b[4:6],'big')!=93:continue
 pos=6
 while pos<len(b):
  slot=b[pos];pos+=1
  if slot>=128:slot=((slot<<8)|b[pos])-32768;pos+=1
  item=int.from_bytes(b[pos:pos+2],'big')-1;pos+=2;count=0
  if item>=0:
   count=b[pos];pos+=1
   if count==255:count=int.from_bytes(b[pos:pos+4],'big');pos+=4
  if (slot,item,count)==(4,output,1):observed_output=True
 assert pos==len(b)
assert observed_output
invs={};stats={}
for _,op,hexes in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',(OUT/'relogin/client.log').read_text(encoding='utf-8')):
 b=bytes.fromhex(hexes.replace(',',''));op=int(op)
 if op==46:stats[(b[2]-128)&255]=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'))
 if op!=22:continue
 inv=int.from_bytes(b[4:6],'big');size=int.from_bytes(b[6:8],'big');pos=8;items=[]
 for slot in range(size):
  count=(128-b[pos])&255;pos+=1
  if count==255:c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
  item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2
  if item>=0:items.append([inv,slot,item,count,0])
 assert pos==len(b);invs[inv]=items
assert invs[93]==initial['inventory']
for skill,current,base,fine in initial['stats']:assert stats[skill]==(current,fine//10)
report={'implementation_id':'OR-MIG-0084','tested_sha':SHA,'status':'PASS','observed_unsaved_output':33,'fine_xp_award':0,'durable_outputs':0,'scope':'Genuine native Tinderbox Use selection/Cancel before target sent no held27frame and preserved whole inventory/Stats. Then one actual selected candle36to33 raw18credit was received; only the freshly owned disposable server terminated with its socket open, no logout/save acknowledgement. SQL restored prior full28raw inventory/Stats/tools/coins/energy; actual restart/native raw22/46 matched all items/Stats, no pending/replay/automatic toggle.','limitations':['Unacknowledged rollback does not inject during a SQL commit.','Separate completed normal run preserves nine lit/nine unlit states; no immutable successful account reexecuted.'],'evidence_sha256':{p:hashlib.sha256((OUT/p).read_bytes()).hexdigest()for p in ['initial-state.json','build.json','produce/client.log','produce/manifest.json','produce/after.json','relogin/client.log','relogin/manifest.json','relogin/after.json','frozen-observer/pg_observer.rs','frozen-observer/build.json']}}
p=OUT/'independent-assessment.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS actual native Use/Cancel, unsaved candle rollback and real restart')
