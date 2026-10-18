"""Independent supported-source comparison for the ordinary food adapter."""
import hashlib,json,sqlite3,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification'
ARCHIVE=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main'
rows=[json.loads(s) for s in (DOC/'IMPLEMENTATION_QUEUE.jsonl').read_text(encoding='utf-8').splitlines()]
r=next(x for x in rows if x['implementation_id']=='OR-MIG-0024' and x['revision']==3);sha=r['implementation_commit_sha'];sources=[];texts={}
db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True)
for ref in r['source_references']:
    if ref['kind']=='openrune':data=(ARCHIVE/ref['reference']).read_bytes()
    else:
        rev,text=db.execute('SELECT revision_id,wikitext FROM pages WHERE title=? COLLATE NOCASE',(ref['reference'],)).fetchone()
        assert str(rev)==ref['revision'];data=text.encode()
    assert hashlib.sha256(data).hexdigest()==ref['sha256'];texts[ref['kind']]=data.decode()
    sources.append({'reference':ref['reference'],'revision':ref['revision'],'sha256':ref['sha256'],'method':'independently inspected source' if ref['kind']=='openrune' else 'independently inspected pinned Wiki from read-only local snapshot'})
db.close()
assert 'restores 20 [[Hitpoints]] when eaten' in texts['wiki'] and '|id = 385' in texts['wiki'] and '|members = Yes' in texts['wiki']
source=texts['openrune'];assert source.index('specialEffects.prepare(')<source.index('if (\n            !replaceConsumedStage(')<source.index('val oldHitpoints')
for term in ['slot = slot','invReplaceSlot(','invDel(','ConsumableDelayState.recordConsumption(','soundSynth(EAT_FOOD_SOUND)','defaultCanOverheal']:
    assert term in source
rust={};rust_text={}
for path in ['crates/world/src/food.rs','crates/world/src/simulation.rs','crates/server/src/runtime.rs','crates/server/src/content.rs','crates/server/src/startup.rs']:
    data=subprocess.check_output(['git','-c','safe.directory='+ROOT.as_posix(),'show',sha+':'+path],cwd=ROOT);rust[path]=hashlib.sha256(data).hexdigest();rust_text[path]=data.decode()
assert '("shark", 20)' in rust_text['crates/world/src/food.rs']
out=DOC/'independent/reference-ordinary-food';out.mkdir(parents=True,exist_ok=True)
assert not (out/'comparison.json').exists(),'Preserve completed comparison'
report={'implementation_id':'OR-MIG-0024','revision':3,'tested_sha':sha,'campaign_kind':'historical','level':'V4','status':'IN_PROGRESS','source_documentation_subcheck':'PASS','sources':sources,'rust_sha256':rust,
 'comparisons':[
  {'scope':'Selected consumption and stage replacement','finding':'Inspected donor dispatch captures clicked inventory, item type and slot; resolves food stage and replaces the next stage in that slot or deletes the terminal item before healing/effects. Rust actual EatItemSlot retains claimed identity and uses strict selected debit/remainder. Full-bag exchange and refusal with a matching unselected duplicate are supported comparison obligations, separately tested.'},
  {'scope':'Atomicity and invalid secondary effects','finding':'Donor prepares special effects before item replacement, then applies healing/delays/secondary effects. Rust stages inventory and all Stats before publication, rejecting invalid later secondary effects without debit, heal or events. This is an explicit stronger local rollback policy, not an observed Kotlin failure-atomicity claim.'},
  {'scope':'Ordinary healing and source coverage','finding':'Pinned Shark15183388 establishes ordinary member385, Eat and20HP; exact target static catalog matches20. This single pinned page does not prove every static food value. Rust caps ordinary healing at live base and preserves pre-existing overheal. Donor delegates maximum-healing policy and supports special/PvP-dependent cases; those cannot be inferred accepted from the ordinary adapter.'},
  {'scope':'Timing, presentation and excluded mechanics','finding':'Donor records distinct consumption/combat delays, resets facing and emits eating sound. Exact Rust adapter publishes animation/message/food delay; its optional multipart trace is not an attack-clock measurement. Existing ordinary local message and cake/pizza delay policy is an explicit adaptation. Rendered/audio behavior, exact combat cadence and area/activity parity remain open, rather than inferred from spaced native actions.'},
  {'scope':'Membership and special-food boundary','finding':'Rust trusted definition membership is checked before ordinary state mutation; do not infer a complete donor activity/access framework. Donor includes blighted Wilderness restriction, sweets run energy, dynamic/random healing and optional secondary effects. Later contracts own these mechanics; this unit does not certify them.'}],
 'reference_execution':'No compiled Kotlin executable was available or executed. These are independently inspected source-derived observations and a pinned-documentation comparison, not executable differential evidence.',
 'limitations':['Full V4 remains partial: detailed activity/combat timing, sound/presentation and complete ordinary profile reference coverage are not established by the two supplied references.','No protected source expressions/assets are published; hashes and paraphrased findings only.']}
(out/'comparison.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({'tested_sha':sha,'source_checks':len(sources),'subcheck':'PASS','V4':'IN_PROGRESS'}))
