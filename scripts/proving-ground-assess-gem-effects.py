"""Correlate real decoded animation blocks with raw selected inventory credits."""
import json,re,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/deep-gem-effects/effects-001';SHA='5743cbe1c7499e2a7476c721d25dcba194666042'
profiles=[(1623,1607,2500),(1621,1605,3375),(1619,1603,4250),(1617,1601,5375),(1631,1615,6875),(6571,6573,8375),(19496,19493,2500)]
expected={4+3*i+j:(raw,cut,award) for i,(raw,cut,award) in enumerate(profiles) for j in [0,1]};items={};sequences=[];credits=[]
for line in (OUT/'produce/client.log').read_text(encoding='utf-8').splitlines():
    sequence=re.fullmatch(r'PG_SEQUENCE tick=(\d+) sequence=(\d+) delay=(\d+)',line)
    if sequence and int(sequence[2])==8833:sequences.append({'tick':int(sequence[1]),'sequence':int(sequence[2]),'delay':int(sequence[3])})
    m=re.fullmatch(r'PG_RX tick=(\d+) opcode=(\d+) payload=\[(.*)\]',line)
    if not m or int(m[2])!=18:continue
    p=bytes.fromhex(m[3].replace(',',''));inv=int.from_bytes(p[4:6],'big');pos=6
    while pos<len(p):
        if p[pos]<128:slot=p[pos];pos+=1
        else:slot=int.from_bytes(p[pos:pos+2],'big')-32768;pos+=2
        item=int.from_bytes(p[pos:pos+2],'big')-1;pos+=2;count=0
        if item>=0:
            count=p[pos];pos+=1
            if count==255:count=int.from_bytes(p[pos:pos+4],'big');pos+=4
        if inv==93 and slot in expected and item==expected[slot][1] and items.get(slot)!=item:
            assert count==1 and sequences
            s=sequences[-1];credits.append({'slot':slot,'cut':item,'animation_sequence':s['sequence'],'animation_delay':s['delay'],'animation_tick':s['tick'],'credit_tick':int(m[1]),'received_clock_delta':int(m[1])-s['tick']})
        if inv==93:items[slot]=item
    assert pos==len(p)
if '--inspect' in sys.argv:print(json.dumps({'observed_native_credits':len(credits),'sequences':len(sequences),'credits':credits}));raise SystemExit(0)
assert len(credits)==14 and len({c['slot'] for c in credits})==14
assert all(c['received_clock_delta']==1 and c['animation_delay']==0 for c in credits),credits
m=json.loads((OUT/'produce/manifest.json').read_text(encoding='utf-8'));assert m['tested_sha']==SHA and m['status']=='PASS' and m['resource_hashes_unchanged'] and m['save_acknowledgement'] and m['graceful_shutdown']
before=json.loads((OUT/'initial-state.json').read_text(encoding='utf-8'));after=json.loads((OUT/'produce/after.json').read_text(encoding='utf-8'))
expected_inv=[x.copy() for x in before['inventory']]
for x in expected_inv:
    if x[0]==93 and x[1] in expected:x[2]=expected[x[1]][1]
assert after['inventory']==expected_inv
for a,b in zip(before['stats'],after['stats']):assert a[:3]==b[:3] and b[3]-a[3]==(66500 if a[0]==12 else 0)
log=(OUT/'produce/client.log').read_text(encoding='utf-8');assert 'PG_CASE_PASS fourteen native gem cuts' in log
r={'implementation_id':'OR-MIG-0056','tested_sha':SHA,'campaign_kind':'historical','status':'PASS','native_cuts':14,'fine_xp_award':66500,'native_effect_observations':credits,
 'scope':'Every gem profile cut in both genuine native selected directions. Actual decoded local-player animation8833(delay0) precedes corresponding raw selected inventory18 credit by exactly one received SERVER_TICK boundary; all14 exact selected outputs, retained chisels/full bag and66500fineXP match authoritative saved state. First-phase stale-pair and movement cancellation checks remain strict.',
 'limitations':['This observes actual native interpreted effects and packet cadence, not a rendered-window/audio comparison. Missing chisel sound remains explicitly omitted by contract.','Received counter may lag absolute authoritative world clock; only relative ordered frame boundaries are compared.','Separate acknowledged restart and bounded crash evidence own V3.']}
(OUT/'independent-assessment.json').write_text(json.dumps(r,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(r))
