"""Pinned ordinary light-orb wiring and uncharged helmet Crafting facts."""
import hashlib,json,re,sqlite3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification';SHA='c7ddebe844820260252fb37df518f8c271f37243'
c=next(r for r in reversed([json.loads(l) for l in (DOC/'IMPLEMENTATION_QUEUE.jsonl').read_text().splitlines()]) if r['implementation_id']=='OR-MIG-0070');assert c['implementation_commit_sha']==SHA
db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True);archive=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main';texts={};sources=[]
for ref in c['source_references']:
 if ref['kind']=='openrune':data=(archive/ref['reference']).read_bytes()
 else:
  rev,text=db.execute('SELECT revision_id,wikitext FROM pages WHERE title=? COLLATE BINARY AND revision_id=?',(ref['reference'],int(ref['revision']))).fetchone();assert str(rev)==str(ref['revision']);data=text.encode()
 assert hashlib.sha256(data).hexdigest()==ref['sha256'];texts[ref['reference']]=data.decode();sources.append({k:ref[k] for k in ['kind','reference','revision','sha256']})
db.close()
def field(text,key,value):assert re.search(r'\|'+key+r'\s*=\s*'+re.escape(str(value))+r'\s*(?:\n|\||$)',text),(key,value)
def recipe(title):
 rows=re.findall(r'\{\{Recipe\s*\n(.*?)\n\}\}',texts[title],re.S);hits=[r for r in rows if re.search(r'\|skill1\s*=\s*Crafting\s*\n',r)];assert len(hits)==1,title;return hits[0]
orb=recipe('Light orb');helm=recipe('Serpentine helm')
for key,value in [('skill1lvl',87),('skill1exp',104),('ticks',0),('mat1','Empty light orb'),('mat2','Cave goblin wire'),('output1','Light orb')]:field(orb,key,value)
for key,value in [('skill1lvl',52),('skill1exp',120),('ticks',1),('tools','Chisel'),('mat1','Serpentine visage'),('output1','Serpentine helm (uncharged)')]:field(helm,key,value)
for title,key,item in [('Light orb','id',10973),('Empty light orb','id',10980),('Cave goblin wire','id',10981),('Serpentine helm','id1',12929),('Serpentine helm','id2',12931),('Serpentine visage','id',12927)]:field(texts[title],key,item)
pack=texts[next(k for k in texts if k.endswith('/pack/Crafting.kt'))]
wiring=pack.split('row("dbrow.crafting_light_orb")',1)[1].split('\n            }',1)[0]
for token in ['input("obj.dorgesh_lightbulb_nofilament")','input("obj.dorgesh_wire")','statReq("stat.crafting", 87)','xp(1040)','output("obj.dorgesh_light_bulb")']:assert token in wiring
carve=pack.split('row("dbrow.crafting_serpentine_helm")',1)[1].split('\n            }',1)[0]
for token in ['category("Chisel")','input("obj.serpentine_visage")','statReq("stat.crafting", 52)','xp(1200)','output("obj.serpentine_helm")','columnRSCM(COL_TOOL, "obj.chisel")']:assert token in carve
section=texts[next(k for k in texts if k.endswith('CraftingSection.kt'))].split('COMBINING(',1)[1].split('\n    ),',1)[0];assert 'ticks = 1' in section
report={'implementation_id':'OR-MIG-0070','tested_sha':SHA,'campaign_kind':'historical','status':'IN_PROGRESS','source_documentation_subcheck':'PASS','sources':sources,'profiles':[{'input':[10980,10981],'output':10973,'members':True,'current_crafting':87,'xp_tenths':1040,'fine_at_125':1300,'wiki_ticks':0},{'tool':1755,'input':[12927],'output':12929,'excluded_charged_output':12931,'members':True,'current_crafting':52,'xp_tenths':1200,'fine_at_125':1500,'wiki_ticks':1}],'scope':'All12exact source/Wiki hashes independently match. Source rows and pinned Recipe facts agree on ordinary owned-material wiring87/104XP and chisel helmet52/120XP, material/output identities and uncharged helm12929distinct from charged12931. Wiki wiring0intentionally differs from generic donor Combining1; helmet Recipe1is explicit. Independent125percent awards1300/1500fineXP calculated.','limitations':['Reference facts only: World/cache/startup/native/lifecycle acceptance unexecuted at exact70.','Hash attestation plus generic Combining1does not itself prove full donor category dispatch or original executable timing; actual route and native clocks remain separate.','Light-orb item page includes unrelated lamp-repair500Crafting/500Firemaking actions; those are excluded from the owned-material104Crafting route.','Helmet charging/scales, combat/equipment/acquisition, rendered/audio and original executable parity are excluded. Chemistry stays deferred.']}
out=DOC/'independent/reference-crafting-finishes';out.mkdir(exist_ok=True);assert not(out/'comparison.json').exists();(out/'comparison.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({'source_documentation_subcheck':'PASS','sources':len(sources),'profiles':2}))
