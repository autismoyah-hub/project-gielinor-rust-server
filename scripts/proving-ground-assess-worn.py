"""Assess an exact-target native worn-amulet Options/relogin campaign."""
import argparse,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('sha');p.add_argument('--directory');a=p.parse_args();assert re.fullmatch('[0-9a-f]{40}',a.sha)
OUT=(ROOT/a.directory).resolve() if a.directory else ROOT/'docs/openrune/verification/independent'/('native-chemistry-worn-'+a.sha)
assert OUT.is_relative_to((ROOT/'docs/openrune/verification/independent').resolve())
def load(path):return json.loads(path.read_text(encoding='utf-8'))
produce=load(OUT/'produce/manifest.json');before=load(OUT/'produce/before.json');after=load(OUT/'produce/after.json')
relogin=load(OUT/'relogin/manifest.json');pre=load(OUT/'relogin/before.json');post=load(OUT/'relogin/after.json')
assert produce['status']==relogin['status']=='PASS'
kind=produce['campaign_kind'];assert relogin['campaign_kind']==kind and kind in ['historical','integration']
assert produce['tested_sha']==relogin['tested_sha']==a.sha
assert before['inventory']==after['inventory']==post['inventory']==[[93,27,995,1000,0],[94,2,21163,1,0]]
assert before['chemistry_state']==[[2,0]] and after['chemistry_state']==post['chemistry_state']==[[2,1]]
assert before['stats']==after['stats']==post['stats'] and pre==after
assert after['integrity_check']==post['integrity_check']=='ok'
for i in [0,1,2,3,6]:assert pre['character'][i]==post['character'][i]
old=json.loads(pre['character'][4]);new=json.loads(post['character'][4]);changed={k:{'before':old.get(k),'after':new.get(k)} for k in old.keys()|new.keys() if old.get(k)!=new.get(k)}
assert set(changed)<={'300','3077','3079','65515'},changed
text=(OUT/'produce/client.log').read_text(encoding='utf-8')
assert 'Native equipment tab selected by label=Worn Equipment' in text
packets=[]
for label,opcode,raw in re.findall(r'Native chemistry worn (Options|Stop) dispatch opcode=(\d+) payload=\[([^\]]+)\]',text):
    packets.append((label,int(opcode),list(bytes(int(x.strip(),16) for x in raw.split(',')))))
assert packets==[('Options',47,[1,131,0,17,255,255,255,255,2]),('Stop',115,[219,0,1,0,0,2])]
assert 'verify=false' in text and 'verify=true' in (OUT/'relogin/client.log').read_text(encoding='utf-8')
report={'implementation_id':'OR-MIG-0006','contract_implementation_sha':produce['implementation_sha'],'tested_sha':a.sha,'campaign_kind':kind,
        'status':'PASS','scope':'Native '+kind+' worn Options -> Stop, equipped-item/coins/XP conservation and actual restart/native relogin; complete V2/V3 acceptance remains open.',
        'native_menu_packet_state':'PASS','real_restart_and_relogin':'PASS','worn_item_and_durable_setting':'PASS',
        'packets':packets,'character_session_varp_changes':changed,'limitations':(['No inferred historical worn-Options acceptance at the earlier implementation SHA'] if kind=='integration' else [])+['No stale worn packet, crash injection or rendered-window assertion'],
        'source_and_binary_evidence':['build.json','resources.json','produce/manifest.json','relogin/manifest.json']}
(OUT/'independent-assessment.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(report,indent=2))
