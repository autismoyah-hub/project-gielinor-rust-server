"""Assess real movement cancellation and interrupted-All persistence."""
import ast,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-combat-interrupt/interrupt-001'
def load(p):return json.loads((OUT/p).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json')
SHA='5a260c1bd6efa5f8a221bed7c96e6ac72fd428bc'
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS'
    assert m['client_exit']==m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and not m['forced_shutdown'] and m['resource_hashes_unchanged']
    assert load(phase+'/after.json')['integrity_check']=='ok'
assert load('relogin/before.json')==after and after['inventory']==post['inventory'] and after['stats']==post['stats']
expected=[[93,0,12695,1,0]]+[x for x in initial['inventory'] if x[1]>=4]
assert after['inventory']==expected
for a,b in zip(initial['stats'],after['stats']):assert a[:3]==b[:3] and b[3]-a[3]==(7500 if a[0]==15 else 0)
traces=[json.loads(s.split('Combat mixing transition: ',1)[1]) for s in (OUT/'produce/server.log').read_text(encoding='utf-8').splitlines() if s.startswith('Combat mixing transition: ')]
assert len(traces)==1;t=traces[0]
assert t['slots']==[0,1,2,3] and t['inputs']==[2436,2440,2442,269] and t['output']==12695 and not t['secondary_saved']
assert t['committed_at']-t['queued_at']==2 and t['fine_after']-t['fine_before']==7500 and t['rate_hundreds']==500
log=(OUT/'produce/client.log').read_text(encoding='utf-8')
assert 'native Walk before first completion' in log and 'PG_CASE_PASS combat native movement cancellation' in log
assert 'PG_DISCONNECT_PARTIAL combat outputs=1' in log
assert 'Combat mixing transition:' not in (OUT/'relogin/server.log').read_text(encoding='utf-8')
relog=(OUT/'relogin/client.log').read_text(encoding='utf-8');assert 'PG_CASE_PASS native relogin' in relog
observed=[ast.literal_eval(s.split(' inventory=',1)[1]) for s in relog.splitlines() if s.startswith('PG_APPLIED') and ' inventory=' in s]
assert [[93,s,i,n,0] for s,i,n in observed[-1] if i>=0]==expected
r={'implementation_id':'OR-MIG-0010','tested_sha':SHA,'campaign_kind':'historical','status':'PASS',
 'scope':'Actual native Make followed by Walk114 before first completion cancels with no consumption/XP. Reopened real All115(count28), one completed four-input product/7500fineXP, then actual socket disconnect with five available groups remaining. Acknowledged save, actual restart/native relogin preserve one output and remaining ingredients with no automatic queue replay.',
 'movement_cancel':'PASS','partial_disconnect_save_restart':'PASS','completed_products':1,'fine_xp_award':7500,
 'limitations':['Acknowledged normal disconnect/restart is not deliberate process-crash atomicity.','Native goggles, quest/level/member refusals, other stale selected claims and full reference acceptance remain separate.','Native post-apply inventory is observed; raw partial18 bytes were not recorded by this observer build.']}
(OUT/'independent-assessment.json').write_text(json.dumps(r,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(r))
