"""Independent pinned source/documentation reconciliation, including conflicts."""
import hashlib,json,re,sqlite3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification';SHA='0c489b46b552ad012e630810c1218dbaeb437a51'
c=next(r for r in reversed([json.loads(l) for l in (DOC/'IMPLEMENTATION_QUEUE.jsonl').read_text().splitlines()]) if r['implementation_id']=='OR-MIG-0064');assert c['implementation_commit_sha']==SHA
db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True);archive=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main';texts={};sources=[]
for ref in c['source_references']:
 if ref['kind']=='openrune':data=(archive/ref['reference']).read_bytes()
 else:
  rev,text=db.execute('SELECT revision_id,wikitext FROM pages WHERE title=? COLLATE BINARY AND revision_id=?',(ref['reference'],int(ref['revision']))).fetchone();assert str(rev)==str(ref['revision']);data=text.encode()
 assert hashlib.sha256(data).hexdigest()==ref['sha256'];texts[ref['reference']]=data.decode();sources.append({k:ref[k] for k in ['kind','reference','revision','sha256']})
db.close();pack=texts[next(k for k in texts if '/pack/' in k)];profiles=[]
for title,key,raw,out,level,xp,fail,low,high,source_low,sequence in [('Opal','opal',1625,1609,1,15,3.8,128,250,100,890),('Jade','jade',1627,1611,13,20,5,100,245,120,891),('Red topaz','red_topaz',1629,1613,16,25,6.3,90,240,140,892)]:
 t=texts[title];assert re.search(r'\|id\s*=\s*'+str(out)+r'\s*(?:\n|\|)',t)
 begin=t.index('|skill1 = Crafting');recipe=t[begin:t.index('\n}}',begin)]
 for field,value in [('skill1lvl',level),('skill1exp',xp),('ticks',2),('members','Yes')]:assert re.search(r'\|'+field+r'\s*=\s*'+str(value)+r'\s*(?:\n|\||$)',recipe),(title,field)
 assert re.search(r'\|low\d?='+str(low)+r'\|high\d?='+str(high),t)
 row=pack.split('row("dbrow.crafting_cut_'+key+'")',1)[1].split('\n            }',1)[0]
 for field,value in [('COL_SUCCESS_LOW',source_low),('COL_SUCCESS_HIGH',252)]:assert f'{field}, {value}' in row
 threshold=lambda l:(low*98+(high-low)*(max(1,min(99,l))-1)+49)//98+1
 profiles.append({'name':title,'raw':raw,'output':out,'level':level,'success_xp':xp,'failure_xp':fail,'wiki_endpoints':[low,high],'source_endpoints':[source_low,252],'sequence':sequence,'requirement_threshold':threshold(level),'level99_threshold':threshold(99),'boost120_threshold':threshold(120)})
assert '3.8 experience' in texts['Opal'] and re.search(r'\|skill1exp\s*=\s*3\.7\s*\n',texts['Crushed gem'])
assert r'\lfloor(\text{level}-1)*145/98+101\rfloor' in texts['Jade']
assert 'clamped to a maximum value of 99' in texts['Skilling success rate'] and '+ 0.5' in texts['Skilling success rate']
report={'implementation_id':'OR-MIG-0064','tested_sha':SHA,'campaign_kind':'historical','status':'IN_PROGRESS','source_documentation_subcheck':'PASS_WITH_REFERENCE_CONFLICTS','sources':sources,'profiles':profiles,'scope':'All12pinned hashes independently verified. Recipe levels/ordinary IDs/XP and three Wiki rate endpoint overrides match the declared adapter. Generic rounded probability documentation supports source-style1-99clamp and visible boosts above99 being ignored. No executable donor or random-stream parity inferred.','reference_conflicts':[{'finding':'PG-REF-001','fact':'Opal page states failure3.8XP while pinned Crushed gem creation row states3.7. Current implementation follows Opal and source3.8; contract does not acknowledge the pinned-page conflict.','requested_resolution':'FORGE record explicit source priority/adaptation or supply stronger original observed reward evidence; retain current independent3.8behavior result until adjudicated.'},{'finding':'PG-REF-001','fact':'Pinned Jade success formula uses floor without the generic+0.5 rounding; at level13 it yields118/256 versus the implemented/generic119/256. Endpoints agree.','requested_resolution':'FORGE explicitly reconcile the Jade-specific formula versus generic rounded rule, documenting the selected behavior or correction. No automatic expectation change to manufacture PASS.'}],'limitations':['Source endpoints differ from Wiki endpoints and are intentionally overridden.','Specific Jade rounding and Opal failure-XP documentation conflicts remain unresolved; no complete V4 acceptance.','Public matrix verifies declared rounded thresholds, not distribution or native RNG-stream equivalence.','No original executable, Jeweller chisel bonus, audio or acquisition comparison.']}
out=DOC/'independent/reference-fallible-gems';out.mkdir(exist_ok=True);assert not (out/'comparison.json').exists();(out/'comparison.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({'sources':len(sources),'profiles':len(profiles),'status':report['source_documentation_subcheck'],'finding':'PG-REF-001'}))
