"""Compare pinned elemental staff source declarations and manual Wiki recipes."""
import hashlib,json,re,sqlite3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification';SHA='e3ced82211ee438fd6a74a110d435e6746295a16'
c=next(r for r in reversed([json.loads(l) for l in (DOC/'IMPLEMENTATION_QUEUE.jsonl').read_text().splitlines()]) if r['implementation_id']=='OR-MIG-0067');assert c['implementation_commit_sha']==SHA
db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True)
archive=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main';texts={};sources=[]
for ref in c['source_references']:
 if ref['kind']=='openrune':data=(archive/ref['reference']).read_bytes()
 else:
  rev,text=db.execute('SELECT revision_id,wikitext FROM pages WHERE title=? COLLATE BINARY AND revision_id=?',(ref['reference'],int(ref['revision']))).fetchone();assert str(rev)==str(ref['revision']);data=text.encode()
 assert hashlib.sha256(data).hexdigest()==ref['sha256'];texts[ref['reference']]=data.decode();sources.append({k:ref[k] for k in ['kind','reference','revision','sha256']})
db.close();profiles=[];pack=texts[next(k for k in texts if '/pack/' in k)]
for element,orb,out,level,xp in [('Water',571,1395,54,1000),('Earth',575,1399,58,1125),('Fire',569,1393,62,1250),('Air',573,1397,66,1375)]:
 title=element+' battlestaff';text=texts[title];begin=text.index('|skill1 = Crafting');recipe=text[begin:text.index('\n}}',begin)]
 for key,value in [('skill1lvl',level),('skill1exp',xp/10),('skill1boostable','Yes'),('members','Yes'),('ticks',2),('mat1',element+' orb'),('mat2','Battlestaff'),('output1',title)]:
  val=str(value).removesuffix('.0');assert re.search(r'\|'+key+r'\s*=\s*'+re.escape(val)+r'\s*(?:\n|\||$)',recipe),(title,key,val)
 assert 'Make-X is 3t, then 2t for the remainder.' in recipe
 assert re.search(r'\|id\s*=\s*'+str(out)+r'\s*(?:\n|\|)',text)
 row=pack.split('row("dbrow.crafting_'+element.lower()+'_battlestaff")',1)[1].split('\n            }',1)[0]
 for fact in ['input("obj.battlestaff")','input("obj.'+element.lower()+'_orb")','statReq("stat.crafting", '+str(level)+')','xp('+str(xp)+')','output("obj.'+element.lower()+'_battlestaff")','spotanim.battlestaff_'+element.lower()+'_crafting_spotanim']:assert fact in row,(title,fact)
 profiles.append({'element':element,'orb':orb,'staff':1391,'output':out,'level':level,'xp_tenths':xp,'fine_xp_at_125':xp*125//100,'manual_ticks':2})
section=texts[next(k for k in texts if k.endswith('CraftingSection.kt'))].split('BATTLESTAVES(',1)[1].split('\n    ),',1)[0]
assert 'ticks = 2' in section and 'shortensFirstCraft = false' in section and 'ANIM_BATTLESTAFF' in section
report={'implementation_id':'OR-MIG-0067','tested_sha':SHA,'campaign_kind':'historical','status':'IN_PROGRESS','source_documentation_subcheck':'PASS','sources':sources,'profiles':profiles,'scope':'All11pinned reference hashes independently verified. Four source rows and pinned Wiki recipes agree on owned ingredient pairs, current Crafting54/58/62/66, member status, exact fractional100/112.5/125/137.5XP and manual2ticks. Source first action explicitly unshortened. Independent125%fine rewards1250/1406/1562/1718 distinguish per-award rounding.','limitations':['Wiki Make-X first3/repeat2 is automatic production, outside this manual registration.','Acquisition, orb enchanting, equipment/combat and other staves are separate.','Missing source attach_orb audio alias is declared omitted; no fabricated target sound or original executable/rendered observation claimed.','Native/runtime/cache/lifecycle acceptance remains separate.']}
out=DOC/'independent/reference-battlestaves';out.mkdir(exist_ok=True);assert not (out/'comparison.json').exists();(out/'comparison.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({'sources':len(sources),'source_documentation_subcheck':'PASS','profiles':profiles}))
