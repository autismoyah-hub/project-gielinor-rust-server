"""Construction-level owned asset families: seams, joinery and material roles.

All shapes are generated from dimensions and explicit semantic parameters. No
cache vertex, face or animation arrays enter the constructors.
"""
import math
from templates import part,recipe
from semantic_materials import library

def asset(name,components,**extra):
    r=recipe(name,components);r['materials'].update(library());r.update(extra);r['authoring_schema']=2;return r

def plank(name,center,size,axis=1,variation=0,group=0):
    direction=[int(i==axis)for i in range(3)]
    return part(name,'stone_block','wood',group,role='board',center=center,size=size,bevel=1,variation=variation,grain_axis=direction,parameters=dict(material_family='wood',construction='separate plank',grain_axis=axis))

def masonry(width=128,height=144,depth=40,rows=4,seed=0,corner=False):
    c=[];rh=height/rows
    for row in range(rows):
        count=2 if row%2==0 else 3;step=width/count
        for n in range(count):
            c.append(part(f'course_{row}_block_{n}','stone_block','stone',role='masonry',center=[-width/2+step*(n+.5),-rh*(row+.5),0],size=[step-2,rh-2,depth],bevel=3,wear=.1 if (row+n+seed)%5==0 else 0,seed=row*7+n+seed,variation=[-2,0,2][(row+n+seed)%3]))
    if corner:
        other=masonry(depth,height,width,rows,seed+3)['components']
        for p in other:p['name']='return_'+p['name'];p['socket']=dict(origin=[width/2-depth/2,0,width/2-depth/2])
        c+=other
    c.append(part('coping','stone_block','stone',center=[0,-height-6,0],size=[width+6,11,depth+7],bevel=3))
    return asset('Bonded masonry module',c)

def arch(width=224,height=224,depth=58,segments=9,repaired=True):
    inner=width*.35;outer=width/2;spring=height-outer;c=[]
    for side in [-1,1]:
        for row in range(3):c.append(part(f'pier_{side}_{row}','stone_block','stone',center=[side*(inner+15),-spring*(row+.5)/3,0],size=[32,spring/3-2,depth+5],bevel=3,wear=.09 if row==0 else 0,seed=row+side,variation=(row%3-1)*2))
        c.append(part(f'impost_{side}','stone_block','stone',center=[side*(inner+15),-spring,0],size=[41,13,depth+12],bevel=2))
    for i in range(segments):
        a=i*math.pi/segments+.009;b=(i+1)*math.pi/segments-.009;key=i==segments//2
        c.append(part('keystone'if key else f'voussoir_{i}','voussoir','stone',role='arch_wedge',center=[0,-spring,0],inner=inner-(3 if key else 0),outer=outer+(7 if key else 0),start_angle=a,end_angle=b,depth=depth+(9 if key else 0),variation=2 if key else[-2,0,1][i%3],parameters=dict(radial_joint=True)))
    if repaired:
        c.append(part('repair_tie','box','metal',center=[-inner-15,-spring*.55,-depth/2-3],size=[34,8,4]))
        for x in [-inner-26,-inner-4]:c.append(part('tie_bolt_'+str(x),'beam','metal',start=[x,-spring*.55,-depth/2-3],end=[x,-spring*.55,-depth/2-7],radii=[3,3],sides=6))
    return asset('Repaired radial masonry arch',c)

def barrel(width=70,height=90,depth=70,staves=10):
    c=[];rx,rz=width/2,depth/2;sections=[[0,rx*.78,rz*.78],[-height*.18,rx*.92,rz*.92],[-height*.5,rx,rz],[-height*.82,rx*.92,rz*.92],[-height,rx*.78,rz*.78]]
    for i in range(staves):c.append(part(f'stave_{i}','stave','wood',role='stave',sections=sections,start_angle=i*math.tau/staves+.014,end_angle=(i+1)*math.tau/staves-.014,thickness=4,variation=[-2,0,2,0][i%4],parameters=dict(construction='individual bowed stave',gap_angle=.028)))
    for i,y in enumerate([-.16*height,-.8*height]):
        r=1-abs(y/height-.5)*.3;c.append(part('hoop_'+str(i),'annulus','metal',center=[0,0,0],radii=[rx*r+2,rz*r+2],rim_width=3,depth=6,sides=staves,socket=dict(origin=[0,y,0],basis=[[1,0,0],[0,0,1],[0,-1,0]])))
    # Recessed coopered head, actual board boundaries; no painted fake grain.
    capr=rx*.75
    for i in range(4):
        x=(i-1.5)*capr*.5;length=2*rz*.74*math.sqrt(max(.12,1-(x/capr)**2));c.append(plank('head_board_'+str(i),[x,-height+3,0],[capr*.5-1,4,length],axis=2,variation=i%3-1))
    c.append(part('bung','beam','wood',start=[0,-height*.53,-rz],end=[0,-height*.53,-rz-3],radii=[4,4],sides=6))
    return asset('Coopered stave barrel',c)

def crate(width=90,height=90,depth=90):
    c=[]
    for side in [-1,1]:
        for i in range(4):
            c.append(plank(f'front_{side}_board_{i}',[(i-1.5)*width/4,-height/2,side*(depth/2-3)],[width/4-1.5,height-5,5],variation=(i%3-1)*2))
            c.append(plank(f'side_{side}_board_{i}',[side*(width/2-3),-height/2,(i-1.5)*depth/4],[5,height-5,depth/4-1.5],variation=((i+1)%3-1)*2))
        for x in [-width*.4,width*.4]:c.append(plank(f'corner_{side}_{x}',[x,-height/2,side*(depth/2+1)],[8,height,6]))
        for y in [-height*.1,-height*.9]:c.append(plank(f'rail_{side}_{y}',[0,y,side*(depth/2+2)],[width,8,7],axis=0))
        c.append(part('diagonal_'+str(side),'beam','wood',start=[-width*.36,-height*.16,side*(depth/2+5)],end=[width*.36,-height*.84,side*(depth/2+5)],radii=[4,4],sides=4,grain_axis=[width*.72,-height*.68,0]))
        for x in [-width*.4,width*.4]:
            for y in [-height*.1,-height*.9]:c.append(part(f'nail_{side}_{x}_{y}','beam','metal',start=[x,y,side*(depth/2+6)],end=[x,y,side*(depth/2+8)],radii=[2,2],sides=4))
    for i in range(4):c.append(plank('lid_board_'+str(i),[(i-1.5)*width/4,-height+2,0],[width/4-1.5,5,depth-5],axis=2,variation=i%3-1))
    return asset('Joined plank crate',c)

def clipped(poly,axis,edge,lower):
    result=[]
    for a,b in zip(poly,poly[1:]+poly[:1]):
        ina=a[axis]>=edge if lower else a[axis]<=edge;inb=b[axis]>=edge if lower else b[axis]<=edge
        if ina:result.append(a)
        if ina!=inb:
            t=(edge-a[axis])/(b[axis]-a[axis]);result.append([a[j]+t*(b[j]-a[j])for j in range(2)])
    return result

def shield(width=76,height=98,depth=8,round_top=False):
    poly=[[-width*.5,-height*.4],[-width*.3,-height*.5],[width*.3,-height*.5],[width*.5,-height*.4],[width*.44,height*.13],[width*.2,height*.42],[0,height*.5],[-width*.2,height*.42],[-width*.44,height*.13]];c=[]
    for i in range(5):
        lo=-width/2+i*width/5+.6;hi=lo+width/5-1.2;p=clipped(clipped(poly,0,lo,True),0,hi,False);c.append(part('board_'+str(i),'profile','wood',role='shield_board',profile=p,depth=depth,variation=(i%3-1)*2,socket=dict(origin=[0,0,-(1-((lo+hi)/width)**2)*3])))
    for j,y in enumerate([-height*.27,height*.1]):c.append(part('back_batten_'+str(j),'box','wood',center=[0,y,depth/2+4],size=[width*.83,6,5],grain_axis=[1,0,0]))
    for x in [-width*.3,width*.3]:
        for y in [-height*.27,height*.1]:c.append(part(f'rivet_{x}_{y}','beam','metal',start=[x,y,-depth/2-4],end=[x,y,-depth/2-6],radii=[2,2],sides=5))
    c.append(part('grip','beam','leather',start=[-9,0,depth/2+9],end=[9,0,depth/2+9],radii=[3,3],sides=6))
    return asset('Framed plank shield',c)

def scimitar(length=96,width=24,curve=24):
    c=[part('blade','sweep','metal',role='curved_blade',sections=[[0,-22,0,width*.26,2],[0,-length*.45,0,width*.32,2],[curve*.2,-length*.68,0,width*.38,2],[curve*.6,-length*.88,0,width*.34,1],[curve,-length,0,1,1]],sides=4),part('guard','sweep','bronze',sections=[[-width*.5,-21,0,2,2],[0,-17,0,3,3],[width*.5,-19,0,2,2]],sides=4),part('handle','beam','leather',start=[0,0,0],end=[0,-17,0],radii=[3,3],sides=6),part('pommel','loft','bronze',sections=[[0,1,0,4,4],[0,-3,0,4,4]],sides=6)]
    return asset('Swept curved blade',c)

def anvil(width=122,height=64,depth=38):
    c=[part('pedestal','loft','metal',sections=[[0,0,0,width*.4,depth*.48],[0,-height*.14,0,width*.4,depth*.48],[0,-height*.3,0,width*.2,depth*.3],[0,-height*.63,0,width*.25,depth*.4],[0,-height*.78,0,width*.34,depth*.47]],sides=4,outline=[[-1,-1],[1,-1],[1,1],[-1,1]]),part('face','stone_block','metal',center=[-width*.09,-height*.88,0],size=[width*.62,height*.25,depth],bevel=2),part('horn','sweep','metal',sections=[[width*.2,-height*.85,0,depth*.42,depth*.34],[width*.35,-height*.85,0,depth*.29,depth*.22],[width*.51,-height*.87,0,1,1]],normal=[0,1,0],sides=6),part('heel','stone_block','metal',center=[-width*.45,-height*.86,0],size=[width*.13,height*.27,depth*.86],bevel=2)]
    return asset('Forged component anvil',c)

def bench(width=150,height=70,depth=65,backrest=True,seat_height_ratio=.64):
    seat=height*seat_height_ratio if backrest else height
    c=[plank('seat_'+str(i),[0,-seat,(i-1)*depth*.27],[width,7,depth*.26],axis=0,variation=i-1)for i in range(3)]
    for x in [-width*.34,width*.34]:
        c.append(plank('trestle_'+str(x),[x,-seat*.5,0],[12,seat,depth*.8]))
        c.append(plank('foot_'+str(x),[x,-5,0],[20,9,depth*.9],axis=2))
    c.append(plank('stretcher',[0,-seat*.35,0],[width*.7,9,9],axis=0))
    if backrest:
        for side in [-1,1]:c.append(plank('back_post_'+str(side),[side*width*.4,-(height+seat)/2,depth*.38],[8,height-seat+10,8]))
        for i in range(2):c.append(plank('back_rail_'+str(i),[0,-height+5+i*(height-seat)*.5,depth*.38],[width,7,7],axis=0))
    return asset('Joined bench with authored backrest',c)

def chest(width=110,height=85,depth=70):
    r=crate(width,height*.65,depth);r['name']='Banded coopered chest';c=r['components'];cy=-height*.65;radius=depth/2
    for i in range(7):
        a=i*math.pi/7+.01;b=(i+1)*math.pi/7-.01;c.append(part('lid_stave_'+str(i),'voussoir','wood',center=[0,cy,0],inner=radius-5,outer=radius,start_angle=a,end_angle=b,depth=width,socket=dict(basis=[[0,0,1],[0,1,0],[-1,0,0]]),variation=i%3-1,grain_axis=[1,0,0]))
    c.append(part('hasp','box','metal',center=[0,cy+8,-depth/2-5],size=[12,25,5]));return r

def pickaxe(length=110,width=90):
    return asset('Bent forged pick',[part('haft','beam','wood',start=[0,0,0],end=[0,-length,0],radii=[4,3],sides=6),part('head','sweep','metal',sections=[[-width/2,-length+15,0,1,1],[-width*.25,-length-1,0,5,4],[0,-length-5,0,7,5],[width*.25,-length-1,0,5,4],[width/2,-length+15,0,1,1]],sides=6),part('wedge','stone_block','metal',center=[0,-length-5,0],size=[11,10,11],bevel=2)])

def hammer(length=105,width=46):
    return asset('Forged crosspeen hammer',[part('haft','beam','wood',start=[0,0,0],end=[0,-length,0],radii=[4,3],sides=6),part('head','stone_block','metal',center=[0,-length+7,0],size=[width,25,25],bevel=3),part('peen','sweep','metal',sections=[[width*.4,-length+7,0,8,8],[width*.7,-length+7,0,3,4]],normal=[0,1,0],sides=4)])
