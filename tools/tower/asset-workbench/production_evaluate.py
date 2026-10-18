"""Sealed production-family evaluation; fixes never overwrite a frozen run."""
import argparse,json,subprocess,sys,hashlib
from pathlib import Path

def main(a):
    a.out=a.out.resolve();freeze=json.loads((a.out/'freeze.json').read_text());here=a.authoring_root.resolve()
    sys.path.insert(0,str(here))
    from analyze import library
    from fit_semantic import build
    for name,digest in freeze['source_hashes'].items():
        if hashlib.sha256((here/name).read_bytes()).hexdigest()!=digest:raise ValueError('Frozen authoring source changed: '+name)
    if hashlib.sha256(a.native.read_bytes()).hexdigest()!=freeze['native_binary_sha256']:raise ValueError('Frozen native executable changed')
    if (a.out/'results.json').exists():raise ValueError('Frozen results already exist; never overwrite a sealed evaluation')
    inspected=[]
    for record in freeze['records']:
        q=subprocess.run([str(a.native),str(a.cache),str(a.out/'reference'),'inspect-static',record['selector']]);inspected.append((record,q.returncode))
    library(a.out/'reference',a.out/'analysis');known=set()
    for root in a.prior_reference:
        for p in root.glob('*/metadata.json'):
            for s in json.loads(p.read_text()).get('sources',[]):known.add(s['payload_sha256'])
    results=[]
    for record,code in inspected:
        selector=record['selector'];key=selector.replace(':','-');row=dict(selector=selector,archetype=record['archetype'],authoring_source_commit=freeze['source_commit'],native_binary_sha256=freeze['native_binary_sha256'],artistic_acceptance=False)
        if code:row.update(status='BLOCKED',reason='Native reference inspection failed; protected log retained')
        else:
            report=json.loads((a.out/'analysis'/(key+'.json')).read_text());source=report['source'];row.update(reference_name=source['name'],reference_model_ids=source['model_ids'],definition_sha256=source['definition_sha256'],source_payload_hashes=[s['payload_sha256']for s in source['sources']],training_overlap_model_ids=[s['model_id']for s in source['sources']if s['payload_sha256']in known]);row['split_valid']=not row['training_overlap_model_ids']
            try:
                recipe=build(record['archetype'],report);p=a.out/'recipes'/(key+'.json');p.parent.mkdir(exist_ok=True);p.write_text(json.dumps(recipe,indent=2));out=a.out/'authored'/key
                command=[sys.executable,str(here/'author_workflow.py'),'--recipe',str(p),'--out',str(out),'--native',str(a.native),'--cache',str(a.cache),'--blender',str(a.blender),'--reference',str(a.out/'reference'/key),'--bounds',str(a.bounds),'--identities',str(a.identities),'--asset-key','production-heldout-v1:'+selector]
                q=subprocess.run(command)
                if q.returncode:row.update(status='BLOCKED',reason='Owned authoring/native workflow failed; failed candidate preserved')
                else:
                    d=json.loads((out/'comparison/comparison.json').read_text());row.update(status='UNAPPROVED'if row['split_valid']else'CONTAMINATED_UNAPPROVED',mean_five_view_iou=sum(v['iou']for v in d.values())/5,views=d,identity=json.loads((out/'authoring-report.json').read_text())['identity'])
            except (ValueError,KeyError,ZeroDivisionError)as e:row.update(status='UNSUPPORTED',reason=str(e))
        results.append(row);(a.out/'results.json').write_text(json.dumps(dict(freeze=freeze,results=results,artistic_acceptance=False),indent=2))
    print('SEALED_PRODUCTION_EVALUATION_COMPLETE',len(results))
if __name__=='__main__':
    p=argparse.ArgumentParser()
    for name in ['native','cache','blender','out','bounds','identities']:p.add_argument('--'+name,type=Path,required=True)
    p.add_argument('--authoring-root',type=Path,default=Path(__file__).resolve().parent)
    p.add_argument('--prior-reference',type=Path,action='append',required=True);main(p.parse_args())
