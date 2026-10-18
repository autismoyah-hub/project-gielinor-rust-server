"""Explicitly post-examination private retest; never rewrites frozen evidence."""
import argparse,json,subprocess,sys,hashlib,math,struct,re
from pathlib import Path
from collections import Counter
from kinematics import trace,frame_at
HERE=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text(encoding='utf-8'))
def run(a):
    if a.out.exists():raise ValueError('Retest output must be new')
    if a.out.resolve().is_relative_to(HERE.parents[2]):raise ValueError('Private output must stay outside source checkout')
    a.out.mkdir(parents=True);rows=[]
    for case in load(a.cases)['cases']:
      frozen=a.sealed/case['key']
      for variant in case['variants']:
        phase=variant['phase'];old=frozen/'reconstructed'/phase;ref=frozen/'reference'/phase/('npc-'+str(variant['npc_id']));sg=load(ref/'geometry.json');meta=load(ref/'metadata.json');r=load(old/'recipe.json');regions=load(frozen/'private-recipes'/phase/'regions.json');r['preserve_authored_groups']=bool(sg.get('vertex_skins'));r['preserve_face_groups']=sg.get('face_skins')is not None
        for p,region in zip(r['components'],regions):
          ids=region['selector']['indices']
          if sg.get('face_skins')is not None:p['face_group']=Counter(sg['face_skins'][i]for i in ids).most_common(1)[0][0]
        root=a.out/case['key']/phase;root.mkdir(parents=True);recipe=root/'input.json';recipe.write_text(json.dumps(r,indent=2),encoding='utf-8');candidate=root/'candidate'
        subprocess.run([sys.executable,str(HERE/'author_workflow.py'),'--recipe',str(recipe),'--out',str(candidate),'--reference',str(ref),'--native',str(a.native),'--cache',str(a.cache),'--blender',str(a.blender),'--bounds',str(a.bounds),'--identities',str(a.identities),'--asset-key','mastery-sealed:'+case['key']+':'+phase,'--skip-motion'],check=True)
        cg=load(candidate/'geometry.json');og=load(old/'geometry.json');assert cg['vertices']==og['vertices']and cg['faces']==og['faces'];actions=[];done={}
        for role,sequence in {**meta['sequences'],**variant.get('additional_sequences',{})}.items():
          source=frozen/'reference-motion'/phase/role/('npc-'+str(variant['npc_id']))/('selected-'+str(sequence));data=load(source/'animation.json')
          if data.get('cached_animation')is not None:actions.append(dict(role=role,sequence=sequence,status='CACHED_WEIGHTED_RECONSTRUCTION_UNSUPPORTED'));continue
          if sequence in done:actions.append(dict(role=role,sequence=sequence,alias_of=done[sequence]));continue
          target=root/'motion'/role;subprocess.run([str(a.native),str(a.cache),str(target),'play-complete',str(candidate/'asset.model'),str(ref/'camera.txt'),str(meta['ambient']),str(meta['contrast']),str(source),*map(str,meta['render_scale'])],check=True)
          duration=sum(f['length']for f in data['frames']);loop=int(re.search(r'loop_frames: (-?\d+)',(source/'sequence.txt').read_text()).group(1));f32=lambda x:struct.unpack('<f',struct.pack('<f',x))[0];worst=0;first=None;moving=0;samples=sorted(target.glob('pose-*.json'));source_samples=list(source.glob('pose-*.json'))
          for path in samples:
            tick=int(path.stem.split('-')[1]);_,frame=frame_at(data['frames'],tick%duration if loop>0 else tick);expected,_=trace(cg,frame);actual=load(path)['vertices'];worst=max(worst,max(abs(f32(v[i]*meta['render_scale'][i]/128)-f32(w[i]))for v,w in zip(expected,actual)for i in range(3)));first=first or actual;moving+=actual!=first
          assert len(samples)==len(source_samples)and worst==0
          actions.append(dict(role=role,sequence=sequence,status='COMPLETE_NATIVE_CLASSIC_TRACE_EXACT',samples=len(samples),moving_samples=moving,max_coordinate_error=worst,source_controls_reused_privately=True));done[sequence]=role
        rows.append(dict(case=case['key'],phase=phase,geometry_unchanged=True,actions=actions,comparison=load(candidate/'comparison/comparison.json'),artistic_acceptance=False));(a.out/'results.json').write_text(json.dumps(dict(kind='POST_EXAMINATION_DEVELOPMENT_RETEST',native_sha256=hashlib.sha256(a.native.read_bytes()).hexdigest(),source_hashes={p.name:hashlib.sha256(p.read_bytes()).hexdigest()for p in HERE.glob('*.py')},results=rows,independent_choreography_claim=False,limitations=['Group ownership correction only; coarse reconstruction fidelity remains under review.','Source classic controls reused privately; cached weighted retargeting unsupported.']),indent=2),encoding='utf-8');print('RETEST_COMPLETE',case['key'],phase,flush=True)
if __name__=='__main__':
    p=argparse.ArgumentParser()
    for n in ['sealed','cases','out','native','cache','blender','bounds','identities']:p.add_argument('--'+n,type=Path,required=True)
    run(p.parse_args())
