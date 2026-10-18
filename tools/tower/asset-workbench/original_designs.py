"""Original Tower design probes, built solely from owned components.

No reference mesh, recipe, animation or image input. These original designs test
art-direction transfer; they are not accepted production assets.
"""
import math,json,argparse
from pathlib import Path
from templates import part,recipe

def finish(r,brief):
    r['provenance']=dict(kind='original_tower_design_probe',source_geometry_reused=False,brief=brief,artistic_acceptance=False,live_integration=False)
    r['materials'].update(limestone=dict(hsl=10304,render_type=0),joint=dict(hsl=10272,render_type=1),iron=dict(hsl=36902,render_type=0),iron_edge=dict(hsl=36924,render_type=0),bronze=dict(hsl=8744,render_type=0),soot=dict(hsl=10259,render_type=0),ceramic=dict(hsl=4773,render_type=0),ceramic_edge=dict(hsl=4790,render_type=0),eye=dict(hsl=7106,render_type=1))
    return r

def saddle():
    c=[part('socket_foundation','loft','limestone',sections=[[0,0,0,82,66],[0,-14,0,82,66],[0,-23,0,72,57]],sides=8)]
    for side in [-1,1]:
        x=side*51;tag='west'if side<0 else'east'
        c += [part(tag+'_pier','loft','limestone',sections=[[x,-22,0,24,43],[x,-38,0,24,43],[x,-46,0,20,34],[x,-154,0,18,30],[x,-178,0,25,38],[x,-192,0,25,38]],sides=8),part(tag+'_iron_shoe','loft','iron',sections=[[x,-48,0,21,35],[x,-57,0,21,35]],sides=8),part(tag+'_collar','loft','joint',sections=[[x,-157,0,20,32],[x,-165,0,20,32]],sides=8)]
        for z in [-35,35]:
            c.append(part(tag+'_stay_'+str(z),'beam','bronze',start=[side*70,-35,z],end=[side*36,-137,z],radii=[4,4],sides=5))
        c.append(part(tag+'_cap','box','limestone',center=[x,-195,0],size=[57,9,82]))
    c += [part('crosshead','box','iron',center=[0,-193,0],size=[111,16,35]),part('saddle_roller','beam','bronze',start=[-28,-162,0],end=[28,-162,0],radii=[20,20],sides=10),part('roller_axle','beam','iron_edge',start=[-44,-162,0],end=[44,-162,0],radii=[7,7],sides=6),part('cable_down','beam','soot',start=[0,-149,-16],end=[0,-24,-16],radii=[3,3],sides=5),part('cable_rear','beam','soot',start=[0,-183,0],end=[0,-125,51],radii=[3,3],sides=5),part('cable_guide','box','iron',center=[0,-35,-16],size=[22,15,15])]
    for side in [-1,1]:
        for y in [-65,-141]:c.append(part(f'fastener_{side}_{y}','beam','iron_edge',start=[side*51,y,-35],end=[side*51,y,-40],radii=[4,4],sides=6))
    return finish(recipe("Tensioner's saddle",c),'Split masonry piers cradle a worn bronze cable roller. Iron shoes and diagonal stays explain its load path; the open center preserves a distinct forked silhouette.')

def governor():
    c=[part('plinth','loft','limestone',sections=[[0,0,0,69,51],[0,-14,0,69,51],[0,-22,0,62,45]],sides=8),part('base_plate','box','iron',center=[0,-25,0],size=[116,7,78])]
    for side,height in [(-1,108),(1,142)]:
        x=side*31;tag='return'if side<0 else'feed'
        c += [part(tag+'_chamber','loft','iron',sections=[[x,-27,0,22,25],[x,-38,0,26,28],[x,-height+14,0,26,28],[x,-height,0,17,21]],sides=8),part(tag+'_flange','loft','bronze',sections=[[x,-55,0,28,30],[x,-64,0,28,30]],sides=8),part(tag+'_cap','loft','iron_edge',sections=[[x,-height+5,0,22,25],[x,-height-2,0,22,25]],sides=8),part(tag+'_drain','beam','bronze',start=[x,-36,-21],end=[x,-36,-50],radii=[7,7],sides=6)]
    c += [part('equalizer','beam','bronze',start=[-31,-88,0],end=[31,-88,0],radii=[9,9],sides=8),part('valve_spindle','beam','iron_edge',start=[31,-111,-20],end=[31,-111,-53],radii=[4,4],sides=6),part('handwheel','annulus','bronze',center=[31,-111,-55],radii=[22,22],rim_width=5,depth=5,sides=10),part('handwheel_cross','box','bronze',center=[31,-111,-55],size=[36,4,5]),part('handwheel_cross2','box','bronze',center=[31,-111,-55],size=[4,36,5]),part('indicator_stem','beam','bronze',start=[-31,-108,0],end=[-31,-138,0],radii=[3,3],sides=6),part('pressure_flag','profile','ceramic',profile=[[-12,0],[0,-16],[12,0],[0,16]],depth=4,socket=dict(origin=[-31,-145,-2]))]
    for x in [-50,50]:
        for z in [-30,30]:c.append(part('anchor_'+str(x)+'_'+str(z),'loft','iron_edge',sections=[[x,-28,z,4,4],[x,-33,z,4,4]],sides=6))
    return finish(recipe('Twin-throat sluice governor',c),'Unequal feed and return chambers share a bronze equalizer. A front handwheel, drains and ceramic pressure flag make its purpose readable without text or glowing effects.')

def kilnback():
    c=[part('abdomen','loft','soot',0,sections=[[0,-22,14,34,49],[0,-43,14,41,55],[0,-62,15,33,47],[0,-70,16,14,23]],sides=8),part('carapace','loft','ceramic',0,sections=[[0,-40,20,40,51],[0,-62,20,34,44],[0,-74,20,15,24]],sides=8),part('thorax','loft','soot',0,sections=[[0,-25,-38,22,23],[0,-50,-38,25,25],[0,-59,-38,14,18]],sides=6),part('head','loft','soot',1,sections=[[0,-20,-65,18,15],[0,-36,-67,23,19],[0,-45,-64,14,14]],sides=6)]
    rig=[dict(name='body',parent=None,origin=[0,-35,0],groups=list(range(15))),dict(name='head',parent='body',origin=[0,-35,-47],groups=[1])];legs=[]
    for side in [-1,1]:
        for n,z in enumerate([-36,-4,32]):
            idx=(0 if side<0 else 3)+n;upper=2+idx*2;lower=upper+1;hip=[side*24,-35,z];knee=[side*(50+(7 if n==1 else 0)),-25,z+(-12 if n==0 else 8)];foot=[side*(61+(7 if n==1 else 0)),0,z+(-24 if n==0 else 16)];name='leg_'+str(idx)
            c += [part(name+'_upper','beam','soot',upper,start=hip,end=knee,radii=[7,5],sides=5),part(name+'_shin','beam','soot',lower,start=knee,end=foot,radii=[5,3],sides=5),part(name+'_toe','beam','bronze',lower,start=foot,end=[foot[0]+side*8,-1,foot[2]-7],radii=[3,1],sides=5)]
            rig += [dict(name=name+'_hip',parent='body',origin=hip,groups=[upper,lower]),dict(name=name+'_knee',parent=name+'_hip',origin=knee,groups=[lower])];legs.append((name,side,idx))
        c += [part('eye_'+str(side),'beam','eye',1,start=[side*14,-35,-81],end=[side*16,-35,-84],radii=[3,3],sides=5),part('mandible_'+str(side),'beam','bronze',1,start=[side*14,-23,-77],end=[side*24,-16,-94],radii=[5,2],sides=5),part('feeler_'+str(side),'beam','iron',1,start=[side*12,-42,-74],end=[side*31,-54,-89],radii=[2,1],sides=5)]
    for n,z in enumerate([-4,12,28,43]):
        c.append(part('shell_vent_'+str(n),'box','soot',center=[0,-75,z],size=[20-abs(z-18)*.2,3,4]))
    c.append(part('tail_hook','beam','bronze',14,start=[0,-35,59],end=[0,-18,77],radii=[8,2],sides=6));rig.append(dict(name='tail',parent='body',origin=[0,-35,59],groups=[14]))
    actions={}
    for action,duration in [('idle',64),('walk',48),('attack',42),('hit',30),('death',72)]:
        keys=[]
        for i in range(13):
            t=i/12;phase=t*math.tau;channels={};translation=[0,0,0]
            if action=='idle':channels={'head':[0,round(math.sin(phase)*3),0],'tail':[round(math.sin(phase)*4),0,0]}
            if action=='walk':
                translation[1]=round(abs(math.sin(phase))*2)
                for tag,side,idx in legs:
                    wave=math.sin(phase+(math.pi if idx in [1,3,5]else 0));channels[tag+'_hip']=[round(wave*10),round(wave*side*7),round(max(0,wave)*side*6)];channels[tag+'_knee']=[0,0,round(max(0,wave)*-side*12)]
            if action=='attack':
                anticipate=-5*max(0,1-abs(t-.2)/.2);strike=15*max(0,1-abs(t-.5)/.23);channels['head']=[round(anticipate+strike),0,0];translation[2]=round(-strike*.7)
            if action=='hit':channels['body']=[0,0,round(math.sin(t*math.pi)*8)];translation[2]=round(math.sin(t*math.pi)*4)
            if action=='death':
                channels['body']=[0,0,round(min(t*1.6,1)*64)]
                for tag,side,idx in legs:channels[tag+'_knee']=[0,0,round(-side*min(t*1.5,1)*28)]
            keys.append(dict(tick=round(t*duration),channels=channels,translation=translation))
        actions[action]=dict(duration_ticks=duration,cadence_ticks=3,loop=action in ['idle','walk'],keys=keys)
        if action=='death':actions[action].update(grounded=True,terminal_zero=True,terminal_hold=300)
    return finish(recipe('Kilnback rummager',c,rig,actions),'A six-legged scavenger shelters a soot-black body beneath chipped ceramic kiln armor. Copper mouthparts and a hooked tail echo foundry tools without reproducing an existing creature.')

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('out',type=Path);a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
    for name,builder in [('tensioners-saddle',saddle),('sluice-governor',governor),('kilnback-rummager',kilnback)]: (a.out/(name+'.json')).write_text(json.dumps(builder(),indent=2))
