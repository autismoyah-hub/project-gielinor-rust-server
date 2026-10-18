"""Owned native client, authoritative server and disposable persistence campaign."""
import argparse, ctypes, hashlib, json, os, re, shutil, sqlite3, subprocess, time
from pathlib import Path
from datetime import datetime, timezone
ROOT=Path(__file__).resolve().parents[1]
SOURCE=ROOT.parent/'rust-server'
SHA='5a260c1bd6efa5f8a221bed7c96e6ac72fd428bc'
FEATURE='OR-MIG-0010'
STATE='pg-combat-native-001'
DRIVER='server_combatmixing.exe'
PIN='combatmixing-001'
SCENARIO='combat'
CONTRACT_SHA=SHA
CHECKOUT=ROOT.parent/'agent7-targets'/SHA
WORK=ROOT/'work/native-combat'
RES=WORK/'resources'
OUT=ROOT/'docs/openrune/verification/independent/deep-combat'
TOOL=ROOT.parent/'rust-client/work/rustup/toolchains/stable-x86_64-pc-windows-gnu/bin'
def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def save(path,obj):
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_text(json.dumps(obj,indent=2)+'\n',encoding='utf-8',newline='\n')
def git(*args):return subprocess.check_output(['git','-c','safe.directory='+CHECKOUT.as_posix(),*args],cwd=CHECKOUT,text=True).strip()
def guard():
    class Memory(ctypes.Structure):
        _fields_=[('length',ctypes.c_ulong),('load',ctypes.c_ulong)]+[(x,ctypes.c_ulonglong) for x in ['total','avail','page','availpage','virtual','availvirtual','extended']]
    m=Memory();m.length=ctypes.sizeof(m);ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(m))
    procs=subprocess.check_output(['tasklist','/FO','CSV','/NH'],text=True)
    active=[s for s in procs.splitlines() if s.lower().startswith(('"cargo.exe"','"rustc.exe"'))]
    def cpu_sample():
        idle=ctypes.c_ulonglong();kernel=ctypes.c_ulonglong();user=ctypes.c_ulonglong()
        assert ctypes.windll.kernel32.GetSystemTimes(ctypes.byref(idle),ctypes.byref(kernel),ctypes.byref(user))
        return idle.value,kernel.value+user.value
    samples=[]
    for _ in range(3):
        a=cpu_sample();time.sleep(1);b=cpu_sample();samples.append(100*(1-(b[0]-a[0])/max(1,b[1]-a[1])))
    cpu=sum(samples)/len(samples)
    report={'available_ram':m.avail,'active_compilers':active,'cpu_percent':round(cpu,2),'cpu_samples':samples,'cpu_ceiling':70,'policy':'Project resources.ps1 / integration-capacity.ps1 sampled 70% CPU; stricter owned 6 GiB RAM retained','disk_free':shutil.disk_usage(ROOT).free}
    print('RESOURCE',json.dumps(report),flush=True)
    # Other agents' activity is measured, not an automatic blanket deferral.
    # Match the actual project's sampled 70% ceiling instead of inventing a
    # 30% limit that repeatedly deferred eligible native runs. Retain RAM headroom.
    permitted=cpu<=70 and m.avail>=6*1024**3
    OUT.mkdir(parents=True,exist_ok=True)
    with (OUT/'resource-observations.jsonl').open('a',encoding='utf-8',newline='\n') as f:
        f.write(json.dumps({'observed_utc':datetime.now(timezone.utc).isoformat(),'decision':'PROCEED' if permitted else 'DEFER','measurement':report})+'\n')
    assert permitted,'Measured resource guard blocked; no other process altered'
    # Restrict only this owned launcher and its future children, as the project
    # resource scripts do. No other agent's affinity or priority is changed.
    api=ctypes.windll.kernel32;api.GetProcessAffinityMask.argtypes=[ctypes.c_void_p,ctypes.POINTER(ctypes.c_size_t),ctypes.POINTER(ctypes.c_size_t)]
    own=ctypes.c_size_t();system=ctypes.c_size_t();assert api.GetProcessAffinityMask(ctypes.c_void_p(-1),ctypes.byref(own),ctypes.byref(system))
    bits=[1<<i for i in range(ctypes.sizeof(ctypes.c_size_t)*8) if own.value&(1<<i)]
    mask=sum(bits[:2]);api.SetProcessAffinityMask.argtypes=[ctypes.c_void_p,ctypes.c_size_t];assert mask and api.SetProcessAffinityMask(ctypes.c_void_p(-1),mask)
    report['owned_affinity_mask']=mask
    return report
def environment():
    env=os.environ.copy();w=ROOT/'work'/SHA
    for name in ['cargo-home','target','tmp']:(w/name).mkdir(parents=True,exist_ok=True)
    env.update(CARGO_HOME=str(w/'cargo-home'),CARGO_TARGET_DIR=str(w/'target'),TEMP=str(w/'tmp'),TMP=str(w/'tmp'),CARGO_BUILD_JOBS='1',PATH=str(TOOL)+os.pathsep+env['PATH'])
    return env
def prepare():
    WORK.mkdir(parents=True,exist_ok=True);OUT.mkdir(parents=True,exist_ok=True)
    assert git('rev-parse','HEAD')==SHA and not git('status','--porcelain','--untracked-files=no')
    identity={}
    for sub in ['cache/game','cache/js5','symbols']:
        source=SOURCE/'work/integration/a-kotlin/.data'/sub;dest=RES/sub
        dest.mkdir(parents=True,exist_ok=True)
        for p in source.rglob('*'):
            if not p.is_file():continue
            relative=p.relative_to(source);target=dest/relative;target.parent.mkdir(parents=True,exist_ok=True)
            expected=digest(p)
            if not target.exists():shutil.copyfile(p,target)
            assert digest(target)==expected
            identity[sub+'/'+relative.as_posix()]=expected
    for name in ['exponent.bin','modulus.bin','client.key']:
        target=RES/'keys'/name;target.parent.mkdir(exist_ok=True)
        if not target.exists():shutil.copyfile(SOURCE/'work/integration/keys'/name,target)
    for source,target in [(SOURCE/'work/integration/test-account.password',RES/'password'),(SOURCE/'work/native/argon2.dll',RES/'argon2.dll')]:
        if not target.exists():shutil.copyfile(source,target)
        assert digest(source)==digest(target)
    if PIN=='pg-observer':
        build=json.loads((ROOT/'docs/openrune/verification/independent/client-observer-build/build.json').read_text(encoding='utf-8'));assert build['status']=='PASS'
        assert build['observer_sources']['pg_observer.rs']==digest(ROOT/'verification/native-observer/pg_observer.rs'),'Current scenario controller has not built successfully; preserve existing frozen runs and defer new preparation'
        assert build['observation_manifest_sha256']==digest(ROOT/'docs/openrune/verification/independent/client-observer-build/source-manifest.json'),'Current observation overlay has not built successfully; do not substitute an older driver'
        pin=ROOT/'work/independent-native-observer/target/debug/examples';expected=build['binaries']['pg_observer.exe'];assert digest(pin/'pg_observer.exe')==expected
        manifest={'origin':'independently rebuilt pinned native client with observation-only overlay','build_evidence':'docs/openrune/verification/independent/client-observer-build/build.json','files':{DRIVER:expected}}
        origin=pin/'pg_observer.exe'
    else:
        pin=SOURCE/'integration/scenario-drivers'/PIN
        manifest=json.loads((pin/'manifest.json').read_text())
        for name,expected in manifest['files'].items():assert digest(pin/name)==expected.lower()
        origin=pin/DRIVER
    driver=RES/DRIVER
    if not driver.exists():shutil.copyfile(origin,driver)
    assert digest(driver)==manifest['files'][driver.name].lower()
    if SCENARIO=='food':
        baseline=SOURCE/'integration/client-builds/client-239.1-baseline-001-build-001'
        probe=json.loads((baseline/'manifest.json').read_text(encoding='utf-8'))
        for name,expected in probe['files'].items():assert digest(baseline/name)==expected.lower()
        target=RES/'pg-live-session.exe'
        if not target.exists():shutil.copyfile(baseline/'bin/live_session.exe',target)
        assert digest(target)==probe['files']['bin/live_session.exe'].lower()
        save(OUT/'relogin-client.json',{'binary_sha256':digest(target),'baseline_manifest_sha256':digest(baseline/'manifest.json'),
          'build_attestation_sha256':digest(baseline/'build-attestation.json'),'scope':'Hash-verified pinned build and source manifest; actual native authentication/packet capture, no fabricated reconnect server.'})
    save(WORK/'private-resource-manifest.json',identity)
    save(OUT/'resources.json',{'cache_files':len(identity),'cache_manifest_sha256':digest(WORK/'private-resource-manifest.json'),
                              'driver_manifest':manifest,'driver_sha256':digest(driver),'scope':'Owned copies; original resources read-only. Private cache, keys, credentials and account template are not published.'})
    print('PREPARED',len(identity),'owned resource files',flush=True)
def build():
    OUT.mkdir(parents=True,exist_ok=True);resource=guard()
    cmd=[str(TOOL/'cargo.exe'),'build','--offline','--locked','-j','1','-p','gielinor-server']
    print('BUILD',cmd,flush=True)
    with (OUT/'build.log').open('w',encoding='utf-8',newline='\n') as log:
        result=subprocess.run(cmd,cwd=CHECKOUT,env=environment(),stdout=log,stderr=subprocess.STDOUT,timeout=300,creationflags=subprocess.BELOW_NORMAL_PRIORITY_CLASS)
    binary=ROOT/'work'/SHA/'target/debug/gielinor-server.exe'
    save(OUT/'build.json',{'implementation_id':FEATURE,'implementation_sha':CONTRACT_SHA,'tested_sha':SHA,'source_tree':git('rev-parse','HEAD^{tree}'),
                         'command':cmd,'exit_code':result.returncode,'binary_sha256':digest(binary) if binary.exists() else None,'resource':resource})
    print('BUILD_EXIT',result.returncode,flush=True);assert result.returncode==0
def dbpath():return CHECKOUT/'work/state'/STATE/'accounts.sqlite'
def seed():
    destination=dbpath();assert not destination.exists(),'Never overwrite an existing test database'
    destination.parent.mkdir(parents=True,exist_ok=True)
    template=SOURCE/'work/preparation/fixtures/basic-v1/state.sqlite'
    source=sqlite3.connect(template.as_uri()+'?mode=ro',uri=True)
    assert source.execute('SELECT COUNT(*) FROM accounts').fetchone()[0]==1
    assert source.execute('SELECT COUNT(*) FROM accounts WHERE login_username=?',('integration239',)).fetchone()[0]==1
    db=sqlite3.connect(destination);source.backup(db);source.close()
    with db:
        cid=db.execute('SELECT c.id FROM characters c JOIN accounts a ON a.id=c.account_id WHERE a.login_username=? AND c.realm_id=1',('integration239',)).fetchone()[0]
        inv=db.execute('SELECT id FROM inventories WHERE character_id=? AND inv_type=93',(cid,)).fetchone()[0]
        db.execute('DELETE FROM inventory_objs WHERE inventories_id IN (SELECT id FROM inventories WHERE character_id=?)',(cid,))
        if FEATURE=='OR-MIG-0056':
            mapping={0:1755,1:1755,27:995}
            for index,item in enumerate([1623,1621,1619,1617,1631,6571,19496]):mapping[4+3*index]=mapping[5+3*index]=item
            items=[(slot,mapping.get(slot,1931),1000 if slot==27 else 1) for slot in range(28)]
        elif FEATURE=='OR-MIG-0063':
            mapping={0:1755,1:1755,27:995}
            for index,item in enumerate([1623,1621,1619,1617,1631,6571,19496,*range(3345,3362,2)]):mapping[4+index]=item
            mapping.update({20:1623,21:1623,22:1623,23:3345,24:3345,25:3345})
            if SCENARIO in ['production-gem-batch','production-shell-batch','production-gem-x','production-gem-x-focus','production-gem-interrupt','production-shell-interrupt','production-gem-crash']:
                mapping={0:1755,1:1755,27:995,**{slot:(1623 if SCENARIO.startswith('production-gem') else 3345) for slot in range(4,27)}}
            items=[(slot,mapping.get(slot,1931),1000 if slot==27 else 1) for slot in range(28)]
        elif FEATURE=='OR-MIG-0062':
            items=[(slot,1785 if slot in [0,1] else 1775 if 4<=slot<=26 else 995 if slot==27 else 1931,1000 if slot==27 else 1) for slot in range(28)]
        elif FEATURE=='OR-MIG-0054':
            items=[(slot,22521 if slot in [4,5] else 995 if slot==27 else 1931,3 if slot==4 else 2 if slot==5 else 1000 if slot==27 else 1) for slot in range(28)]
        elif FEATURE=='OR-MIG-0057':
            mapping={0:1755,1:1755,27:995}
            for index,item in enumerate(range(3345,3362,2)):mapping[4+2*index]=mapping[5+2*index]=item
            items=[(slot,mapping.get(slot,1931),1000 if slot==27 else 1) for slot in range(28)]
        elif FEATURE=='OR-MIG-0051':items=[(slot,526 if slot in [3,17] else 995 if slot==27 else 1931,1000 if slot==27 else 1) for slot in range(28)]
        elif FEATURE=='OR-MIG-0010':
            items=[(slot,[2436,2440,2442,269 if slot<12 else 111][slot%4] if slot<24 else 995 if slot==27 else 1931,1000 if slot==27 else 1) for slot in range(28)]
        elif SCENARIO in ['chemistry-last','chem-interrupt','chem-crash']:items=[(slot,93 if slot%2==0 else 235,1) for slot in range(28)]
        elif SCENARIO=='food-negative':items=[(slot,{1:1891,3:1891,4:2289,5:2289,7:2142,9:6685,27:995}.get(slot,1931),1000 if slot==27 else 1) for slot in range(28)]
        elif SCENARIO in ['food','food-observer']:items=[(1,1891,1),(3,1891,1),(4,2289,1),(5,2289,1),(7,2142,1),(27,995,1000)]
        else:items=[(27,995,1000)] if SCENARIO=='chemistry-worn' else [(3,21163,1),(27,995,1000)]
        for slot,item,count in items:db.execute('INSERT INTO inventory_objs(inventories_id,slot,obj,count,vars) VALUES(?,?,?,?,0)',(inv,slot,item,count))
        if SCENARIO in ['chemistry-worn','chemistry-last','chem-negative','chem-close','chem-interrupt','chem-crash']:
            db.execute('INSERT OR IGNORE INTO inventories(character_id,inv_type) VALUES(?,94)',(cid,))
            worn=db.execute('SELECT id FROM inventories WHERE character_id=? AND inv_type=94',(cid,)).fetchone()[0]
            db.execute('INSERT INTO inventory_objs(inventories_id,slot,obj,count,vars) VALUES(?,2,21163,1,0)',(worn,))
            if SCENARIO in ['chem-interrupt','chem-crash']:db.execute('INSERT INTO inventory_objs(inventories_id,slot,obj,count,vars) VALUES(?,0,29974,1,0)',(worn,))
        db.execute('UPDATE stats SET vis_level=99,base_level=99,fine_xp=130344311 WHERE character_id=? AND stat_id=15',(cid,))
        if SCENARIO in ['food','food-observer','food-negative']:db.execute('UPDATE stats SET vis_level=?,base_level=10,fine_xp=11540 WHERE character_id=? AND stat_id=3',(10 if SCENARIO=='food-negative' else 2,cid))
        varps=json.loads(db.execute('SELECT varps FROM characters WHERE id=?',(cid,)).fetchone()[0]);varps['80']=4
        db.execute('UPDATE characters SET x=3224,z=3218,level=0,world_id=NULL,xp_rate_in_hundreds=500,varps=? WHERE id=?',(json.dumps(varps),cid))
        if FEATURE=='OR-MIG-0054':db.execute('UPDATE characters SET x=3209,z=3222,level=2 WHERE id=?',(cid,))
        db.execute('UPDATE accounts SET members=1 WHERE login_username=?',('integration239',))
        if FEATURE in ['OR-MIG-0056','OR-MIG-0057','OR-MIG-0062','OR-MIG-0063']:
            db.execute('UPDATE stats SET vis_level=99,base_level=99,fine_xp=130344311 WHERE character_id=? AND stat_id=12',(cid,))
            db.execute('UPDATE characters SET xp_rate_in_hundreds=500 WHERE id=?',(cid,))
    if FEATURE=='OR-MIG-0006':
        # Apply the recorded target's real migrations to this fresh clone only.
        for version in range(9,13):
            migration,=list((CHECKOUT/'crates/storage/migrations').glob(f'V{version}__*.sql'))
            db.executescript(migration.read_text(encoding='utf-8'))
            db.execute('INSERT INTO _rust_migrations(version) VALUES(?)',(version,));db.commit()
        db.execute('INSERT INTO chemistry_state(character_id,remaining,stop_on_crumble) VALUES(?,?,?)',(cid,1 if SCENARIO=='chemistry-last' else 2,1 if SCENARIO=='chemistry-last' else 0));db.commit()
    db.close();save(OUT/'initial-state.json',snapshot())
def snapshot():
    db=sqlite3.connect(dbpath().as_uri()+'?mode=ro',uri=True)
    cid=db.execute('SELECT id FROM characters LIMIT 1').fetchone()[0]
    state={'inventory':db.execute('SELECT i.inv_type,o.slot,o.obj,o.count,o.vars FROM inventory_objs o JOIN inventories i ON i.id=o.inventories_id WHERE i.character_id=? ORDER BY i.inv_type,o.slot',(cid,)).fetchall(),
           'stats':db.execute('SELECT stat_id,vis_level,base_level,fine_xp FROM stats WHERE character_id=? ORDER BY stat_id',(cid,)).fetchall(),
           'character':db.execute('SELECT x,z,level,run_energy,varps,last_logout,world_id FROM characters WHERE id=?',(cid,)).fetchone(),
           'migrations':[r[0] for r in db.execute('SELECT version FROM _rust_migrations ORDER BY version')],
           'integrity_check':db.execute('PRAGMA integrity_check').fetchone()[0],
           'xp_rate_hundreds':db.execute('SELECT xp_rate_in_hundreds FROM characters WHERE id=?',(cid,)).fetchone()[0]}
    if db.execute("SELECT COUNT(*) FROM sqlite_master WHERE name='chemistry_state'").fetchone()[0]:
        state['chemistry_state']=db.execute('SELECT remaining,stop_on_crumble FROM chemistry_state WHERE character_id=?',(cid,)).fetchall()
    db.close();return state
def phase(mode):
    guard();name='produce' if mode=='assemble' else 'relogin'
    fault=SCENARIO in ['chem-crash','gem-crash','glass-crash','production-gem-crash'] and mode=='assemble'
    dest=OUT/name;dest.mkdir(parents=True,exist_ok=False)
    shutil.copyfile(Path(__file__),dest/'launcher.txt')
    before=snapshot();save(dest/'before.json',before)
    binary=ROOT/'work'/SHA/'target/debug/gielinor-server.exe'
    build_manifest=json.loads((OUT/'build.json').read_text(encoding='utf-8'))
    assert build_manifest.get('tested_sha',build_manifest['implementation_sha'])==SHA and build_manifest['binary_sha256']==digest(binary),'Build identity changed'
    assert git('rev-parse','HEAD')==SHA and not git('status','--porcelain','--untracked-files=no')
    stop=WORK/(name+'.stop');assert not stop.exists()
    command=[str(binary),'--game-cache',str(RES/'cache/game'),'--js5-cache',str(RES/'cache/js5'),'--symbols',str(RES/'symbols'),
             '--rsa-exponent',str(RES/'keys/exponent.bin'),'--rsa-modulus',str(RES/'keys/modulus.bin'),'--argon2',str(RES/'argon2.dll'),
             '--state-name',STATE,'--bind','127.0.0.1:0','--ticks','700','--stop-file',str(stop)]
    client_binary=RES/('pg-live-session.exe' if SCENARIO=='food' and mode=='verify' else DRIVER)
    manifest={'implementation_id':FEATURE,'implementation_sha':CONTRACT_SHA,'tested_sha':SHA,'campaign_kind':'historical' if SHA==CONTRACT_SHA else 'integration','server_sha256':digest(binary),
              'client_sha256':digest(client_binary),'mode':mode,'started_utc':datetime.now(timezone.utc).isoformat(),
              'state_path':str(dbpath()),'database_sha256_before':digest(dbpath()),'launcher_sha256':digest(Path(__file__)),
              'server_arguments':command[1:],'status':'IN_PROGRESS'}
    save(dest/'manifest.json',manifest)
    client=None
    with (dest/'server.log').open('w',encoding='utf-8',newline='\n') as out,(dest/'server.err').open('w',encoding='utf-8',newline='\n') as err:
        server=subprocess.Popen(command,cwd=WORK,stdout=out,stderr=err,creationflags=subprocess.CREATE_NO_WINDOW|subprocess.BELOW_NORMAL_PRIORITY_CLASS)
        manifest['owned_server_pid']=server.pid
        try:
            deadline=time.monotonic()+60;address=None
            while server.poll() is None and time.monotonic()<deadline:
                text=(dest/'server.log').read_text(encoding='utf-8')
                match=re.search(r'Listening on (127\.0\.0\.1:\d+)',text)
                if match:address=match[1];break
                time.sleep(.2)
            assert address,'Exact-SHA server startup failed: '+(dest/'server.err').read_text(encoding='utf-8')[-2000:]
            print('LISTENING',address,flush=True)
            args=[str(RES/DRIVER),str(RES/'cache/js5'),address,'integration239',str(RES/'password'),str(RES/'keys/client.key'),mode]
            if SCENARIO in ['chem-negative','chem-close','chem-interrupt','chem-crash','combat-interrupt','food-negative','food-observer','prayer','gems','gem-effects','gem-crash','shells','glass','glass-quantities','glass-remaining','glass-interrupt','glass-crash','glass-all','pouch','production-gems','production-shells','production-gem-batch','production-shell-batch','production-gem-x','production-gem-x-focus','production-gem-interrupt','production-shell-interrupt','production-gem-crash']:args[-1]=(SCENARIO if SCENARIO in ['chem-negative','chem-close','chem-interrupt','chem-crash','combat-interrupt','food-negative','prayer','gems','gem-effects','gem-crash','shells','glass','glass-quantities','glass-remaining','glass-interrupt','glass-crash','glass-all','pouch','production-gems','production-shells','production-gem-batch','production-shell-batch','production-gem-x','production-gem-x-focus','production-gem-interrupt','production-shell-interrupt','production-gem-crash'] else 'food') if mode=='assemble' else 'verify'
            if SCENARIO=='food':
                if mode=='assemble':args[-1]=str(dest/'native-events.jsonl')
                else:args=[str(client_binary),str(RES/'cache/js5'),str(RES/'keys/client.key'),address,'integration239',str(RES/'password'),str(WORK/'relogin-private.capture')]
            with (dest/'client.log').open('w',encoding='utf-8',newline='\n') as log:
                client=subprocess.Popen(args,cwd=WORK,stdout=log,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW|subprocess.BELOW_NORMAL_PRIORITY_CLASS)
                manifest['owned_client_pid']=client.pid;save(dest/'manifest.json',manifest)
                try:
                    if fault:
                        deadline=time.monotonic()+390
                        manifest['intentional_crash_injected']=False
                        while client.poll() is None and time.monotonic()<deadline:
                            if not manifest['intentional_crash_injected'] and 'PG_CRASH_READY' in (dest/'client.log').read_text(encoding='utf-8'):
                                assert server.poll() is None,'Owned server ended before requested fault injection'
                                server.kill();server.wait();manifest['intentional_crash_injected']=True
                                manifest['fault_scope']=('Terminate only this freshly launched owned server after an unsaved native glass All result with captured work remaining; no normal logout/save requested' if SCENARIO=='glass-crash' else 'Terminate only this freshly launched owned server after one unsaved native selected gem result; no normal logout/save requested' if SCENARIO=='gem-crash' else 'Terminate only this freshly launched owned server after native All result with ingredients remaining; no normal logout/save requested')
                                if SCENARIO=='production-gem-crash':manifest['fault_scope']='Terminate only this freshly launched owned server during an unsaved native OR63All gem batch with captured raw items remaining; no normal logout/save requested'
                                save(dest/'manifest.json',manifest)
                            time.sleep(.02)
                        rc=client.wait(timeout=1)
                    else:rc=client.wait(timeout=390)
                except subprocess.TimeoutExpired:client.kill();client.wait();rc=124
            manifest['client_exit']=rc
            # Allow disconnect/save processing before requesting a clean owned stop.
            end=time.monotonic()+20
            while server.poll() is None and time.monotonic()<end:
                if 'Save acknowledgement:' in (dest/'server.log').read_text(encoding='utf-8'):break
                time.sleep(.2)
            manifest['status']='PASS' if rc==0 and (not fault or manifest.get('intentional_crash_injected')) else 'FAIL'
        except Exception as exc:
            manifest.update(status='BLOCKED',reason=str(exc));print('NATIVE_BLOCKED',str(exc),flush=True)
        finally:
            stop.write_text('owned graceful stop\n')
            if client is not None and client.poll() is None:client.kill();client.wait()
            forced=False
            try:server.wait(timeout=45)
            except subprocess.TimeoutExpired:forced=True;server.kill();server.wait()
            manifest.update(server_exit=server.returncode,forced_shutdown=forced)
    after=snapshot();save(dest/'after.json',after)
    manifest['database_sha256_after']=digest(dbpath())
    serverlog=(dest/'server.log').read_text(encoding='utf-8')
    acknowledgements=[json.loads(line.split('Save acknowledgement: ',1)[1]) for line in serverlog.splitlines() if line.startswith('Save acknowledgement: ')]
    manifest['save_acknowledgement']=any(a.get('save_completed') is True for a in acknowledgements)
    manifest['graceful_shutdown']='Shutdown completed; accounts drained.' in serverlog
    resources=json.loads((WORK/'private-resource-manifest.json').read_text(encoding='utf-8'))
    manifest['resource_hashes_unchanged']=all(digest(RES/path)==expected for path,expected in resources.items())
    if not manifest['resource_hashes_unchanged']:manifest.update(status='BLOCKED',reason='Owned runtime resource identity changed during the campaign')
    if fault:
        if manifest['status']=='PASS' and (manifest['server_exit']==0 or manifest['save_acknowledgement'] or manifest['graceful_shutdown']):manifest.update(status='BLOCKED',reason='Fault run did not demonstrate unacknowledged abrupt termination')
    elif manifest['status']=='PASS' and (not manifest['save_acknowledgement'] or not manifest['graceful_shutdown'] or manifest['server_exit']!=0 or manifest['forced_shutdown']):manifest.update(status='BLOCKED',reason='Client finished but durable save or graceful shutdown evidence missing')
    save(dest/'manifest.json',manifest);print('NATIVE_RESULT',json.dumps(manifest),flush=True)
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('action',choices=['prepare','build','seed','produce','relogin']);p.add_argument('--feature',choices=['combat','chemistry','chemistry-worn','chemistry-last','chem-negative','chem-close','chem-interrupt','chem-crash','combat-interrupt','food-negative','food','food-observer','prayer','gems','gem-effects','gem-crash','shells','glass','glass-quantities','glass-remaining','glass-interrupt','glass-crash','glass-all','pouch','production-gems','production-shells','production-gem-batch','production-shell-batch','production-gem-x','production-gem-x-focus','production-gem-interrupt','production-shell-interrupt','production-gem-crash'],default='combat');p.add_argument('--integration-sha');p.add_argument('--run-id');a=p.parse_args()
    SCENARIO=a.feature
    policy=ROOT/'docs/openrune/verification/USER_DEFERRALS.json'
    if policy.exists():
        deferred=json.loads(policy.read_text(encoding='utf-8'))
        assert not (deferred['status']=='ACTIVE_UNTIL_USER_RESUMES' and a.feature in deferred['native_features']),'User deferred this chemistry/Herblore/potion scenario; select eligible unrelated verification'
    if a.feature in ['chemistry','chemistry-worn','chemistry-last','chem-negative','chem-close','chem-interrupt','chem-crash']:
        SHA='e7ac3608e61893d80b2de6a1a2e512b3ab785505';FEATURE='OR-MIG-0006';STATE='pg-chemistry-native-001';DRIVER='server_chemistryui.exe';PIN='chemistryui-005'
        CHECKOUT=ROOT.parent/'agent7-targets'/SHA;WORK=ROOT/'work/native-chemistry';OUT=ROOT/'docs/openrune/verification/independent/deep-chemistry'
    if a.feature in ['food','food-observer','food-negative']:
        SHA='ae3d234c71764dd2364f8452fc1974a54a6783f0';FEATURE='OR-MIG-0024';STATE='pg-food-native-001';DRIVER='server_food.exe';PIN='food-001'
        CHECKOUT=ROOT.parent/'agent7-targets'/SHA;WORK=ROOT/'work/native-food';OUT=ROOT/'docs/openrune/verification/independent/deep-food'
    if a.feature=='prayer':
        SHA='f6b0f6ad54fe7813bb6607827eeb8da0e60137fa';FEATURE='OR-MIG-0051';CHECKOUT=ROOT.parent/'agent7-targets'/SHA
    if a.feature in ['gems','gem-effects','gem-crash','shells','glass','glass-quantities','glass-remaining','glass-interrupt','glass-crash','glass-all','pouch','production-gems','production-shells','production-gem-batch','production-shell-batch','production-gem-x','production-gem-x-focus','production-gem-interrupt','production-shell-interrupt','production-gem-crash']:
        SHA='5743cbe1c7499e2a7476c721d25dcba194666042';FEATURE='OR-MIG-0056';CHECKOUT=ROOT.parent/'agent7-targets'/SHA
    if a.feature=='shells':
        SHA='4c6b73cfcd3b35ce52aa79efd9d2a99464939445';FEATURE='OR-MIG-0057';CHECKOUT=ROOT.parent/'agent7-targets'/SHA
    if a.feature in ['glass','glass-quantities','glass-remaining','glass-interrupt','glass-crash','glass-all','pouch','production-gems','production-shells','production-gem-batch','production-shell-batch','production-gem-x','production-gem-x-focus','production-gem-interrupt','production-shell-interrupt','production-gem-crash']:
        SHA='5ce50aede9164cdee8e4008257d0298ea77289e7';FEATURE='OR-MIG-0062';CHECKOUT=ROOT.parent/'agent7-targets'/SHA
    if a.feature=='pouch':
        SHA='7fc1306bb5c01540038ce5ff175b577294ade432';FEATURE='OR-MIG-0054';CHECKOUT=ROOT.parent/'agent7-targets'/SHA
    if a.feature in ['production-gems','production-shells','production-gem-batch','production-shell-batch','production-gem-x','production-gem-x-focus','production-gem-interrupt','production-shell-interrupt','production-gem-crash']:
        SHA='143dab04f28c3daed844da0af33d6af5edcd962d';FEATURE='OR-MIG-0063';CHECKOUT=ROOT.parent/'agent7-targets'/SHA
    CONTRACT_SHA=SHA
    if a.feature=='chemistry-worn':
        STATE='pg-chemistry-worn-native-001';DRIVER='server_chemistryworn.exe';PIN='chemistryworn-004'
        WORK=ROOT/'work/native-chemistry-worn';OUT=ROOT/'docs/openrune/verification/independent/deep-chemistry-worn'
    if a.feature=='chemistry-last':
        STATE='pg-chemistry-last-native-001';DRIVER='server_chemistry.exe';PIN='chemistry-001'
        WORK=ROOT/'work/native-chemistry-last';OUT=ROOT/'docs/openrune/verification/independent/deep-chemistry-last'
    if a.feature in ['chem-negative','chem-close','chem-interrupt','chem-crash','combat-interrupt','food-negative','food-observer','prayer','gems','gem-effects','gem-crash','shells','glass','glass-quantities','glass-remaining','glass-interrupt','glass-crash','glass-all','pouch','production-gems','production-shells','production-gem-batch','production-shell-batch','production-gem-x','production-gem-x-focus','production-gem-interrupt','production-shell-interrupt','production-gem-crash']:
        observer_build=json.loads((ROOT/'docs/openrune/verification/independent/client-observer-build/build.json').read_text(encoding='utf-8'));assert observer_build['status']=='PASS'
        STATE='pg-'+a.feature+'-001';DRIVER='pg_observer-'+observer_build['binaries']['pg_observer.exe'][:12]+'.exe';PIN='pg-observer'
        WORK=ROOT/'work'/('native-'+a.feature);OUT=ROOT/'docs/openrune/verification/independent'/('deep-'+a.feature)
    if a.integration_sha:
        assert re.fullmatch('[0-9a-f]{40}',a.integration_sha)
        assert (ROOT/'docs/openrune/verification/independent'/('queue-'+a.integration_sha+'.jsonl')).exists(),'Integration checkpoint must have a fetched queue snapshot'
        SHA=a.integration_sha;CHECKOUT=ROOT.parent/'agent7-targets'/SHA
        if not CHECKOUT.exists():subprocess.run(['git','-c','safe.directory='+ROOT.as_posix(),'worktree','add','--detach',str(CHECKOUT),SHA],cwd=ROOT,check=True)
        STATE='pg-'+SCENARIO+'-'+SHA[:12];WORK=ROOT/'work'/('native-'+SCENARIO+'-'+SHA[:12]);OUT=ROOT/'docs/openrune/verification/independent'/('native-'+SCENARIO+'-'+SHA)
    if a.run_id:
        assert re.fullmatch('[a-z0-9-]{1,24}',a.run_id)
        STATE='pg-'+SCENARIO+'-'+SHA[:8]+'-'+a.run_id
        WORK=WORK/a.run_id;OUT=OUT/a.run_id
    if PIN=='pg-observer' and a.action in ['produce','relogin']:
        frozen=json.loads((OUT/'resources.json').read_text(encoding='utf-8'))['driver_manifest']['files']
        DRIVER,=frozen;assert digest(RES/DRIVER)==frozen[DRIVER]
    if a.action=='prepare':prepare()
    elif a.action=='build':build()
    elif a.action=='seed':seed()
    else:phase('assemble' if a.action=='produce' else 'verify')
