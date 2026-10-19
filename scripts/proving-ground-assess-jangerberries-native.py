"""Correlate native ordinary-berry effects, actual storage and reconnect packets."""
import argparse,hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='16458999c44037d0e8b24fda975f60739f95ad4b'
p=argparse.ArgumentParser();p.add_argument('--feature',choices=['jangerberries-low','jangerberries-cap','jangerberries-overcap','jangerberries-delay'],required=True);p.add_argument('--run-id',default='native-001');a=p.parse_args();OUT=ROOT/'docs/openrune/verification/independent'/('deep-'+a.feature)/a.run_id
load=lambda n:json.loads((OUT/n).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');assert load('produce/before.json')==initial and load('relogin/before.json')==after
expected=[[93,s,247 if s==1 else 1931,1,0]for s in range(28)if s not in [0,13]]
assert after['inventory']==post['inventory']==expected
for state in [initial,after,post]:assert state['integrity_check']=='ok'and state['account_membership']and state['character'][:3]==initial['character'][:3]
for state in [after,post]:
    for old,new in zip(initial['stats'],state['stats']):
        if old[0]in [15,20]:assert new==([15,3,3,1740]if old[0]==15 else [20,2,2,830])
        else:
            assert old[0]==new[0]and old[2:]==new[2:]
            if old[0]not in [0,1,2,3,5]:assert old==new
prayer={'jangerberries-delay':2,'jangerberries-low':2,'jangerberries-cap':99,'jangerberries-overcap':105}[a.feature]
assert after['stats'][5]==post['stats'][5]==[5,prayer,99,130344311]
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['status']=='PASS'and m['implementation_id']=='OR-MIG-0145'and m['tested_sha']==m['implementation_sha']==SHA
    assert m['client_exit']==m['server_exit']==0 and m['save_acknowledgement']and m['graceful_shutdown']and m['resource_hashes_unchanged']and not m['forced_shutdown']
    assert m['server_sha256']==load('build.json')['binary_sha256']and m['client_sha256']==load('frozen-observer/build.json')['binaries']['pg_observer.exe']
log=(OUT/'produce/client.log').read_text(encoding='utf-8');relog=(OUT/'relogin/client.log').read_text(encoding='utf-8')
pattern=r'PG_BERRY_NATIVE_%s kind='+a.feature+r' index=(\d+) slot=(\d+) tick=(\d+) hp=(\d+) prayer=(\d+) attack=(\d+) defence=(\d+) strength=(\d+)'
begin=[tuple(map(int,v))for v in re.findall(pattern%'BEGIN',log)];end=[tuple(map(int,v))for v in re.findall(pattern%'COMMIT',log)];assert len(begin)==len(end)==2 and[end[i][1]for i in range(2)]==[0,13]
for old,new in zip(begin,end):
    assert old[:2]==new[:2]and new[3]==(min(99,old[3]+2)if old[3]<99 else old[3])
    assert new[4]==(old[4]+1 if old[4]<99 else old[4])and new[6]==max(0,old[6]-1)
    assert new[5]==(min(101,old[5]+2)if old[5]<101 else old[5])and new[7]==(min(100,old[7]+1)if old[7]<100 else old[7])
if a.feature=='jangerberries-delay':
    assert 'PG_BERRY_EARLY_SENT'in log and 'PG_BERRY_EARLY_REFUSED'in log
assert 'PG_CASE_PASS ordinary berry native kind='+a.feature in log and 'PG_NATIVE_NO_PENDING_SKILL_MULTI'in relog
def samples(text,skill):
    rows=[]
    for tick,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',text):
        if int(op)!=46:continue
        b=bytes.fromhex(h.replace(',',''));assert len(b)==7
        if (b[2]-128)&255==skill:rows.append([int(tick),b[1],int.from_bytes(b[5:7]+b[3:5],'big')])
    return rows
seen=samples(log,5);restored=samples(relog,5);assert all(r[2]==13034431 for r in seen+restored)
expected_samples={'jangerberries-delay':[0,1,2],'jangerberries-low':[0,1,2],'jangerberries-cap':[98,99],'jangerberries-overcap':[105]}[a.feature]
assert [r[1]for r in seen]==expected_samples and restored and all(r[1]==prayer for r in restored)
for skill in [0,1,2,3,5]:assert samples(relog,skill)[0][1]==after['stats'][skill][1]
traces=[json.loads(s.split('Food transition: ',1)[1])for s in(OUT/'produce/server.log').read_text(encoding='utf-8').splitlines()if s.startswith('Food transition: ')]
assert len(traces)==2
for t,b,e in zip(traces,begin,end):assert t['slot']==b[1]and t['before']==[247,1,0]and t['after']is None and t['hp_before']==b[3]and t['hp_after']==e[3]and t['fine_xp_before']==t['fine_xp_after']==130344311
assert 'Food transition:'not in(OUT/'relogin/server.log').read_text(encoding='utf-8')
names=['initial-state.json','build.json','frozen-observer/build.json','frozen-observer/pg_observer.rs']+[phase+'/'+n for phase in ['produce','relogin']for n in ['manifest.json','before.json','after.json','client.log','server.log']]
result={'implementation_id':'OR-MIG-0145','tested_sha':SHA,'status':'PASS_SCOPED','scope':'Two genuine native selected Eat actions with fullbag/retainedduplicate match all5currentstat transitions, wholepost-admissionXP/base/gear conservation, authoritative selected traces and real acknowledgedsave/serverrestart/native reconnect. Prayer packet trajectory and absence of unchanged capped/above-base updates independently decoded.','feature':a.feature,'native_begin':begin,'native_end':end,'received_prayer':seen,'reconnect_prayer':restored,'qualifications':['ActualSkills BMP requires separate direct visual inspection; no visual PASS inferred from creation.','Minimum15/20sourcebootstrap before gameplay and passiveHP/boost/drained-stat restoration after actions remain separate from food effects.','V1remainsFAIL9selecteddeathcases. NativeF2P,interruption/recovery andfullV4/sourceclock/audio/animation obligations are separate; no wholecontractacceptance.','Ordinary berry food only; no potion,Herbloreuse,acquisition or production action.'],'evidence_sha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest()for n in names}}
path=OUT/'independent-assessment.json';assert not path.exists();path.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8',newline='\n');print('Native berry',a.feature,'PASS_SCOPED')
