"""Independent SQL, level-curve, native packet, cadence and reward assessment."""
import json,re,math,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='0c489b46b552ad012e630810c1218dbaeb437a51';OUT=ROOT/'docs/openrune/verification/independent/deep-fallible-gems/native-001';load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');expected=[r.copy() for r in initial['inventory']];profiles=[(1625,1609,150,38,890),(1627,1611,200,50,891),(1629,1613,250,63,892)];award=0;outcomes=[]
for i,(raw,cut,xp,fail,seq) in enumerate(profiles):
 outputs=[]
 for slot in range(4+7*i,11+7*i):
  row=next(r for r in after['inventory'] if r[:2]==[93,slot]);assert row[2] in [cut,1633] and row[3:]==[1,0];next(r for r in expected if r[:2]==[93,slot])[2]=row[2];outputs.append(row[2]);award+=(xp if row[2]==cut else fail)*5
 outcomes.append({'raw':raw,'cut':cut,'cut_count':outputs.count(cut),'crushed_count':outputs.count(1633),'both_natural_outcomes_observed':cut in outputs and 1633 in outputs})
assert after['inventory']==post['inventory']==expected and load('relogin/before.json')==after
def level_for_fine(fine):
 points=0;level=1
 for i in range(1,99):
  points+=math.floor(i+300*2**(i/7))
  if fine>=(points//4)*10:level=i+1
 return level
assert level_for_fine(27460)==16
for x,y,z in zip(initial['stats'],after['stats'],post['stats']):
 assert x[0]==y[0] and y==z
 if x[0]==12:assert y[3]==x[3]+award and y[1]==y[2]==level_for_fine(y[3])
 else:assert x==y
assert initial['character'][3]==after['character'][3]==post['character'][3]
for phase in ['produce','relogin']:
 m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS' and m['client_exit']==m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown'];assert load(phase+'/after.json')['integrity_check']=='ok'
log=(OUT/'produce/client.log').read_text(encoding='utf-8');lines=log.splitlines();assert 'PG_CASE_PASS native fallible Make/Close and stale response preserve full inventory/XP' in log and 'PG_CASE_PASS native fallible three profiles both held directions' in log
wire=[(n,int(op),bytes.fromhex(h.replace(',',''))) for n,line in enumerate(lines) if (m:=re.fullmatch(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',line)) for op,h in [m.groups()]]
uses=[(n,b) for n,op,b in wire if op==27];makes=[(n,b) for n,op,b in wire if op==115];assert len(uses)==len(makes)==7
for (n,b),count in zip(makes,[1,1,6,1,6,1,6]):assert b==bytes.fromhex('0e010f00')+count.to_bytes(2,'big')
beadd=lambda b:(b[0]<<8)|((b[1]-128)&255);leadd=lambda b:((b[0]-128)&255)|(b[1]<<8)
for (_,b),(selected,first,target,second) in zip(uses,[(0,1755,4,1625),*[(s,first,t,second) for i,(raw,*_) in enumerate(profiles) for s,first,t,second in [(0,1755,10+7*i,raw),(4+7*i,raw,0,1755)]]]):assert len(b)==16 and (beadd(b[4:6]),beadd(b[10:12]),int.from_bytes(b[8:10],'little'),leadd(b[6:8]))==(selected,first,target,second)
effects=[];seen=set();batch_first_sequences=[]
for batch,(line_index,_) in enumerate(makes[1:]):
 seq=profiles[batch//2][4];next_wire=makes[batch+2][0] if batch+2<len(makes) else len(lines)
 match=next(re.fullmatch(r'PG_SEQUENCE tick=(\d+) sequence='+str(seq)+r' delay=(\d+)',line) for line in lines[line_index+1:next_wire] if re.fullmatch(r'PG_SEQUENCE tick=(\d+) sequence='+str(seq)+r' delay=(\d+)',line));assert int(match[2])==0;batch_first_sequences.append(int(match[1]))
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
  if inv==93 and 4<=slot<=24 and slot not in seen and item in [profiles[(slot-4)//7][1],1633]:
   i=(slot-4)//7;offset=(slot-4)%7;batch=2*i+(0 if offset==6 else 1);step=0 if offset==6 else offset;assert count==1 and tick-batch_first_sequences[batch]==1+2*step,(slot,tick,batch_first_sequences[batch]);effects.append({'slot':slot,'output':item,'batch':batch,'sequence':profiles[i][4],'first_sequence_tick':batch_first_sequences[batch],'credit_tick':tick});seen.add(slot)
 assert pos==len(b)
assert len(effects)==21
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
report={'implementation_id':'OR-MIG-0064','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','actions':21,'outcomes':outcomes,'fine_xp_award':award,'effects':effects,'scope':'Actual startup-registered three semiprecious native choices with genuine1/All six-item quantities and both held directions per profile.21selected full-bag cut/crushed credits and branch-specific fineXP match independent SQL oracle; current/base Crafting match independent level curve. Native first1/repeat2sequence clocks independently decoded. Real Make/Close and stale response/selected-pair refusals do not consume duplicates. Actual save/process restart/native full inventory/Stats decoding preserve complete output/reward state.','limitations':['Natural branch witnesses are recorded per profile; no probability/distribution proof or forced native RNG.','Native custom-X remains unaccepted; PG-INFRA-009 is a separate shared dependency.','Interrupted captured batches and PG-REF-001 reference conflicts remain separate; this scoped PASS is not full V0-V4 acceptance.','No rendered animation frames/audio or original executable parity claimed.'],'evidence_sha256':{p:hashlib.sha256((OUT/p).read_bytes()).hexdigest() for p in ['produce/client.log','produce/manifest.json','relogin/client.log','relogin/manifest.json']}}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
