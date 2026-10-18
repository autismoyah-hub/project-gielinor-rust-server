"""Independent native held hammer menu, whole packet/SQL and same-account restart."""
import argparse,hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='7d4e09b6fdad766312ef55238e61c2730ea0b529';a=argparse.ArgumentParser();a.add_argument('--feature',choices=['imcando-held','imcando-f2p'],required=True);args=a.parse_args();f2p=args.feature=='imcando-f2p';OUT=ROOT/'docs/openrune/verification/independent'/('deep-'+args.feature)/'native-001';load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');assert initial['account_membership']==after['account_membership']==post['account_membership']==(not f2p)
expected=[x[:]for x in initial['inventory']];backpack=[(x[2],x[3])for x in expected if x[0]==93];assert len(backpack)==28 and backpack[3]==(25633,1) and backpack[27]==(995,1000)
plans=[(17,25644,29775),(9,29775,25644),(17,29775,25644),(9,25644,29775),(4,25644,29775),(26,29775,25644)];steps=[]
for slot,input,output in plans:assert backpack[slot]==(input,1);backpack=backpack[:];backpack[slot]=(output,1);steps.append(backpack)
for row in expected:
 if row[0]==93:row[2],row[3]=backpack[row[1]]
assert after['inventory']==post['inventory']==expected and load('relogin/before.json')==after
assert after['stats']==post['stats']==initial['stats'];assert after['character'][3]==post['character'][3]==initial['character'][3]
assert [x for x in expected if x[0]==94]==[[94,0,1155,1,0],[94,4,1129,1,0]]
build=load('build.json');assert build['tested_sha']==SHA and build['exit_code']==0
for phase in ['produce','relogin']:
 m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['server_sha256']==build['binary_sha256'] and m['status']=='PASS' and m['client_exit']==m['server_exit']==0
 assert m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown'];assert load(phase+'/after.json')['integrity_check']=='ok'
log=(OUT/'produce/client.log').read_text(encoding='utf-8');flag='false'if f2p else'true';assert 'PG_CASE_PASS native six held Imcando swaps both forms exact full28 broken/duplicate/gear/Stats retained member='+flag in log
account=re.search(r'PG_IMCANDO_ACCOUNT member=(true|false) world_flags=(-?\d+)',log);assert account and account[1]==flag;world_flags=int(account[2]);rows=re.findall(r'PG_IMCANDO_SWAP index=(\d+) slot=(\d+) input=(\d+) output=(\d+) member=(true|false) tick=(\d+)',log);assert len(rows)==6
for i,(row,plan)in enumerate(zip(rows,plans)):assert tuple(map(int,row[:4]))==(i,*plan) and row[4]==flag
assert log.count('PG_REPLAY label="stale explicit held hammer form with matching duplicate retained"')==6
buttons=[]
for op,h in re.findall(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log):
 b=bytes.fromhex(h.replace(',',''));op=int(op);assert op not in[27,115]
 if op==47 and int.from_bytes(b[:4],'big')==149<<16:assert len(b)==9;buttons.append((int.from_bytes(b[4:6],'big'),int.from_bytes(b[6:8],'big'),b[8]))
assert buttons==[(slot,input,4)for slot,input,output in plans]
def packets(text):
 for tick,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',text):yield int(tick),int(op),bytes.fromhex(h.replace(',',''))
def full(b):
 size=int.from_bytes(b[6:8],'big');pos=8;items=[]
 for slot in range(size):
  count=(128-b[pos])&255;pos+=1
  if count==255:c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
  item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2;items.append((item,count))
 assert pos==len(b);return items
current=None;seen=[]
for tick,op,b in packets(log):
 if op not in[18,22]or int.from_bytes(b[4:6],'big')!=93:continue
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
 if current is None:assert next==[(x[2],x[3])for x in initial['inventory']if x[0]==93]
 elif next!=current:assert len(seen)<6 and next==steps[len(seen)],(tick,len(seen));seen.append(tick)
 current=next
assert len(seen)==6 and current==steps[-1]
relog=(OUT/'relogin/client.log').read_text(encoding='utf-8');assert 'PG_NATIVE_NO_PENDING_SKILL_MULTI' in relog;invs={};stats={}
for tick,op,b in packets(relog):
 if op==46:stats[(b[2]-128)&255]=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'))
 if op==22:invs[int.from_bytes(b[4:6],'big')]=full(b)
for inv in[93,94]:assert [[inv,slot,item,count,0]for slot,(item,count)in enumerate(invs[inv])if item>=0]==[x for x in expected if x[0]==inv]
for skill,current,base,fine in post['stats']:assert stats[skill]==(current,fine//10)
report={'implementation_id':'OR-MIG-0108','tested_sha':SHA,'status':'PASS','feature':args.feature,'account_membership':not f2p,'observed_world_flags':world_flags,'actions':6,'stale_refusals':6,'scope':'Six authentic native held Swap menus/twoforms/two roundtrips and final converted locations, each exact149:0/button4/currentitem/slot raw47. Independent raw22/18decode verifies every whole inventory transition and no stale fallback. Broken item offers noSwap; full28retains duplicate forms/coins/gear/wholeStats with zeroXP/quantity/Use traffic. Acknowledged save and actual process restart/raw22/46 native reconnect preserve exact final hammer locations, worngear and fractionalStats with no replay. Real decoded native account membership flag matches disposable SQL account; world_flags recorded literally.','limitations':['This verifies the pinned Rust client/cache/account mode, not original OSRS F2P world-effective-cache parity. No local member flag/cache mutation performed.','No worn109/acquisition/repair/render/audio/original executable or movingHEAD acceptance.','Separate fresh unsaved recovery remains required for fullV3.'],'evidence_sha256':{name:hashlib.sha256((OUT/name).read_bytes()).hexdigest()for name in ['initial-state.json','build.json','produce/client.log','produce/manifest.json','produce/after.json','relogin/client.log','relogin/manifest.json','relogin/after.json','frozen-observer/pg_observer.rs','frozen-observer/build.json']}}
p=OUT/'independent-assessment.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS native held hammer sixSwap/sixstale/whole gear+Stats/native restart '+args.feature)
