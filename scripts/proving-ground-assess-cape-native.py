"""Independently decode native cape menu actions, durable state and normal regeneration."""
import argparse,hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='7d2133f23e00ad852df8726ead67493e83c901b4'
p=argparse.ArgumentParser();p.add_argument('--feature',required=True);p.add_argument('--run-id',default='native-001');a=p.parse_args()
assert a.feature in ['crafting-cape-boost','crafting-cape-trimmed','crafting-cape-f2p','crafting-cape-drained','crafting-cape-expiry']
OUT=ROOT/'docs/openrune/verification/independent'/('deep-'+a.feature)/a.run_id
load=lambda n:json.loads((OUT/n).read_text(encoding='utf-8'))
f2p=a.feature.endswith('-f2p');drained=a.feature.endswith('-drained');expiry=a.feature.endswith('-expiry');cape=9781 if a.feature.endswith('-trimmed')else 9780
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json')
assert load('produce/before.json')==initial and load('relogin/before.json')==after
for state in [initial,after,post]:
 assert state['integrity_check']=='ok'and state['xp_rate_hundreds']==500
 assert state['inventory']==initial['inventory'] and state['character'][:4]==initial['character'][:4]
 assert [v[:3]for v in state['inventory']if v[0]==94 and v[1]==1]==[[94,1,cape]]
 for before,row in zip(initial['stats'],state['stats']):
  assert row[:1]+row[2:]==before[:1]+before[2:]
  if row[0]!=12:assert row==before
assert initial['stats'][12]==[12,50 if drained else 99,99,130344311]
build=load('build.json');frozen=load('frozen-observer/build.json');assert build['tested_sha']==SHA and build['exit_code']==0 and frozen['status']=='PASS'
for phase in ['produce','relogin']:
 m=load(phase+'/manifest.json');assert m['status']=='PASS'and m['implementation_id']=='OR-MIG-0130'and m['implementation_sha']==m['tested_sha']==SHA
 assert m['client_exit']==m['server_exit']==0 and m['save_acknowledgement']and m['graceful_shutdown']and m['resource_hashes_unchanged']and not m['forced_shutdown']
 assert m['server_sha256']==build['binary_sha256']and m['client_sha256']==frozen['binaries']['pg_observer.exe']

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
            if skill==12 and (not xp or xp[-1]!=value[1]):xp.append(value[1])
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

def skill_samples(log):
 rows=[]
 for t,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',log):
  if int(op)!=46:continue
  b=bytes.fromhex(h.replace(',',''));skill=(b[2]-128)&255
  if skill==12:
   value=(int(t),b[1],int.from_bytes(b[5:7]+b[3:5],'big'))
   if not rows or rows[-1][1:]!=value[1:]:rows.append(value)
 return rows
log=(OUT/'produce/client.log').read_text(encoding='utf-8');relog=(OUT/'relogin/client.log').read_text(encoding='utf-8')
assert f'PG_CAPE_ACCOUNT kind={a.feature} actual_member={str(not f2p).lower()} cape={cape} ' in log
assert 'PG_CASE_PASS native Crafting cape kind='+a.feature in log and 'PG_NATIVE_NO_PENDING_SKILL_MULTI'in relog
results=[tuple(map(int,v))for v in re.findall(r'PG_CAPE_RESULT press=(\d+) before=(\d+) after=(\d+) base=(\d+) noXPitemsgear tick=(\d+)',log)]
assert [v[0]for v in results]==[0,1]
for press,before,now,base,tick in results:
 assert base==99 and now==(before if f2p or before>=100 else before+1)
if not drained:assert results[0][1]==99
else:assert 50<=results[0][1]<99
wire=[(int(t),bytes.fromhex(h.replace(',','')))for t,h in re.findall(r'PG_WIRE_TX tick=(\d+) opcode=47 payload=\[(.*?)\]',log)]
boosts=[]
for tick,b in wire:
 if int.from_bytes(b[:4],'big')==(387<<16)|16:
  assert len(b)==9 and int.from_bytes(b[4:6],'big')==65535 and int.from_bytes(b[6:8],'big')in[65535,cape]and b[8]==2;boosts.append(tick)
assert len(boosts)==2
assert len(re.findall(r'PG_CAPE_MENU press=\d+ option="Boost"',log))==2
assert not re.search(r'PG_WIRE_TX tick=\d+ opcode=(?:27|73|95|114|115) ',log)
for tick,b in wire:
 if int.from_bytes(b[:4],'big')!=(387<<16)|16:assert int.from_bytes(b[:4],'big')>>16==548
samples=skill_samples(log);assert samples[0][1:]==(initial['stats'][12][1],13034431)
assert all(v[2]==13034431 for v in samples)
if f2p:assert all(v[1]==99 for v in samples)and after['stats'][12][1]==99
elif expiry:
 assert 'PG_CAPE_NATIVE_EXPIRY current99'in log and after['stats'][12][1]==99
 assert [v[1]for v in samples]==[99,100,99]
elif not drained:assert after['stats'][12][1]in[99,100]and [v[1]for v in samples]in[[99,100],[99,100,99]]
else:
 assert all(b[1]-a[1]==1 for a,b in zip(samples,samples[1:]))and after['stats'][12][1]in[samples[-1][1],samples[-1][1]+1]
for phase,text,state in [('produce',log,after),('relogin',relog,post)]:
 invs,observed,transitions,xp=decode(text)
 assert len([v for v in transitions if v[1]==93])==len([v for v in transitions if v[1]==94])==1
 for inv in [93,94]:assert [[inv,s,item,n,0]for s,(item,n)in enumerate(invs[inv])if item>=0]==[v for v in initial['inventory']if v[0]==inv]
 for skill,current,base,fine in initial['stats']:
  if skill!=12:assert observed[skill]==(current,fine//10)
 assert xp==[13034431]
recovered=skill_samples(relog);assert recovered[0][1]==after['stats'][12][1]
for one,two in zip(recovered,recovered[1:]):
 assert two[1]==one[1]+(1 if one[1]<99 else -1 if one[1]>99 else 0)
last=recovered[-1][1];assert post['stats'][12][1]in[last,last+(1 if last<99 else-1 if last>99 else 0)]
for t,h in re.findall(r'PG_WIRE_TX tick=(\d+) opcode=47 payload=\[(.*?)\]',relog):
 b=bytes.fromhex(h.replace(',',''));assert not(int.from_bytes(b[:4],'big')==(387<<16)|16 and b[-1]==2)
names=['initial-state.json','build.json','frozen-observer/build.json','frozen-observer/pg_observer.rs']+[phase+'/'+n for phase in ['produce','relogin']for n in ['manifest.json','before.json','after.json','client.log','server.log']]
report={'implementation_id':'OR-MIG-0130','tested_sha':SHA,'status':'PASS_SCOPED','feature':a.feature,'actual_cape':cape,'native_results':results,'produce_crafting_stat_samples':samples,'relogin_crafting_stat_samples':recovered,'saved_current':after['stats'][12][1],'restarted_saved_current':post['stats'][12][1],'scope':'Two actual native equipment Boost menu actions at current387:16 generate genuine operation2/interface47 sends. Independent decoded46 confirms current-level behavior and noXP;whole native93/94andSQL bag/gear/base/fineXP retained. Acknowledged save and actual server restart;first native reconnect stat exactly matches durable current,subsequent changes onlynormalone-step regeneration toward99,withoutanotherBoost action.','limitations':['Restored worn-cape fixture does not prove acquisition/nativeWear or passive-on-equip behavior. Separate equipment control case supplies real Wear/Remove and private Skills softwareframes.','UI layout/menu and received current-stat values observed;no full rendered Skills-panel claim for this basic case.','Membership message is awaited by the frozen observer predicate;rawmessage bytes are not logged independently. No original executable/audio/render parity.'],'evidence_sha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest()for n in names}}
path=OUT/'independent-assessment.json';assert not path.exists();path.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS_SCOPED',a.feature,'saved/restarted',report['saved_current'],report['restarted_saved_current'])
