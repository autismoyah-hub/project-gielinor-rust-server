"""Pinned ordinary normal-furnace lead smithing reference facts."""
import hashlib,json,re,sqlite3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];DOC=ROOT/'docs/openrune/verification';SHA='7bacd9305d84df87a5ebdef95449f1dbc783d8bc'
c=next(r for r in reversed([json.loads(l) for l in (DOC/'IMPLEMENTATION_QUEUE.jsonl').read_text().splitlines()]) if r['implementation_id']=='OR-MIG-0096');assert c['implementation_commit_sha']==SHA
db=sqlite3.connect((ROOT.parent/'osrs-wiki-dump/data/osrs-wiki.sqlite').as_uri()+'?mode=ro',uri=True);archive=ROOT.parent/'source-reference/OpenRune-Server-main/OpenRune-Server-main';texts={};sources=[]
for ref in c['source_references']:
 if ref['kind']=='openrune':data=(archive/ref['reference']).read_bytes()
 else:
  rev,text=db.execute('SELECT revision_id,wikitext FROM pages WHERE title=? COLLATE BINARY AND revision_id=?',(ref['reference'],int(ref['revision']))).fetchone();assert str(rev)==str(ref['revision']);data=text.encode()
 assert hashlib.sha256(data).hexdigest()==ref['sha256'];texts[ref['reference']]=data.decode();sources.append({k:ref[k] for k in ['kind','reference','revision','sha256']})
db.close()
import subprocess
bar=texts['Lead bar'];rows=re.findall(r'\{\{Recipe\s*\n(.*?)\n\}\}',bar,re.S);normal=[r for r in rows if re.search(r'\|output1subtxt\s*=\s*Normal furnace\s*(?:\n|$)',r)];assert len(normal)==1;normal=normal[0]
def field(text,key,value):assert re.search(r'\|'+key+r'\s*=\s*'+re.escape(str(value))+r'\s*(?:\n|\||$)',text),(key,value)
for k,v in [('skill1','Smithing'),('skill1lvl',25),('skill1exp','15.5'),('mat1','Lead ore'),('mat1quantity',2),('output1','Lead bar'),('ticks',5)]:field(normal,k,v)
assert 'Make-X is 4, then 5. A single bar is 6.' in normal
field(bar,'id',32889);field(texts['Lead ore'],'id',31716)
source=texts[next(k for k in texts if k.endswith('/Smithing.kt'))].split('row("dbrow.lead")',1)[1].split('\n        }',1)[0]
for token in ['input("obj.lead_ore", 2)','statReq("stat.smithing", 25)','xp(15)','output("obj.lead_bar")']:assert token in source
pinned={p:subprocess.check_output(['git','show',SHA+':'+p]) for p in ['crates/server/src/content/lead_smelting.rs','crates/server/src/runtime/smelting.rs']}
content=pinned['crates/server/src/content/lead_smelting.rs'].decode();runtime=pinned['crates/server/src/runtime/smelting.rs'].decode()
for token in ['bar: 32889','level: 25','xp: 15.5','ingredients: vec![(31716, 2)]','members: true','repeat_delay: Some(6)']:assert token in content
assert 'xp_rate: 1.0' in runtime
report={'implementation_id':'OR-MIG-0096','tested_sha':SHA,'campaign_kind':'historical','status':'IN_PROGRESS','source_documentation_subcheck':'PASS_WITH_EXPLICIT_TIMING_AND_RATE_LIMITATIONS','sources':sources,'pinned_rust_sha256':{p:hashlib.sha256(data).hexdigest() for p,data in pinned.items()},'facts':{'input':31716,'input_count':2,'output':32889,'current_smithing':25,'member':True,'source_base_xp':15,'wiki_and_target_base_xp':15.5,'fine_at_100':155,'fine_at_125':193,'wiki_make_x_first':4,'wiki_repeat':5,'wiki_single':6,'target_runtime_rate':1.0,'target_repeat_queue_constant':6},'scope':'Five pinned source/Wiki hashes independently match. Source two ores/Smithing25/integer15XP and exact Wiki normal-furnace15.5XP adaptation verified; output32889/ore31716and per-step155/193fine awards independently calculated. Wiki explicitly distinguishes Make-X4then5from single6. Exact target content retains two ores/member25/15.5XP/repeat queue6; actual runtime initializes rate1.0.','limitations':['No source or queue constant is treated as observed elapsed timing. Initial/repeat/single native clocks remain untested; declared implementation lacks a dedicated single6path.','Runtime1.0does not establish intended server-wide5xrate integration; this known integration limit remains explicit.','No World/cache/startup/native/persistence acceptance executed by this reference campaign.','Normal furnace only; Blast Furnace/Superheat, equipment modifiers, acquisition and shipbuilding excluded. Chemistry remains deferred.']}
out=DOC/'independent/reference-lead-smelting';out.mkdir(exist_ok=True);assert not(out/'comparison.json').exists();(out/'comparison.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({'pinned_references':len(sources),'reference_facts':'PASS','timing_and_runtime_rate':'EXPLICIT_LIMITATIONS_RETAINED'}))
