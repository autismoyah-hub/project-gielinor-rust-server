"""Sealed private reference reconstruction and matched native motion examination.

Reference-derived meshes/recipes/images/frames stay outside the source tree.
Original counterparts must be supplied separately as genuinely authored recipes.
"""
import argparse,json,subprocess,sys,hashlib,math
from pathlib import Path
from PIL import Image,ImageDraw
from reconstruction_regions import propose
from section_reconstruction import reconstruct
from motion import selectors,centroid
from component_review import internal
from analyze import foreground
from PIL import ImageChops,ImageStat
HERE=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text(encoding='utf-8'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()

def command(args):subprocess.run([str(x)for x in args],check=True)
def freeze_check(freeze,native):
    if sha(native)!=freeze['native_binary_sha256']:raise ValueError('Native renderer changed after exam freeze')
    for name,value in freeze['source_hashes'].items():
        if sha(HERE/name)!=value:raise ValueError('Frozen authoring module changed: '+name)

def temporal(reference,candidate,source_g,candidate_g,out):
    out.mkdir(parents=True,exist_ok=True);ticks=sorted(int(p.stem.split('-')[1])for p in reference.glob('pose-*.json'));groups=sorted(set(source_g.get('vertex_skins')or[]));bindings={f'native_group_{n}':dict(skin_labels=[n])for n in groups if sum(v==n for v in(source_g.get('vertex_skins')or[]))>=3 and n in(candidate_g.get('vertex_skins')or[])};ai={n:selectors(source_g,b)for n,b in bindings.items()};bi={n:selectors(candidate_g,b)for n,b in bindings.items()};rows=[];frames=[]
    for tick in ticks:
        p=reference/f'pose-{tick:04}.json';q=candidate/p.name
        if not q.exists():continue
        a,b=load(p)['vertices'],load(q)['vertices'];rows.append(dict(tick=tick,landmark_centroid_errors={n:math.dist(centroid(a,ai[n]),centroid(b,bi[n]))for n in bindings}))
        canvas=Image.new('RGB',(1000,490),(16,21,26));draw=ImageDraw.Draw(canvas)
        for col,(name,folder)in enumerate([('Authentic native',reference),('Independent topology / source classic controls',candidate)]):
            im=Image.open(folder/f'pose-{tick:04}.bmp');im.thumbnail((500,460));canvas.paste(im,(500*col,28));draw.text((500*col+8,7),f'{name} | tick {tick}',fill='white')
        frames.append(canvas)
    if frames:frames[0].save(out/'matched-motion.gif',save_all=True,append_images=frames[1:],duration=20,loop=0)
    result=dict(matched_timestamps=len(rows),same_native_clock=True,source_frame_controls_reused_privately=True,independent_animation_authoring_claim=False,group_semantics='Native skin-label proxies; not automatically inferred anatomy',samples=rows,artistic_acceptance=False);(out/'temporal-comparison.json').write_text(json.dumps(result,indent=2));return result

def examine(a):
    freeze=load(a.freeze);freeze_check(freeze,a.native);case=next(r for r in load(a.cases)['cases']if r['key']==a.case);root=a.out/a.case
    if root.exists()and any(root.iterdir()):raise ValueError('Sealed attempt already exists; preserve it and name a separate development retest')
    root.mkdir(parents=True,exist_ok=True);(root/'case-selection.json').write_text(json.dumps(case,indent=2));results=[]
    for variant in case['variants']:
        npc_id=variant['npc_id'];key='npc-'+str(npc_id);row=dict(npc_id=npc_id,phase=variant['phase'],authoring_source_commit=freeze['source_commit'],native_binary_sha256=freeze['native_binary_sha256'],gameplay_integration_tested=False,artistic_acceptance=False)
        try:
            refroot=root/'reference'/variant['phase'];command([a.native,a.cache,refroot,'inspect','npc:'+str(npc_id)]);ref=refroot/key;g=load(ref/'geometry.json');meta=load(ref/'metadata.json');row['previous_payload_overlap']=[s['model_id']for s in meta['sources']if s['payload_sha256']in freeze.get('prior_payload_hashes',[])];row['split_valid']=not row['previous_payload_overlap'];regions=propose(g);recipe,descriptors=reconstruct(g,regions,'Private '+case['label']+' '+variant['phase']+' reconstruction');recipe['native_render_scale']=meta['render_scale'];source=root/'private-recipes'/variant['phase'];source.mkdir(parents=True,exist_ok=True)
            for name,d in [('recipe',recipe),('regions',regions),('section-observations',descriptors)]:(source/(name+'.json')).write_text(json.dumps(d,indent=2))
            candidate=root/'reconstructed'/variant['phase'];command([sys.executable,HERE/'author_workflow.py','--recipe',source/'recipe.json','--out',candidate,'--reference',ref,'--native',a.native,'--cache',a.cache,'--blender',a.blender,'--bounds',a.bounds,'--identities',a.identities,'--asset-key','mastery-sealed:'+a.case+':'+variant['phase'],'--skip-motion']);cg=load(candidate/'geometry.json');comparison=load(candidate/'comparison/comparison.json');surface={}
            for view in ['front','side','rear','quarter','gameplay']:
                aa,bb=[Image.open(p/(view+'.bmp')).convert('RGB')for p in [ref,candidate/'native']];union=ImageChops.lighter(foreground(aa),foreground(bb));delta=ImageChops.difference(aa,bb);surface[view]=dict(foreground_union_rgb_error=sum(ImageStat.Stat(delta,mask=union).mean)/3,internal_edges=internal(aa,bb))
            actions={**meta['sequences'],**variant.get('additional_sequences',{})};motion=[]
            for role,sequence in actions.items():
                action=dict(role=role,source_sequence=sequence)
                try:
                    ar=root/'reference-motion'/variant['phase']/role;command([a.native,a.cache,ar,'animate-complete','npc:'+str(npc_id),sequence]);source_action=ar/key/('selected-'+str(sequence));anim=load(source_action/'animation.json');action['authentic_native_samples']=len(list(source_action.glob('pose-*.json')))
                    if anim.get('cached_animation')is not None:action.update(status='SOURCE_CACHED_WEIGHTED_UNSUPPORTED_FOR_RECONSTRUCTION',reason='Authentic cached motion renders natively; independent weighted authoring/retarget is not implemented. No invented classic substitute is claimed faithful.')
                    else:
                        target=root/'reconstruction-motion'/variant['phase']/role;command([a.native,a.cache,target,'play',candidate/'asset.model',ref/'camera.txt',meta['ambient'],meta['contrast'],source_action,*meta['render_scale']]);m=temporal(source_action,target,g,cg,root/'motion-review'/variant['phase']/role);action.update(status='NATIVE_CLASSIC_RECONSTRUCTION_PLAYBACK_VALIDATED',matched_timestamps=m['matched_timestamps'],source_frame_controls_reused_privately=True,independent_animation_authoring_claim=False)
                except(Exception)as e:action.update(status='BLOCKED',reason=str(e))
                motion.append(action)
            report=load(candidate/'authoring-report.json');row.update(status='TECHNICAL_CONVERSION_VALIDATED',reconstruction_fidelity='UNDER_REVIEW',method='Data-assisted coarse section reconstruction on new topology, distinct from import/export preservation',reference_name=meta['name'],reference_model_ids=meta['model_ids'],definition_sha256=meta['definition_sha256'],source_hashes=meta['sources'],source_vertices=len(g['vertices']),source_faces=len(g['faces']),candidate_vertices=len(cg['vertices']),candidate_faces=len(cg['faces']),identity=report['identity'],mean_five_view_iou=sum(v['iou']for v in comparison.values())/5,views=comparison,surface=surface,motion=motion,limitations=['Coarse group/material regions are not inferred anatomical semantics.','Fine facial, armor, UV, alpha and construction detail may be missing.','Classic source control reuse is a private modeling test, not independent animation mastery.','Cached weighted reconstruction motion is unsupported.'])
        except(Exception)as e:row.update(status='BLOCKED',reason=str(e))
        results.append(row);(root/'results.json').write_text(json.dumps(dict(case=case,freeze=freeze,results=results,artistic_acceptance=False),indent=2))
    print('SEALED_ADVANCED_CASE_COMPLETE',a.case)

if __name__=='__main__':
    p=argparse.ArgumentParser()
    for name in ['freeze','cases','out','native','cache','blender','bounds','identities']:p.add_argument('--'+name,type=Path,required=True)
    p.add_argument('--case',required=True);examine(p.parse_args())
