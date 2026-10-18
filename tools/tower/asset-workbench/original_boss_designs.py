"""Six separate original input designs using the sealed reusable libraries.

No reference geometry, animation, region or section-observation file is read.
This file and its wholly original recipes are safe to publish with the assets.
"""
import sys,json,copy,math
from pathlib import Path
T=Path(__file__).resolve().parent;sys.path.insert(0,str(T))
from mastery_creatures import biped,winged_biped,segmented_crawler
from mastery_construction import armor_lames,CHEST,FACE
from creature_families import mammal
from production_designs import kilnback,provenance
from templates import part

def rename_action(r,old,new):r['actions']={new if k==old else k:v for k,v in r['actions'].items()}
def completion(r,special):
    for name,s in r['actions'].items():
        if name in ['idle','attack','hit']:s['grounded']=True
    if 'walk_entry'not in r['actions']:r['actions']['walk_entry']=dict(duration_ticks=8,cadence_ticks=2,loop=False,keys=[],bridge_controls=dict(source='idle',destination='walk'))
    r['rig_validation']=True;r['geometry_validation']=dict(remove_zero_area_faces=True,preserve_vertex_order=True);r['design_rationale']=special;return r

def hearthbearer():
    h=300;r=biped(h,1.12,False,'none','Basalt Hearthbearer');r['materials']['organic'].update(hsl=10267,shadow=10257,end=10274,edge=10270);r['materials']['ceramic'].update(hsl=4667,shadow=4656,end=4674,edge=4671)
    c=r['components'];c += [part('furnace_back','section_mesh','stone',0,sections=[[0,-h*.5,31,42,30],[0,-h*.73,31,48,34],[0,-h*.83,28,28,23]],sides=8,profiles=[CHEST]*3),part('face_mask','section_mesh','ceramic',1,sections=[[0,-h*.865,-32,11,5],[0,-h*.925,-33,17,6],[0,-h*.985,-27,9,5]],sides=8,profiles=[FACE]*3),part('mouth_grate','stone_block','metal',1,center=[0,-h*.902,-40],size=[18,5,3],bevel=1),part('chest_fire','stone_block','ceramic',0,center=[0,-h*.66,-28],size=[23,24,6],bevel=2)]
    for side,group in [(-1,14),(1,15)]:
        c.append(part('exhaust_'+str(side),'sweep','metal',group,sections=[[side*30,-h*.78,44,12,12],[side*38,-h*.96,45,12,12],[side*55,-h*1.05,39,10,10]],normal=[0,0,1],sides=6));r['rig'].append(dict(name='vent_'+str(side),parent='body',origin=[side*30,-h*.78,44],groups=[group],limits=[[-40,40]]*3))
        for action,spec in r['actions'].items():
            for key in spec['keys']:
                t=key['tick']/spec['duration_ticks'];pulse=max(0,1-abs(t-.45)/.35)if action=='sweep_attack'else math.sin(t*math.tau)*.2;key['channels']['vent_'+str(side)]=[round(pulse*8),0,round(side*pulse*14)]
    rename_action(r,'sweep_attack','furnace_pulse');return completion(r,'A basalt porter carries a repaired ceramic furnace as its own anatomy. Asymmetric exhausts, a narrow mask and open hands distinguish it from the reference giant. Mass, value hierarchy and deliberate anticipation are the lessons; no reference body/gear/frame is reused.')

def warden():
    r=mammal(155,142,235,'bovine');r['name']='Counterweight Warden';r['materials']['organic'].update(hsl=10273,shadow=10259,end=10279,edge=10276);r['materials']['keratin'].update(hsl=25,shadow=16,end=33,edge=35);r['locomotion'].update(support_surface_iterations=2,support_polygon_mode='sole_footprints',stride=32,lift=10,crouch=4)
    for p in r['components']:
        if p['name'].startswith('ear_'):p['material']='metal'
    for side in [-1,1]:
        r['components'] += [part('counterweight_'+str(side),'stone_block','stone',0,center=[side*62,-92,10],size=[30,56,79],bevel=6,wear=.1,seed=side+5),part('flank_cog_'+str(side),'gear','bronze',0,center=[0,0,0],radius=25,root_radius=20,teeth=8,depth=8,bore=8,socket=dict(origin=[side*83,-90,10],basis=[[0,0,1],[0,1,0],[-1,0,0]]))]
    r['components'].append(part('load_yoke','sweep','metal',0,sections=[[-75,-88,10,7,7],[-48,-123,10,7,7],[48,-123,10,7,7],[75,-88,10,7,7]],normal=[0,0,1],sides=6))
    special=copy.deepcopy(r['actions']['attack']);special['duration_ticks']=60;special['events']=dict(anticipation=[0,22],execution=[22,34],recovery=[34,60]);special['grounded']=True
    for key in special['keys']:
        t=key['tick']/42;key['tick']=round(t*60);wind=max(0,1-abs(t-.3)/.3);strike=max(0,1-abs(t-.55)/.14);key['channels']['body']=[round(-10*wind+15*strike),0,0];key['channels']['head']=[round(-15*wind+21*strike),0,0]
    r['actions']['counterweight_drop']=special;return completion(provenance(r,'Original stone-and-iron quadruped with geared side weights and an exposed load yoke.'),'Armor mass and forceful action are transferred to a four-legged industrial guardian. Its ox-like chassis, counterweights and head-driven drop differ from a two-armed armored boss; no source mesh, color-region recipe or motion is reused.')

def harrier():
    r=winged_biped(235,'Bellows Harrier');r['materials']['organic'].update(hsl=10267,shadow=10256,end=10276,edge=10270);r['materials']['leather'].update(hsl=6680,shadow=6669,end=6688,edge=6684)
    r['components'] += [part('filter_beak','sweep','bronze',1,sections=[[0,-217,-21,9,8],[0,-209,-42,7,6],[0,-201,-48,2,2]],normal=[1,0,0],sides=6)]
    for i in range(3):r['components'].append(part('crest_'+str(i),'profile','metal',1,profile=[[-7,-228-i*3],[0,-260-i*4],[7,-228-i*3]],depth=4,socket=dict(origin=[0,0,4+i*8])))
    rename_action(r,'sweep_attack','bellows_burst');return completion(r,'A filter-masked, copper-braced bellows creature learns appendage ownership and aggressive preparation from complex humanoids. Grounded legs, planar industrial membranes and a fan crest create a separate silhouette without demonic horns, spear or recognizable equipment layout.')

def matriarch():
    r=segmented_crawler(375,118,'Siltcoil Matriarch');r['materials']['organic'].update(hsl=33725,shadow=33712,end=33732,edge=33728);r['materials']['ceramic'].update(hsl=23069,shadow=23055,end=23078,edge=23073)
    for i in range(5):r['components'].append(part('sieve_bar_'+str(i),'beam','bronze',11,start=[(i-2)*9,-24,-174],end=[(i-2)*9,-9,-197],radii=[3,2],sides=5))
    rename_action(r,'burrow_prepare','sieve_breach');return completion(r,'An axial, limbless silt-filtering matriarch has overlapping ceramic segments and an opening sieve jaw. Serial motion and an unusual body plan are the lessons; it has no copied wings, encounter poison animation or reference anatomy.')

def crown():
    r=kilnback();factor=2.3;r['name']='Crown-of-Sluices'
    for p in r['components']:p['asset_transform']=dict(scale=[factor]*3,offset=[0,0,0])
    for j in r['rig']:j['origin']=[round(v*factor)for v in j['origin']]
    seen=set()
    for gait in [r['locomotion'],r['actions']['walk']['gait']]:
        if id(gait)in seen:continue
        seen.add(id(gait))
        for field in ['stride','lift','crouch']:gait[field]*=factor
        for limb in gait['limbs']:
            for field in ['hip','knee','tip']:limb[field]=[round(v*factor)for v in limb[field]]
        gait['support_polygon_mode']='sole_footprints'
    for action in r['actions'].values():
        for key in action['keys']:key['translation']=[round(v*factor)for v in key.get('translation',[0,0,0])]
    r['rig'][0]['groups'].append(23);r['rig'].append(dict(name='crown',parent='body',origin=[0,-142,32],groups=[23],limits=[[-40,40]]*3))
    for i in range(5):
        x=(i-2)*24;r['components'].append(part('crown_pipe_'+str(i),'sweep','bronze',23,sections=[[x,-144,32,8,8],[x,-198-abs(i-2)*8,32,8,8],[x*1.2,-217-abs(i-2)*8,19,7,7]],normal=[0,0,1],sides=6))
    special=copy.deepcopy(r['actions']['attack']);special['duration_ticks']=60
    for key in special['keys']:
        t=key['tick']/42;key['tick']=round(t*60);pulse=max(0,1-abs(t-.4)/.35);key['channels']['crown']=[round(pulse*18),round(math.sin(t*math.tau)*8),0]
    items=list(r['actions'].items());r['actions']={k:v for k,v in items if k!='walk_entry'};r['actions']['sluice_roar']=special;r['actions']['walk_entry']=dict(items)['walk_entry'];return completion(r,'The original Kilnback lineage becomes a large six-legged sluice organism with an independently pivoting five-pipe crown. A staged presentation and shell hierarchy are the lessons; no vampire body, eight-legged silhouette, unique equipment or source phase animation is copied.')

def surveyor():
    r=biped(212,.78,False,'spear','Glasswake Surveyor');r['materials']['organic'].update(hsl=10286,shadow=10274,end=10294,edge=10290);r['materials']['leather'].update(hsl=33727,shadow=33713,end=33735,edge=33731);r['materials']['metal'].update(hsl=10286,shadow=10271,end=10296,edge=10291)
    p=next(p for p in r['components']if p['name']=='weapon_head');x=-212*.16*.78*1.25;y=-212*.4;z=-212*.065;p.update(type='section_mesh',sections=[[x,y-212*.42,z,10,4],[x,y-212*.48,z,11,4]],sides=8,profiles=[CHEST]*2)
    for i in [-1,1]:r['components'].append(part('tuning_fork_'+str(i),'beam','bronze',4,start=[x+i*8,y-212*.46,z],end=[x+i*8,y-212*.68,z],radii=[3,2],sides=5))
    r['attachments'][0]['components']+=['tuning_fork_-1','tuning_fork_1'];r['components']+=armor_lames('survey_mantle',[0,-167,0],49,42,33,0,3)
    r['components'].append(part('visor','stone_block','bronze',1,center=[0,-198,-19],size=[25,5,4],bevel=1));rename_action(r,'sweep_attack','sounding_sweep');return completion(r,'A slender pale ceramic surveyor carries a split acoustic measuring fork and a blue-grey mantle. Equipment integration and multiple readable actions are transferred into a separate non-Egyptian industrial identity; no reference spear/sword arrangement or cached animation is reused.')

def designs():return {'basalt-hearthbearer':hearthbearer(),'counterweight-warden':warden(),'bellows-harrier':harrier(),'siltcoil-matriarch':matriarch(),'crown-of-sluices':crown(),'glasswake-surveyor':surveyor()}
if __name__=='__main__':
    import argparse;parser=argparse.ArgumentParser();parser.add_argument('out',type=Path);out=parser.parse_args().out;out.mkdir(parents=True,exist_ok=True)
    for key,r in designs().items():(out/(key+'.json')).write_text(json.dumps(r,indent=2),encoding='utf-8')
    print('SIX_ORIGINAL_BOSS_INPUT_DESIGNS_AUTHORED_WITHOUT_REFERENCE_PAYLOAD_READS')

