"""Original Cinderworks scenery inputs for an offline visual production study.

No reference asset or cache payload is read. These are new authored designs on
the sealed construction primitives, separate from examination reconstruction.
"""
import sys,json,math,argparse,copy
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'asset-workbench'))
from templates import part
from construction import asset,arch,masonry,plank,bench,anvil,barrel,pickaxe,hammer
from mastery_construction import CHEST,framed_threshold
from production_designs import provenance

def finish(name,c,brief):
    r=asset(name,c);r['materials']['wood'].update(hsl=6934,shadow=6921,end=6942,edge=6939);r['materials']['stone'].update(hsl=10279,shadow=10267,end=10287,edge=10283);return provenance(r,brief)
def placed_parts(recipe,prefix,origin=(0,0,0),quarter=0):
    a=quarter*math.pi/2;co,si=round(math.cos(a)),round(math.sin(a));basis=[[co,0,-si],[0,1,0],[si,0,co]];result=[]
    for p in recipe['components']:
        p=copy.deepcopy(p);p['name']=prefix+p['name'];old=p.get('asset_transform')
        if old:raise ValueError('Scenery assembly expects native-unit original parts')
        # A nested authored socket is composed explicitly, preserving winding.
        old=p.get('parent_socket',{});ob=old.get('basis',[[1,0,0],[0,1,0],[0,0,1]]);oo=old.get('origin',[0,0,0]);p['parent_socket']=dict(origin=[origin[j]+sum(oo[k]*basis[k][j]for k in range(3))for j in range(3)],basis=[[sum(row[k]*basis[k][j]for k in range(3))for j in range(3)]for row in ob]);result.append(p)
    return result

def wall_bay(height=176,damaged=False):
    r=masonry(256,height,39,4,seed=3);c=r['components']
    for side in [-1,1]:c.append(part('pier_'+str(side),'section_mesh','stone',sections=[[side*117,0,0,23,30],[side*117,-height*.7,0,18,25],[side*117,-height-10,0,23,29]],sides=8,profiles=[CHEST]*3))
    c.append(part('tie_beam','box','metal',center=[0,-height*.73,-24],size=[222,6,5]))
    if damaged:
        c[:]=[p for p in c if p['name']not in ['course_3_block_0','coping']]
        for p in c:
            if p['type']=='stone_block':p.update(wear=.19,seed=7)
    return finish('Broken wall bay'if damaged else'Bonded industrial wall bay',c,'Original divided masonry, tapering piers and an iron tie. Broken and intact bays share a coherent profile.')

def bell_shrine():
    c=[part('base','section_mesh','stone',sections=[[0,0,0,99,79],[0,-16,0,99,79],[0,-26,0,83,65]],sides=8,profiles=[CHEST]*3)]
    for side in [-1,1]:
        c+= [part('bell_post_'+str(side),'section_mesh','stone',sections=[[side*62,-24,0,17,22],[side*62,-151,0,13,18],[side*62,-165,0,18,23]],sides=8),part('bell_brace_'+str(side),'beam','metal',start=[side*63,-130,0],end=[side*36,-168,0],radii=[4,4],sides=4)]
    c += [plank('crossbeam',[0,-173,0],[166,17,27],axis=0),part('bell','shell','bronze',sections=[[0,-151,0,12,12],[0,-120,0,22,22],[0,-105,0,35,35]],thickness=4,sides=10),part('clapper','beam','metal',start=[0,-148,0],end=[0,-100,0],radii=[3,5],sides=6)]
    for side in [-1,1]:c.append(part('offering_cup_'+str(side),'shell','ceramic',sections=[[side*40,-30,-43,12,12],[side*40,-44,-43,16,16]],thickness=3,sides=8))
    return finish('Arrival bell and votive stand',c,'An original practical sanctuary landmark: a warning/arrival bell, worn base and small offering cups. Structural bracing and a real hollow bell replace a decorative floating symbol.')

def retort():
    c=placed_parts(masonry(208,86,120,3,2),'base_')
    for side in [-1,1]:
        c.append(part('retort_'+str(side),'section_mesh','stone',sections=[[side*51,-80,0,43,59],[side*51,-151,0,43,56],[side*51,-173,0,28,40]],sides=8,profiles=[CHEST]*3))
        c.append(part('door_frame_'+str(side),'shell','metal',sections=[[0,0,0,31,29],[0,-6,0,31,29]],thickness=6,sides=8,socket=dict(origin=[side*51,-118,-60],basis=[[1,0,0],[0,0,1],[0,-1,0]])))
        c.append(part('hot_throat_'+str(side),'box','ceramic',center=[side*51,-119,-53],size=[46,43,5]))
        for i in range(4):c.append(part(f'door_grate_{side}_{i}','beam','metal',start=[side*51+(i-1.5)*10,-143,-64],end=[side*51+(i-1.5)*10,-94,-64],radii=[2,2],sides=4))
        c.append(part('flue_'+str(side),'sweep','metal',sections=[[side*51,-160,16,16,16],[side*51,-242,16,16,16],[side*73,-270,23,13,13]],normal=[0,0,1],sides=8))
    c+=placed_parts(arch(202,163,18,9,False),'hood_',origin=[0,-90,28])
    return finish('Twin-chamber annealing retort',c,'A wholly original masonry retort with modeled door collars, grates, paired flues and a supported arched hood. Bright native ceramic colors represent a hot throat; no emissive shader or borrowed furnace mesh is used.')

def forge_bench():
    r=bench(184,77,73,False);c=r['components'];c+=placed_parts(anvil(66,38,25),'anvil_',origin=[-42,-81,0]);c += [part('vise_fixed','stone_block','metal',center=[49,-91,-29],size=[20,23,12],bevel=2),part('vise_jaw','stone_block','metal',center=[49,-91,-47],size=[20,23,8],bevel=2),part('vise_screw','beam','metal',start=[49,-89,-35],end=[49,-89,-62],radii=[3,3],sides=6),part('vise_handle','beam','wood',start=[37,-89,-59],end=[61,-89,-59],radii=[3,3],sides=6)]
    for i in range(3):c.append(plank('stock_'+str(i),[34,-83-i*4,14],[55,4,7],axis=0,variation=i-1))
    return finish('Smithing bench with vise and stock',c,'A braced working bench with a scaled original anvil, functioning-looking vise construction, stock and open space for a worker. Props serve a visible purpose.')

def ore_cart():
    c=[plank('bed',[0,-24,0],[118,9,90],axis=0)]
    for side in [-1,1]:
        for row in range(3):c.append(plank(f'side_{side}_{row}',[0,-35-row*14,side*44],[119,13,7],axis=0,variation=row-1))
        c.append(plank('end_'+str(side),[side*57,-49,0],[7,50,88]))
        for z in [-30,30]:
            c.append(part(f'wheel_{side}_{z}','annulus','metal',center=[0,0,0],radii=[19,19],rim_width=5,depth=6,sides=8,socket=dict(origin=[side*66,-18,z],basis=[[0,0,1],[0,1,0],[-1,0,0]])))
            c.append(part(f'wheel_spoke_{side}_{z}','beam','metal',start=[side*66,-34,z],end=[side*66,-2,z],radii=[2,2],sides=4))
    for i,(x,z)in enumerate([(-30,-21),(7,-18),(32,12),(-20,23),(6,13)]):c.append(part('ore_'+str(i),'section_mesh','stone',sections=[[x,-57,z,21,18],[x+4,-76-i%3*5,z,17,14],[x+1,-86-i%3*5,z,7,8]],sides=6,variation=i%3-1))
    c.append(part('tow_loop','annulus','metal',center=[0,-30,0],radii=[8,8],rim_width=2,depth=4,sides=8,socket=dict(origin=[-70,0,0],basis=[[0,0,1],[0,1,0],[-1,0,0]])))
    return finish('Ore tub on flanged wheels',c,'Original reinforced boards, visible wheels and clustered authored ore show material delivery from quarry to foundry. No cache cart, rock or texture is reused.')

def sluice_gate():
    r=framed_threshold(252,264,60);c=r['components']
    for i in range(6):c.append(plank('gate_slat_'+str(i),[(i-2.5)*23,-249,2],[21,62,10],variation=i%3-1))
    for y in [-270,-230]:c.append(part('gate_strap_'+str(y),'box','metal',center=[0,y,-7],size=[148,6,6]))
    c += [part('lift_screw','beam','metal',start=[0,-315,3],end=[0,-236,3],radii=[5,5],sides=6),part('lift_wheel','gear','bronze',center=[157,-113,-3],radius=24,root_radius=20,teeth=8,depth=6,bore=9),part('drive_chain','sweep','leather',sections=[[157,-113,-3,2,2],[157,-283,-3,2,2],[0,-297,3,2,2]],normal=[0,0,1],sides=4)]
    return finish('Raised service sluice',c,'A raised timber gate, iron straps, screw and operating wheel fit a buttressed stone channel opening. It explains the direction and control of the waterworks.')

def lamp():
    c=[part('foot','stone_block','stone',center=[0,-6,0],size=[35,12,35],bevel=3),part('post','beam','metal',start=[0,-11,0],end=[0,-155,0],radii=[4,3],sides=6),part('crook','sweep','metal',sections=[[0,-145,0,3,3],[0,-174,0,3,3],[25,-179,0,3,3],[34,-163,0,3,3]],normal=[0,0,1],sides=5),part('lantern_top','loft','metal',sections=[[34,-157,0,15,15],[34,-168,0,4,4]],sides=6),part('lantern_glow','loft','ceramic',sections=[[34,-126,0,10,10],[34,-156,0,10,10]],sides=6),part('lantern_base','loft','metal',sections=[[34,-121,0,13,13],[34,-127,0,13,13]],sides=6)]
    for x in [-1,1]:
        for z in [-1,1]:c.append(part(f'lantern_bar_{x}_{z}','beam','metal',start=[34+x*8,-155,z*8],end=[34+x*8,-126,z*8],radii=[1,1],sides=4))
    return finish('Crooked service lantern',c,'A supported iron post and framed lantern mark safe circulation. Bright native face colors are intentional; no unsupported dynamic point lighting is claimed.')

def crucible():
    c=[part('casting_base','section_mesh','stone',sections=[[0,0,0,111,95],[0,-22,0,111,95],[0,-35,0,94,79]],sides=8,profiles=[CHEST]*3),part('vessel','shell','metal',sections=[[0,-36,0,53,53],[0,-82,0,68,68],[0,-121,0,77,77]],thickness=8,sides=12),part('hot_charge','loft','ceramic',sections=[[0,-100,0,65,65],[0,-103,0,65,65]],sides=12)]
    for side in [-1,1]:c.append(part('trunnion_'+str(side),'beam','bronze',start=[side*64,-79,0],end=[side*97,-79,0],radii=[12,12],sides=8));c.append(part('cradle_'+str(side),'section_mesh','stone',sections=[[side*94,-23,0,20,47],[side*94,-92,0,16,35]],sides=8,profiles=[CHEST]*2))
    return finish('Cradled foundry crucible',c,'An open original casting vessel, hot charge, pivot trunnions and stone cradle form a memorable foundry focal point with credible structural support.')

def rubble():
    c=[]
    for i,(x,z,w,h)in enumerate([(-38,0,35,41),(5,17,42,55),(37,-16,31,34),(-13,-29,29,31)]):c.append(part('fracture_'+str(i),'section_mesh','stone',sections=[[x,4,z,w,w*.78],[x+7,-h*.58,z-3,w*.86,w*.7],[x-3,-h,z+5,w*.32,w*.31]],sides=6,variation=i-2))
    return finish('Fractured quarry outcrop',c,'Four deliberate interlocking rock forms provide a broken geological edge and ore-face context; no random prop scatter or source rock geometry is used.')

def designs():
    result={'wall-bay':wall_bay(),'broken-wall-bay':wall_bay(154,True),'arrival-bell':bell_shrine(),'annealing-retort':retort(),'forge-bench':forge_bench(),'ore-cart':ore_cart(),'service-sluice':sluice_gate(),'service-lantern':lamp(),'casting-crucible':crucible(),'quarry-outcrop':rubble()}
    for key in ['annealing-retort','service-lantern','casting-crucible']:result[key]['materials']['ceramic'].update(hsl=5062,shadow=5050,end=5076,edge=5068)
    return result

def refinement_designs():
    rest=bench(148,73,53,True);stock=[plank('shelf_'+str(i),[0,-22-i*50,0],[155,8,56],axis=0)for i in range(3)]
    for side in [-1,1]:stock.append(plank('upright_'+str(side),[side*70,-81,0],[10,162,48]))
    for row in range(2):
        for n in range(4):stock.append(part(f'bar_stock_{row}_{n}','beam','metal',start=[-58,-29-row*50,-17+n*10],end=[56,-29-row*50,-17+n*10],radii=[3,3],sides=4))
    b=barrel(68,88,68,10);result={'rest-bench':finish('Bellkeeper resting bench',rest['components'],'A joined backrest and visible frame provide a deliberate place to rest beside the sanctuary.'),'stock-rack':finish('Bar-stock service rack',stock,'A supported rack of metal stock explains the working court rather than adding unrelated clutter.'),'supply-barrel':finish('Coopered workshop supply barrel',b['components'],'An original stave-and-hoop supply barrel, using owned native face colors without reference grain textures.'),'service-sluice':sluice_gate(),'worker-threshold':framed_threshold(252,264,60),'annealing-retort':retort()}
    for p in result['annealing-retort']['components']:
        if p['name'].startswith('retort_'):
            x=p['sections'][0][0];p['material_zones']=[dict(center=[x,-170,0],radius=50,hsl=10265),dict(center=[x,-116,-58],radius=47,hsl=6460)]
    result['annealing-retort']['materials']['ceramic'].update(hsl=5062,shadow=5050,end=5076,edge=5068)
    return result

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('out',type=Path);p.add_argument('--refinement',action='store_true');a=p.parse_args()
    if a.out.exists()and any(a.out.iterdir()):raise ValueError('Choose a fresh original recipe output')
    a.out.mkdir(parents=True,exist_ok=True)
    for key,r in(refinement_designs()if a.refinement else designs()).items():(a.out/(key+'.json')).write_text(json.dumps(r,indent=2),encoding='utf-8')
    print('ORIGINAL_FLOOR_SCENERY_INPUTS_READY')
