"""Independent pinned-document/source fact checks; no donor code or assets exported."""
import hashlib,json,re,sqlite3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification';SHA='bc277dc3e7e15088323a4f048f57dc43185fb7c0'
c=next(r for r in reversed([json.loads(s) for s in (DOC/'IMPLEMENTATION_QUEUE.jsonl').read_text().splitlines()]) if r['implementation_id']=='OR-MIG-0071');assert c['implementation_commit_sha']==SHA
archive=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main';db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True);texts={};sources=[]
for ref in c['source_references']:
    if ref['kind']=='openrune':data=(archive/ref['reference']).read_bytes()
    else:
        rev,text=db.execute('SELECT revision_id,wikitext FROM pages WHERE title=? COLLATE BINARY AND revision_id=?',(ref['reference'],int(ref['revision']))).fetchone();assert str(rev)==str(ref['revision']);data=text.encode()
    assert hashlib.sha256(data).hexdigest()==ref['sha256'];texts[ref['reference']]=data.decode();sources.append({k:ref[k] for k in ['kind','reference','revision','sha256']})
db.close();profiles=[]
pack=texts[next(k for k in texts if '/pack/' in k)];section=texts[next(k for k in texts if k.endswith('CraftingSection.kt'))];worker=texts[next(k for k in texts if k.endswith('CraftingWorker.kt'))]
for title,output,count,level,row,symbol in [('Amethyst bolt tips',21338,15,83,'bolt_tips','xbows_bolt_tips_amethyst'),('Amethyst arrowtips',21350,15,85,'arrowtips','amethyst_arrowheads'),('Amethyst javelin tips',21352,5,87,'javelin_heads','amethyst_javelin_head'),('Amethyst dart tip',25853,8,89,'dart_tips','amethyst_dart_tip')]:
    text=texts[title];assert re.search(r'\|id\s*=\s*'+str(output)+r'\s*(?:\n|\|)',text)
    assert re.search(r'\|stackable\s*=\s*Yes',text) and re.search(r'\|members\s*=\s*Yes',text)
    begin=text.index('|skill1 = Crafting');recipe=text[begin:text.index('\n}}',begin)]
    for key,value in [('skill1lvl',level),('skill1exp',60),('members','Yes'),('ticks',2),('tools','Chisel'),('output1',title),('output1quantity',count)]:assert re.search(r'\|'+key+r'\s*=\s*'+re.escape(str(value))+r'\s*(?:\n|\||$)',recipe),(title,key)
    row_text=pack.split('row("dbrow.crafting_amethyst_'+row+'")',1)[1].split('\n            }',1)[0]
    for fact in ['input("obj.amethyst")','statReq("stat.crafting", '+str(level)+')','xp(600)','output("obj.'+symbol+'", '+str(count)+')']:assert fact in row_text
    profiles.append({'output':output,'count_per_raw':count,'level':level,'base_xp_per_raw':60,'fine_xp_at_125':750,'wiki_cycle_ticks':2,'source_first_ticks':1,'source_repeat_ticks':2})
assert re.search(r'\|id\s*=\s*21347\s*(?:\n|\|)',texts['Amethyst'])
assert re.search(r'AMETHYST\([\s\S]*?ticks = 2,',section) and 'shortensFirstCraft: Boolean = true' in section and 'firstCycleTicks = maxOf(1, cycleTicks - 1)' in worker
report={'implementation_id':'OR-MIG-0071','tested_sha':SHA,'campaign_kind':'historical','status':'IN_PROGRESS','source_documentation_subcheck':'PASS','sources':sources,'profiles':profiles,'scope':'All12pinned source/Wiki hashes verified. Four exact member/stackable outputs,15/15/5/8counts,83/85/87/89requirements and60XP per raw match pinned Wiki creation recipes and donor declarations. Source2cycle/default shortened first yields1then2; Wiki lists2cycle. Quantity counts raw batches, not individual ammunition tips.','limitations':['Source facts inspected; original Kotlin executable unavailable and not run.','Runtime/native first-clock timing, effects/audio, counted transport, overflow and persistence acceptance remain separate.','Acquisition/mining and downstream ammunition attachment/use are excluded.']}
out=DOC/'independent/reference-amethyst';out.mkdir(exist_ok=True);assert not (out/'comparison.json').exists();(out/'comparison.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({'sources':len(sources),'profiles':len(profiles),'source_documentation_subcheck':'PASS','tested_sha':SHA}))
