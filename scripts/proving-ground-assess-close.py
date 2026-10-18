"""Assess native modal cancellation using authoritative saved state."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'docs/openrune/verification/independent/deep-chem-close/close-001'
def load(p):return json.loads((OUT/p).read_text(encoding='utf-8'))
m=load('produce/manifest.json');before=load('produce/before.json');after=load('produce/after.json')
assert m['tested_sha']==m['implementation_sha']=='e7ac3608e61893d80b2de6a1a2e512b3ab785505'
assert m['status']=='PASS' and m['client_exit']==m['server_exit']==0
assert m['save_acknowledgement'] and m['graceful_shutdown'] and not m['forced_shutdown'] and m['resource_hashes_unchanged']
log=(OUT/'produce/client.log').read_text(encoding='utf-8')
assert 'label="genuine native Close" tick=129 opcode=95 payload=[]' in log
assert 'label="stale Stop after genuine Close" tick=134 opcode=115 payload=[db, 00, 01, 00, 00, 02]' in log
assert 'PG_PROBE_COMPLETE' in log
assert before['inventory']==after['inventory'] and before['stats']==after['stats']
assert before['chemistry_state']==[[2,0]] and after['chemistry_state']==[[2,1]]
assert after['integrity_check']=='ok'
r={'implementation_id':'OR-MIG-0006','tested_sha':m['tested_sha'],'campaign_kind':'historical','status':'FAIL','finding':'PG-DEF-001','level':'V2',
 'raw_probe_execution':'PASS: all requested interactions executed; this is not a gameplay verdict',
 'expected_chemistry_state':[[2,0]],'actual_chemistry_state':after['chemistry_state'],
 'scope':'Genuine native Action::Close emits opcode95 and closes local modal; replay of old real Continue packet with option changed to Stop is accepted by historical server. Saved Stop policy changes despite closed native modal. Items, charges and XP unchanged.',
 'root_cause':'Recorded server gameplay decoder treats opcode95 as Unhandled; existing chemistry-dialog cancellation is not reached.',
 'severity':'P2 ordinary isolated stale-modal mismatch','implementation_blocking':False,
 'correction':'Decode and route real Close modal to existing modal/chemistry cancellation; refuse old held/worn ResumePause afterward. Preserve ordinary valid controls and test same exact interaction at an explicitly published correction SHA.'}
(OUT/'independent-assessment.json').write_text(json.dumps(r,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(r))
