import builtins
"""Independently assess two unmodified ordinary native worn Swap actions."""
import json,re,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='8bc76c7c464daf1d4abdef4dd7f4172c270abda5';OUT=ROOT/'docs/openrune/verification/independent/deep-imcando-worn-dispatch/native-001';load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
i=load('initial-state.json');after=load('produce/after.json');m=load('produce/manifest.json');build=load('build.json')
assert m['tested_sha']==m['implementation_sha']==build['tested_sha']==SHA and build['exit_code']==0 and m['server_sha256']==build['binary_sha256'] and m['client_sha256']==load('resources.json')['driver_sha256']
assert m['status']=='FAIL' and m['client_exit']==1 and m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown']
assert after['integrity_check']=='ok' and after['inventory']==i['inventory'] and after['stats']==i['stats'] and after['character'][3]==i['character'][3] and after['account_membership']==i['account_membership']==True
assert len([x for x in i['inventory']if x[0]==93])==27;assert [x for x in i['inventory']if x[0]==94]==[[94,0,1155,1,0],[94,3,25644,1,0],[94,4,1129,1,0],[94,5,29775,1,0]]
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'PG_CASE_FAIL two ordinary native Swap menus transmitted but neither changes bag gear or XP' in log
assert re.findall(r'PG_WORN_DISPATCH_MISSING hand=(\d+) item=(\d+) actual_operation=(\d+)',log)==[('3','25644','4'),('5','29775','4')]
assert 'PG_REPLAY' not in log;buttons=[]
for op,h in re.findall(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log):
 op=int(op);b=bytes.fromhex(h.replace(',',''));assert op not in [27,115]
 if op==47 and int.from_bytes(b[:4],'big')in[(387<<16)|18,(387<<16)|20]:buttons.append(b)
assert buttons==[((387<<16)|child).to_bytes(4,'big')+b'\xff\xff\xff\xff\x04'for child in[18,20]]
def packets(text):
 for tick,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',text):yield int(tick),int(op),bytes.fromhex(h.replace(',',''))
def full(b):
 size=int.from_bytes(b[6:8],'big');pos=8;items=[]
 for slot in range(size):
  count=(128-b[pos])&255;pos+=1
  if count==255:c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
  item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2;items.append((item,count))
 assert pos==len(b);return items

images={inv:[builtins.next(((x[2],x[3])for x in i['inventory']if x[:2]==[inv,s]),(-1,0))for s in range(size)]for inv,size in [(93,28),(94,14)]};current={}
for tick,op,b in packets(log):
 if op==46:
  stat=(b[2]-128)&255;value=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'));row=builtins.next(x for x in i['stats']if x[0]==stat);assert value==(row[1],row[3]//10)
 if op not in[18,22]:continue
 inv=int.from_bytes(b[4:6],'big')
 if inv not in images:continue
 if op==22:next=full(b);next +=[(-1,0)]*(len(images[inv])-len(next))
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
 assert next==images[inv],(tick,inv);current[inv]=next
assert current==images
target=ROOT.parent/'agent7-targets'/SHA;runtime=target/'crates/server/src/runtime.rs';profile=target/'crates/server/src/content/imcando_worn_swaps.rs';text=runtime.read_text();assert 'else if button.op == 2 {' in text and 'self.world.worn_item_swaps.get(&item.id)' in text and 'button_operation: 2' in profile.read_text()
symbols=ROOT/'work/native-combat/resources/symbols/param.sym';assert re.search(r'^453\s+wear_op3$',symbols.read_text(),re.M)
report={'implementation_id':'OR-MIG-0109','tested_sha':SHA,'status':'CONFIRMED_IMPLEMENTATION_DEFECT','finding':'PG-DEF-006','severity':'P2','ordinary_native_actions':2,'scope':'Fresh member account has one free backpack slot and both ordinary worn hammers. Two authentic native Worn Equipment Swap menu actions transmit actual387:18/20,slot-1,item-1,operation4. After8ticks each, independently decoded whole bag/gear/rawStats and acknowledged SQL save remain exactly initial: no conversion. Pinned profile and runtime accept worn Swap under operation2; targetparam453iswear_op3, agreeing with real menu4. No hand-crafted requests or replay used.','correction':'Align exact target worn Swap registration/runtime dispatch with actual param453wear_op3/button4 and validate corresponding UI permissions; preserve typed current nullable claim/life guards/atomic staged equipment callbacks. Publish a correctionSHA/contract revision; then test actual menus both forms, occupied/freehand/fullbag,member-F2P,save/reconnect/owned recovery at that exact newSHA.','prior_attempt':'deep-imcando-worn/native-001 stopped at QA assertion expecting2before the Swap reached wire; preserve rawFAIL and distinguish it from this independently transmitted confirmation. Its ordinary menu already generated4.','blocked_followups':['Exact109native converted-form save/restart/crash cannot be reached through actual Swap at this SHA. No newer build substituted.','Prepared F2P/native-001 remains seeded/frozen but unexecuted; no member/F2P conversion inference.','This does not block unrelated crafting/tool-use/bank/rune-altar verification or require FORGE to stop.'],'target_source_sha256':{p.relative_to(target).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in[runtime,profile]},'evidence_sha256':{p:hashlib.sha256((OUT/p).read_bytes()).hexdigest()for p in ['initial-state.json','resources.json','build.json','produce/client.log','produce/manifest.json','produce/after.json','frozen-observer/pg_observer.rs','frozen-observer/build.json']}}
p=OUT/'independent-assessment.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('CONFIRMED PG-DEF-006 two ordinary native worn Swap actions generate4; server bound2')
