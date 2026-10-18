"""Original compact pump-yard ground; generated only for presentation context."""
import sys,json,math
from pathlib import Path
HERE=Path(__file__).resolve().parent;sys.path[:0]=[str(HERE),str(HERE.parent/'asset-workbench')]
from mesh_tools import *
from native_codec import model
out=Path(sys.argv[-1]).resolve();bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
earth=material('Compacted ash',(.25,.245,.21),True);stone=[material('Yard paving '+str(i),(.34+i*.012,.35+i*.01,.32+i*.008),True)for i in range(4)]
# Local skirt cells avoid broad painter-order faces and follow the tile outline.
for x in range(-4,5):
    for y in range(-3,4):
        if abs(x)==4 and abs(y)==3:continue
        cx=x*80+((y%2)*8);cy=y*80
        mesh('foundation_cell',[(cx+a,cy+b,z)for z in [-29,-5]for a,b in [(-39,-39),(39,-39),(39,39),(-39,39)]],[(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7),(0,3,2,1)],earth)
        o=box('paver',(x*80+((y%2)*8),y*80,-4),(77,77,8),stone[(x+2*y)%4],2)
        for v in o.data.vertices:v.co.z+=.6*math.sin(v.co.x+x*3)
dark=material('Gutter iron',(.16,.18,.18));box('drain_bed',(298,-20,1),(30,280,3),dark,1)
for y in range(-150,120,12):box('grate_crossbar',(298,y,4),(29,4,4),stone[0],1)
objs=[o for o in bpy.context.scene.objects if o.type=='MESH'];export(objs,out,model);bpy.ops.wm.save_as_mainfile(filepath=str(out/'source.blend'))
