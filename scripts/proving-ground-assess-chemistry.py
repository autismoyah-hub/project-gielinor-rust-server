"""Assert native chemistry controls and durable per-character state independently."""
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-chemistry'
def load(p):return json.loads(p.read_text(encoding='utf-8'))
before=load(OUT/'produce/before.json');after=load(OUT/'produce/after.json');manifest=load(OUT/'produce/manifest.json')
assert manifest['status']=='PASS' and manifest['resource_hashes_unchanged']
assert before['chemistry_state']==[[2,0]] and after['chemistry_state']==[[5,1]]
assert before['inventory']==[[93,3,21163,1,0],[93,27,995,1000,0]]
assert after['inventory']==[[93,27,995,1000,0]] and after['stats']==before['stats']
assert after['integrity_check']=='ok' and after['character'][6] is None
log=(OUT/'produce/client.log').read_text(encoding='utf-8')
actions=[]
for label,opcode,raw in re.findall(r'Native chemistry (Options|Stop|Break|Confirm) dispatch opcode=(\d+) payload=\[([^\]]+)\]',log):
    payload=bytes(int(x.strip(),16) for x in raw.split(','));actions.append((label,int(opcode),list(payload)))
expected=[('Options',47,[0,149,0,0,0,3,82,171,4]),('Stop',115,[219,0,1,0,0,2]),('Break',47,[0,149,0,0,0,3,82,171,6]),('Confirm',115,[219,0,1,0,0,1])]
assert actions==expected,actions
assert 'Options=Stop,Break=1,coins=1000;verify=false' in log
report={'implementation_id':'OR-MIG-0006','implementation_sha':manifest['implementation_sha'],
        'native_options_stop_confirmed_break':'PASS','durable_charge_and_stop_setting_save':'PASS',
        'exact_packet_sequence':actions,'initial_state':[2,0],'saved_state':[5,1],
        'scope':'Native inventory Options -> Stop -> Break -> Confirm. No production proc frequency or worn controls native coverage claimed.',
        'limitations':['Native stale/cancelled-dialog and worn-equipment scenarios remain queued','No crash injection or rendered-frame assertion']}
if (OUT/'relogin/after.json').exists():
    m=load(OUT/'relogin/manifest.json');pre=load(OUT/'relogin/before.json');post=load(OUT/'relogin/after.json')
    assert m['status']=='PASS' and pre==after
    for key in ['inventory','stats','chemistry_state']:assert post[key]==after[key],key
    assert post['chemistry_state']==[[5,1]] and post['character'][6] is None and post['integrity_check']=='ok'
    text=(OUT/'relogin/client.log').read_text(encoding='utf-8')
    assert 'verify=true' in text and 'Native chemistry Options dispatch' not in text
    previous=json.loads(pre['character'][4]);current=json.loads(post['character'][4])
    changed={k:{'before':previous.get(k),'after':current.get(k)} for k in previous.keys()|current.keys() if previous.get(k)!=current.get(k)}
    assert set(changed)<={'300','3077','3079','65515'},changed
    report['real_restart_and_native_relogin']='PASS';report['relogin_varp_changes']=changed
    report['varp_change_basis']='Target session_vars.rs timer processing: special energy, wall clock, process-relative map clock and accumulated playtime; all other varps stable.'
base=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main/content/skills/herblore/src/main/kotlin/org/rsmod/content/skills/herblore'
report['independent_source_comparison']={'status':'PASS','kind':'directly inspected source; not executable donor observation',
    'files':{name:hashlib.sha256((base/name).read_bytes()).hexdigest() for name in ['AmuletOfChemistry.kt','AmuletOfChemistryScript.kt']},
    'findings':['Source registers held Options/Break and worn Options','Options writes per-character stop-on-crumble selection','Confirmed Break deletes selected amulet and resets charges for the next amulet to five; stop setting remains selected'],
    'exclusions':['Alchemist amulets excluded from this ordinary-amulet contract','No executable OpenRune 241 comparison; full V4 production modifier behavior remains open']}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(report,indent=2))
