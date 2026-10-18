"""Reconcile immutable independently checked recipe facts with contract63 dispatch."""
import hashlib,json,re,sqlite3,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification';SHA='143dab04f28c3daed844da0af33d6af5edcd962d';OUT=DOC/'independent/reference-production'
c=next(r for r in reversed([json.loads(s) for s in (DOC/'IMPLEMENTATION_QUEUE.jsonl').read_text().splitlines()]) if r['implementation_id']=='OR-MIG-0063');assert c['implementation_commit_sha']==SHA
g=json.loads((DOC/'independent/reference-gems/comparison.json').read_text());s=json.loads((DOC/'independent/reference-shells/comparison.json').read_text());assert g['source_documentation_subcheck']==s['source_documentation_subcheck']=='PASS'
prior_refs={r['reference']:r for r in g['sources']+s['sources']};archive=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main';db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True);texts={};sources=[]
for ref in c['source_references']:
    assert prior_refs[ref['reference']]['sha256']==ref['sha256']
    if ref['kind']=='openrune':data=(archive/ref['reference']).read_bytes()
    else:
        rev,text=db.execute('SELECT revision_id,wikitext FROM pages WHERE title=? COLLATE BINARY AND revision_id=?',(ref['reference'],int(ref['revision']))).fetchone();assert str(rev)==str(ref['revision']);data=text.encode()
    assert hashlib.sha256(data).hexdigest()==ref['sha256'];texts[ref['reference']]=data.decode();sources.append({k:ref[k] for k in ['kind','reference','revision','sha256']})
db.close();section=texts[next(k for k in texts if k.endswith('CraftingSection.kt'))];worker=texts[next(k for k in texts if k.endswith('CraftingWorker.kt'))]
for name,cycle in [('GEMS',2),('CARVING',3)]:assert re.search(name+r'\([\s\S]*?ticks = '+str(cycle)+',',section)
assert 'shortensFirstCraft: Boolean = true' in section and 'firstCycleTicks = maxOf(1, cycleTicks - 1)' in worker
startup=subprocess.check_output(['git','-c','safe.directory='+ROOT.as_posix(),'show',SHA+':crates/server/src/startup.rs'],cwd=ROOT)
text=startup.decode();gem=text[text.index('crate::content::gem_cutting('):text.index('crate::content::snelm_carving(')];shell=text[text.index('crate::content::snelm_carving('):text.index('crate::content::amulet_stringing(')]
assert 'ToolCraftingChoices' in gem and 'ToolCraftingChoices' in shell and re.search(r'cycle_ticks\s*:\s*2',gem) and re.search(r'cycle_ticks\s*:\s*3',shell)
profiles=[]
for p in g['profiles']:profiles.append({'raw':p['raw'],'output':p['cut'],'level':p['current_level'],'base_xp':p['base_xp'],'members':p['members'],'source_first':1,'source_repeat':2})
for p in s['profiles']:profiles.append({'raw':p['raw_id'],'output':p['output_id'],'level':p['level'],'base_xp':p['base_xp'],'members':p['member'],'source_first':2,'source_repeat':3})
assert len(profiles)==16 and profiles[6]['base_xp']==50 and sum(not p['members'] for p in profiles)==4
report={'implementation_id':'OR-MIG-0063','tested_sha':SHA,'campaign_kind':'historical','status':'IN_PROGRESS','source_documentation_subcheck':'PASS','sources':sources,'profiles':profiles,'startup_sha256':hashlib.sha256(startup).hexdigest(),'scope':'All22pinned source/Wiki hashes independently rechecked, matching previously independently audited immutable seven-gem/nine-shell recipe facts. Exact63startup now registers16single-choice captured-production handlers with gem2/shell3repeat metadata. Source shortened first gives1/2gems and2/3shells; member/F2P distinctions and Zenyte50/shell32.5XP remain exact.','limitations':['Reuse of immutable source facts is explicit; no manual56/57or moving-HEAD gameplay acceptance inherited.','New target native quantity/dialog/cadence and durable captured-batch acceptance remain separate.','Original compiled Kotlin executable unavailable; no rendered/audio or source-generated override execution inferred.']}
OUT.mkdir(exist_ok=True);assert not (OUT/'comparison.json').exists();(OUT/'comparison.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({'sources':len(sources),'profiles':len(profiles),'source_documentation_subcheck':'PASS','tested_sha':SHA}))
