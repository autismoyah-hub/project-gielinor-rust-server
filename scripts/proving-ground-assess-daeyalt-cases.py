"""Independently decode protected native Daeyalt cases and restarted SQL state."""
import argparse,hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
SHA='bd1294ab9e0ff850155b024364dc3a3a98a37bed'
p=argparse.ArgumentParser();p.add_argument('--feature',required=True);p.add_argument('--run-id',default='native-001');a=p.parse_args()
assert a.feature in ['rune-mind-daeyalt-'+s for s in ['f2p','close-after-reach','move-after-reach','logout-after-reach','crash']]
OUT=ROOT/'docs/openrune/verification/independent'/('deep-'+a.feature)/a.run_id
load=lambda n:json.loads((OUT/n).read_text(encoding='utf-8'))
initial=load('initial-state.json');before=load('produce/before.json');after=load('produce/after.json');post=load('relogin/after.json')
assert initial==before and load('relogin/before.json')==after
f2p=a.feature.endswith('-f2p');crash=a.feature.endswith('-crash');walk='-move-' in a.feature;logout='-logout-' in a.feature;close='-close-' in a.feature
base=[(24704 if s<7 else 1436 if s%2==0 else 7936,1)for s in range(28)]
assert [(v[2],v[3])for v in initial['inventory']if v[0]==93]==base
for state in [initial,after,post]:
    assert state['integrity_check']=='ok' and state['xp_rate_hundreds']==500 and state['account_membership']==(not f2p)
assert initial['stats'][20]==[20,99,99,130344311]
expected_inventory=initial['inventory'];expected_stats=initial['stats'];outcome='NO_COMMIT'
if logout and after['inventory']!=initial['inventory']:
    first=base[:]
    for s in range(7):first[s]=(-1,0)
    first[0]=(558,56)
    expected_inventory=[[93,s,item,n,0]for s,(item,n)in enumerate(first)if item>=0]+[v for v in initial['inventory']if v[0]!=93]
    expected_stats=[v[:]for v in initial['stats']];expected_stats[20][3]=130347198;outcome='ONE_ATOMIC_BATCH_AT_DISCONNECT_BOUNDARY'
assert after['inventory']==post['inventory']==expected_inventory
assert after['stats']==post['stats']==expected_stats
assert after['character'][:4]==post['character'][:4]
expected_character=initial['character'][:4]
if walk:expected_character[0]-=1
assert after['character'][:4]==expected_character
build=load('build.json');assert build['tested_sha']==SHA and build['exit_code']==0
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json')
    assert m['tested_sha']==m['implementation_sha']==SHA and m['implementation_id']=='OR-MIG-0126'
    assert m['status']=='PASS' and m['client_exit']==0 and m['resource_hashes_unchanged'] and not m['forced_shutdown']
    assert m['server_sha256']==build['binary_sha256']
    frozen=load('frozen-observer/build.json');assert m['client_sha256']==frozen['binaries']['pg_observer.exe']
    if crash and phase=='produce':assert m['intentional_crash_injected'] and not m['save_acknowledgement'] and not m['graceful_shutdown'] and m['server_exit']!=0
    else:assert m['save_acknowledgement'] and m['graceful_shutdown'] and m['server_exit']==0
    assert m['server_arguments'][-2]=='--spawn' and m['server_arguments'][-1]==','.join(map(str,load(phase+'/before.json')['character'][:3]))

def packets(log):
    for t,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',log):yield int(t),int(op),bytes.fromhex(h.replace(',',''))

def full(b):
    size=int.from_bytes(b[6:8],'big');pos=8;items=[]
    for slot in range(size):
        count=(128-b[pos])&255;pos+=1
        if count==255:
            c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
        item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2;items.append((item,count))
    assert pos==len(b);return items

def images(log):
    inventories={};stats={};changes=[]
    for tick,op,b in packets(log):
        if op==46:stats[(b[2]-128)&255]=(b[1],int.from_bytes(b[5:7]+b[3:5],'big'))
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
        if old is not None and old!=nxt:changes.append((tick,inv))
        inventories[inv]=nxt
    return inventories,stats,changes

log=(OUT/'produce/client.log').read_text(encoding='utf-8');relog=(OUT/'relogin/client.log').read_text(encoding='utf-8')
assert 'PG_DAEYALT_CASE_ACCOUNT' in log and 'PG_NATIVE_NO_PENDING_SKILL_MULTI' in relog
menus=re.findall(r'PG_DAEYALT_CASE_MENU action=(\d+).*native_action=Object \{ id: (\d+), tile: \[(\d+), (\d+)\], operation: (\d+), sub: (\d+) \}',log)
assert [tuple(map(int,v))for v in menus]==[(i,34761,2785,4840,1,0)for i in range(2 if f2p else 1)]
wire=[(int(t),int(op),bytes.fromhex(h.replace(',','')))for t,op,h in re.findall(r'PG_WIRE_TX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',log)]
for t,op,b in wire:
    assert op not in [27,115]
    if op==47:assert b==bytes.fromhex("02240043ffffffff01"),"Only actual Inventory tab548:67 allowed"
    if op==73:assert len(b)==8 and ((b[0]<<8)|((b[1]-128)&255),int.from_bytes(b[2:4],'little'),b[4],b[5],int.from_bytes(b[6:8],'big'))==(4840,2785,128,128,34761)
wire=[v for v in wire if v[1]!=47]
assert [op for t,op,b in wire]==([73,73]if f2p else [73,95]if close else [73,114]if walk else [73])
if close or walk or logout:
    pending=re.findall(r'PG_DAEYALT_CASE_PENDING actual791 original28essence andXP intact kind=.* tick=(\d+)',log);assert len(pending)==1
    assert 'sequence=791 delay=0' in log
    if not logout:assert wire[-1][0]>=int(pending[0]) and 'PG_DAEYALT_CASE_CANCELLED' in log
if walk:
    b=wire[-1][2];assert len(b)==5 and (int.from_bytes(b[:2],'little'),int.from_bytes(b[2:4],'little'),b[4])==(initial['character'][0]-1,initial['character'][1],128)
if f2p:
    assert log.count('PG_DAEYALT_CASE_MESSAGE action=')==2
    assert log.count('observed="You need to be a member to bind that essence." no_ordinary_fallback=true')==2
invs,stats,changes=images(log)
if not logout:
    expected=base[:]
    if crash:
        for s in range(7):expected[s]=(-1,0)
        expected[0]=(558,56)
        assert len([v for v in changes if v[1]==93])==1 and 'PG_CRASH_READY' in log
    else:assert changes==[]
    assert invs[93]==expected
    assert [[94,slot,item,n,0]for slot,(item,n)in enumerate(invs[94])if item>=0]==[v for v in initial['inventory']if v[0]==94]
    for skill,current,base_level,fine in initial['stats']:
        assert stats[skill]==(current,13034719 if crash and skill==20 else fine//10)
invs,observed_stats,relogin_changes=images(relog);assert relogin_changes==[]
for inv in [93,94]:assert [[inv,s,item,n,0]for s,(item,n)in enumerate(invs[inv])if item>=0]==[v for v in expected_inventory if v[0]==inv]
for skill,current,base_level,fine in expected_stats:assert observed_stats[skill]==(current,fine//10)
names=['initial-state.json','build.json','frozen-observer/build.json','frozen-observer/pg_observer.rs']+[phase+'/'+name for phase in ['produce','relogin']for name in ['manifest.json','before.json','after.json','client.log','server.log']]
scope=('Two actual native Craft-rune menu actions refuse nonmember Daeyalt without falling back to21ordinary essence.'if f2p else 'Actual reached sequence791 with original bag/XP intact followed by genuine native Walk or Close cancels the pending transaction.'if close or walk else 'Native transport disconnect after observed reached sequence791; whole persisted outcome classified without assuming race timing.'if logout else 'Actual native first56runes with21ordinary retained, then abrupt termination of only the fresh owned server before save acknowledgement; restart restores the prior complete account.')
report={'implementation_id':'OR-MIG-0126','tested_sha':SHA,'status':'PASS_SCOPED','feature':a.feature,'persisted_outcome':outcome,'scope':scope+' Exact whole bag/gear/Stats/fineXP and position independently checked against SQL and decoded native reconnect after process restart.','limitations':['Inside-altar disposable fixture does not establish essence acquisition or altar entry.','No during-SQL-commit injection, original executable/render/audio parity or native125rate claim. Character comparison covers position/plane/energy; runtime login varps and logout timestamps legitimately change.','Close uses existing native Action::Close without an open dialog; no visible Close-button or keyboard observation.'if close else 'Native transport disconnect may race one server tick; only the observed atomic result is claimed. No visible Logout-button observation or proof of cancellation before completion.'],'evidence_sha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest()for n in names}}
path=OUT/'independent-assessment.json'
if path.exists():
    existing=load('independent-assessment.json');assert existing['status']=='PASS_SCOPED' and existing['tested_sha']==SHA and existing['persisted_outcome']==outcome
    print('EXISTING_EVIDENCE_RECHECKED no gameplay replay',a.feature,outcome)
else:
    path.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS_SCOPED',a.feature,outcome)
