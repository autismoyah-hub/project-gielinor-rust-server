"""Parameterized authoring archetypes. No cache mesh or animation arrays.

These are modeling starting points, not a claim of universal OSRS proportions.
Recipes for direct reference reconstructions must remain private.
"""
def part(name,kind,material,group=0,role=None,**kw):return dict(name=name,type=kind,material=material,group=group,role=role or name,**kw)
def materials():return {k:dict(hsl=v,render_type=0)for k,v in dict(stone=43051,edge=10262,stone_shadow=10258,iron=10299,iron_edge=10320,wood=8604,wood_dark=8485,leather=3769,skin=10782,skin_shadow=10772,cloth=916,armor=6955,armor_shadow=7075,gray_armor=5169,eye=127,brow=8598).items()}
def attachment_beam(grip,mount,tail_fraction=.75,head_fraction=.5,radius=2):
    direction=[mount[i]-grip[i]for i in range(3)]
    return dict(start=[grip[i]-tail_fraction*direction[i]for i in range(3)],end=[mount[i]+head_fraction*direction[i]for i in range(3)],radii=[radius,radius],sides=6)
def recipe(name,components,rig=None,actions=None):return dict(schema=1,name=name,unit='native;128 units per tile',materials=materials(),components=components,rig=rig or [],actions=actions or {})
def pillar(width=128,height=240,depth=128):
    w,d=width/2,depth/2
    sections=[[0,0,0,w,d],[0,-height*.12,0,w,d],[0,-height*.17,0,w*.7,d*.7],[0,-height*.78,0,w*.64,d*.64],[0,-height*.84,0,w*.8,d*.8],[0,-height*.9,0,w,d],[0,-height,0,w,d]]
    return recipe('Independent faceted pillar',[part('shaft','loft','stone',sections=sections,sides=4),part('foot_joint','box','stone_shadow',center=[0,-height*.125,0],size=[width+2,3,depth+2])])
def anvil(width=122,height=64,depth=38):
    sections=[[0,0,0,width*.4,depth*.48],[0,-height*.16,0,width*.4,depth*.48],[0,-height*.3,0,width*.22,depth*.3],[0,-height*.7,0,width*.27,depth*.42],[0,-height*.78,0,width*.34,depth*.5],[0,-height,0,width*.34,depth*.5]]
    return recipe('Independent industrial anvil',[part('pedestal','loft','iron',sections=sections,sides=4,outline=[[-1,-1],[1,-1],[1,1],[-1,1]]),part('striking_face','box','iron_edge',center=[-width*.06,-height+1,0],size=[width*.58,4,depth]),part('horn','beam','iron',start=[width*.3,-height*.86,0],end=[width*.5,-height*.82,0],radii=[depth*.4,2],sides=6),part('heel','box','iron',center=[-width*.43,-height*.85,0],size=[width*.14,height*.3,depth*.9])])

def shield(width=54,height=58,depth=7,kind='kite'):
    p=[[-width/2,-height/2],[width/2,-height/2],[width*.46,height*.05],[0,height/2],[-width*.46,height*.05]]if kind=='kite'else[[-width/2,-height/2],[width/2,-height/2],[width/2,height/2],[-width/2,height/2]]
    import math
    return recipe('Independent shield',[part('shield','profile','iron',profile=p,depth=depth),part('boss','profile','iron_edge',profile=[[8*math.cos(i*math.tau/8),8*math.sin(i*math.tau/8)]for i in range(8)],depth=3,socket=dict(origin=[0,0,-depth/2-2]))])
def sword(length=100,width=14,curve=0):
    sections=[[0,-15,0,3,3],[0,-30,0,width*.45,3],[curve*.3,-length*.6,0,width*.5,2],[curve,-length,0,2,2]]
    return recipe('Independent blade',[part('blade','loft','iron_edge',sections=sections,sides=4),part('guard','box','iron',center=[0,-15,0],size=[width*2.2,5,7]),part('grip','beam','wood_dark',start=[0,0,0],end=[0,-15,0],radii=[3,3],sides=6)])
def wheel(width=387,height=397,depth=126,duration=64):
    import math
    rx,ry=width/2,height/2;cy=-height/2-10;components=[]
    for z,tag in [(-depth*.36,'front'),(depth*.36,'back')]:components.append(part('rim_'+tag,'annulus','wood',1,center=[0,cy,z],radii=[rx,ry],rim_width=min(rx,ry)*.26,depth=12,sides=12))
    for i in range(4):
        angle=i*math.tau/4;components.append(part('spoke_'+str(i),'beam','wood_dark',1,start=[0,cy,-depth*.36],end=[rx*.86*math.cos(angle),cy+ry*.86*math.sin(angle),-depth*.36],radii=[13,13],sides=4))
    for i in range(12):
        angle=i*math.tau/12;u=[math.cos(angle),-math.sin(angle),0];v=[math.sin(angle),math.cos(angle),0];components.append(part('paddle_'+str(i),'box','wood',1,center=[0,0,0],size=[35,8,depth],socket=dict(origin=[rx*.9*math.sin(angle),cy+ry*.9*math.cos(angle),0],basis=[u,v,[0,0,1]])))
    components.append(part('axle','beam','iron',0,start=[0,cy,-depth*.5],end=[0,cy,depth*.5],radii=[12,12],sides=6))
    rig=[dict(name='rotor',parent=None,origin=[0,cy,0],groups=[1])];actions=dict(turn=dict(duration_ticks=duration,cadence_ticks=2,loop=True,keys=[dict(tick=0,channels=dict(rotor=[0,0,0])),dict(tick=duration,channels=dict(rotor=[0,0,256]))]))
    return recipe('Independent faceted waterwheel',components,rig,actions)
def barrel(width=70,height=90,depth=70):
    w,d=width/2,depth/2;components=[part('staves','loft','wood',sections=[[0,0,0,w*.78,d*.78],[0,-height*.25,0,w,d],[0,-height*.7,0,w,d],[0,-height,0,w*.78,d*.78]],sides=8)]
    for i,y in enumerate([-.15*height,-.8*height]):components.append(part('hoop_'+str(i),'loft','iron',sections=[[0,y+3,0,w+2,d+2],[0,y-3,0,w+2,d+2]],sides=8))
    return recipe('Independent barrel',components)
def crate(width=90,height=90,depth=90):
    c=[part('body','box','wood',center=[0,-height/2,0],size=[width*.95,height*.95,depth*.95])]
    for side in [-1,1]:
        for x in [-width*.4,width*.4]:c.append(part(f'brace_{side}_{x}','box','wood_dark',center=[x,-height/2,side*depth/2],size=[8,height,6]))
        for y in [-height*.1,-height*.9]:c.append(part(f'rail_{side}_{y}','box','wood_dark',center=[0,y,side*depth/2],size=[width,8,6]))
    return recipe('Independent crate',c)
def quadruped(width=75,height=95,depth=180):
    import math
    c=[part('body','loft','skin_shadow',0,sections=[[0,-height*.4,0,width*.45,depth*.36],[0,-height*.7,0,width*.5,depth*.4],[0,-height*.8,0,width*.3,depth*.3]],sides=6),part('head','loft','skin',1,sections=[[0,-height*.6,-depth*.4,width*.25,depth*.14],[0,-height*.9,-depth*.4,width*.28,depth*.16]],sides=6),part('muzzle','beam','skin_shadow',1,start=[0,-height*.68,-depth*.4],end=[0,-height*.65,-depth*.6],radii=[width*.17,width*.1],sides=6)]
    rig=[dict(name='body_root',parent=None,origin=[0,-height*.55,0],groups=list(range(14))),dict(name='head',parent='body_root',origin=[0,-height*.65,-depth*.25],groups=[1])];legs=[]
    for n,(x,z)in enumerate([(-width*.32,-depth*.23),(width*.32,-depth*.23),(-width*.32,depth*.23),(width*.32,depth*.23)]):
        upper,lower=2+n*2,3+n*2;hip=[x,-height*.6,z];knee=[x,-height*.3,z];foot=[x,0,z];tag='leg_'+str(n);c += [part(tag+'_upper','beam','skin_shadow',upper,start=hip,end=knee,radii=[max(3,width*.1),max(3,width*.07)],sides=5),part(tag+'_lower','beam','skin',lower,start=knee,end=foot,radii=[max(3,width*.07),max(3,width*.05)],sides=5)];rig += [dict(name=tag+'_hip',parent='body_root',origin=hip,groups=[upper,lower]),dict(name=tag+'_knee',parent=tag+'_hip',origin=knee,groups=[lower])];legs.append(tag)
    c.append(part('tail','beam','skin_shadow',10,start=[0,-height*.6,depth*.35],end=[0,-height*.35,depth*.65],radii=[max(3,width*.09),3],sides=5));rig.append(dict(name='tail',parent='body_root',origin=[0,-height*.6,depth*.35],groups=[10]))
    actions={}
    for action,duration in [('idle',60),('walk',48),('attack',40),('hit',36),('death',60)]:
        keys=[]
        for i in range(9):
            t=i/8;channels={};translation=[0,0,0]
            if action=='idle':channels['head']=[round(math.sin(t*math.tau)*2),0,0]
            if action=='walk':
                for n,tag in enumerate(legs):swing=round(math.sin(t*math.tau+(math.pi if n in [1,2]else 0))*16);channels[tag+'_hip']=[swing,0,0];channels[tag+'_knee']=[max(0,-swing),0,0]
            if action=='attack':channels['head']=[round(-math.sin(t*math.pi)*22),0,0];translation[2]=round(-math.sin(t*math.pi)*depth*.08)
            if action=='hit':channels['body_root']=[0,0,round(math.sin(t*math.pi)*8)]
            if action=='death':channels['body_root']=[0,0,round(min(t*1.5,1)*64)]
            keys.append(dict(tick=round(t*duration),channels=channels,translation=translation))
        actions[action]=dict(duration_ticks=duration,cadence_ticks=3,keys=keys,loop=action in ['idle','walk'])
        if action=='death':actions[action].update(grounded=True,terminal_zero=True,terminal_hold=300)
    return recipe('Independent quadruped anatomy study',c,rig,actions)
def humanoid(p=None):
    """Anatomical components and ownership are independent of specific cache rigs."""
    p=dict(p or {});height=p.get('height',158);s=height/158;parts=[]
    def add(name,kind,mat,group=0,role=None,**kw):parts.append(part(name,kind,mat,group,role,**kw))
    add('torso','loft','armor',0,sections=[[0,-72,0,21,14],[0,-88,0,24,16],[0,-114,0,26,15],[0,-123,0,19,12]],sides=6)
    add('tunic','loft','cloth',0,sections=[[0,-58,-2,27,19],[0,-77,-2,23,17]],sides=6)
    add('neck','loft','skin',1,role='head',sections=[[0,-122,-4,8,8],[0,-133,-6,8,8]],sides=6)
    hw=p.get('head_width',38)/2;hd=p.get('head_depth',30)/2
    add('skull','loft','skin',1,role='head',sections=[[0,-130,-10,11,9],[0,-140,-13,hw,hd],[0,-152,-13,hw*.92,hd*.82],[0,-158,-12,7,7]],sides=6)
    add('muzzle','loft','skin_shadow',1,role='head',sections=[[0,-132,-26,9,7],[0,-141,-27,12,9]],sides=6)
    add('nose','loft','skin',1,role='head',sections=[[0,-137,-34,4,5],[0,-145,-30,3,3]],sides=5)
    shoulder=p.get('shoulder_span',60)/2;arm=p.get('arm_length',38);ear=p.get('ear_span',66)/2
    for side,prefix,upper,lower,hand,thigh,calf,foot in [(-1,'right',2,3,4,8,9,10),(1,'left',5,6,7,11,12,13)]:
        shoulder_pos=[side*shoulder,-120,0];elbow=[side*(shoulder+arm*.5),-106,-4];wrist=[side*(shoulder+arm),-98,-10]
        add(prefix+'_upperarm','beam','skin',upper,role='arms',start=shoulder_pos,end=elbow,radii=[10,8],sides=6);add(prefix+'_forearm','beam','skin',lower,role='arms',start=elbow,end=wrist,radii=[9,6],sides=6)
        add(prefix+'_pauldron','loft','gray_armor',upper,role='armor',sections=[[side*shoulder,-113,0,12,10],[side*shoulder,-125,0,10,9]],sides=6)
        add(prefix+'_hand','loft','skin',hand,role='hands',sections=[[wrist[0],-91,-12,6,6],[wrist[0],-103,-12,6,6]],sides=5)
        x=side*22;hip=[side*17,-65,0];knee=[x,-31,0];ankle=[x,-7,-3]
        add(prefix+'_thigh','beam','skin',thigh,role='legs',start=hip,end=knee,radii=[10,8],sides=6);add(prefix+'_shin','beam','leather',calf,role='legs',start=knee,end=ankle,radii=[8,7],sides=6);add(prefix+'_foot','loft','skin',foot,role='feet',sections=[[x,0,-16,12,17],[x,-7,-12,10,14]],sides=6)
        ear_profile=[[side*13,-150],[side*16,-137],[side*ear,-156]]
        add(prefix+'_ear','profile','skin',1,role='head',profile=ear_profile,depth=4,socket=dict(origin=[0,0,-10]))
        add(prefix+'_brow','box','brow',1,role='head',center=[side*8,-147,-25],size=[11,4,3]);add(prefix+'_eye','box','eye',1,role='head',center=[side*8,-143,-27],size=[4,2,2])
    for x in [-12,12]:
        for y in [-116,-105,-94]:add(f'armor_seam_{x}_{y}','box','armor_shadow',0,role='armor',center=[x,y,-16],size=[16,2,3])
    # Socket basis makes equipment orientation explicit and testable.
    axe_origin=[-shoulder-arm-7,-84,-40];blade=[[-28,-10],[-22,-25],[-8,-29],[7,-20],[10,-4],[-4,4],[-12,-6]]
    add('weapon_blade','profile','iron',4,role='weapon',profile=blade,depth=p.get('weapon_thickness',4),socket=dict(origin=axe_origin,basis=[[0,0,1],[0,1,0],[-1,0,0]]));add('weapon_handle','beam','wood_dark',4,role='weapon',**attachment_beam([axe_origin[0],-98,-12],axe_origin))
    shield_w=p.get('shield_width',52);shield_h=p.get('shield_height',54)
    for i in range(5):add(f'shield_plank_{i}','box','wood'if i%2 else'wood_dark',7,role='shield',center=[shoulder+arm+16+(i-2)*shield_w/5,-101,-8+(i-2)*3],size=[shield_w/5+1,shield_h-abs(i-2)*5,7])
    rig=[]
    rig.append(dict(name='body_root',parent=None,origin=[0,-75,0],groups=list(range(14))))
    rig.append(dict(name='spine',parent='body_root',origin=[0,-73,0],groups=list(range(8))))
    rig.append(dict(name='head',parent='spine',origin=[0,-128,-5],groups=[1]))
    for side,prefix,upper,lower,hand,thigh,calf,foot in [(-1,'right',2,3,4,8,9,10),(1,'left',5,6,7,11,12,13)]:
        rig += [dict(name=prefix+'_shoulder',parent='spine',origin=[side*shoulder,-120,0],groups=[upper,lower,hand]),dict(name=prefix+'_elbow',parent=prefix+'_shoulder',origin=[side*(shoulder+arm*.5),-106,-4],groups=[lower,hand]),dict(name=prefix+'_hip',parent='body_root',origin=[side*17,-65,0],groups=[thigh,calf,foot]),dict(name=prefix+'_knee',parent=prefix+'_hip',origin=[side*22,-31,0],groups=[calf,foot]),dict(name=prefix+'_ankle',parent=prefix+'_knee',origin=[side*22,-7,-3],groups=[foot])]
    if s!=1:raise ValueError('Use component parameters to resize anatomy deliberately; uniform height fitting is not implicit')
    actions={}
    for name,duration in [('idle',63),('walk',48),('attack',36),('hit',95),('death',90)]:
        keys=[]
        for i in range(9):
            import math
            t=i/8;phase=t*math.tau;channels={};translation=[0,0,0]
            if name=='idle':channels={'head':[0,round(math.sin(phase)*2),0]};translation[1]=round(math.sin(phase)*1)
            elif name=='walk':
                from kinematics import leg_contact_channels
                swing=round(math.sin(phase)*16);channels={'right_shoulder':[-swing//2,0,0],'left_shoulder':[swing//2,0,0]};translation[1]=2+round(abs(math.sin(phase)))
                joints={j['name']:j['origin']for j in rig}
                for prefix,offset in [('right',0),('left',.5)]:
                    legphase=(t+offset)%1
                    if legphase<.5:step_z=10*(1-4*legphase);lift=0
                    else:u=(legphase-.5)*2;step_z=-10+20*(u*u*(3-2*u));lift=7*math.sin(u*math.pi)
                    target=[joints[prefix+'_ankle'][0],-7-lift,-3+step_z];solution=leg_contact_channels(joints[prefix+'_hip'],joints[prefix+'_knee'],joints[prefix+'_ankle'],target,translation)
                    if not solution['reachable']:raise ValueError('walk target exceeds authored limb reach')
                    for joint in ['hip','knee','ankle']:channels[prefix+'_'+joint]=solution[joint]
            elif name=='attack':
                anticipation=-10*max(0,1-abs(t-.18)/.18);strike=45*max(0,1-abs(t-.48)/.28);channels={'right_shoulder':[round(anticipation+strike),round(-strike*.25),round(-strike*.3)],'right_elbow':[round(strike*.4),0,0],'spine':[0,round(strike*.25),0]}
            elif name=='hit':channels={'spine':[round(math.sin(t*math.pi)*14),0,0]};translation[2]=round(math.sin(t*math.pi)*6)
            elif name=='death':channels={'body_root':[round(min(t*1.5,1)*64),0,0],'right_hip':[-round(t*20),0,0],'left_hip':[-round(t*20),0,0],'right_knee':[round(t*35),0,0],'left_knee':[round(t*35),0,0]}
            keys.append(dict(tick=round(t*duration),channels=channels,translation=translation))
        actions[name]=dict(duration_ticks=duration,cadence_ticks=3,loop=name in ['idle','walk'],keys=keys)
        if name=='walk':actions[name]['foot_contacts']=[dict(components=['right_foot'],phase_offset=0),dict(components=['left_foot'],phase_offset=.5)]
        if name=='death':actions[name].update(grounded=True,terminal_zero=True,terminal_hold=300)
    return recipe('Independent humanoid workbench study',parts,rig,actions)
