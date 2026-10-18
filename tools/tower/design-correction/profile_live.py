"""Opt-in timing on the preserved real map, with normal owned-server drain."""
from pathlib import Path
import sys,json,os,time,subprocess,csv,statistics,re
T=Path(__file__).resolve().parent;S=T.parents[2];E=S.parent;R=E/'cinderworks-design-correction-private'
sys.path.insert(0,str(E/'cinderworks-environment-server/tools/tower/environment-production'))
import native_session as n
def main():
 old_record=json.loads((n.R/'latest-native-session.json').read_text());old=Path(old_record['session'])
 before=n.state(old)if(old/'frontend-stopped.json').exists()else n.command(old,'observe');assert before['ready']
 if not(old/'frontend-stopped.json').exists():n.command(old,'stop')
 args=old_record['server_args'];stop=Path(args[args.index('--stop-file')+1]);stop.write_text('stop')
 for _ in range(150):
  if 'Shutdown completed; accounts drained.'in(old/'server.log').read_text():break
  time.sleep(.2)
 else:raise TimeoutError('Owned server drain')
 session=R/('native-profile-'+time.strftime('%Y%m%d-%H%M%S'));session.mkdir();env=os.environ.copy();env.update(old_record['client_env']);env.update(TOWER_CINDERWORKS_ENVIRONMENT=str(n.R/'placements/07-whole-floor-refinement.tsv'),TOWER_ACTION_DIRECTORY=str(session),TOWER_RENDER_METRICS=str(session/'renderer-metrics.json'),OSRS_PRESENT_TRACE=str(session/'presentation.csv'),TOWER_CORRECTION_PROFILE='1')
 args=list(args);args[args.index('--stop-file')+1]=str(session/'server.stop');sf=(session/'server.log').open('w');server=subprocess.Popen(args,cwd=E/'cinderworks-environment-server',env=env,stdout=sf,stderr=subprocess.STDOUT,creationflags=getattr(subprocess,'CREATE_NO_WINDOW',0));time.sleep(3)
 ca=list(old_record['client_args']);ca[0]=str(E/'build/enhanced/release/cinderworks-profile-live.exe');(session/'command.txt').write_text('1 observe');cf=(session/'client.log').open('w');start=time.monotonic();client=subprocess.Popen(ca,cwd=session,env=env,stdout=cf,stderr=subprocess.STDOUT)
 record=dict(old_record,session=str(session),server_pid=server.pid,client_pid=client.pid,server_args=args,client_args=ca,client_binary=ca[0],client_sha256=n.sha(Path(ca[0])),client_env={**old_record['client_env'],'TOWER_ACTION_DIRECTORY':str(session),'TOWER_RENDER_METRICS':str(session/'renderer-metrics.json')},scope='Only timing instrumentation; same preserved prototype map, account and server binary. No design-correction scene loaded.')
 (session/'owned-session.json').write_text(json.dumps(record,indent=2));(R/'latest-native-session.json').write_text(json.dumps(record,indent=2));(n.R/'latest-native-session.json').write_text(json.dumps(record,indent=2))
 for _ in range(100):
  state=n.command(session,'observe')
  if state['ready']:break
  time.sleep(.5)
 else:raise TimeoutError('Profile live ready')
 ready_ms=(time.monotonic()-start)*1000;n.command(session,'camera 2000 0');deadline=time.monotonic()+150
 while time.monotonic()<deadline:
  try:rows=list(csv.DictReader((session/'presentation.csv').open()));rows=[q for q in rows if float(q['start_ms'])>ready_ms+5000]
  except (FileNotFoundError,PermissionError,ValueError):rows=[]
  if len(rows)>=128:break
  time.sleep(.75)
 else:raise TimeoutError('Post-ready frame window')
 rows=rows[-128:];after=n.command(session,'observe');assert not after['modals']
 for key in ['position','xp','journal']:assert before[key]==after[key],key
 assert[(q['slot'],q['id'],q['count'])for q in before['items']]==[(q['slot'],q['id'],q['count'])for q in after['items']]
 with(session/'measured-frames.csv').open('w',newline='')as f:w=csv.DictWriter(f,fieldnames=rows[0].keys());w.writeheader();w.writerows(rows)
 periods=sorted(float(b['start_ms'])-float(a['start_ms'])for a,b in zip(rows,rows[1:]));log=(session/'client.log').read_text();timings=[(float(a),float(b))for a,b in re.findall(r'menu_ms=([\d.]+) interface_and_script_ms=([\d.]+)',log)]
 report={'scope':record['scope'],'session':session.name,'frames':len(rows),'post_ready_warmup_ms':5000,'first_start_ms':float(rows[0]['start_ms']),'ready_observed_process_ms':ready_ms,'median_frame_period_ms':statistics.median(periods),'p95_frame_period_ms':periods[int(.95*(len(periods)-1))],'stage_mean_ms':{k:statistics.mean(float(q[k])for q in rows)for k in ['network_ms','scripts_ms','world_ms','ui_ms','present_ms']},'timing_samples':len(timings),'context_menu_mean_ms':statistics.mean(q[0]for q in timings)if timings else None,'remaining_interface_script_mean_ms':statistics.mean(q[1]for q in timings)if timings else None,'renderer':json.loads((session/'renderer-metrics.json').read_text()),'persistent_state_preserved':True,'failed_presentations':sum(q['success']!='true'for q in rows)}
 (session/'profile-result.json').write_text(json.dumps(report,indent=2));(R/'profile-result.json').write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2),flush=True)
if __name__=='__main__':main()
