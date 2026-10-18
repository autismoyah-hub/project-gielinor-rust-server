"""Bounded coordinate descent of meaningful recipe parameters, all views at once.

The search proposes candidates. It records per-view regressions and never grants
artistic acceptance. Recipes, reference images, masks and generated assets remain
private. No arbitrary vertex-coordinate optimization is supported.
"""
import argparse,copy,json,sys,subprocess,hashlib,time
from pathlib import Path
HERE=Path(__file__).resolve().parent
sys.path[:0]=[str(HERE),str(HERE.parent/'reassessment')]
from evaluate import compare
from compare import render,run

def get(recipe,path):
    x=recipe
    for k in path:x=x[k]
    return x
def put(recipe,path,value):
    x=recipe
    for k in path[:-1]:x=x[k]
    x[path[-1]]=value
def utility(metrics,baseline):
    regressions=sum(max(0,baseline['views'][v]['silhouette_iou']-m['silhouette_iou']-.015)for v,m in metrics['views'].items())
    return metrics['objective']+.12*metrics['worst_iou']-.45*regressions

def optimize(config,kind,source,recipe_file,controls_file,out,blender,budget=8):
    out=Path(out);out.mkdir(parents=True,exist_ok=False)
    config=json.loads(Path(config).read_text());case=config['cases'][kind];root=Path(config['root'])
    reference=root/'baseline'/kind/'authentic';previous=root/'baseline'/kind/'previous'
    base=compare(reference,previous);recipe=json.loads(Path(recipe_file).read_text());controls=json.loads(Path(controls_file).read_text())
    ledger=[];best=None;best_score=-1;best_recipe=copy.deepcopy(recipe);count=0
    candidates=[('initial',recipe)]
    # Each bounded trial starts from the currently selected complete recipe.
    for control in [None]+controls:
        values=[None] if control is None else [-1,1]
        parent=copy.deepcopy(best_recipe)
        for sign in values:
            if count>=budget:break
            candidate=copy.deepcopy(parent)
            if control:
                current=get(parent,control['path']);value=max(control['min'],min(control['max'],current+sign*control['step']))
                if value==current:continue
                put(candidate,control['path'],value);name=control['name']+('-'if sign<0 else'+')
            else:name='initial'
            folder=out/f'{count:02d}-{name}';file=out/f'{count:02d}-recipe.json';file.write_text(json.dumps(candidate,indent=2))
            run([blender,'-b','--factory-startup','--python-exit-code','1','--python',HERE/'surface.py','--',source,file,folder],out/f'{count:02d}-blender.log')
            render(config,case,folder/'asset.model',folder/'native');metrics=compare(reference,folder/'native',base)
            score=utility(metrics,base);selected=score>best_score
            entry={'candidate':folder.name,'reason':'bounded semantic parameter exploration','control':control,'direction':sign,'metrics':metrics,'utility':score,'provisional_metric_selection':selected,'artistic_acceptance':'pending visual review','recipe_sha256':hashlib.sha256(file.read_bytes()).hexdigest(),'model_sha256':hashlib.sha256((folder/'asset.model').read_bytes()).hexdigest()}
            (folder/'metrics.json').write_text(json.dumps(metrics,indent=2));ledger.append(entry)
            if selected:best_score=score;best=folder.name;best_recipe=candidate
            (out/'ledger.json').write_text(json.dumps(ledger,indent=2));(out/'selection.json').write_text(json.dumps({'provisional':best,'utility':best_score,'approval':False,'selected_recipe':best_recipe},indent=2))
            print(kind,folder.name,round(metrics['mean_iou'],4),'regressions',metrics['regressions'],'provisional',selected,flush=True);count+=1
    return best

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('config');p.add_argument('kind');p.add_argument('source');p.add_argument('recipe');p.add_argument('controls');p.add_argument('out');p.add_argument('--blender',required=True);p.add_argument('--budget',type=int,default=8);a=p.parse_args()
    optimize(a.config,a.kind,a.source,a.recipe,a.controls,a.out,a.blender,a.budget)
