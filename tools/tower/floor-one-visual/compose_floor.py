"""Author and render a complete original offline location through the native path."""
import argparse,json,sys,subprocess,math,hashlib,copy
from pathlib import Path
from collections import deque
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'asset-workbench'))
from templates import part
from construction import asset,arch
from production_designs import provenance
from primitives import make
from layout import AREAS,CONNECTORS,OUTER,BRIDGES,ROUTES,region,water,bridge,elevation,inside
HERE=Path(__file__).resolve().parent;TOOLS=HERE.parent/'asset-workbench';TILE=128
load=lambda p:json.loads(p.read_text(encoding='utf-8'))

def ground_recipe():
    components=[];cells={};walk=set();palette={'arrival':10272,'sanctuary':10285,'workshop':10274,'waterworks':10279,'quarry':6680,'slag':10263,'foundry':10272,'connection':10274}
    for ix in range(-26,22):
        for iz in range(-28,27):
            x,z=ix*TILE+64,iz*TILE+64;area=region(x,z)
            if not area:continue
            wet=water(x,z);cross=bridge(x,z);level=elevation(x,z);cells[(ix,iz)]=dict(area=area,water=wet and not cross,elevation=level)
            if wet:
                components.append(part(f'water_{ix}_{iz}','box','water',role='recessed_water',center=[x,12,z],size=[128,3,128]))
            if not wet or cross:
                walk.add((ix,iz));mat='wood'if cross else'floor_stone';gap=0 if area in ['quarry','slag']else 2;top=-10 if cross else level
                color=palette[area]+[-2,0,1,0][(ix*3+iz)%4]
                if area=='quarry':color=10270 if x<-2050 and z<-540 else 6438+(3 if abs(x+1750)<140 else 0)
                height=10 if cross else 184-top;components.append(part(f'floor_{ix}_{iz}','box',mat,role='terrain',center=[x,top+height/2,z],size=[128-gap,height,128-gap],variation=(ix+iz)%3-1,surface_bands=[]if cross else[dict(axis=1,min=top-1,max=top+1,hsl=color)]))
    # Local solid bedrock cells avoid enormous cap triangles crossing rooms in
    # the native model painter. Walking slabs themselves extend to this base.
    for ix in range(-26,22):
        for iz in range(-28,27):
            x,z=ix*128+64,iz*128+64
            if not inside(x,z,OUTER):continue
            if(ix,iz)in walk and not bridge(x,z):continue
            components.append(part(f'bedrock_{ix}_{iz}','box','bedrock',role='continuous_foundation',center=[x,104,z],size=[128,160,128]))
    for(ix,iz),cell in cells.items():
        if cell['water']:continue
        x,z=ix*128+64,iz*128+64
        for side,(dx,dz)in enumerate([(1,0),(-1,0),(0,1),(0,-1)]):
            other=cells.get((ix+dx,iz+dz));edge=[x+dx*64,z+dz*64]
            if other and other['water']:
                components.append(part(f'bank_{ix}_{iz}_{side}','stone_block','stone',role='channel_bank',center=[edge[0],2,edge[1]],size=[18 if dx else 126,22,126 if dx else 18],bevel=2,variation=-1));continue
            if other:continue
            # Deliberately lower the southern cutaway edge; tall north rock faces
            # retain context without a uniform wall obscuring every player view.
            height=90 if z>1100 else 170+(ix+iz)%3*14;angle=math.pi/2 if dx else 0;co,si=math.cos(angle),math.sin(angle)
            square=[[-1,-1],[1,-1],[1,1],[-1,1]];material='stone'if cell['area']in ['arrival','sanctuary','workshop','foundry']else'bedrock'
            components.append(part(f'rock_wall_{ix}_{iz}_{side}','section_mesh',material,role='continuous_cave_boundary',sections=[[0,23,0,65,34],[0,-height*.5,0,65,36],[0,-height,0,65,32]],sides=4,profiles=[square]*3,hem=[[0]*4,[0]*4,[-5,4,8,-2]],parent_socket=dict(origin=[edge[0],0,edge[1]],basis=[[co,0,-si],[0,1,0],[si,0,co]]),variation=(ix-iz)%4-2))
    # A recognizable radial sanctuary floor and an actual raised casting ring.
    for i in range(16):
        components.append(part('bell_inlay_'+str(i),'voussoir','stone',role='radial_paving',center=[0,0,0],inner=190,outer=340,start_angle=i*math.tau/16+.008,end_angle=(i+1)*math.tau/16-.008,depth=5,socket=dict(origin=[-1720,-3,2480],basis=[[1,0,0],[0,0,1],[0,-1,0]]),variation=i%3-1))
    components.append(part('casting_dais','loft','stone',role='supported_dais',sections=[[1450,0,-2350,350,305],[1450,-12,-2350,350,305],[1450,-24,-2350,330,288]],sides=16))
    for i in range(3):components.append(part('casting_step_'+str(i),'box','stone',center=[1450,-3-i*6,-1975-i*35],size=[190,6+i*12,46]))
    # Two paired rails connect a purposeful ore face, tub and working court.
    for side in [-1,1]:
        for i,(start,end)in enumerate([([-2200,-4,-1050],[-1880,-4,-650]),([-1880,-4,-650],[-1750,-4,-100]),([-1750,-4,-100],[-1540,-4,460])]):
            start=list(start);end=list(end);start[0]+=side*30;end[0]+=side*30;start[1]+=elevation(start[0],start[2]);end[1]+=elevation(end[0],end[2]);components.append(part(f'rail_{side}_{i}','beam','metal',role='transport_rail',start=start,end=end,radii=[3,3],sides=4))
    # Authored structural landmarks: refuge pilasters, working timber frames and
    # broad foundry vaults. They have visible supports, not floating roof strips.
    for i,(x,z)in enumerate([(-2200,2310),(-2050,2820),(-1340,2750),(-1160,2210),(-1690,1850)]):
        components.append(part('refuge_pilaster_'+str(i),'section_mesh','stone',role='sanctuary_structure',sections=[[x,0,z,28,32],[x,-185,z,22,26],[x,-205,z,32,36]],sides=8))
    for j,z in enumerate([815,1390]):
        for i,x in enumerate([-2030,-1490,-940]):
            components.append(part(f'timber_post_{j}_{i}','beam','wood',role='working_frame',start=[x,0,z],end=[x,-238,z],radii=[12,10],sides=4))
            components.append(part(f'timber_brace_{j}_{i}','beam','wood',start=[x,-170,z],end=[x+74,-238,z],radii=[5,5],sides=4))
        components.append(part('timber_crossbeam_'+str(j),'beam','wood',start=[-2080,-238,z],end=[-890,-238,z],radii=[12,12],sides=4))
    for j,(x,z)in enumerate([(1110,-2870),(1640,-3000),(2150,-2830)]):
        vault=arch(480,344,62,11,True);inner=480*.35;outer=240;spring=344-outer;pw=outer-inner
        for p in vault['components']:
            if p['name'].startswith('pier_'):
                side=int(p['name'].split('_')[1]);p['center'][0]=side*(inner+pw/2);p['size'][0]=pw-2
            if p['name'].startswith('impost_'):
                side=int(p['name'].split('_')[1]);p['center'][0]=side*(inner+pw/2);p['size'][0]=pw+10
            p['name']='foundry_vault_'+str(j)+'_'+p['name'];p['asset_transform']=dict(scale=[1,1,1],offset=[x,0,z]);components.append(p)
    components.append(part('cooling_return','sweep','metal',role='foundry_water_infrastructure',sections=[[820,-62,-2440,12,12],[780,-62,-2030,12,12],[690,-62,-1600,12,12],[590,-62,-1190,12,12],[490,-62,-1060,10,10]],normal=[0,1,0],sides=8))
    for i,(x,z)in enumerate([(810,-2280),(740,-1800),(640,-1420),(535,-1120)]):components.append(part('pipe_cradle_'+str(i),'stone_block','stone',center=[x,-26,z],size=[39,52,41],bevel=3))
    for i in range(6):components.append(part('descent_step_'+str(i),'box','stone',center=[-2860,-4*(6-i),2830-i*49],size=[180,8*(6-i),56]))
    components.append(part('descent_landing','box','stone',center=[-2860,-24,2860],size=[310,48,180]))
    r=provenance(asset('Complete original Cinderworks terrain and cave shell',components),'Connected original bedrock, regional floors, recessed water, quarry slope, radial bell court, casting dais and ore rails. Offline geometry only.');r['materials']['bedrock']=dict(hsl=10267,render_type=1);r['materials']['floor_stone']={**r['materials']['stone'],'render_type':1};r['materials']['water']=dict(hsl=32284,render_type=1,alpha=24,priority=1);r['materials']['wood'].update(hsl=6934,shadow=6924,end=6940,edge=6938);return r,cells,walk

def placements():
    p=[]
    def add(asset,x,z,y=0,q=0,note='',action=None):p.append(dict(asset=asset,position=[x,y,z],quarter_turns=q,action=action,placement_rationale=note))
    add('arrival-bell',-1720,2480,note='Sanctuary focal point within radial paving.')
    for x,z,q in [(-2310,2460,1),(-1580,1700,0),(-730,1080,1),(830,-1430,0)]:add('worker-threshold',x,z,q=q,note='Supported threshold with human-scale clearance marks a readable transition.')
    add('worker-threshold',-2860,2860,y=-48,note='A solid landing and stair flight make the arrival mouth recognizable.')
    for x,z in [(-2030,2800),(-1260,2500),(-1910,1850),(-1000,1370),(-590,1110),(50,1520),(-1480,100),(-1140,-590),(780,-1190),(1030,-1730)]:add('service-lantern',x,z,note='Lanterns mark thresholds and safe service circulation.')
    for x in [-1900,-1630,-1360]:add('annealing-retort',x,470,q=2,note='Annealing wall follows the ore delivery route.')
    for x,z,q in [(-1840,1040,0),(-1290,790,1),(-1800,1390,0)]:add('forge-bench',x,z,q=q,note='Working benches leave an open circulation spine.')
    for x,z,q in [(-1990,-800,0),(-1580,280,1),(1960,-2640,1)]:add('ore-cart',x,z,y=elevation(x,z)-2,q=q,note='Loaded tubs sit on or beside the material delivery route.')
    for x,z,q in [(220,1010,0),(395,-720,0)]:add('service-sluice',x,z,q=q,note='Raised gate controls a modeled crossing and watercourse.')
    for x,z,q in [(1090,1490,1),(-110,800,3),(740,-1030,1)]:add('eccentric-runoff-pump',x,z,q=q,note='Pumps are positioned on accessible reservoir/runoff banks.')
    for x,z,q in [(-2130,850,1),(-1320,1600,0),(1110,-1540,0),(2140,-2860,0)]:add('wall-bay',x,z,q=q,note='Bonded service walls support workshop/foundry infrastructure.')
    for x,z,q in [(-820,-760,1),(670,-1320,0),(-2420,-1100,0)]:add('broken-wall-bay',x,z,q=q,note='Damage clusters at quarry and slag transitions, not the sanctuary.')
    for x,z in [(-2480,-650),(-2520,-980),(-2200,-1290),(-1620,-1270),(-1000,-1050),(-630,-1210),(580,-1000)]:add('quarry-outcrop',x,z,y=elevation(x,z),q=(x+z)//128%4,note='Authored outcrops terminate ore faces or cooled slag margins.')
    for x,z in [(1110,-2870),(1640,-2980),(2140,-2700)]:add('annealing-retort',x,z,note='Foundry retorts frame the final casting area with distinct service bays.')
    for x,z in [(1880,-2370),(1060,-2600)]:add('casting-crucible',x,z,note='Cradled vessels and return route explain the final foundry.')
    add('crown-of-sluices',1450,-2350,y=-24,note='Original Cinderworks creature on a solid supported casting dais.',action='idle')
    add('kilnback-rummager',-290,-760,note='A smaller original scavenger marks the slag transition.',action='idle')
    for x,z,q in [(-2190,2670,1),(-2020,2920,2),(-1280,2510,3),(-1320,2060,3)]:add('rest-bench',x,z,q=q,note='Resting furniture addresses the bell court and its safe gathering space.')
    for x,z,q in [(-1980,1510,0),(-1060,550,1)]:add('stock-rack',x,z,q=q,note='Bar stock is stored beside the working benches and delivery route.')
    for x,z in [(-2160,2290),(-2100,2260),(-1050,1280),(1220,1600),(-1640,-980),(2230,-2420)]:add('supply-barrel',x,z,y=elevation(x,z),note='Supply barrels serve refuge, workshop, pumps or material delivery.')
    for x,z in [(-1880,2360),(-1460,1050),(55,1360)]:add('worker-figure',x,z,y=-5 if x==-1880 else 0,note='Wholly original clothed worker scale figure; no authentic NPC mesh is used.',action='idle')
    return p

def circulation(cells,walk):
    def nearest(point):return min(walk,key=lambda p:(p[0]*128+64-point[0])**2+(p[1]*128+64-point[1])**2)
    anchors={name:nearest(a['anchor'])for name,a in AREAS.items()};start=anchors['arrival'];seen={start};queue=deque([start])
    while queue:
        x,z=queue.popleft()
        for p in[(x-1,z),(x+1,z),(x,z-1),(x,z+1)]:
            if p in walk and p not in seen:seen.add(p);queue.append(p)
    return dict(cell_size_native=128,walkable_cells=len(walk),reachable_cells=len(seen),anchors={k:list(v)for k,v in anchors.items()},all_areas_floor_connected=all(p in seen for p in anchors.values()),routes=ROUTES,limitations='Offline floor connectivity only. Does not apply live clipping, exact scenery collision, NPC occupancy or gameplay pathfinding.')

def run(a):
    if a.out.exists()and any(a.out.iterdir()):raise ValueError('Choose a fresh floor study output; preserve previous versions')
    a.out.mkdir(parents=True,exist_ok=True);assets={k:Path(v).resolve()for k,v in load(a.assets).items()};recipe,cells,walk=ground_recipe();(a.out/'ground-recipe.json').write_text(json.dumps(recipe,indent=2));nav=circulation(cells,walk);(a.out/'circulation.json').write_text(json.dumps(nav,indent=2))
    if not nav['all_areas_floor_connected']:raise ValueError('An intended area is disconnected in the original floor plan')
    # Native model section lengths are u16 even below the overall vertex limit.
    # Split authored components into real cache-compatible meshes; every chunk
    # still goes through Blender save/reopen and the native model reader.
    ground_placements=[]
    for index,start in enumerate(range(0,len(recipe['components']),180)):
        key='ground-'+str(index).zfill(2);chunk=copy.deepcopy(recipe);chunk['name']+=' / '+key;chunk['components']=chunk['components'][start:start+180];path=a.out/(key+'-recipe.json');path.write_text(json.dumps(chunk,indent=2));root=a.out/key;command=[sys.executable,str(TOOLS/'author_workflow.py'),'--recipe',str(path),'--out',str(root),'--native',str(a.native),'--cache',str(a.cache),'--blender',str(a.blender),'--bounds',str(a.bounds),'--identities',str(a.identities),'--asset-key','mastery-floor:'+key,'--skip-motion'];subprocess.run(command,check=True);assets[key]=root;ground_placements.append(dict(asset=key,position=[0,0,0],quarter_turns=0,action=None,placement_rationale='Real Blender/native chunk of the continuous original bedrock and terrain.'))
    placed=ground_placements+placements();total=sum(len(load(assets[p['asset']]/'geometry.json')['vertices'])for p in placed)
    if total>65535:raise ValueError(f'Native combined vertex limit exceeded: {total}; revise actual geometry rather than using an impostor')
    for key in {p['asset']for p in placed}:
        owned=load(assets[key]/'recipe.json').get('provenance',{});g=load(assets[key]/'geometry.json')
        if owned.get('kind')!='original_tower_asset'or owned.get('source_mesh_or_animation_reuse')is not False:raise ValueError('A floor scene asset lacks original provenance: '+key)
        if any(t>=0 for t in(g.get('face_textures')or[])):raise ValueError('Source-safe floor review requires wholly owned face-colored assets: '+key)
    manifest=a.out/'scene.tsv';rows=['light\t64\t768']
    for p in placed:
        root=assets[p['asset']];rows.append('\t'.join(['owned',str(root/'asset.model'),*map(str,p['position']),str(p['quarter_turns']),str(root/p['action'])if p['action']else'-','0']))
    manifest.write_text('\n'.join(rows)+'\n');views={'overview':'look -7600 -10300 8800 -300 -40 0 2900 2200 1800','reverse':'look 8000 -10800 -8300 -300 -40 0 2900 2200 1800','top-down':'look -300 -12000 1 -300 0 0 3100 2200 2200'}
    for name,area in AREAS.items():
        x,z=area['focus'];views[name+'-a']=f'player {x} -95 {z} 2250 1300 710';views[name+'-b']=f'player {x+170} -100 {z-90} 2050 14000 660'
    for name,camera in views.items():
        path=a.out/(name+'.camera');path.write_text(camera);subprocess.run([str(a.native),str(a.cache),str(a.out/'views'/name),'scene',str(manifest),str(path)],check=True)
    proof=dict(title='Cinderworks — complete original offline Floor One visual study',areas=AREAS,connectors=CONNECTORS,routes=ROUTES,bridges=BRIDGES,placements=placed,combined_native_vertices=total,views=list(views),authentic_cache_meshes_or_textures_in_scene=False,scale='128 native units/tile; X,Y-down,Z; original models at scale 128',cache_modified=False,live_map_modified=False,gameplay_integration_tested=False,artistic_acceptance=False,known_limits=['Cutaway cave/architecture for offline inspection; no production roof/occlusion system.','Native face-colored water and lantern values; no dynamic liquid or point lights.','Exact future collision/elevation/navigation integration remains untested.']);(a.out/'floor-compatibility.json').write_text(json.dumps(proof,indent=2));print('COMPLETE_ORIGINAL_NATIVE_FLOOR_STUDY_RENDERED',total,'vertices',len(views),'views')

if __name__=='__main__':
    p=argparse.ArgumentParser()
    for name in ['assets','out','native','cache','blender','bounds','identities']:p.add_argument('--'+name,type=Path,required=True)
    run(p.parse_args())

