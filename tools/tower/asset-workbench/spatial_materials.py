"""Explicit native face-color brushes on real owned geometry.

Original recipes use authored brush positions/colors. Reference-derived brush
descriptors and reconstructed assets must remain private. No raster painting,
texture extraction, PBR conversion or global lighting change occurs here.
"""
import math

def paint(vertices,faces,colors,zones):
    if not zones:return colors
    for z in zones:
        if len(z['center'])!=3 or not all(math.isfinite(v)for v in z['center'])or not math.isfinite(z['radius'])or z['radius']<=0:raise ValueError('Native color brush needs a finite center and positive radius')
        if not isinstance(z['hsl'],int)or not 0<=z['hsl']<=65535:raise ValueError('Native brush HSL range')
    result=[]
    for f,base in zip(faces,colors):
        center=[sum(vertices[i][a]for i in f)/3 for a in range(3)];zone=min(zones,key=lambda z:sum((center[a]-z['center'][a])**2 for a in range(3))/z['radius']**2);distance=sum((center[a]-zone['center'][a])**2 for a in range(3))
        result.append(zone['hsl']if distance<=zone['radius']**2 else base)
    return result
