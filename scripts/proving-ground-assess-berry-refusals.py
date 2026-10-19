"""Assess immutable145 F2P evidence and classify the late-repeat timing attempt."""
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent';SHA='16458999c44037d0e8b24fda975f60739f95ad4b'
def save(p,obj):
    assert not p.exists(),p
    p.write_text(json.dumps(obj,indent=2)+'\n',encoding='utf-8',newline='\n')
base=OUT/'deep-jangerberries-f2p/native-001'
load=lambda n:json.loads((base/n).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json')
assert load('produce/before.json')==initial and load('relogin/before.json')==after
assert initial['inventory']==after['inventory']==post['inventory']
for state in [initial,after,post]:assert state['integrity_check']=='ok' and not state['account_membership']
for state in [after,post]:
    for old,new in zip(initial['stats'],state['stats']):
        assert new==([15,3,3,1740]if old[0]==15 else [20,2,2,830]if old[0]==20 else old)
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['status']=='PASS' and m['tested_sha']==m['implementation_sha']==SHA
    assert m['client_exit']==m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown']
    assert m['server_sha256']==load('build.json')['binary_sha256'] and m['client_sha256']==load('frozen-observer/build.json')['binaries']['pg_observer.exe']
    assert 'Food transition:' not in (base/phase/'server.log').read_text(encoding='utf-8')
log=(base/'produce/client.log').read_text(encoding='utf-8')
assert 'options=["Examine", "Drop", "Use", "Eat"]' in log and 'PG_CASE_PASS berry F2P BACKEND_REFUSAL' in log
assert 'PG_NATIVE_NO_PENDING_SKILL_MULTI' in (base/'relogin/client.log').read_text(encoding='utf-8')
names=['initial-state.json','build.json','frozen-observer/build.json','frozen-observer/pg_observer.rs']+[p+'/'+n for p in ['produce','relogin']for n in ['manifest.json','before.json','after.json','client.log','server.log']]
save(base/'independent-assessment.json',{'implementation_id':'OR-MIG-0145','tested_sha':SHA,'status':'PASS_SCOPED','scope':'Actual F2P visible Eat menu selected through native UI; backend membership refusal conserves full inventory/all current stats/XP/gear. No Food transition. Acknowledged save and actual server restart/native reconnect conserve the refusal state.','qualifications':['Whole contract remains V1 FAIL for separately proven death-state defects.','No potion or Herblore gameplay; source login minimum15/20 changes precede gameplay.','Native local image directly reviewed separately; no actor/audio parity claim.'],'evidence_sha256':{n:hashlib.sha256((base/n).read_bytes()).hexdigest()for n in names}})
frame=ROOT/'work/native-jangerberries-f2p/native-001/jangerberries-f2p-inventory.bmp'
save(base/'visual-assessment.json',{'implementation_id':'OR-MIG-0145','tested_sha':SHA,'status':'PASS_SCOPED','scope':'Direct image review: full28-slot inventory retains three berry stacks and25pots; chat visibly says You need to be a member to eat that. Unedited native renderer frame.','private_frame':{'path':frame.relative_to(ROOT).as_posix(),'sha256':hashlib.sha256(frame.read_bytes()).hexdigest()}})
base=OUT/'deep-jangerberries-delay/native-001'
m=json.loads((base/'produce/manifest.json').read_text(encoding='utf-8'));assert m['status']=='FAIL' and m['tested_sha']==SHA
traces=[json.loads(s.split('Food transition: ',1)[1])for s in(base/'produce/server.log').read_text(encoding='utf-8').splitlines()if s.startswith('Food transition: ')]
assert len(traces)==2 and [t['slot']for t in traces]==[0,13]
gap=traces[1]['tick']-traces[0]['tick'];assert gap==5 and gap>=traces[0]['configured_delay']==3
save(base/'timing-assessment.json',{'implementation_id':'OR-MIG-0145','tested_sha':SHA,'status':'INCONCLUSIVE_TIMING','scope':'Attempted early native Eat reached authoritative commit five ticks after first commit; configured food delay is three. Both actions validly committed after expiry. The controller waited for delivered inventory/stat updates before issuing repeat; transport/update latency invalidated early-input assumption. No implementation defect inferred.','server_commit_ticks':[t['tick']for t in traces],'configured_delay':3,'next_action':'Use a fresh disposable run with two distinct genuine menu selections dispatched before network polling; retain this failed attempt and its saved database unchanged.','evidence_sha256':{n:hashlib.sha256((base/n).read_bytes()).hexdigest()for n in ['produce/manifest.json','produce/server.log','produce/client.log','produce/after.json','frozen-observer/pg_observer.rs']}})
print('F2P145 PASS_SCOPED; delay001 INCONCLUSIVE_TIMING, not implementation failure')
