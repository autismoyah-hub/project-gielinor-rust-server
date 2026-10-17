"""Independent, source-safe comparison of super-combat production references."""
import hashlib,json,re,sqlite3,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
ARCHIVE=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main'
DOC=ROOT/'docs/openrune/verification';OUT=DOC/'independent/deep-combat'
SHA='5a260c1bd6efa5f8a221bed7c96e6ac72fd428bc'
def sha(data):return hashlib.sha256(data).hexdigest()
work=json.loads((DOC/'WORKLIST.json').read_text(encoding='utf-8'))
rows=[json.loads(s) for s in (DOC/'independent'/('queue-'+work['forge_head']+'.jsonl')).read_text(encoding='utf-8').splitlines()]
r=next(x for x in rows if x['implementation_id']=='OR-MIG-0010')
sources=[]
for source in r['source_references']:
    if source['kind']!='openrune':continue
    p=ARCHIVE/source['reference'];actual=sha(p.read_bytes());assert actual==source['sha256']
    sources.append({'reference':source['reference'],'sha256':actual,'kind':'directly_inspected_source','revision':'configured 241; upstream commit unavailable'})
for name in ['HerbloreGoggles.kt','AmuletOfChemistry.kt']:
    p=ARCHIVE/'content/skills/herblore/src/main/kotlin/org/rsmod/content/skills/herblore'/name
    sources.append({'reference':str(p.relative_to(ARCHIVE)),'sha256':sha(p.read_bytes()),'kind':'directly_inspected_source','revision':'same protected local archive'})
db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True)
pages={}
for title in ['Super combat potion','Prescription goggles','Herblore']:
    row=db.execute('SELECT title,revision_id,wikitext FROM pages WHERE title=? COLLATE NOCASE',(title,)).fetchone();assert row
    pages[title]=row[2]
    sources.append({'reference':row[0],'revision':row[1],'sha256':sha(row[2].encode()),'kind':'pinned_public_documentation_from_read_only_local_snapshot'})
    if title=='Super combat potion':
        expected=next(s for s in r['source_references'] if s['kind']=='wiki')
        assert row[1]==expected['revision'] and sha(row[2].encode())==expected['sha256']
db.close()
wiki=pages['Super combat potion']
for expected in ['|skill1lvl = 90','|skill1boostable = Yes','|skill1exp = 150','|ticks = 2','|mat1 = Super attack(4)','|mat4 = Torstol','|mat4 = Torstol potion (unf)','|output1 = Super combat potion(4)']:
    assert expected in wiki,expected
assert '[[torstol]]s/[[torstol potion]]s when mixing [[super combat potion]]s' in pages['Prescription goggles']
assert 'requires completion of the [[Druidic Ritual]] quest' in pages['Herblore']
claims=[
 {'behavior':'Four-dose attack/strength/defence and clean or unfinished Torstol produce one four-dose super combat','classification':'documented_adaptation','reference':'Wiki Super combat potion revision 15281186; donor Herblore.kt lines 410-418 uses three-dose inputs/output and clean Torstol only','rust':'content::combat_mixing_recipes; combat_mixing::Job::complete'},
 {'behavior':'Herblore 90, boostable; base reward 150 XP','classification':'supported','reference':'Pinned Wiki Creation recipes; donor table agrees level/reward, while donor initial UI uses base stat','rust':'live current Herblore gate; skills::advance at personal rate; at 5x expect 750 XP / 7500 fine XP per operation'},
 {'behavior':'Two-tick production','classification':'documented_adaptation','reference':'Pinned Wiki recipes specify two ticks; FinishedPotionsEvents.kt queues first after one and repeats after four','rust':'simulation/combat_mixing_actions.rs; native authoritative traces must separately establish actual cadence'},
 {'behavior':'Goggles may retain Torstol only; always-four-dose chemistry is ineligible','classification':'supported_adaptation','reference':'Prescription goggles revision 15240969 identifies Torstol forms; donor Goggles helper can retain all listed secondaries; donor Chemistry helper excludes Torstol/four-dose upgrades','rust':'combat_mixing::Job::complete prepares normal/saved candidates; only ingredient index 3 can be retained'},
 {'behavior':'Failed transaction preserves all ingredients, XP and RNG; live eligibility rechecked','classification':'stronger_local_transaction_policy','reference':'Donor FinishedPotionsEvents.kt consumes with compensating refunds and rolls chemistry before later checks; this does not establish donor atomicity','rust':'prepared inventory/stat candidates before draw and publication; requires headless failure tests'},
 {'behavior':'Druidic Ritual and membership required','classification':'supported','reference':'Herblore revision 15347160 and Super combat potion recipe membership','rust':'authoritative queued completion gates checked in exact target'},
]
report={'implementation_id':'OR-MIG-0010','implementation_sha':SHA,'level':'V4','status':'IN_PROGRESS',
        'subchecks':{'source_and_pinned_documentation_comparison':'PASS'},'sources':sources,'claims':claims,
        'limitations':['No executable OpenRune 241 differential was performed; source-derived behavior is not an observed reference execution.',
                       'No claim of literal donor parity: dose policy, boost policy, timing and transaction publication intentionally differ.',
                       'Native modifier combinations and error paths require corresponding gameplay evidence; reference comparison alone cannot certify them.']}
OUT.mkdir(parents=True,exist_ok=True)
(OUT/'reference-comparison.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps({'implementation_sha':SHA,'source_checks':len(sources),'status':report['status'],'source_documentation_subcheck':'PASS'},indent=2))
