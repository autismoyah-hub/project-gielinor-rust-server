"""Assess exact selected gem outcomes and real durable native recovery."""
import json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-gems/cuts-5x001';SHA='5743cbe1c7499e2a7476c721d25dcba194666042'
def load(p):return json.loads((OUT/p).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json')
profiles=[(1623,1607,2500),(1621,1605,3375),(1619,1603,4250),(1617,1601,5375),(1631,1615,6875),(6571,6573,8375),(19496,19493,2500)]
expected=[x.copy() for x in initial['inventory']]
for i,(raw,cut,award) in enumerate(profiles):
    row=next(x for x in expected if x[0]==93 and x[1]==4+3*i);assert row[2]==raw;row[2]=cut
assert len(initial['inventory'])==len(after['inventory'])==28 and after['inventory']==post['inventory']==expected
assert load('relogin/before.json')==after and after['stats']==post['stats']
for a,b in zip(initial['stats'],after['stats']):assert a[:3]==b[:3] and b[3]-a[3]==(33250 if a[0]==12 else 0)
assert initial['xp_rate_hundreds']==after['xp_rate_hundreds']==post['xp_rate_hundreds']==500
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS'
    assert m['client_exit']==m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and not m['forced_shutdown'] and m['resource_hashes_unchanged']
    assert load(phase+'/after.json')['integrity_check']=='ok'
log=(OUT/'produce/client.log').read_text(encoding='utf-8')
events=re.findall(r'PG_GEM_COMMITTED index=(\d+) slot=(\d+) raw=(\d+) cut=(\d+) fine_award=(\d+) display_xp=(\d+) direction_reverse=(true|false)',log);assert len(events)==7
fine=130344311
for i,(event,(raw,cut,award)) in enumerate(zip(events,profiles)):
    fine+=award;assert tuple(map(int,event[:6]))==(i,4+3*i,raw,cut,award,fine//10) and event[6]==('true' if i%2 else 'false')
assert 'PG_CASE_PASS native gem pending action cancelled' in log
assert log.count('stale gem pair while matching raw and tool duplicates remain')==7
relog=(OUT/'relogin/client.log').read_text(encoding='utf-8');assert 'PG_CASE_PASS native relogin' in relog
invs={};stats={}
for tick,op,hexes in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',relog):
    p=bytes.fromhex(hexes.replace(',',''));op=int(op)
    if op==46:stats[(p[2]-128)&255]=(p[1],int.from_bytes(p[5:7]+p[3:5],'big'))
    if op==22:
        inv=int.from_bytes(p[4:6],'big');size=int.from_bytes(p[6:8],'big');pos=8;items=[]
        for slot in range(size):
            count=(128-p[pos])&255;pos+=1
            if count==255:b=p[pos:pos+4];count=int.from_bytes(bytes([b[1],b[0],b[3],b[2]]),'big');pos+=4
            item=int.from_bytes(p[pos:pos+2],'little')-1;pos+=2
            if item>=0:items.append([inv,slot,item,count,0])
        assert pos==len(p);invs[inv]=items
assert invs[93]==expected
for skill,current,base,fine_xp in post['stats']:assert stats[skill]==(current,fine_xp//10)
r={'implementation_id':'OR-MIG-0056','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','V0':'PASS','native_profiles':7,'fine_xp_award':33250,'effective_rate_hundreds':500,
 'scope':'All seven genuine native manual gem cuts, alternating selected direction, full28 bag, exact selected raw replaced, duplicate raw/chisels and all other slots retained, actual stale pair replay refused. Native pre-completion Walk cancels without consumption/XP. Acknowledged save, actual restart and independently decoded native full inventory/Stats confirm all seven outputs, retained tools and exact cumulative fractional XP; no repeat reward.',
 'native_125percent':'BLOCKED at this SHA by unconditional500percent FloorOne login policy; separate authoritative100/125/500matrix passed. Earlier cuts-001 oracle failure preserved.',
 'limitations':['Every profile in both native directions and decoded animation/first-cycle cadence require the separate effects campaign.','Acknowledged normal restart is not process-crash recovery; a separate bounded crash case is planned.','This scoped PASS does not alone promote all V1-V4 levels.']}
(OUT/'independent-assessment.json').write_text(json.dumps(r,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(r))
