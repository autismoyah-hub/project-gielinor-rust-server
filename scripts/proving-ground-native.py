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
    a=cpu_sample();time.sleep(.5);b=cpu_sample();cpu=100*(1-(b[0]-a[0])/max(1,b[1]-a[1]))
    report={'available_ram':m.avail,'active_compilers':active,'cpu_percent':round(cpu,2),'disk_free':shutil.disk_usage(ROOT).free}
    print('RESOURCE',json.dumps(report),flush=True)
    # Other agents' activity is measured, not an automatic blanket deferral.
    # One owned build job is permitted only with spare CPU and >=6 GiB RAM.
    permitted=cpu<30 and m.avail>=6*1024**3
    OUT.mkdir(parents=True,exist_ok=True)
    with (OUT/'resource-observations.jsonl').open('a',encoding='utf-8',newline='\n') as f:
        f.write(json.dumps({'observed_utc':datetime.now(timezone.utc).isoformat(),'decision':'PROCEED' if permitted else 'DEFER','measurement':report})+'\n')
    assert permitted,'Measured resource guard blocked; no other process altered'
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
    pin=SOURCE/'integration/scenario-drivers'/PIN
    manifest=json.loads((pin/'manifest.json').read_text())
    for name,expected in manifest['files'].items():assert digest(pin/name)==expected.lower()
    driver=RES/DRIVER
    if not driver.exists():shutil.copyfile(pin/driver.name,driver)
    assert digest(driver)==manifest['files'][driver.name].lower()
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
        if FEATURE=='OR-MIG-0010':
            items=[(slot,[2436,2440,2442,269 if slot<12 else 111][slot%4] if slot<24 else 995 if slot==27 else 1931,1000 if slot==27 else 1) for slot in range(28)]
        else:items=[(27,995,1000)] if SCENARIO=='chemistry-worn' else [(3,21163,1),(27,995,1000)]
        for slot,item,count in items:db.execute('INSERT INTO inventory_objs(inventories_id,slot,obj,count,vars) VALUES(?,?,?,?,0)',(inv,slot,item,count))
        if SCENARIO=='chemistry-worn':
            db.execute('INSERT OR IGNORE INTO inventories(character_id,inv_type) VALUES(?,94)',(cid,))
            worn=db.execute('SELECT id FROM inventories WHERE character_id=? AND inv_type=94',(cid,)).fetchone()[0]
            db.execute('INSERT INTO inventory_objs(inventories_id,slot,obj,count,vars) VALUES(?,2,21163,1,0)',(worn,))
        db.execute('UPDATE stats SET vis_level=99,base_level=99,fine_xp=130344311 WHERE character_id=? AND stat_id=15',(cid,))
        varps=json.loads(db.execute('SELECT varps FROM characters WHERE id=?',(cid,)).fetchone()[0]);varps['80']=4
        db.execute('UPDATE characters SET x=3224,z=3218,level=0,world_id=NULL,xp_rate_in_hundreds=500,varps=? WHERE id=?',(json.dumps(varps),cid))
        db.execute('UPDATE accounts SET members=1 WHERE login_username=?',('integration239',))
    if FEATURE=='OR-MIG-0006':
        # Apply the recorded target's real migrations to this fresh clone only.
        for version in range(9,13):
            migration,=list((CHECKOUT/'crates/storage/migrations').glob(f'V{version}__*.sql'))
            db.executescript(migration.read_text(encoding='utf-8'))
            db.execute('INSERT INTO _rust_migrations(version) VALUES(?)',(version,));db.commit()
        db.execute('INSERT INTO chemistry_state(character_id,remaining,stop_on_crumble) VALUES(?,2,0)',(cid,));db.commit()
    db.close();save(OUT/'initial-state.json',snapshot())
def snapshot():
    db=sqlite3.connect(dbpath().as_uri()+'?mode=ro',uri=True)
    cid=db.execute('SELECT id FROM characters LIMIT 1').fetchone()[0]
    state={'inventory':db.execute('SELECT i.inv_type,o.slot,o.obj,o.count,o.vars FROM inventory_objs o JOIN inventories i ON i.id=o.inventories_id WHERE i.character_id=? ORDER BY i.inv_type,o.slot',(cid,)).fetchall(),
           'stats':db.execute('SELECT stat_id,vis_level,base_level,fine_xp FROM stats WHERE character_id=? ORDER BY stat_id',(cid,)).fetchall(),
           'character':db.execute('SELECT x,z,level,run_energy,varps,last_logout,world_id FROM characters WHERE id=?',(cid,)).fetchone(),
           'migrations':[r[0] for r in db.execute('SELECT version FROM _rust_migrations ORDER BY version')],
           'integrity_check':db.execute('PRAGMA integrity_check').fetchone()[0]}
    if db.execute("SELECT COUNT(*) FROM sqlite_master WHERE name='chemistry_state'").fetchone()[0]:
        state['chemistry_state']=db.execute('SELECT remaining,stop_on_crumble FROM chemistry_state WHERE character_id=?',(cid,)).fetchall()
    db.close();return state
def phase(mode):
    guard();name='produce' if mode=='assemble' else 'relogin'
    dest=OUT/name;dest.mkdir(parents=True,exist_ok=False)
    before=snapshot();save(dest/'before.json',before)
    binary=ROOT/'work'/SHA/'target/debug/gielinor-server.exe'
    build_manifest=json.loads((OUT/'build.json').read_text(encoding='utf-8'))
    assert build_manifest.get('tested_sha',build_manifest['implementation_sha'])==SHA and build_manifest['binary_sha256']==digest(binary),'Build identity changed'
    assert git('rev-parse','HEAD')==SHA and not git('status','--porcelain','--untracked-files=no')
    stop=WORK/(name+'.stop');assert not stop.exists()
    command=[str(binary),'--game-cache',str(RES/'cache/game'),'--js5-cache',str(RES/'cache/js5'),'--symbols',str(RES/'symbols'),
             '--rsa-exponent',str(RES/'keys/exponent.bin'),'--rsa-modulus',str(RES/'keys/modulus.bin'),'--argon2',str(RES/'argon2.dll'),
             '--state-name',STATE,'--bind','127.0.0.1:0','--ticks','700','--stop-file',str(stop)]
    manifest={'implementation_id':FEATURE,'implementation_sha':CONTRACT_SHA,'tested_sha':SHA,'campaign_kind':'historical' if SHA==CONTRACT_SHA else 'integration','server_sha256':digest(binary),
              'client_sha256':digest(RES/DRIVER),'mode':mode,'started_utc':datetime.now(timezone.utc).isoformat(),
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
            with (dest/'client.log').open('w',encoding='utf-8',newline='\n') as log:
                client=subprocess.Popen(args,cwd=WORK,stdout=log,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW|subprocess.BELOW_NORMAL_PRIORITY_CLASS)
                manifest['owned_client_pid']=client.pid;save(dest/'manifest.json',manifest)
                try:rc=client.wait(timeout=390)
                except subprocess.TimeoutExpired:client.kill();client.wait();rc=124
            manifest['client_exit']=rc
            # Allow disconnect/save processing before requesting a clean owned stop.
            end=time.monotonic()+20
            while server.poll() is None and time.monotonic()<end:
                if 'Save acknowledgement:' in (dest/'server.log').read_text(encoding='utf-8'):break
                time.sleep(.2)
            manifest['status']='PASS' if rc==0 else 'FAIL'
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
    manifest['save_acknowledgement']='Save acknowledgement:' in serverlog
    manifest['graceful_shutdown']='Shutdown completed; accounts drained.' in serverlog
    resources=json.loads((WORK/'private-resource-manifest.json').read_text(encoding='utf-8'))
    manifest['resource_hashes_unchanged']=all(digest(RES/path)==expected for path,expected in resources.items())
    if not manifest['resource_hashes_unchanged']:manifest.update(status='BLOCKED',reason='Owned runtime resource identity changed during the campaign')
    if manifest['status']=='PASS' and (not manifest['save_acknowledgement'] or not manifest['graceful_shutdown'] or manifest['server_exit']!=0 or manifest['forced_shutdown']):manifest.update(status='BLOCKED',reason='Client finished but durable save or graceful shutdown evidence missing')
    save(dest/'manifest.json',manifest);print('NATIVE_RESULT',json.dumps(manifest),flush=True)
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('action',choices=['prepare','build','seed','produce','relogin']);p.add_argument('--feature',choices=['combat','chemistry','chemistry-worn'],default='combat');p.add_argument('--integration-sha');p.add_argument('--run-id');a=p.parse_args()
    SCENARIO=a.feature
    if a.feature in ['chemistry','chemistry-worn']:
        SHA='e7ac3608e61893d80b2de6a1a2e512b3ab785505';FEATURE='OR-MIG-0006';STATE='pg-chemistry-native-001';DRIVER='server_chemistryui.exe';PIN='chemistryui-005'
        CHECKOUT=ROOT.parent/'agent7-targets'/SHA;WORK=ROOT/'work/native-chemistry';OUT=ROOT/'docs/openrune/verification/independent/deep-chemistry'
    CONTRACT_SHA=SHA
    if a.feature=='chemistry-worn':
        STATE='pg-chemistry-worn-native-001';DRIVER='server_chemistryworn.exe';PIN='chemistryworn-004'
        WORK=ROOT/'work/native-chemistry-worn';OUT=ROOT/'docs/openrune/verification/independent/deep-chemistry-worn'
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
    if a.action=='prepare':prepare()
    elif a.action=='build':build()
    elif a.action=='seed':seed()
    else:phase('assemble' if a.action=='produce' else 'verify')
