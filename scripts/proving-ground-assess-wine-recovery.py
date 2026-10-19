"""Assess pre-save wine recovery; explicitly reject neutralized Attack oracles."""
from pathlib import Path
import argparse,hashlib,json,re
ROOT=Path(__file__).resolve().parents[1];SHA='dfc2a1667c6c97e4d525a1aaea1ba7154d9aaad7'
p=argparse.ArgumentParser();p.add_argument('--run-id',choices=['native-001','native-002'],required=True);a=p.parse_args();OUT=ROOT/'docs/openrune/verification/independent/deep-wine-drink-crash'/a.run_id
load=lambda n:json.loads((OUT/n).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json')
assert load('produce/before.json')==initial and load('relogin/before.json')==after
assert after['inventory']==post['inventory']==initial['inventory']and after['stats']==initial['stats']
for state in [initial,after,post]:assert state['integrity_check']=='ok'and state['character'][:3]==initial['character'][:3]
for old,new in zip(initial['stats'],post['stats']):
    if old[0]in [15,20]:assert new==([15,3,3,1740]if old[0]==15 else [20,2,2,830])
    else:
        assert old[0]==new[0]and old[2:]==new[2:]
        if old[0]not in [0,3]:assert old==new
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['status']=='PASS'and m['tested_sha']==m['implementation_sha']==SHA
    assert m['client_exit']==0 and m['resource_hashes_unchanged']and not m['forced_shutdown']
    assert m['server_sha256']==load('build.json')['binary_sha256']and m['client_sha256']==load('frozen-observer/build.json')['binaries']['pg_observer.exe']
    if phase=='produce':assert m['intentional_crash_injected']and m['server_exit']!=0 and not m['save_acknowledgement']and not m['graceful_shutdown']
    else:assert m['server_exit']==0 and m['save_acknowledgement']and m['graceful_shutdown']
log=(OUT/'produce/client.log').read_text(encoding='utf-8');relog=(OUT/'relogin/client.log').read_text(encoding='utf-8');assert 'PG_EXPECTED_CRASH_DISCONNECT'in log and 'PG_NATIVE_NO_PENDING_SKILL_MULTI'in relog
begin=re.search(r'PG_WINE_RECOVERY_BEGIN tick=(\d+) attack=(\d+) hp=(\d+)',log);ready=re.search(r'PG_CRASH_READY unsaved native wine selectedjug1935 attack=(\d+) hp=(\d+) tick=(\d+)',log);assert begin and ready
assert int(ready[1])==int(begin[2])-2 and int(ready[2])==10
traces=[json.loads(s.split('Food transition: ',1)[1])for s in (OUT/'produce/server.log').read_text(encoding='utf-8').splitlines()if s.startswith('Food transition: ')]
assert len(traces)==1 and traces[0]['before']==[1993,1,0]and traces[0]['after']==[1935,1,0]and traces[0]['slot']==0
assert 'Food transition:'not in(OUT/'relogin/server.log').read_text(encoding='utf-8')
def first_stat(text,skill):
    for tick,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',text):
        if int(op)!=46:continue
        b=bytes.fromhex(h.replace(',',''))
        if (b[2]-128)&255==skill:return[int(tick),b[1],int.from_bytes(b[5:7]+b[3:5],'big')]
    raise AssertionError('Missing initial reconnect stat')
attack=first_stat(relog,0);hp=first_stat(relog,3);assert attack[1]==initial['stats'][0][1]and hp[1]==initial['stats'][3][1]
assert attack[2]==initial['stats'][0][3]//10 and hp[2]==initial['stats'][3][3]//10
discriminating=int(ready[1])!=initial['stats'][0][1]
if a.run_id=='native-002':assert discriminating and int(begin[2])==99 and int(ready[1])==97 and attack[1]==99
names=['initial-state.json','build.json','frozen-observer/build.json','frozen-observer/pg_observer.rs']+[phase+'/'+n for phase in ['produce','relogin']for n in ['manifest.json','before.json','after.json','client.log','server.log']]
result={'implementation_id':'OR-MIG-0137','tested_sha':SHA,'status':'PASS_SCOPED','scope':'Fresh owned disposable server stopped after a received unsaved genuine native wine transaction, with no save acknowledgement or graceful drain. ActualSQL retains original whole inventory/all current-base-fineXP stats; real restart/native reconnect restores original wine and HP without replay. Post-login source minimums15/20and passive restoration remain explicitly separate.','observed_unsaved_attack':int(ready[1]),'stored_original_attack':initial['stats'][0][1],'first_reconnect_attack':attack,'first_reconnect_hp':hp,'attack_specific_rollback':'PASS: distinct99to97unsavedchange restores99'if discriminating else 'NOT_PROVEN: unsaved5equals original5afterpassive regeneration; HP/jug rollback remains proven','qualifications':['Only termination before save acknowledgement; no claim of SQLite commit-fault injection.','Native001 Attack oracle was neutralized by passive regeneration and is not accepted as Attack-specific rollback. Native002 is a separately frozen base99variant, never a rewrite of001.','No successful prior disposable account reseeded or six-Drink campaign replayed. V1deathFAILand native interruption/rendered-animation/reference obligations remain separate.'],'evidence_sha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest()for n in names}}
path=OUT/'independent-assessment.json';assert not path.exists();path.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS_SCOPED wine unsavedHP/jug recovery; Attack discriminator:',discriminating)
