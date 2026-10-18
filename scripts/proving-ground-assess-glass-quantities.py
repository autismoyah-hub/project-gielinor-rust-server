"""Retain passing1/5/10native subchecks without changing the raw X failure."""
import json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-glass-quantities/quantities-003';SHA='5ce50aede9164cdee8e4008257d0298ea77289e7';load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');expected=[x.copy() for x in initial['inventory']]
products={4:1919,26:4527,**{s:4527 for s in range(5,9)},**{s:4525 for s in range(9,19)}}
for row in expected:
    if row[0]==93 and row[1] in products:row[2]=products[row[1]]
assert after['inventory']==post['inventory']==expected and len(products)==16 and load('relogin/before.json')==after
for a,b,c in zip(initial['stats'],after['stats'],post['stats']):assert a[:3]==b[:3] and b==c and b[3]-a[3]==(18125 if a[0]==12 else 0)
p=load('produce/manifest.json');r=load('relogin/manifest.json')
assert p['status']=='FAIL' and p['client_exit']==101 and p['server_exit']==0
assert r['status']=='PASS' and r['client_exit']==r['server_exit']==0
for m in [p,r]:assert m['tested_sha']==m['implementation_sha']==SHA and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown']
assert after['integrity_check']==post['integrity_check']=='ok'
native_invs={};native_stats={}
for tick,op,hexes in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',(OUT/'relogin/client.log').read_text(encoding='utf-8')):
    b=bytes.fromhex(hexes.replace(',',''));op=int(op)
    if op==46:native_stats[(b[2]-128)&255]=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'))
    if op==22:
        inv=int.from_bytes(b[4:6],'big');size=int.from_bytes(b[6:8],'big');pos=8;items=[]
        for slot in range(size):
            count=(128-b[pos])&255;pos+=1
            if count==255:c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
            item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2
            if item>=0:items.append([inv,slot,item,count,0])
        assert pos==len(b);native_invs[inv]=items
assert native_invs[93]==expected
for skill,current,base,fine in post['stats']:assert native_stats[skill]==(current,fine//10)
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'native requested count differs from actual control' in log and 'left: 1' in log and 'right: 3' in log
for label,n in [('1',1),('5',5),('10',10)]:
    frames=re.findall(r'PG_TX label="native glass quantity'+label+r' Make" tick=(\d+) opcode=115 payload=\[(.*?)\]',log);assert len(frames)==1
    b=bytes.fromhex(frames[0][1].replace(',',''));assert len(b)==6 and int.from_bytes(b[4:],'big')==n
    assert 'PG_GLASS_BATCH_COMMITTED label='+label+' count='+str(n)+' ' in log
labels={};names=[('Beer glass',1),('Empty candle lantern',4),('Empty oil lamp',12),('Vial',33),('Empty fishbowl',42),('Unpowered orb',46),('Lantern lens',49),('Empty light orb',87)]
for line in log.splitlines():
    m=re.fullmatch(r'PG_GLASS_WIDGET id=\d+ logical=(\d+) text=("(?:[^"\\]|\\.)*") data=("(?:[^"\\]|\\.)*")',line)
    if m:
        logical=int(m[1]);text=re.sub('<[^>]*>','',json.loads(m[3]))
        if logical>>16==270 and 15<=logical&65535<=22:labels[(logical&65535)-15]=text
assert labels=={i:f'{name} (level {level})' for i,(name,level) in enumerate(names)},labels
credits=[];seen={}
for tick,hexes in re.findall(r'PG_RX tick=(\d+) opcode=18 payload=\[(.*?)\]',log):
    b=bytes.fromhex(hexes.replace(',',''));inv=int.from_bytes(b[4:6],'big');pos=6
    while pos<len(b):
        slot=b[pos];pos+=1
        if slot>=128:slot=((slot<<8)|b[pos])-32768;pos+=1
        item=int.from_bytes(b[pos:pos+2],'big')-1;pos+=2;count=0
        if item>=0:
            count=b[pos];pos+=1
            if count==255:count=int.from_bytes(b[pos:pos+4],'big');pos+=4
        if inv==93 and slot in products and item==products[slot] and seen.get(slot)!=item:assert count==1;credits.append({'slot':slot,'output':item,'tick':int(tick)})
        if inv==93:seen[slot]=item
    assert pos==len(b)
assert len(credits)==16
sequences=[(int(t),int(delay)) for t,delay in re.findall(r'PG_SEQUENCE tick=(\d+) sequence=884 delay=(\d+)',log)];assert len(sequences)==16
offset=0;cadence=[]
for n in [1,5,10]:
    group=credits[offset:offset+n];assert group[0]['tick']-sequences[offset][0]==2 and sequences[offset][1]==0
    intervals=[group[i]['tick']-group[i-1]['tick'] for i in range(1,n)];assert all(x==3 for x in intervals)
    cadence.append({'count':n,'first_received_clocks':2,'repeat_received_clocks':intervals,'selected_first_slot':group[0]['slot']});offset+=n
report={'implementation_id':'OR-MIG-0062','tested_sha':SHA,'campaign_kind':'historical','status':'PASS_SUBCHECKS','raw_produce_status':'FAIL','completed_native_controls':[1,5,10],'completed_products':16,'fine_xp_award':18125,'native_option_labels':labels,'batch_cadence':cadence,'scope':'Genuine native1/5/10controls emit actualResume115counts1/5/10and commit exact selected-first full-bag outputs with retained pipes/coins/unrelated slots. All8real dynamic option labels/levels independently decoded from native widget data_text. First credit is2received clocks after884animation, then every3received clocks. Acknowledged save and actual restart/native relogin preserve all16products/18125fineXP and seven remaining raw glass, with no replay.','failure_preserved':'Raw producer FAIL at X remains unchanged. Native menu offered Other quantity, which the verifier filtered out, so X was not correctly invoked. Generated count1Make was caught before the next transport poll; saved state contains only the16completed products. X and All are assigned to a fresh remaining-controls campaign; completed1/5/10state is not reset or replayed.','limitations':['No X/All acceptance or interrupted in-flight batch lifecycle is inferred here. No original-executable/rendered/audio claim.']}
(OUT/'independent-subchecks.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
