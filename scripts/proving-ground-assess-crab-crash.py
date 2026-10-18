"""Verify prior durable state after an owned unsaved crab-production prefix."""
import json,re,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='894646a79fa4ba7e3e8b46af6ec000dc6b796514';OUT=ROOT/'docs/openrune/verification/independent/deep-crab-crash/crash-001';load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');m=load('produce/manifest.json');restart=load('relogin/manifest.json')
assert m['tested_sha']==m['implementation_sha']==restart['tested_sha']==restart['implementation_sha']==SHA
assert m['status']=='PASS' and m['client_exit']==0 and m['server_exit']!=0 and m['intentional_crash_injected'] and 'crab-carving' in m['fault_scope']
assert not m['save_acknowledgement'] and not m['graceful_shutdown'] and m['resource_hashes_unchanged']
assert restart['status']=='PASS' and restart['client_exit']==restart['server_exit']==0 and restart['save_acknowledgement'] and restart['graceful_shutdown'] and restart['resource_hashes_unchanged'] and not restart['forced_shutdown']
for state in[after,post]:assert state['integrity_check']=='ok' and state['inventory']==initial['inventory'] and state['stats']==initial['stats'] and state['character'][3]==initial['character'][3]
assert load('relogin/before.json')==after
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'PG_CRASH_READY native crab captured All observed unsaved prefix' in log and 'PG_EXPECTED_CRASH_DISCONNECT' in log
matches=re.findall(r'PG_CRAB_INTERRUPT_READY tick=(\d+) made=(\d+) good=(\d+) broken=(\d+) fine_observed=(\d+)',log);assert len(matches)==1
tick,made,good,bad,total=map(int,matches[0]);assert 0<made<23 and made==good+bad and total==24110+1625*good
observed={};xp_seen=set();order=[26,*range(4,26)]
for op,hexes in re.findall(r'PG_RX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log):
 b=bytes.fromhex(hexes.replace(',',''));op=int(op)
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
  if slot in order and item in[7539,7541]:assert count==1;observed[slot]=item
 assert pos==len(b)
assert len(observed)>=made and set(order[:made])<=set(observed) and total//10 in xp_seen
assert sum(observed[s]==7539 for s in order[:made])==good and sum(observed[s]==7541 for s in order[:made])==bad
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
assert invs[93]==initial['inventory']
for skill,current,base,fine in initial['stats']:assert stats[skill]==(current,fine//10)
report={'implementation_id':'OR-MIG-0068','tested_sha':SHA,'status':'PASS','campaign_kind':'historical','native_observed_products_before_ready':made,'observed_good':good,'observed_broken':bad,'observed_fine_award_before_ready':1625*good,'durable_outputs':0,'scope':'Captured All23started through genuine native held Use and quantity transport. Consistent carved/broken inventory prefix observed, including valid zero-XP broken outcomes, before termination of only the owned disposable server without save acknowledgement. SQL and independently decoded actual native restart restore complete prior inventory/Stats/energy; no continued queue, duplicated products or XP.','limitations':['This is unacknowledged-state rollback, not injection during SQL transaction commit.','Further already-admitted native observations after ready may appear before owned termination; prior durable whole-state oracle is unchanged.','No natural probability or rendered/audio claim.'],'evidence_sha256':{p:hashlib.sha256((OUT/p).read_bytes()).hexdigest() for p in['produce/client.log','produce/manifest.json','relogin/client.log','relogin/manifest.json']}}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
