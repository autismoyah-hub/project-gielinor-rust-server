"""Independent framework/pinned-Wiki audit for seven deterministic gem cuts."""
import hashlib,json,re,sqlite3,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification';SHA='5743cbe1c7499e2a7476c721d25dcba194666042'
q=[json.loads(s) for s in (DOC/'IMPLEMENTATION_QUEUE.jsonl').read_text(encoding='utf-8').splitlines()];r=next(x for x in reversed(q) if x['implementation_id']=='OR-MIG-0056');assert r['implementation_commit_sha']==SHA
archive=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main';sources=[];texts={};db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True)
for ref in r['source_references']:
    if ref['kind']=='openrune':data=(archive/ref['reference']).read_bytes()
    else:
        rev,text=db.execute('SELECT revision_id,wikitext FROM pages WHERE title=? COLLATE NOCASE',(ref['reference'],)).fetchone();assert str(rev)==ref['revision'];data=text.encode()
    assert hashlib.sha256(data).hexdigest()==ref['sha256'],ref['reference'];texts[ref['reference']]=data.decode();sources.append({'reference':ref['reference'],'revision':ref['revision'],'sha256':ref['sha256'],'kind':ref['kind'],'method':'independently inspected source or pinned local Wiki snapshot'})
db.close();profiles=[]
for name,raw,cut,level,xp,member in [('Sapphire',1623,1607,20,50,False),('Emerald',1621,1605,27,67.5,False),('Ruby',1619,1603,34,85,False),('Diamond',1617,1601,43,107.5,False),('Dragonstone',1631,1615,55,137.5,True),('Onyx',6571,6573,67,167.5,True),('Zenyte',19496,19493,89,50,True)]:
    text=texts[name];assert re.search(r'\|skill1lvl\s*=\s*'+str(level)+r'\s*(?:\n|\|)',text);assert re.search(r'\|skill1exp\s*=\s*'+re.escape(str(xp))+r'\s*(?:\n|\|)',text)
    profiles.append({'name':name,'raw':raw,'cut':cut,'current_level':level,'base_xp':xp,'members':member,'fine_awards_by_rate':{str(rate):int(xp*10*rate/100) for rate in [100,125,500]}})
section=texts[next(k for k in texts if k.endswith('CraftingSection.kt'))];worker=texts[next(k for k in texts if k.endswith('CraftingWorker.kt'))]
assert 'shortensFirstCraft: Boolean = true' in section and 'firstCycleTicks = maxOf(1, cycleTicks - 1)' in worker
rust={}
for path in r['rust_paths']:
    data=subprocess.check_output(['git','-c','safe.directory='+ROOT.as_posix(),'show',SHA+':'+path],cwd=ROOT);rust[path]=hashlib.sha256(data).hexdigest()
out=DOC/'independent/reference-gems';out.mkdir(exist_ok=True);assert not (out/'comparison.json').exists()
report={'implementation_id':'OR-MIG-0056','revision':r['revision'],'tested_sha':SHA,'campaign_kind':'historical','level':'V4','status':'IN_PROGRESS','source_documentation_subcheck':'PASS','sources':sources,'rust_sha256':rust,'profiles':profiles,
 'comparisons':[
  {'behavior':'Seven exact rewards/requirements','finding':'Independently inspected pinned creation recipes support20/27/34/43/55/67/89and50/67.5/85/107.5/137.5/167.5/50XP. Zenyte cutting50is distinct from manufacturing uncut Zenyte. Target manual profile constants are independently checked by selected-slot/rate matrix; this audit does not copy generated donor product rows.'},
  {'behavior':'Shortened first cycle and scope','finding':'Inspected GEMS section uses2cycles and defaults shortened-first; worker computes initial maximum(1,2-1)=1. Manual one-pair local action is therefore a supported initial-cycle adaptation. Donor framework also supplies quantity UI, repeats and other sections that this contract explicitly excludes. Do not infer full Crafting or repeat cadence from this subset.'},
  {'behavior':'Retained chisel and transactional selection','finding':'Source checks tools separately from consumed materials and compensates failed output insertion by returning inputs. Rust binds exact clicked chisel/raw slots and stages inventory/Stats, retaining tool and untouched duplicates and refusing late invalid credit. This is intentionally stronger selected-slot/atomic policy, not an original-executable failure-atomicity observation.'},
  {'behavior':'Deterministic eligibility and presentation','finding':'Sapphire-or-rarer ordinary cuts use deterministic profile outcomes; semiprecious crush/other tool bonuses remain separate. Source binds human_cutting and chisel sound aliases. Target239.1 animation8833is resolved; chisel synth alias is absent and sound is explicitly omitted. Native visual/audio and precise action-clock observations remain separate obligations.'},
  {'behavior':'Native effective XP policy','finding':'Pinned runtime FloorOne login forces500percent even if SQLite seeded125. Native125percent is blocked by that policy; no production code or newer build is substituted. Independent authoritative100/125/500content fixtures remain separate from the actual500percent native campaign.'}],
 'reference_execution':'No compiled original Kotlin executable was available or run; donor statements are explicitly source-derived.',
 'limitations':['V4 remains partial for precise native animation/click timing, missing sound presentation, and original executable differential where available.','No full framework/product-table/quantity/modifier/acquisition parity inferred; protected source expressions/assets are not published.']}
(out/'comparison.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({'tested_sha':SHA,'source_checks':len(sources),'profiles':len(profiles),'subcheck':'PASS','V4':'IN_PROGRESS'}))
