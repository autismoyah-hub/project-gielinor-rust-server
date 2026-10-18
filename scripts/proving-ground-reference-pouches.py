"""Pinned opening semantics only; do not inherit NPC acquisition acceptance."""
import hashlib,json,re,sqlite3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification';SHA='7fc1306bb5c01540038ce5ff175b577294ade432'
q=[json.loads(s) for s in (DOC/'IMPLEMENTATION_QUEUE.jsonl').read_text(encoding='utf-8').splitlines()];c=next(r for r in reversed(q) if r['implementation_id']=='OR-MIG-0054');assert c['implementation_commit_sha']==SHA
db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True);archive=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main';texts={};sources=[]
for ref in c['source_references']:
    if ref['kind']=='openrune':data=(archive/ref['reference']).read_bytes()
    else:
        rev,text=db.execute('SELECT revision_id,wikitext FROM pages WHERE title=? COLLATE BINARY AND revision_id=?',(ref['reference'],int(ref['revision']))).fetchone();assert str(rev)==str(ref['revision']);data=text.encode()
    assert hashlib.sha256(data).hexdigest()==ref['sha256'];texts[ref['reference']]=data.decode();sources.append({'reference':ref['reference'],'revision':ref['revision'],'sha256':ref['sha256'],'kind':ref['kind']})
db.close();wiki=texts['Coin pouch']
for key,value in [('id1','22521'),('bankable','No'),('options','Open-all, Open, Destroy')]:assert re.search(r'\|'+key+r'\s*=\s*'+re.escape(value)+r'\s*(?:\n|\|)',wiki)
script=texts[next(k for k in texts if k.endswith('ThievingScript.kt'))];table=texts[next(k for k in texts if k.endswith('ThievingTables.kt'))]
assert 'onOpHeld1(pouch.obj)' in script and 'onOpHeld2(pouch.obj)' in script and 'openPouches(pouch, all = true)' in script and 'openPouches(pouch, all = false)' in script
assert re.search(r'CoinPouch\("obj.pickpocket_coin_pouch_citizen", coins = 3\)',table)
out=DOC/'independent/reference-pouches';out.mkdir(exist_ok=True);assert not (out/'comparison.json').exists()
report={'implementation_id':'OR-MIG-0054','tested_sha':SHA,'campaign_kind':'historical','status':'IN_PROGRESS','source_documentation_subcheck':'PASS','sources':sources,'scope':'Five exact source/Wiki hashes verified. Source binds first held operation to all and second to one, with citizen pouch3coins; pinned Wiki supports22521, Open-all/Open and not bankable. This is opening semantics, not independent NPC acquisition/Thieving/cap acceptance.','comparisons':['Source all uses inventory type-total count/deletion, while Rust intentionally restricts one server-derived selected stack and preserves malformed duplicate stacks. Checked multiplication, output credit failure and whole-inventory staging are stronger atomic adaptations; do not infer donor failure atomicity.','Target native item operation ordinals2/3reflect its revision239.1 action encoding, while donor held1/held2name the conceptual first/second options. No client-supplied quantity is trusted.','Existing no_bank resolver and actual native deposit refusal, message/presentation observations and real durable restart remain separate. No RNG/XP/repeated queue is expected from opening.'],'reference_execution':'Original Kotlin executable unavailable; statements are source-derived. No source expression/generated tables/assets are published.'}
(out/'comparison.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({'source_checks':len(sources),'subcheck':'PASS','tested_sha':SHA}))
