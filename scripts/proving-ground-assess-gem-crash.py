"""Assess bounded unacknowledged-cut crash recovery without modifying state."""
import json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-gem-crash/crash-001';SHA='5743cbe1c7499e2a7476c721d25dcba194666042'
load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
initial=load('initial-state.json');durable=load('produce/after.json');before=load('relogin/before.json');after=load('relogin/after.json')
for state in [durable,before,after]:
    assert state['inventory']==initial['inventory'] and state['stats']==initial['stats']
    assert state['integrity_check']=='ok'
assert len(initial['inventory'])==28
p=load('produce/manifest.json');r=load('relogin/manifest.json')
for m in [p,r]:assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS' and m['resource_hashes_unchanged'] and not m['forced_shutdown']
assert p['intentional_crash_injected'] and p['client_exit']==0 and p['server_exit']!=0 and not p['save_acknowledgement'] and not p['graceful_shutdown']
assert r['client_exit']==r['server_exit']==0 and r['save_acknowledgement'] and r['graceful_shutdown']
produce=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'PG_CRASH_READY gem unsaved selected Sapphire result observed' in produce and 'PG_EXPECTED_CRASH_DISCONNECT' in produce
# Independent raw partial-inventory proof of the in-memory result before killing.
seen=False
for tick,hexes in re.findall(r'PG_RX tick=(\d+) opcode=18 payload=\[(.*?)\]',produce):
    b=bytes.fromhex(hexes.replace(',',''));pos=6
    while pos<len(b):
        slot=b[pos];pos+=1
        if slot>=128:slot=((slot<<8)|b[pos])-32768;pos+=1
        item=int.from_bytes(b[pos:pos+2],'big')-1;pos+=2;count=0
        if item>=0:
            count=b[pos];pos+=1
            if count==255:count=int.from_bytes(b[pos:pos+4],'big');pos+=4
        if int.from_bytes(b[4:6],'big')==93 and slot==4 and item==1607 and count==1:seen=True
    assert pos==len(b)
assert seen
native_inv={};stats={};log=(OUT/'relogin/client.log').read_text(encoding='utf-8');assert 'PG_CASE_PASS native relogin observation' in log
for tick,op,hexes in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',log):
    b=bytes.fromhex(hexes.replace(',',''));op=int(op)
    if op==46:stats[(b[2]-128)&255]=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'))
    if op==22:
        inv=int.from_bytes(b[4:6],'big');size=int.from_bytes(b[6:8],'big');pos=8;items=[]
        for slot in range(size):
            count=(128-b[pos])&255;pos+=1
            if count==255:c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
            item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2
            if item>=0:items.append([inv,slot,item,count,0])
        assert pos==len(b);native_inv[inv]=items
assert native_inv[93]==initial['inventory']
for skill,current,base,fine in initial['stats']:assert stats[skill]==(current,fine//10)
report={'implementation_id':'OR-MIG-0056','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','scope':'Actual native selected Sapphire cut observed in raw inventory18, then only the owned server was abruptly terminated before any logout/save acknowledgement. Same database passed integrity, actual server restart and native relogin: complete initial28-slot bag, both chisels, selected raw and all exact fine Stats restored together; no partial debit, duplicate cut/XP or automatic replay after6 observed clocks. Subsequent acknowledged logout/save succeeded.','limitations':['This is the unacknowledged in-memory completion crash window, not a kill inside an SQLite commit or a guarantee of retaining unacknowledged progress. Acknowledged output durability is independently tested by cuts-5x001.']}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
