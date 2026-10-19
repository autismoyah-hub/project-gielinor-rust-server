"""Assess each immutable149 native stage campaign without replaying gameplay."""
import argparse,hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='76a011f52a9f2dcaf85eec08e64c9282d0602ced'
p=argparse.ArgumentParser();p.add_argument('--feature',choices=['giant-crab-food','giant-crab-partial','giant-crab-f2p'],required=True);p.add_argument('--run-id',default='native-001');a=p.parse_args();BASE=ROOT/'docs/openrune/verification/independent'/('deep-'+a.feature)/a.run_id
load=lambda n:json.loads((BASE/n).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');assert load('produce/before.json')==initial and load('relogin/before.json')==after
count=0 if a.feature=='giant-crab-f2p'else 2 if a.feature=='giant-crab-partial'else 5
expected=[r[:]for r in initial['inventory']]
if count==5:expected=[r for r in expected if r[1]!=13]
elif count==2:
    for row in expected:
        if row[1]==13:row[2]=7524
assert after['inventory']==post['inventory']==expected
for state in [initial,after,post]:assert state['integrity_check']=='ok'and state['account_membership']==(count>0)
for state in [after,post]:
    for old,new in zip(initial['stats'],state['stats']):
        if old[0]in [15,20]:assert new==([15,3,3,1740]if old[0]==15 else[20,2,2,830])
        elif old[0]==3:assert old[0]==new[0]and old[2:]==new[2:]
        else:assert old==new
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['status']=='PASS'and m['implementation_id']=='OR-MIG-0149'and m['tested_sha']==m['implementation_sha']==SHA
    assert m['client_exit']==m['server_exit']==0 and m['save_acknowledgement']and m['graceful_shutdown']and m['resource_hashes_unchanged']and not m['forced_shutdown']
    assert m['server_sha256']==load('build.json')['binary_sha256']and m['client_sha256']==load('frozen-observer/build.json')['binaries']['pg_observer.exe']
    assert m['launcher_sha256']==hashlib.sha256((BASE/'frozen-launcher.py').read_bytes()).hexdigest()
log=(BASE/'produce/client.log').read_text(encoding='utf-8');relog=(BASE/'relogin/client.log').read_text(encoding='utf-8');assert 'PG_CASE_PASS giantcrab'in log and 'PG_NATIVE_NO_PENDING_SKILL_MULTI'in relog
traces=[json.loads(s.split('Food transition: ',1)[1])for s in(BASE/'produce/server.log').read_text(encoding='utf-8').splitlines()if s.startswith('Food transition: ')]
assert len(traces)==count
ids=[7521,7523,7524,7525,7526]
for i,t in enumerate(traces):
    assert t['slot']==13 and t['before']==[ids[i],1,0]and t['after']==([ids[i+1],1,0]if i<4 else None)
    assert t['configured_delay']==2 and t['attack_delay']==0 and t['hp_after']==t['hp_before']+2 and t['fine_xp_before']==t['fine_xp_after']==130344311
    if i:assert t['tick']>=traces[i-1]['tick']+2
assert 'Food transition:'not in(BASE/'relogin/server.log').read_text(encoding='utf-8')
seq=[tuple(map(int,r))for r in re.findall(r'PG_SEQUENCE tick=(\d+) sequence=(\d+) delay=(\d+)',log)if r[1]=='829'];assert len(seq)==count and all(r[2]==0 for r in seq)
def samples(text):
    rows=[]
    for tick,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',text):
        if int(op)!=46:continue
        b=bytes.fromhex(h.replace(',',''));assert len(b)==7
        if (b[2]-128)&255==3:rows.append([int(tick),b[1],int.from_bytes(b[5:7]+b[3:5],'big')])
    return rows
hp=samples(log);restored=samples(relog);assert restored and restored[0][1]==after['stats'][3][1]
assert all(r[2]==13034431 for r in hp+restored)
if not count:
    assert 'PG_CRAB_F2P_MENU'in log and any(x in log for x in ['F2P BACKEND_REFUSAL','F2P UI_REFUSAL']);assert after['stats'][3]==post['stats'][3]==initial['stats'][3]
else:assert after['stats'][3][1]>=traces[-1]['hp_after']and post['stats'][3][1]>=after['stats'][3][1]
names=['initial-state.json','build.json','frozen-launcher.py','frozen-observer/build.json','frozen-observer/pg_observer.rs']+[phase+'/'+n for phase in ['produce','relogin']for n in ['manifest.json','before.json','after.json','client.log','server.log']]
r={'implementation_id':'OR-MIG-0149','tested_sha':SHA,'status':'PASS_SCOPED','feature':a.feature,'scope':f'Actual native {count} selected Eat commits at13 with duplicate1/fullbag/XP/gear preserved, exact2HP/stage/terminal behavior, readyattack0 trace and829receipt; F2P actualmenu refusal when count0. Realacknowledgedsave/serverrestart/native reconnect preserves exact intermediate/final/refusalinventory and initialreconnectHP, no replay.','server_commit_ticks':[t['tick']for t in traces],'received_hp':hp,'reconnect_hp':restored,'saved_hp':after['stats'][3],'relogin_saved_hp':post['stats'][3],'qualifications':['Native screenshot requires separate direct visual review; no visualPASS inferred from file creation.','Readiness-clock passiveHP restoration and source15/20loginminimums distinguished from food commits; no foodXP.','Native activeweaponattack/Walk exact2tickexpiry and death/actorfreshness are separate; World controls not substituted fornative.','No acquisition/cooking/burning/quest/potion/Herblore gameplay; no fullcontract acceptance.'],'evidence_sha256':{n:hashlib.sha256((BASE/n).read_bytes()).hexdigest()for n in names}}
path=BASE/'independent-assessment.json';assert not path.exists();path.write_text(json.dumps(r,indent=2)+'\n',encoding='utf-8',newline='\n');print(a.feature,'PASS_SCOPED',r['server_commit_ticks'])
