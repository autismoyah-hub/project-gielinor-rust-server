"""Small original scene assembly; uses the existing author/native renderer."""
import sys,json,subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/'asset-workbench'))
from construction import asset
from templates import part
from production_designs import provenance

def run(root):
    repo=HERE.parents[2];experiment=repo.parent;cache=experiment.parents[1]/'resource-staging/server/work/integration/a-kotlin/.data/cache/js5'
    c=[]
    for x in range(-3,4):
        for z in range(-2,3):
            if (x,z)in[(3,-2),(-3,-2)]:continue
            c.append(part(f'paving_{x}_{z}','stone_block','floor',center=[x*128,10,z*128],size=[126,20,126],bevel=2,variation=(x+2*z)%3-1))
    r=provenance(asset('Vignette paving — scene support only',c),'Original cutaway paving for the four-deliverable benchmark. Reuses existing masonry/scenery; not a fifth standalone asset study.')
    r['materials']['floor']=dict(hsl=10273,render_type=1)
    shell=root/'vignette';shell.mkdir(exist_ok=True);recipe=shell/'floor-recipe.json';recipe.write_text(json.dumps(r,indent=2))
    cmd=[sys.executable,str(HERE.parent/'asset-workbench/author_workflow.py'),'--recipe',str(recipe),'--out',str(shell/'floor'),'--native',str(root/'tools/asset_workbench.exe'),'--cache',str(cache),'--blender',r'C:\Program Files\Blender Foundation\Blender 5.2\blender.exe','--bounds',str(root/'private/reference/original-id-bounds.json'),'--identities',str(root/'identities.json'),'--asset-key','astra-benchmark:scene-paving']
    subprocess.run(cmd,check=True)
    old=repo/'art/tower/advanced-mastery/assets';rows=['light\t64\t768'];placements=[]
    def add(name,path,x,y,z,q=0,action=None):
        rows.append('\t'.join(['owned',str(path),str(x),str(y),str(z),str(q),str(action)if action else'-','0']));placements.append(dict(label=name,path=str(path),position=[x,y,z],quarter_turns=q))
    add('Original scene paving',shell/'floor/asset.model',0,0,0)
    add('B: Riveting station',root/'v2/B-riveter/asset.model',-180,0,-100)
    add('C: Counterweighted screw press',root/'v3/C-stamp/asset.model',-210,0,140)
    add('D: Sootsnout',root/'v2/D-sootsnout/asset.model',95,0,-65,0,root/'v2/D-sootsnout/idle')
    for x in [-280,280]:add('Existing original wall bay',old/'wall-bay/asset.model',x,0,280)
    add('Existing original service arch',old/'worker-threshold/asset.model',0,0,280)
    add('Existing original wall return',old/'wall-bay/asset.model',-415,0,75,1)
    add('Existing original supply barrel',old/'supply-barrel/asset.model',320,0,195)
    add('Existing original supply barrel',old/'supply-barrel/asset.model',345,0,110)
    add('Existing original lantern',old/'service-lantern/asset.model',-350,0,230)
    add('Existing original stock rack',old/'stock-rack/asset.model',230,0,245)
    scene=shell/'scene.tsv';scene.write_text('\n'.join(rows)+'\n')
    cameras={'overview':'look 900 -1100 -1300 -15 -65 45 1100 1200 900','player-west':'player -100 -70 -30 2000 15500 420','player-east':'player 70 -65 -10 2050 1100 450','creature-close':'look 440 -300 -520 80 -53 -65 850 900 700'}
    for name,cam in cameras.items():
        camera=shell/(name+'.camera');camera.write_text(cam);subprocess.run([str(root/'tools/asset_workbench.exe'),str(cache),str(shell/'views'/name),'scene',str(scene),str(camera)],check=True)
    (shell/'manifest.json').write_text(json.dumps(dict(placements=placements,cameras=cameras,production_integration=False,authentic_cache_geometry_in_scene=False),indent=2))

if __name__=='__main__':run(Path(sys.argv[1]).resolve())
