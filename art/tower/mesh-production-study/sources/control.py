"""Fresh current-workbench control for the same drake design brief."""
import sys,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;sys.path[:0]=[str(HERE.parent/'asset-workbench')]
from templates import quadruped,part
from author_blender import build,export_asset
from native_codec import model
import bpy
out=Path(sys.argv[-1]).resolve();out.mkdir(parents=True,exist_ok=True)
r=quadruped(88,100,220);r['rig']=[];r['actions']={}
r['materials']['skin'].update(hsl=17577)
r['materials']['skin_shadow'].update(hsl=17570)
r['materials']['horn']=dict(hsl=8485,render_type=0)
r['materials']['scute']=dict(hsl=5405,render_type=0)
for s in [-1,1]:r['components'].append(part('cheek_horn_'+str(s),'sweep','horn',sections=[[s*20,-85,-80,7,7],[s*32,-102,-68,5,5],[s*38,-106,-49,1,1]],normal=[0,0,1],sides=6))
for i in range(7):r['components'].append(part('back_plate_'+str(i),'loft','scute',sections=[[0,-79,-47+i*17,20,11],[0,-105,-47+i*17,2,7]],sides=4))
build(r,None);g=export_asset(r);(out/'geometry.json').write_text(json.dumps(g));(out/'asset.model').write_bytes(model(g));(out/'recipe.json').write_text(json.dumps(r,indent=2));bpy.ops.wm.save_as_mainfile(filepath=str(out/'source.blend'))
