"""Independent ordinary stringing transport, transaction and restart oracle."""
import argparse,json,re,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='fb348dda184b36ec2d503bfe0f1d5393cf8ae151'
p=argparse.ArgumentParser();p.add_argument('--feature',choices=['pair-stringing','pair-stringing-reverse'],default='pair-stringing');p.add_argument('--run-id',default='native-002');a=p.parse_args();flip=a.feature=='pair-stringing-reverse'
OUT=ROOT/'docs/openrune/verification/independent'/('deep-'+a.feature)/a.run_id
load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
profiles=[(1673,1692),(1675,1694),(1677,1696),(1679,1698),(1681,1700),(1683,1702),(6579,6581),(19501,19541),(21099,21108),(21102,21111),(21105,21114),(1714,1716),(1720,1722),(10134,10132)]
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json')
assert initial['inventory']==[[93,i,1759 if i<14 else profiles[i-14][0],1,0] for i in range(28)]
expected=[[93,i,out,1,0] for i,(_,out) in enumerate(profiles)]
assert after['inventory']==post['inventory']==expected and load('relogin/before.json')==after
assert len(initial['stats'])==len(after['stats'])==len(post['stats'])
for x,y,z in zip(initial['stats'],after['stats'],post['stats']):
 assert x[0]==y[0] and y==z
 if x[0]==12:assert x==[12,99,99,130344311] and y==[12,99,99,130347111]
 else:assert x==y
assert initial['character'][3]==after['character'][3]==post['character'][3]
for phase in ['produce','relogin']:
 m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown'];assert load(phase+'/after.json')['integrity_check']=='ok'
 if phase=='produce' and not flip:assert m['status']=='FAIL' and m['client_exit']==124
 else:assert m['status']=='PASS' and m['client_exit']==0
log=(OUT/'produce/client.log').read_text(encoding='utf-8')
if flip:assert 'PG_CASE_PASS native14stringing profiles' in log
else:
 assert len(re.findall(r'^PG_PAIR_STRINGING ',log,re.M))==13
 assert 'PG_CASE_PASS native14stringing profiles' not in log
wire=[(int(op),bytes.fromhex(h.replace(',',''))) for op,h in re.findall(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log)]
uses=[b for op,b in wire if op==27];makes=[b for op,b in wire if op==115];assert len(uses)==len(makes)==15
assert all(b==bytes.fromhex('0e010f000001') for b in makes)
beadd=lambda b:(b[0]<<8)|((b[1]-128)&255);leadd=lambda b:((b[0]-128)&255)|(b[1]<<8)
for n,b in enumerate(uses):
 i=max(0,n-1);raw=profiles[i][0];pair=(0,1759,14,1673) if n==0 else (i,1759,14+i,raw) if ((i%2==0)!=flip) else (14+i,raw,i,1759)
 assert len(b)==16 and (beadd(b[4:6]),beadd(b[10:12]),int.from_bytes(b[8:10],'little'),leadd(b[6:8]))==pair
assert log.count('PG_REPLAY label="native stale completed stringing quantity"')==14 and log.count('PG_REPLAY label="native stale consumed stringing pair"')==14 and log.count('PG_REPLAY label="native stale stringing quantity after Close"')==1
names=['Gold amulet','Sapphire amulet','Emerald amulet','Ruby amulet','Diamond amulet','Dragonstone amulet','Onyx amulet','Zenyte amulet','Opal amulet','Jade amulet','Topaz amulet','Unblessed symbol','Unpowered symbol','Strung rabbit foot']
labels=re.findall(r'PG_GLASS_MENU index=0 option="Make" target="(.*?)"',log);assert labels==['<col=ff9040>Gold amulet (level 1)</col>']+[f'<col=ff9040>{name} (level {37 if i==13 else 1})</col>' for i,name in enumerate(names)]
traces=[json.loads(s.split(': ',1)[1]) for s in (OUT/'produce/server.log').read_text().splitlines() if s.startswith('Assembly transition: ')]
assert len(traces)==14
for i,(t,(raw,out)) in enumerate(zip(traces,profiles)):
 assert (t['first_slot'],t['second_slot'],t['first'],t['second'],t['output'],t['count'])==(i,14+i,1759,raw,out,1)
 assert t['committed_at']-t['queued_at']==1
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
report={'implementation_id':'OR-MIG-0069','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','feature':a.feature,'opposite_direction_campaign':flip,'raw_producer_status':load('produce/manifest.json')['status'],'raw_producer_exit':load('produce/manifest.json')['client_exit'],'producer_timeout_scope':None if flip else 'Bounded390second driver expired after14observed/saved credits and14stale frame pairs, before final control assertion. Raw FAIL preserved; independent transport/server/SQL/restarted native state oracle establishes only this stated scope.','products':14,'fine_xp_award':2800,'scope':('Native14opposite-direction stringing profiles use genuine default Make-one' if flip else 'Native14ordinary stringing profiles alternate both held directions, real All-capped1 quantity dispatch')+' and selected two-input/first-free output transactions from full28. Genuine Close/stale quantity and held refusal,2800fineXP/all unrelated XP retained. Exact complete SQL state survives acknowledged-save/process restart and independently decoded native reconnect. All14server traces show first1clock, explicitly shorter than Wiki nominal2.','limitations':[('Opposite directions in this fresh campaign; earlier immutable forward/alternate campaign remains separate.' if flip else 'Producer raw FAIL/124 retained; independent positive/durable scope does not claim final driver completion. Each profile exercised in one direction; opposite directions and remaining quantity/lifecycle scopes remain separate.'),'No rendered/audio/original executable parity.'],'traces':traces,'evidence_sha256':{p:hashlib.sha256((OUT/p).read_bytes()).hexdigest() for p in ['produce/client.log','produce/server.log','produce/manifest.json','relogin/client.log','relogin/manifest.json','frozen-observer/pg_observer.rs','frozen-observer/build.json']}}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
