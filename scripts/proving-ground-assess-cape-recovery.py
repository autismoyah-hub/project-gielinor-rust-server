"""Independently verify unacknowledged cape state is not replayed after restart."""
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-crafting-cape-crash/native-001';SHA='7d2133f23e00ad852df8726ead67493e83c901b4'
load=lambda n:json.loads((OUT/n).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json')
assert load('produce/before.json')==initial and load('relogin/before.json')==after
for s in [initial,after,post]:
 assert s['inventory']==initial['inventory'] and s['stats']==initial['stats'] and s['character'][:4]==initial['character'][:4]
 assert s['integrity_check']=='ok'and s['xp_rate_hundreds']==500
assert initial['stats'][12]==[12,99,99,130344311]
for phase in ['produce','relogin']:
 m=load(phase+'/manifest.json');assert m['status']=='PASS'and m['tested_sha']==m['implementation_sha']==SHA and m['implementation_id']=='OR-MIG-0130'
 assert m['client_exit']==0 and m['resource_hashes_unchanged']and not m['forced_shutdown']
 assert m['server_sha256']==load('build.json')['binary_sha256']and m['client_sha256']==load('frozen-observer/build.json')['binaries']['pg_observer.exe']
 if phase=='produce':assert m['intentional_crash_injected']and m['server_exit']!=0 and not m['save_acknowledgement']and not m['graceful_shutdown']
 else:assert m['server_exit']==0 and m['save_acknowledgement']and m['graceful_shutdown']
text=(OUT/'produce/client.log').read_text(encoding='utf-8');relog=(OUT/'relogin/client.log').read_text(encoding='utf-8')
assert 'PG_CRASH_READY unsaved native Crafting cape current100'in text and 'PG_EXPECTED_CRASH_DISCONNECT'in text
assert 'PG_NATIVE_NO_PENDING_SKILL_MULTI'in relog
def samples(t):
 out=[]
 for tick,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',t):
  b=bytes.fromhex(h.replace(',',''))
  if int(op)==46 and ((b[2]-128)&255)==12:
   value=[int(tick),b[1],int.from_bytes(b[5:7]+b[3:5],'big')];assert value[2]==13034431
   if not out or out[-1][1:]!=value[1:]:out.append(value)
 return out
seen=samples(text);recovered=samples(relog);assert [s[1]for s in seen]==[99,100]and all(s[1]==99 for s in recovered)
boosts=[]
for tick,h in re.findall(r'PG_WIRE_TX tick=(\d+) opcode=47 payload=\[(.*?)\]',text):
 b=bytes.fromhex(h.replace(',',''));component=int.from_bytes(b[:4],'big')
 if component>>16!=548:assert component==25362448 and b[-1]==2;boosts.append(int(tick))
assert len(boosts)==2
names=['initial-state.json','build.json','frozen-observer/build.json','frozen-observer/pg_observer.rs']+[p+'/'+n for p in ['produce','relogin']for n in ['manifest.json','before.json','after.json','client.log','server.log']]
result={'implementation_id':'OR-MIG-0130','tested_sha':SHA,'status':'PASS_SCOPED','scope':'Fresh disposable account actual nativeBoost99->100andrepeated100 observed inreceived46atconstantXP. Existing owned fault path terminates only this freshly launched localserver beforeSaveack;serverexitnonzero,nogracefuldrain. ActualSQLiteaftertermination andrealserverrestart/native reconnect restoreoriginal99/wholeStatsfineXPbaggear. ReconnectsendsnoBoostandnopendingproduction;no unsavedperk/XP/item replay. No successful immutable account reused or reseeded.','produced_stat_samples':seen,'recovered_stat_samples':recovered,'boost_send_clocks':boosts,'limitations':['Termination is before save acknowledgement;not during SQLitecommit failure injection.','This verifies immediatecapeBoost recovery; pendingRuneproduction interruption and liveMember guards are separately exercised inpublicWorld scenarios atsameSHA.'],'evidence_sha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest()for n in names}}
p=OUT/'independent-assessment.json';assert not p.exists();p.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS_SCOPED unsaved100 restores99 on actualserverrestart; wholeStatsfineXPbaggear retained')
