"""Read-only independent replay of native light inventory packets and menu fields."""
import json,re,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-light-sources/native-002';initial=json.loads((OUT/'initial-state.json').read_text(encoding='utf-8'));log=(OUT/'produce/client.log').read_text(encoding='utf-8')
rows=[(36,33),(38,32),(596,594),(4529,4531),(4532,4534),(4522,4524),(4537,4539),(4548,4550),(5014,5013)];steps=[];state=[(x[2],x[3])for x in initial['inventory']]
for i,(raw,lit)in enumerate(rows):
 for slot,item in[(4+2*i,lit),(4+2*i,raw),(5+2*i,lit)]:state=state[:];state[slot]=(item,1);steps.append(state)
buttons=[]
for op,h in re.findall(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log):
 b=bytes.fromhex(h.replace(',',''))
 if int(op)==47 and int.from_bytes(b[:4],'big')==149<<16:buttons.append((int.from_bytes(b[4:6],'big'),int.from_bytes(b[6:8],'big'),b[8]))
assert buttons==[(4+2*i,lit,op)for i,((_,lit),op)in enumerate(zip(rows,[2,2,6,3,3,3,3,3,4]))]
current=None;seen=[]
for tick,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',log):
 b=bytes.fromhex(h.replace(',',''));op=int(op)
 if op not in[18,22]or int.from_bytes(b[4:6],'big')!=93:continue
 if op==22:
  size=int.from_bytes(b[6:8],'big');pos=8;next=[]
  for _ in range(size):
   count=(128-b[pos])&255;pos+=1
   if count==255:c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
   item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2;next.append((item,count))
 else:
  assert current is not None;next=current[:];pos=6
  while pos<len(b):
   slot=b[pos];pos+=1
   if slot>=128:slot=((slot<<8)|b[pos])-32768;pos+=1
   item=int.from_bytes(b[pos:pos+2],'big')-1;pos+=2;count=0
   if item>=0:
    count=b[pos];pos+=1
    if count==255:count=int.from_bytes(b[pos:pos+4],'big');pos+=4
   next[slot]=(item,count)
 assert pos==len(b)
 if current is None:assert next==[(x[2],x[3])for x in initial['inventory']]
 elif next!=current:assert len(seen)<27 and next==steps[len(seen)],(tick,len(seen));seen.append(int(tick))
 current=next
assert len(seen)==27 and current==steps[-1]
report={'implementation_id':'OR-MIG-0084','tested_sha':'ad739676dd01f5be809cdfe5ecf4f1e6ea77238a','status':'PASS','scope':'Independent decoder reconstructs every received inventory93full22/partial18packet: exactly27complete selected-slot lighting/extinguish transitions in intended order; stale replays introduce no other state changes. Nine genuine IF_BUTTON47frames contain inventory149:0/exact lit ID/clicked slot and target buttons2,2,6,3,3,3,3,3,4. Whole inventory conserved at every received change, not only final SQL. Receive clocks are asynchronous observations, not original/rendered latency.','credit_receive_ticks':seen,'buttons':buttons,'evidence_sha256':{'produce/client.log':hashlib.sha256((OUT/'produce/client.log').read_bytes()).hexdigest(),'initial-state.json':hashlib.sha256((OUT/'initial-state.json').read_bytes()).hexdigest()}}
p=OUT/'packet-assessment.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS every raw22/18inventory change:27whole transitions;9exact Extinguish buttons')
