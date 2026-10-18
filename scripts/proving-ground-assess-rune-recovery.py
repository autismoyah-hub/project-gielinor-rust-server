"""Independently decode an unsaved native Rune altar batch and its actual restart."""
import json,re,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='a2fa7421bfd8f6cdee59e4e1cca82f2db2b2406d';OUT=ROOT/'docs/openrune/verification/independent/deep-rune-mind-crash/native-001'
load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');m=load('produce/manifest.json');r=load('relogin/manifest.json');build=load('build.json')
assert build['tested_sha']==SHA and build['exit_code']==0
for x in [m,r]:
 assert x['tested_sha']==x['implementation_sha']==SHA and x['server_sha256']==build['binary_sha256'] and x['client_sha256']==load('resources.json')['driver_sha256'] and x['status']=='PASS' and x['client_exit']==0 and x['resource_hashes_unchanged']
assert m['intentional_crash_injected'] and m['server_exit']!=0 and not m['save_acknowledgement'] and not m['graceful_shutdown'] and '224Mindrunes/7700fineXP' in m['fault_scope']
assert r['server_exit']==0 and r['save_acknowledgement'] and r['graceful_shutdown'] and not r['forced_shutdown']
for state in [after,post]:
 assert state['integrity_check']=='ok' and state['inventory']==initial['inventory'] and state['stats']==initial['stats'] and state['account_membership']==initial['account_membership']==True and state['character'][3]==initial['character'][3]
assert load('relogin/before.json')==after
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'PG_CRASH_READY one complete unsaved ordinary Rune altar batch observed' in log and 'PG_EXPECTED_CRASH_DISCONNECT' in log
assert re.findall(r'PG_RUNE_RESULT action=(\d+) altar=(\d+) output=(\d+) count=(\d+) fine=(\d+) tick=(\d+)',log)[0][:5]==('0','34761','558','224','130352011')
sends=[]
for op,h in re.findall(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log):
 b=bytes.fromhex(h.replace(',',''));op=int(op);assert op not in[27,115]
 if op==73:sends.append(b)
assert len(sends)==1;b=sends[0];assert len(b)==8 and ((b[0]<<8)|((b[1]-128)&255),int.from_bytes(b[2:4],'little'),b[4],b[5],int.from_bytes(b[6:8],'big'))==(4840,2785,128,128,34761)

def packets(text):
 for tick,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',text):yield int(tick),int(op),bytes.fromhex(h.replace(',',''))
def full(b):
 size=int.from_bytes(b[6:8],'big');pos=8;items=[]
 for slot in range(size):
  count=(128-b[pos])&255;pos+=1
  if count==255:c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
  item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2;items.append((item,count))
 assert pos==len(b);return items
current=None;changes=[];base=[(x[2],x[3])for x in initial['inventory']if x[0]==93];expected=[(558,224)]+[(-1,0)]*27
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
producer_stats={}
for tick,op,b in packets(log):
 if op==46:producer_stats[(b[2]-128)&255]=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'))
for skill,current,base,fine in initial['stats']:assert producer_stats[skill]==(current,130352011//10 if skill==20 else fine//10)
assert 'sequence=791 delay=0'in log and 'decoded_slot_id_height_delay=[(0, 186, 100, 0)]'in log

relog=(OUT/'relogin/client.log').read_text(encoding='utf-8');assert 'PG_NATIVE_NO_PENDING_SKILL_MULTI' in relog;invs={};stats={}
for tick,op,b in packets(relog):
 if op==46:stats[(b[2]-128)&255]=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'))
 if op==22:invs[int.from_bytes(b[4:6],'big')]=full(b)
for inv in [93,94]:assert [[inv,s,item,count,0]for s,(item,count)in enumerate(invs[inv])if item>=0]==[x for x in initial['inventory']if x[0]==inv]
for skill,current,base,fine in initial['stats']:assert stats[skill]==(current,fine//10)
report={'implementation_id':'OR-MIG-0125','tested_sha':SHA,'status':'PASS','scope':'One genuine rendered native Mind altar Craft-rune menu converts28mixed essence into224runes and7700fineXP with791/186effects; exact location73 send and whole raw22/18/46 independently decoded. Only this freshly owned disposable local server is terminated before save/logout. SQL prior28essence/worn gear/Stats/fractions/membership/energy restored; actual restarted native raw22/46 matches exactly with no pending credit or duplication.','limitations':['Unacknowledged rollback does not inject during a SQL commit.','Separate completed member/nonmember normal campaigns are not rerun.'],'evidence_sha256':{p:hashlib.sha256((OUT/p).read_bytes()).hexdigest()for p in ['initial-state.json','build.json','produce/client.log','produce/manifest.json','produce/after.json','relogin/client.log','relogin/manifest.json','relogin/after.json','frozen-observer/pg_observer.rs','frozen-observer/build.json']}}
p=OUT/'independent-assessment.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS native unsaved Rune altar rollback and real restart')
