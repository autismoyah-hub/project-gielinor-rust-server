"""Blender saved-source shell export in its recorded component order."""
from pathlib import Path
import bpy,sys,json
T=Path(__file__).resolve().parent;sys.path[:0]=[str(T.parent/'mesh-study'),str(T.parent/'asset-workbench')]
from mesh_tools import export
from native_codec import model
a=sys.argv[sys.argv.index('--')+1:];source,out=map(Path,a)
names=[q['name']for q in json.loads((source/'conversion.json').read_text())['components']]
objects=[bpy.data.objects[name]for name in names]
assert all(not q.get('reused_asset')for q in objects)
g=export(objects,out,model);g['vertex_skins']=None;(out/'asset.model').write_bytes(model(g));(out/'geometry.json').write_text(json.dumps(g))
print('REEXPORTED_DESIGN_SHELL',source.name,len(objects),len(g['faces']),flush=True)
