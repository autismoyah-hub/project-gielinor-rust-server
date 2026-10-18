"""Independent exact inventory/Stats, raw transport and real restart oracle at70."""
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'docs/openrune/verification/independent/deep-crafting-finishes/native-001'
SHA='c7ddebe844820260252fb37df518f8c271f37243'
load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json')
mapping={0:1755,1:1755,27:995,**{i:10980 for i in range(4,8)},**{i:10981 for i in range(8,12)},**{i:12927 for i in [17,19,21,23,25,26]}}
assert initial['inventory']==[[93,s,mapping.get(s,1931),1000 if s==27 else 1,0] for s in range(28)]
expected={r[1]:r[:] for r in initial['inventory']}
for slot in range(4,8):expected[slot]=[93,slot,10973,1,0]
for slot in range(8,12):del expected[slot]
for slot in [17,19,21,23]:expected[slot]=[93,slot,12929,1,0]
expected=[expected[s] for s in sorted(expected)]
assert after['inventory']==post['inventory']==expected and load('relogin/before.json')==after
assert len(initial['stats'])==len(after['stats'])==len(post['stats'])
for x,y,z in zip(initial['stats'],after['stats'],post['stats']):
    assert x[0]==y[0] and y==z
    if x[0]==12:assert x==[12,99,99,130344311] and y==[12,99,99,130389111]
    else:assert x==y
assert initial['character'][3]==after['character'][3]==post['character'][3]
build=load('build.json');assert build['tested_sha']==SHA and build['exit_code']==0
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json')
    assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS' and m['client_exit']==m['server_exit']==0
    assert m['server_sha256']==build['binary_sha256'] and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown']
    assert load(phase+'/after.json')['integrity_check']=='ok'
log=(OUT/'produce/client.log').read_text(encoding='utf-8')
assert 'PG_CASE_PASS native four wire and four manual helm credits;' in log
assert len(re.findall(r'^PG_FINISH_CREDIT ',log,re.M))==8
assert log.count('PG_REPLAY label="native stale consumed manual finish pair"')==8
assert 'native pending helm Walk cancellation' in log
wire=[(int(op),bytes.fromhex(h.replace(',',''))) for op,h in re.findall(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log)]
assert not any(op==115 for op,b in wire)
uses=[b for op,b in wire if op==27];assert len(uses)==9
pairs=[(0,1755,17,12927),(4,10980,8,10981),(9,10981,5,10980),(6,10980,10,10981),(11,10981,7,10980),(0,1755,17,12927),(19,12927,0,1755),(0,1755,21,12927),(23,12927,0,1755)]
beadd=lambda b:(b[0]<<8)|((b[1]-128)&255)
leadd=lambda b:((b[0]-128)&255)|(b[1]<<8)
for b,pair in zip(uses,pairs):assert len(b)==16 and (beadd(b[4:6]),beadd(b[10:12]),int.from_bytes(b[8:10],'little'),leadd(b[6:8]))==pair
traces=[json.loads(s.split(': ',1)[1]) for s in (OUT/'produce/server.log').read_text(encoding='utf-8').splitlines() if s.startswith('Assembly transition: ')]
assert len(traces)==4
for i,t in enumerate(traces):assert (t['first_slot'],t['second_slot'],t['first'],t['second'],t['output'],t['count'])==(4+i,8+i,10980,10981,10973,1) and t['queued_at']==t['committed_at']
invs={};stats={}
for op,h in re.findall(r'PG_RX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',(OUT/'relogin/client.log').read_text(encoding='utf-8')):
    b=bytes.fromhex(h.replace(',',''));op=int(op)
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
report={'implementation_id':'OR-MIG-0070','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','products':8,'fine_xp_award':44800,'scope':'Actual native four wiring and four manual helm products/both directions from full28; exact selected inputs and raw frame pairs, pending helm Walk cancellation, eight stale pair refusals, no quantity response/automatic repeat. Four server wiring traces are synchronous. Whole ordinary outputs/uncharged zero vars, retained chisels/two duplicate visages/coins/allStats/energy persist through acknowledged save, actual process restart and independently decoded native reconnect.','limitations':['Normal native/persistence scope only; PG-DEF-004 four authoritative wiring life failures remain unresolved.','No native death reachability, crash atomicity, rendered/audio or original executable parity.','No precise native helm logical-clock equivalence inferred from output observation; compensated World timing remains separately tested.'],'evidence_sha256':{p:hashlib.sha256((OUT/p).read_bytes()).hexdigest() for p in ['initial-state.json','build.json','produce/client.log','produce/server.log','produce/manifest.json','produce/after.json','relogin/client.log','relogin/manifest.json','relogin/after.json','frozen-observer/pg_observer.rs','frozen-observer/build.json']}}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(report))
