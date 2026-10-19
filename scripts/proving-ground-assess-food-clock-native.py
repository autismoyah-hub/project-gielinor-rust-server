"""Correlate pinned148 genuine food/combo menus, traces, SQL and real reconnect."""
from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parents[1];SHA='3640ac17f8b4e7c293c9bc1fcb78ca1b96509744';BASE=ROOT/'docs/openrune/verification/independent/deep-food-clock-orders/native-002'
load=lambda name:json.loads((BASE/name).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json')
assert load('produce/before.json')==initial and load('relogin/before.json')==after
expected=[row for row in initial['inventory']if row[1]not in [0,1,13,14]]
assert after['inventory']==post['inventory']==expected
for state in [initial,after,post]:assert state['integrity_check']=='ok'
for state in [after,post]:
    for old,new in zip(initial['stats'],state['stats']):
        if old[0]in [15,20]:assert new==([15,3,3,1740]if old[0]==15 else [20,2,2,830])
        elif old[0]==3:assert old[0]==new[0]and old[2:]==new[2:]
        else:assert old==new
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['status']=='PASS'and m['implementation_id']=='OR-MIG-0148'and m['tested_sha']==m['implementation_sha']==SHA
    assert m['client_exit']==m['server_exit']==0 and m['save_acknowledgement']and m['graceful_shutdown']and m['resource_hashes_unchanged']and not m['forced_shutdown']
    assert m['server_sha256']==load('build.json')['binary_sha256']and m['client_sha256']==load('frozen-observer/build.json')['binaries']['pg_observer.exe']
    assert m['launcher_sha256']==hashlib.sha256((BASE/'frozen-launcher.py').read_bytes()).hexdigest()
log=(BASE/'produce/client.log').read_text(encoding='utf-8');relog=(BASE/'relogin/client.log').read_text(encoding='utf-8')
assert 'PG_CASE_PASS foodclock FOOD COMBO only'in log and 'PG_NATIVE_NO_PENDING_SKILL_MULTI'in relog
traces=[json.loads(s.split('Food transition: ',1)[1])for s in(BASE/'produce/server.log').read_text(encoding='utf-8').splitlines()if s.startswith('Food transition: ')]
assert len(traces)==4 and [t['slot']for t in traces]==[0,13,14,1]
assert traces[0]['tick']==traces[1]['tick']and traces[2]['tick']>=traces[1]['tick']+3 and traces[3]['tick']>=traces[2]['tick']+3
for t,id,heal in zip(traces,[2142,2162,2162,2142],[3,2,2,3]):
    assert t['before']==[id,1,0]and t['after']is None and t['configured_delay']==3 and t['attack_delay']==0
    assert t['hp_after']==t['hp_before']+heal and t['fine_xp_before']==t['fine_xp_after']==130344311
assert 'Food transition:'not in(BASE/'relogin/server.log').read_text(encoding='utf-8')
assert log.count('PG_SEQUENCE ')>=3
seq=[tuple(map(int,r))for r in re.findall(r'PG_SEQUENCE tick=(\d+) sequence=(\d+) delay=(\d+)',log)if r[1]=='829'];assert len(seq)==len({t['tick']for t in traces})==3 and all(r[2]==0 for r in seq)
def samples(text,skill):
    rows=[]
    for tick,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',text):
        if int(op)!=46:continue
        b=bytes.fromhex(h.replace(',',''));assert len(b)==7
        if (b[2]-128)&255==skill:rows.append([int(tick),b[1],int.from_bytes(b[5:7]+b[3:5],'big')])
    return rows
hp=samples(log,3);restored=samples(relog,3);assert hp and restored and all(r[2]==13034431 for r in hp+restored)
assert restored[0][1]==after['stats'][3][1]
assert after['stats'][3][1]>=traces[-1]['hp_after']and post['stats'][3][1]>=after['stats'][3][1]
assert after['stats'][3][1]-traces[-1]['hp_after']<=1 and post['stats'][3][1]-after['stats'][3][1]<=2
# Any later positive HP tick is qualified passive restoration, not an extra Eat.
frame=ROOT/'work/native-food-clock-orders/native-002/food-clock-orders-inventory.bmp'
names=['initial-state.json','build.json','frozen-launcher.py','frozen-observer/build.json','frozen-observer/pg_observer.rs']+[phase+'/'+n for phase in ['produce','relogin']for n in ['manifest.json','before.json','after.json','client.log','server.log']]
r={'implementation_id':'OR-MIG-0148','tested_sha':SHA,'status':'PASS_SCOPED','scope':'Genuine native Food0/Combo13 commit in same authoritative tick; third distinct Food1 refused. Later Combo14 first blocks Food1, then Food1 succeeds after expiry. Four exact selected transactions, allHP/XP/gear/wholebag checks, readyweapon0attacktrace, native829x3 across3distinctcommitticks, actualacknowledgedsave/serverrestart/native reconnect retain results withoutreplay.','server_commit_ticks':[t['tick']for t in traces],'native_hp_samples':hp,'reconnect_hp_samples':restored,'saved_hp':after['stats'][3],'relogin_saved_hp':post['stats'][3],'qualifications':['Two foods in one server tick share one native829actor sequence block; four food commits produce three distinct-tick sequences, not four independently rendered animations.','FOOD/COMBO-only; potionmixedorders and consumption remain user-deferred; no wholeV2/V3PASS.','Actual melee/ranged/spell timing, realnativeWalk/activeweapon attacks and fresh-relogin cooldown admission remain separate obligations.','World18case30tick sentinel proves both realdeathhooks reset; thisnativecampaign doesnotperform death.','Membership SQL was omitted from the original148snapshot selector; nativecontroller asserted actualmembers=true before gameplay. No three-snapshot membership-SQL conservation claim.','Source login15/20minimums and subsequent passiveHPticks are separate from four food commits; no foodXP inferred.','Initialnative001FAIL from starterHPreset retained; freshnative002seedAttack99avoids reset, identicalserver/controller.','Directly viewed uneditednativeInventoryBMP shows24pots,fouremptyslots,HP63andfourfoodchatmessages; native sequence receipt/application observed, no avatar/audio parityclaim.'],'private_frame':{'path':frame.relative_to(ROOT).as_posix(),'sha256':hashlib.sha256(frame.read_bytes()).hexdigest()},'evidence_sha256':{n:hashlib.sha256((BASE/n).read_bytes()).hexdigest()for n in names}}
path=BASE/'independent-assessment.json';assert not path.exists();path.write_text(json.dumps(r,indent=2)+'\n',encoding='utf-8',newline='\n');print('148native FOOD/COMBO PASS_SCOPED',r['server_commit_ticks'])
