"""Independent native last-charge, queue-stop and real restart assessment."""
import json
from collections import Counter
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-chemistry-last/last-001';SHA='e7ac3608e61893d80b2de6a1a2e512b3ab785505'
def load(p):return json.loads((OUT/p).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json')
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS'
    assert m['client_exit']==m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and not m['forced_shutdown'] and m['resource_hashes_unchanged']
assert initial['chemistry_state']==[[1,1]] and after['chemistry_state']==post['chemistry_state']==[[5,1]]
traces=[json.loads(s.split('Finished potion transition: ',1)[1]) for s in (OUT/'produce/server.log').read_text(encoding='utf-8').splitlines() if s.startswith('Finished potion transition: ')]
n=len(traces);assert 0<n<14
for i,t in enumerate(traces):
    assert (t['index'],t['unfinished_slot'],t['secondary_slot'],t['unfinished'],t['secondary'])==(i,i*2,i*2+1,93,235)
    assert t['committed_at']-t['queued_at']==2 and t['fine_after']-t['fine_before']==1875 and not t['secondary_saved']
    assert t['extra_dose']==t['amulet_crumbled']==t['stop_on_crumble']==(i==n-1)
    assert t['output']==(2446 if i==n-1 else 175) and t['charges_remaining']==(5 if i==n-1 else 1)
counts=Counter({})
for inv,slot,item,count,vars in after['inventory']:
    assert inv==93 and count==1 and vars==0;counts[item]+=count
assert counts==Counter({93:14-n,235:14-n,175:n-1,2446:1})
for a,b in zip(initial['stats'],after['stats']):
    assert a[:3]==b[:3] and b[3]-a[3]==(1875*n if a[0]==15 else 0)
assert after['inventory']==post['inventory'] and after['stats']==post['stats'] and after['integrity_check']==post['integrity_check']=='ok'
assert 'Finished potion transition:' not in (OUT/'relogin/server.log').read_text(encoding='utf-8')
assert f'potions={n},extra=1,crumbled=true;verify=true' in (OUT/'relogin/client.log').read_text(encoding='utf-8')
report={'implementation_id':'OR-MIG-0006','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','potions':n,'ordinary_outputs':n-1,'extra_outputs':1,'remaining_pairs':14-n,'fine_xp_award':1875*n,
 'scope':'Native All quantity production from full28 bag: last charge gives exactly one extra dose without extra XP, removes ordinary neck amulet, resets next-amulet charges5 and interrupts queue with ingredients remaining. Acknowledged save, real restart and native relogin preserve partial production, stop policy and fineXP; no production replay.',
 'limitations':['This native sample does not estimate proc probability. Deterministic exhaustive probability and modifier branches require separate authoritative tests.','Manual movement/logout interruption, simultaneous goggles and negative dialog campaigns are separate obligations.']}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
