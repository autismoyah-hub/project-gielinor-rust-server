"""Owned small-humanoid anatomy and manufactured armor/equipment controls."""
import copy,math
from templates import humanoid,part
from semantic_materials import material
from construction import shield

def small_humanoid():
    r=humanoid(dict(ear_span=48,head_width=34,head_depth=28,shield_width=80,shield_height=61));r['name']='Semantic small-humanoid study';parts=r['components'];by={p['name']:p for p in parts};by['torso']['sections']=[[0,-87,0,18,14],[0,-100,0,20,18],[0,-119,0,21,16],[0,-128,-1,14,10]];by['tunic']['sections']=[[0,-57,-1,26,18],[0,-86,-1,19,14]]
    by['skull']['sections']=[[0,-131,-10,10,8],[0,-140,-13,17,13],[0,-151,-12,15,11],[0,-158,-11,7,6]];by['skull']['surface_bands']=[dict(axis=1,min=-159,max=-149,hsl=8598)]
    by['muzzle']['sections']=[[0,-132,-25,7,6],[0,-140,-24,10,7]];by['nose'].update(type='sweep',sections=[[0,-145,-29,3,3],[0,-139,-34,4,3],[0,-134,-35,1,1]],sides=4)
    # Retire the overlarge seam boxes; shallow lapped plates have a clear material
    # and construction role instead of six unrelated stripes on a smooth torso.
    parts[:]=[p for p in parts if not p['name'].startswith('armor_seam_')]
    for side,prefix in [(-1,'right'),(1,'left')]:
        by[prefix+'_foot']['sections']=[[side*22,0,-14,9,16],[side*22,-9,-12,8,14]]
        by[prefix+'_hand']['sections']=[[by[prefix+'_hand']['sections'][0][0],-92,-12,6,6],[by[prefix+'_hand']['sections'][0][0],-104,-12,6,6]]
        by[prefix+'_pauldron'].update(type='stone_block',center=[side*30,-120,0],size=[23,22,20],bevel=4)
        by[prefix+'_brow'].update(size=[10,3,3],center=[side*7,-146,-25])
        for j,y in enumerate([-116,-105,-94]):parts.append(part(f'breast_lap_{prefix}_{j}','stone_block','armor_shadow',0,role='armor_plate',center=[side*8,y,-17],size=[15-j*1.5,6,3],bevel=1))
    for mat in ['skin','skin_shadow','armor','armor_shadow','gray_armor','cloth','wood','wood_dark','iron','iron_edge','leather']:
        old=r['materials'][mat];family='organic'if mat.startswith('skin')else'metal'if mat in ['armor','armor_shadow','gray_armor','iron','iron_edge']else'wood'if mat.startswith('wood')else'leather';r['materials'][mat]=material(family,hsl=old['hsl'],shadow=max(2,old['hsl']-5),end=old['hsl']+3,edge=old['hsl']+5)
    # Head/neck and equipment remain explicit group ownership; no source rig or
    # original transform arrays are imported into the independent authoring path.
    # An explicit grip socket controls the shield plane and preserves attachment.
    angle=math.radians(36);co,si=math.cos(angle),math.sin(angle);basis=[[co,0,-si],[0,1,0],[si,0,co]];pivot=[68,-98,-12];origin=[pivot[a]-sum(pivot[i]*basis[i][a]for i in range(3))for a in range(3)]
    for p in parts:
        if p['name'].startswith('shield_plank_'):p['parent_socket']=dict(origin=origin,basis=basis)
    joints={j['name']:j['origin']for j in r['rig']};limbs=[]
    for prefix,offset in [('right',0),('left',.5)]:limbs.append(dict(name=prefix,hip=joints[prefix+'_hip'],knee=joints[prefix+'_knee'],tip=joints[prefix+'_ankle'],components=[prefix+'_foot'],phase_offset=offset,knee_axis=0))
    gait=dict(kind='biped',limbs=limbs,stride=24,lift=8,stance_fraction=.62,crouch=3,cycle_ticks=48);r['locomotion']=gait;r['actions']['walk']['gait']=gait;r['actions']['walk'].pop('foot_contacts',None)
    r['authoring_schema']=2;return r
