"""Decode actual emitted Use-on-item packets independently of driver branches."""
import json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-gem-effects/effects-001'
log=(OUT/'produce/client.log').read_text(encoding='utf-8')
packets=re.findall(r'PG_TX label="native Use target" tick=(\d+) opcode=27 payload=\[(.*?)\]',log)
assert len(packets)==15
rows=[]
for j,(tick,hexes) in enumerate(packets[1:]):
    p=bytes.fromhex(hexes.replace(',',''));assert len(p)==16
    beadd=lambda b:(b[0]<<8)|((b[1]-128)&255)
    leadd=lambda b:((b[0]-128)&255)|(b[1]<<8)
    selected_slot=beadd(p[4:6]);target_id=leadd(p[6:8]);target_slot=int.from_bytes(p[8:10],'little');selected_id=beadd(p[10:12])
    i=j%7;second=j//7;raw=[1623,1621,1619,1617,1631,6571,19496][i];slot=4+3*i+second
    reverse=i%2==1 if not second else i%2==0
    actual=(selected_slot,selected_id,target_slot,target_id)
    assert actual==((slot,raw,0,1755) if reverse else (0,1755,slot,raw)),(j,actual)
    rows.append({'selected_slot':selected_slot,'selected_item':selected_id,'target_slot':target_slot,'target_item':target_id,'raw_slot':slot,'reverse':reverse,'tick':int(tick)})
r={'implementation_id':'OR-MIG-0056','tested_sha':'5743cbe1c7499e2a7476c721d25dcba194666042','campaign_kind':'historical','status':'PASS','scope':'All14 successful native Use27 packets independently decoded: each of seven profiles appears once chisel-to-raw and once raw-to-chisel, with exact actual selected/target slot and item IDs. Initial cancelled request excluded from14 successful credits.','packets':rows}
(OUT/'direction-assessment.json').write_text(json.dumps(r,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(r))
