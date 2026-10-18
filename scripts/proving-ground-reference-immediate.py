"""Pinned reference facts for immediate Dramen and limestone actions; no gameplay claim."""
import hashlib,json,re,sqlite3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification';SHA='4f4aacb2929e8ad775d091527e2d80070100ee69';c=next(r for r in reversed([json.loads(s) for s in (DOC/'IMPLEMENTATION_QUEUE.jsonl').read_text().splitlines()]) if r['implementation_id']=='OR-MIG-0065');assert c['implementation_commit_sha']==SHA
db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True);archive=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main';texts={};sources=[]
for ref in c['source_references']:
    if ref['kind']=='openrune':data=(archive/ref['reference']).read_bytes()
    else:
        rev,text=db.execute('SELECT revision_id,wikitext FROM pages WHERE title=? COLLATE BINARY AND revision_id=?',(ref['reference'],int(ref['revision']))).fetchone();assert str(rev)==str(ref['revision']);data=text.encode()
    assert hashlib.sha256(data).hexdigest()==ref['sha256'];texts[ref['reference']]=data.decode();sources.append({k:ref[k] for k in ['kind','reference','revision','sha256']})
db.close();profiles=[]
for title,output,level,xp,tool in [('Dramen staff',772,31,0,'Knife'),('Limestone brick',3420,12,6,'Chisel'),('Rock (limestone)',968,12,1.5,'Chisel')]:
    text=texts[title];assert re.search(r'\|id\s*=\s*'+str(output)+r'\s*(?:\n|\|)',text);begin=text.index('|skill1 = Crafting');recipe=text[begin:text.index('\n}}',begin)]
    for key,value in [('skill1lvl',level),('skill1exp',xp),('ticks',0),('members','Yes'),('tools',tool),('output1',title)]:assert re.search(r'\|'+key+r'\s*=\s*'+re.escape(str(value))+r'\s*(?:\n|\||$)',recipe),(title,key)
    profiles.append({'name':title,'output':output,'level':level,'base_xp':xp,'fine_xp_at_125':int(xp*10)*125//100,'wiki_ticks':0,'members':True})
for title,raw in [('Dramen branch',771),('Limestone',3211)]:assert re.search(r'\|id\s*=\s*'+str(raw)+r'\s*(?:\n|\|)',texts[title])
assert '1.5 experience' in texts['Rock (limestone)'] and 'stop failing at level 40' in texts['Rock (limestone)'] and 'Partial completion of [[Lost City]]' in texts['Dramen staff']
pack=texts[next(k for k in texts if '/pack/' in k)];dramen=pack.split('row("dbrow.crafting_dramen_staff")',1)[1].split('\n            }',1)[0];limestone=pack.split('row("dbrow.crafting_limestone_brick")',1)[1].split('\n            }',1)[0]
for fact in ['statReq("stat.crafting", 31)','xp(0)','column(COL_TICKS, 0)']:assert fact in dramen
for fact in ['statReq("stat.crafting", 12)','xp(60)','COL_SUCCESS_LOW, 137','COL_SUCCESS_HIGH, 434','COL_FAIL_ITEM']:assert fact in limestone
assert 'COL_FAIL_XP' not in limestone
section=texts[next(k for k in texts if k.endswith('CraftingSection.kt'))];worker=texts[next(k for k in texts if k.endswith('CraftingWorker.kt'))];product=texts[next(k for k in texts if k.endswith('CraftingProduct.kt'))]
assert re.search(r'LIMESTONE\([\s\S]*?ticks = 0,',section) and 'if (cycleTicks <= 0)' in worker and '(failXp ?: 0)' in product
threshold=lambda level:min(256,max(0,(137*98+297*(max(1,min(99,level))-1)+49)//98+1));assert threshold(12)==171 and threshold(39)<256 and threshold(40)==256
report={'implementation_id':'OR-MIG-0065','tested_sha':SHA,'campaign_kind':'historical','status':'IN_PROGRESS','source_documentation_subcheck':'PASS','sources':sources,'profiles':profiles,'scope':'All12pinned source/Wiki hashes verified. Dramen31/zeroXP/zero-cycle and limestone12/6successXP/1.5failureXP/zero-cycle facts match exact recipe variants. Donor endpoints137/434independently yield171of256at12and certainty40with the declared rounded byte rule; Wiki independently supports certainty40, not the whole lower-level curve. Donor limestone failure XP is omitted/default zero; Wiki1.5override is explicit.','limitations':['Partial Lost City on the Wiki is distinguished from acquiring an already-owned branch; no quest engine or missing source row gate inferred.','Actual immediate dispatch, optional advancement/RNG consumption/native messages/effects and durability still require exact-SHA execution.','Original compiled Kotlin executable unavailable; no original random stream, rendering/audio or complete low-level probability parity claimed.']}
out=DOC/'independent/reference-immediate';out.mkdir(exist_ok=True);assert not (out/'comparison.json').exists();(out/'comparison.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({'sources':len(sources),'profiles':len(profiles),'source_documentation_subcheck':'PASS','tested_sha':SHA}))
