"""Independently decode an unsaved native hammer conversion and its actual restart."""
import json,re,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='7d4e09b6fdad766312ef55238e61c2730ea0b529';OUT=ROOT/'docs/openrune/verification/independent/deep-imcando-crash/native-001'
load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');m=load('produce/manifest.json');r=load('relogin/manifest.json');build=load('build.json')
assert build['tested_sha']==SHA and build['exit_code']==0
for x in [m,r]:
 assert x['tested_sha']==x['implementation_sha']==SHA and x['server_sha256']==build['binary_sha256'] and x['client_sha256']==load('resources.json')['driver_sha256'] and x['status']=='PASS' and x['client_exit']==0 and x['resource_hashes_unchanged']
assert m['intentional_crash_injected'] and m['server_exit']!=0 and not m['save_acknowledgement'] and not m['graceful_shutdown'] and 'unsaved held hammer Swap' in m['fault_scope']
assert r['server_exit']==0 and r['save_acknowledgement'] and r['graceful_shutdown'] and not r['forced_shutdown']
for state in [after,post]:
 assert state['integrity_check']=='ok' and state['inventory']==initial['inventory'] and state['stats']==initial['stats'] and state['account_membership']==initial['account_membership']==True and state['character'][3]==initial['character'][3]
assert load('relogin/before.json')==after
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'PG_CRASH_READY one unsaved native held hammer Swap' in log and 'PG_EXPECTED_CRASH_DISCONNECT' in log
assert re.findall(r'PG_IMCANDO_SWAP index=(\d+) slot=(\d+) input=(\d+) output=(\d+) member=(true|false) tick=(\d+)',log)[0][:5]==('0','17','25644','29775','true')
buttons=[]
for op,h in re.findall(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log):
 b=bytes.fromhex(h.replace(',',''));op=int(op);assert op not in [27,115]
 if op==47 and int.from_bytes(b[:4],'big')==149<<16:buttons.append((int.from_bytes(b[4:6],'big'),int.from_bytes(b[6:8],'big'),b[8]))
assert buttons==[(17,25644,4)]
def packets(text):
 for tick,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',text):yield int(tick),int(op),bytes.fromhex(h.replace(',',''))
def full(b):
 size=int.from_bytes(b[6:8],'big');pos=8;items=[]
 for slot in range(size):
  count=(128-b[pos])&255;pos+=1
  if count==255:c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
  item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2;items.append((item,count))
 assert pos==len(b);return items
current=None;changes=[];base=[(x[2],x[3])for x in initial['inventory']if x[0]==93];expected=base[:];expected[17]=(29775,1)
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
 if current is None:assert next==base
 elif next!=current:assert next==expected and not changes;changes.append(tick)
 current=next
assert len(changes)==1 and current==expected
relog=(OUT/'relogin/client.log').read_text(encoding='utf-8');assert 'PG_NATIVE_NO_PENDING_SKILL_MULTI' in relog;invs={};stats={}
for tick,op,b in packets(relog):
 if op==46:stats[(b[2]-128)&255]=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'))
 if op==22:invs[int.from_bytes(b[4:6],'big')]=full(b)
for inv in [93,94]:assert [[inv,s,item,count,0]for s,(item,count)in enumerate(invs[inv])if item>=0]==[x for x in initial['inventory']if x[0]==inv]
for skill,current,base,fine in initial['stats']:assert stats[skill]==(current,fine//10)
report={'implementation_id':'OR-MIG-0108','tested_sha':SHA,'status':'PASS','scope':'One authentic native inventory Swap at selected17 converts25644to29775, independently decoded whole raw22/18. Only this freshly owned disposable local server is terminated before save/logout. SQL prior full inventory/worn gear/Stats/fractions/membership/energy restored; actual restarted native raw22/46 matches exactly, no pending replay or duplicate.','limitations':['Unacknowledged rollback does not inject during a SQL commit.','Separate completed member/nonmember normal campaigns are not rerun.'],'evidence_sha256':{p:hashlib.sha256((OUT/p).read_bytes()).hexdigest()for p in ['initial-state.json','build.json','produce/client.log','produce/manifest.json','produce/after.json','relogin/client.log','relogin/manifest.json','relogin/after.json','frozen-observer/pg_observer.rs','frozen-observer/build.json']}}
p=OUT/'independent-assessment.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS native unsaved hammer rollback and real restart')
