"""Independent native light menu, whole SQL, raw inventory and restart assessment."""
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='ad739676dd01f5be809cdfe5ecf4f1e6ea77238a';OUT=ROOT/'docs/openrune/verification/independent/deep-light-sources/native-002'
load=lambda p:json.loads((OUT/p).read_text(encoding='utf-8'))
rows=[(36,33),(38,32),(596,594),(4529,4531),(4532,4534),(4522,4524),(4537,4539),(4548,4550),(5014,5013)]
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');expected=[x[:]for x in initial['inventory']];assert len(expected)==28
for i,(raw,lit)in enumerate(rows):
 assert expected[4+2*i]==[93,4+2*i,raw,1,0] and expected[5+2*i]==[93,5+2*i,raw,1,0];expected[5+2*i][2]=lit
assert expected[0]==[93,0,590,1,0] and expected[1]==[93,1,590,1,0] and expected[27]==[93,27,995,1000,0]
assert after['inventory']==post['inventory']==expected and load('relogin/before.json')==after
assert initial['stats']==after['stats']==post['stats'];assert initial['character'][3]==after['character'][3]==post['character'][3]
assert next(x for x in initial['stats']if x[0]==11)==[11,99,99,130344311]
build=load('build.json');assert build['tested_sha']==SHA and build['exit_code']==0
for phase in ['produce','relogin']:
 m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['status']=='PASS' and m['client_exit']==m['server_exit']==0
 assert m['server_sha256']==build['binary_sha256'] and m['save_acknowledgement'] and m['graceful_shutdown'] and m['resource_hashes_unchanged'] and not m['forced_shutdown'];assert load(phase+'/after.json')['integrity_check']=='ok'
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'PG_CASE_PASS native nine light sources both held directions18lighting9extinguish;' in log
assert len(re.findall(r'^PG_LIGHT_LIT ',log,re.M))==18 and len(re.findall(r'^PG_LIGHT_UNLIT ',log,re.M))==9
assert log.count('PG_REPLAY label="stale selected unlit source after lighting"')==18 and log.count('PG_REPLAY label="stale extinguish explicit item after unlighting"')==9
assert len(re.findall(r'^PG_LIGHT_MESSAGE row=\d+ text="You extinguish the light source\."',log,re.M))==9
wire=[(int(op),bytes.fromhex(h.replace(',','')))for op,h in re.findall(r'PG_WIRE_TX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',log)];uses=[b for op,b in wire if op==27];assert len(uses)==18 and not any(op==115 for op,b in wire)
beadd=lambda b:(b[0]<<8)|((b[1]-128)&255);leadd=lambda b:((b[0]-128)&255)|(b[1]<<8)
for index,b in enumerate(uses):
 i,j=divmod(index,2);slot=4+2*i+j;raw=rows[i][0];assert len(b)==16;assert (beadd(b[4:6]),beadd(b[10:12]),int.from_bytes(b[8:10],'little'),leadd(b[6:8]))==((0,590,slot,raw)if j==0 else(slot,raw,0,590))
invs={};stats={};relog=(OUT/'relogin/client.log').read_text(encoding='utf-8');assert 'PG_NATIVE_NO_PENDING_SKILL_MULTI' in relog
for op,h in re.findall(r'PG_RX tick=\d+ opcode=(\d+) payload=\[(.*?)\]',relog):
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
report={'implementation_id':'OR-MIG-0084','tested_sha':SHA,'status':'PASS','actions':{'lighting':18,'extinguish':9,'stale_refusal':27},'fine_xp_award':0,'scope':'Native real Tinderbox-on-item in both held directions for all nine profiles and each actual Extinguish menu. Whole full28 inventory, duplicated retained tinderboxes/coins, no XP/no quantity flow, nine extinguish acknowledgements,18stale held and9stale extinguish refusals. Exact nine lit/nine unlit ordinary forms and every Stats/fraction survive acknowledged-save/process restart/raw22/46 native reconnect; no pending quantity/replay.','limitations':['Normal saved state only; separate unsaved recovery/cancelled input must be assessed independently.','Native received clocks do not establish render/audio or donor executable parity.','Synthetic World negative/actual cache binding/reference acceptance remain separate.'],'evidence_sha256':{name:hashlib.sha256((OUT/name).read_bytes()).hexdigest()for name in ['initial-state.json','build.json','produce/client.log','produce/manifest.json','produce/after.json','relogin/client.log','relogin/manifest.json','relogin/after.json','frozen-observer/pg_observer.rs','frozen-observer/build.json']}}
p=OUT/'independent-assessment.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS native27light actions/27stale refusals/zeroXP/actual saved reconnect')
