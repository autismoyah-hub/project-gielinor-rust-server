"""Assess exact food runtime actions and real native restart capture independently."""
import hashlib,json,re,struct
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-food'
SHA='ae3d234c71764dd2364f8452fc1974a54a6783f0'
def load(p):return json.loads(p.read_text(encoding='utf-8'))
p=load(OUT/'produce/manifest.json');r=load(OUT/'relogin/manifest.json')
before=load(OUT/'produce/before.json');after=load(OUT/'produce/after.json');pre=load(OUT/'relogin/before.json');post=load(OUT/'relogin/after.json')
partial=p['status']=='FAIL'
for m in [p,r]:
    assert m['tested_sha']==m['implementation_sha']==SHA and m['campaign_kind']=='historical'
    assert m['server_exit']==0 and not m['forced_shutdown']
    assert m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged']
assert r['status']=='PASS' and r['client_exit']==0
assert p['client_exit']==(1 if partial else 0)
expected_inventory=[[93,1,1891,1,0],[93,4,2289,1,0],[93,27,995,1000,0]]
if partial:expected_inventory=[[93,1,1891,1,0],[93,3,1893,1,0],[93,4,2289,1,0],[93,5,2289,1,0],[93,27,995,1000,0]]
assert after['inventory']==post['inventory']==expected_inventory
assert pre==after and after['integrity_check']==post['integrity_check']=='ok'
for initial,final in zip(before['stats'],after['stats']):
    assert initial[0]==final[0] and initial[2:]==final[2:]
    assert final[1]==(10 if initial[0]==3 else initial[1])
assert after['stats']==post['stats']
events=[json.loads(s) for s in (OUT/'produce/native-events.jsonl').read_text(encoding='utf-8').splitlines()]
expected=[(7,2142,-1),(3,1891,1893),(3,1893,1895),(3,1895,-1),(5,2289,2291),(5,2291,-1)]
if partial:expected=expected[:1]
assert [(e['slot'],e['food'],e['remainder']) for e in events]==expected
assert all(e['display_xp']==1154 for e in events)
log=(OUT/'produce/client.log').read_text(encoding='utf-8')
assert len(re.findall(r'Native interface operation 2: component=9764864, child=\d+, outbound=\[47\]',log))==(2 if partial else 6)
if partial:
    assert 'native food timeout' in log
    traces=[json.loads(s.split('Food transition: ',1)[1]) for s in (OUT/'produce/server.log').read_text(encoding='utf-8').splitlines() if s.startswith('Food transition: ')]
    assert len(traces)==1 and traces[0]['tick']==99 and traces[0]['slot']==3 and traces[0]['before']==[1891,1,0] and traces[0]['after']==[1893,1,0]
    assert (traces[0]['hp_before'],traces[0]['hp_after'],traces[0]['fine_xp_before'],traces[0]['fine_xp_after'])==(5,9,11540,11540)
# Capture is made by the hash-verified genuine native client after the server is
# restarted. It contains server packets only and stays private; publish decoded
# relevant state and its hash, not raw account/session metadata.
capture=ROOT/'work/native-food/relogin-private.capture';data=capture.read_bytes();assert data[:8]==b'R239CAP1'
offset=8;inventories={};stats={};packet_count=0
while offset<len(data):
    opcode,length=struct.unpack_from('>HI',data,offset);offset+=6
    payload=data[offset:offset+length];assert len(payload)==length;offset+=length;packet_count+=1
    if opcode==22:
        component,inv,capacity=struct.unpack_from('>iHH',payload);pos=8;items=[]
        for slot in range(capacity):
            count=(128-payload[pos])&255;pos+=1
            if count==255:
                b=payload[pos:pos+4];assert len(b)==4;count=int.from_bytes(bytes([b[1],b[0],b[3],b[2]]),'big');pos+=4
            item=int.from_bytes(payload[pos:pos+2],'little')-1;pos+=2
            if item>=0:items.append([inv,slot,item,count,0])
            else:assert count==0
        assert pos==len(payload);inventories[inv]=items
    elif opcode==46:
        assert len(payload)==7;skill=(payload[2]-128)&255
        xp=int.from_bytes(payload[5:7]+payload[3:5],'big');stats[skill]={'current':payload[1],'display_xp':xp}
assert inventories[93]==expected_inventory
for skill,current,base,fine_xp in post['stats']:
    assert stats[skill]=={'current':current,'display_xp':fine_xp//10},(skill,stats.get(skill),current,fine_xp)
relog=(OUT/'relogin/client.log').read_text(encoding='utf-8');assert 'Authenticated; local index' in relog and 'Failed ' not in relog
report={'implementation_id':'OR-MIG-0024','revision':3,'tested_sha':SHA,'campaign_kind':'historical','status':'PARTIAL' if partial else 'PASS',
 'scope':('Native meat consumption confirmed by client, and cake stage selected-slot commit confirmed by server trace and restarted native inventory capture. Six-action driver timed out at its strict intermediate-HP equality check; remaining four bites are untested. ' if partial else 'Six genuine native Eat actions completed. ')+ 'Acknowledged save, actual restart/native authentication with independently decoded durable inventory and displayed stats PASS. Full V2/V3 acceptance remains open.',
 'raw_production_driver_result':p['status'],'durable_native_relogin_subcheck':'PASS',
 'native_actions':events,'native_relogin_capture_sha256':hashlib.sha256(data).hexdigest(),'capture_packet_count':packet_count,
 'native_relogin_inventory':inventories[93],'native_relogin_stats':stats,
 'limitations':['No rendered-window or exact attack/eat cadence assertion; actions deliberately spaced.',
                 'No native malformed/stale packet, overheal, full-bag or crash/interruption campaign in this run.',
                 'Pinned native build/source correspondence relies on preserved manifest/attestation; no independent client rebuild claimed.']}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps({'tested_sha':SHA,'status':report['status'],'client_confirmed_actions':len(events),'durable_native_relogin':'PASS','relogin_packet_count':packet_count}))
