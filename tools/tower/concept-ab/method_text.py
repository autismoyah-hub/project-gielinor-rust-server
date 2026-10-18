"""Text-only Ashhorn construction. Written and frozen before concept generation.

Factory-empty inputs. No image readers or imports from the competing run.
"""
import json,sys,colorsys
from pathlib import Path
def hsl(rgb):
    h,l,s=colorsys.rgb_to_hls(*rgb);return (int(h*64)<<10)|(min(7,int(s*8))<<7)|max(2,int(l*127))
def build(spec,stage):
    ops=[]
    def add(action,**kw):ops.append(dict(op=action,**kw))
    def loft(name,stations,mat='hide',**kw):add('loft',name=name,stations=stations,material=mat,**kw)
    def tube(name,points,radii,mat='hide',sides=8):add('tube',name=name,points=points,radii=radii,material=mat,sides=sides)
    loft('anatomy_torso',[[-77,18,80,121],[-58,32,58,132],[-35,36,49,131],[-4,27,58,119],[22,21,66,111],[57,27,61,112],[78,18,70,104],[84,8,76,94]])
    loft('anatomy_neck',[[-103,18,91,126],[-82,24,74,129],[-59,28,68,132]])
    for s in [-1,1]:
        add('limb',name=f'anatomy_fore_{s}',points=[[s*24,-47,105],[s*34,-37,76],[s*33,-31,55],[s*34,-43,19],[s*34,-46,9]],radii=[[18,20],[14,16],[10,11],[6,8],[7,8]],material='hide')
        add('limb',name=f'anatomy_hind_{s}',points=[[s*22,60,89],[s*31,47,69],[s*33,40,54],[s*31,85,28],[s*31,73,10]],radii=[[17,19],[15,16],[10,11],[5.5,6],[6,7]],material='hide')
    if stage>0:
        add('fuse',target='^anatomy_',name='continuous_species_anatomy',voxel=1.5,triangles=1600,material='hide')
        add('sculpt',target='^continuous',center=[0,-43,98],radius=[49,53,44],scale=[1.035,1,1.025])
        add('relax',target='^continuous',center=[0,-65,111],radius=[30,29,24],iterations=2,strength=.18)
        add('box_region',target='^continuous',bounds=[[-29,29],[-70,66],[0,68]],material='ventral')
    loft('skull',[[ -140,12,103,115],[-131,16,102,120],[-117,17,103,128],[-104,22,101,133],[-89,22,102,130],[-77,18,99,122]],section='cranium')
    loft('lower_jaw',[[-138,11,94,100],[-124,15,91,102],[-105,17,86,102],[-88,17,91,106]],'ventral',section='cranium')
    loft('mouth_cavity',[[-136,11,99,104],[-117,15,98,105],[-97,13,98,105]],'dark',section='cranium')
    for s in [-1,1]:
        for rear in [False,True]:
            x=s*(31 if rear else 34);y=68 if rear else -49;w=10 if rear else 12
            loft(f'paw_{s}_{rear}',[[y-15,w*.7,1,11,x],[y-5,w,0,16,x],[y+9,w*.65,3,16,x]],'dark')
            if stage>0:
                for toe in [-1,0,1]:
                    tx=x+toe*7
                    loft(f'toe_{s}_{rear}_{toe}',[[y-20,2.5,1,7,tx],[y-14,3.5,0,10,tx],[y-6,3.5,2,12,tx]],'hide',section='cranium')
                    tube(f'claw_{s}_{rear}_{toe}',[[tx,y-17,5],[tx,y-22,6],[tx,y-27,1]],[3,2.5,.35],'claw',6)
        add('shell',name=f'ear_{s}',outline=[[-83,124],[-67,145],[-62,130],[-76,119]],depth=5,offset=[s*23,0,0],axis='X',material='hide')
        if stage>0:
            add('shell',name=f'ear_inner_{s}',outline=[[-79,127],[-68,140],[-66,132]],depth=1,offset=[s*26,0,0],axis='X',material='ventral')
            # The eye and brow are ray-fitted to the authored skull.
            add('patch',target='^skull$',name=f'orbital_socket_{s}',side=s,outline=[[-119,117],[-111,123],[-100,120],[-108,113]],material='dark',offset=1.2)
            add('patch',target='^skull$',name=f'eye_{s}',side=s,outline=[[-115,118],[-109,120],[-105,118],[-110,116]],material='eye',offset=1.8)
            tube(f'brow_{s}',[[s*17,-121,122],[s*22,-110,126],[s*22,-98,124]],[3,4,3],'hide',6)
            loft(f'cheek_{s}',[[-116,4,101,114,s*16],[-101,8,96,119,s*19],[-86,4,100,118,s*17]],'hide',section='cranium')
    tube('long_left_horn',[[-19,-91,125],[-25,-87,143],[-30,-75,160],[-30,-54,176]],[10,8,5,.6],'horn',10)
    tube('broken_right_horn',[[19,-91,125],[24,-87,140],[25,-82,149]],[10,8,7],'horn',10)
    if stage>0:
        tube('fracture_cap',[[25,-82,149],[25,-81,151]],[7,6],'dark',7)
        for i,(p,q,r) in enumerate([([-22,-89,135],[-23,-88,138],9),([-27,-83,150],[-28,-81,153],7),([-30,-70,164],[-30,-67,167],4)]):tube(f'horn_growth_band_{i}',[p,q],[r,r*.91],'dark',10)
    tube('tail',[[0,77,84],[1,106,76],[8,137,65],[17,167,65],[26,190,81]],[9,7,5,3,1],'hide',10)
    if stage>0:
        for i,(y,w,z) in enumerate([(-48,27,134),(-23,24,128),(3,20,119),(28,18,114),(55,19,114)]):
            loft(f'dorsal_plate_{i}',[[y-17,w*.45,z-5,z+2],[y-5,w,z-6,z+9],[y+13,w*.62,z-9,z+1]],'plate',section='cranium')
            loft(f'dorsal_worn_edge_{i}',[[y-18,w*.42,z-4,z],[y-14,w*.66,z-4,z+4]],'plate_edge',section='cranium')
        for s in [-1,1]:
            add('shell',name=f'shoulder_cap_{s}',outline=[[-66,107],[-58,128],[-36,123],[-20,101],[-34,87],[-50,92]],depth=5,offset=[s*34,0,0],axis='X',material='plate')
            add('shell',name=f'shoulder_lip_{s}',outline=[[-50,91],[-33,86],[-20,100],[-24,103],[-35,91]],depth=6,offset=[s*34,0,0],axis='X',material='plate_edge')
        loft('nose_pad',[[-142,10,104,113],[-137,12,103,115]],'dark',section='cranium')
    if stage>=2:
        add('sculpt',target='^continuous',center=[0,23,86],radius=[34,37,31],scale=[.94,1,1])
        add('sculpt',target='^skull$',center=[0,-127,113],radius=[22,24,18],scale=[1.06,1,.95])
        add('sculpt',target='^continuous',center=[0,66,88],radius=[40,31,35],scale=[1.04,1,1])
        for s in [-1,1]:
            tube(f'scapular_crease_{s}',[[s*29,-19,99],[s*26,-10,85],[s*22,-4,74]],[1.6,1.4,.4],'dark',5)
            tube(f'jaw_ridge_{s}',[[s*12,-135,96],[s*17,-117,92],[s*18,-99,94]],[1.7,2.3,2.3],'ventral',6)
    if stage>=3:
        # Native side/front inspection: cheeks were spherical, shoulder caps
        # penetrated the scapula, and the lower lip read as a broad smile.
        for o in ops:
            if o.get('name','').startswith('cheek_'):
                for row in o['stations']:row[1]*=.65
            if o.get('name')=='lower_jaw':
                for row in o['stations']:row[2]+=2;row[3]-=1;row[1]*=.92
            if o.get('name','').startswith('shoulder_cap_'):
                s=-1 if o['name'].endswith('-1') else 1
                o.clear();o.update(op='patch',target='^continuous_species_anatomy$',name=f'shoulder_cap_{s}',side=s,outline=[[-62,106],[-56,124],[-38,122],[-24,102],[-34,92],[-49,95]],material='plate',offset=2.8)
            if o.get('name','').startswith('shoulder_lip_'):o['offset'][0]*=1.12
        add('sculpt',target='^continuous',center=[0,-20,113],radius=[39,31,22],scale=[1,1,.96])
        for s in [-1,1]:
            add('patch',target='^skull$',name=f'muzzle_fold_{s}',side=s,outline=[[-134,109],[-125,110],[-119,107],[-129,106]],material='ventral',offset=.8)
    if stage>=4:
        # A final native pass removes floating crease tubes and fractured-looking
        # growth-band overlays. Boundaries are authored on the real horn surface.
        ops=[o for o in ops if not o.get('name','').startswith(('horn_growth_band_','jaw_ridge_','muzzle_fold_','shoulder_lip_','scapular_crease_'))]
        for o in ops:
            if o.get('name','').startswith('shoulder_cap_'):
                s=-1 if o['name'].endswith('-1') else 1
                o.clear();o.update(op='shell',name=f'shoulder_cap_{s}',outline=[[-63,109],[-56,126],[-38,123],[-23,104],[-34,92],[-50,96]],depth=5,offset=[s*41,0,0],axis='X',material='plate')
            if o['op']=='box_region':o['bounds'][2]=[49,69]
        for lo,hi in [(134,139),(149,154),(164,168)]:add('band',target='^long_left_horn$',axis=2,range=[lo,hi],material='dark')
        for s in [-1,1]:
            add('shell',name=f'shoulder_worn_edge_{s}',outline=[[-51,97],[-34,93],[-24,104],[-27,106],[-35,97]],depth=5.5,offset=[s*41,0,0],axis='X',material='plate_edge')
    return {'name':f'Ashhorn text-only stage {stage}','frozen_spec_sha256':'ce0cb460a95c8b5a62c1a561f40bf21e4571befd9b1cd55ef2b03dc3937518f8','materials':{k:hsl(v) for k,v in spec['palette_rgb'].items()},'operations':ops}
if __name__=='__main__':
    spec,out,stage=sys.argv[1:];Path(out).write_text(json.dumps(build(json.loads(Path(spec).read_text()),int(stage)),indent=2)+'\n')
