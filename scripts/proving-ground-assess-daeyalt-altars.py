"""Independent native inventory/stat decoding for the remaining five Daeyalt altars."""
import argparse,hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='bd1294ab9e0ff850155b024364dc3a3a98a37bed'
p=argparse.ArgumentParser();p.add_argument('--feature',required=True);p.add_argument('--run-id',default='native-001');a=p.parse_args()
id,output,interval,award,x,z={'rune-air-daeyalt':(34760,556,11,50,2843,4833),'rune-water-daeyalt':(34762,555,19,60,2715,4835),'rune-earth-daeyalt':(34763,557,26,65,2657,4840),'rune-fire-daeyalt':(34764,554,35,70,2584,4837),'rune-body-daeyalt':(34765,559,46,75,2522,4839)}[a.feature]
OUT=ROOT/'docs/openrune/verification/independent'/('deep-'+a.feature)/a.run_id
load=lambda n:json.loads((OUT/n).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json')
assert load('produce/before.json')==initial and load('relogin/before.json')==after
base=[(24704 if s<7 else 1436 if s%2==0 else 7936,1)for s in range(28)]
assert [(v[2],v[3])for v in initial['inventory']if v[0]==93]==base
per=99//interval+1;first=base[:]
for s in range(7):first[s]=(-1,0)
first[0]=(output,7*per);final=[(output,28*per)]+[(-1,0)]*27
fine1=130344311+7*award*150*500//10000;fine2=fine1+21*award*5
expected=[[93,0,output,28*per,0]]+[v for v in initial['inventory']if v[0]!=93]
stats=[v[:]for v in initial['stats']];assert stats[20]==[20,99,99,130344311];stats[20][3]=fine2
assert after['inventory']==post['inventory']==expected and after['stats']==post['stats']==stats
for state in [initial,after,post]:
    assert state['integrity_check']=='ok' and state['xp_rate_hundreds']==500 and state['account_membership']is True
    assert state['character'][:4]==initial['character'][:4]
build=load('build.json');frozen=load('frozen-observer/build.json');assert build['exit_code']==0 and build['tested_sha']==SHA and frozen['status']=='PASS'
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['implementation_id']=='OR-MIG-0126' and m['implementation_sha']==m['tested_sha']==SHA
    assert m['status']=='PASS' and m['client_exit']==m['server_exit']==0 and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown']
    assert m['server_sha256']==build['binary_sha256'] and m['client_sha256']==frozen['binaries']['pg_observer.exe']
    assert m['server_arguments'][-2:]==['--spawn',','.join(map(str,initial['character'][:3]))]

def full(b):
    size=int.from_bytes(b[6:8],'big');pos=8;items=[]
    for slot in range(size):
        count=(128-b[pos])&255;pos+=1
        if count==255:
            c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
        item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2;items.append((item,count))
    assert pos==len(b);return items

def decode(log):
    inventories={};stats={};transitions=[];xp=[]
    for tick,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',log):
        tick=int(tick);op=int(op);b=bytes.fromhex(h.replace(',',''))
        if op==46:
            skill=(b[2]-128)&255;value=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'));stats[skill]=value
            if skill==20 and (not xp or xp[-1]!=value[1]):xp.append(value[1])
        if op not in [18,22]:continue
        inv=int.from_bytes(b[4:6],'big');old=inventories.get(inv)
        if op==22:nxt=full(b)
        else:
            assert old is not None;nxt=old[:];pos=6
            while pos<len(b):
                slot=b[pos];pos+=1
                if slot>=128:slot=((slot<<8)|b[pos])-32768;pos+=1
                item=int.from_bytes(b[pos:pos+2],'big')-1;pos+=2;count=0
                if item>=0:
                    count=b[pos];pos+=1
                    if count==255:count=int.from_bytes(b[pos:pos+4],'big');pos+=4
                nxt[slot]=(item,count)
            assert pos==len(b)
        if old is None or old!=nxt:transitions.append((tick,inv,nxt))
        inventories[inv]=nxt
    return inventories,stats,transitions,xp

log=(OUT/'produce/client.log').read_text(encoding='utf-8');relog=(OUT/'relogin/client.log').read_text(encoding='utf-8')
assert 'PG_CASE_PASS native special-essence altar kind='+a.feature in log and 'PG_NATIVE_NO_PENDING_SKILL_MULTI' in relog
menus=re.findall(r'PG_DAE_ALTAR_MENU action=(\d+).*native_action=Object \{ id: (\d+), tile: \[(\d+), (\d+)\], operation: (\d+), sub: (\d+) \}',log)
assert [tuple(map(int,v))for v in menus]==[(n,id,x,z,1,0)for n in range(3)]
wire=[(int(t),int(op),bytes.fromhex(h.replace(',','')))for t,op,h in re.findall(r'PG_WIRE_TX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',log)]
for tick,op,b in wire:
    assert op in [47,73]
    if op==47:assert b==bytes.fromhex('02240043ffffffff01')
    else:assert len(b)==8 and ((b[0]<<8)|((b[1]-128)&255),int.from_bytes(b[2:4],'little'),b[4],b[5],int.from_bytes(b[6:8],'big'))==(z,x,128,128,id)
sends=[tick for tick,op,b in wire if op==73];assert len(sends)==3
invs,observed,transitions,xp=decode(log);bag_states=[(tick,image)for tick,inv,image in transitions if inv==93]
assert [image for tick,image in bag_states]==[base,first,final]
assert xp==[13034431,fine1//10,fine2//10]
sequences=[int(t)for t in re.findall(r'PG_SEQUENCE tick=(\d+) sequence=791 delay=0',log)]
assert len(sequences)==2 and [tick-seq for (tick,image),seq in zip(bag_states[1:],sequences)]==[1,1]
assert log.count('decoded_slot_id_height_delay=[(0, 186, 100, 0)]')==2
assert bag_states[1][0]>sends[0] and bag_states[2][0]>sends[1] and sends[2]>=bag_states[2][0]
# Opcode83 increments the receive clock when applied; PG_RX is the pre-apply value.
assert max(int(t)for t in re.findall(r'PG_APPLIED tick=(\d+) opcode=83',log))>=sends[2]+5
for inv in [93,94]:assert [[inv,s,item,n,0]for s,(item,n)in enumerate(invs[inv])if item>=0]==[v for v in expected if v[0]==inv]
for skill,current,base_level,fine in stats:assert observed[skill]==(current,fine//10)
invs,observed,transitions,xp=decode(relog)
assert len([v for v in transitions if v[1]==93])==1 and xp==[fine2//10]
for inv in [93,94]:assert [[inv,s,item,n,0]for s,(item,n)in enumerate(invs[inv])if item>=0]==[v for v in expected if v[0]==inv]
for skill,current,base_level,fine in stats:assert observed[skill]==(current,fine//10)
names=['initial-state.json','build.json','frozen-observer/build.json','frozen-observer/pg_observer.rs']+[phase+'/'+name for phase in ['produce','relogin']for name in ['manifest.json','after.json','client.log','server.log']]
report={'implementation_id':'OR-MIG-0126','tested_sha':SHA,'status':'PASS_SCOPED','feature':a.feature,'runes':28*per,'fine_xp_awards':[fine1-130344311,fine2-fine1],'inventory_change_ticks':[tick for tick,image in bag_states[1:]],'scope':'Three actual rendered native Craft-rune menus and independently decoded real location sends: first7Daeyalt only, retains21ordinary; separate second action converts remainingordinary; delivered emptythird grants nothing. Exactly two complete native bag transitions, wholeStats/gear, correct Rune yield and one aggregate1.5XP award for specialessence only. Each actual791sequence precedes credit by one receivedclock; two186spots height100. Exact acknowledged save, process restart and independently decoded sameaccount reconnect retain full quantity and fineXP.','limitations':['Actual effective500native only;125variant remains separately blocked by pinned login policy.','Wire stats expose wholeXP only; first expectedfine is checked through independently verified World behavior, summed separate fineawards are checked in durableSQL. No claim of intermediate SQL fineXP snapshot for this account.','No essence acquisition/altar entry/native boost activation/original executable or audio/render parity.'],'evidence_sha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest()for n in names}}
path=OUT/'independent-assessment.json';assert not path.exists();path.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS_SCOPED',a.feature,28*per,'runes',fine2,'fineXP saved/restarted')
