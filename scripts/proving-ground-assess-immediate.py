"""Independent SQL and received-packet oracles for exact-SHA immediate crafting."""
import argparse,json,re,hashlib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('kind',choices=['staff','limestone','certainty']);a=p.parse_args()
ROOT=Path(__file__).resolve().parents[1];SHA='4f4aacb2929e8ad775d091527e2d80070100ee69';OUT=ROOT/'docs/openrune/verification/independent'/('deep-immediate-'+a.kind)/'native-001'
load=lambda name:json.loads((OUT/name).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');expected=[r.copy() for r in initial['inventory']];staff=a.kind=='staff';n=3 if staff else 12;tool,raw,seq=(946,771,10937) if staff else (1755,3211,1309)
award=0;outputs=[]
for slot in range(4,4+n):
    actual=next(r for r in after['inventory'] if r[:2]==[93,slot]);assert actual[3:]==[1,0]
    assert actual[2] in ([772] if staff else [3420] if a.kind=='certainty' else [3420,968])
    next(r for r in expected if r[:2]==[93,slot])[2]=actual[2];outputs.append(actual[2]);award+=0 if staff else 300 if actual[2]==3420 else 75
assert after['inventory']==post['inventory']==expected and load('relogin/before.json')==after
assert initial['character'][3]==after['character'][3]==post['character'][3]
for x,y,z in zip(initial['stats'],after['stats'],post['stats']):
    assert x[0]==y[0] and y==z and y[3]-x[3]==(award if x[0]==12 else 0)
    if x[0]!=12 or staff:assert x==y
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS' and m['client_exit']==m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown'];assert load(phase+'/after.json')['integrity_check']=='ok'
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'PG_CASE_PASS native immediate kind=immediate-'+a.kind in log
uses=re.findall(r'PG_WIRE_TX tick=(\d+) opcode=27 payload=\[(.*?)\]',log);assert len(uses)==n
for index,(_,hexes) in enumerate(uses):
    b=bytes.fromhex(hexes.replace(',',''));assert len(b)==16;beadd=lambda v:(v[0]<<8)|((v[1]-128)&255);leadd=lambda v:((v[0]-128)&255)|(v[1]<<8)
    pair=(beadd(b[4:6]),beadd(b[10:12]),int.from_bytes(b[8:10],'little'),leadd(b[6:8]));assert pair==((0,tool,4+index,raw) if index%2==0 else (4+index,raw,0,tool))
assert len(re.findall('PG_REPLAY label="stale immediate selected held pair',log))==n
assert not re.search(r'PG_WIRE_TX tick=\d+ opcode=115 ',log)
effects=[];latest=None;seen=set()
for line in log.splitlines():
    s=re.fullmatch(r'PG_SEQUENCE tick=(\d+) sequence='+str(seq)+r' delay=(\d+)',line)
    if s:latest=(int(s[1]),int(s[2]))
    rx=re.fullmatch(r'PG_RX tick=(\d+) opcode=18 payload=\[(.*)\]',line)
    if not rx:continue
    b=bytes.fromhex(rx[2].replace(',',''));inv=int.from_bytes(b[4:6],'big');pos=6
    while pos<len(b):
        slot=b[pos];pos+=1
        if slot>=128:slot=((slot<<8)|b[pos])-32768;pos+=1
        item=int.from_bytes(b[pos:pos+2],'big')-1;pos+=2;count=0
        if item>=0:
            count=b[pos];pos+=1
            if count==255:count=int.from_bytes(b[pos:pos+4],'big');pos+=4
        if inv==93 and 4<=slot<4+n and item==outputs[slot-4] and slot not in seen:
            assert count==1 and latest and latest==(int(rx[1]),0);effects.append({'slot':slot,'sequence':seq,'sequence_and_credit_tick':int(rx[1]),'output':item});seen.add(slot)
    assert pos==len(b)
assert len(effects)==n
messages=re.findall(r'PG_IMMEDIATE_MESSAGE kind=immediate-'+a.kind+r' index=(\d+) tick=\d+ message="(.*?)"',log)
if not staff:
    for index,out in enumerate(outputs):assert (str(index),'You cut the limestone brick.' if out==3420 else 'You fail to cut the limestone into a brick.') in messages
invs={};stats={}
for _,op,hexes in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',(OUT/'relogin/client.log').read_text(encoding='utf-8')):
    b=bytes.fromhex(hexes.replace(',',''));op=int(op)
    if op==46:stats[(b[2]-128)&255]=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'))
    if op!=22:continue
    inv=int.from_bytes(b[4:6],'big');size=int.from_bytes(b[6:8],'big');pos=8;items=[]
    for slot in range(size):
        count=(128-b[pos])&255;pos+=1
        if count==255:c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
        item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2
        if item>=0:items.append([inv,slot,item,count,0])
    assert pos==len(b);invs[inv]=items
assert invs[93]==expected
for skill,current,base,fine in post['stats']:assert stats[skill]==(current,fine//10)
report={'implementation_id':'OR-MIG-0065','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','kind':a.kind,'actions':n,'outputs':outputs,'fine_xp_award':award,'both_natural_outcomes_observed':3420 in outputs and 968 in outputs,'messages_observed':len(messages),'effects':effects,'evidence_sha256':{name:hashlib.sha256((OUT/name).read_bytes()).hexdigest() for name in ['produce/client.log','produce/manifest.json','relogin/client.log','relogin/manifest.json']},'scope':'Real native held Use both directions, full28 selected-only outputs and exact fineXP; no production Resume115; captured stale requests refuse with matching raw/tool duplicates retained. Native sequence and credit share received clock with zero sequence delay. Acknowledged save, process restart and fully independently decoded native inventory/Stats retain exact outputs/XP without replay.','limitations':['Limestone natural outcomes witness branches, not a statistical proof of the probability curve.','Immediate-before-World-tick is established by separate public World tests; native receive clocks do not prove network latency or rendered timing.','Staff original frozen controller did not assert received flavor message; contract failure-message requirement is exercised by limestone.','No original executable/audio or rendered animation frames compared.']}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
