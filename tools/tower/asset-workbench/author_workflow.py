"""Reproducible owned-asset authoring, registration, native views and motion QA."""
import argparse,subprocess,json,sys,hashlib,struct
from pathlib import Path
from native_codec import model
from registry import IdentityAllocator,object_package
from analyze import compare,geometry
from kinematics import trace,frame_at
HERE=Path(__file__).resolve().parent
def run(command):subprocess.run([str(x)for x in command],check=True)
def main():
    p=argparse.ArgumentParser();p.add_argument('--recipe',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--native',type=Path,required=True);p.add_argument('--cache',type=Path,required=True);p.add_argument('--blender',type=Path,required=True);p.add_argument('--reference',type=Path);p.add_argument('--bounds',type=Path,required=True);p.add_argument('--identities',type=Path,required=True);p.add_argument('--asset-key',required=True);p.add_argument('--blend',type=Path);p.add_argument('--patch',type=Path);p.add_argument('--skip-motion',action='store_true');p.add_argument('--replace-actions',action='store_true');a=p.parse_args();a.out=a.out.resolve();a.reference=a.reference.resolve()if a.reference else None;
    if a.out.is_relative_to(a.cache.resolve())or a.out.is_relative_to(HERE.parents[2]):raise ValueError('Calibration output must stay outside repository and original cache')
    if a.out.exists()and any(a.out.iterdir()):raise ValueError('Output is not empty; preserve previous runs and choose a new output directory')
    a.out.mkdir(parents=True,exist_ok=True)
    r=json.loads(a.recipe.read_text());identity=IdentityAllocator(a.identities,json.loads(a.bounds.read_text())).allocate(a.asset_key,animated=bool(r.get('rig')))
    if r.get('rig'):r.update(skeleton_base=identity['skeleton_base'],frame_group_base=identity['frame_group_base'])
    (a.out/'input-recipe.json').write_text(json.dumps(r,indent=2));recipe=a.out/'input-recipe.json'
    if a.blend and a.patch:run([a.blender,'-b',a.blend,'--python-exit-code','1','--python',HERE/'edit_blender.py','--',recipe,a.patch,a.out])
    elif a.blend:run([a.blender,'-b',a.blend,'--python-exit-code','1','--python',HERE/'author_blender.py','--',recipe,a.out,'--reopen',*(['--replace-actions']if a.replace_actions else [])])
    else:
        palette=a.reference.parent/'palette.json'if a.reference else a.out/'no-palette.json';run([a.blender,'-b','--python-exit-code','1','--factory-startup','--python',HERE/'author_blender.py','--',recipe,a.out/'created',*(['--palette',palette]if palette.exists()else [])])
        run([a.blender,'-b',a.out/'created/asset.blend','--python-exit-code','1','--python',HERE/'author_blender.py','--',recipe,a.out,'--reopen'])
        if (a.out/'created/asset.model').read_bytes()!=(a.out/'asset.model').read_bytes():raise AssertionError('Owned geometry changed on saved Blender reopen')
    g=json.loads((a.out/'geometry.json').read_text())
    meta=json.loads((a.reference/'metadata.json').read_text())if a.reference else dict(ambient=64,contrast=768,selector=None,definition_sha256=None)
    if a.reference:
        camera=a.reference/'camera.txt';run([a.native,a.cache,a.out/'native','render',a.out/'asset.model',camera,meta['ambient'],meta['contrast']]);compare(a.reference,a.out/'native',a.out/'comparison')
    else:
        run([a.native,a.cache,a.out/'native','render-own',a.out/'asset.model',meta['ambient'],meta['contrast']]);camera=a.out/'native/camera.txt'
    bounds=json.loads(a.bounds.read_text());object_package(a.out/'object-package',[g],dict(shape=10,size=[1,1],non_flat_shading=True),[identity['model_id']],identity['object_id'],bounds)
    results=[]
    if not a.skip_motion:
        for index,path in enumerate(sorted(a.out.glob('*/animation.json'))):
            action=path.parent.name;data=json.loads(path.read_text());target=a.out/'motion'/action;run([a.native,a.cache,target,'play',a.out/'asset.model',camera,meta['ambient'],meta['contrast'],path.parent]);duration=sum(row['length']for row in data['frames']);expected_times=set(range(min(duration,256)+1))|{duration,duration//4,duration//2,duration*3//4};actual_times={int(p.stem.split('-')[1])for p in target.glob('pose-*.json')}
            if actual_times!=expected_times:raise AssertionError(f'Native timestamp coverage mismatch {action}')
            worst=0;checked=0
            for pose in target.glob('pose-*.json'):
                tick=int(pose.stem.split('-')[1]);sample=tick%duration if data['policy'].get('loop_frames',-1)>0 else tick;_,frame=frame_at(data['frames'],sample);predicted,_=trace(g,frame);actual=json.loads(pose.read_text())['vertices'];
                if len(predicted)!=len(actual):raise AssertionError('Native vertex count mismatch')
                error=max(abs(x-y)for v,w in zip(predicted,actual)for x,y in zip(v,w));worst=max(worst,error);checked+=1
            if worst:raise AssertionError(f'Authored joint compiler disagrees with native {action}: {worst}')
            results.append(dict(action=action,native_pose_samples=checked,max_coordinate_error_native=worst,sequence_id=identity['sequence_base']+index,skeleton_id=data['skeleton_id'],frame_group=data['frame_group']))
    summary=dict(asset_key=a.asset_key,identity=identity,geometry=geometry(g),motion=results,native_binary_sha256=hashlib.sha256(a.native.read_bytes()).hexdigest(),model_sha256=hashlib.sha256((a.out/'asset.model').read_bytes()).hexdigest(),artistic_acceptance=False,source_geometry_reused=False,reference_selector=meta['selector'],reference_definition_sha256=meta['definition_sha256'])
    (a.out/'authoring-report.json').write_text(json.dumps(summary,indent=2));print('AUTHORING_NATIVE_WORKFLOW_PASS',a.asset_key,'motion samples',sum(v['native_pose_samples']for v in results))
if __name__=='__main__':main()
