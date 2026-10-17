# Springhouse Service Suite — scene calibration review

This is one connected sanctuary/workshop candidate for artistic review. Full-floor production is paused. The latest complete-floor study, earlier concepts, gameplay baseline and native/headless pipeline evidence remain preserved. Nothing here is approval to replace the live map or extend this treatment across Floor One.

Open `index.html` for original-only public review. The local `scene-fidelity/review.html` also places genuine cache scenes beside the candidate. Those reference pixels remain local because distribution permission is not assumed.

## What changed

Three real cache environments were reconstructed from terrain and original map placements, rather than assembling isolated references. Their geometry/material inspection and matched native views exposed several separate causes of the difference: camera projection, model-based ground instead of native terrain, missing surrounding context, oversized/sparse interiors, absent small occupational detail and inappropriate normal handling for closed walls.

The candidate is a compact occupied springhouse connected through a usable threshold to repair, furnace and quenching stations. It uses original native terrain colors/heights, maintained sleeping/seating/storage pockets, a wages/ledger station, correctly oriented hot work, original wall fixtures and restrained edge debris. Original native rendering remains authoritative. Blender provides source geometry and topology inspection, not acceptance beauty shots.

The retained package has58 original models/6,596 unique triangles.38 families appear in93 placements;4,096 authored terrain tiles provide the small suite's surrounding context. Six new families retain IDs1100081–1100086/63081–63086. Source geometry, native outputs, manifests, an editable original parent kit and sample additions are included. Exact native bytes and registry were reproduced after fresh headless Blender generation.

## Evidence to review

- [Current suite](views/suite-quarter.png), [sanctuary](views/sanctuary-quarter.png), [workshop](views/workshop-quarter.png) and [reverse](views/suite-reverse.png).
- [Unchanged full-floor baseline under matched camera](views/baseline-quarter.png) and [earlier small sample at that camera](views/previous-suite-quarter.png).
- [Uniform terrain material control](views/control-uniform-material.png) and [flat-height control](views/control-flat-height.png).
- [Rejected all-smooth wall experiment](views/rejected-all-smooth.png) and [rejected selective smooth experiment](views/rejected-selective-smooth.png).
- Original close comparisons: [rack before](asset-comparisons/rack-before.png) / [after](asset-comparisons/rack-after.png), [rock before](asset-comparisons/rock-before.png) / [after](asset-comparisons/rock-after.png). These use an explicitly different close-inspection camera, held fixed within each pair.
- [Actual scene inventory and source hashes](SCENE_REFERENCE_INVENTORY.json), [reconstruction method](SCENE_RECONSTRUCTION.md), [camera/renderer comparison](RENDERER_CAMERA_COMPARISON.md), [geometry/material analysis](GEOMETRY_AND_MATERIAL_ANALYSIS.md), [visual difference ledger](VISUAL_DIFFERENCE_LEDGER.md).

The final set contains nine authentic-reference and fifteen original-only frames. All24 report complete requested source regions and no pending models. The references contain real textures where applicable; the original candidate is deliberately an opaque, untextured original-art package. This difference remains visible and documented.

## What remains unresolved

The walls still approximate the original surface/normal treatment. Simply copying smooth render-type values onto the closed custom topology produced poorer results and was rejected. The broader cave boundary is smoother and less layered than the originals. Original texture support, more precise dirt/wear, water animation and actual animated player-scale validation remain potential follow-ups. The static1.72-tile ruler is labeled; a separate genuine human NPC bind-pose measurement is included and is not relabeled as a live player.

These are real limitations, not concealed by a different light rig, exposure or background. Passing model/terrain tests establishes compatibility, not artistic coherence. The next decision is the user's review of this **small sample**, including whether the remaining surface, boundary and density differences meet the desired standard.

## Technical checks and scope

Every native vertex/index/color/render-type, model binding, footprint and scene-shading flag validates. The native terrain contract passes, including malformed/duplicate ID rejection, cache-texture rejection and preventing study-floor installation through ordinary cache readers. Regeneration produces identical native model/definition bytes and registry. The50 protected cache files retain every original SHA256 across453,219,756 bytes.

The original server checkout remains clean at `f9e4433f2451235805f85fe379d351d095b7535b`; the original client checkout remains clean at `ff9cef17fd1ce3ec2ebf8d1181fee31411999c4c`. Experiments stay on isolated `experiment/fallen-cinderworks-design` branches. No gameplay, persistence, character database, live map, ORACLE parity branch or FORGE branch is changed. No new live client/server session is used for this calibration.

## Reproduce in a clean isolated workspace

Use sibling `rust-server` and `rust-client-reference` experiment checkouts, a Rust toolchain, Blender5.2.2 LTS and Python/Pillow. Authorized original cache resources are required only for genuine reference reconstruction. They and all extracted reference meshes/images must remain outside the repositories.

1. Build `tools/tower/offline/render/Cargo.toml` with `cargo build --release --bin tower-scene-render`. It depends on the sibling client. Build client examples `custom_kit_validate` and `original_terrain_contract` for native validation.
2. The published `original-assets` is already a complete original native package with registry and expected geometry. It needs no protected cache for original-only rendering. Copy the candidate scene/manifest and optional terrain controls into a local candidate directory.
3. To regenerate geometry, rebuild the preserved parent original kit using `blender/create_calibrated_kit.py --full --layout docs/tower/art/full-floor-unapproved/design-manifest.json`, then run `blender/create_scene_sample_assets.py --input ABS_PARENT_GENERATED --output ABS_SAMPLE_GENERATED --wall-shading flat` headlessly. The latter reads only original Tower GLBs. Convert with `converter/package_cinderworks.py SAMPLE_GENERATED SAMPLE_PACKAGE`.
4. Run `design/compose_scene_calibration.py SAMPLE_PACKAGE LOCAL_CANDIDATE --refined`. Run the native validators against that package; the terrain validator may also take CACHE_JS5 to confirm ordinary-cache rejection.
5. `design/render_scene_fidelity.py RENDERER CACHE_JS5 SAMPLE_PACKAGE LOCAL_REVIEW_ROOT --candidate LOCAL_CANDIDATE --baseline PRESERVED_FULL_SCENE --baseline-package PRESERVED_FULL_PACKAGE --pitch 2560` renders matched references, baseline, candidate and controls. It creates the three `N`-only reference recipes if absent and refuses silently to replace a different source recipe. Supply `--previous-candidate` only when the earlier local sample is available.
6. Preserve raw BMPs, sidecars, native logs and manifests locally. Use `publish_scene_calibration.py` only with the explicit original-only whitelist; never publish reference images, placement TSVs, extracted JSON arrays or reference Blender projects.

The two published original `.blend` files are editable source artifacts; generation scripts remain the reproduction authority. Binary `.blend` file identity is not promised, while native model/definition/registry identity was actually verified. ATELIER can reuse the modular import, generation, conversion and review steps without account/database state.

Do not resume a complete-floor rebuild or promote this candidate until the user has provided artistic review.
