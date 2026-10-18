"""Pinned manual Crafting reference facts; cooking and automatic production excluded."""
import hashlib,json,re,sqlite3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification';SHA='5582c276d1657a0142151936e5a9fec022d7c938';c=next(r for r in reversed([json.loads(l) for l in (DOC/'IMPLEMENTATION_QUEUE.jsonl').read_text().splitlines()]) if r['implementation_id']=='OR-MIG-0066');assert c['implementation_commit_sha']==SHA
db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True);archive=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main';texts={};sources=[]
for ref in c['source_references']:
 if ref['kind']=='openrune':data=(archive/ref['reference']).read_bytes()
 else:
  rev,text=db.execute('SELECT revision_id,wikitext FROM pages WHERE title=? COLLATE BINARY AND revision_id=?',(ref['reference'],int(ref['revision']))).fetchone();assert str(rev)==str(ref['revision']);data=text.encode()
 assert hashlib.sha256(data).hexdigest()==ref['sha256'];texts[ref['reference']]=data.decode();sources.append({k:ref[k] for k in ['kind','reference','revision','sha256']})
db.close();text=texts['Sinew'];begin=text.index('|skill1 = Crafting');recipe=text[begin:text.index('\n}}',begin)]
for key,value in [('skill1lvl',10),('skill1exp',15),('skill1boostable','Yes'),('members','Yes'),('ticks',1),('tools','Knife'),('mat1','Damaged monkey tail'),('output1','Sinew')]:assert re.search(r'\|'+key+r'\s*=\s*'+str(value)+r'\s*(?:\n|\||$)',recipe),(key,value)
assert 'Make-all is 0, 3, then 4 ticks.' in recipe
for title,item in [('Sinew',9436),('Damaged monkey tail',19665)]:assert re.search(r'\|id\s*=\s*'+str(item)+r'\s*(?:\n|\|)',texts[title])
pack=texts[next(k for k in texts if '/pack/' in k)];row=pack.split('row("dbrow.crafting_sinew")',1)[1].split('\n            }',1)[0]
for fact in ['damaged_ballista_rope','xbows_sinew','statReq("stat.crafting", 10)','xp(150)']:assert fact in row,fact
assert 'COL_FAIL_ITEM' not in row;section=texts[next(k for k in texts if k.endswith('CraftingSection.kt'))];assert re.search(r'KNIFE\([\s\S]*?ticks = 1,[\s\S]*?ANIM_KNIFE_CUTTING',section)
report={'implementation_id':'OR-MIG-0066','tested_sha':SHA,'campaign_kind':'historical','status':'IN_PROGRESS','source_documentation_subcheck':'PASS','sources':sources,'profile':{'raw':19665,'tool':946,'output':9436,'level':10,'base_xp':15,'fine_xp_at_125':187,'manual_ticks':1,'source_alias':'damaged_ballista_rope','target_display_name':'Damaged monkey tail','members':True},'scope':'All6pinned hashes independently verified. Source owned damaged-tail alias, Crafting10/15XP and no failure row align with the pinned Wiki manual knife/tail/Sinew recipe and1tick; Knife section declares1. Source alias versus target display-name adaptation is explicit. Wiki Make-all0/3/4note is not manual timing or implemented automatic production.','limitations':['Actual startup/cache/native timing, authoritative conservation and durable reload remain separate.','Cooking from meat, Hunter acquisition, automatic Make-all, downstream spinning and audio/rendered parity are outside this manual adapter.','No original compiled Kotlin executable observation claimed.']}
out=DOC/'independent/reference-sinew';out.mkdir(exist_ok=True);assert not (out/'comparison.json').exists();(out/'comparison.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({'tested_sha':SHA,'sources':len(sources),'source_documentation_subcheck':'PASS'}))
