"""Original Cinderworks assets using semantic construction and baked contact rigs."""
import math,json,argparse,copy
from pathlib import Path
from construction import asset,arch,barrel,plank,masonry
from creature_families import basic_actions
from templates import part
from semantic_materials import material

def provenance(r,brief):
    r['geometry_validation']=dict(remove_zero_area_faces=True,preserve_vertex_order=True)
    r['provenance']=dict(kind='original_tower_asset',source_mesh_or_animation_reuse=False,brief=brief,artistic_acceptance=False,live_integration=False);return r

def arcade():
    r=arch(252,242,70,9,True);r['name']='Sluice keeper arcade';c=r['components']
    for side in [-1,1]:
        c.append(part('foundation_'+str(side),'stone_block','stone',center=[side*107,-8,0],size=[60,16,90],bevel=4,wear=.08,seed=side+2))
        c.append(part('guide_rail_'+str(side),'box','metal',center=[side*77,-68,12],size=[7,128,9]))
        c.append(part('guide_shoe_'+str(side),'stone_block','metal',center=[side*77,-12,12],size=[18,18,20],bevel=2))
    c += [part('lintel_cable_eye','annulus','bronze',center=[0,-219,-39],radii=[11,13],rim_width=4,depth=5,sides=8),part('repair_splice','box','metal',center=[108,-53,-39],size=[45,7,5])]
    r['materials']['stone'].update(hsl=10286,shadow=10274,end=10294,edge=10291)
    return provenance(r,'A bonded-stone sluice opening with radial voussoirs, a projecting keystone, iron gate guides and a repaired west impost. Course joints and one restrained spalled corner make construction readable.')

def hoist():
    c=[plank('foundation',[0,-6,0],[164,12,96],axis=0)]
    for side in [-1,1]:
        c += [plank('standard_'+str(side),[side*63,-79,0],[15,146,21]),plank('foot_'+str(side),[side*63,-12,0],[28,12,103],axis=2),part('brace_'+str(side),'beam','wood',start=[side*63,-22,-40],end=[side*63,-104,0],radii=[5,5],sides=4,grain_axis=[0,-82,40]),part('bearing_'+str(side),'stone_block','metal',center=[side*63,-113,0],size=[24,24,25],bevel=3)]
    drum=barrel(48,104,48,10)
    for p in drum['components']:
        if not(p['name'].startswith('stave_')or p['name'].startswith('hoop_')):continue
        p=copy.deepcopy(p);p['name']='drum_'+p['name'];p['parent_socket']=dict(origin=[52,-113,0],basis=[[0,1,0],[1,0,0],[0,0,-1]]);p['grain_axis']=[1,0,0];c.append(p)
    for side in [-1,1]:
        c.append(part('drum_flange_'+str(side),'annulus','metal',center=[0,0,0],radii=[29,29],rim_width=5,depth=5,sides=12,socket=dict(origin=[side*53,-113,0],basis=[[0,1,0],[0,0,1],[1,0,0]])))
    c += [part('axle','beam','metal',start=[-78,-113,0],end=[80,-113,0],radii=[6,6],sides=8),part('crank','sweep','metal',sections=[[78,-113,0,4,4],[82,-113,0,4,4],[82,-113,28,4,4],[96,-113,28,4,4]],normal=[0,1,0],sides=4),part('crank_grip','beam','wood',start=[88,-113,28],end=[102,-113,28],radii=[5,5],sides=6,grain_axis=[1,0,0]),part('counterweight_rope','sweep','leather',sections=[[-25,-133,-5,2,2],[-25,-130,-24,2,2],[-25,-35,-24,2,2]],normal=[1,0,0],sides=5),part('counterweight','stone_block','stone',center=[-25,-22,-24],size=[25,28,25],bevel=3),part('pawl','beam','metal',start=[-65,-130,11],end=[-65,-115,28],radii=[3,2],sides=4)]
    for i in range(10):
        a=math.tau*i/10;c.append(part('ratchet_tooth_'+str(i),'stone_block','metal',center=[-56,-113+30*math.sin(a),30*math.cos(a)],size=[6,6,6],bevel=1))
    r=asset('Counterweight sluice hoist',c);r['materials']['wood'].update(hsl=6934,shadow=6924,end=6940,edge=6938)
    return provenance(r,'A coopered timber drum in a braced trestle frame, with bearing blocks, ratchet/pawl, an accessible crank and a guided stone counterweight. End grain, separate boards, iron and cable retain distinct native palettes.')

def kilnback():
    c=[part('abdomen','sweep','organic',0,role='torso',sections=[[0,-33,-19,29,20],[0,-43,12,40,29],[0,-41,39,32,24],[0,-32,60,17,16]],normal=[0,1,0],sides=8),part('thorax','sweep','organic',0,role='neck',sections=[[0,-34,-44,19,15],[0,-41,-29,26,22],[0,-38,-12,29,23]],normal=[0,1,0],sides=6),part('head','sweep','organic',1,role='head',sections=[[0,-33,-43,17,13],[0,-31,-63,23,16],[0,-24,-78,17,10]],normal=[0,1,0],sides=6)]
    for i,(z,width,length,top)in enumerate([(-8,34,28,-67),(18,39,31,-74),(43,28,27,-64)]):
        c.append(part('shell_plate_'+str(i),'loft','ceramic',0,role='carapace',sections=[[0,top+17,z,width,length/2],[0,top+4,z,width*.78,length*.45],[0,top,z,width*.4,length*.35]],sides=8,variation=i-1))
        for x in [-6,6]:c.append(part(f'vent_{i}_{x}','box','organic',center=[x,top-1,z],size=[3,2,length*.32]))
    rig=[dict(name='body',parent=None,origin=[0,-35,0],groups=list(range(23))),dict(name='head',parent='body',origin=[0,-35,-43],groups=[1,21,22])];legs=[]
    for side in [-1,1]:
        for n,z in enumerate([-34,-1,32]):
            idx=(0 if side<0 else 3)+n;upper=2+idx*3;lower=upper+1;footgroup=upper+2;hip=[side*24,-35,z];knee=[side*(49+(7 if n==1 else 0)),-23,z+(-9 if n==0 else 7)];tip=[side*(61+(7 if n==1 else 0)),-3,z+(-17 if n==0 else 13)];name='leg_'+str(idx)
            c += [part(name+'_coxa','loft','organic',upper,role='leg',sections=[[hip[0],hip[1]+5,hip[2],9,9],[hip[0],hip[1]-5,hip[2],7,7]],sides=6),part(name+'_upper','beam','organic',upper,start=hip,end=knee,radii=[6,4],sides=5),part(name+'_shin','beam','organic',lower,start=knee,end=tip,radii=[4,3],sides=5),part(name+'_pad','stone_block','organic',footgroup,role='foot',center=[tip[0],-2,tip[2]-3],size=[9,4,15],bevel=1)]
            for claw in [-1,1]:c.append(part(f'{name}_claw_{claw}','beam','bronze',footgroup,start=[tip[0]+claw*3,-2,tip[2]-8],end=[tip[0]+claw*4,-1,tip[2]-13],radii=[2,1],sides=4))
            rig += [dict(name=name+'_hip',parent='body',origin=hip,groups=[upper,lower,footgroup]),dict(name=name+'_knee',parent=name+'_hip',origin=knee,groups=[lower,footgroup]),dict(name=name+'_ankle',parent=name+'_knee',origin=tip,groups=[footgroup])]
            legs.append(dict(name=name,hip=hip,knee=knee,tip=tip,components=[name+'_pad'],phase_offset=.5 if idx in [1,3,5]else 0,knee_axis=2))
        group=21 if side<0 else 22;label='left'if side<0 else'right';mouth=[side*14,-22,-73]
        c += [part('eye_'+label,'beam','eye',1,start=[side*15,-34,-73],end=[side*17,-35,-77],radii=[3,3],sides=5),part('mandible_'+label,'sweep','bronze',group,role='mandible',sections=[[side*14,-22,-73,5,4],[side*25,-16,-88,4,3],[side*15,-15,-96,1,1]],normal=[0,1,0],sides=5),part('feeler_'+label,'sweep','organic',1,sections=[[side*11,-42,-67,2,2],[side*23,-53,-79,2,2],[side*32,-52,-92,1,1]],normal=[0,1,0],sides=4)]
        rig.append(dict(name='mandible_'+label,parent='head',origin=mouth,groups=[group]))
    c.append(part('tail','sweep','bronze',20,sections=[[0,-31,55,7,6],[0,-21,68,5,4],[0,-13,75,1,1]],normal=[1,0,0],sides=6));rig.append(dict(name='tail',parent='body',origin=[0,-31,55],groups=[20]))
    gait=dict(kind='alternating_tripod',limbs=legs,stride=30,lift=10,stance_fraction=.62,crouch=3,cycle_ticks=48,support_surface_iterations=2);actions=basic_actions('body','head',gait)
    for name,spec in actions.items():
        if name in ['idle','attack','hit']:spec['grounded']=True
        spec['events']={'attack':dict(anticipation=[0,12],execution=[12,22],recovery=[22,42]),'death':dict(collapse=[0,61],final_state=[61,174])}.get(name,{})
        for key in spec['keys']:
            t=key['tick']/spec['duration_ticks']
            if name=='idle':key['channels']['tail']=[round(math.sin(t*math.tau)*3),0,0]
            if name=='attack':
                open_=max(0,1-abs(t-.28)/.28);snap=max(0,1-abs(t-.51)/.12);key['channels']['mandible_left']=[0,round(-12*open_+8*snap),0];key['channels']['mandible_right']=[0,round(12*open_-8*snap),0]
                key['channels']['body']=[round(-4*open_),0,0];key['translation'][2]=round(4*open_-max(0,19*max(0,1-abs(t-.5)/.2))*.9)
            if name=='hit':
                pulse=max(0,1-abs(t-.25)/.25);key['channels']['body']=[0,0,round(pulse*7)];key['translation'][2]=round(pulse*5)
            if name=='death':
                for limb in legs:key['channels'][limb['name']+'_knee']=[0,0,round((-1 if limb['tip'][0]>0 else 1)*min(t/.72,1)*34)]
    actions['walk_entry']=dict(duration_ticks=8,cadence_ticks=2,loop=False,keys=[],bridge_controls=dict(source='idle',destination='walk'),interpolation='LINEAR')
    r=asset('Kilnback rummager · articulated revision',c,rig=rig,actions=actions,locomotion=gait);r['materials']['organic'].update(hsl=10264,shadow=10257,end=10273,edge=10269);r['materials']['eye']=dict(hsl=7106,render_type=1)
    return provenance(r,'A low six-legged foundry scavenger with overlapping ceramic plates, articulated feet, curved mouthparts and an actual neck transition. Alternating tripod support, readable jaw attack and a persistent collapsed corpse are authored through native classic controls.')

def terrain():
    c=[]
    for x in range(-4,4):
        for z in range(-3,3):
            cx=x*128+64;cz=z*128+64
            if abs(cx)<90:continue
            c.append(part(f'paving_{x}_{z}','stone_block','stone',role='terrain',center=[cx,7,cz],size=[125,14,125],bevel=2,variation=[-2,0,1][(x+z)%3],wear=.08 if(x-z)%5==0 else 0,seed=x*7+z))
    for side in [-1,1]:
        for z in range(-3,3):c.append(part('infill_'+str(side)+'_'+str(z),'stone_block','stone',center=[side*107,7,z*128+64],size=[42,14,125],bevel=2,variation=-1))
    for z in range(-3,3):
        c.append(part('water_'+str(z),'box','water',center=[0,14,z*128+64],size=[126,3,128]))
        for side in [-1,1]:c.append(part('channel_curb_'+str(side)+'_'+str(z),'stone_block','stone',center=[side*75,-5,z*128+64],size=[18,18,123],bevel=2,variation=-1))
    for i in range(6):c.append(plank('bridge_'+str(i),[0,-13,-120+i*17],[260,10,15],axis=0,variation=i%3-1))
    r=asset('Cinderworks paving, runoff and plank crossing',c);r['materials']['stone'].update(hsl=10274,shadow=10267,end=10280,edge=10278);r['materials']['wood'].update(hsl=6934,shadow=6924,end=6940,edge=6938);r['materials']['water']=dict(hsl=32286,render_type=1)
    return provenance(r,'An isolated eight-by-six-tile floor vignette: divided paving, a recessed runoff channel and a short timber crossing. No production map tiles are replaced.')

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('out',type=Path);a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
    for key,fn in [('sluice-keeper-arcade',arcade),('counterweight-hoist',hoist),('kilnback-rummager',kilnback),('showcase-terrain',terrain)]: (a.out/(key+'.json')).write_text(json.dumps(fn(),indent=2))
