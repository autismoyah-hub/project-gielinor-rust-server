"""Deterministic scenario verdicts. Exit zero alone never proves gameplay success."""
import argparse,json,re
from pathlib import Path
RULES={
 'login':['Authenticated; local index','World rebuild:','Received '],
 'actions':['Live native login accepted','Server confirmed player movement','Server interface changes after Talk-to:'],
 'dialogue-ui':['Native Continue'],
 'arrival':['Live native login accepted','Arrival walk submitted: [3222, 3218]','Server confirmed first Tower arrival: [10424, 120]','Server confirmed first-arrival inventory and equipment cleared','Scenario passed; arrival first;'],
 'inventory':['Server confirmed equip and appearance','Server confirmed unequip and item conservation','Server confirmed final equipped state'],
 'inventory-verify':['Native login restored worn sword and its appearance'],
 'prayer':['Prayer actions ready after','Client observed Thick Skin active: bit4104=1','Client observed Thick Skin inactive: bit4104=0','Observed server prayer requirement notice:','Prayer scenario observed activation, deactivation, requirement notice and native Continue closure'],
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
        clicks=re.findall(r'Native Continue click[^\r\n]*handled=true, outbound=\[([\d, ]*)\]',log)
        emitted=any(115 in [int(op.strip()) for op in ops.split(',') if op.strip()] for ops in clicks)
        if not (emitted and 'Server NPC response:' in log and 'Scenario passed; protocol_probe=false' in log and meta.get('clientExit')==0):status='FAIL';reason='Native Continue success not observed'
        if not emitted and 'Native Continue click' not in log:
            if 'Talk-to man definition' not in log:reason='NPC target selection not observed; Continue UNKNOWN'
            elif 'confirmed generic-person greeting' not in log:reason='NPC greeting not observed; Continue UNKNOWN'
        if 'Live native login accepted' not in log:status='BLOCKED';reason='Authentication not observed; Continue was not exercised'
    if mode in ('inventory','inventory-verify') and 'Live native login accepted' not in log:
        status='BLOCKED';reason='Authentication not observed; inventory behavior UNKNOWN'
    elif mode=='inventory' and 'Native interface operation' not in log:
        status='FAIL';reason='Native inventory action dispatch not observed; server operation behavior UNKNOWN'
    if mode=='prayer':
        operations=re.findall(r'Native prayer operation: component=(\d+), outbound=\[([\d, ]*)\]',log)
        dispatched=[int(component) for component,ops in operations if 47 in [int(op.strip()) for op in ops.split(',') if op.strip()]]
        continues=re.findall(r'Native prayer Continue: outbound=\[([\d, ]*)\]',log)
        continued=any(115 in [int(op.strip()) for op in ops.split(',') if op.strip()] for ops in continues)
        if 'Prayer actions ready after' not in log:
            status='BLOCKED';reason='Authenticated prayer readiness not observed; prayer behavior UNKNOWN'
        elif dispatched!=[(541<<16)|9,(541<<16)|9,(541<<16)|10] or not continued:
            status='FAIL';reason='Expected native prayer dispatch sequence not observed; server operation behavior UNKNOWN'
    if not meta or not end:status='BLOCKED';reason='Run did not produce complete lifecycle evidence'
    elif end.get('forced') is not False or end.get('exitCode')!=0:status='FAIL';reason='Server did not terminate cleanly'
    limitations=[]
    persistence=read('logout-observation.json')
    if persistence.get('save_completed') is not True:
        limitations.append('Logout persistence completion UNKNOWN or incomplete')
        if status=='PASS':status='BLOCKED';reason='Targeted actions passed but logout save completion not observed'
    if 'Unhandled ' in log or 'Pending: packet' in log:limitations.append('Client reports unhandled packets; targeted pass is not complete packet coverage')
    if mode=='actions':limitations.append('Headless direct world-action dispatch; not graphical pointer acceptance')
    if mode=='render':limitations.append('Automated frame smoke; manual screenshot review is separate')
    if mode=='prayer':limitations.append('Prayer bits are client observations and may be predicted; activation acceptance and rendered appearance require separate evidence')
    return {'run_id':run.name,'scenario':mode,'status':status,'reason':reason,'first_failing_action':errors[0] if errors else (missing[0] if missing else None),'expected':RULES.get(mode,[]),'missing':missing,'client_sha256':meta.get('clientHash'),'evidence':[str(run/'client.log'),str(run/'result.json'),str(run/'shutdown.json')],'limitations':limitations,'reproduction':f'scripts/integration-preparation-run.ps1 -Environment ENV -Run new-unique-id -Mode {mode}'}
def main():
    p=argparse.ArgumentParser();p.add_argument('run',type=Path);p.add_argument('output',type=Path);a=p.parse_args();result=assess(a.run)
    with a.output.open('x',encoding='utf8') as f:json.dump(result,f,indent=2)
    print(json.dumps({'run':a.run.name,'status':result['status'],'reason':result['reason']}))
if __name__=='__main__':main()
