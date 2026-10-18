"""Independent factory-empty construction from the accepted original concept.

No text-run imports, models, recipes, or mesh components. Semantic landmarks
come from the shared frozen specification and concept/consistency.json.
"""
from pathlib import Path
import json,sys,colorsys
def color(rgb):
    h,l,s=colorsys.rgb_to_hls(*rgb)
    return (int(h*64)<<10)|(min(7,int(s*8))<<7)|max(2,int(l*127))
def build(spec,stage):
    ops=[]
    def put(op,**kw):ops.append(dict(op=op,**kw))
    def profile(name,rows,material='hide',**kw):put('loft',name=name,stations=rows,material=material,**kw)
    def path(name,points,radii,material='hide',sides=8):put('tube',name=name,points=points,radii=radii,material=material,sides=sides)
    profile('mass_ribcage_lumbar_pelvis',[[-75,19,79,120],[-61,30,59,130],[-46,36,49,132],[-26,33,52,127],[-2,26,60,118],[22,21,66,111],[43,23,65,110],[63,28,61,112],[79,19,71,103],[85,7,81,92]])
    profile('mass_throat_scapular_neck',[[-101,17,91,121],[-86,22,77,125],[-72,26,66,130],[-53,31,62,132]])
    for side in [-1,1]:
        put('limb',name=f'mass_fore_{side}',points=[[side*23,-49,107],[side*31,-48,84],[side*33,-31,55],[side*34,-41,27],[side*34,-45,12]],radii=[[19,20],[17,16],[11,12],[7,8],[7,9]],material='hide')
        put('limb',name=f'mass_hind_{side}',points=[[side*23,58,88],[side*31,47,69],[side*33,40,54],[side*32,65,40],[side*31,86,27],[side*31,70,10]],radii=[[17,19],[16,16],[11,11],[7,9],[5,6],[6,8]],material='hide')
    if stage>0:
        put('fuse',target='^mass_',name='integrated_anatomy',voxel=1.4,triangles=1450,material='hide')
        put('relax',target='^integrated',center=[0,-68,104],radius=[39,38,34],iterations=2,strength=.16)
        put('region',target='^integrated',axes=[1,2],polygon=[[-88,90],[-62,66],[-36,48],[-8,59],[21,69],[54,63],[72,78],[63,87],[32,79],[1,77],[-35,77],[-66,95]],side_min=0,material='ventral')
    profile('cranial_keel_and_zygomatic_planes',[[-140,12,101,114],[-131,15,100,118],[-121,16,99,123],[-112,21,96,128],[-103,22,97,132],[-91,21,101,130],[-78,17,102,123]],section='cranium')
    profile('mandible',[[-139,10,93,100],[-126,14,88,99],[-108,17,84,100],[-91,15,93,106]],'ventral',section='cranium')
    profile('mouth_negative_space',[[-138,10,99,103],[-120,14,97,102],[-102,15,97,103]],'dark',section='cranium')
    for side in [-1,1]:
        put('shell',name=f'swept_ear_{side}',outline=[[-88,124],[-68,144],[-64,132],[-78,119]],depth=4,offset=[side*24,0,0],axis='X',material='hide')
        if stage>0:
            put('shell',name=f'ear_recess_{side}',outline=[[-81,127],[-68,139],[-68,132]],depth=1,offset=[side*26.3,0,0],axis='X',material='dark')
            put('patch',target='^cranial_',name=f'orbital_recess_{side}',side=side,outline=[[-122,118],[-111,125],[-101,121],[-109,113]],material='dark',offset=.8)
            put('patch',target='^cranial_',name=f'amber_eye_{side}',side=side,outline=[[-117,118],[-110,121],[-106,119],[-110,116]],material='eye',offset=1.6)
            put('patch',target='^cranial_',name=f'sculpted_brow_plane_{side}',side=side,outline=[[-124,122],[-111,128],[-99,126],[-105,122],[-115,124]],material='hide',offset=2.7)
            put('patch',target='^cranial_',name=f'zygomatic_ridge_{side}',side=side,outline=[[-117,109],[-104,116],[-94,113],[-101,105],[-110,101]],material='hide',offset=1.6)
    path('left_swept_horn',[[-19,-91,125],[-22,-90,134],[-24,-88,140],[-28,-82,150],[-30,-74,160],[-31,-64,170],[-30,-54,176]],[10,9,8,6.5,4.5,2.5,.4],'horn',10)
    path('right_fractured_horn',[[19,-91,125],[22,-88,136],[25,-83,146],[25,-82,150]],[10,9,8,7],'horn',9)
    if stage>0:
        for lo,hi in [(130,137),(146,153),(162,167)]:put('band',target='^left_swept_horn$',axis=2,range=[lo,hi],material='dark')
        put('band',target='^right_fractured_horn$',axis=2,range=[146,152],material='dark')
    path('tapered_tail',[[0,76,83],[2,103,76],[9,138,62],[22,169,65],[26,190,81]],[9,7,5,3,1],'hide',8)
    for side in [-1,1]:
        for kind,x,y,width in [('fore',side*34,-48,12),('hind',side*31,68,10)]:
            profile(f'{kind}_metapodial_{side}',[[y-13,width*.7,1,11,x],[y-3,width,0,16,x],[y+9,width*.6,3,16,x]],'hide',section='cranium')
            if stage>0:
                for digit in [-1,0,1]:
                    dx=x+digit*7
                    profile(f'{kind}_toe_{side}_{digit}',[[y-22,2.7,1,6,dx],[y-15,3.7,0,10,dx],[y-5,3.4,2,12,dx]],'hide',section='cranium')
                    path(f'{kind}_claw_{side}_{digit}',[[dx,y-19,6],[dx,y-24,5],[dx,y-28,1]],[3,2.3,.4],'claw',6)
    if stage>0:
        # Each dorsal plate follows this run's anatomy, not a copied candidate.
        for i,(y,w) in enumerate([(-48,28),(-23,25),(3,21),(28,18),(55,21)]):
            put('patch',target='^integrated_anatomy$',name=f'dorsal_shield_{i}',side=1,axis=2,outline=[[-w*.72,y-12],[-w*.3,y-17],[w*.3,y-17],[w*.72,y-12],[w,y+2],[w*.63,y+11],[0,y+17],[-w*.63,y+11],[-w,y+2]],material='plate',offset=3.4)
            put('patch',target='^integrated_anatomy$',name=f'dorsal_rust_lip_{i}',side=1,axis=2,outline=[[-w*.63,y+8],[0,y+14],[w*.63,y+8],[w*.63,y+11],[0,y+17],[-w*.63,y+11]],material='plate_edge',offset=3.8)
        for side in [-1,1]:
            put('patch',target='^integrated_anatomy$',name=f'shoulder_cap_upper_{side}',side=side,outline=[[-66,111],[-56,126],[-39,123],[-24,108],[-34,97],[-49,100]],material='plate',offset=3.0)
            put('patch',target='^integrated_anatomy$',name=f'shoulder_cap_lower_{side}',side=side,outline=[[-51,102],[-37,107],[-24,97],[-30,83],[-43,88]],material='plate',offset=3.4)
            put('patch',target='^integrated_anatomy$',name=f'shoulder_worn_rim_{side}',side=side,outline=[[-44,89],[-30,84],[-25,96],[-28,97],[-32,89]],material='plate_edge',offset=4.0)
        profile('nasal_pad',[[-142,10,103,113],[-137,12,103,116]],'dark',section='cranium')
        put('shading',target='dorsal_|shoulder_|brow_plane|zygomatic_',flat=True)
    if stage>=2:
        put('sculpt',target='^integrated',center=[0,23,89],radius=[35,34,32],scale=[.96,1,1])
        put('sculpt',target='^cranial_',center=[0,-131,111],radius=[20,25,18],scale=[1.04,1,.96])
        for side in [-1,1]:
            put('sculpt',target='^integrated',center=[side*30,-42,83],radius=[17,22,31],scale=[1.045,1.02,1])
            put('sculpt',target='^integrated',center=[side*29,49,70],radius=[18,25,27],scale=[1.035,1,1])
    return {'name':f'Ashhorn concept-guided stage {stage}','frozen_spec_sha256':'ce0cb460a95c8b5a62c1a561f40bf21e4571befd9b1cd55ef2b03dc3937518f8','materials':{k:color(v) for k,v in spec['palette_rgb'].items()},'operations':ops}
if __name__=='__main__':
    spec,out,stage=sys.argv[1:];Path(out).write_text(json.dumps(build(json.loads(Path(spec).read_text()),int(stage)),indent=2)+'\n')
