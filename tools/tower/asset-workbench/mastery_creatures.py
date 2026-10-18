"""Independent landmark-driven creature families and choreographed classic rigs.

These families are developed on ordinary calibration forms. No deferred boss
identity, cache model topology or authentic frame array is embedded here.
"""
import math,copy
from templates import part
from construction import asset
from creature_families import basic_actions,mammal,bat
from mastery_construction import fitted_torso,articulated_hand,armor_lames,FACE,CHEST
from production_designs import provenance,kilnback

def biped(height=180,bulk=1,armor=True,equipment='hammer',name='Original industrial sentinel'):
    h=height;w=h*.16*bulk;d=h*.09;hipy=-h*.47;chesty=-h*.77;necky=-h*.85;c=[]
    c.append(fitted_torso('torso',hipy,chesty,necky,[w*.72,w*.65,w,w*.45],[d*.9,d*.8,d,d*.6]))
    c.append(part('pelvis','section_mesh','leather',0,role='pelvis',sections=[[0,-h*.41,0,w*.75,d*.8],[0,hipy,0,w*.8,d*.9],[0,-h*.55,0,w*.68,d*.8]],sides=8,profiles=[CHEST]*3))
    c.append(part('neck','loft','organic',1,sections=[[0,-h*.83,0,w*.34,d*.5],[0,-h*.89,-1,w*.32,d*.45]],sides=6))
    c.append(part('head','section_mesh','organic',1,role='head',sections=[[0,-h*.86,-h*.02,w*.38,d*.6],[0,-h*.91,-h*.025,w*.5,d*.77],[0,-h*.97,0,w*.48,d*.72],[0,-h,-1,w*.3,d*.5]],sides=8,profiles=[FACE]*4))
    c += [part('nose','sweep','organic',1,role='nose',sections=[[0,-h*.947,-d*.77,3,3],[0,-h*.915,-d*1.05,4,3],[0,-h*.902,-d*.88,2,2]],normal=[1,0,0],sides=4),part('jaw','section_mesh','organic',1,sections=[[0,-h*.88,-d*.35,w*.32,d*.47],[0,-h*.91,-d*.35,w*.4,d*.53]],sides=8,profiles=[FACE]*2)]
    rig=[dict(name='body',parent=None,origin=[0,hipy,0],groups=list(range(16)),limits=[[-80,80]]*3),dict(name='head',parent='body',origin=[0,-h*.84,0],groups=[1],limits=[[-32,32]]*3)];legs=[];attachments=[]
    for side in [-1,1]:
        prefix='right'if side<0 else'left';a=2 if side<0 else 5;leg=8 if side<0 else 11
        shoulder=[side*w*.95,chesty,0];elbow=[side*w*1.3,-h*.59,-h*.015];wrist=[side*w*1.25,-h*.43,-h*.045];hip=[side*w*.45,hipy,0];knee=[side*w*.48,-h*.255,-h*.01];ankle=[side*w*.5,-h*.045,-h*.015]
        c += [part(prefix+'_upper_arm','sweep','organic',a,role='upper_arm',sections=[shoulder+[w*.33,d*.75],[(shoulder[0]+elbow[0])*.5,-h*.68,-1,w*.38,d*.8],elbow+[w*.23,d*.5]],normal=[0,0,1],sides=6),part(prefix+'_forearm','sweep','organic',a+1,role='forearm',sections=[elbow+[w*.26,d*.55],[(elbow[0]+wrist[0])*.5,-h*.52,-h*.03,w*.29,d*.6],wrist+[w*.19,d*.4]],normal=[0,0,1],sides=6)]
        c+=articulated_hand(prefix,wrist,side,h/180,a+2,closed=equipment!='none')
        c += [part(prefix+'_thigh','sweep','organic',leg,role='thigh',sections=[hip+[w*.35,d*.7],[(hip[0]+knee[0])*.5,-h*.37,1,w*.4,d*.75],knee+[w*.24,d*.5]],normal=[0,0,1],sides=6),part(prefix+'_shin','sweep','organic',leg+1,role='shin',sections=[knee+[w*.25,d*.52],[(knee[0]+ankle[0])*.5,-h*.17,2,w*.28,d*.6],ankle+[w*.18,d*.38]],normal=[0,0,1],sides=6),part(prefix+'_foot','section_mesh','leather',leg+2,role='foot',sections=[[ankle[0],0,-h*.045,w*.28,d*.8],[ankle[0],-h*.04,-h*.045,w*.29,d*.85],[ankle[0],-h*.075,-h*.02,w*.22,d*.55]],sides=8,profiles=[CHEST]*3)]
        rig += [dict(name=prefix+'_shoulder',parent='body',origin=shoulder,groups=[a,a+1,a+2]),dict(name=prefix+'_elbow',parent=prefix+'_shoulder',origin=elbow,groups=[a+1,a+2]),dict(name=prefix+'_wrist',parent=prefix+'_elbow',origin=wrist,groups=[a+2]),dict(name=prefix+'_hip',parent='body',origin=hip,groups=[leg,leg+1,leg+2]),dict(name=prefix+'_knee',parent=prefix+'_hip',origin=knee,groups=[leg+1,leg+2]),dict(name=prefix+'_ankle',parent=prefix+'_knee',origin=ankle,groups=[leg+2])]
        legs.append(dict(name=prefix,hip=hip,knee=knee,tip=ankle,components=[prefix+'_foot'],phase_offset=.5 if side>0 else 0,knee_axis=0))
        c += [part(prefix+'_brow','beam','organic',1,start=[side*w*.12,-h*.947,-d*.73],end=[side*w*.38,-h*.945,-d*.67],radii=[2,2],sides=4),part(prefix+'_eye','beam','eye',1,start=[side*w*.22,-h*.936,-d*.74],end=[side*w*.28,-h*.934,-d*.76],radii=[max(1,h*.007)]*2,sides=4)]
        if armor:
            c+=armor_lames(prefix+'_shoulder',[side*w*.97,-h*.8,0],w*.85,d*1.65,h*.095,a,2)
            c+=armor_lames(prefix+'_vambrace',[(elbow[0]+wrist[0])*.5,-h*.56,-h*.03],w*.65,d*1.2,h*.1,a+1,2)
            c+=armor_lames(prefix+'_greave',[ankle[0],-h*.21,0],w*.6,d*1.35,h*.15,leg+1,3)
    if armor:
        c+=armor_lames('breastplate',[0,-h*.77,-1],w*1.9,d*2.1,h*.17,0,3)
        c.append(part('collar','shell','bronze',0,sections=[[0,-h*.83,0,w*.55,d*.8],[0,-h*.79,0,w*.7,d]],thickness=max(2,h*.012),sides=8))
    if equipment!='none':
        x=-w*1.25;y=-h*.4;z=-h*.065;g=4
        c.append(part('weapon_haft','beam','wood',g,start=[x,y+h*.08,z],end=[x,y-h*.42,z],radii=[max(2,h*.015)]*2,sides=6))
        if equipment=='spear':
            c.append(part('weapon_head','sweep','metal',g,sections=[[x,y-h*.4,z,w*.13,3],[x,y-h*.5,z,w*.25,3],[x,y-h*.65,z,1,1]],normal=[0,0,1],sides=4))
        else:
            c.append(part('weapon_head','section_mesh','metal',g,sections=[[x,y-h*.29,z,w*.85,d*.7],[x,y-h*.35,z,w*.93,d*.77],[x,y-h*.42,z,w*.77,d*.65]],sides=8,profiles=[CHEST]*3))
        attachments.append(dict(name='weapon_grip',joint='right_wrist',components=['weapon_haft','weapon_head'],origin=[x,y,z]))
    gait=dict(kind='biped',limbs=legs,stride=h*.17,lift=h*.045,stance_fraction=.64,crouch=h*.015,cycle_ticks=48,support_surface_iterations=2,support_polygon_mode='sole_footprints',support_weight_transfer=.9)
    actions=basic_actions('body','head',gait)
    for action,spec in actions.items():
        if action in ['idle','attack','hit']:spec['grounded']=True
        spec['events']={'attack':dict(anticipation=[0,13],execution=[13,24],recovery=[24,42]),'death':dict(collapse=[0,61],terminal=[61,174])}.get(action,{})
        for key in spec['keys']:
            t=key['tick']/spec['duration_ticks'];s=math.sin(t*math.tau);ch=key['channels']
            if action=='walk':ch['right_shoulder']=[round(s*9),0,0];ch['left_shoulder']=[round(-s*9),0,0]
            if action=='attack':
                anticipation=max(0,1-abs(t-.29)/.29);strike=max(0,1-abs(t-.55)/.17);ch['right_shoulder']=[round(-28*anticipation+36*strike),round(-5*anticipation+10*strike),round(-8*anticipation)];ch['right_elbow']=[round(-20*anticipation+9*strike),0,0];ch['body']=[round(-3*anticipation+6*strike),round(-7*anticipation+11*strike),0]
            if action=='death':
                f=min(t/.72,1);ch['right_elbow']=[round(-15*f),0,0];ch['left_elbow']=[round(-24*f),0,0];ch['right_knee']=[round(22*f),0,0];ch['left_knee']=[round(29*f),0,0]
    special=copy.deepcopy(actions['attack']);special['duration_ticks']=60;special['events']=dict(anticipation=[0,22],execution=[22,35],recovery=[35,60]);special.pop('gait',None)
    for key in special['keys']:
        t=key['tick']/42;key['tick']=round(t*60);wind=max(0,1-abs(t-.35)/.35);strike=max(0,1-abs(t-.58)/.15);key['channels'].update(body=[0,round(-24*wind+26*strike),0],right_shoulder=[round(-34*wind+28*strike),0,round(-20*wind)],left_shoulder=[round(-20*wind+16*strike),0,round(20*wind)])
    actions['sweep_attack']=special
    actions['walk_entry']=dict(duration_ticks=8,cadence_ticks=2,loop=False,keys=[],bridge_controls=dict(source='idle',destination='walk'))
    r=asset(name,c,rig=rig,actions=actions,locomotion=gait,attachments=attachments,rig_validation=True,geometry_validation=dict(remove_zero_area_faces=True,preserve_vertex_order=True));r['materials']['organic'].update(hsl=4569,shadow=4558,end=4577,edge=4571);r['materials']['eye']=dict(hsl=8,render_type=1)
    return provenance(r,'Independent landmark-driven biped with fitted layered armor, articulated hands, a named weapon socket, readable windup/strike/recovery and a seven-clip native classic rig.')

def calibrated_small():return biped(159,.83,True,'spear','Independent small armored humanoid')

def refined_quadruped():
    r=mammal(70,100,170,'wolf');r['name']='Independent articulated canine revision';r['geometry_validation']=dict(remove_zero_area_faces=True,preserve_vertex_order=True)
    # Species-aware shoulder/haunch volumes and hock silhouette, not global scale.
    for p in r['components']:
        if p['name']=='torso':p['sections']=[[0,-50,-55,17,19],[0,-53,-33,26,26],[0,-43,9,22,22],[0,-42,45,24,24]]
        if p['name']=='muzzle':p['sections']=[[0,-67,-80,12,9],[0,-63,-108,8,6]]
        if p['name']=='skull':p['sections']=[[0,-73,-63,15,19],[0,-77,-78,18,17],[0,-72,-91,12,11]]
    r['locomotion']['support_surface_iterations']=2;r['locomotion']['support_polygon_mode']='sole_footprints';r['actions']['walk']['gait']=r['locomotion']
    for a in ['idle','attack','hit']:r['actions'][a]['grounded']=True
    return r

def segmented_crawler(length=310,width=92,name='Original segmented runoff crawler'):
    c=[];count=8;rig=[dict(name='body',parent=None,origin=[0,-width*.32,0],groups=list(range(12)))];front=-length*.38
    c += [part('thorax','sweep','organic',0,role='thorax',sections=[[0,-width*.34,front+length*.13,width*.4,width*.3],[0,-width*.37,front,width*.35,width*.28]],normal=[0,1,0],sides=8),part('head','sweep','organic',1,role='head',sections=[[0,-width*.35,front,width*.31,width*.25],[0,-width*.32,front-length*.12,width*.37,width*.22],[0,-width*.25,front-length*.18,width*.25,width*.13]],normal=[0,1,0],sides=8)]
    rig.append(dict(name='head',parent='body',origin=[0,-width*.34,front],groups=[1,11]));previous='body'
    for i in range(count):
        z=front+length*(.15+i*.105);t=i/(count-1);rx=width*(.45-.33*t);ry=width*(.32-.22*t);g=i+2;tag='segment_'+str(i)
        c.append(part(tag,'sweep','organic',g,role='body_segment',sections=[[0,-ry,z-length*.04,rx*.9,ry*.9],[0,-ry,z,rx,ry],[0,-ry,z+length*.055,rx*.88,ry*.85]],normal=[0,1,0],sides=8))
        c.append(part(tag+'_plate','shell','ceramic',g,role='armored_segment',sections=[[0,0,0,rx*.96,length*.047],[0,-width*.1,0,rx*.7,length*.035]],thickness=3,sides=8,socket=dict(origin=[0,-ry*1.8,z])))
        for side in [-1,1]:c.append(part(tag+'_barb_'+str(side),'sweep','bronze',g,sections=[[side*rx*.7,-ry,z,4,4],[side*(rx+width*.13),-ry*.8,z+length*.04,3,3],[side*(rx+width*.17),-ry*.6,z+length*.08,1,1]],normal=[0,1,0],sides=4))
        rig.append(dict(name=tag,parent=previous,origin=[0,-ry,z-length*.04],groups=list(range(g,10))));previous=tag
    c.append(part('jaw','sweep','bronze',11,role='jaw',sections=[[0,-width*.19,front-length*.08,width*.25,4],[0,-width*.12,front-length*.18,width*.23,3]],normal=[0,1,0],sides=6));rig.append(dict(name='jaw',parent='head',origin=[0,-width*.2,front-length*.08],groups=[11]))
    actions=basic_actions('body','head',{})
    for action,spec in actions.items():
        spec.pop('gait',None);spec.update(grounded=True,support_mode='body_contact')
        for key in spec['keys']:
            t=key['tick']/spec['duration_ticks'];channels=key['channels']
            for i in range(count):channels['segment_'+str(i)]=[0,round(math.sin(t*math.tau-i*.65)*(8 if action=='walk'else 2))if action!='death'else round(min(t/.72,1)*(12 if i%2 else-10)),0]
            if action=='attack':channels['jaw']=[round(-15*max(0,1-abs(t-.3)/.3)+8*max(0,1-abs(t-.55)/.13)),0,0]
    actions['burrow_prepare']=copy.deepcopy(actions['idle']);actions['burrow_prepare']['loop']=False
    for key in actions['burrow_prepare']['keys']:
        t=key['tick']/60
        for i in range(count):key['channels']['segment_'+str(i)]=[round(math.sin(t*math.pi)*(-7 if i<3 else 5)),round(math.sin(t*math.pi-i*.5)*6),0]
    r=asset(name,c,rig=rig,actions=actions,rig_validation=True);r['materials']['organic'].update(hsl=6460,shadow=6447,end=6465,edge=6462)
    return provenance(r,'Original armored limbless crawler with a serial axial hierarchy, independent jaw, phase-shifted body locomotion and a preparation clip. Body contact is declared; no foot-contact fidelity claim.')

def winged_biped(height=215,name='Original winged industrial sentinel'):
    r=biped(height,.95,False,'none',name);h=height
    for side,group in [(-1,14),(1,15)]:
        origin=[side*h*.13,-h*.76,h*.04];poly=[[side*h*.13,-h*.76],[side*h*.32,-h*1.02],[side*h*.68,-h*.93],[side*h*.55,-h*.7],[side*h*.76,-h*.53],[side*h*.42,-h*.61],[side*h*.25,-h*.47]]
        r['components'].append(part('wing_'+str(side),'profile','leather',group,role='wing',profile=poly,depth=3,socket=dict(origin=[0,0,h*.09])))
        for i in [1,2,4,6]:r['components'].append(part(f'wing_strut_{side}_{i}','beam','bronze',group,start=origin,end=[poly[i][0],poly[i][1],h*.09],radii=[3,1],sides=5))
        r['rig'].append(dict(name='wing_'+str(side),parent='body',origin=origin,groups=[group],limits=[[-64,64]]*3))
        for action,spec in r['actions'].items():
            for key in spec['keys']:
                t=key['tick']/spec['duration_ticks'];key['channels']['wing_'+str(side)]=[0,round(side*math.sin(t*math.tau)*(8 if action=='walk'else 3)),round(side*min(t/.72,1)*42)if action=='death'else round(side*max(0,1-abs(t-.3)/.3)*16)if action in ['attack','sweep_attack']else 0]
    return provenance(r,'An original ground-walking winged sentinel with articulated hands and independently owned braced membranes; secondary motion uses the same classic native rig, not a preview-only deformation.')

if __name__=='__main__':
    import argparse,json
    from pathlib import Path
    p=argparse.ArgumentParser();p.add_argument('out',type=Path);a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
    for key,r in [('small-armored',calibrated_small()),('large-armored',biped(272,1.18,True,'hammer')),('canine',refined_quadruped())]:(a.out/(key+'.json')).write_text(json.dumps(r,indent=2))
