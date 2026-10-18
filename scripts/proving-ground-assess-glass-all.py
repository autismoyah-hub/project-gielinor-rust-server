"""Assess full native All completion/cadence and independently decoded recovery."""
import json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-glass-all/all-001';SHA='5ce50aede9164cdee8e4008257d0298ea77289e7';load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');expected=[x.copy() for x in initial['inventory']]
for row in expected:
    if row[0]==93 and 4<=row[1]<=26:row[2]=6667
assert after['inventory']==post['inventory']==expected and load('relogin/before.json')==after
for a,b,c in zip(initial['stats'],after['stats'],post['stats']):assert a[:3]==b[:3] and b==c and b[3]-a[3]==(48875 if a[0]==12 else 0)
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS' and m['client_exit']==m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown'];assert load(phase+'/after.json')['integrity_check']=='ok'
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'PG_CASE_PASS native All23 emptyFishbowls' in log and 'stale completed native All23 glass batch' in log
wire=re.findall(r'PG_WIRE_TX tick=(\d+) opcode=115 payload=\[(.*?)\]',log);assert len(wire)==1;b=bytes.fromhex(wire[0][1].replace(',',''));component=int.from_bytes(bytes([b[1],b[0],b[3],b[2]]),'big');assert component==((270<<16)|19) and int.from_bytes(b[4:],'big')==23
credits=[];seen=set()
for tick,hexes in re.findall(r'PG_RX tick=(\d+) opcode=18 payload=\[(.*?)\]',log):
    b=bytes.fromhex(hexes.replace(',',''));inv=int.from_bytes(b[4:6],'big');pos=6
    while pos<len(b):
        slot=b[pos];pos+=1
        if slot>=128:slot=((slot<<8)|b[pos])-32768;pos+=1
        item=int.from_bytes(b[pos:pos+2],'big')-1;pos+=2;count=0
        if item>=0:
            count=b[pos];pos+=1
            if count==255:count=int.from_bytes(b[pos:pos+4],'big');pos+=4
        if inv==93 and item==6667 and slot not in seen:assert count==1;credits.append({'slot':slot,'tick':int(tick)});seen.add(slot)
    assert pos==len(b)
assert [r['slot'] for r in credits]==[26,*range(4,26)]
sequence=[(int(t),int(d)) for t,d in re.findall(r'PG_SEQUENCE tick=(\d+) sequence=884 delay=(\d+)',log)];assert len(sequence)==23 and sequence[0][1]==0 and credits[0]['tick']-sequence[0][0]==2
intervals=[credits[i]['tick']-credits[i-1]['tick'] for i in range(1,23)];assert all(x==3 for x in intervals),intervals
invs={};stats={}
for tick,op,hexes in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',(OUT/'relogin/client.log').read_text(encoding='utf-8')):
    b=bytes.fromhex(hexes.replace(',',''));op=int(op)
    if op==46:stats[(b[2]-128)&255]=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'))
    if op==22:
        inv=int.from_bytes(b[4:6],'big');size=int.from_bytes(b[6:8],'big');pos=8;items=[]
        for slot in range(size):
            count=(128-b[pos])&255;pos+=1
            if count==255:c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
            item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2
            if item>=0:items.append([inv,slot,item,count,0])
        assert pos==len(b);invs[inv]=items
assert invs[93]==expected
for skill,current,base,fine in post['stats']:assert stats[skill]==(current,fine//10)
report={'implementation_id':'OR-MIG-0062','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','native_count':23,'fine_xp_award':48875,'selected_first_slot':26,'first_received_clocks':2,'repeat_received_clocks':intervals,'scope':'Genuine native All selects exact available23and transport-submits component270:19/count23. Full28bag remains full; selected26then captured4through25become23emptyFishbowls6667, both pipes/coins/junk retained. First884animation-to-credit2received clocks then22three-clock intervals independently observed. Stale actual response does not repeat. Acknowledged save, actual restart and independently decoded native full inventory/Stats confirm23outputs/exact48875fineXP without replay.','limitations':['Native X remains blocked byPG-INFRA-009; this run deliberately avoids keyboard custom quantity.','In-flight batch logout/crash recovery and original-executable/rendered/audio parity are separate.']}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
