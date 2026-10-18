import builtins
"""Independent native worn hammer menu, complete container and durable restart oracle."""
import argparse,json,re,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='8bc76c7c464daf1d4abdef4dd7f4172c270abda5';p=argparse.ArgumentParser();p.add_argument('--feature',choices=['imcando-worn','imcando-worn-f2p','imcando-worn-full'],required=True);a=p.parse_args();f2p=a.feature=='imcando-worn-f2p';fullbag=a.feature=='imcando-worn-full';OUT=ROOT/'docs/openrune/verification/independent'/('deep-'+a.feature)/'native-001';load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
i=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');build=load('build.json');assert build['tested_sha']==SHA and build['exit_code']==0
for phase in ['produce','relogin']:
 m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS' and m['server_sha256']==build['binary_sha256'] and m['client_sha256']==load('resources.json')['driver_sha256'] and m['client_exit']==m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown']
for d in [after,post]:assert d['integrity_check']=='ok' and d['stats']==i['stats'] and d['character'][3]==i['character'][3] and d['account_membership']==i['account_membership']==(not f2p)
assert load('relogin/before.json')==after
image=lambda data,inv,size:[builtins.next(((x[2],x[3])for x in data['inventory']if x[:2]==[inv,s]),(-1,0))for s in range(size)]
bag=image(i,93,28);gear=image(i,94,14);assert gear[3]==(25644,1) and gear[5]==(29775,1);assert sum(x[0]<0 for x in bag)==(0 if fullbag else 2 if f2p else 1)
plans=[(3,25644,29775,5),(5,29775,25644,3)]
if not f2p and not fullbag:plans+=plans[:]
steps=[];autos=[]
for source,input,output,target in plans:
 assert gear[source]==(input,1);bag=bag[:];gear=gear[:];auto=not f2p and gear[target][0]<0;autos.append(auto)
 if not fullbag:
  gear[source]=(-1,0)
  if auto:gear[target]=(output,1)
  else:bag[builtins.next(s for s,v in enumerate(bag)if v[0]<0)]=(output,1)
 steps.append({93:bag,94:gear})
expected=[[inv,s,id,count,0]for inv,items in [(93,bag),(94,gear)]for s,(id,count)in enumerate(items)if id>=0];assert after['inventory']==post['inventory']==expected
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert f'PG_CASE_PASS native worn hammer {a.feature} complete whole bag gear Stats' in log
account=re.search(r'PG_WORN_ACCOUNT member=(true|false) world_flags=(-?\d+) kind=(\S+)',log);assert account and account[1]==('false'if f2p else'true') and account[3]==a.feature
rows=re.findall(r'PG_WORN_SWAP index=(\d+) from=(\d+) input=(\d+) output=(\d+) to=(\d+) auto=(true|false) tick=(\d+)',log)
if fullbag:assert not rows and len(re.findall('PG_WORN_FULL_REFUSAL',log))==2
else:
 assert len(rows)==len(plans)
 for j,(row,plan)in enumerate(zip(rows,plans)):assert tuple(map(int,row[:5]))==(j,*plan) and (row[5]=='true')==autos[j]
 assert log.count('PG_REPLAY label="worn original component after source hand emptied"')==len(plans)
def packets(text):
 for tick,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',text):yield int(tick),int(op),bytes.fromhex(h.replace(',',''))
def full(b):
 size=int.from_bytes(b[6:8],'big');pos=8;items=[]
 for slot in range(size):
  count=(128-b[pos])&255;pos+=1
  if count==255:c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
  item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2;items.append((item,count))
 assert pos==len(b);return items

current={};step=-1;before=None;want=None;buttons=[];markers=0
for line in log.splitlines():
 tx=re.search(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',line)
 if tx:
  op=int(tx[1]);b=bytes.fromhex(tx[2].replace(',',''));assert op not in [27,115]
  if op==47 and int.from_bytes(b[:4],'big')in [(387<<16)|18,(387<<16)|20]:
   step+=1;assert step<len(plans);source,input,output,target=plans[step];assert int.from_bytes(b[:4],'big')==(387<<16)|(18 if source==3 else 20);assert b[4:6]==b'\xff\xff' and int.from_bytes(b[6:8],'big')in [65535,input] and b[8]==2;buttons.append(b.hex());before={inv:items[:]for inv,items in current.items()};want=steps[step]
 for tick,op,b in packets(line):
  if op not in [18,22]:continue
  inv=int.from_bytes(b[4:6],'big')
  if inv not in [93,94]:continue
  if op==22:
   next=full(b);next +=[(-1,0)]*((28 if inv==93 else 14)-len(next))
  else:
   assert inv in current;next=current[inv][:];pos=6
   while pos<len(b):
    slot=b[pos];pos+=1
    if slot>=128:slot=((slot<<8)|b[pos])-32768;pos+=1
    item=int.from_bytes(b[pos:pos+2],'big')-1;pos+=2;count=0
    if item>=0:
     count=b[pos];pos+=1
     if count==255:count=int.from_bytes(b[pos:pos+4],'big');pos+=4
    next[slot]=(item,count)
   assert pos==len(b)
  if step<0:assert next==image(i,inv,28 if inv==93 else 14)
  else:
   for slot,value in enumerate(next):assert value in [before[inv][slot],want[inv][slot]],(step,inv,slot,value)
  current[inv]=next
 if line.startswith('PG_WORN_SWAP ')or line.startswith('PG_WORN_FULL_REFUSAL '):assert current==steps[step];markers+=1;before={inv:items[:]for inv,items in current.items()}
assert len(buttons)==len(plans)==markers and current==steps[-1]
relog=(OUT/'relogin/client.log').read_text(encoding='utf-8');assert 'PG_NATIVE_NO_PENDING_SKILL_MULTI' in relog;invs={};stats={}
for tick,op,b in packets(relog):
 if op==46:stats[(b[2]-128)&255]=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'))
 if op==22:invs[int.from_bytes(b[4:6],'big')]=full(b)
for inv in [93,94]:assert [[inv,s,item,count,0]for s,(item,count)in enumerate(invs[inv])if item>=0]==[x for x in expected if x[0]==inv]
for skill,current,base,fine in post['stats']:assert stats[skill]==(current,fine//10)
report={'implementation_id':'OR-MIG-0109','tested_sha':SHA,'status':'PASS','feature':a.feature,'account_membership':not f2p,'observed_world_flags':int(account[2]),'native_actions':len(plans),'scope':'Actual native Worn Equipment tab and mapped387:18/20 Swap/button2/current nullable item and slot-1. Independent raw22/18 decoding conserves every bag/gear slot, permits only the exact expected hand conversion/removal/newoutput, and matches each action marker; full Stats/fractions/energy/duplicateheldforms/helmet/body/coins unchanged. Separate acknowledged SQL save and real process restart/native raw22/46 reconnect match the complete final account, no pending/replay. '+('Full28bag refuses both directions without gear movement.'if fullbag else'Both worn forms convert under observed real membership policy; original source-hand packet replay after its removal makes no change.'),'limitations':['No rendered/audio/original executable parity or movingHEADacceptance.','Static worn packets can carry nullable item fields; refusal after an emptied source hand does not imply all old nullable packets are invalid forever after a later re-equip.','Separate fresh unsaved recovery and explicit claimed-ID/wrong slot/component scenarios remain independent requirements.'],'evidence_sha256':{p:hashlib.sha256((OUT/p).read_bytes()).hexdigest()for p in ['initial-state.json','resources.json','build.json','produce/client.log','produce/manifest.json','produce/after.json','relogin/client.log','relogin/manifest.json','relogin/after.json','frozen-observer/pg_observer.rs','frozen-observer/build.json']}}
p=OUT/'independent-assessment.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS native worn hammer whole-state save and actual restart '+a.feature)
