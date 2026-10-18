"""Pinned crab documentation/source comparison with explicit Wiki overrides."""
import hashlib,json,re,sqlite3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification';SHA='894646a79fa4ba7e3e8b46af6ec000dc6b796514'
c=next(r for r in reversed([json.loads(l) for l in (DOC/'IMPLEMENTATION_QUEUE.jsonl').read_text().splitlines()]) if r['implementation_id']=='OR-MIG-0068');assert c['implementation_commit_sha']==SHA
db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True);archive=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main';texts={};sources=[]
for ref in c['source_references']:
 if ref['kind']=='openrune':data=(archive/ref['reference']).read_bytes()
 else:
  rev,text=db.execute('SELECT revision_id,wikitext FROM pages WHERE title=? COLLATE BINARY AND revision_id=?',(ref['reference'],int(ref['revision']))).fetchone();assert str(rev)==str(ref['revision']);data=text.encode()
 assert hashlib.sha256(data).hexdigest()==ref['sha256'];texts[ref['reference']]=data.decode();sources.append({k:ref[k] for k in ['kind','reference','revision','sha256']})
db.close();profiles=[]
for title,rawtitle,broken,raw,out,fail in [('Crab helmet','Fresh crab shell','Broken crab shell',7538,7539,7541),('Crab claw','Fresh crab claw','Broken crab claw',7536,7537,7540)]:
 text=texts[title];begin=text.index('|skill1 = Crafting');recipe=text[begin:text.index('\n}}',begin)]
 for key,value in [('skill1lvl',15),('skill1exp',32.5),('skill1boostable','Yes'),('members','Yes'),('ticks',3),('tools','Chisel'),('mat1',rawtitle),('output1',title)]:assert re.search(r'\|'+key+r'\s*=\s*'+re.escape(str(value))+r'\s*(?:\n|\||$)',recipe),(title,key)
 assert '|low1=40|high1=240|req1=15' in text
 assert re.search(r'\|skill1exp\s*=\s*0\s*(?:\n|\|)',texts[broken])
 for name,item in[(title,out),(rawtitle,raw),(broken,fail)]:assert re.search(r'\|id\s*=\s*'+str(item)+r'\s*(?:\n|\|)',texts[name])
 assert 'Mogre Camp' in texts[rawtitle] and 'Freeing Pirate Pete' in texts[rawtitle]
 profiles.append({'raw':raw,'success':out,'failure':fail,'level':15,'success_xp_tenths':325,'failure_xp_tenths':0,'fine_success_at_125':406,'wiki_ticks':3,'low':40,'high':240})
section=texts[next(k for k in texts if k.endswith('CraftingSection.kt'))];carving=section.split('CARVING(',1)[1].split('\n    ),',1)[0];assert 'ticks = 3' in carving and 'shortensFirstCraft' not in carving
pack=texts[next(k for k in texts if '/pack/' in k)]
for alias in ['crafting_crab_helmet','crafting_crab_claw']:
 row=pack.split('row("dbrow.'+alias+'")',1)[1].split('\n            }',1)[0]
 assert 'statReq("stat.crafting", 15)' in row and 'xp(325)' in row and 'COL_FAIL_ITEM' not in row
report={'implementation_id':'OR-MIG-0068','tested_sha':SHA,'campaign_kind':'historical','status':'IN_PROGRESS','source_documentation_subcheck':'PASS','sources':sources,'profiles':profiles,'scope':'All13pinned hashes independently verified. Source two rows agree on member Crafting15/32.5XP and no row quest/failure policy. Pinned Wiki adds40/240failure curves and explicit0XP broken outcomes; Wiki manual3first/repeat intentionally overrides donor CARVING3with default first shortening. Independent125%success406fineXP versus failure0 aligns with controlled byte tests.','limitations':['Crab item recipe notes mention partial Freeing Pirate Pete; fresh-material pages tie that quest to Mogre Camp access/acquisition. Contract deliberately supports already-owned materials without an added craft quest gate; no quest/acquisition parity accepted.','No original global random stream/audio/rendered executable observation claimed.','Actual World mixed-batch/quantity/native timing/lifecycle acceptance remains separate.']}
out=DOC/'independent/reference-crab';out.mkdir(exist_ok=True);assert not (out/'comparison.json').exists();(out/'comparison.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({'sources':len(sources),'source_documentation_subcheck':'PASS'}))
