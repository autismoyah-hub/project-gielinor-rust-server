"""Reconnect only this workbench's owned client; preserve its real server/account."""
import csv,json,os,subprocess,time,statistics
from pathlib import Path
import native_session as n

def run(mode):
    previous=json.loads((n.R/'latest-native-session.json').read_text());old=Path(previous['session'])
    before=n.command(old,'observe');n.command(old,'stop')
    origin=Path(previous.get('original_session',str(old)))
    if not before['ready']:before=json.loads((origin/'state.json').read_text())
    # Closing a frontend uses disconnect grace. Drain the owned isolated server
    # normally between modes instead of editing account admission/persistence.
    args=previous['server_args'];stop=Path(args[args.index('--stop-file')+1]);stop.write_text('stop')
    log_path=old/'server.log'
    if not log_path.exists():log_path=origin/'server.log'
    for _ in range(100):
        if 'Shutdown completed; accounts drained.'in log_path.read_text():break
        time.sleep(.2)
    else:raise TimeoutError('Owned server normal drain')
    stamp=time.strftime('%Y%m%d-%H%M%S');session=n.R/('native-'+stamp+'-'+mode);session.mkdir()
    env=os.environ.copy();env.update(previous['client_env']);env.update(TOWER_ACTION_DIRECTORY=str(session),TOWER_RENDERER=mode,TOWER_GPU_ADAPTER='0'if mode=='lite'else'1',TOWER_RENDER_METRICS=str(session/'renderer-metrics.json'),OSRS_PRESENT_TRACE=str(session/'presentation.csv'))
    env['TOWER_CINDERWORKS_ENVIRONMENT']=str(n.R/'placements/07-whole-floor-refinement.tsv')
    args=list(args);args[args.index('--stop-file')+1]=str(session/'server.stop')
    sf=(session/'server.log').open('w');server=subprocess.Popen(args,cwd=n.S,env=env,stdout=sf,stderr=subprocess.STDOUT,creationflags=getattr(subprocess,'CREATE_NO_WINDOW',0));time.sleep(3)
    (session/'command.txt').write_text('1 observe');log=(session/'client.log').open('w')
    process=subprocess.Popen(previous['client_args'],cwd=session,env=env,stdout=log,stderr=subprocess.STDOUT)
    record=dict(previous,session=str(session),server_pid=server.pid,server_args=args,client_pid=process.pid,mode=mode,client_env={k:env[k]for k in previous['client_env']},original_session=str(origin))
    (session/'owned-session.json').write_text(json.dumps(record,indent=2));(n.R/'latest-native-session.json').write_text(json.dumps(record,indent=2))
    end=time.monotonic()+90
    while time.monotonic()<end:
        if process.poll()is not None:raise RuntimeError('Owned client exited')
        state=n.command(session,'observe',20)
        if state['ready']:break
        time.sleep(1)
    else:raise TimeoutError('Reconnect ready')
    n.command(session,'camera 2000 0');time.sleep(2)
    trace_path=session/'presentation.csv'
    def rows():
        try:return list(csv.DictReader(trace_path.open()))
        except (FileNotFoundError,PermissionError):return []
    initial=rows();start_frame=int(initial[-1]['frame'])if initial else 0
    deadline=time.monotonic()+120
    while time.monotonic()<deadline:
        current=rows()
        if current and int(current[-1]['frame'])>=max(384,start_frame+256):break
        time.sleep(.75)
    else:raise TimeoutError('Stable frame sample')
    after=n.command(session,'observe');capture=n.command(session,'capture');time.sleep(1)
    # Runtime widget/menu fields appear only after onboarding. Compare saved
    # inventory identity/count/slot, rather than incidental UI widget handles.
    def persistent(v,k):
        return [(q['slot'],q['id'],q['count'])for q in v[k]]if k in ['items','equipment']else v[k]
    keys=['position','items','equipment','xp','journal'];checks={k:persistent(before,k)==persistent(after,k)for k in keys};assert all(checks.values()),checks
    (session/'persistence-check.json').write_text(json.dumps(checks,indent=2));(session/'review-state.json').write_text(json.dumps(after,indent=2))
    trace=rows()[-256:]
    with (session/'measured-frames.csv').open('w',newline='')as f:
        writer=csv.DictWriter(f,fieldnames=trace[0].keys());writer.writeheader();writer.writerows(trace)
    periods=[float(b['start_ms'])-float(a['start_ms'])for a,b in zip(trace,trace[1:])];periods.sort()
    work=[sum(float(row[k])for k in ['network_ms','scripts_ms','world_ms','ui_ms','present_ms'])for row in trace]
    result={'mode':mode,'session':str(session),'frames':len(trace),'frame_period_ms':{'median':statistics.median(periods),'p95':periods[int(.95*(len(periods)-1))]},'frame_work_mean_ms':statistics.mean(work),'failed_frames':sum(row['success']!='true'for row in trace),'persistence_checks':checks,'capture_sequence':capture['sequence'],'scope':'Same sanctuary tile, camera 2000/0, 1280x900, same real account/database. Owned isolated server normally drained/restarted between modes after direct reconnect admission failed. Rolling trace after warmup; not a multi-room FPS guarantee.'}
    if mode!='classic':result['renderer']=json.loads((session/'renderer-metrics.json').read_text())
    (session/'mode-result.json').write_text(json.dumps(result,indent=2));print(json.dumps({k:v for k,v in result.items()if k!='renderer'}),flush=True)
    return result

if __name__=='__main__':
    import sys
    run(sys.argv[1])
