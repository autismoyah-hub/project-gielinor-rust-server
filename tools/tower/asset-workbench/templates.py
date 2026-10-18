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
    p=[[-width*.48,0],[-width*.5,-height*.18],[-width*.26,-height*.3],[-width*.18,-height*.7],[-width*.5,-height*.78],[-width*.5,-height],[width*.22,-height],[width*.5,-height*.9],[width*.22,-height*.78],[width*.1,-height*.7],[width*.18,-height*.3],[width*.38,-height*.18],[width*.4,0]]
    return recipe('Independent industrial anvil',[part('body','profile','iron',profile=p,depth=depth),part('striking_face','box','iron_edge',center=[-width*.12,-height-1,0],size=[width*.62,3,depth*.9])])
def shield(width=54,height=58,depth=7,kind='kite'):
    p=[[-width/2,-height/2],[width/2,-height/2],[width*.46,height*.05],[0,height/2],[-width*.46,height*.05]]if kind=='kite'else[[-width/2,-height/2],[width/2,-height/2],[width/2,height/2],[-width/2,height/2]]
    return recipe('Independent shield',[part('shield','profile','iron',profile=p,depth=depth),part('boss','loft','iron_edge',sections=[[0,-6,-depth,8,4],[0,5,-depth,8,4]],sides=6)])
def sword(length=100,width=14,curve=0):
    sections=[[0,-15,0,3,3],[0,-30,0,width*.45,3],[curve*.3,-length*.6,0,width*.5,2],[curve,-length,0,2,2]]
    return recipe('Independent blade',[part('blade','loft','iron_edge',sections=sections,sides=4),part('guard','box','iron',center=[0,-15,0],size=[width*2.2,5,7]),part('grip','beam','wood_dark',start=[0,0,0],end=[0,-15,0],radii=[3,3],sides=6)])
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
                swing=round(math.sin(phase)*16);channels={'right_hip':[swing,0,0],'left_hip':[-swing,0,0],'right_knee':[max(0,-swing),0,0],'left_knee':[max(0,swing),0,0],'right_ankle':[-max(0,swing),0,0],'left_ankle':[-max(0,-swing),0,0],'right_shoulder':[-swing//2,0,0],'left_shoulder':[swing//2,0,0]};translation[1]=round(-abs(math.sin(phase))*2)
            elif name=='attack':
                anticipation=-10*max(0,1-abs(t-.18)/.18);strike=45*max(0,1-abs(t-.48)/.28);channels={'right_shoulder':[round(anticipation+strike),round(-strike*.25),round(-strike*.3)],'right_elbow':[round(strike*.4),0,0],'spine':[0,round(strike*.25),0]}
            elif name=='hit':channels={'spine':[round(math.sin(t*math.pi)*14),0,0]};translation[2]=round(math.sin(t*math.pi)*6)
            elif name=='death':channels={'body_root':[round(min(t*1.5,1)*64),0,0],'right_hip':[-round(t*20),0,0],'left_hip':[-round(t*20),0,0],'right_knee':[round(t*35),0,0],'left_knee':[round(t*35),0,0]}
            keys.append(dict(tick=round(t*duration),channels=channels,translation=translation))
        actions[name]=dict(duration_ticks=duration,cadence_ticks=3,loop=name in ['idle','walk'],keys=keys)
        if name=='death':actions[name].update(grounded=True,terminal_zero=True,terminal_hold=300)
    return recipe('Independent humanoid workbench study',parts,rig,actions)
