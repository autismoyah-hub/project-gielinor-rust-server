"""Independent SQL, level-curve, native packet, cadence and reward assessment."""
import argparse,json,re,math,hashlib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--relogin-phase',default='relogin');args=p.parse_args();assert re.fullmatch('relogin(?:-[a-z0-9-]{1,16})?',args.relogin_phase)
ROOT=Path(__file__).resolve().parents[1];SHA='894646a79fa4ba7e3e8b46af6ec000dc6b796514';OUT=ROOT/'docs/openrune/verification/independent/deep-crab-carving/native-001'
def source(name):return OUT/(args.relogin_phase+name[len('relogin'):] if name.startswith('relogin/') else name)
load=lambda p:json.loads(source(p).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');expected=[r.copy() for r in initial['inventory']];profiles=[(7538,7539,7541,325,0,1280),(7536,7537,7540,325,0,1280)];award=0;outcomes=[]
for i,(raw,cut,broken,xp,fail,seq) in enumerate(profiles):
 outputs=[]
 for slot in range(2+12*i,14+12*i):
  row=next(r for r in after['inventory'] if r[:2]==[93,slot]);assert row[2] in [cut,broken] and row[3:]==[1,0];next(r for r in expected if r[:2]==[93,slot])[2]=row[2];outputs.append(row[2]);award+=(xp if row[2]==cut else fail)*5
 outcomes.append({'raw':raw,'cut':cut,'cut_count':outputs.count(cut),'broken_count':outputs.count(broken),'both_natural_outcomes_observed':cut in outputs and broken in outputs})
assert after['inventory']==post['inventory']==expected
if args.relogin_phase=='relogin':assert load('relogin/before.json')==after
else:
    failed=json.loads((OUT/'relogin/manifest.json').read_text());failed_after=json.loads((OUT/'relogin/after.json').read_text())
    assert failed['status']=='FAIL' and failed['client_sha256']==load('produce/manifest.json')['client_sha256']==load('relogin/manifest.json')['client_sha256']
    assert load('relogin/before.json')==failed_after
    for key in ['inventory','stats']:assert failed_after[key]==after[key]
    assert failed_after['character'][3]==after['character'][3]
def level_for_fine(fine):
 points=0;level=1
 for i in range(1,99):
  points+=math.floor(i+300*2**(i/7))
  if fine>=(points//4)*10:level=i+1
 return level
assert level_for_fine(24110)==15
for x,y,z in zip(initial['stats'],after['stats'],post['stats']):
 assert x[0]==y[0] and y==z
 if x[0]==12:assert y[3]==x[3]+award and y[1]==y[2]==level_for_fine(y[3])
 else:assert x==y
assert initial['character'][3]==after['character'][3]==post['character'][3]
for phase in ['produce','relogin']:
 m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS' and m['client_exit']==m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown'];assert load(phase+'/after.json')['integrity_check']=='ok'
log=(OUT/'produce/client.log').read_text(encoding='utf-8');lines=log.splitlines();assert 'PG_CASE_PASS native crab Make/Close and stale response preserve full inventory/XP' in log and 'PG_CASE_PASS native crab two profiles both held directions' in log
wire=[(n,int(op),bytes.fromhex(h.replace(',',''))) for n,line in enumerate(lines) if (m:=re.fullmatch(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',line)) for op,h in [m.groups()]]
uses=[(n,b) for n,op,b in wire if op==27];makes=[(n,b) for n,op,b in wire if op==115];assert len(uses)==len(makes)==5
for (n,b),count in zip(makes,[1,1,11,1,11]):assert b==bytes.fromhex('0e010f00')+count.to_bytes(2,'big')
beadd=lambda b:(b[0]<<8)|((b[1]-128)&255);leadd=lambda b:((b[0]-128)&255)|(b[1]<<8)
for (_,b),(selected,first,target,second) in zip(uses,[(0,1755,2,7538),*[(s,first,t,second) for i,(raw,*_) in enumerate(profiles) for s,first,t,second in [(0,1755,13+12*i,raw),(2+12*i,raw,0,1755)]]]):assert len(b)==16 and (beadd(b[4:6]),beadd(b[10:12]),int.from_bytes(b[8:10],'little'),leadd(b[6:8]))==(selected,first,target,second)
effects=[];seen=set();batch_first_sequences=[]
for batch,(line_index,_) in enumerate(makes[1:]):
 seq=profiles[batch//2][5];next_wire=makes[batch+2][0] if batch+2<len(makes) else len(lines)
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
  if inv==93 and 2<=slot<=25 and slot not in seen and item in [profiles[(slot-2)//12][1],profiles[(slot-2)//12][2]]:
   i=(slot-2)//12;offset=(slot-2)%12;batch=2*i+(0 if offset==11 else 1);step=0 if offset==11 else offset;assert count==1 and tick-batch_first_sequences[batch]==3+3*step,(slot,tick,batch_first_sequences[batch]);effects.append({'slot':slot,'output':item,'batch':batch,'sequence':profiles[i][5],'first_sequence_tick':batch_first_sequences[batch],'credit_tick':tick});seen.add(slot)
 assert pos==len(b)
assert len(effects)==24
invs={};stats={}
for _,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',source('relogin/client.log').read_text(encoding='utf-8')):
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
for i,(raw,cut,broken,xp,failed,seq) in enumerate(profiles):
    messages=re.findall(r'PG_CRAB_MESSAGE index='+str(i)+r' second=(?:false|true) message="(.*?)"',log)
    assert messages.count('You carve the '+('Crab helmet' if i==0 else 'Crab claw')+'.')==outcomes[i]['cut_count']
    assert messages.count('You accidentally break the crab material.')==outcomes[i]['broken_count']
report={'implementation_id':'OR-MIG-0068','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','actions':24,'outcomes':outcomes,'fine_xp_award':award,'effects':effects,'scope':'Actual startup-registered two crab choices with genuine1/All eleven-item quantities and both held directions per profile.24selected full-bag carved/broken credits and success325tenths/failure0fineXP match independent SQL oracle; current/base Crafting match independent level curve. Native first3/repeat3receive clocks independently decoded. Actual branch feedback counts match observed products. Make/Close and stale quantity/selected-pair refusals conserve full state. Actual save/process restart/native full inventory/Stats decoding retain all output/reward state.','limitations':['Natural branch witnesses recorded per profile; no probability/distribution proof or forced native RNG.','Native custom-X remains unaccepted; PG-INFRA-009is a separate shared dependency.','Interrupted captured batches and applicable crash/death/native boundary work remain separate; this scoped PASS is not full V0-V4 acceptance.','No rendered animation frames/audio or original executable parity claimed.'],'evidence_sha256':{source(p).relative_to(OUT).as_posix():hashlib.sha256(source(p).read_bytes()).hexdigest() for p in ['produce/client.log','produce/manifest.json','relogin/client.log','relogin/manifest.json']}}
report['relogin_phase']=args.relogin_phase
if args.relogin_phase!='relogin':report['preserved_failed_relogin_manifest_sha256']=hashlib.sha256((OUT/'relogin/manifest.json').read_bytes()).hexdigest()
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
