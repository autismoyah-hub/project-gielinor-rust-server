"""Supported source and pinned-documentation audit for selected burial."""
import hashlib,json,sqlite3,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification';SHA='f6b0f6ad54fe7813bb6607827eeb8da0e60137fa'
q=[json.loads(s) for s in (DOC/'IMPLEMENTATION_QUEUE.jsonl').read_text(encoding='utf-8').splitlines()];r=next(x for x in q if x['implementation_id']=='OR-MIG-0051')
archive=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main';sources=[];texts={}
db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True)
for ref in r['source_references']:
    if ref['kind']=='wiki':
        rev,text=db.execute('SELECT revision_id,wikitext FROM pages WHERE title=? COLLATE NOCASE',(ref['reference'],)).fetchone();assert str(rev)==ref['revision'];data=text.encode()
    else:data=((ROOT.parent/'Project-Gielinor' if ref['kind']=='project_gielinor' else archive)/ref['reference']).read_bytes()
    assert hashlib.sha256(data).hexdigest()==ref['sha256'],ref['reference'];texts[ref['reference']]=data.decode()
    sources.append({'reference':ref['reference'],'kind':ref['kind'],'sha256':ref['sha256'],'revision':ref['revision'],'method':'independently inspected source/documentation; no original executable observation'})
db.close();assert 'Bones give 4.5' in texts['Bones'] and 'give 72 [[Prayer]] experience when buried' in texts['Dragon bones']
donor=texts[next(k for k in texts if k.endswith('PrayerBuryEvents.kt'))];pg=texts[next(k for k in texts if k.endswith('scripts/Prayer.kt'))]
for term in ['it.slot','task.slot','queue("queue.prayer_bury", 2','player.shouldConsume()','soundSynth(2738)']:assert term in donor
assert 'BURY_DELAY: Int = 1' in pg and 'invTotal(inv, bones)' in pg and 'invDel(inv, bones, 1)' in pg
rust={}
for p in r['rust_paths']:
    data=subprocess.check_output(['git','-c','safe.directory='+ROOT.as_posix(),'show',SHA+':'+p],cwd=ROOT);rust[p]=hashlib.sha256(data).hexdigest()
out=DOC/'independent/reference-selected-burial';out.mkdir(exist_ok=True);assert not (out/'comparison.json').exists()
report={'implementation_id':'OR-MIG-0051','tested_sha':SHA,'campaign_kind':'historical','level':'V4','status':'IN_PROGRESS','source_documentation_subcheck':'PASS','sources':sources,'rust_sha256':rust,
 'comparisons':[
  {'behavior':'Selected slot versus type-total reference','finding':'Inspected OpenRune handler retains clicked slot through queued completion and refuses if that slot no longer has the original type. Preserved local PG handler instead checks total count and deletes by type after suspension. Rust network route deliberately follows selected-slot safety while retaining legacy/reference APIs. Native stale replay and synthetic full-bag tests are separate runtime observations, not attributed to original Kotlin execution.'},
  {'behavior':'Fractional reward and effective configured policy','finding':'Pinned Bones15327876 gives4.5base XP and Dragon bones15344031 gives72. Independent read-only target cache resolves Bones45, Big bones150 and Dragon bones720 base fine units; at the controlled500percent rate awards225/750/3600fine. Other packed/custom/floor policies must be measured individually. Native integer22 cannot establish stored22.5 without SQLite fine-XP evidence.'},
  {'behavior':'Timing, effects and optional dependencies','finding':'OpenRune queues burial2, emits sound2738, optional Zealot conservation and a completion event. Local PG suspends1and Rust preserves one-clock internal completion. Exact input/animation cadence, sound and optional hook/equipment behavior are not certified parity. Donor ash differentiation and Catacombs hooks are separate contracts/dependencies.'},
  {'behavior':'Atomic rollback and restoration','finding':'Rust validates caller reward/rate/modifier/restore policy and publishes cloned PlayerData only on complete success. PG source deletes before restoration/XP. This is an intentional stronger rollback policy. Pinned necklace page supports bone-dependent restoration; exact target selected mapping covers only its configured subset. Known above-base legacy Heal refusal retains the bone safely but remains a functional restoration mismatch, not accepted parity.'},
  {'behavior':'Chaos and membership','finding':'OpenRune presents a bone confirmation in its area definition; local PG uses PrayerRules and Rust a configured rectangle. Exact shape/native cancellation, captured slot through confirmation and membership revocation require their own observations. Loaded target Dragon bones is member-only, ordinary/big bones are F2P; native game membership is trusted server state.'}],
 'reference_execution':'No compiled original Kotlin executable was available or run. Original behavior is source-derived; pinned documentation and local PG additions are explicitly separate references.',
 'limitations':['V4 remains partial for precise timing/presentation/sound, all packed reward/modifier rows, Chaos geometry and unresolved above-base necklace restoration.','No protected expressions/assets are published, only source hashes and independent paraphrases.']}
(out/'comparison.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({'tested_sha':SHA,'source_checks':len(sources),'subcheck':'PASS','V4':'IN_PROGRESS'}))
