"""Pinned held-only Imcando behavior; worn swapping is separately contracted."""
import hashlib,json,re,sqlite3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification';SHA='7d4e09b6fdad766312ef55238e61c2730ea0b529';c=next(x for x in map(json.loads,(DOC/'IMPLEMENTATION_QUEUE.jsonl').read_text(encoding='utf-8').splitlines())if x['implementation_id']=='OR-MIG-0108');assert c['implementation_commit_sha']==SHA
archive=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main';db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True);texts={};sources=[]
for ref in c['source_references']:
 if ref['kind']=='openrune':data=(archive/ref['reference']).read_bytes()
 else:
  rev,text=db.execute('SELECT revision_id,wikitext FROM pages WHERE title=? COLLATE BINARY AND revision_id=?',(ref['reference'],ref['revision'])).fetchone();assert str(rev)==ref['revision'];data=text.encode()
 assert hashlib.sha256(data).hexdigest()==ref['sha256'];texts[ref['reference']]=data.decode();sources.append({k:ref[k]for k in['kind','reference','revision','sha256']})
db.close();source=texts[next(k for k in texts if k.endswith('/ImcandoHammerScript.kt'))]
assert 'onOpHeld3(STANDARD) { invReplace(inv, STANDARD, 1, OFFHAND) }' in source and 'onOpHeld3(OFFHAND) { invReplace(inv, OFFHAND, 1, STANDARD) }' in source
assert 'onOpWorn4(STANDARD)' in source and 'onOpWorn4(OFFHAND)' in source
for title,id,key in[('Imcando hammer',25644,'id1'),('Imcando hammer (off-hand)',29775,'id')]:
 text=texts[title];assert re.search(r'\|'+key+r'\s*=\s*'+str(id)+r'\s*\n',text);assert 'Swap' in text and 'free-to-play' in text and 'regardless of the player' in text
assert re.search(r'\|id2\s*=\s*25633\s*\n',texts['Imcando hammer'])
report={'implementation_id':'OR-MIG-0108','tested_sha':SHA,'status':'PASS_REFERENCE_FACTS','sources':sources,'scope':'Three exact source/Wiki hashes independently match repaired25644/offhand29775bidirectional held Swap; broken25633excluded. Source held3callbacks simply replace one item with no explicit membership/skill/quest/XP/message/delay gate. Separate worn4callbacks unequip/replace/conditionallyequip and are excluded from108. Target strict selected-slot/ordinary metadata/full-credit guards strengthen source inventory-wide replacement; sparse bags can insert at an earlier free slot. Both Wiki pages support members-only equipping and ordinary hammer functionality retained in F2P; they do not independently prove F2P Swap menu availability.','limitations':['Actual target cache/startup/native member/F2P effective menu/save/recovery acceptance remains separate.','No worn109/equipment/acquisition/repair or currentHEAD acceptance inherited.','No original donor executable/render/audio comparison claimed.']}
o=DOC/'independent/reference-imcando-held';o.mkdir(exist_ok=True);p=o/'comparison.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS three pinned held hammer references; nativeF2P remains separate')
