"""Reusable fitted anatomy/armor, mechanical sections and original variations."""
import math,copy
from templates import part
from construction import asset,arch,masonry,plank,scimitar,shield,anvil,barrel
from production_designs import provenance

CHEST=[[-1,-.65],[-.63,-1],[.63,-1],[1,-.65],[1,.6],[.6,1],[-.6,1],[-1,.6]]
PELVIS=[[-.84,-.7],[-.48,-1],[.48,-1],[.84,-.7],[1,.5],[.5,1],[-.5,1],[-1,.5]]
FACE=[[-.8,-.5],[-.52,-1],[.52,-1],[.8,-.5],[1,.4],[.5,1],[-.5,1],[-1,.4]]

def fitted_torso(name,hip,chest,neck,widths,depths,material='organic',group=0):
    return part(name,'section_mesh',material,group,role='anatomy_torso',sections=[[0,hip,0,widths[0],depths[0]],[0,(hip+chest)*.5,0,widths[1],depths[1]],[0,chest,0,widths[2],depths[2]],[0,neck,1,widths[3],depths[3]]],profiles=[PELVIS,CHEST,CHEST,FACE],sides=8,parameters=dict(landmarks=['pelvis','waist','thorax','neck']))

def articulated_hand(name,wrist,side,scale=1,group=0,closed=False,material='organic'):
    x,y,z=wrist;c=[part(name+'_palm','section_mesh',material,group,role='hand',sections=[[x,y,z,5*scale,4*scale],[x,y+10*scale,z-2*scale,6*scale,4*scale]],sides=6)]
    for n in range(3):
        px=x+(n-1)*4*scale;end=[px,y+(13 if closed else 20)*scale,z-(8 if closed else 4)*scale]
        c.append(part(name+'_finger_'+str(n),'sweep',material,group,role='finger',sections=[[px,y+9*scale,z-2*scale,2*scale,2*scale],[px,y+15*scale,z-3*scale,2*scale,2*scale],end+[max(1,scale),max(1,scale)]],normal=[1,0,0],sides=4))
    c.append(part(name+'_thumb','beam',material,group,start=[x+side*5*scale,y+2*scale,z],end=[x+side*9*scale,y+10*scale,z-5*scale],radii=[3*scale,2*scale],sides=5));return c

def armor_lames(name,center,width,depth,height,group=0,rows=3):
    x,y,z=center;c=[]
    for i in range(rows):
        cy=y+i*height/rows;rx=width*(.5-.035*i);rz=depth*.5
        c.append(part(name+'_lame_'+str(i),'shell','metal',group,role='lapped_armor',sections=[[x,cy,z,rx,rz],[x,cy+height/rows+3,z-1,rx*.94,rz*.94]],thickness=3,sides=8,start_angle=math.pi,end_angle=math.tau,variation=(i%2)*2))
    return c

def framed_threshold(width=220,height=220,depth=55):
    r=arch(width,height,depth,11,True);r['name']='Counterfort threshold';c=r['components']
    c.append(part('worn_threshold','section_mesh','stone',role='threshold',sections=[[0,0,0,width*.5,depth*.65],[0,-10,0,width*.5,depth*.65]],sides=8,profiles=[CHEST,CHEST]))
    for side in [-1,1]:
        c.append(part('buttress_'+str(side),'section_mesh','stone',sections=[[side*width*.46,0,depth*.2,26,depth*.8],[side*width*.46,-height*.5,depth*.12,21,depth*.55],[side*width*.46,-height*.7,0,14,depth*.4]],sides=8,profiles=[CHEST]*3))
        for i in range(3):c.append(part(f'corbel_{side}_{i}','stone_block','stone',center=[side*width*.46,-height*.65-i*11,-depth*.44-i*2],size=[34-i*5,10,depth*.7+i*5],bevel=2))
    return provenance(r,'A buttressed radial threshold with a worn sill, corbel transitions and repaired iron ties; independently authored modular construction.')

def geared_pump():
    c=[part('cast_plinth','section_mesh','stone',sections=[[0,0,0,88,65],[0,-14,0,88,65],[0,-24,0,74,53]],sides=8,profiles=[CHEST]*3)]
    for side in [-1,1]:
        c.append(part('pedestal_'+str(side),'section_mesh','metal',sections=[[side*57,-24,0,17,30],[side*57,-95,0,12,19],[side*57,-109,0,20,20]],sides=8))
        c.append(part('gear_'+str(side),'gear','bronze',role='cog',center=[side*38,-108,-29],radius=34 if side<0 else 26,root_radius=28 if side<0 else 21,teeth=10 if side<0 else 8,depth=8,bore=10))
    c += [part('cross_shaft','beam','metal',start=[-78,-108,-29],end=[78,-108,-29],radii=[7,7],sides=8),part('riser','sweep','metal',sections=[[30,-26,23,12,12],[30,-119,23,12,12],[17,-141,23,11,11],[-24,-141,23,11,11],[-43,-120,23,9,9]],normal=[0,0,1],sides=8)]
    for y in [-45,-96]:c.append(part('riser_flange_'+str(y),'loft','bronze',sections=[[30,y-4,23,17,17],[30,y+4,23,17,17]],sides=8))
    c.append(part('repair_wrap','shell','bronze',sections=[[30,-70,23,14,14],[30,-56,23,14,14]],thickness=2,sides=8))
    return provenance(asset('Eccentric runoff pump',c),'An exposed geared pump with unequal cog wheels, bearing pedestals, cast plinth, bent riser and a repaired coupling. No cache mesh reuse.')

def tower_blade():
    r=scimitar(125,28,32);r['name']='Ash-hook cleaver';r['components'][0]['sections']=[[0,-23,0,7,3],[-3,-57,0,11,3],[7,-91,0,13,2],[25,-119,0,9,2],[40,-127,0,1,1]]
    r['components']+= [part('spine_binding','sweep','bronze',sections=[[-7,-37,0,2,3],[-8,-65,0,2,3],[0,-88,0,2,2]],sides=4),part('hilt_rivet','beam','bronze',start=[0,-11,-4],end=[0,-11,4],radii=[2,2],sides=5)]
    return provenance(r,'An original forward-hook foundry cleaver with a reinforced spine and offset tip; curved weapon lessons without copying a reference silhouette.')

def tower_shield():
    r=shield(83,112,7);r['name']='Lockkeeper braced buckler';r['components']+=armor_lames('brace',[0,-26,-6],64,16,35,rows=2)
    r['components'].append(part('boss','loft','bronze',sections=[[0,4,-10,12,5],[0,-4,-10,14,7]],sides=8));return provenance(r,'Separate clipped boards, back joinery, lapped iron reinforcement and a small boss define an original lockkeeper shield.')

def variations():return {'counterfort-threshold':framed_threshold(),'eccentric-runoff-pump':geared_pump(),'ash-hook-cleaver':tower_blade(),'lockkeeper-shield':tower_shield()}

if __name__=='__main__':
    import argparse,json
    from pathlib import Path
    p=argparse.ArgumentParser();p.add_argument('out',type=Path);a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
    for key,r in variations().items():(a.out/(key+'.json')).write_text(json.dumps(r,indent=2))
