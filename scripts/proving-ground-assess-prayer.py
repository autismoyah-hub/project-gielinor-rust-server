"""Independent loaded-policy, native selected burial and restart assessment."""
import ast,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];BASE=ROOT/'docs/openrune/verification/independent';OUT=BASE/'deep-prayer/burial-001';SHA='f6b0f6ad54fe7813bb6607827eeb8da0e60137fa'
def load(p):return json.loads((OUT/p).read_text(encoding='utf-8'))
cache=[]
for path in BASE.glob('integration-external-prayer-cache-*/manifest.json'):
    m=json.loads(path.read_text(encoding='utf-8'))
    if m['status']=='PASS' and m['tested_integration_sha']==SHA:cache.append(path)
assert cache,'Independent loaded reward oracle required';oracle=cache[-1]
policies=[json.loads(s.split('PG_LOADED_BONE ',1)[1]) for s in (oracle.parent/'run-1.log').read_text(encoding='utf-8').splitlines() if s.startswith('PG_LOADED_BONE ')]
reward=next(x for x in policies if x['id']==526);assert reward['base_fine_xp']==45 and reward['personal_rate_hundreds']==500
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');assert len(initial['inventory'])==28
expected=[x for x in initial['inventory'] if x[1]!=17]
assert after['inventory']==post['inventory']==expected and load('relogin/before.json')==after
for a,b in zip(initial['stats'],after['stats']):
    assert a[:3]==b[:3] and b[3]-a[3]==(reward['expected_award_fine'] if a[0]==5 else 0)
assert after['stats']==post['stats'] and reward['expected_award_fine']==225
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS'
    assert m['client_exit']==m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and not m['forced_shutdown'] and m['resource_hashes_unchanged']
    assert load(phase+'/after.json')['integrity_check']=='ok'
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'PG_CASE_PASS native selected burial full28 slot17' in log and 'stale Bury slot17 with untouched matching bone slot3' in log
relog=(OUT/'relogin/client.log').read_text(encoding='utf-8');assert 'PG_CASE_PASS native relogin' in relog
observed=[ast.literal_eval(s.split(' inventory=',1)[1]) for s in relog.splitlines() if s.startswith('PG_APPLIED') and ' inventory=' in s]
assert [[93,s,i,n,0] for s,i,n in observed[-1] if i>=0]==expected
stats={}
for tick,op,hexes in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',relog):
    if int(op)!=46:continue
    p=bytes.fromhex(hexes.replace(',',''));stats[(p[2]-128)&255]=(p[1],int.from_bytes(p[5:7]+p[3:5],'big'))
for skill,current,base,fine in post['stats']:assert stats[skill]==(current,fine//10)
r={'implementation_id':'OR-MIG-0051','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','V0':'PASS','loaded_policy_oracle':str(oracle.relative_to(ROOT)).replace('\\','/'),
 'scope':'Actual exact-SHA cache-backed runtime and native Bury of clicked slot17 from full28 bag, preserving earlier bone slot3 and all unrelated items. Stale replay of actual Bury packet cannot consume another bone or grant XP. Actual configured4.5base at500percent produces225fineXP, displayed22. Acknowledged save, actual process restart and native relogin preserve inventory, exact fractional XP and all Stats without repeat reward.',
 'fine_xp_award':225,'displayed_prayer_xp':22,'selected_slot':17,'preserved_duplicate_slot':3,
 'limitations':['V1/V2/V3 remain partial for native Chaos/cancelled confirmation, member/level/modifier and interruption/crash cases.','No rendered/audio acceptance or original executable differential; source/pinned documentation comparison is separate.','Raw partial inventory18 is omitted by this observer; actual native post-apply state and authoritative SQL are checked, with independently decoded stat46 on real restart.']}
(OUT/'independent-assessment.json').write_text(json.dumps(r,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(r))
