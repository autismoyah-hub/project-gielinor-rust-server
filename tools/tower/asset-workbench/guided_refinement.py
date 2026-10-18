"""Preserved, targeted Blender edits with named feedback and multi-view review.

This is a development loop; it may not overwrite a sealed evaluation attempt.
"""
import argparse,json,subprocess,sys,hashlib
from pathlib import Path
from component_feedback import feedback
from iterate import metrics
HERE=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text())
def main(a):
    if a.out.exists()and any(a.out.iterdir()):raise ValueError('Use an empty history directory; frozen/superseded evidence is immutable')
    a.out.mkdir(parents=True,exist_ok=True);bindings=load(a.bindings);reference=load(a.reference/'geometry.json');current=a.baseline.resolve();history=[]
    for i in range(a.iterations):
        step=a.out/f'iteration-{i+1:02}';step.mkdir();report=feedback(reference,load(current/'geometry.json'),bindings);patch=report['proposal'];(step/'actionable-feedback.json').write_text(json.dumps(report,indent=2));(step/'patch.json').write_text(json.dumps(patch,indent=2))
        if not patch['operations']:history.append(dict(status='No eligible component edit; inspect missing construction',feedback=str(step/'actionable-feedback.json')));break
        out=step/'authored';command=[sys.executable,str(HERE/'author_workflow.py'),'--recipe',str(current/'recipe.json'),'--blend',str(current/'asset.blend'),'--patch',str(step/'patch.json'),'--out',str(out),'--reference',str(a.reference),'--native',str(a.native),'--cache',str(a.cache),'--blender',str(a.blender),'--bounds',str(a.bounds),'--identities',str(a.identities),'--asset-key',a.asset_key,'--skip-motion'];subprocess.run(command,check=True)
        before,after=metrics(current),metrics(out);pareto=all(after['views'][v]>=before['views'][v]for v in before['views'])and after['mean_rgb_error']<=before['mean_rgb_error'];improved=after['mean_iou']>before['mean_iou'];history.append(dict(iteration=i+1,parent=str(current),candidate=str(out),before=before,after=after,pareto_nonregression=pareto,selected_for_review=bool(pareto and improved),patch_sha256=hashlib.sha256((step/'patch.json').read_bytes()).hexdigest(),artistic_acceptance=False))
        if pareto and improved:current=out.resolve()
        else:break
    (a.out/'history.json').write_text(json.dumps(dict(iterations=history,selected_for_review=str(current),method='Curated component corrections; every view and color channel reviewed separately. Pareto retention is a development policy, not a universal artistic threshold.',artistic_acceptance=False),indent=2));print('GUIDED_COMPONENT_HISTORY_COMPLETE',len(history))
if __name__=='__main__':
    p=argparse.ArgumentParser()
    for name in ['baseline','reference','bindings','out','native','cache','blender','bounds','identities']:p.add_argument('--'+name,type=Path,required=True)
    p.add_argument('--asset-key',required=True);p.add_argument('--iterations',type=int,default=3);main(p.parse_args())
