"""Pinned stringing and staff quantity reference facts; manual timing remains separate."""
import hashlib,json,re,sqlite3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification';SHA='fb348dda184b36ec2d503bfe0f1d5393cf8ae151'
c=next(r for r in reversed([json.loads(l) for l in (DOC/'IMPLEMENTATION_QUEUE.jsonl').read_text().splitlines()]) if r['implementation_id']=='OR-MIG-0069');assert c['implementation_commit_sha']==SHA
db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True);archive=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main';texts={};sources=[]
for ref in c['source_references']:
 if ref['kind']=='openrune':data=(archive/ref['reference']).read_bytes()
 else:
  rev,text=db.execute('SELECT revision_id,wikitext FROM pages WHERE title=? COLLATE BINARY AND revision_id=?',(ref['reference'],int(ref['revision']))).fetchone();assert str(rev)==str(ref['revision']);data=text.encode()
 assert hashlib.sha256(data).hexdigest()==ref['sha256'];texts[ref['reference']]=data.decode();sources.append({k:ref[k] for k in ['kind','reference','revision','sha256']})
db.close();stringing=[]
amulets=['Gold','Sapphire','Emerald','Ruby','Diamond','Dragonstone','Onyx','Zenyte','Opal','Jade','Topaz']
rows=[(n+' amulet',n+' amulet (u)',raw,out) for n,raw,out in zip(amulets,c['identifiers']['amulet_raw_item_ids'],c['identifiers']['amulet_output_item_ids'])]+[('Unblessed symbol','Unstrung symbol',1714,1716),('Unpowered symbol','Unstrung emblem',1720,1722),('Strung rabbit foot','Rabbit foot',10134,10132)]
for title,rawtitle,raw,out in rows:
 text=texts[title];recipes=re.findall(r'\{\{Recipe\s*\n(.*?)\n\}\}',text,re.S);matches=[r for r in recipes if re.search(r'\|skill1\s*=\s*Crafting\s*\n',r) and re.search(r'\|mat[12]\s*=\s*Ball of wool\s*(?:\n|$)',r)];assert len(matches)==1,title;recipe=matches[0]
 for key,value in [('skill1exp',4),('ticks',2),('output1',title)]:assert re.search(r'\|'+key+r'\s*=\s*'+re.escape(str(value))+r'\s*(?:\n|\||$)',recipe),(title,key)
 assert set(re.findall(r'\|mat[12]\s*=\s*([^\n|]+)',recipe))=={rawtitle,'Ball of wool'},title
 for name,item in[(title,out),(rawtitle,raw)]:assert re.search(r'\|id\s*=\s*'+str(item)+r'\s*(?:\n|\|)',texts[name]),(name,item)
 stringing.append({'raw':raw,'wool':1759,'output':out,'xp_tenths':40,'fine_at_125':50,'wiki_ticks':2})
staff=[]
for element,xp in [('Water',1000),('Earth',1125),('Fire',1250),('Air',1375)]:
 text=texts[element+' battlestaff'];begin=text.index('|skill1 = Crafting');recipe=text[begin:text.index('\n}}',begin)]
 assert 'Make-X is 3t, then 2t for the remainder.' in recipe
 assert re.search(r'\|skill1exp\s*=\s*'+re.escape(str(xp/10).removesuffix('.0'))+r'\s*\n',recipe)
 staff.append({'element':element,'xp_tenths':xp,'fine_at_125':xp*125//100,'wiki_quantity_first':3,'wiki_quantity_repeat':2,'wiki_manual_ticks':2})
section=texts[next(k for k in texts if k.endswith('CraftingSection.kt'))];string_section=section.split('AMULET_STRINGING(',1)[1].split('\n    ),',1)[0];assert 'ticks = 2' in string_section and 'shortensFirstCraft = false' not in string_section
staff_section=section.split('BATTLESTAVES(',1)[1].split('\n    ),',1)[0];assert 'ticks = 2' in staff_section and 'shortensFirstCraft = false' in staff_section
report={'implementation_id':'OR-MIG-0069','tested_sha':SHA,'campaign_kind':'historical','status':'IN_PROGRESS','source_documentation_subcheck':'PASS','sources':sources,'stringing':stringing,'staff':staff,'scope':'All39pinned hashes independently verified. Fourteen owned-material wool conversions retain4XP/50fine at125%, paired item IDs and Wiki2ticks; source stringing2with default first shortening is declared first1/repeat2. Four staff pages explicitly give quantity first3/repeat2and manual2, with independently calculated per-step fractional1250/1406/1562/1718fine rewards. No manual acceptance inherited into new quantity dispatch.','limitations':['Source first-shortened stringing1versus Wiki nominal2requires actual native input/sequence/credit comparison; neither source declaration alone proves exact observed clock equivalence.','Existing member/current-level and actual cache/startup require independent exact-SHA scenarios; item pages also describe separate Magic String Jewellery which is not this wool route.','No quest/acquisition/equipment/original random stream/audio/rendered parity accepted.','Native token/quantity and durable partial/crash acceptance remain separate.']}
out=DOC/'independent/reference-pair-production';out.mkdir(exist_ok=True);assert not (out/'comparison.json').exists();(out/'comparison.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({'sources':len(sources),'stringing':len(stringing),'staff':len(staff),'source_documentation_subcheck':'PASS'}))
