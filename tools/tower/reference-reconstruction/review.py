"""Publish a static, original-only review package; no cache files or environments."""
from pathlib import Path
import json,shutil,hashlib,html,re
from PIL import Image,ImageDraw,ImageFont
from evaluate import SELECTED,COMPARISON
from run import R
T=Path(__file__).resolve().parent;S=T.parents[2];P=S/'art/tower/ashhorn-reference-reconstruction'
def copy(src,dest):
    dest=P/dest;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(src,dest)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def figure(path,caption):return '<figure><a href="'+path+'"><img loading="lazy" src="'+path+'" alt="'+html.escape(caption)+'"></a><figcaption>'+html.escape(caption)+'</figcaption></figure>'

v=json.loads((R/'verification.json').read_text());e=json.loads((R/'evaluation.json').read_text());assert v['selected']==SELECTED==e['selected']
candidate=R/'candidates'/SELECTED;P.mkdir(parents=True,exist_ok=True)
for name in ['baseline.json','discrepancy-assessment.json','constraints.json','reference-arrangement.json','evaluation.json','verification.json','reference-workspace.blend']:
    copy(R/name,Path('technical')/name)
for f in (R/'baseline/original-art').glob('*.png'):copy(f,Path('reference/original')/f.name)
for f in (R/'references').iterdir():
    if f.suffix in ['.png','.json']:copy(f,Path('reference/prepared')/f.name)
for label,source in [('previous',R/'baseline/previous'),('reconstruction',candidate)]:
    for f in source.iterdir():
        if f.is_file()and f.suffix in ['.blend','.model','.json','.py']:copy(f,Path('models')/label/f.name)
    native=COMPARISON/'previous/native'if label=='previous'else candidate/'native'
    for f in native.glob('*.png'):copy(f,Path('models')/label/'native'/f.name)
    for f in (native/'reference').glob('*.png'):copy(f,Path('models')/label/'native/reference'/f.name)
    for f in (COMPARISON/label/'closeups').glob('*.png'):copy(f,Path('models')/label/'closeups'/f.name)
    for f in (COMPARISON/label/'clay').glob('*.png'):copy(f,Path('models')/label/'clay'/f.name)
    for f in (COMPARISON/label).glob('*-contours.png'):copy(f,Path('models')/label/f.name)
    copy(COMPARISON/label/'clay.model',Path('models')/label/'clay.model')
history=[]
for c in sorted((R/'candidates').iterdir()):
    if not(c/'iteration.json').exists():continue
    record=json.loads((c/'iteration.json').read_text());history.append(record)
    for f in c.iterdir():
        if f.is_file()and f.suffix in ['.model','.json','.py']:copy(f,Path('history')/c.name/f.name)
    for view in ['front','quarter']:copy(c/'native'/(view+'.png'),Path('history')/c.name/(view+'.png'))
(P/'technical/history.json').write_text(json.dumps(history,indent=2)+'\n')
notes={
 'status':'Original static reconstruction study, ready for review; artistic fidelity not approved and no production integration.',
 'selected':SELECTED,'preserved_candidates':len(history),'canonical_reference':'reference/original/turnaround-v2.png',
 'improvements':['Broader reference-derived forequarter and paw envelope; the old numeric width is superseded.',
 'Explicit cranial topology, oblique brow, closed orbital cups, nasal and cheek planes, a deeper lower jaw.',
 'Backward-swept banded long horn on the illustrated side, fractured short horn, folded asymmetric ears.',
 'Continuous anatomical torso/limbs, narrower sternum, differentiated forearms and bent knee/hock chains.',
 'Five real dorsal shells and four shoulder shells with dense interior clearance and narrow worn rims.',
 'Raised curved tail, underside color, twelve separate toes and twelve exposed curved claws.'],
 'remaining_gaps':['The face remains an angular approximation: forehead, cheek transition, socket shape and eye expression still fall short of the illustration.',
 'Shoulder shields and dorsal shingles remain too regular and lack the reference\'s authored wear and local plane hierarchy.',
 'Chest/limb color boundaries remain coarse; the full boundary-cut trial was rejected because native quantization introduced thin-triangle artifacts.',
 'The reference\'s thigh/calf silhouette, paw weight and fine muscle planes are not fully reproduced.',
 'A single coherent static stance cannot exactly match all four inconsistent drawings. Tail, rear legs and quarter-view perspective remain approximate.',
 f'Native export has {v["topology"]["boundary_edges"]} boundary edges and {v["conversion"]["collapsed_triangles"]} collapsed source triangles, largely fine details/seams. No watertight or production-ready claim is made.'],
 'rejected_trials':{'04':'Unrestricted projection mixed nearby tan, claw and plate colors into the hide.',
 '06-08':'Sparse shell controls allowed shoulder surfaces to disappear into the body.',
 '08':'Palette assignment erased the orbital cup\'s dark material; fixed in09.',
 '14':'Global boundary cuts created sliver triangles and visible native shading artifacts; retained as a rejected experiment.',
 '15':'Fresh-build triangle order varied after Blender tessellation;16 canonicalizes oriented face order without changing the surface.'},
 'rendering':'Unchanged native renderer, ambient72/contrast850, light[-30,-50,-30], common cameras; no enhanced render, photo billboard or source image texture.',
 'scope':'No animation, gameplay, NPC definition, live map, cache mutation or production content changes.'}
(P/'technical/artistic-assessment.json').write_text(json.dumps(notes,indent=2)+'\n')
style='''body{margin:0;background:#10191e;color:#d9dfdf;font:17px/1.6 system-ui}main{max-width:1640px;margin:auto;padding:36px}h1{font:clamp(42px,5vw,76px)/1.05 Georgia;margin:20px 0}h2{font:34px Georgia;margin-top:65px}h3{font-size:21px}.eyebrow{color:#cda16b;letter-spacing:.15em;text-transform:uppercase}p{max-width:1000px}a{color:#e6bc83}nav{display:flex;gap:22px;flex-wrap:wrap}section{border-top:1px solid #39454b;margin-top:40px}.grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:15px}.pair{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:16px}figure{margin:0;background:#0f151a;padding:12px}img{width:100%;height:auto;display:block}figcaption{font-size:14px;color:#aebfc3;padding:10px 0}table{border-collapse:collapse;width:100%}th,td{padding:12px;text-align:left;border-bottom:1px solid #39454b}aside{border-left:3px solid #bc8850;padding:12px 22px;background:#1a252a}details{margin:15px 0}code{color:#edc59c}li{max-width:1100px;margin:9px 0}.history{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:15px}@media(max-width:850px){main{padding:20px}.grid,.pair,.history{grid-template-columns:1fr}table{font-size:13px}}'''
parts=['<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Ashhorn — Reference Reconstruction</title><style>'+style+'</style><main>',
 '<div class="eyebrow">ASCENT · Static asset research · 18 October2026</div><h1>Reconstruct the existing design.</h1><p>The original Ashhorn turnaround remains the artistic target. This study rebuilds the anatomy around named measurements, a calibrated Blender workspace and independently authored surfaces.</p>',
 '<aside><strong>Better measured outline. Artistic fidelity remains unapproved.</strong><p>The candidate improves the broad proportions and several recognizable structures, but the face, armor and anatomical planes still fall short of the concept. This is an experimental review checkpoint, with no live integration.</p></aside>',
 '<nav><a href="#matched">Matched views</a><a href="#close">Close inspection</a><a href="#clay">Geometry</a><a href="#measure">Measurements</a><a href="#history">Iterations</a><a href="#source">Editable source</a></nav>',
 '<section><h2>The unchanged canonical sheet</h2>'+figure('reference/original/turnaround-v2.png','Untouched1536×1024 original. No replacement concept was generated.')+'</section>',
 '<section id="matched"><h2>Drawing → Previous mesh → Reconstruction</h2><p>One frozen perspective camera per drawing, shared by both meshes. Images retain their original viewport and pixel dimensions. Click any image to inspect it at full size. The drawings contain incompatible poses and perspective; these fits are estimates.</p>']
for view in ['front','left','rear','quarter']:
    parts.append('<h3>'+view.title()+'</h3><div class="grid">'+figure('reference/prepared/'+view+'-panel.png','Original drawing; background removed only in the modeling copy.')+figure('models/previous/native/reference/'+view+'.png','Previous concept-guided mesh — common camera.')+figure('models/reconstruction/native/reference/'+view+'.png','New static reconstruction — common camera.')+'</div>')
parts.append('</section><section id="close"><h2>Native close views</h2><p>Each pair uses identical submission, lighting and camera settings. These are actual native model renders.</p>')
for name in ['face-front','face-quarter','shoulder','forepaw','hock']:
    parts.append('<h3>'+name.replace('-',' ').title()+'</h3><div class="pair">'+figure('models/previous/closeups/'+name+'.png','Previous mesh')+figure('models/reconstruction/closeups/'+name+'.png','Reconstruction')+'</div>')
parts.append('</section><section id="clay"><h2>Real geometry, without color assistance</h2><p>The gray model has exactly the same vertices, faces and native shading flags. Only face colors change. Reference images and construction cages never enter the native export.</p>')
for view in ['front','quarter','side','rear','above','gameplay']:
    parts.append('<h3>'+view.title()+'</h3><div class="pair">'+figure('models/previous/clay/'+view+'.png','Previous — uniform gray')+figure('models/reconstruction/clay/'+view+'.png','Reconstruction — uniform gray')+'</div>')
parts.append('</section><section id="measure"><h2>Measured evidence and its limits</h2><p>Outline overlap rises in all four frozen views. This is not a perceptual-quality score, proof of anatomical correctness or an approval threshold. Mask cleanup, hidden surfaces and the inconsistent illustrations limit its interpretation.</p><table><tr><th>Drawing view</th><th>Previous IoU</th><th>New IoU</th><th>Mean contour distance, previous → new</th></tr>')
for view in ['front','left','rear','quarter']:
    a=e['models']['previous']['views'][view];b=e['models']['reconstruction']['views'][view]
    parts.append(f'<tr><td>{view}</td><td>{a["silhouette_iou"]:.3f}</td><td>{b["silhouette_iou"]:.3f}</td><td>{a["contour"]["mean_pixels"]:.1f} → {b["contour"]["mean_pixels"]:.1f}px</td></tr>')
parts.append('</table><p>57 named3D estimates; per-observation uncertainty and camera residuals are recorded in <a href="technical/constraints.json">constraints.json</a>. The fitted side view has low elevation; elevated standard views expose the armor surfaces as well.</p><div class="pair">'+figure('reference/prepared/front-landmarks.png','Front calibration: observed estimate, projected framework and residual.')+figure('reference/prepared/left-landmarks.png','Side calibration: estimates, not exact ground truth.')+'</div><div class="pair">'+figure('models/previous/front-contours.png','Front outline: reference teal; previous orange.')+figure('models/reconstruction/front-contours.png','Front outline: reference teal; reconstruction orange.')+'</div>')
parts.append(f'<h3>Integrity checks</h3><p>{v["topology"]["vertices"]:,} vertices · {v["topology"]["triangles"]:,} native triangles; previous mesh 3,621 triangles. Saved Blender source re-exports byte for byte. A fresh build reproduces the native model and all ten whole-model renders exactly. Native vertices, faces, colors and shading flags decode exactly. Twelve toes, twelve claws, five dorsal and four shoulder shells are counted. No image textures, armature, animation actions or skin labels participate.</p><p>All 23 frozen baseline files and the five other recorded workspaces are unchanged. The original workbench executable hash is unchanged. A small new camera adapter submits image offsets to the existing native camera API; no renderer implementation changed.</p><p>The export reports {v["conversion"]["collapsed_triangles"]} collapsed source triangles and {v["topology"]["boundary_edges"]} boundary edges, with zero remaining degenerate triangles and zero nonmanifold index edges. Separate shells and fine quantized details are not claimed to form a watertight production asset. See <a href="technical/verification.json">full verification</a>.</p>')
parts.append('<h3>What still needs artistic work</h3><ul>'+''.join('<li>'+html.escape(n)+'</li>'for n in notes['remaining_gaps'])+f'</ul></section><section id="history"><h2>Preserved iterations</h2><p>{len(history)} mesh checkpoints are retained. Full Blender files remain in the private experiment; published history includes original models, construction snapshots and native renders. Candidate 14 is explicitly rejected. Candidate 16 freezes face ordering; candidates 17–18 refine facial proportions and jaw exposure.</p><div class="history">')
for record in history:
    name=record['label'];parts.append(figure('history/'+name+'/quarter.png',name+' ·'+str(record['triangles'])+' triangles'))
parts.append('</div></section><section id="source"><h2>Editable source and audit</h2><ul><li><a href="models/reconstruction/source.blend">Reconstructed static Blender model</a> — packed original guides, editable anatomical surfaces, named controls and cameras.</li><li><a href="technical/reference-workspace.blend">Reference workspace before creature geometry</a> — original guides, framework and orthographic working cameras.</li><li><a href="models/reconstruction/asset.model">Native static model</a> · <a href="models/reconstruction/geometry.json">decoded geometry</a> · <a href="models/previous/source.blend">immutable previous Blender source</a></li><li><a href="technical/discrepancy-assessment.json">Fourteen-category diagnosis written before geometry</a> · <a href="technical/artistic-assessment.json">artistic assessment and rejected trials</a></li><li><a href="technical/baseline.json">Frozen source/tool identities</a> · <a href="technical/evaluation.json">measurement data and limitations</a></li></ul><p>No cache archives, cache models, texture files, environment pixels, binaries or runtime palettes are included in this original-only publication. No map, production registry or gameplay content was changed.</p></section></main></html>')
(P/'index.html').write_text(''.join(parts),encoding='utf8')
overview=Image.new('RGB',(1800,1540),'#10191e');draw=ImageDraw.Draw(overview);font=ImageFont.truetype('C:/Windows/Fonts/arial.ttf',25)
for i,title in enumerate(['Canonical illustration','Previous mesh','Reconstruction study']):draw.text((i*600+20,20),title,font=font,fill='#e5c18f')
for row,view in enumerate(['front','left','rear','quarter']):
    paths=[P/'reference/prepared'/(view+'-panel.png'),P/'models/previous/native/reference'/(view+'.png'),P/'models/reconstruction/native/reference'/(view+'.png')]
    for col,path in enumerate(paths):
        im=Image.open(path).convert('RGBA');im.thumbnail((585,340));x=col*600+(600-im.width)//2;y=85+row*360+(340-im.height)//2;overview.paste(im,(x,y),im)
    draw.text((8,65+row*360),view,font=font,fill='#aebfc3')
overview.save(P/'comparison-overview.png')
# File-level check without executing a browser or using an alternate surface.
page=(P/'index.html').read_text(encoding='utf8');refs=re.findall(r'(?:src|href)="([^"]+)"',page)
assert all((P/url).is_file()for url in refs if not url.startswith('#'))
for f in P.rglob('*.png'):
    with Image.open(f)as im:im.verify()
files={str(f.relative_to(P)).replace('\\','/'):sha(f)for f in P.rglob('*')if f.is_file()and f.name!='manifest.json'}
(P/'manifest.json').write_text(json.dumps({'scope':'Original Ashhorn art, models and original-only native renders; no proprietary cache resources','selected':SELECTED,'files':files},indent=2)+'\n')
print('REVIEW_PACKAGE',P,'files',len(files),'links',len(refs),'images verified; browser interaction not performed',flush=True)
