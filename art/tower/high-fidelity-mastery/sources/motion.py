"""Full classic creature actions on explicitly authored anatomical groups."""
import sys,json,math
from pathlib import Path
HERE=Path(__file__).resolve().parent;sys.path.insert(0,str(HERE.parent/'asset-workbench'))
import bpy
from author_blender import actions
folder=Path(sys.argv[-1]).resolve();g=json.loads((folder/'geometry.json').read_text());audit=json.loads((folder/'conversion.json').read_text());g['components']={};offset=0
for c in audit['components']:
    g['components'][c['name']]=dict(vertices=list(range(offset,offset+c['vertices'])),group=c['group']);offset+=c['vertices']
rig=[dict(name='body',parent=None,origin=[0,-61,0],groups=list(range(16))),dict(name='head',parent='body',origin=[0,-78,-62],groups=[1,15]),dict(name='jaw',parent='head',origin=[0,-65,-80],groups=[15]),dict(name='tail',parent='body',origin=[0,-52,57],groups=[14])]
limbs=[]
for limb in json.loads((folder/'limbs.json').read_text()):
    i=limb['index'];tag='leg_'+str(i);upper,lower,foot=[limb[x]for x in ['upper','lower','foot']]
    rig += [dict(name=tag+'_hip',parent='body',origin=limb['hip'],groups=[upper,lower,foot]),dict(name=tag+'_knee',parent=tag+'_hip',origin=limb['knee'],groups=[lower,foot]),dict(name=tag+'_ankle',parent=tag+'_knee',origin=limb['ankle'],groups=[foot])]
    footnames=[n for n,c in g['components'].items()if c['group']==foot]
    limbs.append(dict(name=tag,hip=limb['hip'],knee=limb['knee'],tip=limb['ankle'],components=footnames,phase_offset=.5 if i in [1,2]else 0,knee_axis=0))
specs={}
for name,duration in [('idle',80),('walk',64),('attack',60),('hit',40),('death',100)]:
    keys=[]
    for tick in range(0,duration+1,4):
        t=tick/duration;phase=t*math.tau;ch={};tr=[0,0,0]
        if name=='idle':
            ch=dict(body=[0,0,round(math.sin(phase)*1)],head=[round(math.sin(phase)*2),round(math.sin(phase)*2),0],jaw=[round(-2-max(0,math.sin(phase))*2),0,0],tail=[0,round(math.sin(phase)*4),0])
        elif name=='walk':
            ch=dict(body=[0,round(math.sin(phase)*1),0],head=[round(math.sin(phase)*2),0,0],jaw=[-2,0,0],tail=[0,round(math.sin(phase)*6),0]);tr[1]=-round(abs(math.sin(phase))*2)
        elif name=='attack':
            wind=max(0,1-abs(t-.27)/.27);strike=max(0,1-abs(t-.55)/.17);recover=max(0,1-abs(t-.78)/.20)
            ch=dict(body=[round(-4*wind+3*strike),0,0],head=[round(-9*wind+16*strike-2*recover),0,0],jaw=[round(-19*wind-10*strike),0,0],tail=[0,round(8*wind),0]);tr=[0,round(-3*wind),round(8*wind-22*strike)]
            for i in [0,2]:ch['leg_'+str(i)+'_hip']=[round(-7*wind+12*strike),0,0];ch['leg_'+str(i)+'_knee']=[round(8*wind-5*strike),0,0]
        elif name=='hit':
            recoil=math.sin(min(t/.4,1)*math.pi/2)*(max(0,1-(t-.4)/.6)if t>.4 else 1)
            ch=dict(body=[round(recoil*5),0,round(-recoil*6)],head=[round(-recoil*8),round(recoil*3),0],jaw=[round(-recoil*8),0,0],tail=[0,round(-recoil*7),0]);tr=[round(recoil*3),0,round(recoil*10)]
        else:
            fold=min(1,t/.45);roll=max(0,min(1,(t-.3)/.45));settle=min(1,max(0,(t-.7)/.3))
            ch=dict(body=[round(fold*5),0,round(roll*54)],head=[round(fold*11),round(-roll*4),0],jaw=[round(-fold*12+settle*3),0,0],tail=[0,round(roll*11),round(-roll*5)])
            for i in range(4):ch['leg_'+str(i)+'_hip']=[round(-fold*(13 if i in [0,2]else -10)),0,round(roll*40)];ch['leg_'+str(i)+'_knee']=[round(fold*23),0,0];ch['leg_'+str(i)+'_ankle']=[round(-fold*12),0,0]
            tr=[0,0,round(fold*7)]
        keys.append(dict(tick=tick,channels=ch,translation=tr))
    specs[name]=dict(duration_ticks=duration,cadence_ticks=2,loop=name in ['idle','walk'],keys=keys,grounded=name!='walk')
    if name=='walk':specs[name]['gait']=dict(kind='quadruped',limbs=limbs,stride=24,lift=10,stance_fraction=.68,cycle_ticks=duration,support_surface_iterations=2,crouch=4)
    if name=='death':specs[name]['terminal_pose_hold']=60
r=dict(rig=rig,actions=specs,skeleton_base=64600,frame_group_base=62400)
delivery=r
if '--only-death'in sys.argv:delivery=dict(r,actions={'death':specs['death']},skeleton_base=64604,frame_group_base=62404)
actions(delivery,g,folder);(folder/'rig.json').write_text(json.dumps(r,indent=2));(folder/'geometry-rigged.json').write_text(json.dumps(g));bpy.ops.wm.save_as_mainfile(filepath=str(folder/'source-rigged.blend'));print('NATIVE_ACTIONS_AUTHORED')
