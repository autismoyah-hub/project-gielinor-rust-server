"""Verify owned abrupt termination restores prior whole immediate-crafting state."""
import json,re,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='4f4aacb2929e8ad775d091527e2d80070100ee69';OUT=ROOT/'docs/openrune/verification/independent/deep-immediate-crash/crash-001';load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');m=load('produce/manifest.json');restart=load('relogin/manifest.json')
assert m['tested_sha']==m['implementation_sha']==restart['tested_sha']==restart['implementation_sha']==SHA
assert m['status']=='PASS' and m['client_exit']==0 and m['server_exit']!=0 and m['intentional_crash_injected'] and 'immediate limestone' in m['fault_scope']
assert not m['save_acknowledgement'] and not m['graceful_shutdown'] and m['resource_hashes_unchanged']
assert restart['status']=='PASS' and restart['client_exit']==restart['server_exit']==0 and restart['save_acknowledgement'] and restart['graceful_shutdown'] and restart['resource_hashes_unchanged'] and not restart['forced_shutdown']
for state in [after,post]:assert state['integrity_check']=='ok' and state['inventory']==initial['inventory'] and state['stats']==initial['stats'] and state['character'][3]==initial['character'][3]
assert load('relogin/before.json')==after
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'PG_CRASH_READY immediate one unsaved limestone credit and fineXP observed' in log and 'PG_EXPECTED_CRASH_DISCONNECT' in log
committed=re.findall(r'PG_IMMEDIATE_COMMITTED kind=immediate-crash index=0 selected_slot=4 raw=3211 output=(\d+) fine_award=(\d+) fine_total=(\d+)',log);assert len(committed)==1
output,award,total=map(int,committed[0]);assert (output,award) in [(3420,300),(968,75)] and total==15840+award
observed_output=False;observed_xp=False
for op,hexes in re.findall(r'PG_RX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log):
 b=bytes.fromhex(hexes.replace(',',''));op=int(op)
 if op==46 and (b[2]-128)&255==12 and int.from_bytes(b[5:7]+b[3:5],'big')==total//10:observed_xp=True
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
assert observed_output and observed_xp
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
report={'implementation_id':'OR-MIG-0065','tested_sha':SHA,'status':'PASS','campaign_kind':'historical','observed_unsaved_output':output,'observed_unsaved_fine_award':award,'durable_outputs':0,'scope':'One native immediate limestone credit/reward observed, stale request refused, then only the owned server terminated with socket open and no save acknowledgement. SQL retains prior complete inventory/Stats; actual process restart and independently decoded native full inventory/Stats confirm complete rollback, no replay or material/XP duplication.','limitations':['This is unacknowledged-state rollback and recovery, not an injection during SQL transaction commit.'],'evidence_sha256':{p:hashlib.sha256((OUT/p).read_bytes()).hexdigest() for p in ['produce/client.log','produce/manifest.json','relogin/client.log','relogin/manifest.json']}}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
