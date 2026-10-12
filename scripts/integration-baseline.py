"""Read-only, whole-tree before/after snapshot; never replaces an existing pin."""
import argparse, hashlib, json, os, shutil, stat, time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ACTIVE = ROOT.parent / 'rust-client'
PARTS = ['src', 'examples', 'tests', 'vendor', 'licenses', 'tools', 'Cargo.toml', 'Cargo.lock', 'build.ps1', 'README.md', 'PROGRESS.md', 'PARITY_GAPS.md', 'PORTING.md', 'LICENSE-RSPROX', '.gitignore']
def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda:f.read(1024*1024), b''): h.update(block)
    return h.hexdigest()
def files(root):
    result=[]
    for part in PARTS:
        p=root/part
        if p.is_file(): result.append(p)
        elif p.is_dir(): result.extend(x for x in p.rglob('*') if x.is_file() and not any(t in ('target','.git','__pycache__') for t in x.relative_to(p).parts))
    return sorted(result)
def hashes(root): return {p.relative_to(root).as_posix():sha(p) for p in files(root)}
def verify(path):
    manifest=json.loads((path/'manifest.json').read_text())
    for relative, expected in manifest['files'].items():
        if sha(path/relative)!=expected: raise ValueError(f'Baseline modified: {relative}')
    actual={p.relative_to(path/'source').as_posix() for p in files(path/'source')}
    expected={p[7:] for p in manifest['files'] if p.startswith('source/')}
    if actual!=expected: raise ValueError('Baseline source file set changed')
    return manifest
def main():
    parser=argparse.ArgumentParser();parser.add_argument('mode',choices=['snapshot','verify']);parser.add_argument('baseline');args=parser.parse_args()
    if '..' in args.baseline or not args.baseline.replace('-','').replace('.','').isalnum(): raise ValueError('Invalid baseline name')
    path=ROOT/'integration/client-baselines'/args.baseline
    if args.mode=='verify':
        m=verify(path);print(json.dumps({'baseline':args.baseline,'integrity':'PASS','files':len(m['files']),'validation':m['validation']}));return
    if path.exists(): raise ValueError('Baseline already exists; explicit new identifier required')
    path.mkdir(parents=True)
    m={'schema':1,'baseline':args.baseline,'revision':'239.1','validation':'UNVERIFIED','source':str(ACTIVE),'created_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),'binary_source_correspondence':'Not attested; copied release binaries may predate captured source. Build an experimental copy to establish correspondence.','files':{}}
    for attempt in range(3):
        before=hashes(ACTIVE)
        destination=path/'source';destination.mkdir(exist_ok=True)
        for relative in before:
            out=destination/relative;out.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ACTIVE/relative,out)
        copied=hashes(destination);after=hashes(ACTIVE)
        if before==after==copied:
            m['consistent_source_attempt']=attempt+1;break
        # Keep evidence of unsuccessful attempts; do not delete or overwrite a pin.
        destination.rename(path/f'changing-source-attempt-{attempt+1}')
    else:
        (path/'snapshot-error.json').write_text(json.dumps({'error':'Source changed during all three bounded snapshot attempts'}));raise ValueError('No coherent snapshot; retained attempts')
    for rel in ['gielinor-client.exe','examples/live_session.exe','examples/live_actions.exe','examples/reconnect_session.exe']:
        src=ACTIVE/'target/release'/rel
        if not src.exists(): continue
        before=sha(src);out=path/'bin'/Path(rel).name;out.parent.mkdir(exist_ok=True);shutil.copyfile(src,out)
        if before!=sha(src) or before!=sha(out): raise ValueError(f'Executable changed during copy: {rel}')
    for name in ['native-music.log','music-native-build.log','full-tests-music.log','music-activity-tests.log','login-transfer-tests.log','login-transfer-replay.log']:
        src=ACTIVE/'work'/name
        if src.is_file():
            before=sha(src);out=path/'evidence'/name;out.parent.mkdir(exist_ok=True);shutil.copyfile(src,out)
            if before!=sha(src) or before!=sha(out): raise ValueError(f'Evidence changed while copying: {name}')
    for p in sorted(path.rglob('*')):
        if p.is_file(): m['files'][p.relative_to(path).as_posix()]=sha(p)
    (path/'manifest.json').write_text(json.dumps(m,indent=2)+'\n')
    for p in path.rglob('*'):
        if p.is_file(): p.chmod(stat.S_IREAD)
    print(json.dumps({'baseline':args.baseline,'files':len(m['files']),'validation':'UNVERIFIED','source_consistency':'PASS'}))
if __name__=='__main__':main()
