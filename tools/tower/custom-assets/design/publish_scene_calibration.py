"""Publish an explicit original-only whitelist and safe reference metadata.

Never traverses/copies a reference image directory or extracted model study.
The local gallery may link to reference images; it is not copied to the repo.
"""
import argparse,json,hashlib,shutil,html
from pathlib import Path

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('root',type=Path);p.add_argument('output',type=Path);a=p.parse_args();a.root=a.root.resolve();a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=True)
 c=a.root/'scene-fidelity/candidate-v2';manifest=json.loads((c/'fidelity-manifest.json').read_text());out=a.output;views=out/'views';views.mkdir(exist_ok=True)
 selected=['suite-quarter','suite-front','suite-reverse','sanctuary-quarter','workshop-quarter','control-uniform-material','control-flat-height','baseline-quarter','baseline-front','baseline-old-camera','previous-suite-quarter']
 frames={f['name']:f for f in manifest['frames']}
 for name in selected:
  assert frames[name]['original_only'];source=c/'views'/f'{name}.png';assert sha(source)==frames[name]['png_sha256'];shutil.copy2(source,views/source.name)
 for name in ['candidate.scene','uniform-material.scene','flat-height.scene','candidate-manifest.json']:
  shutil.copy2(c/name,out/name)
 # All of this package is original art. It contains no cache models/textures.
 dest=out/'original-assets';dest.mkdir(exist_ok=True)
 cat=json.loads((c/'package/catalogue.json').read_text())
 for d in cat['assets']:
  assert d['object_id']>=63000 and d['model_id']>=1100000
  name=d['name'];shutil.copy2(c/'generated'/f'{name}.glb',dest/f'{name}.glb')
  for extension in ['model','object']:
   file=f'{name.replace("_","-")}.{extension}';shutil.copy2(c/'package'/file,dest/file)
 validation=dest/'validation';validation.mkdir(exist_ok=True)
 for src in (c/'package/validation').iterdir():
  if src.is_file() and src.suffix in ['.txt','.shading','.definition']:shutil.copy2(src,validation/src.name)
 for file in ['catalogue.json','registry.txt']:shutil.copy2(c/'package'/file,dest/file)
 shutil.copy2(c/'generated/kit-manifest.json',dest/'generation-manifest.json')
 shutil.copy2(c/'generated/scene-sample-additions.blend',dest/'scene-sample-additions.blend')
 shutil.copy2(a.root/'art-redesign/production-v2/generated/calibrated-kit.blend',dest/'parent-original-kit.blend')
 checks={'native-model-validation.txt':'scene-candidate-final-native-validation.log','native-terrain-contract.txt':'original-terrain-contract.log','cache-integrity.txt':'cache-integrity-scene-calibration.log','HUMAN_SCALE_REFERENCE.txt':'human-scale-reference.log'}
 for name,source in checks.items():shutil.copy2(a.root/'evidence'/source,out/name)
 shutil.copy2(a.root/'scene-fidelity/reproduction/result.txt',out/'asset-reproduction.txt')
 (out/'SCENE_REFERENCE_INVENTORY.json').write_text(json.dumps(dict(cache_revision='239.1',source='Authorized staged paired cache; no extracted geometry or reference pixels included',static_windows=manifest['reference_inventory']),indent=2)+'\n')
 safe=dict(manifest);safe['frames']=[f for f in manifest['frames'] if f['original_only']];(out/'CUSTOM_RENDER_MANIFEST.json').write_text(json.dumps(safe,indent=2)+'\n')
 (out/'SHARED_RENDER_CONFIGURATION.json').write_text(json.dumps(manifest['native_shared_configuration'],indent=2)+'\n')
 # Rejected shading experiments are original-only and retain their own labels.
 for version,label in [('candidate-v3','rejected-all-smooth'),('candidate-v4','rejected-selective-smooth')]:
  source=a.root/'scene-fidelity'/version/'suite-first.png';shutil.copy2(source,views/(label+'.png'))
 comparisons=out/'asset-comparisons';comparisons.mkdir(exist_ok=True)
 for name in ['rack-before','rack-after','rock-before','rock-after']:
  shutil.copy2(a.root/'scene-fidelity/asset-comparisons'/(name+'.png'),comparisons/(name+'.png'))
 shutil.copy2(a.root/'scene-fidelity/asset-comparisons/manifest.json',comparisons/'manifest.json')
 style='body{background:#11191b;color:#e4e1d7;font:17px/1.55 system-ui;max-width:1560px;margin:36px auto;padding:0 24px}h1{font-size:34px}h2{margin-top:36px}a{color:#9cc9c1}img{max-width:100%;height:auto;border:1px solid #3e4a47}.pair{display:grid;grid-template-columns:1fr 1fr;gap:18px}figure{margin:0}figcaption,small{color:#bdbcae}details{margin:24px 0}summary{cursor:pointer;color:#acc7bc}.notice{border-left:3px solid #aa8c5e;padding:8px 18px}select{background:#1e2a2a;color:#eee;padding:8px;margin:6px 0 18px} @media(max-width:900px){.pair{grid-template-columns:1fr}}'
 intro='<h1>Cinderworks · scene fidelity calibration</h1><p class="notice">One connected sanctuary and workshop sample, awaiting artistic review. Full-floor production remains paused. Every image below is an offline native Rust render.</p>'
 public=f'<!doctype html><meta charset="utf-8"><title>Cinderworks scene calibration</title><style>{style}</style>'+intro
 public+='<p><a href="SCENE_CALIBRATION_REVIEW.md">Review status</a> · <a href="SCENE_RECONSTRUCTION.md">Reference reconstruction</a> · <a href="RENDERER_CAMERA_COMPARISON.md">Camera and renderer comparison</a> · <a href="VISUAL_DIFFERENCE_LEDGER.md">Difference ledger</a></p><h2>Current sample</h2><img src="views/suite-quarter.png"><div class="pair"><figure><img src="views/sanctuary-quarter.png"><figcaption>Occupied springhouse and rest bay</figcaption></figure><figure><img src="views/workshop-quarter.png"><figcaption>Repair, hot work and quenching</figcaption></figure></div>'
 public+='<details><summary>Matched baseline and iteration</summary><div class="pair"><figure><img src="views/baseline-quarter.png"><figcaption>Unapproved full-floor sanctuary, matched camera</figcaption></figure><figure><img src="views/suite-quarter.png"><figcaption>Small scene sample, same camera parameters</figcaption></figure></div></details><details><summary>Terrain controls and rejected shading experiments</summary><p>Identical placement records and camera; the terrain material or height layer is changed independently. Raised props naturally follow the altered height field in the height control.</p><div class="pair"><img src="views/control-uniform-material.png"><img src="views/control-flat-height.png"></div><p>Rejected all-smooth and selective wall experiments. They are preserved as findings, not promoted.</p><div class="pair"><img src="views/rejected-all-smooth.png"><img src="views/rejected-selective-smooth.png"></div></details><p>Authentic reference images and extracted geometry remain local. Source-safe scene identifiers, hashes, counts and model IDs are in <a href="SCENE_REFERENCE_INVENTORY.json">the inventory</a>.</p>'
 public+='<details><summary>Original asset before / after</summary><p>Native close inspection, separate from the matched gameplay camera.</p><div class="pair"><img src="asset-comparisons/rack-before.png"><img src="asset-comparisons/rack-after.png"><img src="asset-comparisons/rock-before.png"><img src="asset-comparisons/rock-after.png"></div></details>'
 (out/'index.html').write_text(public,encoding='utf-8')
 # Local-only review adds protected cache scenes beside the original candidate.
 local=f'<!doctype html><meta charset="utf-8"><title>Cinderworks · matched scene review</title><style>{style}</style>'+intro
 local+='<p>Shared configuration: 765 × 503 native viewport, pitch 56.25°, focal 308, orbit 1560 with native zoom, brightness 0.8, hidden upper roofs. No actors or UI. Reference scenes use actual terrain and map locations, not object arrangements.</p><h2>Authentic scene / original candidate</h2><select onchange="document.getElementById(\'reference\').src=this.value"><option value="references/industrial-front.png">Industrial architecture</option><option value="references/dungeon-front.png">Stone dungeon</option><option value="references/river-front.png">River terrain and settlement</option><option value="references/industrial-quarter.png">Industrial architecture · second angle</option></select><div class="pair"><figure><img id="reference" src="references/industrial-front.png"><figcaption>LOCAL CACHE REFERENCE · static map reconstruction</figcaption></figure><figure><img src="candidate-v2/views/suite-front.png"><figcaption>ORIGINAL TOWER SAMPLE · same native camera configuration</figcaption></figure></div><h2>Connected sample</h2><img src="candidate-v2/views/suite-quarter.png"><div class="pair"><img src="candidate-v2/views/sanctuary-quarter.png"><img src="candidate-v2/views/workshop-quarter.png"></div>'
 local+='<details><summary>Before / after and controlled comparisons</summary><div class="pair"><figure><img src="candidate-v2/views/baseline-quarter.png"><figcaption>Unchanged full-floor proposal under matched conditions</figcaption></figure><figure><img src="candidate-v2/views/suite-quarter.png"><figcaption>Current small sample</figcaption></figure></div><h3>Terrain material / terrain height controls</h3><div class="pair"><img src="candidate-v2/views/control-uniform-material.png"><img src="candidate-v2/views/control-flat-height.png"></div><h3>Rejected wall-shading experiments</h3><div class="pair"><img src="candidate-v3/suite-first.png"><img src="candidate-v4/suite-first.png"></div></details>'
 local+='<p>The differences ledger retains unresolved texture, wall-normal and surrounding-scene limitations. Passing model/scene checks is not artistic approval. The complete floor stays frozen pending review.</p>'
 local+='<details><summary>Original asset before / after</summary><div class="pair"><img src="asset-comparisons/rack-before.png"><img src="asset-comparisons/rack-after.png"><img src="asset-comparisons/rock-before.png"><img src="asset-comparisons/rock-after.png"></div></details>'
 (a.root/'scene-fidelity/review.html').write_text(local,encoding='utf-8');print('SOURCE_SAFE_SCENE_REVIEW_PUBLISHED',len(selected),'selected original-only frames;',len(cat['assets']),'original models; reference pixels remain local')
if __name__=='__main__':main()
