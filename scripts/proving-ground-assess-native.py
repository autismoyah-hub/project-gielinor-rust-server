"""Independent packet, transaction and durable-state assertions for native runs."""
import json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'docs/openrune/verification/independent/deep-combat'
def load(p):return json.loads(p.read_text(encoding='utf-8'))
def save(p,obj):p.write_text(json.dumps(obj,indent=2)+'\n',encoding='utf-8',newline='\n')
def decode(b):
    assert len(b)==16
    return {'source_component':int.from_bytes(b[:4],'big'),'source_slot':(b[4]<<8)|((b[5]-128)&255),
            'source_item':(b[10]<<8)|((b[11]-128)&255),'target_component':int.from_bytes(bytes([b[14],b[15],b[12],b[13]]),'big'),
            'target_slot':int.from_bytes(b[8:10],'little'),'target_item':((b[6]-128)&255)|(b[7]<<8)}
before=load(OUT/'produce/before.json');after=load(OUT/'produce/after.json')
manifest=load(OUT/'produce/manifest.json');assert manifest['status']=='PASS'
client=(OUT/'produce/client.log').read_text(encoding='utf-8');server=(OUT/'produce/server.log').read_text(encoding='utf-8')
packets=[]
for opcode,payload in re.findall(r'Native generated interface-target opcode=(\d+) payload=\[([^\]]+)\]',client):
    assert opcode=='27';packets.append(decode(bytes(int(x.strip(),16) for x in payload.split(','))))
expected=[(3,269,0,2436),(5,2440,7,269),(11,269,10,2442),(12,2436,15,111),(19,111,17,2440),(22,2442,23,111)]
assert len(packets)==6
for p,(a,b,c,d) in zip(packets,expected):assert p==dict(source_component=149<<16,source_slot=a,source_item=b,target_component=149<<16,target_slot=c,target_item=d)
make=re.findall(r'Native finished profile dispatch opcode=(\d+) payload=\[([^\]]+)\]',client)
assert len(make)==6 and all(op=='115' and bytes(int(x.strip(),16) for x in payload.split(','))==bytes([14,1,15,0,0,1]) for op,payload in make)
transitions=[json.loads(s.split(': ',1)[1]) for s in server.splitlines() if s.startswith('Combat mixing transition: ')]
assert len(transitions)==6
for i,t in enumerate(transitions):
    assert t['slots']==list(range(4*i,4*i+4))
    assert t['inputs']==[2436,2440,2442,269 if i<3 else 111]
    assert t['output']==12695 and t['secondary_saved'] is False
    assert t['committed_at']-t['queued_at']==2 and t['fine_after']-t['fine_before']==7500
    assert t['fine_before']==130344311+7500*i and t['fine_after']==130344311+7500*(i+1)
expected_inventory=[[93,i,12695,1,0] for i in range(6)]+[[93,i,1931,1,0] for i in range(24,27)]+[[93,27,995,1000,0]]
assert after['inventory']==expected_inventory,after['inventory']
for old,new in zip(before['stats'],after['stats']):
    assert old[0]==new[0]
    if old[0]==15:assert new==[15,99,99,130389311]
    else:assert new==old,(old,new)
assert after['integrity_check']=='ok' and after['character'][6] is None
assert after['character'][3]==before['character'][3], 'Unexpected run energy mutation'
report={'implementation_id':'OR-MIG-0010','implementation_sha':manifest['implementation_sha'],
        'normal_native_matrix':'PASS','authoritative_transaction_trace':'PASS','durable_production_save':'PASS',
        'decoded_packets':packets,'transitions':transitions,'checks':['six actual native inventory/menu selections','six native Make requests',
          'six authoritative two-tick exact-slot transactions','45000 fine XP; no other stat change','six outputs; all four inputs per operation consumed','1000 coins and fillers conserved','clean save acknowledgement; detached world_id; SQLite integrity ok'],
        'limitations':['Native errors/interrupt/modifier matrix not yet complete','No rendered frame captured; programmatic native interface/network acceptance only','Acknowledged-save durability is not crash atomicity']}
if (OUT/'relogin/after.json').exists():
    relog=load(OUT/'relogin/manifest.json');pre=load(OUT/'relogin/before.json');post=load(OUT/'relogin/after.json')
    assert relog['status']=='PASS'
    assert pre==after,'Database changed between production and owned restart'
    assert post['inventory']==after['inventory'] and post['stats']==after['stats']
    assert post['character'][3]==after['character'][3] and post['integrity_check']=='ok' and post['character'][6] is None
    for i in [0,1,2,3,6]:assert post['character'][i]==pre['character'][i]
    oldvars=json.loads(pre['character'][4]);newvars=json.loads(post['character'][4])
    changed={k:{'before':oldvars.get(k),'after':newvars.get(k)} for k in oldvars.keys()|newvars.keys() if oldvars.get(k)!=newvars.get(k)}
    assert set(changed)<={'300','3077','3079','65515'},changed
    assert newvars['65515']>=oldvars['65515'] and newvars['300']>=oldvars['300']
    text=(OUT/'relogin/client.log').read_text(encoding='utf-8')
    assert 'verify=true' in text and 'Native generated interface-target' not in text
    assert 'Combat mixing transition:' not in (OUT/'relogin/server.log').read_text(encoding='utf-8')
    report['real_restart_and_native_relogin']='PASS'
    report['relogin_no_replay_or_extra_reward']='PASS'
    report['session_varp_differences']={k:{**v,'classification':{'300':'special energy regeneration','3077':'packed wall-clock date fields','3079':'process-relative map clock','65515':'elapsed playtime'}[k]} for k,v in changed.items()}
    report['session_difference_basis']='Exact target session_vars.rs::Config::tick and server startup.rs lines127-135; owned target varp symbols map these IDs. All other persisted varps and character coordinates/plane/run energy/world_id unchanged.'
    report['character_changes_on_relogin']={str(i):{'before':a,'after':b} for i,(a,b) in enumerate(zip(pre['character'],post['character'])) if a!=b}
save(OUT/'independent-assessment.json',report)
print(json.dumps({k:v for k,v in report.items() if k not in ['decoded_packets','transitions','checks']},indent=2))
