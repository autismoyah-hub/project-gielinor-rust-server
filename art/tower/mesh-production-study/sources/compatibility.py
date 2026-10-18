"""Controlled UV-loss and native-shading tests, entirely original test geometry."""
import sys,json,math
from pathlib import Path
HERE=Path(__file__).resolve().parent;sys.path[:0]=[str(HERE),str(HERE.parent/'asset-workbench')]
import bpy
from mesh_tools import *
from native_codec import model
out=Path(sys.argv[-1]).resolve();out.mkdir(parents=True,exist_ok=True)
bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
mat=material('Original four-color UV checker',(.4,.4,.4));mat.use_nodes=True
im=bpy.data.images.new('Authored albedo calibration',width=64,height=64)
colors=[(.58,.29,.12,1),(.24,.43,.35,1),(.62,.57,.42,1),(.17,.20,.23,1)]
im.pixels=[v for y in range(64)for x in range(64)for v in colors[(x//8+y//8)%4]];im.pack()
node=mat.node_tree.nodes.new('ShaderNodeTexImage');node.image=im;mat.node_tree.links.new(node.outputs['Color'],mat.node_tree.nodes.get('Principled BSDF').inputs['Base Color'])
results={}
for name,n,sample in [('material-only',16,False),('coarse-UV-bake',1,True),('subdivided-UV-bake',16,True)]:
    vs=[(-96+x*192/n,0,z*160/n)for z in range(n+1)for x in range(n+1)];fs=[]
    for z in range(n):
        for x in range(n):i=z*(n+1)+x;fs.append((i,i+1,i+n+2,i+n+1))
    o=mesh(name,vs,fs,mat);layer=o.data.uv_layers.new(name='UVMap')
    for p in o.data.polygons:
        for loop in p.loop_indices:
            v=o.data.vertices[o.data.loops[loop].vertex_index].co;layer.data[loop].uv=((v.x+96)/192,v.z/160)
    results[name]=export([o],out/name,model,sample_images=sample);o.hide_render=True
im.filepath_raw=str(out/'original-albedo.png');im.file_format='PNG';im.save()
bpy.ops.wm.save_as_mainfile(filepath=str(out/'uv-source.blend'))
print('UV_CONTROL_COMPLETE')
