"""Inspect actual furnace battery, boiler connections and earned court routes."""
import argparse,csv,json,time
from pathlib import Path
from tower_gateway import Gateway
p=argparse.ArgumentParser();p.add_argument('--directory',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--label',default='rebuilt');a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True);g=Gateway(a.directory,a.output/f'{a.label}-furnace-actions.ndjson');before=g.wait(lambda s:s['ready'] and 'Done: Claim reward'in s['journal'],120);g.act('inspect');rows=list(csv.DictReader((a.directory/'scene-objects.csv').open(encoding='utf8',newline='')))
expected=[(10419,142,18525,10,1),(10423,142,18525,10,1),(10428,142,18525,10,1),(10422,142,7016,10,0),(10427,142,7016,10,0),(10437,140,9116,10,0),(10442,140,9120,10,2)]
actual=[]
for x,z,id,shape,angle in expected:
 matches=[r for r in rows if r['plane']=='0' and r['x']==str(x) and r['z']==str(z) and r['shape']==str(shape) and r['rejected']=='false']
 assert len(matches)==1 and int(matches[0]['id'])==id and int(matches[0]['rotation'])==angle,(x,z,matches)
 actual.append(matches[0])
assert not any(r['id']=='6915'and r['x']=='10426'and r['z']=='142'and r['shape']=='10'and r['rejected']=='false'for r in rows),'Obsolete rubble overlaps furnace mouth'
visits=[]
for x,z in [(10424,130),(10422,135),(10427,135),(10422,138),(10427,138),(10423,136),(10439,136),(10424,120)]:
 g.act(f'walk {x} {z}');s=g.wait(lambda s:s['ready']and s['position'][1:]==[x,z],65);visits.append(s['position'])
after=g.observe();pack=lambda s:sorted((i['slot'],i['id'],i['count'])for i in s['items']if i['count']>0);assert pack(before)==pack(after)and before['equipment']==after['equipment']and before['xp']==after['xp']and before['journal']==after['journal'];(a.output/f'{a.label}-furnace-summary.json').write_text(json.dumps({'accepted':True,'actual':actual,'ordinary_earned_court_and_coolant_routes':visits,'exact_state_preserved':True,'scope':'Earned reconstruction and static routes only; fresh live encounters require separate tests'},indent=2));print('Actual three-mouth furnace bank, boiler connections and earned coolant/court routes PASS',flush=True)
