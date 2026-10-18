"""Independent captured-pair quantity and interrupted persistence oracles."""
import argparse,json,re,hashlib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('feature',choices=['pair-counts','pair-ten','pair-interrupt','pair-crash']);p.add_argument('--run-id',default='native-001');a=p.parse_args()
ROOT=Path(__file__).resolve().parents[1];SHA='fb348dda184b36ec2d503bfe0f1d5393cf8ae151';OUT=ROOT/'docs/openrune/verification/independent'/('deep-'+a.feature)/a.run_id
load=lambda name:json.loads((OUT/name).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');crash=a.feature=='pair-crash';partial=a.feature=='pair-interrupt'
assert initial['inventory']==[[93,s,1759 if s<13 else 1673 if s<26 else 1931 if s==26 else 995,1000 if s==27 else 1,0] for s in range(28)]
log=(OUT/'produce/client.log').read_text(encoding='utf-8');plan=[1,5,7] if a.feature=='pair-counts' else [10,3] if a.feature=='pair-ten' else [13]
observed=None
if partial or crash:
 m=re.search(r'PG_PAIR_INTERRUPT_READY products=(\d+) fine_observed=(\d+) raw_remaining=(\d+) wool_remaining=(\d+)',log);assert m
 observed=int(m[1]);assert 0<observed<13 and int(m[2])==130344311+observed*200 and int(m[3])==int(m[4])==13-observed
else:assert 'PG_CASE_PASS native pair batch kind='+a.feature in log
durable=sum(r[3] for r in after['inventory'] if r[2]==1692)
assert durable==(0 if crash else 13) if not partial else observed<=durable<13
expected=[r.copy() for r in initial['inventory']];actions=[];batches=[]
for batch,count in enumerate(plan):
 ws=sorted(r[1] for r in expected if r[2]==1759);rs=sorted(r[1] for r in expected if r[2]==1673);w=ws.pop();r=rs.pop();pairs=[(w,r),*zip(ws,rs)]
 batches.append({'batch':batch,'selected_wool':w,'selected_raw':r,'requested':count})
 for step,(w,r) in enumerate(pairs[:count]):
  expected=[x for x in expected if x[1] not in [w,r]];free=next(s for s in range(28) if not any(x[1]==s for x in expected));expected.append([93,free,1692,1,0]);expected.sort();actions.append({'batch':batch,'step':step,'wool':w,'raw':r,'output_slot':free})
  if partial and len(actions)==durable:break
 if partial:break
if crash:expected=initial['inventory']
assert after['inventory']==post['inventory']==expected and load('relogin/before.json')==after
assert len(initial['stats'])==len(after['stats'])==len(post['stats'])
for x,y,z in zip(initial['stats'],after['stats'],post['stats']):
 assert x[0]==y[0] and y==z
 if x[0]==12:assert x==[12,99,99,130344311] and y==[12,99,99,130344311+durable*200]
 else:assert x==y
assert initial['character'][3]==after['character'][3]==post['character'][3]
for phase in ['produce','relogin']:
 m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS' and m['client_exit']==0 and m['resource_hashes_unchanged'];assert load(phase+'/after.json')['integrity_check']=='ok'
 if crash and phase=='produce':assert m['server_exit']!=0 and m['intentional_crash_injected'] and 'owned disposable' in m['fault_scope'] and not m['save_acknowledgement'] and not m['graceful_shutdown'];assert 'PG_EXPECTED_CRASH_DISCONNECT' in log
 else:assert m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and not m['forced_shutdown']
wire=[(int(op),bytes.fromhex(h.replace(',',''))) for op,h in re.findall(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log)]
uses=[b for op,b in wire if op==27];makes=[b for op,b in wire if op==115];assert len(uses)==len(makes)==len(plan)
beadd=lambda b:(b[0]<<8)|((b[1]-128)&255);leadd=lambda b:((b[0]-128)&255)|(b[1]<<8)
for (i,b),m,batch in zip(enumerate(uses),makes,batches):
 assert m==bytes.fromhex('0e010f00')+plan[i].to_bytes(2,'big')
 pair=(batch['selected_wool'],1759,batch['selected_raw'],1673) if i%2==0 else (batch['selected_raw'],1673,batch['selected_wool'],1759)
 assert len(b)==16 and (beadd(b[4:6]),beadd(b[10:12]),int.from_bytes(b[8:10],'little'),leadd(b[6:8]))==pair
traces=[json.loads(s.split(': ',1)[1]) for s in (OUT/'produce/server.log').read_text().splitlines() if s.startswith('Assembly transition: ')]
if not crash:
 assert len(traces)==durable
 for t,v in zip(traces,actions):
  assert (t['first_slot'],t['second_slot'],t['first'],t['second'],t['output'],t['count'])==(v['wool'],v['raw'],1759,1673,1692,1)
  assert t['committed_at']-t['queued_at']==(1 if v['step']==0 else 2)
 if not partial:assert log.count('PG_REPLAY label="native stale completed pair batch quantity"')==len(plan) and log.count('PG_REPLAY label="native stale consumed pair batch request"')==len(plan)
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
report={'implementation_id':'OR-MIG-0069','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','feature':a.feature,'requested_counts':plan,'observed_partial_products':observed,'durable_products':durable,'fine_xp_award':durable*200,'actions':actions if not crash else [],'traces':traces,'scope':'Genuine captured two-input native production with independent complete inventory/Stats/energy, selected-first/ascending pairs, first-free outputs, fineXP and real restarted raw22/46state. '+('Owned unacknowledged termination restores prior whole state; no mid-SQL fault claimed.' if crash else 'Consistent acknowledged partial prefix persists without queue replay.' if partial else 'Builtin quantities, first1/repeat2server clocks and stale completed quantity/held refusal pass.'),'limitations':['Gold stringing batch scope; other profile/timing/member/modifier cases remain separately identified.','No custom input, rendered/audio or original executable parity claim.'],'evidence_sha256':{p:hashlib.sha256((OUT/p).read_bytes()).hexdigest() for p in ['produce/client.log','produce/server.log','produce/manifest.json','relogin/client.log','relogin/manifest.json']}}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
