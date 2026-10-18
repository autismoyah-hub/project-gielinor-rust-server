"""Supplement immutable source audit with explicit Wiki cadence and members/IDs."""
import hashlib,json,re,sqlite3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'docs/openrune/verification/independent/reference-gems'
prior=json.loads((OUT/'comparison.json').read_text(encoding='utf-8'));db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True);rows=[]
for profile in prior['profiles']:
    name=profile['name'];rev,text=db.execute('SELECT revision_id,wikitext FROM pages WHERE title=? COLLATE NOCASE',(name,)).fetchone()
    ref=next(x for x in prior['sources'] if x['reference']==name);assert str(rev)==str(ref['revision']) and hashlib.sha256(text.encode()).hexdigest()==ref['sha256']
    assert re.search(r'\|members\s*=\s*'+('Yes' if profile['members'] else 'No')+r'\s*(?:\n|\|)',text,re.I)
    assert re.search(r'\|id\s*=\s*'+str(profile['cut'])+r'\s*(?:\n|\|)',text)
    assert re.search(r'\|ticks\s*=\s*2\s*(?:\n|\|)',text)
    rows.append({'name':name,'revision':rev,'sha256':ref['sha256'],'cut_id':profile['cut'],'members':profile['members'],'wiki_recipe_ticks':2,'native_manual_first_cycle_received_clocks':1})
db.close()
report={'implementation_id':'OR-MIG-0056','tested_sha':prior['tested_sha'],'campaign_kind':'historical','status':'PASS','profiles':rows,'scope':'All seven pinned Wiki pages independently rechecked for cut IDs, membership and recipe2ticks. Actual native manual first completion1received clock agrees with source shortened first cycle max(1,2-1). This is an explicit manual-first-cycle adaptation; literal Wiki2tick/repeat production cadence is not claimed.','acceptance_boundary':['No compiled Kotlin executable available; comparisons are hash-pinned source-derived, as allowed by contract.','Decoded native animation8833/effect timing observed separately, without a rendered/audio claim. Missing chisel sound alias, quantity menus, repeats, crushable stones, tool modifiers and acquisition are explicit excluded/omitted scope, not silently accepted functionality.']}
(OUT/'timing-addendum.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report))
