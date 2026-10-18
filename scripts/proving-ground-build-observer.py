"""Build owned native observers from the hash-verified pinned client source."""
import argparse,hashlib,importlib.util,json,os,shutil,subprocess,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];WORK=ROOT/'work/independent-native-observer';SOURCE=WORK/'source'
BASE=ROOT.parent/'rust-server/integration/client-baselines/client-239.1-baseline-001'
OUT=ROOT/'docs/openrune/verification/independent/client-observer-build';OUT.mkdir(parents=True,exist_ok=True)
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def save(p,o):p.write_text(json.dumps(o,indent=2)+'\n',encoding='utf-8',newline='\n')
p=argparse.ArgumentParser();p.add_argument('action',choices=['prepare','build']);a=p.parse_args()
if a.action=='prepare':
    assert not SOURCE.exists(),'Never overwrite prepared observer source'
    manifest=json.loads((BASE/'manifest.json').read_text(encoding='utf-8'));hashes={}
    for name,expected in manifest['files'].items():
        if not name.startswith('source/'):continue
        origin=BASE/name;assert digest(origin)==expected.lower(),name
        relative=Path(name).relative_to('source');dest=SOURCE/relative;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(origin,dest);assert digest(dest)==expected.lower();hashes[relative.as_posix()]=expected.lower()
    cargo=SOURCE/'Cargo.toml';cargo.write_text(cargo.read_text(encoding='utf-8')+'\n[workspace]\n',encoding='utf-8')
    runtime=SOURCE/'src/live_runtime.rs';text=runtime.read_text(encoding='utf-8')
    needle='                Event::Packet(packet) => {';assert text.count(needle)==1
    instrumentation='''
                    if matches!(packet.opcode,15|22|46|83) {
                        println!("PG_RX tick={} opcode={} payload={:02x?}",client.server_tick,packet.opcode,packet.payload);
                    }
'''
    text=text.replace(needle,needle+instrumentation)
    needle='                                if let Some((scene, _)) = &self.world {';assert text.count(needle)==1
    instrumentation2='''                                if matches!(packet.opcode,15|22|46|83) {
                                    println!("PG_APPLIED tick={} opcode={} hp={} xp={} busy={} scripts={} inventory={:?}",client.server_tick,packet.opcode,client.data.boosted[3],client.data.experience[3],self.scripts_busy(),client.interfaces.scripts.len(),(0..28).map(|s|{let i=client.inventories.item(93,s);(s,i.id,i.count)}).collect::<Vec<_>>());
                                }
'''
    text=text.replace(needle,instrumentation2+needle);runtime.write_text(text,encoding='utf-8',newline='\n')
    save(OUT/'source-manifest.json',{'baseline_manifest_sha256':digest(BASE/'manifest.json'),'verified_source_files':hashes,
      'owned_overlays':{'Cargo.toml':'Empty workspace for isolation','src/live_runtime.rs':'Observability only: log decoded inventory/stat/server-clock packet bytes and post-apply HP/inventory/script-busy state; no state mutation or gameplay substitution'},'instrumented_runtime_sha256':digest(runtime)})
    print('PREPARED',len(hashes),'verified source/vendor files',flush=True)
else:
    if (OUT/'build.json').exists():
        history=OUT/('attempt-'+str(time.time_ns()));history.mkdir()
        for name in ['build.json','build.log']:shutil.copyfile(OUT/name,history/name)
        for p in (SOURCE/'examples').glob('pg_observer.rs'):shutil.copyfile(p,history/p.name)
    spec=importlib.util.spec_from_file_location('capacity',ROOT/'scripts/proving-ground-native.py');capacity=importlib.util.module_from_spec(spec);spec.loader.exec_module(capacity);capacity.OUT=OUT;resource=capacity.guard()
    examples=[]
    for src in (ROOT/'verification/native-observer').glob('*.rs'):
        shutil.copyfile(src,SOURCE/'examples'/src.name);examples+=['--example',src.stem]
    assert examples,'No observer scenarios'
    for name in ['cargo-home','target','tmp']:(WORK/name).mkdir(parents=True,exist_ok=True)
    env=os.environ.copy();env.update(CARGO_HOME=str(WORK/'cargo-home'),CARGO_TARGET_DIR=str(WORK/'target'),TEMP=str(WORK/'tmp'),TMP=str(WORK/'tmp'),PATH=str(capacity.TOOL)+os.pathsep+env['PATH'],CARGO_PROFILE_DEV_DEBUG='0',CARGO_PROFILE_DEV_INCREMENTAL='false',CARGO_PROFILE_DEV_CODEGEN_UNITS='2')
    cmd=[str(capacity.TOOL/'cargo.exe'),'build','--offline','--locked','-j','1',*examples];print('BUILD',cmd,flush=True)
    with (OUT/'build.log').open('w',encoding='utf-8',newline='\n') as f:rc=subprocess.run(cmd,cwd=SOURCE,env=env,stdout=f,stderr=subprocess.STDOUT,timeout=600,creationflags=subprocess.BELOW_NORMAL_PRIORITY_CLASS).returncode
    save(OUT/'build.json',{'status':'PASS' if rc==0 else 'BLOCKED','exit_code':rc,'resource':resource,'command':cmd,
      'observer_sources':{p.name:digest(p) for p in (ROOT/'verification/native-observer').glob('*.rs')},
      'binaries':{p.name:digest(p) for p in (WORK/'target/debug/examples').glob('*.exe')},'scope':'Fresh independent native-client build; pinned core source plus recorded read-only observation overlay and owned scenario controllers.'})
    print('BUILD_EXIT',rc,flush=True);raise SystemExit(rc)
