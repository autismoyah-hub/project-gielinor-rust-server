"""Creature-specific owned anatomy; shared rig primitives are explicit, not inferred."""
import math
from templates import part,recipe
from construction import asset
from semantic_materials import material

def mammal(width=70,height=100,depth=170,species='wolf'):
    bulky=species in ['bear','bovine'];w=width;h=height;d=depth;c=[]
    body=[[-d*.35,h*.32,w*.26,h*.2],[-d*.2,h*.34,w*.44,h*.27],[d*.12,h*.38,w*.43,h*.29],[d*.35,h*.43,w*.32,h*.24]]
    # Local sweep axis follows Z; width and vertical radius are independent.
    c.append(part('torso','sweep','organic',0,role='torso',sections=[[0,-cy,z,rx,ry]for z,cy,rx,ry in body],normal=[0,1,0],sides=8))
    c += [part('neck','sweep','organic',1,role='neck',sections=[[0,-h*.4,-d*.25,w*.28,h*.22],[0,-h*.58,-d*.36,w*.25,h*.2],[0,-h*.65,-d*.43,w*.22,h*.18]],normal=[0,1,0],sides=6),part('skull','sweep','organic',1,role='head',sections=[[0,-h*.68,-d*.37,w*.23,h*.2],[0,-h*.7,-d*.48,w*.27,h*.19],[0,-h*.65,-d*.53,w*.19,h*.14]],normal=[0,1,0],sides=6),part('muzzle','sweep','organic',1,role='muzzle',sections=[[0,-h*.61,-d*.49,w*.2,h*.1],[0,-h*.58,-d*.64,w*.13,h*.08]],normal=[0,1,0],sides=5),part('nose','stone_block','keratin',1,role='nose',center=[0,-h*.58,-d*.65],size=[w*.23,h*.09,5],bevel=1),part('lower_jaw','beam','organic',1,start=[0,-h*.52,-d*.48],end=[0,-h*.49,-d*.63],radii=[w*.12,w*.09],sides=5)]
    rig=[dict(name='body',parent=None,origin=[0,-h*.4,0],groups=list(range(15))),dict(name='head',parent='body',origin=[0,-h*.51,-d*.26],groups=[1])];legs=[]
    for side in [-1,1]:
        for n,z in enumerate([-d*.24,d*.25]):
            idx=(0 if side<0 else 2)+n;upper=2+idx*3;lower=upper+1;footgroup=upper+2;hip=[side*w*.29,-h*.47,z];knee=[side*w*.34,-h*.24,z+(d*.06 if n else-d*.04)];ankle=[side*w*.35,-7,z-(d*.02 if n else 0)];tag='leg_'+str(idx)
            c += [part(tag+'_upper','beam','organic',upper,start=hip,end=knee,radii=[w*(.18 if bulky else .13),w*.09],sides=6),part(tag+'_shin','beam','organic',lower,start=knee,end=ankle,radii=[w*.09,w*.055],sides=5),part(tag+'_paw','stone_block','organic',footgroup,role='foot',center=[ankle[0],-4,ankle[2]-6],size=[w*(.23 if bulky else .18),8,d*.12],bevel=2)]
            for claw in [-1,0,1]:c.append(part(tag+'_claw_'+str(claw),'beam','keratin',footgroup,start=[ankle[0]+claw*w*.05,-2,ankle[2]-d*.07],end=[ankle[0]+claw*w*.05,-1,ankle[2]-d*.1],radii=[2,1],sides=4))
            rig += [dict(name=tag+'_hip',parent='body',origin=hip,groups=[upper,lower,footgroup]),dict(name=tag+'_knee',parent=tag+'_hip',origin=knee,groups=[lower,footgroup]),dict(name=tag+'_ankle',parent=tag+'_knee',origin=ankle,groups=[footgroup])];legs.append(dict(name=tag,hip=hip,knee=knee,tip=ankle,components=[tag+'_paw'],phase_offset=.5 if idx in [1,2]else 0,knee_axis=0))
        earh=h*(.12 if bulky else .21);earx=side*w*.2
        c.append(part('ear_'+str(side),'profile','organic',1,profile=[[earx-side*w*.09,-h*.78],[earx+side*w*.08,-h*.78],[earx,-h*.78-earh]],depth=5,socket=dict(origin=[0,0,-d*.42])))
        c.append(part('eye_'+str(side),'beam','eye',1,start=[side*w*.22,-h*.71,-d*.51],end=[side*w*.235,-h*.71,-d*.52],radii=[2,2],sides=5))
    c.append(part('tail','sweep','organic',14,sections=[[0,-h*.4,d*.29,w*.1,w*.1],[0,-h*.31,d*.48,w*.12,w*.12],[0,-h*.15,d*.57,2,2]],normal=[1,0,0],sides=6));rig.append(dict(name='tail',parent='body',origin=[0,-h*.4,d*.29],groups=[14]));r=asset('Owned '+species+' anatomy',c,rig=rig,actions={});r['materials']['eye']=dict(hsl=8,render_type=1);r['materials']['keratin']=material('organic',hsl=12,shadow=8,end=16,edge=18,subtype='keratin')
    if species=='wolf':r['materials']['organic'].update(hsl=10283,shadow=10270,end=10294,edge=10288)
    if species=='bear':r['materials']['organic'].update(hsl=6688,shadow=6677,end=6696,edge=6693)
    r['locomotion']=dict(kind='quadruped',limbs=legs,stride=32,lift=10,stance_fraction=.62,cycle_ticks=48)
    r['actions']=basic_actions('body','head',r['locomotion']);return r

def basic_actions(body,head,locomotion):
    actions={}
    for name,duration in [('idle',60),('walk',48),('attack',42),('hit',30),('death',84)]:
        keys=[]
        for i in range(13):
            t=i/12;phase=t*math.tau;channels={};translation=[0,0,0]
            if name=='idle':channels[head]=[round(math.sin(phase)*2),0,0]
            if name=='walk':translation[1]=-round(abs(math.sin(phase))*2)
            if name=='attack':
                envelope=-5*max(0,1-abs(t-.22)/.22)+19*max(0,1-abs(t-.5)/.2);channels[head]=[round(envelope),0,0];translation[2]=round(-max(0,envelope)*.6)
            if name=='hit':channels[body]=[0,0,round(math.sin(t*math.pi)*7)];translation[2]=round(math.sin(t*math.pi)*5)
            if name=='death':channels[body]=[0,0,round(min(t/ .72,1)*56)];channels[head]=[round(min(t/.7,1)*12),0,0]
            keys.append(dict(tick=round(t*duration),channels=channels,translation=translation))
        actions[name]=dict(duration_ticks=duration,cadence_ticks=3,loop=name in ['idle','walk'],keys=keys)
        if name=='walk':actions[name]['gait']=locomotion
        if name=='death':actions[name].update(grounded=True,terminal_pose_hold=90)
    return actions

def bat(width=120,height=70,depth=60):
    w,h,d=width,height,depth;c=[part('body','loft','organic',0,sections=[[0,-h*.35,0,w*.1,d*.2],[0,-h*.65,0,w*.14,d*.25],[0,-h*.8,0,w*.07,d*.15]],sides=6),part('head','loft','organic',1,sections=[[0,-h*.65,-d*.1,w*.1,d*.15],[0,-h*.87,-d*.1,w*.12,d*.17],[0,-h*.94,-d*.1,w*.07,d*.1]],sides=6)];rig=[dict(name='body',parent=None,origin=[0,-h*.6,0],groups=[0,1,2,3]),dict(name='head',parent='body',origin=[0,-h*.74,-d*.1],groups=[1])]
    for side in [-1,1]:
        g=2 if side<0 else 3;poly=[[side*w*.09,-d*.1],[side*w*.28,-d*.35],[side*w*.5,-d*.13],[side*w*.34,-d*.06],[side*w*.43,d*.25],[side*w*.27,d*.15],[side*w*.18,d*.38],[side*w*.08,d*.18]]
        c.append(part('membrane_'+str(side),'profile','leather',g,role='wing',profile=poly,depth=2,socket=dict(origin=[0,-h*.7,0],basis=[[1,0,0],[0,0,1],[0,-1,0]])))
        for j in [1,2,4,6]:c.append(part(f'wing_rib_{side}_{j}','beam','organic',g,start=[side*w*.09,-h*.7,-d*.1],end=[poly[j][0],-h*.7,poly[j][1]],radii=[2,1],sides=4))
        c.append(part('ear_'+str(side),'profile','organic',1,profile=[[side*w*.03,-h*.86],[side*w*.11,-h*.86],[side*w*.1,-h*1.12]],depth=3,socket=dict(origin=[0,0,-d*.1])))
        rig.append(dict(name='wing_'+str(side),parent='body',origin=[side*w*.09,-h*.7,-d*.1],groups=[g]))
    actions=basic_actions('body','head',{})
    for name,spec in actions.items():
        spec.pop('gait',None);spec['support_mode']='airborne'if name!='death'else'grounded'
        for key in spec['keys']:
            t=key['tick']/spec['duration_ticks']
            for side in [-1,1]:key['channels']['wing_'+str(side)]=[0,0,round(side*math.sin(t*math.tau)*(18 if name in ['walk','attack']else 7))]if name!='death'else[0,0,round(side*min(t/.7,1)*48)]
    return asset('Owned winged-mammal anatomy',c,rig=rig,actions=actions)
