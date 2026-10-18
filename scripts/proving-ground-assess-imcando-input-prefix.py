"""Preserve five successful native refusals and defensive malformed operation disconnect."""
import json,re,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='7d4e09b6fdad766312ef55238e61c2730ea0b529';OUT=ROOT/'docs/openrune/verification/independent/deep-imcando-inputs/native-001';load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
i=load('initial-state.json');after=load('produce/after.json');m=load('produce/manifest.json');build=load('build.json')
assert m['tested_sha']==m['implementation_sha']==build['tested_sha']==SHA and build['exit_code']==0 and m['server_sha256']==build['binary_sha256'] and m['client_sha256']==load('resources.json')['driver_sha256']
assert m['status']=='FAIL' and m['client_exit']==1 and m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown']
assert after['integrity_check']=='ok' and after['inventory']==i['inventory'] and after['stats']==i['stats'] and after['character'][3]==i['character'][3] and after['account_membership']==i['account_membership']==True
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert [int(x)for x in re.findall(r'PG_IMCANDO_REFUSAL index=(\d+)',log)]==list(range(5));assert 'unexpected native disconnect' in log
replays=re.findall(r'PG_REPLAY label="([^"]+)" tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',log);assert len(replays)==6
base=(149<<16).to_bytes(4,'big')+(17).to_bytes(2,'big')+(25644).to_bytes(2,'big')+bytes([4]);expected=[]
for j in range(6):
 b=bytearray(base)
 if j==0:b[:4]=(0xffff0000).to_bytes(4,'big')
 if j==1:b[:4]=((12<<16)|13).to_bytes(4,'big')
 if j==2:b[6:8]=(29775).to_bytes(2,'big')
 if j==3:b[4:6]=(28).to_bytes(2,'big')
 if j==4:b[4:6]=(65535).to_bytes(2,'big')
 if j==5:b[8]=0
 expected.append(bytes(b))
for row,b in zip(replays,expected):assert int(row[2])==47 and bytes.fromhex(row[3].replace(',',''))==b
source=ROOT.parent/'agent7-targets'/SHA/'crates/server/src/gameplay.rs';text=source.read_text();assert 'if !(1..=10).contains(&button.op)' in text and 'Unhandled interface operation' in text
def packets(text):
 for tick,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',text):yield int(tick),int(op),bytes.fromhex(h.replace(',',''))
def full(b):
 size=int.from_bytes(b[6:8],'big');pos=8;items=[]
 for slot in range(size):
  count=(128-b[pos])&255;pos+=1
  if count==255:c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
  item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2;items.append((item,count))
 assert pos==len(b);return items

current=None;bag=[(x[2],x[3])for x in i['inventory']if x[0]==93]
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
 assert next==bag;current=next
assert current==bag
report={'implementation_id':'OR-MIG-0108','tested_sha':SHA,'status':'PASS_SCOPED','refusals':5,'malformed_disconnects':1,'raw_campaign_status':'FAIL','reason':'QA observer expected connection continuity after out-of-range operation0. Pinned decoder explicitly rejects values outside1..10; actual owned native connection closes, whole raw inventory remains unchanged and acknowledged disposable SQL save conserves inventory/gear/Stats/fractions/membership/energy. Five earlier wrong/closed component, wrong explicit form and slot28/65535 refusals passed independently; no implementation defect alleged. Remaining in-range wrong operations/broken/UseCancel/followup move to a fresh run,successful prefix not repeated.','target_decoder_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'evidence_sha256':{p:hashlib.sha256((OUT/p).read_bytes()).hexdigest()for p in ['initial-state.json','build.json','resources.json','produce/client.log','produce/manifest.json','produce/after.json','frozen-observer/pg_observer.rs','frozen-observer/build.json']}}
p=OUT/'independent-assessment.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS scoped5 native refusals and malformed0disconnect; raw QA failure preserved')
