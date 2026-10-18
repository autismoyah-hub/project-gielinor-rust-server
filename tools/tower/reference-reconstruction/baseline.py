"""Preserve the original turnaround and previous mesh before reconstruction."""
from pathlib import Path
import json,hashlib,shutil,subprocess,datetime
T=Path(__file__).resolve().parent;S=T.parents[2];E=S.parent;R=E/'ashhorn-reference-private'
R.mkdir(exist_ok=True)
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def git(p,*a):return subprocess.check_output(['git',*a],cwd=p,text=True).strip()
old=E/'concept-ab-private';dest=R/'baseline';dest.mkdir(exist_ok=True)
art=dest/'original-art';art.mkdir(exist_ok=True)
for name in ['hero.png','turnaround-v1.png','turnaround-v2.png']:
    target=art/name
    if target.exists():assert sha(target)==sha(old/'concept'/name)
    else:shutil.copy2(old/'concept'/name,target)
previous=dest/'previous';previous.mkdir(exist_ok=True)
for name in ['source.blend','asset.model','geometry.json','recipe.json','conversion.json','components.json','generation-snapshot.py','plate-authoring-snapshot.py']:
    target=previous/name
    if target.exists():assert sha(target)==sha(old/'concept/final'/name)
    else:shutil.copy2(old/'concept/final'/name,target)
(previous/'native').mkdir(exist_ok=True)
for view in ['front','side','rear','quarter','above','gameplay']:
    for ext in ['png','camera']:
        source=old/'concept/final/native'/(view+'.'+ext);target=previous/'native'/source.name
        if target.exists():assert sha(target)==sha(source)
        else:shutil.copy2(source,target)
rows=[
 (1,'Overall silhouette','The old frozen numeric width constrained the forequarters and paws to about100 native units, while the sheet suggests about140 at a common height. The body and stance read too narrow. New target is the existing drawing, not the old specification.'),
 (2,'Skull and face','The previous station-loft skull lacks the broad temple/zygomatic structure, lowered nasal bridge and integrated brow hierarchy. Eye and cheek overlay planes do not create convincing underlying facial anatomy.'),
 (3,'Muzzle','The mesh projects a long generic wedge with a shallow underside. The drawing has a blunt faceted nose, broader plane changes and a higher, stronger mandibular corner.'),
 (4,'Eye sockets and brows','Small flat amber patches sit on a generic side surface. The reference has deep dark sockets under a heavy oblique brow with a distinct focused expression. Geometry must establish the orbit before color.'),
 (5,'Shoulder and forelimbs','The drawing has substantially thicker deltoid/triceps masses and wrists, shorter-looking lower forelegs and a wider grounded stance. The mesh has thin elongated stick-like distal segments.'),
 (6,'Neck and chest','The mesh throat/chest is a rounded undifferentiated mass and the underside mask a broad tan region. The drawing separates an angular throat, deep narrow sternum and scapular/chest volumes.'),
 (7,'Torso, spine, waist and pelvis','The old body remains a generic smooth quadruped profile; lateral volume and lumbar curvature do not reproduce the drawing. The sheet varies between views; a coherent measured ribcage/waist/pelvis framework is needed.'),
 (8,'Hindlimbs','The reference has strong forward knee, tight angular hock and distinct thigh-to-calf transitions. The previous body union softens those features and the posed near/far foot locations differ from the side drawing.'),
 (9,'Paws, toes and claws','Counts are correct in the previous mesh but paws are too small and uniform. The art shows broad weight-bearing toe lobes, distinct knuckles and thick curved claws; the mesh reads as narrow pads with pale spikes.'),
 (10,'Ears and horns','The previous long horn rises too vertically and its numeric laterality conflicts with the sheet. The accepted image shows a more backward-swept long horn, visible bands, broad chipped stump and taller sharply folded ears. The new coordinate convention will honor drawn laterality.'),
 (11,'Armor layers','The previous interior-sampled shields avoid major intersections but lack the broad layered wrap, strongly shaped shoulder caps and coherent peaked rear shingle silhouette. They remain plain generic plates.'),
 (12,'Tail','The old frozen low tip was deliberately used instead of the concept arc. The new canonical sheet shows a longer raised curl; rear and side arcs still conflict and require a recorded spatial interpretation.'),
 (13,'Low-poly faceting','The mesh has broad profile planes and arbitrary fusion triangles rather than designed cheek, scapular, muscle and armor planes. Add form-driven topology before surface microdetail.'),
 (14,'Materials and color distribution','Eight broad face colors omit the reference hide-plane variation, warm anatomical underside and deliberate rusty plate margins. Painted highlights must not be mistaken for geometry; native color refinement comes after shape.')]
assessment=[{'rank':rank,'component':name,'finding':finding,'status':'Diagnosed before new geometry'} for rank,name,finding in rows]
(R/'discrepancy-assessment.json').write_text(json.dumps(assessment,indent=2)+'\n')
audit={'created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'assignment':'Existing Ashhorn turnaround is canonical; previous text dimensions are superseded, no new concept or A/B experiment','canonical_image':'baseline/original-art/turnaround-v2.png','previous_mesh':'baseline/previous/asset.model','source_checkpoint':git(S,'rev-parse','HEAD'),'branch':git(S,'branch','--show-current'),'blender_version':subprocess.check_output(['C:/Program Files/Blender Foundation/Blender 5.2/blender.exe','--version'],text=True).splitlines()[:6],'native_workbench_sha256':sha(E/'astra-benchmark/tools/asset_workbench.exe'),'preserved_files':{str(p.relative_to(R)):sha(p)for p in dest.rglob('*')if p.is_file()},'other_workspaces':{name:{'head':git(E/name,'rev-parse','HEAD'),'status':git(E/name,'status','--porcelain')}for name in ['concept-ab-server','constrained-server','enhanced-client','rust-client-reference','rust-server']}}
assert audit['source_checkpoint']=='56af70ef03668fd6f429e0c3dd958bcdef7e8b40'
assert audit['native_workbench_sha256']=='f6e6e12da1a80f7824c9d90a5254964fdc8d738298417c7dcd51e0793caea7cf'
(R/'baseline.json').write_text(json.dumps(audit,indent=2)+'\n')
parts=['<!doctype html><meta charset="utf-8"><title>Ashhorn reconstruction: discrepancy assessment</title><style>body{background:#121b20;color:#d9e0df;font:17px/1.6 system-ui;max-width:1300px;margin:40px auto;padding:20px}h1{font:46px Georgia}.pair{display:grid;grid-template-columns:1fr 1fr;gap:20px}img{width:100%}article{border-top:1px solid #53636a;padding:15px 0}b{color:#cdb18b}</style><h1>Reconstruct the existing design.</h1><p>Immutable concept and previous model; diagnosis prepared before changing geometry. The image is now the canonical artistic target.</p><div class="pair"><figure><img src="baseline/original-art/turnaround-v2.png"><figcaption>Accepted original turnaround — preserved untouched</figcaption></figure><figure><img src="baseline/previous/native/quarter.png"><figcaption>Previous concept-guided mesh — preserved untouched</figcaption></figure></div>']
for row in assessment:parts.append('<article><b>'+str(row['rank'])+'. '+row['component']+'</b><p>'+row['finding']+'</p></article>')
(R/'diagnosis.html').write_text(''.join(parts),encoding='utf8')
print('Immutable baseline captured:',len(audit['preserved_files']),'files. Fourteen discrepancy categories ranked before geometry changes.')
