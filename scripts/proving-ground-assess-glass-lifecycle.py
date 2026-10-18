"""Independent exact-state assessment of real glass disconnect/crash recovery."""
import argparse,json,re
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('mode',choices=['interrupt','crash']);p.add_argument('--kind',choices=['glass','gem','shell'],default='glass');a=p.parse_args()
ROOT=Path(__file__).resolve().parents[1];SHA='5ce50aede9164cdee8e4008257d0298ea77289e7' if a.kind=='glass' else '143dab04f28c3daed844da0af33d6af5edcd962d'
output,unit=(1919,875) if a.kind=='glass' else (1607,2500) if a.kind=='gem' else (3327,1625)
OUT=ROOT/'docs/openrune/verification/independent'/(('deep-glass' if a.kind=='glass' else 'deep-production-'+a.kind)+'-'+a.mode)/(a.mode+'-001')
load=lambda s:json.loads((OUT/s).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json')
log=(OUT/'produce/client.log').read_text(encoding='utf-8')
observed=int(re.search((r'PG_GLASS_PARTIAL observed_products=' if a.kind=='glass' else r'PG_PRODUCTION_PARTIAL shells=(?:true|false) observed_products=')+r'(\d+)',log)[1]);assert 0<observed<23
if a.kind=='glass':
    assert log.count('PG_NEGATIVE_GLASS_RESUME component=')==2
    assert 'native Close after foreign/index refusal' in log
made=sum(x[3] for x in after['inventory'] if x[0]==93 and x[2]==output)
expected=[x.copy() for x in initial['inventory']]
if a.mode=='interrupt':
    assert observed<=made<23 and ('PG_DISCONNECT_PARTIAL glass' if a.kind=='glass' else 'PG_DISCONNECT_PARTIAL production') in log
    for slot in [26,*range(4,26)][:made]:next(x for x in expected if x[:2]==[93,slot])[2]=output
else:
    assert made==0 and 'PG_EXPECTED_CRASH_DISCONNECT' in log
assert after['inventory']==post['inventory']==expected and load('relogin/before.json')==after
for before,saved,recovered in zip(initial['stats'],after['stats'],post['stats']):
    assert before[:3]==saved[:3] and saved==recovered and saved[3]-before[3]==(unit*made if before[0]==12 else 0)
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS' and m['client_exit']==0 and m['resource_hashes_unchanged']
    assert load(phase+'/after.json')['integrity_check']=='ok'
    if a.mode=='crash' and phase=='produce':assert m['server_exit']!=0 and not m['save_acknowledgement'] and not m['graceful_shutdown']
    else:assert m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and not m['forced_shutdown']
# Independently decode actual native restart inventory and Stats, beyond SQL.
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
scope=('Actual native All23 Beer batch disconnected with work remaining; exact committed captured prefix and875fineXP per output saved. Real server restart and native inventory/Stats show remaining raws retained and no queued replay.' if a.mode=='interrupt' else 'Owned server abruptly terminated during actual native All23 Beer batch before save acknowledgement. Prior whole inventory/Stats restored; real restart/native packets show no partial debit, credit or queued replay. No mid-SQL-commit injection claimed.')
if a.kind!='glass':scope=('Actual native All23 '+a.kind+' production disconnected with captured work remaining; exact committed selected prefix and'+str(unit)+'fineXP per product saved. Real server restart and independently decoded native inventory/Stats retain remaining raws and cancel the queue without replay.' if a.mode=='interrupt' else 'Owned exact-SHA server abruptly terminated during actual native All23 gem production before save acknowledgement. Prior whole inventory/Stats restored; actual restart and native packets prove no partial debit, credit or queued replay. No mid-SQL-commit injection claimed.')
report={'implementation_id':'OR-MIG-0062' if a.kind=='glass' else 'OR-MIG-0063','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','native_observed_products':observed,'durably_saved_products':made,'fine_xp_award':unit*made,'scope':scope,'limitations':['Native quantity controls/reference acceptance remain separate.','No original executable, rendered or audio comparison inferred.']}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
