"""Cumulative room milestones. All scenery is placed in the actual fixed map."""
from pathlib import Path
import csv,json
from package import generate,R,P
catalog=generate();inventory=list(csv.DictReader((R/'baseline-world/objects.tsv').open(),delimiter='\t'));rows=[]
def layer(s):return 0 if s<4 else 1 if s<9 else 2 if s<22 else 3
occupied={(int(r['x']),int(r['z']),layer(int(r['shape']))):int(r['id'])for r in inventory}
def add(asset,x,z,room,key,angle=0,shape=10,expected=None,note=''):
    data=catalog[asset];site=(x,z,layer(shape));old=occupied.get(site)
    if old!=expected:raise ValueError(('unexpected existing site',key,site,old,expected))
    if any((r['x'],r['z'],layer(r['shape']))==site for r in rows):raise ValueError(('duplicate planned site',key,site))
    w,h=data['size'];w,h=(h,w)if angle%2 else(w,h)
    for xx in range(x,x+w):
        for zz in range(z,z+h):
            assert not(50<=xx<=62 and 67<=zz<=77)and not(68<=xx<=75 and 68<=zz<=75),key
    rows.append(dict(key=key,asset=asset,object_id=data['object_id'],x=x,z=z,world=[10368+x,64+z,0],shape=shape,angle=angle,expected=expected,room=room,collision_footprint=[w,h],blocking=data['blocking'],interaction_position=None,animation_anchor=None,existing_gameplay_hook=None,status='Original decorative scenery; current gameplay anchors preserved',note=note))
def gate(x,z,room,key,vertical=False):
    # Desired opening centre is(x+1,z), or(x+1,z+1) for the rotated kit.
    add('arch',x,z,room,key,3 if vertical else 0)
    feet=[(x+1,z-1),(x+1,z+2)]if vertical else[(x-1,z),(x+2,z)]
    for i,(xx,zz)in enumerate(feet):add('arch-foot',xx,zz,room,key+'-foot-'+str(i),note='Real support footprint beneath matching visible arch pier; two complete walking tiles remain open.')
def save(stage):
    out=R/'placements';out.mkdir(exist_ok=True);text='CINDERWORKS_ENVIRONMENT 1\n'+''.join(f'{r["object_id"]} {r["x"]} {r["z"]} {r["shape"]} {r["angle"]} {r["expected"]if r["expected"]is not None else"empty"} {r["key"]}\n'for r in rows)
    (out/(stage+'.tsv')).write_text(text);(out/(stage+'.json')).write_text(json.dumps(rows,indent=2)+'\n');print('MILESTONE_PLACEMENTS',stage,len(rows))

# Architectural replacement is limited to static original pillars. The server
# uses the new wall palette at the exact existing shell edge coordinates.
for r in inventory:
    if int(r['id'])in[7016,7017]:add('pier',int(r['x']),int(r['z']),'architecture','existing-pier-'+r['x']+'-'+r['z'],int(r['angle']),int(r['shape']),int(r['id']),note='Same established support tile and ordinary collision footprint.')
save('02-architecture')

# Arrival / opening workshop: the visual standard is concentrated here first.
for x,z,room,key,vertical in [(55,48,'A','sanctuary-south',False),(47,55,'A','workshop-threshold',True),(63,55,'A','waterworks-threshold',True),(55,64,'A','sanctuary-north',False)]:gate(x,z,room,key,vertical)
for x,z in [(54,54),(56,54),(54,56),(56,56)]:add('paving-bond',x,z,'A',f'sanctuary-bond-{x}-{z}',shape=22)
add('plate-mill',40,55,'B','mill-feed-line');add('workstation',37,54,'B','riveting-bench');add('tool-rack',36,55,'B','bench-tool-rack');add('furnace-bank',39,49,'B','auxiliary-forge-bank')
for x,z in [(45,50),(39,53),(46,53)]:add('stock-crate',x,z,'B',f'workshop-stock-{x}-{z}')
for z in [34,36,38,40,42,44,46]:add('paving-bond',55,z,'S',f'processional-bond-{z}',shape=22)
save('03-arrival-workshop')

# Slate -> cart -> straight rail -> plate mill: gauge and centre line agree.
add('slate-headwall',39,62,'C','worked-slate-face');add('rail-span',40,57,'C','ore-track-south',shape=22);add('rail-span',40,59,'C','ore-track-north',shape=22);add('ore-cart',40,59,'C','loaded-slate-cart')
gate(39,64,'D1','grove-maintenance-arch')
for x,z in [(36,69),(39,70),(45,69),(40,75),(35,74),(45,76),(34,65),(44,67)]:add('moss-fern',x,z,'D1',f'reclaimed-growth-{x}-{z}',shape=22)
for x,z in [(36,67),(44,73),(40,77)]:add('ash-rubble',x,z,'D1',f'grove-fallen-stone-{x}-{z}',shape=22)
add('pump',68,61,'D2','headgate-pump');add('water-return',70,62,'D2','pump-return-to-channel',note='Exact local pump port (119,-18,73) meets return port(-265,-82,73); bends around preserved pillar(70,61). Ground-height offset must be verified on native terrain.')
add('sluice',74,62,'D2','abandoned-headgate');add('tool-rack',66,54,'D2','range-preparation-tools');add('stock-crate',66,55,'D2','kitchen-stock')
save('04-slate-water')

# Furnace approach and ash courts: no clutter inside preserved encounters.
gate(55,66,'E','foundry-threshold');gate(71,67,'F3','imp-threshold')
for x,z in [(66,69),(77,69),(66,73),(77,73)]:add('ash-rubble',x,z,'F3',f'vent-gallery-ash-{x}-{z}',shape=22)
for x,z in [(37,35),(44,43),(34,44),(67,38),(76,42),(74,46)]:add('ash-rubble',x,z,'F1'if x>64 else'F2',f'supply-yard-rubble-{x}-{z}',shape=22)
for x,z in [(33,40),(46,43),(65,40),(77,38)]:add('stock-crate',x,z,'F1'if x>64 else'F2',f'provision-stock-{x}-{z}')
save('05-furnace-ash')

# Monumental infrastructure is behind the encounter boundary, not on fighting tiles.
add('chimney-backdrop',49,78,'G','western-foundry-stack')
add('chimney-backdrop',63,78,'G','eastern-foundry-stack')
add('reward-surround',55,50,'H1','completion-chest-frame',note='Native model depth offset128 aligns around the existing chest at(55,51); chest ID and action unchanged.')
gate(63,59,'H2','departure-frame',True)
save('06-boss-reward')

for x,z in [(51,54),(60,54),(51,61),(60,61)]:add('moss-fern',x,z,'A',f'sanctuary-edge-growth-{x}-{z}',shape=22)
add('arrival-inlay',56,56,'A','first-arrival-inlay');add('arrival-inlay',56,49,'A','recovery-arrival-inlay');add('hearth-ring',55,58,'A','central-hearth-surround',shape=22)
save('07-whole-floor-refinement')
(P/'planning/prop-placement-contracts.json').write_text(json.dumps(rows,indent=2)+'\n')
(P/'planning/utility-connections.json').write_text(json.dumps({'ore_flow':['worked-slate-face','loaded-slate-cart','ore-track-north','ore-track-south','mill-feed-line'],'rail_gauge_units':96,'mill_and_rail_world_centre_x':10409,'water_connection':{'pump_anchor':[68,61],'pump_port_author':[119,-18,73],'return_anchor':[70,62],'return_port_author':[-265,-82,73],'endpoint_local_fine_xz':[8951,7918],'native_pump_ground':-240,'native_return_ground':0,'return_model_y_offset':-240,'expected_matching_port_native_y':-313,'height_status':'Matched using actual native height-grid samples; final render and native scene inspection remain required.'}},indent=2)+'\n')
