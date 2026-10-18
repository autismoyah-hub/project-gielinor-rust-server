"""Exact saved source, independent decoder, terrain IDs and source isolation."""
from pathlib import Path
import json,hashlib,subprocess,sys,collections,re,statistics
T=Path(__file__).resolve().parent;S=T.parents[2];E=S.parent;W=E.parents[1];P=S/'art/tower/cinderworks-design-correction';R=E/'cinderworks-design-correction-private'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 checks=[];blender='C:/Program Files/Blender Foundation/Blender 5.2/blender.exe'
 for source in sorted((P/'studies').iterdir()):
  if not source.is_dir():continue
  out=R/'source-verification'/source.name;out.mkdir(parents=True,exist_ok=True)
  args=[blender,'-b',str(source/'study.blend'),'--python-exit-code','1','--python',str(T/'reexport_study.py'),'--',str(source),str(out)]
  v=subprocess.run(args,capture_output=True,text=True);(out/'blender.log').write_text(v.stdout+v.stderr);v.check_returncode();assert sha(source/'architecture.model')==sha(out/'asset.model'),source.name
  # The independently built native workbench decodes only our original .model.
  camera=out/'decode.camera';camera.write_text('0 -84 0 420');decoded=out/'native-decode'
  v=subprocess.run([str(E/'astra-benchmark/tools/asset_workbench.exe'),str(E/'live-client-cache-v1'),str(decoded),'render',str(source/'architecture.model'),str(camera),'72','850'],capture_output=True,text=True);(out/'decode.log').write_text(v.stdout+v.stderr);v.check_returncode()
  a=json.loads((source/'geometry.json').read_text());b=json.loads((decoded/'decoded.json').read_text());assert all(a[k]==b[k]for k in ['vertices','faces','colors','render_types'])
  lines=(source/'study.scene').read_text().splitlines();floor_ids={int(q.split()[2])for q in lines if q.startswith('P U ')}
  ground_ids={int(q.split()[4])-1 for q in lines if q.startswith('A ')};assert ground_ids<=floor_ids;assert all(5000<=i<=32000 for i in floor_ids)
  assert not any(q.startswith(('T ','N '))for q in lines),'Protected cache terrain prohibited in published original studies'
  assert json.loads((source/'conversion.json').read_text())['texture_image_samples']==0
  checks.append({'study':source.name,'saved_source_native_bytes_exact':True,'independent_decode_exact':True,'underlay_definition_ids_valid':True,'protected_cache_geometry_or_terrain':False,'model_sha256':sha(source/'architecture.model'),'source_blend_sha256':sha(source/'study.blend'),'vertices':len(a['vertices']),'triangles':len(a['faces'])});print('DESIGN_SOURCE_VERIFIED',source.name,flush=True)
 # Protected prototype and other agents' checkouts are read-only to this task.
 repos={}
 expected={'cinderworks-environment-server':'75e8ce364013756754873cc037b18d6a85c8a266','cinderworks-environment-client':'29d665a9e41995632d942cea27e2f6782eca561d','rust-server':'f9e4433f2451235805f85fe379d351d095b7535b','rust-client-reference':'ff9cef17fd1ce3ec2ebf8d1181fee31411999c4c'}
 for name,head in expected.items():
  root=W/name if name.startswith('rust-')else E/name;actual=subprocess.check_output(['git','-C',str(root),'rev-parse','HEAD'],text=True).strip();status=subprocess.check_output(['git','-C',str(root),'status','--porcelain'],text=True).strip();assert actual==head and not status,(name,status);repos[name]={'head':head,'unchanged_head':True,'clean':True}
 profile=json.loads((R/'profile-result.json').read_text());assert profile['persistent_state_preserved']and profile['timing_samples']>=10
 session=R/profile['session'];log=(session/'client.log').read_text();cycles=[int(v)for v in re.findall(r'CORRECTION_PROFILE cycle=(\d+)',log)];increments=collections.Counter(b-a for a,b in zip(cycles,cycles[1:]))
 timings=[(float(a),float(b))for a,b in re.findall(r'menu_ms=([\d.]+) interface_and_script_ms=([\d.]+)',log)]
 profile.update(timing_samples=len(timings),context_menu_mean_ms=statistics.mean(q[0]for q in timings),remaining_interface_script_mean_ms=statistics.mean(q[1]for q in timings),cpu_interval_sample_scope='Bounded first256loggedframes after server_tick60; full-client trace is a separate128framepost-ready window on the same steady scene.')
 profile['observed_client_cycle_increments']=dict(increments);profile['diagnosis']='Most time is in the interface/script interval, with repeated10tick catch-up steps. Menu construction is a smaller measured part. GPU execution is much shorter than the full frame. CPU world submission/readback also needs profiling. These observations identify a client task; they do not prove a particular safe algorithmic fix.'
 profile['comparison_limit']='Instrumented sample on the same shared development PC is not a controlled A/B test against the earlier214ms sample. No performance improvement or regression attribution is claimed.'
 (P/'performance-investigation.json').write_text(json.dumps(profile,indent=2)+'\n')
 (P/'validation.json').write_text(json.dumps({'original_studies':checks,'preserved_checkouts':repos,'new_runtime_gameplay_map':False,'new_npc_art_or_ai':False,'full_floor_expansion':False,'native_playability_accepted':False,'checkpoint':'Three representative static native-rendered fragments and one proposed full-floor plan only'},indent=2)+'\n')
 print('CHECKPOINT_SOURCE_VALIDATED',len(checks),'studies; protected checkouts unchanged')
if __name__=='__main__':main()
