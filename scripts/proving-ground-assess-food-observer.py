"""Decode actual native observer packets and correlate food/regen independently."""
import ast,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-food-observer/observer-001'
SHA='ae3d234c71764dd2364f8452fc1974a54a6783f0'
def load(p):return json.loads((OUT/p).read_text(encoding='utf-8'))
def packets(log):
    for line in log.splitlines():
        m=re.fullmatch(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*)\]',line)
        if m:yield int(m[1]),int(m[2]),bytes.fromhex(m[3].replace(',',''))
def inventory(p):
    inv=int.from_bytes(p[4:6],'big');capacity=int.from_bytes(p[6:8],'big');pos=8;items=[]
    for slot in range(capacity):
        count=(128-p[pos])&255;pos+=1
        if count==255:
            b=p[pos:pos+4];count=int.from_bytes(bytes([b[1],b[0],b[3],b[2]]),'big');pos+=4
        item=int.from_bytes(p[pos:pos+2],'little')-1;pos+=2
        if item>=0:items.append([inv,slot,item,count,0])
        else:assert count==0
    assert pos==len(p);return inv,items
def stat(p):
    assert len(p)==7;return (p[2]-128)&255,{'current':p[1],'display_xp':int.from_bytes(p[5:7]+p[3:5],'big')}
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json')
expected=[[93,1,1891,1,0],[93,4,2289,1,0],[93,27,995,1000,0]]
assert after['inventory']==post['inventory']==expected
assert load('relogin/before.json')==after and after['stats']==post['stats']
for a,b in zip(initial['stats'],after['stats']):
    assert a[0]==b[0] and a[2:]==b[2:]
    assert b[1]==(10 if a[0]==3 else a[1])
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS'
    assert m['client_exit']==m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and not m['forced_shutdown'] and m['resource_hashes_unchanged']
    assert load(phase+'/after.json')['integrity_check']=='ok'
log=(OUT/'produce/client.log').read_text(encoding='utf-8')
actions=[tuple(map(int,m)) for m in re.findall(r'PG_FOOD_BEGIN index=(\d+) tick=(\d+) slot=(\d+) id=(\d+) hp=(\d+) xp=(\d+)',log)]
assert [(a[0],a[2],a[3]) for a in actions]==[(0,7,2142),(1,3,1891),(2,3,1893),(3,3,1895),(4,5,2289),(5,5,2291)]
assert len(re.findall('PG_TX label=Eat index=',log))==6 and len(re.findall('PG_FOOD_COMMITTED index=',log))==6
assert all(a[5]==1154 for a in actions) and 'PG_CASE_PASS food six actual' in log
hp=[];inv_updates=[]
for tick,op,p in packets(log):
    if op==46:
        skill,s=stat(p)
        if skill==3:assert s['display_xp']==1154;hp.append({'observed_tick':tick,**s})
    if op==22:
        inv,items=inventory(p)
        if inv==93:inv_updates.append(items)
assert [h['current'] for h in hp]==[2,3,6,10],hp
assert hp[1]['observed_tick']<actions[0][1] and actions[0][4]==3 and actions[1][4]==6
applied=[ast.literal_eval(s.split(' inventory=',1)[1]) for s in log.splitlines() if s.startswith('PG_APPLIED') and ' inventory=' in s]
assert [[93,slot,item,count,0] for slot,item,count in applied[-1] if item>=0]==expected
traces=[json.loads(s.split('Food transition: ',1)[1]) for s in (OUT/'produce/server.log').read_text(encoding='utf-8').splitlines() if s.startswith('Food transition: ')]
assert len(traces)==5
for t,(item,remainder,slot,delay) in zip(traces,[(1891,1893,3,2),(1893,1895,3,2),(1895,None,3,3),(2289,2291,5,1),(2291,None,5,2)]):
    assert t['slot']==slot and t['before']==[item,1,0] and t['after']==([remainder,1,0] if remainder else None)
    assert t['configured_delay']==delay and t['fine_xp_before']==t['fine_xp_after']==11540
    assert t['hp_after']==min(10,t['hp_before']+(4 if slot==3 else 7))
stats={};invs={}
for tick,op,p in packets((OUT/'relogin/client.log').read_text(encoding='utf-8')):
    if op==22:inv,items=inventory(p);invs[inv]=items
    if op==46:skill,s=stat(p);stats[skill]=s
assert invs[93]==expected
for skill,current,base,fine in post['stats']:assert stats[skill]=={'current':current,'display_xp':fine//10}
r={'implementation_id':'OR-MIG-0024','revision':3,'tested_sha':SHA,'campaign_kind':'historical','status':'PASS','native_actions':6,'received_hp_updates':hp,
 'scope':'All six genuine native meat/cake/pizza actions, including four formerly unexecuted bites, selected duplicates, remainder progression, HP cap and no XP; acknowledged save, real restart and independently decoded native relogin durable state PASS.',
 'observation_blocker':'PG-INFRA-004 resolved for continued testing: instrumented actual received HP2->3 before first Eat proves passive regeneration is independently observable; meat then3->6 and cake6->10. Inventory acknowledgements identify consumption independently of HP equality. The old campaign exact missed-observation sequence remains inferred because its driver did not capture packets.',
 'limitations':['Fresh run has a different regeneration phase from the retained failed driver; no fabricated reconstruction of its unrecorded packets.','This observer records raw full inventory22 and HP46, but omitted raw partial inventory18. Partial transitions are actual native post-apply observations plus authoritative SQL/trace; restarted full inventory22 is independently decoded.','No native full-bag, malformed/stale Eat, pre-existing overheal, interruption or crash-recovery verdict in this run.','Client packet clock can lag authoritative commit trace while VM/script backlog drains; correlate ordered updates/actions and exact trace identity, not equality of these two clock fields.','Scoped PASS does not certify all V2/V3 obligations.']}
(OUT/'independent-assessment.json').write_text(json.dumps(r,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(r))
