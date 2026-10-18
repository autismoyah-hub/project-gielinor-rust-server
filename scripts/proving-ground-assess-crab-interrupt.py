"""Independent durable captured-prefix assessment after normal game-client departure."""
import json,re,math,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='894646a79fa4ba7e3e8b46af6ec000dc6b796514';OUT=ROOT/'docs/openrune/verification/independent/deep-crab-interrupt/interrupt-002';load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');order=[26,*range(4,26)];by_slot={r[1]:r for r in after['inventory'] if r[0]==93};credited=[s for s in order if by_slot[s][2]!=7538];n=len(credited);assert 0<n<23 and credited==order[:n]
expected=[r.copy() for r in initial['inventory']];award=0;good=bad=0
for slot in credited:
 row=by_slot[slot];assert row[2] in [7539,7541] and row[3:]==[1,0];next(r for r in expected if r[:2]==[93,slot])[2]=row[2];award+=1625 if row[2]==7539 else 0;good+=row[2]==7539;bad+=row[2]==7541
assert after['inventory']==post['inventory']==expected and load('relogin/before.json')==after
points=0;level=1
for i in range(1,99):
 points+=math.floor(i+300*2**(i/7))
 if 24110+award>=(points//4)*10:level=i+1
for x,y,z in zip(initial['stats'],after['stats'],post['stats']):
 assert x[0]==y[0] and y==z
 if x[0]==12:assert y[3]==24110+award and y[1]==y[2]==level
 else:assert x==y
assert initial['character'][3]==after['character'][3]==post['character'][3]
for phase in ['produce','relogin']:
 m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS' and m['client_exit']==m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown'];assert load(phase+'/after.json')['integrity_check']=='ok'
log=(OUT/'produce/client.log').read_text(encoding='utf-8');matches=re.findall(r'PG_CRAB_INTERRUPT_READY tick=(\d+) made=(\d+) good=(\d+) broken=(\d+) fine_observed=(\d+)',log);assert len(matches)==1;tick,made,observed_good,observed_bad,fine=map(int,matches[0]);assert made==observed_good+observed_bad and 0<made<=n and fine==24110+1625*observed_good+0*observed_bad
uses=re.findall(r'PG_WIRE_TX tick=\d+ opcode=27 payload=\[(.*?)\]',log);makes=re.findall(r'PG_WIRE_TX tick=\d+ opcode=115 payload=\[(.*?)\]',log);assert len(uses)==len(makes)==1 and bytes.fromhex(makes[0].replace(',',''))==bytes.fromhex('0e010f000017')
b=bytes.fromhex(uses[0].replace(',',''));beadd=lambda b:(b[0]<<8)|((b[1]-128)&255);leadd=lambda b:((b[0]-128)&255)|(b[1]<<8);assert (beadd(b[4:6]),beadd(b[10:12]),int.from_bytes(b[8:10],'little'),leadd(b[6:8]))==(26,7538,0,1755)
observed={};native_xp=[]
for op,h in re.findall(r'PG_RX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log):
 b=bytes.fromhex(h.replace(',',''));op=int(op)
 if op==46 and (b[2]-128)&255==12:native_xp.append(int.from_bytes(b[5:7]+b[3:5],'big'))
 if op!=18 or int.from_bytes(b[4:6],'big')!=93:continue
 pos=6
 while pos<len(b):
  slot=b[pos];pos+=1
  if slot>=128:slot=((slot<<8)|b[pos])-32768;pos+=1
  item=int.from_bytes(b[pos:pos+2],'big')-1;pos+=2;count=0
  if item>=0:
   count=b[pos];pos+=1
   if count==255:count=int.from_bytes(b[pos:pos+4],'big');pos+=4
  if slot in order and item in [7539,7541]:assert count==1;observed[slot]=item
 assert pos==len(b)
assert len(observed)==made and set(observed)==set(order[:made]) and native_xp[-1]==fine//10
assert all(by_slot[s][2]==item for s,item in observed.items())
invs={};stats={}
for _,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',(OUT/'relogin/client.log').read_text(encoding='utf-8')):
 b=bytes.fromhex(h.replace(',',''));op=int(op)
 if op==46:stats[(b[2]-128)&255]=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'))
 if op!=22:continue
 inv=int.from_bytes(b[4:6],'big');size=int.from_bytes(b[6:8],'big');pos=8;items=[]
 for slot in range(size):
  count=(128-b[pos])&255;pos+=1
  if count==255:c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
  item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2
  if item>=0:items.append([inv,slot,item,count,0])
 assert pos==len(b);invs[inv]=items
assert invs[93]==expected
for skill,current,base,fine in post['stats']:assert stats[skill]==(current,fine//10)
report={'implementation_id':'OR-MIG-0068','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','native_observed_products':made,'durable_products':n,'durable_carved':good,'durable_broken':bad,'durable_fine_xp_award':award,'remaining_raws':23-n,'captured_credit_order':credited,'scope':'Real native reverse held Use and All23; normal client departure after a consistent observed carved/broken inventory/XP prefix. Acknowledged storage preserves exact selected-first prefix and remaining raws/tools/coins without fallback. Actual process restart and independently decoded native full inventory/Stats preserve the whole saved state with no continued queue or reward replay.','limitations':['Additional already-admitted credits before disconnect processing are included in durable prefix, not mislabeled as duplication.','Normal acknowledged save/restart is not crash atomicity or a mid-transaction failure.','Native custom-Xand applicable crash acceptance remain separate outstanding obligations.'],'evidence_sha256':{p:hashlib.sha256((OUT/p).read_bytes()).hexdigest() for p in ['produce/client.log','produce/manifest.json','relogin/client.log','relogin/manifest.json']}}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
