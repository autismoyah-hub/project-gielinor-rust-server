"""Original, reference-free examples exercising the reusable authoring controls.

These recipes read no game geometry, reference measurements or prior studies.
They are tooling demonstrations, not proposed Tower gameplay additions.
"""
import json,sys
from pathlib import Path
def op(action,**kw):return dict(op=action,**kw)
def reed_runner(muzzle_length=1.0):
    ops=[op('loft',name='runner_chest_and_tapered_waist',stations=[[-53,13,49,81],[-33,24,26,88],[0,20,30,76],[27,17,34,72],[47,22,29,77],[58,12,40,67]],material='hide')]
    for side in [-1,1]:
        for rear in [False,True]:
            y=42 if rear else -29
            ops +=[op('limb',name=f'runner_leg_{side}_{rear}',points=[[side*16,y,52],[side*22,y+5,29],[side*20,y-2,6]],radii=[[10,12],[6,7],[4,5]],material='hide'),op('loft',name=f'runner_pad_{side}_{rear}',stations=[[y-12,5,0,8,side*20],[y-4,8,0,10,side*20],[y+4,4,2,9,side*20]],material='dark')]
    ops +=[op('fuse',target='runner_chest|runner_leg',name='runner_connected_anatomy',material='hide',voxel=1.7,triangles=700),op('loft',name='runner_skull_and_muzzle',section='cranium',stations=[[-91*muzzle_length,7,64,74],[-77*muzzle_length,10,65,79],[-58,17,67,94],[-46,13,64,88]],material='hide'),op('tube',name='runner_tail',points=[[0,54,59],[0,76,62],[8,96,76]],radii=[7,5,1],material='accent',sides=8),op('patch',target='runner_connected_anatomy',name='fitted_ceramic_saddle',side=1,axis=2,outline=[[-12,-12],[-13,14],[0,23],[13,14],[12,-12],[0,-20]],material='plate',offset=1.2),op('landmark',target='runner_skull',source=[0,-91*muzzle_length,70],destination=[0,-91*muzzle_length,75],radius=[15,22,15])]
    return dict(name='Original Reed Runner anatomy and fitted plate',materials={'hide':18083,'dark':6502,'accent':28490,'plate':31428},operations=ops)
def harvesting_blade():
    return dict(name='Original bronze harvesting bill',materials={'metal':6045,'edge':7059,'wood':4387,'guard':8111},operations=[op('blade',name='hooked_harvesting_blade',stations=[[22,0,5,3],[1,0,8,3],[-24,8,12,2.8],[-48,24,13,2.3],[-67,43,10,1.8],[-70,62,5,1],[-61,75,.5,.4]],material='metal',edge='edge',height=5),op('box',name='forged_guard',c=[0,24,5],s=[25,6,10],material='guard'),op('loft',name='octagonal_wood_grip',stations=[[27,4,1,9],[57,4,1,9]],material='wood',section='cranium'),op('box',name='butt_cap',c=[0,59,5],s=[11,5,10],material='guard')])
if __name__=='__main__':
    out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
    for name,recipe in [('reed-runner',reed_runner()),('reed-runner-long-muzzle',reed_runner(1.15)),('harvesting-bill',harvesting_blade())]:(out/(name+'.json')).write_text(json.dumps(recipe,indent=2))
