"""Read exact-title/revision pinned references, avoiding case-variant redirects."""
import hashlib,json,re,sqlite3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification';SHA='4c6b73cfcd3b35ce52aa79efd9d2a99464939445'
q=[json.loads(s) for s in (DOC/'IMPLEMENTATION_QUEUE.jsonl').read_text(encoding='utf-8').splitlines()];c=next(r for r in reversed(q) if r['implementation_id']=='OR-MIG-0057');assert c['implementation_commit_sha']==SHA
db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True);archive=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main';texts={};sources=[]
for ref in c['source_references']:
    if ref['kind']=='openrune':data=(archive/ref['reference']).read_bytes()
    else:
        rev,text=db.execute('SELECT revision_id,wikitext FROM pages WHERE title=? COLLATE BINARY AND revision_id=?',(ref['reference'],int(ref['revision']))).fetchone();assert str(rev)==str(ref['revision']);data=text.encode()
    assert hashlib.sha256(data).hexdigest()==ref['sha256'];texts[ref['reference']]=data.decode();sources.append({'reference':ref['reference'],'revision':ref['revision'],'sha256':ref['sha256'],'kind':ref['kind']})
db.close();names=['Myre snelm',"Blood'n'tar snelm",'Ochre snelm','Bruise blue snelm','Broken bark snelm','Pointed myre snelm',"Pointed blood'n'tar snelm",'Pointed ochre snelm','Pointed bruise blue snelm'];profiles=[]
for i,name in enumerate(names):
    text=texts[name]
    for key,value in [('id',3327+2*i),('members','Yes'),('skill1','Crafting'),('skill1lvl',15),('skill1exp',32.5),('ticks',2)]:
        assert re.search(r'\|'+key+r'\s*=\s*'+re.escape(str(value))+r'\s*(?:\n|\|)',text),(name,key)
    profiles.append({'name':name,'raw_id':3345+2*i,'output_id':3327+2*i,'member':True,'level':15,'base_xp':32.5,'wiki_recipe_ticks':2,'fine_awards':{'100':325,'125':406,'500':1625}})
section=texts[next(k for k in texts if k.endswith('CraftingSection.kt'))];worker=texts[next(k for k in texts if k.endswith('CraftingWorker.kt'))]
assert re.search(r'CARVING\([\s\S]*?ticks = 3,[\s\S]*?anim = CraftingConstants.ANIM_SNAIL_SHELL_CUT',section)
assert 'shortensFirstCraft: Boolean = true' in section and 'firstCycleTicks = maxOf(1, cycleTicks - 1)' in worker
report={'implementation_id':'OR-MIG-0057','tested_sha':SHA,'campaign_kind':'historical','status':'IN_PROGRESS','source_documentation_subcheck':'PASS','sources':sources,'profiles':profiles,'scope':'Four protected source files and10exact-title/revision pinned Wiki pages hash-verified. Nine ordinary round/pointed/bark output IDs, Crafting15/member/32.5XP and2tick recipes independently checked. Source CARVING3cycles/default first-cycle shortening gives2, agreeing with Wiki. Raw names round/pointed can coincide; IDs remain distinct and require independent selected/native routing checks.','reference_execution':'No compiled original Kotlin executable available; source statements are explicitly source-derived. Generated donor product rows were not copied or executed.','remaining':['Actual native animation1280 and2clock packet cadence, selected round/pointed directions and durable recovery remain separate.','Missing chisel sound, generic cut/carve wording, menus/repeats, later Vampyre/perfect-shell/acquisition/equipment exclusions remain explicit.']}
out=DOC/'independent/reference-shells';out.mkdir(exist_ok=True);assert not (out/'comparison.json').exists();(out/'comparison.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({'source_checks':len(sources),'profiles':len(profiles),'subcheck':'PASS','tested_sha':SHA}))
