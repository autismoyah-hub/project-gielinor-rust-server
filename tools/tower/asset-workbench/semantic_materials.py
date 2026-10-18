"""Semantic native face materials for owned geometry; no Blender PBR inference.

Construction families carry a restricted packed-HSL palette and explicit surface
roles. Geometry, local face orientation and component variation choose a native
face color. Original imports never pass through this module.
"""
import math
from spatial_materials import paint

PALETTES={
 'wood':dict(hsl=8600,shadow=8479,end=8487,edge=8605),
 'stone':dict(hsl=10282,shadow=10272,end=10288,edge=10292),
 'metal':dict(hsl=33,shadow=24,end=41,edge=45),
 'bronze':dict(hsl=6687,shadow=6678,end=6695,edge=6698),
 'organic':dict(hsl=6697,shadow=6685,end=6704,edge=6701),
 'ceramic':dict(hsl=4773,shadow=4763,end=4781,edge=4786),
 'leather':dict(hsl=3741,shadow=3730,end=3747,edge=3744),
}

def material(family,**overrides):
    if family not in PALETTES:raise ValueError('Unknown native material family '+family)
    return {'family':family,'render_type':0,**PALETTES[family],**overrides}

def library():return {k:material(k)for k in PALETTES}

def shift_lightness(hsl,amount):return (int(hsl)&65408)|max(2,min(126,(int(hsl)&127)+int(amount)))

def colors(vertices,faces,mat,spec):
    if not mat.get('family'):return paint(vertices,faces,[mat['hsl']]*len(faces),spec.get('material_zones'))
    base=shift_lightness(mat['hsl'],spec.get('variation',0));axis=spec.get('grain_axis',[0,1,0]);length=math.sqrt(sum(x*x for x in axis))
    if length==0:raise ValueError('Grain axis must have a direction')
    axis=[x/length for x in axis];result=[];middle_y=(min(v[1]for v in vertices)+max(v[1]for v in vertices))/2
    for f in faces:
        a,b,c=[vertices[i]for i in f];u=[b[i]-a[i]for i in range(3)];v=[c[i]-a[i]for i in range(3)];n=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]];ln=math.sqrt(sum(x*x for x in n));n=[x/ln for x in n]if ln else [0,0,0];h=base
        if mat['family']=='wood'and abs(sum(n[i]*axis[i]for i in range(3)))>.92:h=mat['end']
        elif mat['family']in ['stone','metal','bronze','ceramic']:
            if abs(n[1])>.65:h=mat['end']if sum(vertices[i][1]for i in f)/3<=middle_y else mat['shadow']
            elif .16<abs(n[1])<.65:h=mat['edge']
        # Authored broad surface bands; never a noisy per-triangle random shader.
        for band in spec.get('surface_bands',[]):
            center=sum(vertices[i][band.get('axis',1)]for i in f)/3
            if band['min']<=center<=band['max']:h=band['hsl']
        result.append(h)
    return paint(vertices,faces,result,spec.get('material_zones'))
