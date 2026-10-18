"""Independent native hammer invalid-input and real-menu follow-up assessment."""
import json,re,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='7d4e09b6fdad766312ef55238e61c2730ea0b529';OUT=ROOT/'docs/openrune/verification/independent/deep-imcando-inputs/native-002';load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');m=load('produce/manifest.json');build=load('build.json')
assert m['tested_sha']==m['implementation_sha']==build['tested_sha']==SHA and build['exit_code']==0 and m['server_sha256']==build['binary_sha256'] and m['client_sha256']==load('resources.json')['driver_sha256']
assert m['status']=='PASS' and m['client_exit']==m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown']
expected=[x[:]for x in initial['inventory']]
for row in expected:
 if row[:2]==[93,17]:assert row[2:]==[25644,1,0];row[2]=29775
assert after['integrity_check']=='ok' and after['inventory']==expected and after['stats']==initial['stats'] and after['character'][3]==initial['character'][3] and after['account_membership']==initial['account_membership']==True
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'PG_CASE_PASS native hammer four remaining invalid inputs UseCancel and legitimate followup' in log
refusals=re.findall(r'PG_IMCANDO_REFUSAL index=(\d+) label="([^"]+)" tick=(\d+)',log);assert [int(x[0])for x in refusals]==list(range(5,9));last_refusal=int(refusals[-1][2])
base=(149<<16).to_bytes(4,'big')+(17).to_bytes(2,'big')+(25644).to_bytes(2,'big')+bytes([4]);expected_inputs=[]
for i in range(5,9):
 b=bytearray(base)
 if i==0:b[:4]=(0xffff0000).to_bytes(4,'big')
 if i==1:b[:4]=((12<<16)|13).to_bytes(4,'big')
 if i==2:b[6:8]=(29775).to_bytes(2,'big')
 if i==3:b[4:6]=(28).to_bytes(2,'big')
 if i==4:b[4:6]=(65535).to_bytes(2,'big')
 if i in [5,6,7]:b[8]=[6,1,10][i-5]
 if i==8:b[4:6]=(3).to_bytes(2,'big');b[6:8]=(25633).to_bytes(2,'big')
 expected_inputs.append(bytes(b))
replays=re.findall(r'PG_REPLAY label="([^"]+)" tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',log);assert len(replays)==4
for row,b in zip(replays,expected_inputs):assert int(row[2])==47 and bytes.fromhex(row[3].replace(',',''))==b
actual=[]
for op,h in re.findall(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log):
 op=int(op);b=bytes.fromhex(h.replace(',',''));assert op not in [27,115]
 if op==47 and int.from_bytes(b[:4],'big')==149<<16:actual.append(b)
assert actual==[base]
def packets(text):
 for tick,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',text):yield int(tick),int(op),bytes.fromhex(h.replace(',',''))
def full(b):
 size=int.from_bytes(b[6:8],'big');pos=8;items=[]
 for slot in range(size):
  count=(128-b[pos])&255;pos+=1
  if count==255:c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
  item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2;items.append((item,count))
 assert pos==len(b);return items

current=None;seen=[];bag=[(x[2],x[3])for x in initial['inventory']if x[0]==93];result=[(x[2],x[3])for x in expected if x[0]==93]
for tick,op,b in packets(log):
 if op not in [18,22]or int.from_bytes(b[4:6],'big')!=93:continue
 if op==22:next=full(b)
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
 if current is None:assert next==bag
 elif current!=next:assert not seen and tick>last_refusal and next==result;seen.append(tick)
 current=next
assert len(seen)==1 and current==result
report={'implementation_id':'OR-MIG-0108','tested_sha':SHA,'status':'PASS','scope':'Four remaining exact native negative frames: in-range wrong content operation6/1/10 and broken25633. Five completed earlier refusals and malformed0disconnect are independently recorded separately. Controller confirms whole bag/gear/XP after each; independent raw22/18 reconstruction confirms zero changes before final genuine menu Swap. Actual Use selection and Cancel emit no held-target27; final genuine149:0/current17/current25644/button4 produces one and only one whole inventory change. SQL acknowledged save retains exact gear/Stats/fractions/energy; no chemistry actions.','limitations':['Negative protocol frames are explicit test inputs on the owned local native connection, not user menu actions.','Closed-bank component acceptance does not imply closing the permanently attached normal inventory149; no nonexistent inventory-close flow claimed.','Separate immutable normal member/nonmember and unsaved recovery campaigns establish restart behavior.'],'evidence_sha256':{p:hashlib.sha256((OUT/p).read_bytes()).hexdigest()for p in ['initial-state.json','resources.json','build.json','produce/client.log','produce/manifest.json','produce/after.json','frozen-observer/pg_observer.rs','frozen-observer/build.json']}}
p=OUT/'independent-assessment.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS native hammer four remaining invalid inputs/UseCancel/legitimate followup')
