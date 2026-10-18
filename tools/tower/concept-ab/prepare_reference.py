"""Noncreative panel extraction and explicit world calibration of original art."""
from pathlib import Path
import json,hashlib,datetime
from PIL import Image
T=Path(__file__).resolve().parent;R=T.parents[3]/'concept-ab-private';D=R/'concept'
im=Image.open(D/'turnaround-v2.png')
panels={'front':[0,0,584,512],'left':[584,0,1536,512],'rear':[0,512,584,1024],'quarter':[584,512,1536,1024]}
for k,b in panels.items():im.crop(b).save(D/(k+'.png'))
data={
 'accepted_for':'Visual planning of planes, anatomical relationships and material placement; NOT accepted as exact geometric ground truth or artistically approved production character',
 'accepted_image':'turnaround-v2.png','approved_internally_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'raw_image_size':list(im.size),'panel_crops':panels,
 'canonical_priority':'The frozen specification wins over generated-image inconsistencies. No enlargement of the frozen width or high tail curl is allowed in the mesh.',
 'consistency':{
 'horn_count_and_laterality':'Two; long anatomical left is viewer right in front and viewer left in rear. Pass.',
 'ear_eye_placement':'Two ears, lateral eyes under brows. Front and side differ in brow width; canonical skull width43.',
 'limbs':'Four. Side drawing uses staggered stance rather than exact orthographic neutral pose; canonical paired landmarks are used.',
 'toes':'Hero and turnaround-v1 include four-toe forepaws. Targeted edit v2 corrects the clearly visible paws to three. Occluded far toes cannot all be counted; mesh has exactly12 independently named toes/claws.',
 'proportions':'Front forequarters/paws are too wide relative to height. Do not anisotropically warp images or expand geometry to improve overlap. Canonical width92, shoulder72, waist42.',
 'tail':'Generated side tail tip is aboutZ107 rather than81; rear curvature also disagrees. Canonical root[0,76,83], tip[26,190,81] overrides images.',
 'plates':'Five low dorsal shields identifiable in rear; side silhouettes overlap. Shoulder cap is interpreted as one two-layer assembly per side, not extra dorsal shields.',
 'color':'Mottled texture is not modeled. Eight frozen face-color materials apply identically to both runs; native lighting will differ from painted illumination.'},
 'calibration':{
 'front':{'pixel_origin_ground':[273,493],'pixels_per_native_unit':2.70,'world_axes':['X','Z'],'uncertainty':'Estimated +/-4pixels for landmarks; width conflict is genuine, not fitted away'},
 'left':{'pixel_nose_ground':[42,490],'pixels_per_native_unit':2.72,'world_axes':['Y','Z'],'nose_native_y':-140,'uncertainty':'Pose is slightly three-quarter and rear feet are staggered'},
 'rear':{'pixel_origin_ground':[273,469],'pixels_per_native_unit':2.70,'world_axes':['-X','Z'],'uncertainty':'Tail occludes centerline; front and rear stances differ'}},
 'canonical_landmarks':{'nose':[0,-140,109],'crown':[0,-100,132],'cheek':[21,-104,111],'withers':[0,-46,132],'waist':[0,22,111],'hip':[0,63,112],'fore_elbow':[33,-31,55],'fore_wrist':[34,-45,17],'rear_knee':[33,40,54],'rear_hock':[31,86,27],'left_horn_tip':[-30,-54,176],'right_horn_break':[25,-82,150],'tail_tip':[26,190,81]},
 'image_sha256':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in D.glob('*.png')},
 'no_protected_inputs':True,
 'limitations':'Generated art was automatically displayed by the image tool. Final model labels are randomized and concealed, but user exposure to the concept can weaken blinding. The maker cannot be blinded; there is one sequential trial per method.'}
(D/'consistency.json').write_text(json.dumps(data,indent=2)+'\n')
print('Reference interpretation frozen with explicit conflicts; no competing geometry read.')
