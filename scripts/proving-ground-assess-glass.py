"""Assess eight native choices and real recovery; quantity/labels stay separate."""
import json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-glass/choices-001';SHA='5ce50aede9164cdee8e4008257d0298ea77289e7';load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
profiles=[(1919,875),(4527,950),(4525,1250),(229,1750),(6667,2125),(567,2625),(4542,2750),(10980,3500)]
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');expected=[x.copy() for x in initial['inventory']]
for i,(output,award) in enumerate(profiles):next(x for x in expected if x[0]==93 and x[1]==4+i)[2]=output
assert len(expected)==28 and after['inventory']==post['inventory']==expected and load('relogin/before.json')==after
for a,b,c in zip(initial['stats'],after['stats'],post['stats']):assert a[:3]==b[:3] and b==c and b[3]-a[3]==(15825 if a[0]==12 else 0)
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS' and m['client_exit']==m['server_exit']==0
    assert m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown'] and load(phase+'/after.json')['integrity_check']=='ok'
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'PG_CASE_PASS eight native glass choices' in log and 'PG_CASE_PASS native glass queued movement and genuine Close/stale continuation' in log
assert re.search(r'PG_TX label="genuine native glass Close" tick=\d+ opcode=95 payload=\[\]',log)
packets=re.findall(r'PG_TX label="native Use target" tick=(\d+) opcode=27 payload=\[(.*?)\]',log);assert len(packets)==10
for i,(tick,hexes) in enumerate(packets[2:]):
    b=bytes.fromhex(hexes.replace(',',''));assert len(b)==16
    beadd=lambda p:(p[0]<<8)|((p[1]-128)&255)
    leadd=lambda p:((p[0]-128)&255)|(p[1]<<8)
    actual=(beadd(b[4:6]),beadd(b[10:12]),int.from_bytes(b[8:10],'little'),leadd(b[6:8]));assert actual==((4+i,1775,0,1785) if i%2 else (0,1785,4+i,1775))
choices=re.findall(r'PG_TX label="native glass chosen one product" tick=(\d+) opcode=115 payload=\[(.*?)\]',log);assert len(choices)==8
for i,(tick,hexes) in enumerate(choices):
    b=bytes.fromhex(hexes.replace(',',''));component=int.from_bytes(bytes([b[1],b[0],b[3],b[2]]),'big');assert len(b)==6 and component==((270<<16)|(15+i)) and int.from_bytes(b[4:],'big')==1
assert log.count('stale glass quantity response after completed dialog')==8 and 'stale glass Make after genuine native Close' in log
sequences=[];credits=[];items={}
for line in log.splitlines():
    m=re.fullmatch(r'PG_SEQUENCE tick=(\d+) sequence=884 delay=(\d+)',line)
    if m:sequences.append({'tick':int(m[1]),'delay':int(m[2])})
    m=re.fullmatch(r'PG_RX tick=(\d+) opcode=18 payload=\[(.*)\]',line)
    if not m:continue
    b=bytes.fromhex(m[2].replace(',',''));inv=int.from_bytes(b[4:6],'big');pos=6
    while pos<len(b):
        slot=b[pos];pos+=1
        if slot>=128:slot=((slot<<8)|b[pos])-32768;pos+=1
        item=int.from_bytes(b[pos:pos+2],'big')-1;pos+=2;count=0
        if item>=0:
            count=b[pos];pos+=1
            if count==255:count=int.from_bytes(b[pos:pos+4],'big');pos+=4
        if inv==93 and 4<=slot<=11 and item==profiles[slot-4][0] and items.get(slot)!=item:
            assert count==1 and sequences;sequence=sequences[-1];credits.append({'slot':slot,'output':item,'animation':884,'animation_delay':sequence['delay'],'animation_tick':sequence['tick'],'credit_tick':int(m[1]),'received_clock_delta':int(m[1])-sequence['tick']})
        if inv==93:items[slot]=item
    assert pos==len(b)
assert len(credits)==8 and all(x['received_clock_delta']==2 and x['animation_delay']==0 for x in credits),credits
log=(OUT/'relogin/client.log').read_text(encoding='utf-8');assert 'PG_CASE_PASS native relogin observation' in log;invs={};stats={}
for tick,op,hexes in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',log):
    b=bytes.fromhex(hexes.replace(',',''));op=int(op)
    if op==46:stats[(b[2]-128)&255]=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'))
    if op==22:
        inv=int.from_bytes(b[4:6],'big');size=int.from_bytes(b[6:8],'big');pos=8;rows=[]
        for slot in range(size):
            count=(128-b[pos])&255;pos+=1
            if count==255:c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
            item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2
            if item>=0:rows.append([inv,slot,item,count,0])
        assert pos==len(b);invs[inv]=rows
assert invs[93]==expected
for skill,current,base,fine in post['stats']:assert stats[skill]==(current,fine//10)
report={'implementation_id':'OR-MIG-0062','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','native_choices':8,'fine_xp_award':15825,'effects':credits,'scope':'Actual native pipe/glass use in both directions, each of8real Resume115choice components with count1, correct distinct output and fineXP, full28bag and retained duplicate pipes/raws. Native pre-completion Walk cancels, genuine emptyClose95then stale actual Resume refuses without inventory/XP changes, fresh controls remain valid and8completed-dialog response replays do not repeat. Decoded884animation precedes each raw selected inventory18credit by2received server clocks. Acknowledged save/actual restart/native full inventory and Stats confirm all8outputs and exact15825fineXP without replay.','limitations':['This first native run logged header text, not dynamic choice data_text/hover labels. Exact labels and1/5/10/All/Xcontrols plus repeated batch cadence require the next fresh quantity campaign.','No compiled original Kotlin, rendered/audio, in-flight batch logout or process-crash acceptance inferred. Chemistry-specific PG123candidate atb11260e remains user-deferred; this is nonchemistry Close evidence at5ce50ae only.']}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
