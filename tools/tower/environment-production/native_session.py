"""Owned native test session through existing ordinary menu/walking actions."""
from pathlib import Path
import subprocess,os,json,time,secrets,socket,hashlib,datetime,sys
T=Path(__file__).resolve().parent;S=T.parents[2];E=S.parent;W=E.parents[1];R=E/'cinderworks-environment-private'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def start(mode='gpu',port=63241):
    with socket.socket()as test:
        test.settimeout(.25)
        if test.connect_ex(('127.0.0.1',port))==0:raise RuntimeError('Refusing occupied loopback port')
    stamp=datetime.datetime.now().strftime('%Y%m%d-%H%M%S');session=R/('native-'+stamp+'-'+mode);session.mkdir()
    private=R/'private';private.mkdir(exist_ok=True);password=private/(stamp+'.password');password.write_text(secrets.token_urlsafe(24),encoding='ascii')
    username='cinder'+datetime.datetime.now().strftime('%H%M%S');state_name='cinderworks-env-'+stamp;state=S/'work/state'/state_name;state.mkdir(parents=True)
    server=E/'build/cinderworks-server/debug/gielinor-server.exe';client=E/'build/enhanced/release/gielinor-client.exe';argon=W/'rust-server/work/native/argon2.dll'
    result=subprocess.run([str(E/'build/cinderworks-server/debug/examples/cinderworks_environment_account.exe'),str(state/'accounts.sqlite'),str(argon),username,str(password)],capture_output=True,text=True);(session/'account-provision.log').write_text(result.stdout+result.stderr);result.check_returncode()
    env=os.environ.copy();env.update(TOWER_ASSET_PACKAGE=str(R/'package-v1'),TOWER_CINDERWORKS_ENVIRONMENT=str(R/'placements/07-whole-floor-refinement.tsv'))
    data=W/'resource-staging/server/work/integration/a-kotlin/.data';keys=W/'work/disposable-startup-keys'
    server_args=[str(server),'--game-cache',str(data/'cache/game'),'--js5-cache',str(data/'cache/js5'),'--symbols',str(data/'symbols'),'--rsa-exponent',str(keys/'private-exponent.bin'),'--rsa-modulus',str(keys/'modulus.bin'),'--argon2',str(argon),'--state-name',state_name,'--bind',f'127.0.0.1:{port}','--ticks','100000','--stop-file',str(session/'server.stop')]
    # Hide the background service console. The native game window is visible.
    sf=(session/'server.log').open('w');process=subprocess.Popen(server_args,cwd=S,env=env,stdout=sf,stderr=subprocess.STDOUT,creationflags=getattr(subprocess,'CREATE_NO_WINDOW',0))
    (session/'owned-server.json').write_text(json.dumps({'pid':process.pid,'binary':str(server),'sha256':sha(server),'stop_file':str(session/'server.stop'),'args':server_args},indent=2))
    for _ in range(60):
        if process.poll()is not None:raise RuntimeError('Owned server exited: '+(session/'server.log').read_text()[-1500:])
        with socket.socket()as test:
            test.settimeout(.25)
            if test.connect_ex(('127.0.0.1',port))==0:break
        time.sleep(.5)
    else:
        (session/'server.stop').write_text('stop')
        raise RuntimeError('Owned server logged startup but loopback listener was unreachable from this execution context; graceful stop requested.')
    env.update(TOWER_ACTION_DIRECTORY=str(session),TOWER_RENDERER=mode,TOWER_GPU_ADAPTER='0'if mode=='lite'else'1',TOWER_RENDER_METRICS=str(session/'renderer-metrics.json'),TOWER_WINDOW_SIZE='1280x900')
    (session/'command.txt').write_text('1 observe',encoding='ascii');cf=(session/'client.log').open('w')
    client_args=[str(client),'live-view',str(E/'live-client-cache-v1'),f'127.0.0.1:{port}',username,str(password),str(keys/'public-key.txt')]
    frontend=subprocess.Popen(client_args,cwd=session,env=env,stdout=cf,stderr=subprocess.STDOUT)
    record={'session':str(session),'server_pid':process.pid,'client_pid':frontend.pid,'server_binary':str(server),'client_binary':str(client),'server_sha256':sha(server),'client_sha256':sha(client),'server_args':server_args,'client_args':client_args,'username':username,'state':str(state),'mode':mode,'new_account_initial_position':[3219,3218,0],'client_env':{k:env[k]for k in ['TOWER_ACTION_DIRECTORY','TOWER_RENDERER','TOWER_GPU_ADAPTER','TOWER_RENDER_METRICS','TOWER_WINDOW_SIZE','TOWER_ASSET_PACKAGE']},'scope':'New disposable account initially placed on the established recovery pad, without earned state/items/skills; actual first-arrival policy and subsequent ordinary client actions. No existing account or gameplay progression was edited.'}
    (session/'owned-session.json').write_text(json.dumps(record,indent=2));(R/'latest-native-session.json').write_text(json.dumps(record,indent=2));print('NATIVE_SESSION',session,'serverPID',process.pid,'clientPID',frontend.pid,mode,flush=True)
    return session
def state(session):
    try:return json.loads((session/'state.json').read_text())
    except(FileNotFoundError,json.JSONDecodeError,PermissionError):return None
def command(session,verb,timeout=45):
    old=state(session);seq=max(int(old.get('sequence',0))if old else 0,int((session/'command.txt').read_text().split()[0]))+1
    temp=session/'command.next';temp.write_text(str(seq)+' '+verb,encoding='ascii')
    for attempt in range(100):
        try:temp.replace(session/'command.txt');break
        except PermissionError:
            if attempt==99:raise
            time.sleep(.02)
    before=time.monotonic()
    while time.monotonic()-before<timeout:
        value=state(session)
        if value and int(value['sequence'])>=seq:return value
        time.sleep(.08)
    raise TimeoutError(('Native command receipt',seq,verb))
if __name__=='__main__':
    if len(sys.argv)>1 and sys.argv[1]=='start':start(sys.argv[2]if len(sys.argv)>2 else'gpu')
