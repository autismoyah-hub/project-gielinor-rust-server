"""Deterministic scenario verdicts. Exit zero alone never proves gameplay success."""
import argparse,json,re
from pathlib import Path
RULES={
 'login':['Authenticated; local index','World rebuild:','Received '],
 'actions':['Live native login accepted','Server confirmed player movement','Server interface changes after Talk-to:'],
 'dialogue-ui':['Native Continue'],
 'inventory':['Server confirmed equip and appearance','Server confirmed unequip and item conservation','Server confirmed final equipped state'],
 'inventory-verify':['Native login restored worn sword and its appearance'],
 'floor':['Server confirmed floor waypoint 3','Server confirmed withdrawal and conservation'],
 'floor-content':['Server confirmed repeated vial Search granted ten vials total'],
 'floor-altar':['You recharge your Prayer points.'],
 'render':['Native interface window:','0 missing regions'],
}
def assess(run):
    read=lambda n:json.loads((run/n).read_text(encoding='utf-8-sig')) if (run/n).exists() else {}
    meta=read('result.json');end=read('shutdown.json');mode=meta.get('mode')
    log=(run/'client.log').read_text(encoding='utf-8-sig',errors='replace') if (run/'client.log').exists() else ''
    missing=[s for s in RULES.get(mode,[]) if s not in log]
    errors=[{'line':i,'category':'native_ui_continue'} for i,s in enumerate(log.splitlines(),1) if 'native Continue did not emit' in s]
    errors += [{'line':i,'category':'client_script_or_packet_error'} for i,s in enumerate(log.splitlines(),1) if s.startswith('Failed ') or 'unsupported' in s.lower() and s.startswith('Error:')]
    status='PASS';reason='All targeted assertions and clean shutdown observed'
    if mode not in RULES:status='NOT TESTED';reason='No registered deterministic rule'
    elif errors or meta.get('clientExit')!=0 or missing:status='FAIL';reason='Missing expected effect or explicit client failure'
    if mode=='dialogue-ui':
        # The current driver intentionally exits unsuccessfully on hidden Continue;
        # no generic log prefix counts as proof of a successful continuation.
        if not ('Native Continue click' in log and '115' in log and 'Scenario passed; protocol_probe=false' in log and meta.get('clientExit')==0):status='FAIL';reason='Native Continue success not observed'
    if not meta or not end:status='BLOCKED';reason='Run did not produce complete lifecycle evidence'
    elif end.get('forced') is not False or end.get('exitCode')!=0:status='FAIL';reason='Server did not terminate cleanly'
    limitations=[]
    if 'Unhandled ' in log or 'Pending: packet' in log:limitations.append('Client reports unhandled packets; targeted pass is not complete packet coverage')
    if mode=='actions':limitations.append('Headless direct world-action dispatch; not graphical pointer acceptance')
    if mode=='render':limitations.append('Automated frame smoke; manual screenshot review is separate')
    return {'run_id':run.name,'scenario':mode,'status':status,'reason':reason,'first_failing_action':errors[0] if errors else (missing[0] if missing else None),'expected':RULES.get(mode,[]),'missing':missing,'client_sha256':meta.get('clientHash'),'evidence':[str(run/'client.log'),str(run/'result.json'),str(run/'shutdown.json')],'limitations':limitations,'reproduction':f'scripts/integration-preparation-run.ps1 -Environment ENV -Run new-unique-id -Mode {mode}'}
def main():
    p=argparse.ArgumentParser();p.add_argument('run',type=Path);p.add_argument('output',type=Path);a=p.parse_args();result=assess(a.run)
    with a.output.open('x',encoding='utf8') as f:json.dump(result,f,indent=2)
    print(json.dumps({'run':a.run.name,'status':result['status'],'reason':result['reason']}))
if __name__=='__main__':main()
