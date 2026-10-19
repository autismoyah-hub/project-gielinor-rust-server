"""Assess genuine F2P wine repetition and actual saved/restarted state."""
from pathlib import Path
import hashlib,json,re,struct
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-wine-drink-f2p/native-001';SHA='dfc2a1667c6c97e4d525a1aaea1ba7154d9aaad7'
load=lambda name:json.loads((OUT/name).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');build=load('build.json');frozen=load('frozen-observer/build.json')
assert load('produce/before.json')==initial and load('relogin/before.json')==after
expected=[[93,s,1935 if s==0 else 1993 if s in [1,4,7,13,20,27]else 1931,1,0]for s in range(28)]
assert after['inventory']==post['inventory']==expected
for state in [initial,after,post]:
    assert not state['account_membership']and state['integrity_check']=='ok'
    assert state['character'][:3]==initial['character'][:3]
for state in [after,post]:
    for old,new in zip(initial['stats'],state['stats']):
        if old[0]in [15,20]:assert new==([15,3,3,1740]if old[0]==15 else [20,2,2,830])
        else:
            assert old[0]==new[0]and old[2:]==new[2:]
            if old[0]not in [0,3]:assert old==new
    assert state['stats'][3]==[3,10,10,11540]
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['status']=='PASS'and m['implementation_sha']==m['tested_sha']==SHA
    assert m['client_exit']==m['server_exit']==0 and not m['forced_shutdown']and m['graceful_shutdown']and m['save_acknowledgement']and m['resource_hashes_unchanged']
    assert m['server_sha256']==build['binary_sha256']and m['client_sha256']==frozen['binaries']['pg_observer.exe']
log=(OUT/'produce/client.log').read_text(encoding='utf-8');relog=(OUT/'relogin/client.log').read_text(encoding='utf-8')
assert 'PG_CASE_PASS wine F2P genuine Drink menu'in log and 'PG_NATIVE_NO_PENDING_SKILL_MULTI'in relog
sent=re.findall(r'PG_TX label="(F2P Drink[^"]+)" tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',log)
assert len(sent)==2 and sent[0][1:]==sent[1][1:]and sent[0][0]!=sent[1][0]
begin=re.search(r'PG_WINE_F2P_BEGIN tick=(\d+) member=false attack=(\d+) hp=(\d+)',log);end=re.search(r'PG_WINE_F2P_COMMIT tick=(\d+) attack=(\d+) hp=(\d+) retainedDuplicate=true emptyJugDrinkAbsent=true noXPgear=true',log);assert begin and end
assert int(end[2])==max(0,int(begin[2])-2)and int(end[3])==10
traces=[json.loads(s.split('Food transition: ',1)[1])for s in (OUT/'produce/server.log').read_text(encoding='utf-8').splitlines()if s.startswith('Food transition: ')]
assert len(traces)==1;t=traces[0];assert t['slot']==0 and t['before']==[1993,1,0]and t['after']==[1935,1,0]and t['hp_before']==int(begin[3])and t['hp_after']==10 and t['fine_xp_before']==t['fine_xp_after']==11540
def samples(text,skill):
    a=[]
    for tick,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',text):
        if int(op)!=46:continue
        b=bytes.fromhex(h.replace(',',''));assert len(b)==7
        if (b[2]-128)&255==skill:a.append((int(tick),b[1],int.from_bytes(b[5:7]+b[3:5],'big')))
    return a
attack=samples(log,0);restart=samples(relog,0);assert all(s[2]==13034431 for s in attack+restart)
assert any(s[1]==int(end[2])and s[0]>=int(begin[1])for s in attack)
assert restart[0][1]==after['stats'][0][1]
assert post['stats'][0][1]==restart[-1][1]+1  # One source-qualified passive tick during logout/save; not received after disconnect.
frame=ROOT/'work/native-wine-drink-f2p/native-001/wine-f2p-inventory.bmp';raw=frame.read_bytes();width,height=struct.unpack_from('<ii',raw,18);assert(width,abs(height))==(765,503)
names=['initial-state.json','build.json','frozen-observer/build.json','frozen-observer/pg_observer.rs']+[phase+'/'+n for phase in ['produce','relogin']for n in ['manifest.json','before.json','after.json','client.log','server.log']]
result={'implementation_id':'OR-MIG-0137','tested_sha':SHA,'status':'PASS_SCOPED','scope':'Actual F2P native Drink menu derived two rapid selections at the same client clock; one selected wine consumed, one jug retained, Attack8to6/HP5to10, other six wines/fullbag/gear/post-admissionXP conserved. One authoritative food trace and actual saved/restarted F2P account confirm no duplicate effect/replay. Native inventory frame directly viewed: empty jug0and retained wines visible.','sent_labels':[s[0]for s in sent],'native_begin':list(begin.groups()),'native_end':list(end.groups()),'received_attack':attack,'reconnect_attack':restart,'private_frame':{'path':frame.relative_to(ROOT).as_posix(),'sha256':hashlib.sha256(raw).hexdigest(),'width':width,'signed_height':height},'qualifications':['Repeated action may encounter delay, stale-item or submission coalescing; this combined conservation control does not isolate cooldown alone.','Controller item_action returned an error for Drink on the visible empty jug; no forced backend stale payload was sent. Detailed menu-list evidence is a separate stricter observation if required.','Initial FloorOne admission raises15/20minimums before drinks; no Herblore item or potion action tested. Native reconnect first packet restores the acknowledged savedAttack6 exactly. Later SQL7after logout is consistent with one normal100clock restoration while the server drains; that final+1was not received after disconnect and remains source-qualified, not independently timed. No replay/extra wine consumption occurred.','Bounded native inventory rendering only; no full scene/actor animation/audio or independent combat-clock parity. V1historicalFAIL and remaining V2/V3/V4 obligations persist.'],'evidence_sha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest()for n in names}}
p=OUT/'independent-assessment.json';assert not p.exists();p.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8',newline='\n');print('F2P wine repetition/save/restart PASS_SCOPED')
