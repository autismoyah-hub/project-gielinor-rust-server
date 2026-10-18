"""Purpose-built original maintenance yard; no cache textures or meshes."""
import sys,math,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;sys.path[:0]=[str(HERE.parent/'mesh-study'),str(HERE.parent/'asset-workbench')]
import bpy
from mesh_tools import *
from native_codec import model
out=Path(sys.argv[-1]).resolve();bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
ash=material('Compacted workshop ash',(.27,.265,.23),True);pavers=[material('Stone floor '+str(i),(.36+i*.012,.37+i*.01,.33+i*.008),True)for i in range(4)];iron=material('Gutter iron',(.17,.20,.20));rubble=material('Scoria',(.25,.27,.25),True)
for x in range(-4,5):
    for y in range(-3,4):
        if abs(x)==4 and abs(y)==3:continue
        cx=x*80;cy=y*80
        # Every local top owns its native depth order; no broad hidden slab.
        box('earth_cell',(cx,cy,-15),(80,80,26),ash,0)
        paved=(abs(x)<=1)or (x<0 and -1<=y<=2)or y>=2
        if paved:
            o=box('worn_paver',(cx,cy,-2),(77,77,8),pavers[(x+2*y)%4],2)
            if (x,y)in [(-3,-1),(1,1),(-1,-2)]:
                for v in o.data.vertices:
                    if v.co.x>25 and v.co.y<-20:v.co.x-=4
        if x>=2 and y<=0:
            # Sparse scoria on the unpaved creature approach, not uniform noise.
            for i in range(2):
                xx=cx-16+i*32;yy=cy+18*(1 if(x+i)%2 else-1)
                ellipsoid('ash_fragment',(xx,yy,2),(4+i*2,7-i,3),rubble,segments=6,rings=4)
box('drain_recess',(-300,-5,1),(29,290,4),iron,1)
for y in range(-145,140,13):box('drain_crossbar',(-300,y,5),(27,4,4),pavers[1],1)
objs=[o for o in bpy.context.scene.objects if o.type=='MESH'];export(objs,out,model);bpy.ops.wm.save_as_mainfile(filepath=str(out/'source.blend'))
