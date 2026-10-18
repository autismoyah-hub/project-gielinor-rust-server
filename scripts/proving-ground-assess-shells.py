"""Correlate shell native wire directions, effects and actual durable restart."""
import json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-shells/shells-001';SHA='4c6b73cfcd3b35ce52aa79efd9d2a99464939445'
load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');expected=[r.copy() for r in initial['inventory']]
for row in expected:
    if row[0]==93 and 4<=row[1]<=21:row[2]=3327+2*((row[1]-4)//2)
assert len(expected)==28 and after['inventory']==post['inventory']==expected
assert load('relogin/before.json')==after
for a,b,c in zip(initial['stats'],after['stats'],post['stats']):assert a[:3]==b[:3] and b==c and b[3]-a[3]==(29250 if a[0]==12 else 0)
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS'
    assert m['client_exit']==m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown']
    assert load(phase+'/after.json')['integrity_check']=='ok'
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'PG_CASE_PASS eighteen native shell carvings' in log and 'PG_CASE_PASS native shell pending cancellation' in log
assert log.count('stale shell pair with matching raw and chisel duplicates retained')==9
packets=re.findall(r'PG_TX label="native Use target" tick=(\d+) opcode=27 payload=\[(.*?)\]',log);assert len(packets)==19
directions=[]
for j,(tick,hexes) in enumerate(packets[1:]):
    b=bytes.fromhex(hexes.replace(',',''));assert len(b)==16
    beadd=lambda p:(p[0]<<8)|((p[1]-128)&255)
    leadd=lambda p:((p[0]-128)&255)|(p[1]<<8)
    actual=(beadd(b[4:6]),beadd(b[10:12]),int.from_bytes(b[8:10],'little'),leadd(b[6:8]))
    i=j%9;second=j//9;slot=4+2*i+second;raw=3345+2*i;reverse=(i+second)%2==1
    assert actual==((slot,raw,0,1755) if reverse else (0,1755,slot,raw))
    directions.append({'slot':slot,'raw_id':raw,'reverse':reverse,'tick':int(tick)})
sequences=[];credits=[];items={}
for line in log.splitlines():
    m=re.fullmatch(r'PG_SEQUENCE tick=(\d+) sequence=1280 delay=(\d+)',line)
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
        if inv==93 and 4<=slot<=21 and item==3327+2*((slot-4)//2) and items.get(slot)!=item:
            assert count==1 and sequences;sequence=sequences[-1]
            credits.append({'slot':slot,'output':item,'animation':1280,'animation_delay':sequence['delay'],'animation_tick':sequence['tick'],'credit_tick':int(m[1]),'received_clock_delta':int(m[1])-sequence['tick']})
        if inv==93:items[slot]=item
    assert pos==len(b)
assert len(credits)==18 and len({c['slot'] for c in credits})==18 and all(c['received_clock_delta']==2 and c['animation_delay']==0 for c in credits),credits
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
report={'implementation_id':'OR-MIG-0057','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','native_profiles':9,'native_cuts':18,'fine_xp_award':29250,'wire_directions':directions,'effect_observations':credits,'scope':'All nine exact round/pointed/bark shell IDs carved in both actual native Use27 directions. Each decoded animation1280(delay0) precedes its raw selected inventory18 credit by two received server-clock boundaries. Full28bag, both chisels and unrelated slots conserved, nine actual stale-pair replays refuse substitution, native pre-completion movement cancels without award. All18helmets and29250fineXP survive acknowledged save, actual restart and independently decoded native full inventory/Stats, with no reconnect replay.','limitations':['Decoded native effects/relative ordered clocks are observed; no rendered/audio or absolute World/client tick equality claim.','No process-crash scenario is included in this assessment; menus/repeats/sound/acquisition/equipment and source executable parity remain excluded or separate.']}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
