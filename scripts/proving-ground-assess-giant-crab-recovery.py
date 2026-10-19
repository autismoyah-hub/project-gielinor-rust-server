"""Assess fresh owned149 pre-save stage rollback and actual native recovery."""
from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parents[1];SHA='76a011f52a9f2dcaf85eec08e64c9282d0602ced';BASE=ROOT/'docs/openrune/verification/independent/deep-giant-crab-crash/native-001'
load=lambda n:json.loads((BASE/n).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');assert load('produce/before.json')==initial and load('relogin/before.json')==after
assert initial['inventory']==after['inventory']==post['inventory']and after['stats']==initial['stats']
assert next(r for r in after['inventory']if r[1]==13)==[93,13,7521,1,0]
assert initial['stats'][3]==after['stats'][3]==[3,50,99,130344311]
for state in [initial,after,post]:assert state['integrity_check']=='ok'and state['account_membership']and state['character'][:3]==initial['character'][:3]
for old,new in zip(initial['stats'],post['stats']):
    if old[0]in [15,20]:assert new==([15,3,3,1740]if old[0]==15 else[20,2,2,830])
    else:
        assert old[0]==new[0]and old[2:]==new[2:]
        if old[0]!=3:assert old==new
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['status']=='PASS'and m['tested_sha']==m['implementation_sha']==SHA and m['implementation_id']=='OR-MIG-0149'
    assert m['client_exit']==0 and m['resource_hashes_unchanged']and not m['forced_shutdown']
    assert m['server_sha256']==load('build.json')['binary_sha256']and m['client_sha256']==load('frozen-observer/build.json')['binaries']['pg_observer.exe']
    assert m['launcher_sha256']==hashlib.sha256((BASE/'frozen-launcher.py').read_bytes()).hexdigest()
    if phase=='produce':assert m['intentional_crash_injected']and m['server_exit']!=0 and not m['save_acknowledgement']and not m['graceful_shutdown']
    else:assert m['server_exit']==0 and m['save_acknowledgement']and m['graceful_shutdown']
log=(BASE/'produce/client.log').read_text(encoding='utf-8');relog=(BASE/'relogin/client.log').read_text(encoding='utf-8');assert 'PG_EXPECTED_CRASH_DISCONNECT'in log and 'PG_NATIVE_NO_PENDING_SKILL_MULTI'in relog
commits=[tuple(map(int,r))for r in re.findall(r'PG_CRAB_NATIVE_COMMIT kind=giant-crab-crash index=(\d+) next=(\d+) tick=(\d+) hp=(\d+)',log)];assert len(commits)==2 and [r[1]for r in commits]==[7523,7524]
ready=re.search(r'PG_CRASH_READY unsaved native crab stage7524 hp=(\d+) tick=(\d+)',log);assert ready and int(ready[1])==commits[-1][3]>50
traces=[json.loads(s.split('Food transition: ',1)[1])for s in(BASE/'produce/server.log').read_text(encoding='utf-8').splitlines()if s.startswith('Food transition: ')]
assert len(traces)==2
for i,t in enumerate(traces):assert t['slot']==13 and t['before']==[7521 if i==0 else 7523,1,0]and t['after']==[7523 if i==0 else 7524,1,0]and t['hp_after']==t['hp_before']+2 and t['fine_xp_before']==t['fine_xp_after']==130344311
assert 'Food transition:'not in(BASE/'relogin/server.log').read_text(encoding='utf-8')
samples=[]
for tick,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',relog):
    if int(op)!=46:continue
    b=bytes.fromhex(h.replace(',',''));assert len(b)==7
    if (b[2]-128)&255==3:samples.append([int(tick),b[1],int.from_bytes(b[5:7]+b[3:5],'big')])
assert samples and samples[0][1]==50 and all(r[2]==13034431 for r in samples)
assert all(b[1]-a[1]in [0,1]for a,b in zip(samples,samples[1:]))and post['stats'][3][1]>=samples[-1][1]
names=['initial-state.json','build.json','frozen-launcher.py','frozen-observer/build.json','frozen-observer/pg_observer.rs']+[phase+'/'+n for phase in ['produce','relogin']for n in ['manifest.json','before.json','after.json','client.log','server.log']]
r={'implementation_id':'OR-MIG-0149','tested_sha':SHA,'status':'PASS_SCOPED','scope':'Freshgenuine native2bites observedstage7524andHPgreaterthan50, thenonlyowneddisposableserver stoppedbeforeanySaveack. Durablewholeinventory/allStats matchinitialstage7521/HP50. Actualrestart/nativefirstHP50andoriginalinventory confirmdiscriminatingrollback, nofoodreplay; subsequentnormalacknowledgedsave completes.','native_unsaved_commits':commits,'reconnect_hp':samples,'post_relogin_sql_hp':post['stats'][3],'qualifications':['Pre-save actor-state recovery only; noSQLitecommit/writefault atomicity claim. Startup migrations/login metadata canchange; notbyte-identicalwholeDB.','Source15/20loginminimums reapplyonlyonnewadmission; passiveHPafterfirstreconnect50 remainsseparate from food.','No native death/activeweapon/Walk/sourceparity claim; V1FAIL45unresolvedfoodlifecases remains.','No acquisition/cooking/potion/Herbloregameplay; previoussuccessfulfull/intermediate/F2P DBs not touched.'],'evidence_sha256':{n:hashlib.sha256((BASE/n).read_bytes()).hexdigest()for n in names}}
path=BASE/'independent-assessment.json';assert not path.exists();path.write_text(json.dumps(r,indent=2)+'\n',encoding='utf-8',newline='\n');print('149unsaved-stage/nativeHP rollback PASS_SCOPED',commits,samples[0])
