"""Assess full bag, stale Eat and actual potion-overheal food case."""
import ast,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-food-negative/negative-003'
def load(p):return json.loads((OUT/p).read_text(encoding='utf-8'))
SHA='ae3d234c71764dd2364f8452fc1974a54a6783f0';initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json')
assert len(initial['inventory'])==28
expected=[x.copy() for x in initial['inventory']]
for x in expected:
    if x[1]==3:x[2]=1895
    if x[1]==9:x[2]=6687
assert after['inventory']==post['inventory']==expected
assert load('relogin/before.json')==after
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS'
    assert m['client_exit']==m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and not m['forced_shutdown'] and m['resource_hashes_unchanged']
    assert load(phase+'/after.json')['integrity_check']=='ok'
for a,b,c in zip(initial['stats'],after['stats'],post['stats']):assert a[0]==b[0]==c[0] and a[2:]==b[2:]==c[2:]
assert next(s[1] for s in after['stats'] if s[0]==3)==13
log=(OUT/'produce/client.log').read_text(encoding='utf-8')
assert 'PG_FOOD_OVERHEAL before=13 after=13 base=10' in log and 'PG_CASE_PASS full28' in log
assert 'stale whole cake while matching duplicate remains' in log
traces=[json.loads(s.split('Food transition: ',1)[1]) for s in (OUT/'produce/server.log').read_text(encoding='utf-8').splitlines() if s.startswith('Food transition: ')]
assert len(traces)==2
for t,(before,item,hp) in zip(traces,[(1891,1893,10),(1893,1895,13)]):
    assert t['slot']==3 and t['before']==[before,1,0] and t['after']==[item,1,0]
    assert t['hp_before']==t['hp_after']==hp and t['fine_xp_before']==t['fine_xp_after']==11540 and t['configured_delay']==2
relog=(OUT/'relogin/client.log').read_text(encoding='utf-8');assert 'PG_CASE_PASS native relogin' in relog
observed=[ast.literal_eval(s.split(' inventory=',1)[1]) for s in relog.splitlines() if s.startswith('PG_APPLIED') and ' inventory=' in s]
assert [[93,s,i,n,0] for s,i,n in observed[-1] if i>=0]==expected
initial_stats={}
for tick,op,hexes in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',relog):
    if int(op)!=46:continue
    p=bytes.fromhex(hexes.replace(',',''));skill=(p[2]-128)&255
    initial_stats.setdefault(skill,(p[1],int.from_bytes(p[5:7]+p[3:5],'big')))
for skill,current,base,fine in after['stats']:assert initial_stats[skill]==(current,fine//10)
r={'implementation_id':'OR-MIG-0024','tested_sha':SHA,'campaign_kind':'historical','status':'PASS',
 'scope':'Native full28 selected cake replacement, replayed stale whole-cake claim with untouched whole-cake duplicate, actual Saradomin brew HP10->13, delayed legal Eat preserving HP13 and no XP. Whole bag remains occupied. Acknowledged save, actual restart and native relogin preserve selected partial cake, brew dose and all other slots; received initial Stats match durable pre-restart values.',
 'negative_checks':['Stale claimed item cannot consume the partial stage or another whole cake','Full bag does not relocate or lose remainder','Ordinary food neither lowers nor increases existing overheal','All fineXP/base levels unchanged'],
 'limitations':['This uses six received clocks after brew; it does not establish earliest legal potion/food cadence.','Normal acknowledged restart is not process-crash atomicity.','Any later passive stat decay during relogin is distinct from initial durable packet validation.','Full V2/V3 remains partial pending other member/interruption/presentation/clock cases.']}
(OUT/'independent-assessment.json').write_text(json.dumps(r,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(r))
