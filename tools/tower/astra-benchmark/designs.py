"""Four controlled benchmark designs. No cache geometry or animation is read.

A is a hand-authored reconstruction from visual observation and coarse dimensions.
B/C/D are original designs. Existing geometry and classic action tooling is reused.
"""
import sys, json, math, copy
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'asset-workbench'))
from templates import part
from construction import asset, anvil, plank
from creature_families import mammal
from production_designs import provenance

SQUARE=[[-1,-1],[1,-1],[1,1],[-1,1]]
def finish(r,brief):
    r=provenance(r,brief)
    r['materials'].update(iron=dict(hsl=28,render_type=0),polished=dict(hsl=39,render_type=0),rust=dict(hsl=5658,render_type=1),timber=dict(hsl=6430,render_type=1),endgrain=dict(hsl=6425,render_type=1),ash=dict(hsl=10278,render_type=1))
    return r

def reference(refined=False):
    if not refined:
        r=anvil(112,64,34)
        for c in r['components']:c['material']='iron'
    else:
        c=[part('broad_foot','loft','iron',sections=[[-4,0,0,32,24],[-4,-18,0,32,24],[-4,-25,0,22,16],[-4,-40,0,14,12]],outline=SQUARE),
           part('striking_block','loft','iron',sections=[[-4,-40,0,32,16],[-4,-59,0,32,16],[-4,-64,0,30,14]],outline=SQUARE),
           part('heel_neck','box','iron',center=[-40,-47,0],size=[10,20,23]),
           part('heel','loft','iron',sections=[[-50,-40,0,6,17],[-50,-56,0,6,17],[-48,-64,0,6,14]],outline=SQUARE),
           part('horn','sweep','iron',sections=[[26,-55,0,9,9],[40,-56,0,7,6],[56,-57,0,4,4]],normal=[0,1,0],sides=6)]
        r=asset('A — visually authored anvil reconstruction',c)
    r=finish(r,'Faithful reconstruction of object 2097, independently modeled from native images and coarse visible dimensions. No imported cache vertices, topology or hidden geometry.')
    r['provenance']['kind']='independently_authored_reference_reconstruction'
    r['materials']['iron']['hsl']=28 if refined else 33
    return r

def riveting(refined=False):
    c=[]
    # Low oak bolster, broad front chamfers and forged clamps carry a compact
    # double die. The slot is modeled as a real opening between two cheeks.
    for i in range(3):c.append(part('bolster_'+str(i),'loft','timber',sections=[[0,0,(i-1)*26,44,12],[0,-54,(i-1)*26,41,12]],outline=SQUARE))
    c += [part('iron_shoe','stone_block','iron',center=[0,-6,0],size=[98,12,86],bevel=3),part('seat_plate','stone_block','iron',center=[0,-57,0],size=[93,9,77],bevel=2),part('waisted_die','loft','iron',sections=[[0,-62,0,31,27],[0,-78,0,22,19],[0,-94,0,32,25]],outline=SQUARE)]
    for side in [-1,1]:
        c.append(part('die_cheek_'+str(side),'stone_block','polished',center=[side*20,-100,0],size=[35,14,56],bevel=1))
        c.append(part('clamp_'+str(side),'sweep','iron',sections=[[side*36,-66,-24,5,5],[side*49,-66,-24,5,5],[side*49,-32,-24,5,5]],normal=[0,0,1],sides=4))
        c.append(part('clamp_pin_'+str(side),'beam','polished',start=[side*49,-36,-30],end=[side*49,-36,-33],radii=[3,3],sides=4))
    c += [part('bending_horn','sweep','iron',sections=[[38,-95,0,13,11],[58,-96,0,8,6],[72,-98,0,2,2]],normal=[0,1,0],sides=6),part('punch_holder','box','iron',center=[-44,-72,26],size=[16,14,22])]
    for i in range(3):c.append(part('punch_'+str(i),'beam','polished',start=[-49+i*5,-77,26],end=[-49+i*5,-91-i*3,26],radii=[2,2],sides=4))
    if refined:
        c += [part('rivet_blank','beam','rust',start=[18,-108,-9],end=[18,-116,-9],radii=[3,3],sides=6),part('rivet_head','beam','polished',start=[18,-116,-9],end=[18,-119,-9],radii=[5,5],sides=6),part('hammer_handle','beam','timber',start=[-26,-110,-16],end=[15,-110,15],radii=[3,3],sides=5),part('hammer_head','stone_block','iron',center=[-26,-112,-16],size=[13,11,25],bevel=2)]
        # Less golden timber; larger uninterrupted iron shapes read at game scale.
        for p in c:
            if p['name'].startswith('bolster_'):p['surface_bands']=[dict(axis=1,min=-10,max=1,hsl=6418)]
    r=finish(asset('B — Cinderworks split-die riveting station',c),'A low oak bolster supports a split riveting die, bending horn and punch rack. A real central slot, two forged clamps and a restrained worn striking face establish its own silhouette and function.')
    if not refined:r['materials']['timber']['hsl']=7440
    return r

def press(refined=False):
    c=[part('plinth','stone_block','ash',center=[0,-8,0],size=[150,16,112],bevel=5),part('bed','stone_block','iron',center=[0,-22,0],size=[126,16,90],bevel=3)]
    for side in [-1,1]:
        c += [part('upright_'+str(side),'loft','iron',sections=[[side*53,-29,0,15,26],[side*53,-59,0,11,18],[side*53,-163,0,11,18],[side*53,-180,0,16,24]],outline=SQUARE),part('foot_'+str(side),'stone_block','iron',center=[side*53,-35,0],size=[36,18,67],bevel=3)]
    c += [part('crosshead','stone_block','iron',center=[0,-178,0],size=[142,30,55],bevel=5),part('ram','beam','polished',start=[0,-196,0],end=[0,-85,0],radii=[10,10],sides=8),part('ram_collar','loft','iron',sections=[[0,-162,0,17,17],[0,-143,0,17,17]],sides=8),part('upper_die','stone_block','iron',center=[0,-80,0],size=[43,16,43],bevel=2),part('lower_die','stone_block','polished',center=[0,-37,0],size=[49,14,47],bevel=2),part('operating_lever','beam','iron',start=[-58,-207,0],end=[64,-207,0],radii=[5,5],sides=6)]
    c.append(part('weight','loft','iron',sections=[[-62,-195,0,17,15],[-62,-217,0,17,15]],sides=8))
    c.append(part('grip','beam','timber',start=[58,-207,0],end=[78,-207,0],radii=[7,7],sides=6))
    if refined:
        # Return springs and a geared rack make the load path legible without
        # filling the central work opening with decorative machinery.
        for side in [-1,1]:
            c.append(part('return_rod_'+str(side),'beam','polished',start=[side*22,-155,0],end=[side*22,-87,0],radii=[3,3],sides=5))
            for i in range(5):c.append(part(f'spring_{side}_{i}','loft','iron',sections=[[side*22,-143+i*7,0,6,6],[side*22,-140+i*7,0,6,6]],sides=6))
        next(p for p in c if p['name']=='ram')['start']=[0,-212,0]
        c.append(part('screw_thread','sweep','polished',sections=[[12*math.cos(i*math.tau/8),-152+i,12*math.sin(i*math.tau/8),2,2] for i in range(41)],normal=[0,1,0],sides=4))
        c += [part('threaded_crosshead_boss','loft','iron',sections=[[0,-193,0,20,20],[0,-202,0,20,20]],sides=8),part('hot_billet','stone_block','rust',center=[0,-48,0],size=[36,8,20],bevel=1),part('makers_plate','stone_block','bronze',center=[-54,-91,-20],size=[14,21,3],bevel=1)]
    r=finish(asset('C — counterweighted screw press',c),'A continuous lever and spindle drive a visible helical screw through the crosshead. Paired spring rods guide the upper die. Broad feet carry the pressing load to a low stone foundation. The open work throat is the main negative space.')
    return r

def creature(refined=False):
    r=mammal(98,118,180,'bear');r['name']='D — Sootsnout ash burrower'
    r['components']=[p for p in r['components'] if p['name'].startswith('leg_')]
    c=r['components']
    c += [part('low_back','sweep','hide',0,sections=[[0,-48,-41,34,28],[0,-62,-20,46,35],[0,-54,30,42,31],[0,-39,65,25,23]],normal=[0,1,0],sides=8),part('shoulder_ridge','sweep','scute',0,sections=[[0,-67,-39,39,25],[0,-83,-17,34,20],[0,-72,7,30,17]],normal=[0,1,0],sides=6),part('neck','sweep','hide',1,sections=[[0,-58,-36,28,21],[0,-61,-63,25,21]],normal=[0,1,0],sides=6),part('shovel_skull','sweep','hide',1,sections=[[0,-69,-57,28,20],[0,-65,-79,33,20],[0,-50,-101,23,11]],normal=[0,1,0],sides=6),part('nasal_plate','sweep','scute',1,sections=[[0,-77,-71,23,5],[0,-63,-96,24,5],[0,-53,-109,18,4]],normal=[0,1,0],sides=4),part('lower_jaw','sweep','muzzle',15,sections=[[0,-43,-72,23,7],[0,-38,-100,22,6]],normal=[0,1,0],sides=5),part('tail','sweep','hide',14,sections=[[0,-39,57,15,13],[4,-26,84,10,9],[14,-16,105,2,2]],normal=[0,1,0],sides=6)]
    for p in c:
        if p['name'].startswith('leg_'):p['material']='claw' if 'claw' in p['name'] else 'hide'
    for side in [-1,1]:
        c += [part('ear_'+str(side),'loft','hide',1,sections=[[side*25,-76,-55,9,7],[side*29,-93,-52,5,3]],sides=5),part('eye_socket_'+str(side),'stone_block','socket',1,center=[side*28,-71,-79],size=[7,9,11],bevel=1),part('eye_'+str(side),'stone_block','eye',1,center=[side*32,-72,-81],size=[3,3,5],bevel=0)]
        for i in range(2):c.append(part(f'tooth_{side}_{i}','beam','claw',15,start=[side*(15+i*5),-42,-94+i*8],end=[side*(16+i*5),-49,-94+i*8],radii=[3,1],sides=4))
    # Native parent groups include a new articulated jaw, never imported frames.
    r['rig'][0]['groups'].append(15);r['rig'][1]['groups'].append(15)
    r['rig'].insert(2,dict(name='jaw',parent='head',origin=[0,-45,-67],groups=[15]))
    r['locomotion'].update(stride=36,lift=13,stance_fraction=.66,cycle_ticks=48,support_surface_iterations=2,support_polygon_mode='sole_footprints',crouch=3)
    r['actions']['walk']['gait']=r['locomotion']
    for name,s in r['actions'].items():
        if name in ['idle','attack','hit']:s['grounded']=True
        for k in s['keys']:
            t=k['tick']/s['duration_ticks']
            k['channels']['jaw']=[0,0,0]
            k['channels']['tail']=[0,round(math.sin(t*math.tau)*5),0]
            if name=='idle':k['channels']['head']=[round(math.sin(t*math.tau)*2),round(math.sin(t*math.tau)*3),0]
            if name=='attack':
                wind=max(0,1-abs(t-.25)/.25);strike=max(0,1-abs(t-.55)/.16)
                k['channels']['head']=[round(-12*wind+24*strike),0,0];k['channels']['jaw']=[round(-23*wind-4*strike),0,0];k['translation']=[0,0,round(8*wind-17*strike)]
            if name=='hit':k['channels']['head']=[0,round(-12*math.sin(t*math.pi)),0]
            if name=='death':k['channels']['jaw']=[-12,0,0];k['channels']['tail']=[0,round(18*min(t/.7,1)),0]
    r=finish(r,'Original stocky ash burrower: low shoulder crest, broad shovel snout, digging paws, short tail and layered refractory hide. It hunts warm slag rather than wearing human equipment. Shared parametric limb rig only; no Kilnback geometry or cache creature data.')
    r['materials'].update(hide=dict(hsl=10280,render_type=1),scute=dict(hsl=5664,render_type=1),muzzle=dict(hsl=10293,render_type=1),claw=dict(hsl=10301,render_type=1),socket=dict(hsl=8,render_type=1),eye=dict(hsl=5050,render_type=1))
    if refined:
        r['materials']['hide']['hsl']=10278
        r['materials']['scute']['hsl']=5653
        for p in c:
            if p['name']=='shovel_skull':p['sections'][1][3]=36
            if p['name'].endswith('_paw') and p['name'] in ['leg_0_paw','leg_2_paw']:p['size'][0]*=1.25;p['size'][2]*=1.15
            if p['name'] in ['leg_0_upper','leg_2_upper','leg_0_shin','leg_2_shin']:p['radii']=[v*1.22 for v in p['radii']]
            if p['name'].startswith('ear_'):
                side=-1 if p['name'].endswith('-1') else 1
                p.clear();p.update(part('ear_'+str(side),'profile','hide',1,profile=[[side*20,-79],[side*36,-76],[side*33,-94]],depth=6,socket=dict(origin=[0,0,-54])))
        for k in r['actions']['attack']['keys']:
            t=k['tick']/42;wind=max(0,1-abs(t-.3)/.3);strike=max(0,1-abs(t-.57)/.14)
            k['channels']['jaw']=[round(-40*wind-3*strike),0,0]
            k['channels']['body']=[round(-6*wind+10*strike),0,0]
            k['channels']['head']=[round(-19*wind+31*strike),0,0]
            k['translation']=[0,round(4*wind),round(12*wind-22*strike)]
        for k in r['actions']['death']['keys']:
            t=k['tick']/84;fold=min(t/.28,1);roll=max(0,min((t-.24)/.46,1));roll=roll*roll*(3-2*roll)
            k['channels']['body']=[round(8*min(t/.3,1)),0,round(61*roll)]
            for i in range(4):
                k['channels']['leg_'+str(i)+'_hip']=[round((-24 if i in [0,2] else 24)*fold),0,0]
                k['channels']['leg_'+str(i)+'_knee']=[round((37 if i in [0,2] else -30)*fold),0,0]
        r['actions']['attack']['events']=dict(anticipation=[0,14],execution=[14,25],recovery=[25,42])
        r['actions']['death']['terminal_pose_hold']=60
    r['rig_validation']=True
    return r

if __name__=='__main__':
    import argparse
    p=argparse.ArgumentParser();p.add_argument('out',type=Path);p.add_argument('--refined',action='store_true');a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
    for name,fn in [('A-anvil',reference),('B-riveter',riveting),('C-stamp',press),('D-sootsnout',creature)]:
        (a.out/(name+'.json')).write_text(json.dumps(fn(a.refined),indent=2),encoding='utf-8')
