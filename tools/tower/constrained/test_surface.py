"""Headless Blender checks on synthetic, reference-free construction only."""
import sys,json
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
import bpy,bmesh
from mathutils import Vector
import surface,cookbook
bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
mat=surface.native_color('Synthetic material',18083)
o=surface.mesh('brush locality',[(0,0,0),(1,0,0),(3,0,0),(0,1,0)],[(0,1,3)],mat)
outside=o.data.vertices[2].co.copy();surface.masked_deform([o],[0,0,0],[2,2,2],delta=[1,0,0]);assert o.data.vertices[2].co==outside;assert abs(o.data.vertices[0].co.x-1)<1e-6
recipe=cookbook.harvesting_blade();surface.remove('.*');surface.execute(recipe)
blade=surface.select('hooked_harvesting_blade')[0];vectors=[]
for i in range(len(recipe['operations'][0]['stations'])):
    vectors.append((blade.data.vertices[i*4+2].co-blade.data.vertices[i*4].co).normalized())
dots=[a.dot(b)for a,b in zip(vectors,vectors[1:])];assert min(dots)>0,'Curved sweep section flipped at hook bend'
bm=bmesh.new();bm.from_mesh(blade.data);assert all(e.is_manifold for e in bm.edges);bm.free()
result={'masked_brush_preserves_unselected_boundary':True,'curved_section_sign_is_continuous':True,'minimum_consecutive_section_dot':min(dots),'unquantized_blade_is_closed':True,'reference_geometry_read':False}
Path(sys.argv[-1]).write_text(json.dumps(result,indent=2));print('SYNTHETIC_SURFACE_TESTS_PASS')
