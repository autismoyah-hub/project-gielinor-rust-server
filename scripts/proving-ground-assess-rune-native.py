"""Independent native altar packets and acknowledged same-account persistence."""
import argparse,hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
SHA='a2fa7421bfd8f6cdee59e4e1cca82f2db2b2406d'
p=argparse.ArgumentParser();p.add_argument('--feature',required=True);p.add_argument('--run-id',required=True);a=p.parse_args()
cancel=a.feature.endswith('-close')
key=a.feature.removeprefix('rune-').removesuffix('-f2p').removesuffix('-close')
id,output,interval,xp,x,z={'air':(34760,556,11,50,2843,4833),'mind':(34761,558,14,55,2785,4840),'water':(34762,555,19,60,2715,4835),'earth':(34763,557,26,65,2657,4840),'fire':(34764,554,35,70,2584,4837),'body':(34765,559,46,75,2522,4839)}[key]
OUT=ROOT/'docs/openrune/verification/independent'/('deep-'+a.feature)/a.run_id
load=lambda n:json.loads((OUT/n).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json')
count=28*(99//interval+1);fine=130344311+28*xp*5
expected=[[93,0,output,count,0]]+[r for r in initial['inventory']if r[0]!=93]
assert after['inventory']==post['inventory']==expected
stats=[r[:]for r in initial['stats']];stats[20][3]=fine
assert after['stats']==post['stats']==stats
assert load('relogin/before.json')==after
for state in[initial,after,post]:
 assert state['integrity_check']=='ok' and state['xp_rate_hundreds']==500
 assert state['account_membership']==(not a.feature.endswith('-f2p'))
 assert state['character'][3]==10000
build=load('build.json');assert build['tested_sha']==SHA and build['exit_code']==0
for phase in['produce','relogin']:
 m=load(phase+'/manifest.json')
 assert m['status']=='PASS' and m['tested_sha']==m['implementation_sha']==SHA
 assert m['server_sha256']==build['binary_sha256'] and m['client_exit']==m['server_exit']==0
 assert m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown']
 assert '--spawn'in m['server_arguments'] and m['server_arguments'][-1]==','.join(map(str,initial['character'][:3]))
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
log=(OUT/'produce/client.log').read_text(encoding='utf-8')
assert log.count('PG_RUNE_MENU action=')==2 and log.count('PG_RUNE_RESULT action=')==(1 if cancel else 2)
if cancel:
 assert 'PG_CASE_PASS native Rune Close cancelled before first award then fresh actual altar click succeeds'in log
 assert log.count('PG_RUNE_CANCELLED genuine scene Craft-rune/native Close before first flush sixclock no inventory gear XP change')==1
else:assert f'PG_CASE_PASS native ordinary altar {a.feature}'in log
assert f'sequence=791 delay=0'in log and 'decoded_slot_id_height_delay=[(0, 186, 100, 0)]'in log
menus=re.findall(r'PG_RUNE_MENU action=(\d+).*native_action=Object \{ id: (\d+), tile: \[(\d+), (\d+)\], operation: (\d+), sub: (\d+) \}',log)
assert [tuple(map(int,r))for r in menus]==[(n,id,x,z,1,0)for n in range(2)]
wire=[(int(op),bytes.fromhex(h.replace(',','')))for op,h in re.findall(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log)]
generated=[bytes.fromhex(h.replace(',',''))for h in re.findall(r'PG_TX label="ordinary actual scene Craft-rune menu" tick=\d+ opcode=73 payload=\[(.*?)\]',log)]
assert len(generated)==2 and generated[0]==generated[1] and not any(op in[27,115]for op,b in wire)
location_sends=[b for op,b in wire if op==73]
assert not location_sends or location_sends==generated
if cancel:assert [op for op,b in wire if op in[73,95]]==[73,95,73]
for b in generated:
 # Revision239 OpLoc1 Z is a big-endian u16 with Add128 low byte.
 assert len(b)==8 and ((b[0]<<8)|((b[1]-128)&255),int.from_bytes(b[2:4],'little'),(b[4]-128)&255,(b[5]-128)&255,int.from_bytes(b[6:8],'big'))==(z,x,0,0,id)
current=None;changes=[]
for tick,op,b in packets(log):
 if op not in[18,22]or int.from_bytes(b[4:6],'big')!=93:continue
 if op==22:nxt=full(b)
 else:
  assert current is not None;nxt=current[:];pos=6
  while pos<len(b):
   slot=b[pos];pos+=1
   if slot>=128:slot=((slot<<8)|b[pos])-32768;pos+=1
   item=int.from_bytes(b[pos:pos+2],'big')-1;pos+=2;quantity=0
   if item>=0:
    quantity=b[pos];pos+=1
    if quantity==255:quantity=int.from_bytes(b[pos:pos+4],'big');pos+=4
   nxt[slot]=(item,quantity)
  assert pos==len(b)
 if current is None:assert nxt==[(r[2],r[3])for r in initial['inventory']if r[0]==93]
 elif nxt!=current:changes.append(tick)
 current=nxt
assert len(changes)==1 and current==[(output,count)]+[(-1,0)]*27
relog=(OUT/'relogin/client.log').read_text(encoding='utf-8');assert 'PG_NATIVE_NO_PENDING_SKILL_MULTI'in relog
invs={};observed_stats={}
for tick,op,b in packets(relog):
 if op==22:invs[int.from_bytes(b[4:6],'big')]=full(b)
 if op==46:observed_stats[(b[2]-128)&255]=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'))
for inv in[93,94]:assert [[inv,s,item,n,0]for s,(item,n)in enumerate(invs[inv])if item>=0]==[r for r in expected if r[0]==inv]
for skill,current,base,f in stats:assert observed_stats[skill]==(current,f//10)
names=['initial-state.json','build.json','produce/client.log','produce/manifest.json','produce/after.json','relogin/client.log','relogin/manifest.json','relogin/after.json','frozen-observer/pg_observer.rs','frozen-observer/build.json']
report={'implementation_id':'OR-MIG-0125','tested_sha':SHA,'status':'PASS_SCOPED','feature':a.feature,'output':output,'runes':count,'fine_xp_award':28*xp*5,'inventory_change_ticks':changes,'scope':'Two actual rendered native altar Craft-rune menus generate matching native location actions; one complete28 mixed essence transaction, exact runes/wholeStats/fractionalXP/gear and no additional credit after empty repeat. Actual791/186-height100 decoded effects. Acknowledged save, server process restart and independently decoded same-account native reconnect preserve whole inventory/Stats.','limitations':['Current transport logger records27/47/95/115 only, not73. First altar delivery established by genuine server reward/effects; generated second action and subsequent no-credit observation do not independently attest its wire delivery or server refusal.','Inside-altar disposable fixture does not establish acquisition/entry/ruins/talisman/tiara access.','No original executable/audio/render parity, unsaved crash, native interruption/negative completion or other altar acceptance. Historical stock defect PG-DEF-007 remains.'],'evidence_sha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest()for n in names}}
report['observed_location_wire_sends']=len(location_sends)
if location_sends:
 report['scope']+=' Both genuine location sends independently logged at native transport boundary and decoded to exact altar/coordinates/Op1; second empty click delivery and no repeated credit attested.'
 report['limitations']=report['limitations'][1:]
if cancel:
 second_send=int(re.findall(r'PG_WIRE_TX tick=(\d+) opcode=73',log)[1])
 assert changes[0]>second_send
 report['scope']='Two actual rendered native altar Craft-rune menus. First genuine location73 followed by native Close95 before the first network poll preserves full28essence/gear/wholeStats for six received clocks; independently logged73/95/73 order and raw inventory confirms no transition before second fresh action. Fresh second click commits224Mindrunes/7700fineXP with791/186effects and whole gear/Stats retained. Acknowledged save,process restart and independently decoded native reconnect preserve exact224runes and fractionalXP. This tests cancellation before first completion, not a guaranteed already-running weak-queue window.'
 report['limitations']=['Inside-altar disposable fixture does not establish acquisition/entry/access.','Other native interruption/late-context scopes remain separate; no during-SQL-commit injection or original executable/audio/render parity. Historical PG-DEF-007 remains.']
path=OUT/'independent-assessment.json';assert not path.exists();path.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS_SCOPED',a.feature,count,'runes',fine,'fineXP saved/restarted')
