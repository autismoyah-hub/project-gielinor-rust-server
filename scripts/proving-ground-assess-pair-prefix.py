"""Decode retained native prefix witnesses without repeating gameplay."""
import argparse,hashlib,json,re
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('feature',choices=['pair-interrupt','pair-crash']);a=p.parse_args()
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent'/('deep-'+a.feature)/'native-001';SHA='fb348dda184b36ec2d503bfe0f1d5393cf8ae151'
load=lambda name:json.loads((OUT/name).read_text(encoding='utf-8'))
initial=load('initial-state.json');original={r[1]:(r[2],r[3]) for r in initial['inventory'] if r[0]==93};assert len(original)==28
def prefix(count):
 expected=original.copy()
 for first,second in [(12,25),*zip(range(12),range(13,25))][:count]:
  assert expected.pop(first)==(1759,1) and expected.pop(second)==(1673,1)
  free=next(s for s in range(28) if s not in expected);expected[free]=(1692,1)
 return expected
inv={};xp=None;level=None;full_seen=False;checked=0;maximum=0;ready=None
log=(OUT/'produce/client.log').read_text(encoding='utf-8')
for line in log.splitlines():
 marker=re.fullmatch(r'PG_PAIR_INTERRUPT_READY products=(\d+) fine_observed=(\d+) raw_remaining=(\d+) wool_remaining=(\d+)',line)
 if marker:
  count=int(marker[1]);assert full_seen and inv==prefix(count) and xp==(130344311+200*count)//10 and level==99
  assert int(marker[2])==130344311+200*count and int(marker[3])==int(marker[4])==13-count
  ready={'products':count,'fine_at_marker':int(marker[2]),'display_xp_from_raw46':xp,'remaining_pairs':13-count,'whole_inventory_from_raw_packets_matches':True}
 m=re.fullmatch(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',line)
 if not m:continue
 op=int(m[2]);b=bytes.fromhex(m[3].replace(',',''))
 if op==46 and (b[2]-128)&255==12:level=b[1];xp=int.from_bytes(b[5:7]+b[3:5],'big')
 if op not in [18,22] or int.from_bytes(b[4:6],'big')!=93:continue
 if op==22:
  size=int.from_bytes(b[6:8],'big');pos=8;inv={}
  for slot in range(size):
   count=(128-b[pos])&255;pos+=1
   if count==255:c=b[pos:pos+4];count=int.from_bytes(bytes([c[1],c[0],c[3],c[2]]),'big');pos+=4
   item=int.from_bytes(b[pos:pos+2],'little')-1;pos+=2
   if item>=0:inv[slot]=(item,count)
  full_seen=True
 else:
  assert full_seen
  pos=6
  while pos<len(b):
   slot=b[pos];pos+=1
   if slot>=128:slot=((slot<<8)|b[pos])-32768;pos+=1
   item=int.from_bytes(b[pos:pos+2],'big')-1;pos+=2;count=0
   if item>=0:
    count=b[pos];pos+=1
    if count==255:count=int.from_bytes(b[pos:pos+4],'big');pos+=4
    inv[slot]=(item,count)
   else:inv.pop(slot,None)
 assert pos==len(b)
 count=sum(n for item,n in inv.values() if item==1692);assert count>=maximum and inv==prefix(count)
 maximum=count;checked+=1
assert ready is not None and checked>0
traces=[json.loads(s.split(': ',1)[1]) for s in (OUT/'produce/server.log').read_text(encoding='utf-8').splitlines() if s.startswith('Assembly transition: ')]
assert maximum<=len(traces)<=13
for i,(t,(first,second)) in enumerate(zip(traces,[(12,25),*zip(range(12),range(13,25))])):
 assert (t['first_slot'],t['second_slot'],t['first'],t['second'],t['output'],t['count'])==(first,second,1759,1673,1692,1)
 assert t['committed_at']-t['queued_at']==(1 if i==0 else 2)
durable=sum(r[3] for r in load('produce/after.json')['inventory'] if r[2]==1692)
assert durable==(0 if a.feature=='pair-crash' else len(traces))
report={'implementation_id':'OR-MIG-0069','tested_sha':SHA,'status':'PASS','feature':a.feature,'initial_native_marker':ready,'maximum_native_products_in_all_raw_packets':maximum,'whole_inventory_packets_checked':checked,'authoritative_unsaved_or_saved_commits':len(traces),'durable_products':durable,'scope':'Independent raw22/18whole inventory and raw46Crafting witness at the native interruption marker. Every observed inventory packet is a valid captured selected-first/ascending prefix with first-free outputs, both input counts and1000coins conserved. Server trace clocks/pairs independently checked. Native first observation, later server commits and actual durable state remain distinct.','evidence_sha256':{name:hashlib.sha256((OUT/name).read_bytes()).hexdigest() for name in ['produce/client.log','produce/server.log','produce/after.json','produce/manifest.json']}}
dest=OUT/'native-prefix-witness.json';assert not dest.exists();dest.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
