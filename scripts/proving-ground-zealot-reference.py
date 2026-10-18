"""Read-only comparison against the contract's seven exact protected references."""
import hashlib,json,sqlite3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
row=next(json.loads(s)for s in (ROOT/'docs/openrune/verification/IMPLEMENTATION_QUEUE.jsonl').read_text(encoding='utf-8').splitlines()if json.loads(s)['implementation_id']=='OR-MIG-0134')
donor=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main'
db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True)
sources=[]
for ref in row['source_references']:
 if ref['kind']=='openrune':raw=(donor/ref['reference']).read_bytes()
 else:
  text,=db.execute('SELECT wikitext FROM pages WHERE title=? COLLATE BINARY AND revision_id=?',(ref['reference'],ref['revision'])).fetchone();raw=text.encode('utf-8')
 assert hashlib.sha256(raw).hexdigest()==ref['sha256'],ref['reference']
 sources.append(ref)
db.close()
out=ROOT/'docs/openrune/verification/independent/reference-zealot-burial/comparison.json';assert not out.exists();out.parent.mkdir(parents=True,exist_ok=True)
result={'implementation_id':'OR-MIG-0134','tested_sha':row['implementation_commit_sha'],'status':'PASS_REFERENCE_FACTS','sources':sources,
 'scope':'Seven exact hashes independently match. Protected donor reads current hat/torso/legs/feet, sums 0.0125 each capped0.05, consumes with a random double below1-saveChance. Selected burial checks its original slot and excludes five named demonic ashes; both saved and consumed outcomes publish the same row XP/completion event. Pinned Wiki confirms1.25percent per piece/max5percent, ordinary burial and demonic ash exclusion. Four individual snapshots confirm25438/head,25434/body,25436/legs,25440/feet and members-only ordinary nonstackable equipment.',
 'adaptations':['Uniform80 outcomes preserve exact probability and upper save interval, without claiming Kotlin random-stream identity. Zero eligible chance avoids a donor random call.','Target stricter count1/vars0/catalog/current-member authority and staging-before-RNG are explicit independent adaptations. Inspected donor robe-helper body does not explicitly check membership/count/vars; possible global donor gates were not inspected.','Normal saved-bone XP is supported directly by donor action; Wiki describes repeat resource saving and increased aggregate XP rather than a special per-action multiplier.'],
 'limitations':['Reference facts only; no134target compile/cache/World/native/persistence acceptance yet.','Donor generated prayer metadata and executable241 runtime are unavailable; target239.1 numerical bone/XP/restore/animation profiles need separate verification.','Other robe consumers, acquisitions, equipment restrictions, altar synergy and storage/death remain outside selected burial contract.']}
out.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS_REFERENCE_FACTS',len(sources),'exact references')
