"""Portable orchestration CLI. All generated references/derivatives are private.

No downloads, running game services, user input automation or cache writes.
"""
import argparse,json,subprocess,sys,hashlib
from pathlib import Path
from native_codec import model,classic
from PIL import Image,ImageChops
from analyze import library,compare
from registry import object_package
HERE=Path(__file__).resolve().parent
def run(args):subprocess.run([str(x)for x in args],check=True)
def identity(args):
    return dict(native_binary_sha256=hashlib.sha256(args.native.read_bytes()).hexdigest(),catalog_sha256=hashlib.sha256(args.catalog.read_bytes()).hexdigest(),inputs='read-only original cache; protected derivative output',production_integration=False)
def main():
    p=argparse.ArgumentParser();p.add_argument('--native',type=Path,required=True);p.add_argument('--cache',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--blender',type=Path);p.add_argument('--reference-root',type=Path);p.add_argument('--catalog',type=Path,default=HERE/'reference_catalog.json');p.add_argument('--split',choices=['training','held_out'],default='training');p.add_argument('command',choices=['inspect','analyze','roundtrip']);a=p.parse_args();a.native=a.native.resolve();a.out=a.out.resolve();a.cache=a.cache.resolve();a.catalog=a.catalog.resolve()
    if a.out.is_relative_to(a.cache)or a.out.is_relative_to(HERE.parents[2]):raise ValueError('Reference outputs must remain outside source repository and original cache')
    a.out.mkdir(parents=True,exist_ok=True);records=[r for r in json.loads(a.catalog.read_text())['records']if r['split']==a.split];reference=a.reference_root.resolve()if a.reference_root else a.out/a.split/'reference';analysis=a.out/a.split/'analysis'
    if a.command=='inspect':run([a.native,a.cache,reference,'inspect',*[r['selector']for r in records]])
    if a.command in ['inspect','analyze']:library(reference,analysis)
    if a.command=='roundtrip':
        if not a.blender:raise ValueError('--blender required for saved-file round trips')
        outputs=a.out/a.split/'roundtrip';outputs.mkdir(parents=True,exist_ok=True)
        run([a.blender,'-b','--python-exit-code','1','--factory-startup','--python',HERE/'blender_io.py','--','import',reference,outputs])
        run([a.blender,'-b',outputs/'reference-roundtrips.blend','--python-exit-code','1','--python',HERE/'blender_io.py','--','export',reference,outputs/'exported'])
        expected_actions=len(list(reference.glob('*/*/animation.json')));actual_actions=len(list((outputs/'exported/actions').glob('*/*/animation.json')))
        if expected_actions!=actual_actions:raise AssertionError(f'Animation export coverage {actual_actions}/{expected_actions}')
        results=[]
        for f in sorted((outputs/'exported').glob('*.json')):
            name=f.stem
            if '--raw-model-'in name:continue
            g=json.loads(f.read_text());meta=json.loads((reference/name/'metadata.json').read_text());target=outputs/name;target.mkdir(exist_ok=True);payload=model(g);(target/'roundtrip.model').write_bytes(payload)
            run([a.native,a.cache,target,'render',target/'roundtrip.model',reference/name/'camera.txt',meta['ambient'],meta['contrast'],*meta['render_scale']])
            compare(reference/name,target,target/'comparison')
            decoded=json.loads((target/'decoded.json').read_text());original=json.loads((reference/name/'geometry.json').read_text());channel={k:original.get(k)==decoded.get(k)for k in original}
            if original.get('priorities')is not None:channel['priority']='unused global fallback; effective per-face arrays compared'
            if original.get('face_textures')is not None:
                coords_a=original.get('texture_coords');coords_b=decoded.get('texture_coords')
                if coords_a is not None and coords_b is not None:channel['texture_coords_effective']=all(t<0 or x==y for t,x,y in zip(original['face_textures'],coords_a,coords_b))
            pixels=json.loads((target/'comparison/comparison.json').read_text());results.append(dict(selector=meta['selector'],new_model_id=1101000+len(results),original_model_ids=meta['model_ids'],channels=channel,views={k:v['changed_pixels']for k,v in pixels.items()},all_pixels_exact=all(v['changed_pixels']==0 for v in pixels.values()),note='new isolated diagnostic model; not installed in live cache'))
        bounds=json.loads((reference/'original-id-bounds.json').read_text())
        for index,deferred in enumerate(json.loads((outputs/'deferred.json').read_text())):
            name=deferred['asset'];meta=json.loads((reference/name/'metadata.json').read_text());factory=json.loads((reference/name/'factory.json').read_text());target=outputs/name;target.mkdir(exist_ok=True);parts=[json.loads((outputs/'exported'/(name+f'--raw-model-{i}.json')).read_text())for i in meta['model_ids']];ids=[1101200+index*10+j for j in range(len(parts))];oid=63350+index;object_package(target/'package',parts,factory,ids,oid,bounds)
            run([a.native,a.cache,target,'package',target/'package',oid,factory['shape'],reference/name/'camera.txt',meta['ambient'],meta['contrast']]);compare(reference/name,target,target/'comparison');pixels=json.loads((target/'comparison/comparison.json').read_text());g=json.loads((reference/name/'geometry.json').read_text());decoded=json.loads((target/'decoded.json').read_text());results.append(dict(selector=meta['selector'],new_model_ids=ids,new_object_id=oid,channels={k:g.get(k)==decoded.get(k)for k in g},views={k:v['changed_pixels']for k,v in pixels.items()},all_pixels_exact=all(v['changed_pixels']==0 for v in pixels.values()),note='Raw Blender mesh round trip plus native custom-definition transforms; no evaluated-coordinate quantization'))
        (outputs/'results.json').write_text(json.dumps(results,indent=2));print('BATCH_ROUNDTRIP_COMPLETE',len(results),'pixel-exact',sum(r['all_pixels_exact']for r in results))
        action_results=[]
        for path in sorted((outputs/'exported/actions').glob('*/*/animation.json')):
            asset=path.parent.parent.name;action=path.parent.name;data=json.loads(path.read_text());target=outputs/asset/'actions'/action;classic(data,target/'package');meta=json.loads((reference/asset/'metadata.json').read_text());run([a.native,a.cache,target/'native','play',outputs/asset/'roundtrip.model',reference/asset/'camera.txt',meta['ambient'],meta['contrast'],target/'package',*meta['render_scale']]);samples=[]
            for original in sorted((reference/asset/action).glob('pose-*.bmp')):
                converted=target/'native'/original.name
                if not converted.exists():raise AssertionError('Missing matching native animation timestamp '+str(converted))
                difference=ImageChops.difference(Image.open(original).convert('RGB'),Image.open(converted).convert('RGB'));samples.append(dict(tick=int(original.stem.split('-')[1]),pixel_exact=difference.getbbox()is None))
            action_results.append(dict(asset=asset,action=action,source_sequence=data['source_sequence'],new_skeleton=data['skeleton_id'],new_frame_group=data['frame_group'],native_samples=samples,all_pixels_exact=bool(samples)and all(s['pixel_exact']for s in samples),limits=data['not_encoded']))
        (outputs/'animation-results.json').write_text(json.dumps(action_results,indent=2));print('BATCH_ANIMATION_ROUNDTRIP_COMPLETE',len(action_results),'actions;',sum(len(r['native_samples'])for r in action_results),'samples; pixel-exact',sum(r['all_pixels_exact']for r in action_results))
    (a.out/(a.split+'-identity.json')).write_text(json.dumps(identity(a),indent=2))
if __name__=='__main__':main()
