"""Preserve152native failure; assess only genuinely exercised first-option scope."""
import argparse,hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='79a16ed1b33a402b59d57ace40a7ec978e28b32e'
p=argparse.ArgumentParser();p.add_argument('--feature',choices=['gnome-food-operations','gnome-food-f2p'],required=True);a=p.parse_args();BASE=ROOT/'docs/openrune/verification/independent'/('deep-'+a.feature)/'native-001';f2p=a.feature.endswith('f2p')
load=lambda n:json.loads((BASE/n).read_text(encoding='utf-8'))
initial=load('initial-state.json');after=load('produce/after.json');post=load('relogin/after.json');assert load('produce/before.json')==initial and load('relogin/before.json')==after
expected=initial['inventory']if f2p else[row for row in initial['inventory']if row[1]!=0]
assert after['inventory']==post['inventory']==expected
for state in [initial,after,post]:assert state['integrity_check']=='ok'and state['account_membership']==(not f2p)
for state in [after,post]:
    for old,new in zip(initial['stats'],state['stats']):
        if old[0]in[15,20]:assert new==([15,3,3,1740]if old[0]==15 else[20,2,2,830])
        elif old[0]==3:assert old[0]==new[0]and old[2:]==new[2:]
        else:assert old==new
for phase in ['produce','relogin']:
    m=load(phase+'/manifest.json');assert m['tested_sha']==m['implementation_sha']==SHA and m['implementation_id']=='OR-MIG-0152'
    assert m['server_exit']==0 and m['save_acknowledgement']and m['graceful_shutdown']and m['resource_hashes_unchanged']and not m['forced_shutdown']
    assert m['server_sha256']==load('build.json')['binary_sha256']and m['client_sha256']==load('frozen-observer/build.json')['binaries']['pg_observer.exe']
    assert m['launcher_sha256']==hashlib.sha256((BASE/'frozen-launcher.py').read_bytes()).hexdigest()
    if phase=='produce'and not f2p:assert m['status']=='FAIL'and m['client_exit']!=0
    else:assert m['status']=='PASS'and m['client_exit']==0
log=(BASE/'produce/client.log').read_text(encoding='utf-8');relog=(BASE/'relogin/client.log').read_text(encoding='utf-8');assert 'PG_NATIVE_NO_PENDING_SKILL_MULTI'in relog
traces=[json.loads(s.split('Food transition: ',1)[1])for s in(BASE/'produce/server.log').read_text(encoding='utf-8').splitlines()if s.startswith('Food transition: ')]
assert 'Food transition:'not in(BASE/'relogin/server.log').read_text(encoding='utf-8')
sent=[]
for tick,h in re.findall(r'PG_WIRE_TX tick=(\d+) opcode=47 payload=\[(.*?)\]',log):
    b=bytes.fromhex(h.replace(',',''))
    if len(b)==9 and int.from_bytes(b[:4],'big')==149<<16:sent.append([int(tick),int.from_bytes(b[4:6],'big'),int.from_bytes(b[6:8],'big'),b[8]])
assert any(r[1:]==[0,2128,2]for r in sent)and any(r[1:]==[13,2152,6]for r in sent)
if f2p:
    assert not traces and after['stats'][3]==post['stats'][3]==initial['stats'][3]
    assert 'PG_CASE_PASS gnomefood F2P'in log
else:
    assert len(traces)==1 and traces[0]['slot']==0 and traces[0]['before']==[2128,1,0]and traces[0]['after']is None
    assert traces[0]['hp_after']==traces[0]['hp_before']+1 and traces[0]['fine_xp_before']==traces[0]['fine_xp_after']==130344311
def samples(text):
    out=[]
    for tick,op,h in re.findall(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*?)\]',text):
        if int(op)!=46:continue
        b=bytes.fromhex(h.replace(',',''));assert len(b)==7
        if (b[2]-128)&255==3:out.append([int(tick),b[1],int.from_bytes(b[5:7]+b[3:5],'big')])
    return out
received=samples(relog);assert received and received[0][1]==after['stats'][3][1]and all(r[2]==13034431 for r in received)
names=['initial-state.json','build.json','frozen-launcher.py','frozen-observer/build.json','frozen-observer/pg_observer.rs']+[phase+'/'+n for phase in ['produce','relogin']for n in ['manifest.json','before.json','after.json','client.log','server.log']]
r={'implementation_id':'OR-MIG-0152','tested_sha':SHA,'status':'PASS_SCOPED','feature':a.feature,'scope':('F2P genuineEquaEatoperation2 causesnoitem/Stats/XP/gear changes; actualsave/restart/native reconnectconservesfullbag/HP99/memberfalse. Toaoperation6alsohasnoeffect butcannotbeclassifiedasamembershiprefusal.'if f2p else'OnlyEqua0genuineEatoperation2consumed/healed1HP; rawcombinednativecampaignFAILfromToad6versusserver5retained. Actualacknowledgedsave/restart/native reconnectpreservesEqua0debit/unselectedduplicate/Toadsstock/fineXP/base andfirstHP, nofoodreplay.'),'actual_native_sends':sent,'reconnect_hp':received,'saved_hp':after['stats'][3],'post_relogin_hp':post['stats'][3],'qualifications':['PG-DEF-008 blocksToadEat atoriginalpin; no positiveToadnativeconsumption orToadF2Pmembershipproof, no newerbuild/forcedoperation5substituted.','RawF2Pcontroller labels bothvisibleEatpaths BACKEND_REFUSAL; onlyEqua2reachesvalidfoodoperation. Toa6nonconsumptionisnotisolatedmembershipcoverage.','Source15/20bootstrap andpassiveHPchanges distinctfromfoodXP/healing; no ingredient-use/potiongameplay.','V0/V2FAILnativeoperationmismatch,V1FAIL18life,fullToadV3BLOCKED;passingEquascope doesnotupgradewholelevels.','Nativeframe inspection ifavailable isseparate; no renderedactor/audio parityclaim.'],'evidence_sha256':{n:hashlib.sha256((BASE/n).read_bytes()).hexdigest()for n in names}}
path=BASE/'independent-assessment.json';assert not path.exists();path.write_text(json.dumps(r,indent=2)+'\n',encoding='utf-8',newline='\n');print(a.feature,'Equa-only PASS_SCOPED;Toadnativeblocked')
