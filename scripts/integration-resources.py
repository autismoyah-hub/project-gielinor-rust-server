"""Hash only owned integration caches, symbols and isolated Kotlin runtime inputs."""
import argparse,hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
OUT=ROOT/'integration/resource-manifest.json'
def sha(p):
    h=hashlib.sha256()
    with p.open('rb') as f:
        for block in iter(lambda:f.read(1024*1024),b''):h.update(block)
    return h.hexdigest()
def cache(environment):
    base=ROOT/'work/integration'/('a-kotlin/.data' if environment=='kotlin' else 'b-rust')
    files={p.relative_to(base).as_posix():sha(p) for directory in ('cache/game','cache/js5','symbols') for p in sorted((base/directory).rglob('*')) if p.is_file()}
    if not files:raise ValueError('Missing cache inputs')
    return {'files':files,'cache_id':hashlib.sha256(json.dumps(files,sort_keys=True).encode()).hexdigest()}
def main():
    p=argparse.ArgumentParser();p.add_argument('mode',choices=['record','verify']);p.add_argument('--environment',choices=['kotlin','rust']);a=p.parse_args()
    if a.mode=='record':
        data={e:cache(e) for e in ('kotlin','rust')}
        cp=ROOT/'work/integration/kotlin-runtime-classpath.txt'
        paths=[Path(s) for s in cp.read_text(encoding='utf-8-sig').strip().split(';')]
        data['kotlin_runtime']={'classpath_sha256':sha(cp),'files':{str(path):sha(path) for path in paths if path.is_file()},'unhashed_directories':[str(path) for path in paths if path.is_dir()]}
        data['kotlin_overlay']={str(p.relative_to(ROOT)):sha(p) for p in sorted((ROOT/'work/integration/kotlin-overlay/classes').rglob('*.class'))}
        with OUT.open('x',encoding='utf8') as f:json.dump(data,f,indent=2)
        print(json.dumps({'matching_cache':data['kotlin']['cache_id']==data['rust']['cache_id'],'cache_id':data['rust']['cache_id'],'runtime_files':len(data['kotlin_runtime']['files'])}));return
    if not a.environment:raise ValueError('Environment required')
    manifest=json.loads(OUT.read_text());expected=manifest[a.environment];actual=cache(a.environment)
    if actual!=expected:raise ValueError('Owned cache or symbols changed; explicit new resource pin required')
    if a.environment=='kotlin':
        runtime=manifest['kotlin_runtime']
        if runtime['unhashed_directories']:raise ValueError('Unpinned runtime directories')
        if sha(ROOT/'work/integration/kotlin-runtime-classpath.txt')!=runtime['classpath_sha256']:raise ValueError('Kotlin classpath changed')
        for path,digest in runtime['files'].items():
            if sha(Path(path))!=digest:raise ValueError('Kotlin runtime input changed')
        for path,digest in manifest['kotlin_overlay'].items():
            if sha(ROOT/path)!=digest:raise ValueError('Kotlin overlay changed')
    print(actual['cache_id'])
if __name__=='__main__':main()
