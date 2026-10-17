"""Catch real shrimp from every normal fishing spot behind the safe quay edge."""
import argparse,json,time
from pathlib import Path
from tower_gateway import Gateway
p=argparse.ArgumentParser();p.add_argument('--directory',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True);g=Gateway(a.directory,a.output/'four-fishing-positions-actions.ndjson')
count=lambda s,id:sum(i['count']for i in s['items']if i['id']==id)
def walk(x,z):
 end=time.monotonic()+65;retry=0
 while True:
  s=g.observe()
  if s['ready']and s['position'][1:]==[x,z]:return s
  if time.monotonic()>end:raise TimeoutError(f'Normal dry fishing approach blocked {x},{z}')
  if s['ready']and time.monotonic()>retry:g.act(f'walk {x} {z}');retry=time.monotonic()+4
  time.sleep(.2)
before=g.wait(lambda s:s['ready']and 'Done: Claim reward'in s['journal'],120);caught=[];drops=[]
for z in [114,116,118,120]:
 g.interface(35913795);s=g.wait(lambda s:s['ready'],30)
 if sum(i['count']>0 for i in s['items'])>=28:
  food=next(i for i in s['items']if i['id']==315 and 'Drop'in i['operations']);old=count(s,315);g.act(f"item {food['slot']} 7");s=g.wait(lambda s:count(s,315)==old-1,20);drops.append({'slot':food['slot'],'item':315,'count':1})
 s=walk(10438,z);fish=next(n for n in s['npcs']if n['id']==3317 and n['x']==10440 and n['z']==z);raw,xp=count(s,317),s['xp'][10];g.act(f"npc {fish['index']} 1");g.wait(lambda s:s['ready']and count(s,317)>raw and s['xp'][10]>xp,90);s=walk(10438,z-1);caught.append({'npc_position':[10440,z],'actual_index':fish['index'],'flat_stance':[10438,z],'raw_delta':count(s,317)-raw,'fishing_xp_delta':s['xp'][10]-xp})
after=walk(10424,120);fixed=lambda s:sorted((i['slot'],i['id'],i['count'])for i in s['items']if i['count']>0 and i['id']not in[315,317]);assert fixed(before)==fixed(after)and before['equipment']==after['equipment']and before['journal']==after['journal'];assert all(before['xp'][i]==after['xp'][i]for i in range(25)if i!=10);assert after['xp'][10]>before['xp'][10];assert count(after,315)==count(before,315)-len(drops);assert count(after,317)-count(before,317)==sum(r['raw_delta']for r in caught)
(a.output/'four-fishing-positions-summary.json').write_text(json.dumps({'accepted':True,'scope':'Four real normal NPC net actions from flat dry stances; unchanged existing fishing reach; only actual shrimp/FishingXP and recorded capacity drops of cooked food change','catches':caught,'normal_food_capacity_drops':drops,'all_other_items_equipment_journal_and24XP_preserved':True,'before':before,'after':after},indent=2));print('All four genuine quay fishing spots yield real shrimp and FishingXP; other earned state retained',flush=True)
