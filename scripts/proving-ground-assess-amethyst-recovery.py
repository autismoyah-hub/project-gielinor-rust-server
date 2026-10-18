"""Independent counted-stack partial save / owned unsaved recovery oracle."""
import argparse,hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='bc277dc3e7e15088323a4f048f57dc43185fb7c0'
p=argparse.ArgumentParser();p.add_argument('--feature',choices=['amethyst-interrupt','amethyst-crash'],required=True);a=p.parse_args();crash=a.feature=='amethyst-crash'
OUT=ROOT/'docs/openrune/verification/independent'/('deep-'+a.feature)/'native-001'
load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');m=load('produce/manifest.json');r=load('relogin/manifest.json')
assert initial['inventory']==[[93,s,1755 if s in [0,1] else 21347 if 4<=s<=26 else 995 if s==27 else 1931,1000 if s==27 else 1,0] for s in range(28)]
def expected(k):
    inv={row[1]:row[:] for row in initial['inventory']}
    if k:
        inv[26]=[93,26,21338,15*k,0]
        for slot in range(4,k+3):del inv[slot]
    stats=[row[:] for row in initial['stats']]
    for row in stats:
        if row[0]==12:assert row==[12,99,99,130344311];row[3]+=3000*k
    return [inv[s] for s in sorted(inv)],stats
assert m['tested_sha']==m['implementation_sha']==r['tested_sha']==r['implementation_sha']==SHA
assert m['status']=='PASS' and m['client_exit']==0 and m['resource_hashes_unchanged']
assert m['server_sha256']==r['server_sha256']==load('build.json')['binary_sha256'] and m['client_sha256']==r['client_sha256']==load('resources.json')['driver_sha256']
if crash:
    assert m['intentional_crash_injected'] and m['server_exit']!=0 and not m['save_acknowledgement'] and not m['graceful_shutdown'] and 'counted-amethyst prefix' in m['fault_scope']
    durable=0
else:
    assert m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and not m['forced_shutdown']
    tips=[row for row in after['inventory'] if row[2]==21338];assert len(tips)==1 and tips[0][1]==26 and tips[0][3]%15==0
    durable=tips[0][3]//15;assert 0<durable<23
assert r['status']=='PASS' and r['client_exit']==r['server_exit']==0 and r['save_acknowledgement'] and r['graceful_shutdown'] and not r['forced_shutdown'] and r['resource_hashes_unchanged']
inv,stats=expected(durable)
for state in [after,post]:assert state['inventory']==inv and state['stats']==stats and state['character'][3]==initial['character'][3] and state['integrity_check']=='ok'
assert load('relogin/before.json')==after
log=(OUT/'produce/client.log').read_text(encoding='utf-8')
prefix=re.findall(r'PG_AMETHYST_PREFIX observed_raws=(\d+) tips=(\d+) remaining_raws=(\d+) display_xp=(\d+) tick=(\d+)',log);assert len(prefix)==1
observed,tips,remaining,xp,tick=map(int,prefix[0]);assert 0<observed<23 and (tips,remaining,xp)==(15*observed,23-observed,(130344311+3000*observed)//10)
if crash:assert 'PG_CRASH_READY native counted amethyst unsaved prefix' in log and 'PG_EXPECTED_CRASH_DISCONNECT' in log
else:assert 'PG_DISCONNECT_PARTIAL amethyst captured All23' in log and durable>=observed
wire=[(int(op),bytes.fromhex(h.replace(',',''))) for op,h in re.findall(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log)]
assert sum(op==27 for op,b in wire)==1 and [b for op,b in wire if op==115]==[bytes([14,1,15,0,0,23])]
def full(b):
    inv_id=int.from_bytes(b[4:6],'big');size=int.from_bytes(b[6:8],'big');pos=8;items={}
    for slot in range(size):
        count=(128-b[pos])&255;pos+=1
        if count==255:c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
        item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2
        if item>=0:items[slot]=(item,count)
    assert pos==len(b);return inv_id,items
state={row[1]:(row[2],row[3]) for row in initial['inventory']};prefixes=[];xp_values=[]
for op,h in re.findall(r'PG_RX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log):
    b=bytes.fromhex(h.replace(',',''));op=int(op)
    if op==46 and (b[2]-128)&255==12:xp_values.append(int.from_bytes(b[5:7]+b[3:5],'big'))
    if op not in [18,22] or int.from_bytes(b[4:6],'big')!=93:continue
    if op==22:_,state=full(b)
    else:
        pos=6
        while pos<len(b):
            slot=b[pos];pos+=1
            if slot>=128:slot=((slot<<8)|b[pos])-32768;pos+=1
            item=int.from_bytes(b[pos:pos+2],'big')-1;pos+=2;count=0
            if item>=0:
                count=b[pos];pos+=1
                if count==255:count=int.from_bytes(b[pos:pos+4],'big');pos+=4
                state[slot]=(item,count)
            else:state.pop(slot,None)
        assert pos==len(b)
    output=[(slot,n) for slot,(item,n) in state.items() if item==21338]
    if output:assert len(output)==1 and output[0][0]==26 and output[0][1]%15==0
    k=output[0][1]//15 if output else 0;assert 0<=k<=23
    assert [[93,s,*state[s],0] for s in sorted(state)]==expected(k)[0];prefixes.append(k)
assert observed in prefixes and xp in xp_values and all(value in [13034431+300*k for k in range(24)] for value in xp_values)
invs={};native_stats={}
restart_log=(OUT/'relogin/client.log').read_text(encoding='utf-8');assert 'PG_NATIVE_NO_PENDING_SKILL_MULTI' in restart_log
for op,h in re.findall(r'PG_RX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',restart_log):
    b=bytes.fromhex(h.replace(',',''));op=int(op)
    if op==46:native_stats[(b[2]-128)&255]=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'))
    if op==22:i,items=full(b);invs[i]=[[i,s,*items[s],0] for s in sorted(items)]
assert invs[93]==inv
for skill,current,base,fine in stats:assert native_stats[skill]==(current,fine//10)
scope=('Observed unsaved native whole-tip prefix, owned server abrupt stop/no save acknowledgement, complete prior SQL/real native restart rollback without replay.' if crash else 'Observed native partial All23prefix and acknowledged partial save; actual completed durable prefix is independently reconstructed, remaining raw preserved and no production/dialog replay after process restart/native reconnect. Durable prefix need not equal earlier asynchronous observation.')
report={'implementation_id':'OR-MIG-0071','tested_sha':SHA,'campaign_kind':'historical','feature':a.feature,'status':'PASS','native_observed_prefix':observed,'native_max_received_prefix':max(prefixes),'durable_raw_credits':durable,'durable_tips':15*durable,'remaining_raws':23-durable,'fine_xp_award':3000*durable,'scope':scope+' Every received raw inventory packet is a complete valid selected-first/ascending captured prefix with whole15-tip credits; XP values are per-raw and whole SQL Stats/coins/chisels/energy survive.','limits':['No exact server commit count at physical disconnect is inferred from asynchronous client observations.','Unsaved recovery is not an injection inside SQL commit; no rendered/audio/original executable or currentHEAD acceptance.'],'evidence_sha256':{p:hashlib.sha256((OUT/p).read_bytes()).hexdigest() for p in ['initial-state.json','produce/client.log','produce/manifest.json','produce/after.json','relogin/client.log','relogin/manifest.json','relogin/after.json','frozen-observer/pg_observer.rs','frozen-observer/build.json']}}
destination=OUT/'independent-assessment.json';assert not destination.exists();destination.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
