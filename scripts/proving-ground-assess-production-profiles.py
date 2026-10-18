"""Independent native single-choice production/Close/restart/effect assessment."""
import argparse,json,re
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('kind',choices=['gems','shells']);a=p.parse_args()
ROOT=Path(__file__).resolve().parents[1];SHA='143dab04f28c3daed844da0af33d6af5edcd962d';OUT=ROOT/'docs/openrune/verification/independent'/('deep-production-'+a.kind)/'profiles-001';load=lambda s:json.loads((OUT/s).read_text(encoding='utf-8'))
profiles=[(1623,1607,2500),(1621,1605,3375),(1619,1603,4250),(1617,1601,5375),(1631,1615,6875),(6571,6573,8375),(19496,19493,2500)]+[(3345+2*i,3327+2*i,1625) for i in range(9)]
indices=list(range(7) if a.kind=='gems' else range(7,16));initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');expected=[r.copy() for r in initial['inventory']];award=sum(profiles[i][2] for i in indices)
for i in indices:next(r for r in expected if r[:2]==[93,4+i])[2]=profiles[i][1]
assert after['inventory']==post['inventory']==expected and load('relogin/before.json')==after
for x,y,z in zip(initial['stats'],after['stats'],post['stats']):assert x[:3]==y[:3] and y==z and y[3]-x[3]==(award if x[0]==12 else 0)
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS' and m['client_exit']==m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown'];assert load(phase+'/after.json')['integrity_check']=='ok'
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'PG_CASE_PASS native production Make/Close and stale continuation refused before award' in log and 'PG_CASE_PASS registered native single-choice profiles shells='+str(a.kind=='shells').lower() in log
assert re.search(r'PG_TX label="native production Close before first award" tick=\d+ opcode=95 payload=\[\]',log)
choices=re.findall(r'PG_TX label="native registered single-choice Make-one" tick=(\d+) opcode=115 payload=\[(.*?)\]',log);assert len(choices)==len(indices)
for _,hexes in choices:assert bytes.fromhex(hexes.replace(',',''))==bytes.fromhex('0e010f000001')
uses=re.findall(r'PG_TX label="native Use target" tick=\d+ opcode=27 payload=\[(.*?)\]',log);assert len(uses)==1+len(indices)
for index,hexes in zip(indices,uses[1:]):
    b=bytes.fromhex(hexes.replace(',',''));assert len(b)==16;beadd=lambda v:(v[0]<<8)|((v[1]-128)&255);leadd=lambda v:((v[0]-128)&255)|(v[1]<<8)
    pair=(beadd(b[4:6]),beadd(b[10:12]),int.from_bytes(b[8:10],'little'),leadd(b[6:8]));raw=profiles[index][0];assert pair==((0,1755,4+index,raw) if index%2==0 else (4+index,raw,0,1755))
sequences=[];effects=[];seen=set();expected_sequence=8833 if a.kind=='gems' else 1280;expected_delay=1 if a.kind=='gems' else 2
for line in log.splitlines():
    s=re.fullmatch(r'PG_SEQUENCE tick=(\d+) sequence='+str(expected_sequence)+r' delay=(\d+)',line)
    if s:sequences.append((int(s[1]),int(s[2])))
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
        index=slot-4
        if inv==93 and index in indices and item==profiles[index][1] and slot not in seen:
            assert count==1 and sequences;tick,delay=sequences[-1];assert delay==0 and int(rx[1])-tick==expected_delay;effects.append({'index':index,'slot':slot,'sequence':expected_sequence,'sequence_tick':tick,'credit_tick':int(rx[1]),'received_clocks':expected_delay});seen.add(slot)
    assert pos==len(b)
assert len(effects)==len(indices)
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
report={'implementation_id':'OR-MIG-0063','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','profiles':len(indices),'fine_xp_award':award,'effects':effects,'scope':'Actual startup-registered '+a.kind+' single-choice quantity UI: exact cache-name/level labels, native Use both directions across profiles, one-use Make115/count1, full-bag selected credits and fineXP. Genuine Make/Close95before award and stale response refuse; completed responses do not repeat. Real acknowledged save/restart plus independently decoded native full inventory/Stats preserve all outputs/XP.','limitations':['This fresh scope does not inherit manual56/57acceptance.','Native built-in batch quantities/repeated cadence/custom X, interrupted batch persistence and reference aggregation remain separate.','No rendered/audio or original executable comparison inferred.']}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
