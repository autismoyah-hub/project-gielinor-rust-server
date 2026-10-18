"""Independent counted native choices, whole SQL state and actual reconnect."""
import argparse,hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='bc277dc3e7e15088323a4f048f57dc43185fb7c0'
parser=argparse.ArgumentParser();parser.add_argument('--feature',choices=['amethyst','amethyst-batch'],default='amethyst');a=parser.parse_args();batch=a.feature=='amethyst-batch'
OUT=ROOT/'docs/openrune/verification/independent'/('deep-'+a.feature)/'native-001'
load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json')
assert initial['inventory']==[[93,s,1755 if s in [0,1] else 21347 if 4<=s<=26 else 995 if s==27 else 1931,1000 if s==27 else 1,0] for s in range(28)]
expected={r[1]:r[:] for r in initial['inventory']}
if batch:
    plans=[(26,0,1,[26]),(25,1,5,[25,4,5,6,7]),(24,2,10,[24,8,9,10,11,12,13,14,15,16]),(23,3,7,[23,17,18,19,20,21,22])]
    for selected,choice,count,slots in plans:
        out,per_raw=[(21338,15),(21350,15),(21352,5),(25853,8)][choice]
        for slot in slots:
            assert expected[slot][2:]==[21347,1,0];del expected[slot]
            target=next((s for s,row in expected.items() if row[2]==out),None)
            if target is None:
                target=next(s for s in range(28) if s not in expected);expected[target]=[93,target,out,per_raw,0]
            else:expected[target][3]+=per_raw
else:
    for slot,out,count in [(4,21338,30),(5,21350,30),(6,21352,10),(7,25853,16)]:expected[slot]=[93,slot,out,count,0]
    for slot in [23,24,25,26]:del expected[slot]
expected=[expected[s] for s in sorted(expected)]
assert after['inventory']==post['inventory']==expected and load('relogin/before.json')==after
assert len(initial['stats'])==len(after['stats'])==len(post['stats'])
for x,y,z in zip(initial['stats'],after['stats'],post['stats']):
    assert x[0]==y[0] and y==z
    if x[0]==12:assert x==[12,99,99,130344311] and y==[12,99,99,130413311 if batch else 130368311]
    else:assert x==y
assert initial['character'][3]==after['character'][3]==post['character'][3]
build=load('build.json');assert build['tested_sha']==SHA and build['exit_code']==0
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS' and m['client_exit']==m['server_exit']==0
    assert m['server_sha256']==build['binary_sha256'] and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown']
    assert load(phase+'/after.json')['integrity_check']=='ok'
log=(OUT/'produce/client.log').read_text(encoding='utf-8')
if batch:
    assert 'PG_CASE_PASS native amethyst1/5/10/All7;' in log
    rows=re.findall(r'^PG_AMETHYST_BATCH choice=(\d+) label=(\S+) count=(\d+) captured=(\[.*?\]) output=(\d+) per_raw=(\d+) fine=(\d+)',log,re.M);assert len(rows)==4
    total=130344311
    for i,(row,plan) in enumerate(zip(rows,plans)):
        selected,choice,count,slots=plan;out,per_raw=[(21338,15),(21350,15),(21352,5),(25853,8)][choice];total+=3000*count
        assert (int(row[0]),row[1],int(row[2]),json.loads(row[3]),int(row[4]),int(row[5]),int(row[6]))==(choice,['1','5','10','All'][i],count,slots,out,per_raw,total)
    assert log.count('PG_REPLAY label="native stale completed counted amethyst batch"')==4 and log.count('PG_REPLAY label="native stale consumed counted amethyst batch input"')==4
    wire=[(int(op),bytes.fromhex(h.replace(',',''))) for op,h in re.findall(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log)]
    uses=[b for op,b in wire if op==27];makes=[b for op,b in wire if op==115];assert len(uses)==len(makes)==4
    beadd=lambda b:(b[0]<<8)|((b[1]-128)&255)
    leadd=lambda b:((b[0]-128)&255)|(b[1]<<8)
    for i,(use,make,plan) in enumerate(zip(uses,makes,plans)):
        slot,choice,count,_=plan;pair=(0,1755,slot,21347) if i%2==0 else (slot,21347,0,1755)
        assert len(use)==16 and (beadd(use[4:6]),beadd(use[10:12]),int.from_bytes(use[8:10],'little'),leadd(use[6:8]))==pair
        assert make==bytes([14,1,15+choice,0,0,count])
else:
    log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'PG_CASE_PASS native four counted amethyst choices both held directions;' in log
    credits=re.findall(r'^PG_AMETHYST_CREDIT index=(\d+) choice=(\d+) raw_slot=(\d+) output=(\d+) count=(\d+) fine=(\d+) label="(.*?)"',log,re.M);assert len(credits)==8
    slots=[4,26,5,25,6,24,7,23];profiles=[(21338,15,83,'Amethyst bolt tips'),(21350,15,85,'Amethyst arrowtips'),(21352,5,87,'Amethyst javelin tips'),(25853,8,89,'Amethyst dart tip')]
    for i,credit in enumerate(credits):
        out,count,level,name=profiles[i//2];assert tuple(map(int,credit[:6]))==(i,i//2,slots[i],out,count,130344311+3000*(i+1))
        assert credit[6]==f'{name} x{count} (level {level})'
    assert log.count('PG_REPLAY label="native stale completed amethyst quantity"')==8 and log.count('PG_REPLAY label="native stale consumed amethyst held pair"')==8 and log.count('PG_REPLAY label="native stale amethyst quantity after Close"')==1
    wire=[(int(op),bytes.fromhex(h.replace(',',''))) for op,h in re.findall(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log)]
    uses=[b for op,b in wire if op==27];makes=[b for op,b in wire if op==115];assert len(uses)==len(makes)==9
    beadd=lambda b:(b[0]<<8)|((b[1]-128)&255)
    leadd=lambda b:((b[0]-128)&255)|(b[1]<<8)
    for n,(use,make) in enumerate(zip(uses,makes)):
        i=max(0,n-1);slot=17 if n==0 else slots[i];pair=(0,1755,slot,21347) if n==0 or i%2==0 else (slot,21347,0,1755)
        assert len(use)==16 and (beadd(use[4:6]),beadd(use[10:12]),int.from_bytes(use[8:10],'little'),leadd(use[6:8]))==pair
        assert make==bytes([14,1,15+(0 if n==0 else i//2),0,0,1])
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
report={'implementation_id':'OR-MIG-0071','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','products_in_raw_batches':23 if batch else 8,'tip_counts':[15,75,50,56] if batch else [30,30,10,16],'fine_xp_award':69000 if batch else 24000,'scope':('Native real builtin1/5/10/All7choices process23raws with exact selected-first/ascending captures, whole stack15/75/50/56and69000fineXP; retained chisels/coins/allStats/energy and ordinary zero-vars outputs survive acknowledged save/process restart/raw native reconnect. Four stale quantity/consumed-input replays leave state unchanged.' if batch else 'Native four real counted choices/both held directions with actual count/level labels, nine genuine held/Make-one frame pairs including cancelled choice, first-free stack creation and merges from full28. Eight stale quantity/consumed-pair replays and genuine Close/stale reply preserve state. XP is60per raw, not per tip. Whole ordinary tips/retained15raws/two chisels/coins/Stats/energy and existing fractional XP survive acknowledged-save/process restart/independently decoded native reconnect.'),'limitations':[('Builtin native batches/normal persistence only; custom input/negative policy/partial/crash/effect-clock scopes remain separate.' if batch else 'This is a native Make-one/normal persistence subcampaign. Built-in batch quantities/custom input/negative policy/partial/crash/effect-clock scopes remain separate.'),'No rendered/audio/original executable or latestFORGEHEAD acceptance.'],'evidence_sha256':{p:hashlib.sha256((OUT/p).read_bytes()).hexdigest() for p in ['initial-state.json','build.json','produce/client.log','produce/manifest.json','produce/after.json','relogin/client.log','relogin/manifest.json','relogin/after.json','frozen-observer/pg_observer.rs','frozen-observer/build.json']}}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
