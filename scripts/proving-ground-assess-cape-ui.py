"""Assess same-account native UI observation after privately inspecting both BMPs."""
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-crafting-cape-equipment/native-002/continuation-001';PRIVATE=ROOT/'work/native-crafting-cape-equipment/native-002/continuation-001'
load=lambda n:json.loads((OUT/n).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json')
assert load('produce/before.json')==initial and load('relogin/before.json')==after
for state in [initial,after,post]:
 assert state['inventory']==initial['inventory'] and state['integrity_check']=='ok' and state['xp_rate_hundreds']==500
 assert state['character'][:4]==initial['character'][:4]
 for a,b in zip(initial['stats'],state['stats']):assert a==b if a[0]!=12 else a[:1]+a[2:]==b[:1]+b[2:]
assert initial['stats'][12]==[12,99,99,130344311] and after['stats'][12][1]in[99,100]and post['stats'][12][1]==99
source=load('continuation-source.json');assert source['original_database_sha256_before_continuation']==load('produce/manifest.json')['database_sha256_before']
assert hashlib.sha256(Path(source['private_snapshot']['path']).read_bytes()).hexdigest()==source['private_snapshot']['sha256']
for phase in ['produce','relogin']:
 m=load(phase+'/manifest.json');assert m['status']=='PASS'and m['tested_sha']==m['implementation_sha']=='7d2133f23e00ad852df8726ead67493e83c901b4'
 assert m['client_exit']==m['server_exit']==0 and m['save_acknowledgement']and m['graceful_shutdown']and not m['forced_shutdown']and m['resource_hashes_unchanged']
 assert m['server_sha256']==load('build.json')['binary_sha256']and m['client_sha256']==load('frozen-observer/build.json')['binaries']['pg_observer.exe']
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'PG_CASE_PASS sameaccount native cape UI observation'in log
frames=[]
for n,t in [('cape-skills-overlay100',220),('cape-skills-overlay-repeat100',245)]:
 raw=(PRIVATE/(n+'.bmp')).read_bytes();h=hashlib.sha256(raw).hexdigest();assert h=='54034406e182e1c362748ced0a4089d5fe48e3d9320adaa289ffeb169354f3dc'
 assert raw[:2]==b'BM'and int.from_bytes(raw[18:22],'little',signed=True)==765 and abs(int.from_bytes(raw[22:26],'little',signed=True))==503
 assert f'PG_CAPE_NATIVE_FRAME name={n} width=765 height=503 sha256={h} actual_current=100 tick={t}'in log
 frames.append({'private_name':n+'.bmp','sha256':h,'received_clock':t,'width':765,'height':503,'inspection':'Viewed local BMP pixels directly: right-hand Skills panel Crafting current100 abovebase99; actual native fonts/icons/widgets. Both captures show100/99 and are intentionally identical for repeat-no-stack.'})
actions=[]
for tick,h in re.findall(r'PG_WIRE_TX tick=(\d+) opcode=47 payload=\[(.*?)\]',log):
 b=bytes.fromhex(h.replace(',',''));component=int.from_bytes(b[:4],'big')
 if component>>16!=548:assert component==25362448 and b[-1]==2;actions.append(int(tick))
assert len(actions)==2
names=['continuation-source.json','initial-state.json','build.json','frozen-observer/build.json','frozen-observer/source-manifest.json','frozen-observer/pg_observer.rs']+[p+'/'+n for p in ['produce','relogin']for n in ['manifest.json','before.json','after.json','client.log','server.log']]
report={'implementation_id':'OR-MIG-0130','tested_sha':'7d2133f23e00ad852df8726ead67493e83c901b4','status':'PASS_SCOPED','scope':'Actual same-account continuation closes the earlier scene-only observation gap without reseeding or replaying Remove/Wear. Two genuine native equipmentBoost actions and Skills tab opens render the existing native viewport interface overlay; privately inspected pixels show100/99 afterBoost and afterrepeat. Complete bag/gear/allbase/fineXP preserved across acknowledgedsave, realserverrestart/native reconnect and ordinaryexpiry to99. Privatebefore-continuation SQLite backup preserved and hashchecked.','frames':frames,'boost_send_clocks':actions,'limitations':['Original equipment frames remain scene-only and do not become rendered evidence retroactively.','Actual existing interface renderer draws246widgets; unrelated pending minimap child-player/overlap visibility remains, so no claim of full scene/minimap/audio parity.','Private cache/assets/BMPs/database are not published; source-safe frame hashes and inspection notes only.'],'evidence_sha256':{n:hashlib.sha256((OUT/n).read_bytes()).hexdigest()for n in names}}
p=OUT/'independent-assessment.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS_SCOPED native Skills100/99 two private frames; actualrestart/baggear/fineXP retained')
