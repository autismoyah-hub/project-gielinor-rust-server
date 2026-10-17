"""The Fallen Cinderworks: complete original offline environmental composition.

Coordinates are design tiles, not the functional Tower's gameplay coordinates.
All geometry is original. A conservative authored walk-mask is a feasibility
diagram only; it does not claim live collision, interactions or encounters.
"""
import argparse
from collections import Counter,deque
import json
import math
from pathlib import Path
import random

AREAS=[
 ('arrival','The Broken Processional',[(60,31),(74,31),(79,36),(76,43),(70,47),(62,46),(57,40)],'paving',(67,38)),
 ('sanctuary','The Fourfold Refuge',[(56,47),(72,47),(79,54),(79,60),(71,68),(59,68),(52,61),(52,54)],'sanctuary_paving',(66,57)),
 ('workshop','The Brass Service Hall',[(32,57),(42,54),(50,59),(52,69),(47,77),(33,78),(29,71),(29,63)],'forge_paving',(41,67)),
 ('quarry','The Fractured Haulage Court',[(29,79),(37,76),(44,81),(47,89),(40,96),(28,94),(25,88)],'ash_paving',(36,86)),
 ('waterworks','The Headgate and Wheel',[(83,54),(95,51),(103,59),(104,71),(98,79),(87,80),(79,73),(79,63)],'paving',(91,66)),
 ('garden','The Breached Root Cistern',[(93,79),(104,78),(111,85),(110,94),(102,99),(93,94),(90,87)],'garden_soil',(101,88)),
 ('vents','The Cinder Lung',[(77,83),(87,82),(94,89),(91,98),(83,103),(74,99),(70,91)],'ash_paving',(82,92)),
 ('narthex','The Last Cooling Gallery',[(59,82),(67,82),(72,87),(71,94),(65,99),(57,95),(55,88)],'paving',(63,90)),
 ('foundry','The Bound Crucible',[(54,98),(70,98),(77,105),(77,116),(70,123),(54,123),(47,116),(47,105)],'forge_paving',(62,110)),
 ('ascent','The Upper Bearing',[(56,122),(69,122),(74,127),(72,132),(58,133),(52,129)],'sanctuary_paving',(64,127)),
]
PATHS=[([(67,28),(67,37),(66,48)],5), ([(54,58),(49,59),(46,64)],4),
       ([(77,57),(83,58),(87,62)],4), ([(39,74),(37,80),(36,86)],4),
       ([(67,65),(65,75),(63,86)],5), ([(48,72),(53,77),(63,80),(65,85)],4),
       ([(91,75),(95,81),(98,85)],4), ([(86,77),(84,83),(82,88)],4),
       ([(68,76),(75,78),(79,85)],4), ([(74,96),(70,98),(68,102)],4),
       ([(63,94),(62,100),(62,105)],5), ([(39,93),(43,98),(49,104)],3),
       ([(70,120),(70,125),(66,127)],4)]

def inside(x,y,poly):
    result=False;j=len(poly)-1
    for i,(xi,yi) in enumerate(poly):
        xj,yj=poly[j]
        if (yi>y)!=(yj>y) and x<(xj-xi)*(y-yi)/(yj-yi)+xi:result=not result
        j=i
    return result

def distance(x,y,a,b):
    dx,dy=b[0]-a[0],b[1]-a[1]
    t=max(0,min(1,((x-a[0])*dx+(y-a[1])*dy)/(dx*dx+dy*dy)))
    return math.hypot(x-a[0]-t*dx,y-a[1]-t*dy)

def simplify(points,epsilon=.72):
    if len(points)<3:return points
    distances=[distance(*q,points[0],points[-1]) for q in points[1:-1]]
    if not distances or max(distances)<=epsilon:return [points[0],points[-1]]
    i=distances.index(max(distances))+1
    return simplify(points[:i+1],epsilon)[:-1]+simplify(points[i:],epsilon)

def curtain_recipe(cells):
    edges=set()
    for x,z in cells:
        for missing,start,end in [((x,z-1),(x,z),(x+1,z)),((x+1,z),(x+1,z),(x+1,z+1)),((x,z+1),(x+1,z+1),(x,z+1)),((x-1,z),(x,z+1),(x,z))]:
            if missing not in cells:edges.add((start,end))
    loops=[]
    while edges:
        start,end=min(edges);edges.remove((start,end));points=[start,end]
        while points[-1]!=points[0]:
            candidates=sorted(e for e in edges if e[0]==points[-1])
            if not candidates:break
            edge=candidates[0];edges.remove(edge);points.append(edge[1])
        if len(points)>4:
            points=points[:-1];split=max(range(1,len(points)),key=lambda i:math.dist(points[0],points[i]))
            simple=simplify(points[:split+1])[:-1]+simplify(points[split:]+[points[0]])[:-1]
            loops.append(simple+[simple[0]])
    grouped={key:[] for key,*_ in AREAS}
    for loop in loops:
        for start,end in zip(loop,loop[1:]):
            if max(start[1],end[1])<32:continue
            mid=((start[0]+end[0])/2,(start[1]+end[1])/2)
            key=min(AREAS,key=lambda row:math.dist(mid,row[4]))[0]
            grouped[key].append([start,end])
    return grouped

def main():
    p=argparse.ArgumentParser();p.add_argument('package',type=Path);p.add_argument('output',type=Path);a=p.parse_args()
    a.output.mkdir(parents=True,exist_ok=True)
    cat=json.loads((a.package/'catalogue.json').read_text());assets={v['name']:v for v in cat['assets']}
    cells={};props=[];blocked=set();bridges=set();random.seed(217)
    for x in range(25,113):
        for z in range(27,134):
            if any(any(distance(x+.5,z+.5,u,v)<=width/2 for u,v in zip(points,points[1:])) for points,width in PATHS):cells[x,z]=('spine','paving')
            for key,title,poly,material,center in AREAS:
                if inside(x+.5,z+.5,poly):cells[x,z]=(key,material)
    water=set()
    for x,z in cells:
        if cells[x,z][0]=='waterworks' and 53<=z<=78 and x in [91,92]:water.add((x,z))
        if cells[x,z][0]=='garden' and ((x+.5-100)/3.1)**2+((z+.5-88)/3.6)**2<1:water.add((x,z))
    # The cistern feeds the wheel; a narrow two-tile race links both bodies.
    for z in range(78,85):
        for x in [92+(z-78)//3,93+(z-78)//3]:
            if (x,z) in cells:water.add((x,z))

    def put(name,x,z,rotation=0,obstacle=True,story=''):
        entry=assets[name];w,l=entry['footprint'];w,l=(l,w) if rotation%2 else (w,l)
        row=dict(asset=name,id=entry['object_id'],x=x,z=z,shape=10,rotation=rotation,footprint=[w,l],story=story)
        props.append(row)
        if obstacle:
            for dx in range(w):
                for dz in range(l):
                    if name in ['arch','broken_arch','great_arch']:
                        across=dz if rotation%2 else dx
                        span=l if rotation%2 else w
                        if across not in [0,span-1]:continue
                    blocked.add((x+dx,z+dz))
        if name in ['bridge','long_bridge']:
            bridges.update((x+dx,z+dz) for dx in range(w) for dz in range(l))

    floors=[]
    for (x,z),(area,material) in sorted(cells.items()):
        if (x,z) in water:material='water'
        elif area=='sanctuary':
            radius=math.hypot(x+.5-66.5,z+.5-57.5)
            if 4.1<radius<5.5:material='paving'
        elif area=='foundry' and (x in [53,71] or z in [102,117]):material='ash_paving'
        elif area=='foundry' and 54<=x<=71 and 103<=z<=116:material='ash_paving'
        elif area=='garden' and 'moss_ground' in assets and math.hypot(x-101,z-88)<7:material='moss_ground'
        if material not in ['water','moss_ground'] and f'{material}_v0' in assets:
            variation=(x//3*7+z//4*11)%13
            if variation in [0,1,2]:material+=f'_v{variation%2}'
        floors.append(dict(asset=material,id=assets[material]['object_id'],x=x,z=z,shape=10,rotation=((x+z)%2)*2))

    # Outer masonry follows the complete irregular footprint, with a cutaway
    # near edge for player views. Water banks remain open and separately railed.
    for x,z in sorted(cells) if not any(n.startswith('curtain_') for n in assets) else []:
        area=cells[x,z][0]
        for dx,dz,rotation in [(0,1,0),(1,0,1),(0,-1,2),(-1,0,3)]:
            if (x+dx,z+dz) in cells:continue
            if z<32 and 63<=x<=70:continue
            broken=area in ['quarry','garden'] or area=='arrival' and (x+z)%7==0
            name='wall_broken' if broken else ('parapet' if rotation in [1,2] else 'wall')
            put(name,x,z,rotation,False)
    for name,entry in assets.items():
        if name.startswith('curtain_'):put(name,*entry['world_anchor'],obstacle=False)

    # Arrival: a monumental surviving gateway and visibly failed service arch.
    put('great_arch',64,32,story='The once-grand worker entrance still frames the refuge beyond.')
    put('broken_arch',60,40,1,story='Fallen stones mark an older collapsed side entry.')
    for x,z in [(60,34),(74,35),(59,39)]:put('rubble',x,z)
    for x,z in [(62,37),(73,38)]:put('brazier',x,z)
    put('banner',71,42);put('scale_figure',67,40,obstacle=False)

    # Refuge: a clear circular ambulatory, purposeful niches and one landmark.
    put('sanctuary_seal',63,54,obstacle=False)
    put('restoration_font',64,55)
    put('bank_counter',55,56,story='West niche: banking, guidance and a worker ledger.')
    put('workbench',72,60,story='Recovery and restoration alcove outside the circulation ring.')
    put('coolant_trough',70,51)
    for x,z in [(55,51),(75,53),(56,63),(73,64)]:put('pillar',x,z)
    for x,z in [(58,53),(73,62)]:put('banner',x,z)
    for x,z,r in [(64,46,0),(51,57,1),(77,56,1),(63,67,0)]:put('arch',x,z,r)
    for x,z in [(60,48),(71,48),(58,65),(71,65)]:put('brazier',x,z)
    put('scale_figure',61,57,obstacle=False)

    # Forge: working cast-house reads as a single process from cart to furnace,
    # then smithing and maintenance benches. Clear central handling space.
    put('furnace',32,68,story='The old smelting hearth receives ore directly from the haulage court.')
    put('furnace',39,73,2,story='A paired casting hearth has cooled to a dull red.')
    put('pressure_pump',46,70);put('anvil',44,64);put('anvil',47,66)
    for x,z,r in [(32,60,0),(36,57,0),(47,60,1)]:put('workbench',x,z,r)
    for x,z in [(31,64),(36,63),(44,74)]:put('stores',x,z)
    put('timber_stack',30,66);put('ore_cart',39,66)
    for z in range(74,87,2):put('track',37,z,obstacle=False)
    put('ore_cart',36,81);put('ore_outcrop',29,84);put('ore_outcrop',28,88)
    put('ore_outcrop',32,91);put('ore_outcrop',41,85);put('ore_outcrop',41,89)
    put('rubble',26,86);put('rubble',30,93);put('rubble',42,93)
    put('broken_arch',41,97,story='Collapsed maintenance return; future repair shortcut, not a required route.')
    put('rubble',43,98)
    for x,z in [(31,58),(49,69),(33,76)]:put('pillar',x,z)
    put('scale_figure',43,68,obstacle=False)
    if 'casting_hoist' in assets:
        put('casting_hoist',40,69)
        for x,z,r in [(30,61,0),(34,57,0),(46,74,1),(56,60,1),(96,76,0)]:put('supply_rack',x,z,r)
        for x,z in [(27,82),(28,90),(34,93),(42,90),(40,78),(106,94),(108,82)]:put('rock_buttress',x,z)

    # Headgate: visible water source, wheel, pressure conversion and pipework.
    put('waterwheel',90,63,story='Coolant drives the wheel before feeding the western pump bank.')
    for x,z in [(90,57),(90,69),(90,76)]:put('bridge',x,z,obstacle=False)
    for x,z in [(83,64),(85,70),(98,63)]:put('pressure_pump',x,z)
    for x,z in [(84,60),(84,62),(86,74),(86,76)]:put('pipe',x,z)
    put('coolant_trough',96,72);put('workbench',99,59,1);put('stores',99,68)
    for z in range(54,79,3):
        for x in [90,93]:
            if (x,z) in cells and (x,z) not in water:put('railing',x,z,1,False)
    put('grand_pillar',88,52);put('pillar',101,70)
    put('scale_figure',87,67,obstacle=False)
    # Breached roof admits roots; moisture, timber and fungi have a reason here.
    for x,z in [(105,82),(107,90),(94,92)]:put('root_tree',x,z)
    for x,z in [(96,84),(104,94),(92,86),(106,87)]:put('fungus',x,z)
    put('timber_stack',103,80);put('rubble',108,92);put('rubble',94,95)
    put('long_bridge' if 'long_bridge' in assets else 'bridge',97,87,obstacle=False)

    # Central heat transition: quiet, constricted cooling gallery before the
    # final broad reveal. Pipe pairs form a deliberate visual route north.
    for z in [71,75,79,84,90]:
        for x in [60,68]:
            if (x,z) in cells:put('pipe',x,z)
    for x,z in [(60,85),(67,85),(57,91),(69,91)]:put('pillar',x,z)
    put('coolant_trough',56,87);put('coolant_trough',67,93)
    put('arch',61,81);put('arch',60,95)
    for x,z in [(58,83),(68,93)]:put('brazier',x,z)

    # The Cinder Lung: edge machinery creates an arena, not clutter in it.
    for x,z in [(74,87),(87,86),(88,95),(75,97)]:put('vent',x,z)
    for x,z in [(72,90),(90,90),(81,99)]:put('pressure_pump',x,z)
    for x,z in [(73,95),(89,94),(76,84)]:put('rubble',x,z)
    for x,z in [(79,87),(84,87),(79,96),(84,96)]:put('metal_plate',x,z,obstacle=False)
    put('broken_arch',79,82);put('arch',71,96,1)
    put('scale_figure',81,93,obstacle=False)
    if 'heat_fissure' in assets:
        for x,z in [(76,89),(87,93),(82,97),(72,92),(89,88),(56,117),(68,117),(55,101),(69,101)]:put('heat_fissure',x,z,obstacle=False)

    # Climax: the clear combat rectangle is framed by tall buttresses, twin
    # coolant alcoves and a single unmistakable furnace silhouette.
    put('great_arch',59,97)
    put('great_crucible',58,117,story='The bound furnace is the floor’s visual and industrial destination.')
    for x,z in [(49,103),(74,103),(49,114),(74,114)]:put('grand_pillar',x,z)
    put('coolant_trough',49,109);put('coolant_trough',73,109)
    for x,z in [(50,106),(73,106)]:put('pressure_pump',x,z)
    for x,z in [(50,118),(71,119)]:put('vent',x,z)
    for x in range(55,71,3):put('metal_plate',x,102,obstacle=False);put('metal_plate',x,115,obstacle=False)
    for x,z in [(54,100),(70,100),(52,117),(72,117)]:put('brazier',x,z)
    put('scale_figure',62,109,obstacle=False)
    put('ascent_stair',60,125,story='A surviving upper bearing hints at the next floor; no live transition is implemented.')
    put('arch',68,122,1)
    put('banner',55,126);put('banner',69,129);put('brazier',56,128)

    walkable=(set(cells)-water-blocked)|bridges
    anchors={'arrival':(67,38),'sanctuary':(61,57),'workshop':(43,68),'quarry':(36,87),
             'waterworks':(87,67),'garden':(103,88),'vents':(82,93),'narthex':(64,90),
             'foundry':(62,109),'ascent':(67,127)}
    start=anchors['arrival'];visited={start};queue=deque([start])
    while queue:
        x,z=queue.popleft()
        for q in [(x-1,z),(x+1,z),(x,z-1),(x,z+1)]:
            if q in walkable and q not in visited:visited.add(q);queue.append(q)
    connectivity={name:tile in visited for name,tile in anchors.items()}
    scene=['# OFFLINE ORIGINAL ART ONLY — no cache scenery or gameplay integration','S 96 112']
    for row in floors+props:scene.append('L {x} {z} 0 {id} {shape} {rotation}'.format(**row))
    (a.output/'fallen-cinderworks.scene').write_text('\n'.join(scene)+'\n',encoding='utf-8')
    plan=dict(title='The Fallen Cinderworks',stage='Offline artistic proposal; not gameplay-tested content',revision=1,
              origin=[24,24],scene_size=[96,112],tile_units=128,areas=[dict(key=k,title=t,polygon=poly,material=m,center=c) for k,t,poly,m,c in AREAS],
              paths=[dict(points=p,width=w) for p,w in PATHS],placements=props,floor_tiles=len(floors),water_tiles=len(water),
              authored_walkable_tiles=len(walkable),reachable_walkable_tiles=len(visited),anchors=anchors,connectivity=connectivity,
              encounter_proposals={'ash_imp':dict(clear_bounds=[78,89,86,96],size=[9,8],entry=[81,85],exit=[73,97]),'ash_warden':dict(clear_bounds=[54,103,71,116],size=[18,14],entry=[62,99],coolant=[[50,110],[74,110]],exit=[69,122])},
              asset_usage=dict(Counter(p['asset'] for p in floors+props)),curtain_recipe=curtain_recipe(cells))
    (a.output/'design-manifest.json').write_text(json.dumps(plan,indent=2)+'\n')
    print('COMPOSITION',len(floors),'floor/water tiles',len(props),'architectural/prop placements')
    print('AUTHORED_MASK_CONNECTIVITY',json.dumps(connectivity))

if __name__=='__main__':main()
