"""Blender geometry regressions for the reusable authoring fixes."""
import sys,json
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
from anatomy import *
bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
m=material('test',(.3,.4,.2))
points=[(0,0,100),(0,8,75),(0,0,50),(0,-8,25),(0,0,0)]
o=landmark_limb('singularity crossing',points,[(7,8)]*5,m)
normals=[(o.data.vertices[i*10].co-Vector(p)).normalized()for i,p in enumerate(points)]
dots=[a.dot(b)for a,b in zip(normals,normals[1:])];assert min(dots)>.99,dots
old=[]
for i,p in enumerate(points):
    t=(Vector(points[min(i+1,4)])-Vector(points[max(i-1,0)])).normalized();a=t.cross(Vector((0,0,1)))
    if a.length<.1:a=t.cross(Vector((0,1,0)))
    old.append(a.normalized())
old_dots=[a.dot(b)for a,b in zip(old,old[1:])];assert min(old_dots)<0,'Adversarial path must expose the old frame flip'
skull=cranial_planes('test skull',[(-10,8,0,10),(0,12,0,14),(10,8,0,10)],m)
patch=fitted_patch('surface fit',skull,1,[(-2,8),(2,8),(2,6),(-2,6)],m)
assert all(v.co.x>0 for v in patch.data.vertices)
try:fitted_patch('intentional miss',skull,1,[(-2,80),(2,80),(2,60)],m)
except ValueError:miss_rejected=True
else:raise AssertionError('Missing facial surface must fail explicitly')
report=dict(parallel_transport_consecutive_dot=dots,old_frame_consecutive_dot=old_dots,facial_patch_fit=True,facial_patch_miss_rejected=miss_rejected)
Path(sys.argv[-1]).write_text(json.dumps(report,indent=2));print('AUTHORING_HELPERS_VERIFIED')
