"""Independent pinned native special-essence priority, whole packets and same-save restart."""
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='bd1294ab9e0ff850155b024364dc3a3a98a37bed';OUT=ROOT/'docs/openrune/verification/independent/deep-rune-mind-daeyalt/native-001'
load=lambda n:json.loads((OUT/n).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');build=load('build.json')
assert build['tested_sha']==SHA and build['exit_code']==0
base=[(24704 if s<7 else 1436 if s%2==0 else 7936,1)for s in range(28)]
assert [(r[2],r[3])for r in initial['inventory']if r[0]==93]==base
first=base[:]
for s in range(7):first[s]=(-1,0)
first[0]=(558,56);final=[(558,224)]+[(-1,0)]*27
expected=[[93,0,558,224,0]]+[r for r in initial['inventory']if r[0]!=93]
assert after['inventory']==post['inventory']==expected
stats=[r[:]for r in initial['stats']];assert stats[20]==[20,99,99,130344311];stats[20][3]=130346475
assert after['stats']==post['stats']==stats and load('relogin/before.json')==after
for state in[initial,after,post]:assert state['integrity_check']=='ok' and state['xp_rate_hundreds']==125 and state['account_membership']is True and state['character'][3]==10000
for phase in['produce','relogin']:
 m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['server_sha256']==build['binary_sha256'] and m['status']=='PASS' and m['client_exit']==m['server_exit']==0
 assert m['save_acknowledgement'] and m['graceful_shutdown'] and not m['forced_shutdown'] and m['resource_hashes_unchanged']
 assert m['server_arguments'][-2:]==['--spawn','2784,4841,0']
def packets(log):
 for t,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',log):yield int(t),int(op),bytes.fromhex(h.replace(',',''))
def full(b):
 size=int.from_bytes(b[6:8],'big');pos=8;items=[]
 for slot in range(size):
  count=(128-b[pos])&255;pos+=1
  if count==255:
   c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
  item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2;items.append((item,count))
 assert pos==len(b);return items
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'PG_DAEYALT_ACCOUNT member=true' in log and 'rate=125' in log
assert 'PG_CASE_PASS native Daeyalt first7 retain21ordinary secondclick emptythird' in log
rows=re.findall(r'PG_DAEYALT_RESULT action=(\d+) output=(\d+) count=(\d+) fine=(\d+) tick=(\d+)',log)
assert [tuple(map(int,row[:4]))for row in rows]==[(0,558,56,130345032),(1,558,224,130346475),(2,558,224,130346475)]
menus=re.findall(r'PG_DAEYALT_MENU action=(\d+).*native_action=Object \{ id: (\d+), tile: \[(\d+), (\d+)\], operation: (\d+), sub: (\d+) \}',log)
assert [tuple(map(int,row))for row in menus]==[(n,34761,2785,4840,1,0)for n in range(3)]
sends=[]
for op,h in re.findall(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log):
 op=int(op);b=bytes.fromhex(h.replace(',',''));assert op not in[27,115]
 if op==73:sends.append(b)
assert len(sends)==3
for b in sends:assert len(b)==8 and ((b[0]<<8)|((b[1]-128)&255),int.from_bytes(b[2:4],'little'),b[4],b[5],int.from_bytes(b[6:8],'big'))==(4840,2785,128,128,34761)
current=None;changes=[];producer_stats={};xp_history=[]
for tick,op,b in packets(log):
 if op==46:
  skill=(b[2]-128)&255;value=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'));producer_stats[skill]=value
  if skill==20 and(not xp_history or xp_history[-1]!=value[1]):xp_history.append(value[1])
 if op not in[18,22]or int.from_bytes(b[4:6],'big')!=93:continue
 if op==22:nxt=full(b)
 else:
  assert current is not None;nxt=current[:];pos=6
  while pos<len(b):
   slot=b[pos];pos+=1
   if slot>=128:slot=((slot<<8)|b[pos])-32768;pos+=1
   item=int.from_bytes(b[pos:pos+2],'big')-1;pos+=2;count=0
   if item>=0:
    count=b[pos];pos+=1
    if count==255:count=int.from_bytes(b[pos:pos+4],'big');pos+=4
   nxt[slot]=(item,count)
  assert pos==len(b)
 if current is None:assert nxt==base
 elif nxt!=current:assert nxt==[first,final][len(changes)];changes.append(tick)
 current=nxt
assert len(changes)==2 and current==final and xp_history==[13034431,13034503,13034647]
for skill,cur,baselevel,fine in stats:assert producer_stats[skill]==(cur,fine//10)
assert log.count('sequence=791 delay=0')>=2 and log.count('decoded_slot_id_height_delay=[(0, 186, 100, 0)]')>=2
relog=(OUT/'relogin/client.log').read_text(encoding='utf-8');assert 'PG_NATIVE_NO_PENDING_SKILL_MULTI' in relog;invs={};observed_stats={}
for tick,op,b in packets(relog):
 if op==22:invs[int.from_bytes(b[4:6],'big')]=full(b)
 if op==46:observed_stats[(b[2]-128)&255]=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'))
for inv in[93,94]:assert [[inv,s,item,n,0]for s,(item,n)in enumerate(invs[inv])if item>=0]==[row for row in expected if row[0]==inv]
for skill,cur,baselevel,fine in stats:assert observed_stats[skill]==(cur,fine//10)
names=['initial-state.json','build.json','resources.json','produce/client.log','produce/manifest.json','produce/after.json','relogin/client.log','relogin/manifest.json','relogin/after.json','frozen-observer/pg_observer.rs','frozen-observer/build.json']
report={'implementation_id':'OR-MIG-0126','tested_sha':SHA,'status':'PASS_SCOPED','actions':3,'runes':224,'fine_xp_awards':[721,1443],'inventory_change_ticks':changes,'scope':'Three actual rendered native Mind Craft-rune menus and independent raw73senddecode:7Daeyalt first yields56Mindrunes and retains21ordinary essence;second click converts retained21to168more,third empty delivered click adds nothing. Exact aggregate721then1443fineXP at personal125percent,whole gear/allStats and791/186effects. Acknowledged save,real server restart and independently decoded same-account raw22/46 native reconnect preserve224runes and130346475fineXP including final halfXP.','limitations':['Inside-altar disposable source inventory does not establish essence acquisition/shard trading/bank/pouch/ruins/tiara access.','This member positive/normal durability subcampaign does not accept all6special essence native cases,F2P refusals,interruption,crash,other negative or original executable/audio/render parity.'],'evidence_sha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest()for n in names}}
p=OUT/'independent-assessment.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS_SCOPED native Daeyalt priority/224runes/2164fineXP/same-save restart')
