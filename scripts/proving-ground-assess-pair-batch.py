"""Independent captured-pair quantity and interrupted persistence oracles."""
import argparse,json,re,hashlib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('feature',choices=['pair-counts','pair-ten','pair-x','pair-earth-batch','pair-pending','pair-interrupt','pair-crash']);p.add_argument('--run-id',default='native-001');a=p.parse_args()
ROOT=Path(__file__).resolve().parents[1];SHA='fb348dda184b36ec2d503bfe0f1d5393cf8ae151';OUT=ROOT/'docs/openrune/verification/independent'/('deep-'+a.feature)/a.run_id
load=lambda name:json.loads((OUT/name).read_text(encoding='utf-8'))
first_type,second_type,output_type,fine_award,first_delay=(1391,575,1399,5625,3) if a.feature=='pair-earth-batch' else (1759,1673,1692,200,1)
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');crash=a.feature=='pair-crash';partial=a.feature=='pair-interrupt';pending=a.feature=='pair-pending'
assert initial['inventory']==[[93,s,first_type if s<13 else second_type if s<26 else 1931 if s==26 else 995,1000 if s==27 else 1,0] for s in range(28)]
log=(OUT/'produce/client.log').read_text(encoding='utf-8');plan=[] if pending else [1,5,7] if a.feature=='pair-counts' else [10,3] if a.feature=='pair-ten' else [3,10] if a.feature=='pair-x' else [13]
observed=None
if partial or crash:
 m=re.search(r'PG_PAIR_INTERRUPT_READY products=(\d+) fine_observed=(\d+) raw_remaining=(\d+) wool_remaining=(\d+)',log);assert m
 observed=int(m[1]);assert 0<observed<13 and int(m[2])==130344311+observed*fine_award and int(m[3])==int(m[4])==13-observed
else:assert 'PG_CASE_PASS native pair batch kind='+a.feature in log
durable=sum(r[3] for r in after['inventory'] if r[2]==output_type)
assert durable==(0 if crash or pending else 13) if not partial else observed<=durable<13
expected=[r.copy() for r in initial['inventory']];actions=[];batches=[]
for batch,count in enumerate(plan):
 ws=sorted(r[1] for r in expected if r[2]==first_type);rs=sorted(r[1] for r in expected if r[2]==second_type);w=ws.pop();r=rs.pop();pairs=[(w,r),*zip(ws,rs)]
 batches.append({'batch':batch,'selected_wool':w,'selected_raw':r,'requested':count})
 for step,(w,r) in enumerate(pairs[:count]):
  expected=[x for x in expected if x[1] not in [w,r]];free=next(s for s in range(28) if not any(x[1]==s for x in expected));expected.append([93,free,output_type,1,0]);expected.sort();actions.append({'batch':batch,'step':step,'first_slot':w,'second_slot':r,'output_slot':free})
  if partial and len(actions)==durable:break
 if partial:break
if crash:expected=initial['inventory']
assert after['inventory']==post['inventory']==expected and load('relogin/before.json')==after
assert len(initial['stats'])==len(after['stats'])==len(post['stats'])
for x,y,z in zip(initial['stats'],after['stats'],post['stats']):
 assert x[0]==y[0] and y==z
 if x[0]==12:assert x==[12,99,99,130344311] and y==[12,99,99,130344311+durable*fine_award]
 else:assert x==y
assert initial['character'][3]==after['character'][3]==post['character'][3]
for phase in ['produce','relogin']:
 m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS' and m['client_exit']==0 and m['resource_hashes_unchanged'];assert load(phase+'/after.json')['integrity_check']=='ok'
 if crash and phase=='produce':assert m['server_exit']!=0 and m['intentional_crash_injected'] and 'owned disposable' in m['fault_scope'] and not m['save_acknowledgement'] and not m['graceful_shutdown'];assert 'PG_EXPECTED_CRASH_DISCONNECT' in log
 else:assert m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and not m['forced_shutdown']
if a.feature=='pair-x':
 assert 'PG_X_INPUT_PROMPT' in log and 'Enter amount (1 - 13):' in log
 assert 'PG_X_FOCUSED_TYPED count=3 prompt_observed=true separate_pointer_frames=true' in log
wire=[(int(op),bytes.fromhex(h.replace(',',''))) for op,h in re.findall(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log)]
uses=[b for op,b in wire if op==27];makes=[b for op,b in wire if op==115];assert (len(uses)==1 and len(makes)==0) if pending else len(uses)==len(makes)==len(plan)
beadd=lambda b:(b[0]<<8)|((b[1]-128)&255);leadd=lambda b:((b[0]-128)&255)|(b[1]<<8)
if pending:
 b=uses[0];assert len(b)==16 and (beadd(b[4:6]),beadd(b[10:12]),int.from_bytes(b[8:10],'little'),leadd(b[6:8]))==(12,1759,25,1673)
 assert 'PG_NATIVE_NO_PENDING_SKILL_MULTI' in (OUT/'relogin/client.log').read_text(encoding='utf-8')
 assert 'PG_GLASS_OPTION_READY' in log
for (i,b),m,batch in zip(enumerate(uses),makes,batches):
 assert m==bytes.fromhex('0e010f00')+plan[i].to_bytes(2,'big')
 pair=(batch['selected_wool'],first_type,batch['selected_raw'],second_type) if i%2==0 else (batch['selected_raw'],second_type,batch['selected_wool'],first_type)
 assert len(b)==16 and (beadd(b[4:6]),beadd(b[10:12]),int.from_bytes(b[8:10],'little'),leadd(b[6:8]))==pair
traces=[json.loads(s.split(': ',1)[1]) for s in (OUT/'produce/server.log').read_text().splitlines() if s.startswith('Assembly transition: ')]
if not crash:
 assert len(traces)==durable
 for t,v in zip(traces,actions):
  assert (t['first_slot'],t['second_slot'],t['first'],t['second'],t['output'],t['count'])==(v['first_slot'],v['second_slot'],first_type,second_type,output_type,1)
  assert t['committed_at']-t['queued_at']==(first_delay if v['step']==0 else 2)
 if not partial:assert log.count('PG_REPLAY label="native stale completed pair batch quantity"')==len(plan) and log.count('PG_REPLAY label="native stale consumed pair batch request"')==len(plan)
native_effects=[]
if a.feature=='pair-earth-batch':
 import ast
 sequence=next(int(m[1]) for line in log.splitlines() if (m:=re.fullmatch(r'PG_SEQUENCE tick=(\d+) sequence=7531 delay=0',line)))
 spots=[(int(m[1]),ast.literal_eval(m[2])) for line in log.splitlines() if (m:=re.fullmatch(r'PG_SPOTS tick=(\d+) decoded_slot_id_height_delay=(\[.*\])',line))]
 assert any(t==sequence and (0,1371,0,0) in values for t,values in spots)
 seen=set()
 for line in log.splitlines():
  m=re.fullmatch(r'PG_RX tick=(\d+) opcode=18 payload=\[(.*?)\]',line)
  if not m:continue
  tick=int(m[1]);b=bytes.fromhex(m[2].replace(',',''));pos=6
  if int.from_bytes(b[4:6],'big')!=93:continue
  while pos<len(b):
   slot=b[pos];pos+=1
   if slot>=128:slot=((slot<<8)|b[pos])-32768;pos+=1
   item=int.from_bytes(b[pos:pos+2],'big')-1;pos+=2;count=0
   if item>=0:
    count=b[pos];pos+=1
    if count==255:count=int.from_bytes(b[pos:pos+4],'big');pos+=4
   if item==1399 and slot not in seen:
    step=len(native_effects);assert count==1 and slot==actions[step]['output_slot'] and tick-sequence==3+2*step
    native_effects.append({'step':step,'slot':slot,'output':item,'sequence':7531,'spot':1371,'first_sequence_tick':sequence,'credit_tick':tick});seen.add(slot)
  assert pos==len(b)
 assert len(native_effects)==13
 assert next(v[3] for v in post['stats'] if v[0]==12)==130417436
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
report={'implementation_id':'OR-MIG-0069','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','feature':a.feature,'recipe':{'first':first_type,'second':second_type,'output':output_type,'fine_per_credit':fine_award,'first_delay':first_delay,'repeat_delay':2},'requested_counts':plan,'observed_partial_products':observed,'durable_products':durable,'fine_xp_award':durable*fine_award,'actions':actions if not crash else [],'native_effects':native_effects,'traces':traces,'scope':('Genuine native quantity dialog opened without a Make reply; one held packet/no115/no product or XP. Full inventory/Stats/energy preserved through acknowledged save and real process/native restart. Raw22/46independently decoded and no visible270dialog after reconnect. This is pending-choice invalidation, not completed production.' if pending else ('Genuine captured two-input native production with independent complete inventory/Stats/energy, selected-first/ascending pairs, first-free outputs, fineXP and real restarted raw22/46state. '+('Real SkillMulti dialog opened without a Make reply; no115sent/no products or XP; exact prior state and no visible270dialog after actual restart.' if pending else 'Owned unacknowledged termination restores prior whole state; no mid-SQL fault claimed.' if crash else 'Consistent acknowledged partial prefix persists without queue replay.' if partial else 'Native one-digit customX3with observed prompt/keyboard and subsequent All10, exact first1/repeat2and stale refusal pass; multi-digitX10remains separate.' if a.feature=='pair-x' else 'Builtin quantities, profile-specific first/repeat2server clocks and stale completed quantity/held refusal pass.'))),'limitations':['Single declared recipe batch scope; other profile/timing/member/modifier cases remain separately identified.','Only the recorded quantities are accepted; multi-digit X10remains separate. No rendered/audio/original executable parity claimed.'],'evidence_sha256':{p:hashlib.sha256((OUT/p).read_bytes()).hexdigest() for p in ['produce/client.log','produce/server.log','produce/manifest.json','relogin/client.log','relogin/manifest.json']}}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
