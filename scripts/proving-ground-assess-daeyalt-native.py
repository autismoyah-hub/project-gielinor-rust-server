"""Preserved native500 first batch and same-account continuation;125 remains blocked."""
import hashlib,json,re,sqlite3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='bd1294ab9e0ff850155b024364dc3a3a98a37bed';OUT=ROOT/'docs/openrune/verification/independent/deep-rune-mind-daeyalt/native-001';CONT=OUT/'continuation-001'
def load(folder,n):return json.loads((folder/n).read_text(encoding='utf-8'))
initial=load(OUT,'initial-state.json');first_sql=load(OUT,'produce/after.json');final_sql=load(CONT,'produce/after.json');post=load(CONT,'relogin/after.json');proof=load(CONT,'continuation-source.json');blocked=load(OUT,'rate125-blocker.json')
assert blocked['status']=='BLOCKED' and blocked['tested_sha']==SHA
base=[(24704 if s<7 else 1436 if s%2==0 else 7936,1)for s in range(28)];first=base[:]
for s in range(7):first[s]=(-1,0)
first[0]=(558,56);final=[(558,224)]+[(-1,0)]*27
assert [(v[2],v[3])for v in initial['inventory']if v[0]==93]==base
first_expected=[[93,s,item,n,0]for s,(item,n)in enumerate(first)if item>=0]+[v for v in initial['inventory']if v[0]!=93]
final_expected=[[93,0,558,224,0]]+[v for v in initial['inventory']if v[0]!=93]
stats1=[v[:]for v in initial['stats']];assert stats1[20]==[20,99,99,130344311];stats1[20][3]=130347198
stats2=[v[:]for v in stats1];stats2[20][3]=130352973
assert first_sql['inventory']==first_expected and first_sql['stats']==stats1
assert final_sql['inventory']==post['inventory']==final_expected and final_sql['stats']==post['stats']==stats2
assert initial['xp_rate_hundreds']==125
for state in[first_sql,load(CONT,'initial-state.json'),final_sql,post]:assert state['xp_rate_hundreds']==500 and state['account_membership']is True and state['integrity_check']=='ok' and state['character'][3]==10000
for key in['inventory','stats','xp_rate_hundreds','account_membership']:assert load(CONT,'initial-state.json')[key]==first_sql[key]
assert load(CONT,'produce/before.json')==load(CONT,'initial-state.json') and load(CONT,'relogin/before.json')==final_sql
original=load(OUT,'produce/manifest.json');assert original['status']=='FAIL' and original['client_exit']==1
for folder,phase in[(OUT,'produce'),(CONT,'produce'),(CONT,'relogin')]:
 m=load(folder,phase+'/manifest.json');b=load(folder,'build.json');assert m['tested_sha']==m['implementation_sha']==b['tested_sha']==SHA and b['exit_code']==0
 assert m['server_sha256']==original['server_sha256']==b['binary_sha256'] and m['state_path']==original['state_path']
 assert m['save_acknowledgement'] and m['graceful_shutdown'] and m['server_exit']==0 and not m['forced_shutdown'] and m['resource_hashes_unchanged']
 if folder==CONT:assert m['status']=='PASS' and m['client_exit']==0
assert load(CONT,'produce/manifest.json')['database_sha256_before']==original['database_sha256_after']==proof['original_database_sha256_before_continuation']
backup=Path(proof['private_snapshot']['path']);assert hashlib.sha256(backup.read_bytes()).hexdigest()==proof['private_snapshot']['sha256']
db=sqlite3.connect(backup.as_uri()+'?mode=ro',uri=True);cid=db.execute('SELECT id FROM characters LIMIT 1').fetchone()[0]
assert [list(v)for v in db.execute('SELECT i.inv_type,o.slot,o.obj,o.count,o.vars FROM inventory_objs o JOIN inventories i ON i.id=o.inventories_id WHERE i.character_id=? ORDER BY i.inv_type,o.slot',(cid,))]==first_expected
assert [list(v)for v in db.execute('SELECT stat_id,vis_level,base_level,fine_xp FROM stats WHERE character_id=? ORDER BY stat_id',(cid,))]==stats1
assert db.execute('PRAGMA integrity_check').fetchone()[0]=='ok';db.close()
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

def decode_image(log,base,expected_changes,expected_xp):
 current=None;changes=[];stats={};history=[];sends=[]
 for op,h in re.findall(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log):
  op=int(op);b=bytes.fromhex(h.replace(',',''));assert op not in[27,115]
  if op==73:sends.append(b)
 for b in sends:assert len(b)==8 and((b[0]<<8)|((b[1]-128)&255),int.from_bytes(b[2:4],'little'),b[4],b[5],int.from_bytes(b[6:8],'big'))==(4840,2785,128,128,34761)
 for tick,op,b in packets(log):
  if op==46:
   skill=(b[2]-128)&255;value=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'));stats[skill]=value
   if skill==20 and(not history or history[-1]!=value[1]):history.append(value[1])
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
  elif nxt!=current:assert nxt==expected_changes[len(changes)];changes.append(tick)
  current=nxt
 assert len(changes)==len(expected_changes) and current==expected_changes[-1] and history==expected_xp
 return changes,stats,len(sends)
log1=(OUT/'produce/client.log').read_text(encoding='utf-8');log2=(CONT/'produce/client.log').read_text(encoding='utf-8')
assert 'observer deadline'in log1 and log1.count('PG_DAEYALT_MENU action=')==1
assert 'PG_DAEYALT_RESUME_INITIAL original56runes21ordinary received member=true'in log2 and 'effective_rate=500'in log2
assert 'PG_CASE_PASS native same-account Daeyalt continuation'in log2
c1,s1,n1=decode_image(log1,base,[first],[13034431,13034719]);c2,s2,n2=decode_image(log2,first,[final],[13034719,13035297]);assert(n1,n2)==(1,2)
for skill,cur,baselevel,fine in stats1:assert s1[skill]==(cur,fine//10)
for skill,cur,baselevel,fine in stats2:assert s2[skill]==(cur,fine//10)
rows=re.findall(r'PG_DAEYALT_RESULT action=(\d+) output=(\d+) count=(\d+) fine=(\d+) tick=(\d+)',log2);assert[tuple(map(int,row[:4]))for row in rows]==[(1,558,224,130352973),(2,558,224,130352973)]
for text in[log1,log2]:assert 'sequence=791 delay=0'in text and 'decoded_slot_id_height_delay=[(0, 186, 100, 0)]'in text
relog=(CONT/'relogin/client.log').read_text(encoding='utf-8');assert 'PG_NATIVE_NO_PENDING_SKILL_MULTI'in relog;invs={};observed={}
for tick,op,b in packets(relog):
 if op==22:invs[int.from_bytes(b[4:6],'big')]=full(b)
 if op==46:observed[(b[2]-128)&255]=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'))
for inv in[93,94]:assert [[inv,s,item,n,0]for s,(item,n)in enumerate(invs[inv])if item>=0]==[v for v in final_expected if v[0]==inv]
for skill,cur,baselevel,fine in stats2:assert observed[skill]==(cur,fine//10)
names=['initial-state.json','build.json','produce/client.log','produce/manifest.json','produce/after.json','rate125-blocker.json','frozen-observer/pg_observer.rs','frozen-observer/build.json','continuation-001/continuation-source.json','continuation-001/initial-state.json','continuation-001/produce/client.log','continuation-001/produce/manifest.json','continuation-001/produce/after.json','continuation-001/relogin/client.log','continuation-001/relogin/manifest.json','continuation-001/relogin/after.json','continuation-001/frozen-observer/pg_observer.rs','continuation-001/frozen-observer/build.json']
report={'implementation_id':'OR-MIG-0126','tested_sha':SHA,'status':'PASS_SCOPED','effective_native_rate':500,'rate125_native_variant':{'status':'BLOCKED','evidence':'rate125-blocker.json','reason':'Pinned unconditional FloorOne login rewrites125to500;no native125acceptance or server/policy substitution.'},'raw_first_status':'FAIL','first_failure_scope':'Controllerexpected125andtimedout;independent raw73/22/18/46andSQLestablish only actual500first batch. Printed125wasintendedfixturemetadata,not observed effective rate.','actions':3,'runes':224,'fine_xp_awards':[2887,5775],'inventory_change_ticks':[c1,c2],'scope':'At exact126SHA,actualnativeMindCraft-rune sends:7Daeyaltfirst->56runes/+2887fineXP retaining21ordinary. Acknowledgedfirstsave and privateSQLsnapshot preserved. Actual sameaccount/database processrestart loads56runes/exactfraction;continuationconsumesonlyremaining21ordinary->168more/+5775fineXP,thirdemptydeliveredclickaddsnone. No seed/first-actionreplay. Final acknowledgedsave/restartednative raw22/46preserve224runes/130352973fineXP(.3),wholegear/allStats.791/186effects independently observed inboth producingphases.','limitations':['Native125ratevariantremainsBLOCKED;only World/API125coverage exists.500isexplicitnewscopeunder same pinnedserver,not a replacement acceptance.','FirstrawFAILretained;whole originaldriver completion isnotclaimed. Same-account tail uses a separately frozen observer and preserves the private precontinuationSQLite snapshot.','Insidealtar acquiredinventoryfixture,nopouch/acquisition/shard/bank/ruinsaccess proof. Other5specialessencealtars/F2Prefusals/nativeinterruptions/crash/originalexecutable/audio/renderparity remain open.'],'evidence_sha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest()for n in names}}
p=OUT/'independent-assessment.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS_SCOPED native500Daeyalt first+same-accounttail224runes8662fineXP;125variantBLOCKED')
