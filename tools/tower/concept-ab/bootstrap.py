"""Freeze the original design and audit held tooling without modifying it."""
from pathlib import Path
import json,hashlib,subprocess,datetime
E=Path(__file__).resolve().parents[4]
R=E/'concept-ab-private'
R.mkdir(exist_ok=True)
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def git(p,*args):return subprocess.check_output(['git',*args],cwd=p,text=True).strip()
design={
 'schema':1,'name':'Ashhorn Stalker','role':'Original static hostile tunnel predator of the abandoned Cinderworks; visual study only',
 'coordinates':'Native authoring units: 128 per tile; X creature right, Y rearward, Z up; ground Z=0',
 'dimensions':{'nose_y':-140,'rump_y':80,'tail_tip_y':190,'shoulder_height':132,'hip_height':112,'maximum_horn_height':176,'torso_half_width_shoulder':36,'waist_half_width':21,'paw_outer_width':92,'tolerance_fraction':0.10},
 'body_plan':'Exactly four legs. Heavy but athletic forequarters, sloping back, deep narrow sternum, drawn-up waist, compact pelvis. No wings, extra limbs, saddle, gear or fire particles.',
 'head':{'description':'Low wedge-shaped skull, broad zygomatic cheeks, defined brow, long but broad blunt muzzle, distinct lower jaw and dark mouth seam. Small lateral amber eyes below horns. Two swept triangular ears behind horns.','nose_y':-140,'occiput_y':-78,'muzzle_width':28,'skull_width':43,'jaw_low':84,'crown_high':132},
 'horns':{'count':2,'anatomical_left':'Long backward-curving horn rising from left brow X=-19, Y=-91, Z=125; arcs outward to X=-30 and back to Y=-54, tip Z=176. Three charcoal growth bands, pale worn tip.','anatomical_right':'Broken horn X=19 Y=-91 Z=125; broad stump to Z=150 with dark irregular fracture cap. Not a second full-length horn.'},
 'torso':{'shoulder_y':-46,'chest_bottom':49,'withers':132,'waist_y':22,'waist_bottom':66,'waist_top':111,'hip_y':63,'hip_top':112,'description':'Raised scapular mass tapering into an arched lumbar region; no spherical torso or separate floating shoulders.'},
 'limbs':{'fore':'Elbows at about (+/-33,-31,55), straight lower forelegs to (+/-34,-45,17), heavy wrist; muscular shoulders attach continuously to torso.','hind':'Thigh (+/-27,61,79), forward knee (+/-33,40,54), rear hock (+/-31,86,27), paw (+/-31,67,8); digitigrade silhouette.','feet':'Exactly three distinguishable broad toes per paw, each with one short forward-curving ivory claw. Four paws, twelve toes and twelve claws. Forepaws larger than hindpaws.'},
 'tail':{'length':115,'root':[0,76,83],'path':'Tapers to Y=190, curls slightly sideways and upward near tip; dark unarmored tail, no mace or flame.'},
 'plates':{'dorsal_count':5,'dorsal_centers_y':[-48,-23,3,28,55],'description':'Five overlapping low basalt dorsal shields, largest at shoulders and smaller toward hip; rust-brown warm edges. One natural layered shoulder cap each side, no spines along entire back. Armor grows from hide, no metal straps.'},
 'palette_rgb':{'hide':[0.24,0.25,0.24],'ventral':[0.37,0.33,0.28],'plate':[0.27,0.19,0.14],'plate_edge':[0.49,0.33,0.20],'horn':[0.64,0.57,0.42],'claw':[0.71,0.66,0.52],'dark':[0.10,0.105,0.10],'eye':[0.89,0.46,0.09]},
 'silhouette':{'primary':'Forward-weighted quadruped with sharply rising asymmetric horns, dipping waist, bent rear hocks and long slender tail.','secondary':'Wedge muzzle, strong cheek and brow, compact layered shoulder caps, five low dorsal plates, three blunt claws visible on every paw.'},
 'surface':'Scorched leathery hide, angular brows, sparse deliberate creases, warm dust on ventral surfaces. Face-color materials only; no photoreal fur, emissive effects or painted billboard detail.',
 'complexity':{'target_triangles':[2500,5000],'hard_limit_triangles':6500,'static_only':True,'textures':False,'rigs':False,'source_mesh':'Factory-empty Blender scene, independently authored for each method. No prior original creature mesh or cache mesh imported.'},
 'shared_observation':{'native_ambient':72,'native_contrast':850,'light':[-30,-50,-30],'palette_brightness':0.8,'camera_focus_height':84,'camera_distance':420,'reference_library':'Held revision 239.1 cow2790 wolf106 goblin655 scimitar1323 native images; read-only style observation for both methods'},
 'interpretation_policy':'Approximate dimensions have 10% tolerance; structural counts, horn laterality, material palette and body plan are invariant. Record deviations rather than silently revising this specification.'
}
p=R/'design.json'
if p.exists():raise SystemExit('Design already frozen; refusing overwrite')
p.write_text(json.dumps(design,indent=2)+'\n')
(R/'design.sha256').write_text(sha(p)+'\n')
renderer=E/'astra-benchmark/tools/asset_workbench.exe'
bl=Path('C:/Program Files/Blender Foundation/Blender 5.2/blender.exe')
audit={'started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'frozen_design_sha256':sha(p),'renderer_sha256':sha(renderer),'blender_version':subprocess.check_output([str(bl),'--version'],text=True).splitlines()[:6],'sources':{},'files':{}}
for n in ['constrained-server','concept-ab-server','enhanced-client','rust-client-reference','rust-server','reconstruction-server']:
    audit['sources'][n]={'head':git(E/n,'rev-parse','HEAD'),'branch':git(E/n,'branch','--show-current'),'status':git(E/n,'status','--porcelain')}
for p in (E/'concept-ab-server/tools/tower/constrained').glob('*.py'):audit['files'][str(p.relative_to(E))]=sha(p)
for p in [E/'workbench/identities.json',E/'workbench/discovery/original-id-bounds.json',E/'concept-ab-server/tools/tower/asset-workbench/native_codec.py',E/'concept-ab-server/tools/tower/asset-workbench/registry.py']:
    audit['files'][str(p.relative_to(E))]=sha(p)
assert audit['renderer_sha256']=='f6e6e12da1a80f7824c9d90a5254964fdc8d738298417c7dcd51e0793caea7cf'
(R/'baseline.json').write_text(json.dumps(audit,indent=2)+'\n')
print('Frozen design',audit['frozen_design_sha256']);print('Baseline audited',len(audit['sources']),'repositories')
