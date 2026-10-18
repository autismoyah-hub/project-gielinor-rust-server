"""Assess immutable native pouch subchecks/restart, retaining failed bank attempts."""
import argparse,json,re
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('run');a=p.parse_args();assert re.fullmatch('pouch-[0-9]{3}',a.run)
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-pouch'/a.run;SHA='7fc1306bb5c01540038ce5ff175b577294ade432';load=lambda s:json.loads((OUT/s).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');m=load('produce/manifest.json');banked=m['status']=='PASS'
expected=[r.copy() for r in initial['inventory'] if r[:2]!=[93,4]]
next(r for r in expected if r[:2]==[93,27])[3]=1009
if banked:expected=[r for r in expected if r[:2]!=[93,27]]+[[95,0,995,1009,0]]
assert after['inventory']==post['inventory']==expected and load('relogin/before.json')==after
assert initial['stats']==after['stats']==post['stats']
for phase in ['produce','relogin']:
    manifest=load(phase+'/manifest.json');assert manifest['tested_sha']==manifest['implementation_sha']==SHA and manifest['server_exit']==0 and manifest['save_acknowledgement'] and manifest['graceful_shutdown'] and manifest['resource_hashes_unchanged'] and not manifest['forced_shutdown']
    assert load(phase+'/after.json')['integrity_check']=='ok'
    if phase=='relogin' or banked:assert manifest['status']=='PASS' and manifest['client_exit']==0
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'PG_CASE_PASS native pouch Open-one/Open-all' in log and 'stale selected pouch slot4with duplicate slot5intact' in log
for op,label in [(3,'Open-one'),(2,'Open-all')]:
    packet=re.search('PG_TX label="native selected pouch '+label+r'" tick=\d+ opcode=47 payload=\[(.*?)\]',log);assert packet
    b=bytes.fromhex(packet[1].replace(',',''));assert b==bytes.fromhex('00950000000457f9')+bytes([op])
if banked:assert 'PG_POUCH_BANK_REFUSAL text="A magical force prevents you from banking this item!"' in log and 'PG_CASE_PASS native no_bank Coinpouch deposit refusal' in log
else:assert m['status']=='FAIL' and m['client_exit']==101 and 'PG_BANK_MENU slot=27 option="Deposit-All"' in log and 'assertion `left == right` failed' in log
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
assert invs[93]==[r for r in expected if r[0]==93]
for skill,current,base,fine in post['stats']:assert stats[skill]==(current,fine//10)
report={'implementation_id':'OR-MIG-0054','tested_sha':SHA,'campaign_kind':'historical','status':'PASS' if banked else 'PASS_SUBCHECKS','raw_produce_status':m['status'],'scope':'Genuine native selected Open-one/op3 and Open-all/op2 exchange3pouches for9coins; duplicate stack2retained, stale selected slot replay refuses, all Stats unchanged. Acknowledged save, actual restart and independently decoded full native inventory/Stats preserve exact resulting state without replay.'+(' Native scripted1009coin bank-deposit positive control and retargeted actual-frame restricted pouch refusal with specific no_bank message also passed; bank1009coins persist through restart SQL.' if banked else ' Later bank harness immediate-packet assertion failed before deposit submission; bank acceptance remains open as PG-INFRA-010.'),'limitations':['NPC Bank operation cache-resolved; no rendered hit-test claim.','No acquisition/cap/diary/outfit/death/trade or original executable acceptance inferred.']}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
