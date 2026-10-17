#!/usr/bin/env python3
"""Validate versioned OpenRune contracts; this does not certify gameplay behavior."""
import argparse,json,re,subprocess
from pathlib import Path,PurePosixPath

REQUIRED={'schema_version','implementation_id','revision','feature_name','source_references','provenance','rust_paths','registration_dispatch','implementation_commit_sha','dependencies','expected_normal_behavior','expected_negative_behavior','identifiers','timing_and_transactions','required_verification_levels','known_limitations','implementation_status','verification_status','behavior_basis','legacy_evidence','supersedes'}
def require(ok,message):
    if not ok:raise ValueError(message)
def strings(value,nonempty=True):return isinstance(value,list) and (bool(value) or not nonempty) and all(isinstance(v,str) and bool(v.strip()) for v in value)
def relative(path):return isinstance(path,str) and bool(path) and '\\' not in path and not PurePosixPath(path).is_absolute() and ':' not in path and '..' not in PurePosixPath(path).parts
def validate(records,root=None):
    latest={};ids={r.get('implementation_id') for r in records}
    for index,r in enumerate(records,1):
        prefix=f'line{index}'
        require(isinstance(r,dict) and set(r)==REQUIRED,prefix+': missing/unknown fields')
        require(r['schema_version']==1 and type(r['schema_version']) is int,prefix+': schema_version')
        ident=r['implementation_id'];require(isinstance(ident,str) and re.fullmatch(r'OR-MIG-\d{4}',ident),prefix+': stable ID')
        require(type(r['revision']) is int and r['revision']>0,prefix+': revision')
        prev=latest.get(ident)
        if prev:
            require(r['revision']==prev['revision']+1 and r['supersedes']=={'revision':prev['revision'],'implementation_commit_sha':prev['implementation_commit_sha']},prefix+': revision must link previous exact implementation')
        else:require(r['revision']==1 and r['supersedes'] is None,prefix+': first revision')
        sha=r['implementation_commit_sha'];require(isinstance(sha,str) and re.fullmatch('[0-9a-f]{40}',sha),prefix+': full implementation SHA required')
        require(isinstance(r['feature_name'],str) and r['feature_name'].strip(),prefix+': feature')
        require(strings(r['rust_paths']) and all(relative(p) for p in r['rust_paths']),prefix+': Rust paths')
        require(isinstance(r['source_references'],list) and bool(r['source_references']),prefix+': source references')
        for source in r['source_references']:
            require(set(source)=={'kind','reference','sha256','availability','revision'},prefix+': source fields')
            require(source['kind'] in ('openrune','wiki','project_gielinor') and isinstance(source['reference'],str) and source['reference'],prefix+': source identity')
            require(isinstance(source['sha256'],str) and re.fullmatch('[0-9a-f]{64}',source['sha256']),prefix+': source SHA256')
            require(source['availability'] in ('protected_local_reference','public_revision','repository'),prefix+': source availability')
            require(source['revision'] is None or isinstance(source['revision'],(str,int)),prefix+': source revision')
        provenance=r['provenance'];require(set(provenance)=={'status','basis','restrictions'},prefix+': provenance fields')
        require(provenance['status'] in ('independent_behavior_adaptation','cleared_adaptation') and isinstance(provenance['basis'],str) and provenance['basis'].strip() and strings(provenance['restrictions']),prefix+': provenance assessment')
        require(isinstance(r['registration_dispatch'],list) and bool(r['registration_dispatch']),prefix+': dispatch chain')
        for step in r['registration_dispatch']:
            require(set(step)=={'path','symbol','role'} and relative(step['path']) and all(isinstance(step[k],str) and step[k].strip() for k in ('symbol','role')),prefix+': dispatch step')
        for name in ('dependencies','known_limitations'):require(strings(r[name],name=='known_limitations'),prefix+': '+name)
        for dep in r['dependencies']:require(dep in ids or dep.startswith('ENV:'),prefix+': unresolved dependency '+dep)
        for name in ('expected_normal_behavior','expected_negative_behavior'):require(strings(r[name]),prefix+': '+name)
        require(isinstance(r['identifiers'],dict) and bool(r['identifiers']) and all(strings(v) or (isinstance(v,list) and bool(v) and all(type(i) is int for i in v)) for v in r['identifiers'].values()),prefix+': identifier lists')
        require(set(r['timing_and_transactions'])=={'timing','transactions'} and all(strings(v) for v in r['timing_and_transactions'].values()),prefix+': timing/transactions')
        require(set(r['behavior_basis'])=={'observed_in_source','wiki_supported','intentional_adaptations','inferred_or_unresolved'} and all(strings(v,False) for v in r['behavior_basis'].values()),prefix+': evidence basis')
        levels=r['required_verification_levels'];require(isinstance(levels,list) and len(levels)==5 and {v.get('level') for v in levels}=={'V0','V1','V2','V3','V4'},prefix+': enumerate all verification levels')
        for level in levels:require(set(level)=={'level','required','reason'} and type(level['required']) is bool and isinstance(level['reason'],str) and level['reason'].strip(),prefix+': level rationale')
        require(next(v for v in levels if v['level']=='V0')['required'],prefix+': V0 required')
        require(r['implementation_status'] in ('implemented_pending_verification','correction_pending_retest'),prefix+': implementation status')
        require(r['verification_status']=='queued_independent',prefix+': FORGE cannot assert independent acceptance')
        require(isinstance(r['legacy_evidence'],list),prefix+': legacy evidence')
        for evidence in r['legacy_evidence']:
            require(set(evidence)=={'reference','scope','independent_verification'} and relative(evidence['reference']) and isinstance(evidence['scope'],str) and evidence['scope'].strip() and evidence['independent_verification'] is False,prefix+': legacy evidence must not impersonate independent verification')
        if root:
            def git(*args):return subprocess.check_output(['git','-c','safe.directory='+str(root),*args],cwd=root,text=True,encoding='utf-8')
            git('cat-file','-e',sha+'^{commit}')
            for path in r['rust_paths']:git('cat-file','-e',sha+':'+path)
            for step in r['registration_dispatch']:require(step['symbol'] in git('show',sha+':'+step['path']),prefix+': dispatch symbol absent at implementation SHA: '+step['symbol'])
        latest[ident]=r
    visiting=set();done=set()
    def visit(ident):
        require(ident not in visiting,'cyclic implementation dependency: '+ident)
        if ident in done:return
        visiting.add(ident)
        for dep in latest[ident]['dependencies']:
            if dep in latest:visit(dep)
        visiting.remove(ident);done.add(ident)
    for ident in latest:visit(ident)
    return len(latest)
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('queue',type=Path);p.add_argument('--git',action='store_true');a=p.parse_args()
    records=[]
    for i,line in enumerate(a.queue.read_text(encoding='utf-8').splitlines(),1):
        require(bool(line.strip()),f'line{i}: blank JSONL record');records.append(json.loads(line))
    require(bool(records),'empty queue');count=validate(records,a.queue.resolve().parents[3] if a.git else None)
    print(f'PASS {len(records)} records; {count} implementation units; contract integrity only, not gameplay acceptance')
if __name__=='__main__':main()
