"""Run omitted queue cases from an already attested exact-SHA test executable.

No compiler is launched. This is a small one-thread test replay, suitable while
the heavy-build resource guard has deferred Cargo due to another agent's build.
"""
import hashlib,json,re,subprocess,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];SHA='caefcf424869d2a54d711acd22a279351f098be0'
OUT=ROOT/'docs/openrune/verification/independent'/('OR-MIG-0023-'+SHA[:12]+'-cached-'+str(time.time_ns()));OUT.mkdir(parents=True)
prior=json.loads((OUT.parent/('OR-MIG-0023-'+SHA[:12])/'manifest.json').read_text(encoding='utf-8'))
entry=next((path,value) for path,value in prior['test_binary_sha256'].items() if Path(path).name.startswith('gielinor_world-'))
binary=ROOT/'work'/SHA/entry[0];assert hashlib.sha256(binary.read_bytes()).hexdigest()==entry[1]
checkout=ROOT.parent/'agent7-targets'/SHA
args=['git','-c','safe.directory='+checkout.as_posix()]
assert subprocess.check_output([*args,'rev-parse','HEAD'],cwd=checkout,text=True).strip()==SHA
assert not subprocess.check_output([*args,'status','--porcelain','--untracked-files=no'],cwd=checkout,text=True).strip()
cmd=[str(binary),'item_pack','--test-threads=1'];log=OUT/'queue-replay.log'
with log.open('w',encoding='utf-8',newline='\n') as f:
    result=subprocess.run(cmd,cwd=checkout,stdout=f,stderr=subprocess.STDOUT,timeout=60,creationflags=subprocess.BELOW_NORMAL_PRIORITY_CLASS)
text=log.read_text(encoding='utf-8');count=sum(map(int,re.findall(r'test result: ok\. (\d+) passed;',text)))
report={'implementation_id':'OR-MIG-0023','implementation_commit_sha':SHA,'source_tree':prior['source_tree'],
        'outcome':'PARTIAL' if result.returncode==0 and count else 'FAILED','independent_verification':True,
        'scope':'Expanded authoritative item-pack repeat, movement cancellation and stale-selected-slot tests; attested binary replay without Cargo or compilation.',
        'test_binary_sha256':{entry[0]:entry[1]},'runs':[{'command':cmd,'exit_code':result.returncode,'tests_passed':count,'zero_tests':count==0,'log':str(log.relative_to(ROOT))}]}
(OUT/'manifest.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n')
print(text);print(json.dumps(report))
