"""Pinned held-only Imcando behavior; worn swapping is separately contracted."""
import hashlib,json,re,sqlite3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification';SHA='8bc76c7c464daf1d4abdef4dd7f4172c270abda5';c=next(x for x in map(json.loads,(DOC/'IMPLEMENTATION_QUEUE.jsonl').read_text(encoding='utf-8').splitlines())if x['implementation_id']=='OR-MIG-0109');assert c['implementation_commit_sha']==SHA
archive=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main';db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True);texts={};sources=[]
for ref in c['source_references']:
 if ref['kind']=='openrune':data=(archive/ref['reference']).read_bytes()
 else:
  rev,text=db.execute('SELECT revision_id,wikitext FROM pages WHERE title=? COLLATE BINARY AND revision_id=?',(ref['reference'],ref['revision'])).fetchone();assert str(rev)==ref['revision'];data=text.encode()
 assert hashlib.sha256(data).hexdigest()==ref['sha256'];texts[ref['reference']]=data.decode();sources.append({k:ref[k]for k in['kind','reference','revision','sha256']})
db.close();source=texts[next(k for k in texts if k.endswith('/ImcandoHammerScript.kt'))]
assert 'onOpHeld3(STANDARD) { invReplace(inv, STANDARD, 1, OFFHAND) }' in source and 'onOpHeld3(OFFHAND) { invReplace(inv, OFFHAND, 1, STANDARD) }' in source
assert 'onOpWorn4(STANDARD)' in source and 'onOpWorn4(OFFHAND)' in source
body=source.split('private fun ProtectedAccess.swapWorn(',1)[1].split('private companion object',1)[0];assert body.index('val targetFree')<body.index('wornUnequip(fromSlot)')<body.index('invReplace(inv, fromType, 1, toType)')<body.index('if (targetFree)')<body.index('invEquip(swapped)');assert 'if (unequip is WornUnequipResult.Fail)' in body and 'inv.slotOf(toType)' in body
for title,id,key in[('Imcando hammer',25644,'id1'),('Imcando hammer (off-hand)',29775,'id')]:
 text=texts[title];assert re.search(r'\|'+key+r'\s*=\s*'+str(id)+r'\s*\n',text);assert 'Swap' in text and 'free-to-play' in text and 'regardless of the player' in text
assert re.search(r'\|id2\s*=\s*25633\s*\n',texts['Imcando hammer'])
report={'implementation_id':'OR-MIG-0109','tested_sha':SHA,'status':'PASS_REFERENCE_FACTS','sources':sources,'scope':'Three exact source/Wiki hashes independently match the two worn4callback forms and original sequence: capture target-hand availability, attempt unequip with failure return, replace held form, and equip first matching output only if target was free. Source helper has no explicit entitlement gate; global effective equip policy remains unresolved. Wiki restricts new equipping to members and retains ordinary hammer functionality in F2P. Target109stages all operations and binds newly removed item, with F2Pconversion-to-bag/no-new-equip as an explicit adaptation; no native/source global parity inferred.','limitations':['Actual target cache/startup/native member/F2P effective menu/save/recovery acceptance remains separate.','No target World ordered refresh/full bag/occupied-hand/native/lifecycle or currentHEAD acceptance inherited.','No original donor executable/render/audio comparison claimed.']}
o=DOC/'independent/reference-imcando-worn';o.mkdir(exist_ok=True);p=o/'comparison.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS three pinned worn hammer references; global F2P equip policy unresolved')
