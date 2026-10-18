"""Decode genuine native wine observations and actual durable account images."""
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'docs/openrune/verification/independent/deep-wine-drink/native-002'
SHA='dfc2a1667c6c97e4d525a1aaea1ba7154d9aaad7'
load=lambda name:json.loads((OUT/name).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json')
assert load('produce/before.json')==initial and load('relogin/before.json')==after
expected=[[93,slot,1935 if slot in [0,4,7,13,20,27] else 1993 if slot==1 else 1931,1,0]for slot in range(28)]
assert after['inventory']==post['inventory']==expected
for state in [initial,after,post]:
    assert state['integrity_check']=='ok' and state['account_membership']
    for old,new in zip(initial['stats'],state['stats']):
        if old[0]in [15,20]:
            expected_bootstrap=[15,3,3,1740]if old[0]==15 else [20,2,2,830]
            if state is not initial:assert new==expected_bootstrap
        else:
            assert old[0]==new[0] and old[2:]==new[2:]
            if old[0]not in [0,3]:assert old==new
assert initial['stats'][0]==[0,5,99,130344311] and initial['stats'][3]==[3,2,10,11540]
assert after['stats'][3]==post['stats'][3]==[3,10,10,11540]
build=load('build.json');frozen=load('frozen-observer/build.json')
assert build['tested_sha']==SHA and build['exit_code']==0 and frozen['status']=='PASS'
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json')
    assert m['status']=='PASS'and m['implementation_id']=='OR-MIG-0137'and m['implementation_sha']==m['tested_sha']==SHA
    assert m['client_exit']==m['server_exit']==0 and m['save_acknowledgement']and m['graceful_shutdown']and m['resource_hashes_unchanged']and not m['forced_shutdown']
    assert m['server_sha256']==build['binary_sha256']and m['client_sha256']==frozen['binaries']['pg_observer.exe']
log=(OUT/'produce/client.log').read_text(encoding='utf-8')
begin=[tuple(map(int,v))for v in re.findall(r'PG_WINE_NATIVE_BEGIN index=(\d+) slot=(\d+) tick=(\d+) hp=(\d+) attack=(\d+)',log)]
commit=[tuple(map(int,v))for v in re.findall(r'PG_WINE_NATIVE_COMMIT index=(\d+) slot=(\d+) tick=(\d+) hp=(\d+) attack=(\d+)',log)]
assert len(begin)==len(commit)==6 and [x[1]for x in begin]==[0,4,7,13,20,27]
for a,b in zip(begin,commit):assert a[:2]==b[:2]and b[2]>=a[2]and b[3]==10 and b[4]==max(0,a[4]-2)
assert 'PG_CASE_PASS wine six genuine Drink menus'in log
traces=[json.loads(s.split('Food transition: ',1)[1])for s in (OUT/'produce/server.log').read_text(encoding='utf-8').splitlines()if s.startswith('Food transition: ')]
assert len(traces)==6
for t,b in zip(traces,begin):
    assert t['slot']==b[1]and t['before']==[1993,1,0]and t['after']==[1935,1,0]
    assert t['hp_before']==b[3]and t['hp_after']==10 and t['fine_xp_before']==t['fine_xp_after']==11540
    assert t['configured_delay']==3 and t['attack_delay']is None
def stat_samples(text,skill):
    result=[]
    for tick,op,payload in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',text):
        if int(op)!=46:continue
        b=bytes.fromhex(payload.replace(',',''));assert len(b)==7
        if (b[2]-128)&255!=skill:continue
        result.append({'tick':int(tick),'current':b[1],'xp':int.from_bytes(b[5:7]+b[3:5],'big')})
    return result
bootstrap={}
for skill,current,xp in [(15,3,174),(20,2,83)]:
    observed=[v for v in stat_samples(log,skill)if v['tick']<begin[0][2]];assert observed and observed[0]['current']==current and observed[0]['xp']==xp;bootstrap[skill]=observed
attack=stat_samples(log,0);hp=stat_samples(log,3)
assert attack and hp and all(x['xp']==13034431 for x in attack)and all(x['xp']==1154 for x in hp)
for action in commit:
    available=[x for x in attack if x['tick']<=action[2]+5];assert available and available[-1]['current']==action[4]
assert attack[-1]['current']==0 and after['stats'][0][1]==1
# Runtime continues while logout/save drains; one100clock passive restoration
# follows the last zero drain before the actual observed save. This is qualified
# separately from the six directly correlated consumption transitions.
relog=(OUT/'relogin/client.log').read_text(encoding='utf-8');samples=stat_samples(relog,0)
assert samples and samples[0]['current']==after['stats'][0][1]and post['stats'][0][1]==samples[-1]['current']
assert all(x['xp']==13034431 for x in samples)and 'PG_NATIVE_NO_PENDING_SKILL_MULTI'in relog
names=['initial-state.json','build.json','frozen-observer/build.json','frozen-observer/pg_observer.rs']+[phase+'/'+n for phase in ['produce','relogin']for n in ['manifest.json','before.json','after.json','client.log','server.log']]
result={'implementation_id':'OR-MIG-0137','tested_sha':SHA,'status':'PASS_SCOPED','scope':'Six genuine native Drink menu actions independently correlate selected jug retention, capped11HP healing and currentAttack subtraction floor0 with received stat packets/server transitions, then acknowledged storage and actual server restart/native reconnect. Full28bag, retained duplicate and post-admission XP/base stats conserved. Initial runtime FloorOne admission raises15to3/fine1740and20to2/fine830; this pre-drink bootstrap is explicitly separate from consumption. Saved Attack1/reconnect1 is qualified normal restoration after last observed Attack0 during logout/save, not an assertion that0remains frozen.','pre_drink_bootstrap_stat_packets':bootstrap,'native_begin':begin,'native_commit':commit,'received_attack':attack,'received_hp':hp,'reconnect_attack':samples,'qualifications':['Native001 preserved launcher failure sent no Drink. This native002 is the first execution of these six actions; no successful campaign replay or DB overwrite.','Initial source floor_progression::Policy::login raises minimum15/20before drinks; raw initial packets must establish this baseline. No Herblore item or potion action was tested.','Saved Attack1 versus last native0 is consistent with the exact100clock drained-stat timer continuing during logout/save. The six consumption steps are directly observed; precise save-time passive-timer ordering is source-qualified, not independently fault-tested.','Passive regeneration before drinks is independently observed; native attack sequence starts at actual7, not an invented seed5. Exact5to3to1to0 separately verified in pinned headless tests.','Rendered BMPs require separate direct visual assessment. No visual PASS inferred from file existence or observer log.','V1 remains FAIL from18fresh life cases. Native F2P, rapid refusal, interruption and unsaved recovery remain separate; no whole contract or V2/V3 acceptance.','Generic3tick food protection and attack_delay:null are qualified host behavior; no independent combat-clock/audio parity claim.'],'evidence_sha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest()for n in names}}
path=OUT/'independent-assessment.json';assert not path.exists();path.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8',newline='\n');print('Native wine positive/save/restart PASS_SCOPED; contract remains open')
