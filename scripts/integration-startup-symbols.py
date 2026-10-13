"""Check literal startup lookups against the actual owned symbol bundle.

This deliberately reports partial coverage: dynamically constructed names and
definition payloads still require a real startup/integration test.
"""
import argparse,hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent

def audit():
    source=ROOT/'crates/server/src/startup.rs'
    symbols=ROOT/'work/integration/a-kotlin/.data/symbols'
    lookups=re.findall(r'names\.(?:id|short)\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*\)',source.read_text())
    missing=[];resolved=[];groups={}
    for kind,name in sorted(set(lookups)):
        if kind not in groups:
            groups[kind]={line.split('\t',1)[1] for line in (symbols/(kind+'.sym')).read_text().splitlines() if '\t' in line and not line.startswith('#')}
        (resolved if name in groups[kind] else missing).append({'kind':kind,'name':name})
    return {'status':'FAIL' if missing else 'PASS','source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'lookups':len(resolved)+len(missing),'missing':missing,'coverage':'literal Names.id/short lookups in startup.rs only; dynamic names and payloads UNKNOWN'}

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('output',type=Path);a=p.parse_args();result=audit()
    with a.output.open('x',encoding='utf8') as f:json.dump(result,f,indent=2)
    print(json.dumps(result))
    raise SystemExit(result['status']!='PASS')
