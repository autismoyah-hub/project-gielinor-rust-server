"""Source-safe independently inspected dynamic food comparison, pinned by hashes."""
import hashlib,json,sqlite3,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification'
ARCHIVE=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main'
rows=[json.loads(s) for s in (DOC/'IMPLEMENTATION_QUEUE.jsonl').read_text(encoding='utf-8').splitlines()]
r=next(x for x in rows if x['implementation_id']=='OR-MIG-0035');sha=r['implementation_commit_sha']
sources=[];pages={}
db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True)
for ref in r['source_references']:
    if ref['kind']=='openrune':
        data=(ARCHIVE/ref['reference']).read_bytes();kind='independently_inspected_protected_source'
    else:
        row=db.execute('SELECT revision_id,wikitext FROM pages WHERE title=? COLLATE NOCASE',(ref['reference'],)).fetchone()
        assert row and str(row[0])==str(ref['revision']),(ref['reference'],row[0] if row else None,ref['revision']);data=row[1].encode();pages[ref['reference']]=row[1];kind='independently_inspected_pinned_documentation'
    actual=hashlib.sha256(data).hexdigest();assert actual==ref['sha256'],ref['reference']
    sources.append({'reference':ref['reference'],'sha256':actual,'revision':ref['revision'],'kind':kind})
db.close()
assert '34–50||3' in pages['Strawberry'] and '84–99||6' in pages['Strawberry']
assert '1 + 6%' in pages['Strawberry'] and '5% + 1' in pages['Watermelon slice']
assert '1+10%' in pages['Cooked sweetcorn'] and 'rounded down' in pages['Cooked sweetcorn']
assert 'healing identical' in pages['Sweetcorn (bowl)']
args=['git','-c','safe.directory='+ROOT.as_posix()]
rust={}
for path in ['crates/world/src/food.rs','crates/world/src/simulation.rs','crates/server/src/content.rs','crates/server/src/startup.rs']:
    data=subprocess.check_output([*args,'show',sha+':'+path],cwd=ROOT)
    rust[path]=hashlib.sha256(data).hexdigest()
report={'implementation_id':r['implementation_id'],'implementation_sha':sha,'level':'V4','status':'IN_PROGRESS',
 'source_documentation_subcheck':'PASS','sources':sources,'exact_rust_source_sha256':rust,
 'independent_comparison':[
  {'profile':'Strawberry / 5504','classification':'source_and_explicit_table_supported; prose_conflict_retained',
   'expectation':'Six-percent ceiling; healing increases at base levels 1,17,34,51,67,84. Explicit table gives base50->3 and84..99->6. Introductory 1+6% prose disagrees at some levels.',
   'resolution':'Current Rust chooses the explicit table and source special-effect arithmetic. This is a documented choice, not proof that all Wiki wording agrees.'},
  {'profile':'Watermelon slice / 5984','classification':'intentional_documented_donor_difference',
   'expectation':'Pinned Wiki gives floor five-percent plus one; at base20/40/60/80 this is2/3/4/5. Protected source instead uses five-percent ceiling and gives1/2/3/4 at those boundaries.',
   'resolution':'Current Rust chooses the Wiki table/prose. Do not report literal donor parity or award cutting seeds during Eat.'},
  {'profile':'Cooked sweetcorn / 5988 and bowl / 7088','classification':'supported_with_explicit_bowl_return_provenance',
   'expectation':'Floor ten-percent plus one, calculated from base HP. Protected special-effect handler and both pinned pages agree. Protected food stage list returns empty bowl1923; bowl Wiki page establishes healing but is silent on the return.',
   'resolution':'Rust supplies the common healing policy, with selected-slot bowl return. This does not certify cache registration or native reachability.'},
  {'profile':'Shared consumption, eligibility and publication','classification':'local_integration_policy_with_source_comparison',
   'expectation':'Source food script prepares the outcome, replaces the stage before healing/effects, and has ordinary three-cycle defaults. Rust computes from live base HP before its staged selected-slot transaction; preserves existing overheal and XP, applies member gates and emits only after success.',
   'resolution':'Independent action tests address arithmetic, atomicity and repeated input. Runtime binding, actual native cadence/sound/activity/combat interactions and persistence require separate acceptance.'}],
 'limitations':['No executable original-reference differential was performed; source behavior is inferred from inspected code.',
                'The strawberry prose/table conflict is preserved rather than treated as universally resolved documentation.',
                'Native presentation, complete registration/eligibility and durable scenarios remain open; this is a scoped comparison, not full feature acceptance.']}
out=DOC/'independent'/('reference-food-'+sha);out.mkdir(parents=True,exist_ok=True)
(out/'comparison.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps({'implementation_sha':sha,'source_checks':len(sources),'source_documentation_subcheck':'PASS','level_status':'IN_PROGRESS'}))
