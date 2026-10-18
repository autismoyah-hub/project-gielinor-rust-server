"""Frozen, descriptor-driven held-out evaluation with payload-overlap auditing."""
import argparse,json,subprocess,sys,hashlib
from pathlib import Path
from analyze import library
from instantiate import make

def evaluate(a):
    protocol=json.loads((a.out/'protocol.json').read_text());freeze=json.loads((a.out/'tool-freeze.json').read_text());here=Path(__file__).parent
    for name,digest in freeze.items():
        path=a.native if name=='native_binary_sha256'else here/name
        if hashlib.sha256(path.read_bytes()).hexdigest()!=digest:raise ValueError('Frozen tool changed before held-out evaluation: '+name)
    subprocess.run([str(a.native),str(a.cache),str(a.out/'reference'),'inspect',*protocol['archetypes']],check=True);library(a.out/'reference',a.out/'analysis')
    training=json.loads((a.training/'catalog.json').read_text());train_hashes={h for r in training for h in r['source_payload_hashes']};rows=[]
    for selector,kind in protocol['archetypes'].items():
        key=selector.replace(':','-');report=json.loads((a.out/'analysis'/(key+'.json')).read_text());overlap=[s['model_id']for s in report['source']['sources']if s['payload_sha256']in train_hashes];row=dict(selector=selector,name=report['source']['name'],kind=kind,source_model_ids=report['source']['model_ids'],overlapping_training_models=overlap,split_valid=not overlap,artistic_acceptance=False)
        if kind is None:row.update(status='UNSUPPORTED',reason=protocol['unsupported_reasons'][selector])
        else:
            recipe=a.out/'recipes'/(key+'.json');recipe.parent.mkdir(parents=True,exist_ok=True);recipe.write_text(json.dumps(make(kind,report),indent=2));out=a.out/'authored'/key
            command=[sys.executable,str(here/'author_workflow.py'),'--recipe',str(recipe),'--out',str(out),'--native',str(a.native),'--cache',str(a.cache),'--blender',str(a.blender),'--reference',str(a.out/'reference'/key),'--bounds',str(a.bounds),'--identities',str(a.identities),'--asset-key','heldout-v1:'+selector]
            result=subprocess.run(command);row['native_workflow_exit_code']=result.returncode
            if result.returncode:row.update(status='TECHNICAL_FAILURE')
            else:
                metrics=json.loads((out/'comparison/comparison.json').read_text());row.update(status='RENDERED_UNAPPROVED'if not overlap else 'CONTAMINATED_RENDERED_UNAPPROVED',mean_five_view_iou=sum(v['iou']for v in metrics.values())/5,per_view_iou={k:v['iou']for k,v in metrics.items()},mean_rgb_absolute_error=sum(v['mean_rgb_absolute_error']for v in metrics.values())/5)
        rows.append(row);(a.out/'results.json').write_text(json.dumps(dict(protocol=protocol['protocol'],tool_freeze=freeze,results=rows,artistic_acceptance=False),indent=2))
    print('HELD_OUT_EVALUATION_COMPLETE',len(rows),flush=True)
if __name__=='__main__':
    p=argparse.ArgumentParser()
    for key in ['native','cache','blender','out','training','bounds','identities']:p.add_argument('--'+key,type=Path,required=True)
    evaluate(p.parse_args())
