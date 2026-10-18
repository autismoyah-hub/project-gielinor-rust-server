"""Independent builtin5/10quantity, durable state and raw cadence oracles."""
import json,re,math,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='894646a79fa4ba7e3e8b46af6ec000dc6b796514';OUT=ROOT/'docs/openrune/verification/independent/deep-crab-quantities/quantities-001';load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');expected=[r.copy() for r in initial['inventory']];batches=[[26,4,5,6,7],list(range(8,18))];award=good=bad=0
for slot in sum(batches,[]):
 row=next(r for r in after['inventory'] if r[:2]==[93,slot]);assert row[2] in[7539,7541] and row[3:]==[1,0];next(r for r in expected if r[:2]==[93,slot])[2]=row[2];good+=row[2]==7539;bad+=row[2]==7541;award+=1625 if row[2]==7539 else 0
assert good+bad==15 and sum(r[3] for r in expected if r[2]==7538)==8
assert after['inventory']==post['inventory']==expected and load('relogin/before.json')==after
points=0;level=1
for i in range(1,99):
 points+=math.floor(i+300*2**(i/7))
 if 24110+award>=(points//4)*10:level=i+1
for x,y,z in zip(initial['stats'],after['stats'],post['stats']):
 assert x[0]==y[0] and y==z
 if x[0]==12:assert y==[12,level,level,24110+award]
 else:assert x==y
assert initial['character'][3]==after['character'][3]==post['character'][3]
for phase in['produce','relogin']:
 m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS' and m['client_exit']==m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown'];assert load(phase+'/after.json')['integrity_check']=='ok'
log=(OUT/'produce/client.log').read_text(encoding='utf-8');lines=log.splitlines();assert 'PG_CASE_PASS native crab builtin5then10genuine quantity frames' in log
wire=[(n,int(m[1]),bytes.fromhex(m[2].replace(',',''))) for n,line in enumerate(lines) if(m:=re.fullmatch(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',line))]
uses=[b for _,op,b in wire if op==27];makes=[(n,b) for n,op,b in wire if op==115];assert len(uses)==len(makes)==2
for(_,b),count in zip(makes,[5,10]):assert b==bytes.fromhex('0e010f00')+count.to_bytes(2,'big')
beadd=lambda b:(b[0]<<8)|((b[1]-128)&255);leadd=lambda b:((b[0]-128)&255)|(b[1]<<8)
for b,pair in zip(uses,[(0,1755,26,7538),(8,7538,0,1755)]):assert len(b)==16 and(beadd(b[4:6]),beadd(b[10:12]),int.from_bytes(b[8:10],'little'),leadd(b[6:8]))==pair
assert log.count('PG_REPLAY label="native stale completed crab builtin quantity"')==2 and log.count('PG_REPLAY label="native stale selected crab builtin held pair"')==2
starts=[]
for i,(line,_)in enumerate(makes):
 end=makes[i+1][0] if i+1<len(makes)else len(lines)
 m=next(re.fullmatch(r'PG_SEQUENCE tick=(\d+) sequence=1280 delay=(\d+)',s) for s in lines[line+1:end] if re.fullmatch(r'PG_SEQUENCE tick=(\d+) sequence=1280 delay=(\d+)',s));assert int(m[2])==0;starts.append(int(m[1]))
effects=[];seen=set()
for line in lines:
 m=re.fullmatch(r'PG_RX tick=(\d+) opcode=18 payload=\[(.*?)\]',line)
 if not m:continue
 tick=int(m[1]);b=bytes.fromhex(m[2].replace(',',''));inv=int.from_bytes(b[4:6],'big');pos=6
 while pos<len(b):
  slot=b[pos];pos+=1
  if slot>=128:slot=((slot<<8)|b[pos])-32768;pos+=1
  item=int.from_bytes(b[pos:pos+2],'big')-1;pos+=2;count=0
  if item>=0:
   count=b[pos];pos+=1
   if count==255:count=int.from_bytes(b[pos:pos+4],'big');pos+=4
  if inv==93 and slot in sum(batches,[])and slot not in seen and item in[7539,7541]:
   batch=0 if slot in batches[0]else 1;step=batches[batch].index(slot);assert count==1 and tick-starts[batch]==3+3*step;effects.append({'batch':batch,'step':step,'slot':slot,'output':item,'credit_tick':tick,'sequence_tick':starts[batch]});seen.add(slot)
 assert pos==len(b)
assert len(effects)==15
invs={};stats={}
for op,h in re.findall(r'PG_RX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',(OUT/'relogin/client.log').read_text(encoding='utf-8')):
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
report={'implementation_id':'OR-MIG-0068','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','quantities':[5,10],'products':15,'good':good,'broken':bad,'fine_xp_award':award,'remaining_raws':8,'effects':effects,'scope':'Genuine native builtin5and10quantity frames, both held directions, exact captured selected-first slot order/full28and15carved/broken rewards. Exact per-outcome fineXP, retained duplicate tools/1000coins/8raws, stale completed quantity/held refusal, first3/repeat3effects and acknowledged-save/actual native restart state all independently verified.','limitations':['Builtin quantity scope only; custom-Xinput remains a separate unresolved dependency.','No rendered/audio or natural probability claim.'],'evidence_sha256':{p:hashlib.sha256((OUT/p).read_bytes()).hexdigest() for p in['produce/client.log','produce/manifest.json','relogin/client.log','relogin/manifest.json']}}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
