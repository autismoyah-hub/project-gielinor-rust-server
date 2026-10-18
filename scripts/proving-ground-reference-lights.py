"""Independent pinned light-source facts; publish observations and hashes only."""
import hashlib,json,re,sqlite3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification';SHA='ad739676dd01f5be809cdfe5ecf4f1e6ea77238a'
c=next(x for x in map(json.loads,(DOC/'IMPLEMENTATION_QUEUE.jsonl').read_text(encoding='utf-8').splitlines()) if x['implementation_id']=='OR-MIG-0084');assert c['implementation_commit_sha']==SHA
archive=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main';original=ROOT.parent/'Project-Gielinor';db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True);texts={};sources=[]
for ref in c['source_references']:
 if ref['kind']=='openrune':data=(archive/ref['reference']).read_bytes()
 elif ref['kind']=='project_gielinor':data=(original/ref['reference']).read_bytes()
 else:
  rev,text=db.execute('SELECT revision_id,wikitext FROM pages WHERE title=? COLLATE BINARY AND revision_id=?',(ref['reference'],ref['revision'])).fetchone();assert str(rev)==ref['revision'];data=text.encode()
 assert hashlib.sha256(data).hexdigest()==ref['sha256'];texts[(ref['kind'],ref['reference'])]=data.decode();sources.append({k:ref[k] for k in ['kind','reference','revision','sha256']})
db.close()
def field(text,key,value):assert re.search(r'\|'+key+r'\s*=\s*'+re.escape(str(value))+r'\s*(?:\n|\||$)',text),(key,value)
rows=[('Candle',36,33,1,'candle'),('Black candle',38,32,1,'black_candle'),('Torch',596,594,1,'torch'),('Candle lantern',4529,4531,4,'candle_lantern'),('Candle lantern',4532,4534,4,'candle_lantern_black'),('Oil lamp',4522,4524,12,'oil_lamp'),('Oil lantern',4537,4539,26,'oil_lantern'),('Bullseye lantern',4548,4550,49,'bullseye_lantern'),('Mining helmet',5014,5013,65,'mining_helmet')]
pack=texts[next(k for k in texts if k[1].endswith('/Firemaking.kt'))]
for title,raw,lit,level,key in rows:
 text=texts[('wiki',title)];assert re.search(r'\|id\d*\s*=\s*'+str(raw)+r'\s*\n',text);assert re.search(r'\|id\d*\s*=\s*'+str(lit)+r'\s*\n',text);field(text,'members','Yes');info=re.search(r'\{\{Firemaking info(.*?)\}\}',text,re.S)[1];field(info,'level',level);field(info,'type','light');row=pack.split('row("dbrow.light_source_'+key+'")',1)[1].split('}',1)[0];assert 'column(COL_LEVEL_LIGHT, '+str(level)+')' in row
field(texts[('wiki','Tinderbox')],'id',590);field(texts[('wiki','Tinderbox')],'members','No')
events=texts[next(k for k in texts if k[1].endswith('/LightSourceEvents.kt'))];assert 'onOpHeldU("obj.tinderbox", row.unlit)' in events and 'onOpHeld2(row.lit)' in events and 'player.firemakingLvl < source.level' in events;assert not re.search(r'\b(?:queue|anim|xp|statAdvance)\(',events);assert 'You extinguish the light source.' in events
mapping=[]
for k,text in texts.items():
 if k[1].endswith('/HeldOpScript.kt'):
  mapping.append({'kind':k[0],'buttons':re.findall(r'IfButtonOp\.Op(\d+)\s*->\s*HeldOp\.Op(\d+)',text)})
assert len(mapping)==2 and all(x['buttons']==[['2','1'],['3','2'],['4','3'],['6','4'],['7','5']] for x in mapping)
report={'implementation_id':'OR-MIG-0084','tested_sha':SHA,'status':'PASS_REFERENCE_FACTS','sources':sources,'profiles':[{'title':title,'unlit':raw,'lit':lit,'firemaking_level':level}for title,raw,lit,level,_ in rows],'held_mappings':mapping,'scope':'Thirteen exact source/original-Kotlin/Wiki hashes independently match. Nine completed owned light types and current Firemaking levels1/1/1/4/4/12/26/49/65 agree; Tinderbox590 retained. Source is immediate inventory replacement with no queue, animation or XP calls; its generic held2 extinguish differs from actual target item-specific operations. Target selected-slot atomic/member/ordinary-item guards are deliberate stronger policy. Oil filling, candle/frame acquisition, worn and ambient effects excluded; Wiki acquisition/filling recipe ticks do not prove lighting latency. Native raw clocks/container/message and lifecycle acceptance remain separate.','limitations':['No donor executable or rendered/native frame or audio parity claimed.','Wiki Firemaking info omits XP field on most pages; no-XP behavior independently established from source and World/native state, not inferred from that omission.','Donor lighting success uses unlit display name while target names can be shorter; feedback comparison must preserve wording differences.']}
o=DOC/'independent/reference-lights';o.mkdir(exist_ok=True);p=o/'comparison.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print('PASS thirteen pinned light references/nine profiles; native/persistence separate')
