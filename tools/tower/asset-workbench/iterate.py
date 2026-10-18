"""Repeatable local refinement with preserved proposals and explicit regressions."""
import argparse,json,sys,subprocess,hashlib
from pathlib import Path
from diagnose import diagnose
from refine import propose
HERE=Path(__file__).resolve().parent
def metrics(directory):
    data=json.loads((directory/'comparison/comparison.json').read_text());return dict(mean_iou=sum(v['iou']for v in data.values())/len(data),mean_rgb_error=sum(v['mean_rgb_absolute_error']for v in data.values())/len(data),views={k:v['iou']for k,v in data.items()})
def main():
    p=argparse.ArgumentParser();p.add_argument('--baseline',type=Path,required=True);p.add_argument('--reference',type=Path,required=True);p.add_argument('--bindings',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--native',type=Path,required=True);p.add_argument('--cache',type=Path,required=True);p.add_argument('--blender',type=Path,required=True);p.add_argument('--bounds',type=Path,required=True);p.add_argument('--identities',type=Path,required=True);p.add_argument('--asset-key',required=True);p.add_argument('--iterations',type=int,default=4);a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True);reference=json.loads((a.reference/'geometry.json').read_text());bindings=json.loads(a.bindings.read_text());current=a.baseline.resolve();history=[dict(path=str(current),decision='baseline',metrics=metrics(current))]
    for i in range(1,a.iterations+1):
        target=a.out/f'iteration-{i}';target.mkdir(parents=True,exist_ok=True);g=json.loads((current/'geometry.json').read_text());report=diagnose(reference,g,bindings);patch=propose(report,bindings,g);(target/'diagnostics-before.json').write_text(json.dumps(report,indent=2));(target/'proposal.json').write_text(json.dumps(patch,indent=2));
        if not patch['operations']:break
        command=[sys.executable,HERE/'author_workflow.py','--recipe',current/'recipe.json','--out',target/'authored','--blend',current/'asset.blend','--patch',target/'proposal.json','--reference',a.reference,'--native',a.native,'--cache',a.cache,'--blender',a.blender,'--bounds',a.bounds,'--identities',a.identities,'--asset-key',a.asset_key,'--skip-motion'];subprocess.run([str(x)for x in command],check=True)
        before,after=metrics(current),metrics(target/'authored');regressions=[k for k in before['views']if after['views'][k]<before['views'][k]-.04];retained=after['mean_iou']>before['mean_iou']+.002 and not regressions and after['mean_rgb_error']<=before['mean_rgb_error']*1.06
        history.append(dict(path=str((target/'authored').resolve()),decision='diagnostically retained for visual review'if retained else'rejected diagnostic regression; preserved',parent=str(current),metrics=after,regressed_views=regressions,patch_sha256=hashlib.sha256((target/'proposal.json').read_bytes()).hexdigest(),artistic_approval=False))
        if retained:current=(target/'authored').resolve()
        else:break
    (a.out/'history.json').write_text(json.dumps(dict(selected_for_review=str(current),iterations=history,limits='Bounds/PCA/palette descriptors guide local edits; numeric retention never means artistic approval.'),indent=2));print('REFINEMENT_HISTORY_COMPLETE',a.out/'history.json')
if __name__=='__main__':main()
