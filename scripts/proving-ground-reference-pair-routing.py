"""New source-route subcheck; preserve the completed39reference campaign."""
import json,re,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification';SHA='fb348dda184b36ec2d503bfe0f1d5393cf8ae151';archive=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main'
c=next(json.loads(l) for l in (DOC/'IMPLEMENTATION_QUEUE.jsonl').read_text().splitlines() if json.loads(l)['implementation_id']=='OR-MIG-0069');assert c['implementation_commit_sha']==SHA
texts={};sources=[]
for ref in c['source_references']:
 if ref['kind']!='openrune' or Path(ref['reference']).name not in ['Crafting.kt','CraftingSection.kt','CraftingProduct.kt','CraftingWorker.kt']:continue
 data=(archive/ref['reference']).read_bytes();assert hashlib.sha256(data).hexdigest()==ref['sha256'];texts[Path(ref['reference']).name]=data.decode();sources.append({k:ref[k] for k in ['reference','revision','sha256']})
assert len(texts)==4
pack=texts['Crafting.kt'];section=texts['CraftingSection.kt'];worker=texts['CraftingWorker.kt'];product=texts['CraftingProduct.kt']
assert 'val shortensFirstCraft: Boolean = true' in section
assert 'firstCycleTicks = maxOf(1, cycleTicks - 1)' in worker and 'firstCycleTicks + phaseCompensation' in worker
assert 'ticks = ticks.ifEmpty { listOf(section.ticks) }' in product
facts=[]
names=['string_'+n+'_amulet' for n in ['gold','sapphire','emerald','ruby','diamond','dragonstone','onyx','zenyte','opal','jade','topaz']]+['string_symbol','string_emblem','strung_rabbit_foot','water_battlestaff','earth_battlestaff','fire_battlestaff','air_battlestaff']
for i,name in enumerate(names):
 marker='row("dbrow.crafting_'+name+'")';at=pack.index(marker);body=pack[at+len(marker):].split('\n            }',1)[0];group=re.findall(r'section\("([^"]+)"',pack[:at])[-1];inputs=re.findall(r'input\("obj\.([^\"]+)"(?:,\s*(\d+))?\)',body);out=re.findall(r'output\("obj\.([^\"]+)"',body);level=int(re.search(r'statReq\("stat.crafting",\s*(\d+)\)',body)[1]);xp=int(re.search(r'xp\((\d+)\)',body)[1]);assert len(inputs)==2 and len(out)==1 and all(n in ['', '1'] for _,n in inputs)
 if i<14:assert xp==40 and level==(37 if i==13 else 1) and group==('Combining' if i==13 else 'AmuletStringing')
 else:assert (level,xp)==[(54,1000),(58,1125),(62,1250),(66,1375)][i-14] and group=='Battlestaves'
 assert 'column(COL_TICKS' not in body
 facts.append({'source_row':name,'source_section':group,'inputs':[v for v,_ in inputs],'output':out[0],'crafting_level':level,'xp_tenths':xp,'row_tick_override':False})
for name,ticks in [('AMULET_STRINGING',2),('BATTLESTAVES',2),('COMBINING',1)]:assert 'ticks = '+str(ticks) in section.split(name+'(',1)[1].split('\n    ),',1)[0]
assert 'mode = CraftingMode.COMBINE' in section.split('COMBINING(',1)[1].split('\n    ),',1)[0]
r={'implementation_id':'OR-MIG-0069','tested_sha':SHA,'campaign_kind':'historical','status':'REFERENCE_QUALIFICATION_REQUIRED','finding_id':'PG-REF-002','implementation_blocking':False,'sources':sources,'source_rows':facts,'facts':'Thirteen wool rows are in source AmuletStringing2/default-shortened-first. Rabbit-foot row is instead Combining1/COMBINE, with no row tick override; source product defaults to its section ticks. All18source material pairs/current levels/fineXP rows independently extracted and matched. Source worker explicitly uses max(1,cycleTicks-1) when first-shortened, plus phase compensation.','difference':'Target quantity policy first1/repeat2for rabbit foot is supported by pinned Wiki2and independently passes World; it must not be described as inherited source AmuletStringing2. Source Combines full dispatch/repeat executable behavior is not inferred from the section constant alone.','correction':'Clarify OR69reference basis:13AmuletStringing routes versus Rabbit Combining1and explicit Wiki-backed target quantity/timing adaptation. Preserve expected target1/2, exact historical fbSHA and all completed evidence. No game-code defect or FORGE stop alleged; ordinary reference correction only.','acceptance_limit':'Keep V4partial until final source/Wiki/native comparison explicitly qualifies this difference. Original39reference artifact remains immutable and superseded only within this newly inspected source-route scope; no whole source parity claim.','prior_evidence':'docs/openrune/verification/independent/reference-pair-production/comparison.json'}
out=DOC/'independent/pair-reference-routing-addendum.json';assert not out.exists();out.write_text(json.dumps(r,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({'source_rows':len(facts),'source_hashes':len(sources),'status':r['status'],'finding':'PG-REF-002'}))
