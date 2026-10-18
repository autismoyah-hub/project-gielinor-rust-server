# Reference-constrained static authoring

Reusable Blender geometry controls and fixed-camera native diagnostics. The four
reconstruction studies, their constraints, reference meshes, measurements and
images remain private. This directory contains no cache geometry or target-specific
anatomical recipes. Original reference-free examples are in
`art/tower/constrained-tooling`.

## Scope and result

The development cycle preserves 52 generated trial records (49 distinct native payloads), including 28 bounded
coordinate-search trials, against a frozen six-camera baseline. Four selected
independent studies improve their six-view mean silhouette overlap:

| Study | Previous | Selected | Artistic status |
|---|---:|---:|---|
| Cow | 0.719 | 0.799 | Front view still regresses; not approved |
| Wolf | 0.746 | 0.773 | Skull/ruff still too generic; not approved |
| Goblin | 0.659 | 0.701 | Face, hands and equipment details remain approximate |
| Iron scimitar | 0.706 | 0.768 | Blade/hilt contours still need refinement |

These measurements do **not** establish artistic fidelity or production readiness.
The private review includes native reference/previous/selected comparisons,
history selection, silhouette and overlay diagnostics, 24 fixed-camera Cycles
renders, component close-ups, landmark/occlusion probes, full recipes, source
Blends and explicit limitations. User approval is pending for every category.

This is a substantial expansion of authoring control and regression evidence,
not a claim that the requested artistic standard has been reached. The cow's
front-view tradeoff remains a priority for additional work. The renderer, live
map, animation, gameplay, cache identifiers and production state are unchanged.

## Authoring capabilities

`surface.py` operates only on an independently authored source Blend or a
factory-empty scene and a declarative JSON recipe:

* Separate dorsal and ventral profile lofts with width, height, cross-section,
  center offset and twist; X/Y/Z orientations; species-specific sections.
* Parallel-transport limb rings and controlled joint fusion. Distal ring probes
  reject fusion that erases thin limbs. Pre-fusion meshes remain in a hidden
  editable construction collection excluded from native export.
* Compact-support local sculpt brushes, component transforms and scales,
  landmark displacements, local relaxation with radius recovery, fitted patches
  and deliberate surface-normal material regions.
* Colored surface regions with optional explicit cutting, and conforming coat
  layer copies that do not split the structural body's triangulation.
* Curved centerline sweeps with width, thickness, taper, diamond sections and
  transported or fixed axial section frames. Transported frame sign continuity
  is maintained across a tight hook bend.
* Native HSL materials, optional flat-face treatment and native conversion
  through the existing unchanged codec/exporter.

Recipes are the parameter authority. Saved Blends include semantic control
empties, the complete recipe text, final meshes and retained construction cages.
These are editable snapshots, not a new interactive modifier dependency graph:
regeneration applies parameter edits and recomputes the final surfaces.

The system reuses the prior `reassessment/anatomy.py`, `mesh-study/mesh_tools.py`
and `asset-workbench/native_codec.py` capabilities. It does not read reference
vertices into the authoring mesh. Private observational measurements constrain
meaningful dimensions, poses and profiles; no reference topology is relabeled
as independently authored work.

## Evaluation and selection

`evaluate.py` uses fixed full-image coordinates for all six views. It records
silhouette overlap, missing/excess area, contour agreement at several pixel
radii, significant interior-edge agreement, and display-space region agreement.
The region metric is a chromaticity/value proxy, not verified material semantics.

`search.py` performs bounded coordinate descent over declared recipe fields.
Its proposal utility combines shape, contour, region and edge diagnostics,
worst-view coverage and a regression penalty. A loss of 0.015 silhouette IoU or
0.04 contour F1 in a view is flagged. Scores propose candidates; they do not
automatically approve them. Tradeoffs and final artistic selection are recorded
separately. No arbitrary vertex-coordinate fitting or machine-learning
infrastructure is introduced.

`geometry_audit.py` measures component-mask bounds, topology and coarse convex
cross-section envelopes. These are approximate volume diagnostics, not physical
solid-volume measurements. `landmarks.py` uses named observation anchors, nearest
surface probes and opaque double-sided ray tests at the exact fixed cameras.
Those probes are not guaranteed anatomical correspondences or native picking.

The private ledger stores changed components, complete parameter differences,
reasons, material operations, before/after native evidence, per-view changes,
source/model/engine hashes, and selection decisions. Major variants were
visually inspected; bounded search proposals remain marked as provisional
numerical experiments rather than falsely labeled manual artistic approvals.

## Native authority and cameras

The frozen unchanged native `asset_workbench.exe` SHA-256 is
`f6e6e12da1a80f7824c9d90a5254964fdc8d738298417c7dcd51e0793caea7cf`.
Five close cameras use focal length 1000 and a 1000×900 viewport: front, side,
rear, three-quarter and elevated. A sixth uses the native player-camera
position contract, pitch 2000, yaw 14500, focal 308 and a 765×503 viewport.
Per-case focus, distance, ambient/contrast, coordinate origin and all exact
camera files are frozen with the private baseline. The native light direction
is `[-30,-50,-30]`, palette brightness 0.8. There is no candidate-dependent
centering, scaling or camera fit. Contact sheets use a shared union crop only
for display; full fixed-coordinate images remain available.

`studio.py` matches those camera poses/projections in Cycles/AgX at 32 samples.
Its lights are fixed from the reference bounds and origin, not candidate bounds.
Blender studio output is supplementary; it cannot substitute for native evidence.

## Meaningful rejected trials

* A simplified replacement goblin head improved some numerical shape coverage
  but lost facial identity. The more coherent earlier face construction was
  retained and locally corrected.
* Unbounded material-boundary cuts produced slivers and visibly unstable
  native planes. Stable body topology and separate conforming coat layers were
  preferred; exact coat-edge fidelity remains unfinished.
* A low-density quad-remeshing trial erased thin cow-leg geometry. It is retained
  privately as a failed trial. Distal limb preservation now guards future fusion.
* Flat-face and highly subdivided color-plane treatments made the wolf harsh
  or crumpled. The selected anatomy uses controlled smooth native normals and
  explicit meaningful regions instead.
* A fixed axial scimitar section frame regressed the three-quarter view; the
  selected blade uses a transported profile. The reference-free hooked blade
  then exposed a separate frame-sign flip, now covered by a synthetic test.

## Reproduce original examples

Requires Blender 5.2 (tested 5.2.2 LTS), Python with NumPy and Pillow, and the
existing workbench source dependencies. The recipes do not need game resources.
Use a new output directory; generation refuses to overwrite an existing one.

```powershell
python tools/tower/constrained/cookbook.py work/original-recipes
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' -b --factory-startup --python-exit-code 1 --python tools/tower/constrained/surface.py -- - work/original-recipes/reed-runner.json work/new-reed-runner
python -m unittest discover -s tools/tower/constrained -p test_diagnostics.py -v
```

Headless synthetic surface checks:

```powershell
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' -b --factory-startup --python-exit-code 1 --python tools/tower/constrained/test_surface.py -- work/helper-validation.json
```

`reexport.py` verifies byte-exact export from a saved Blend while excluding
construction cages. `replay.py` re-executes a trusted saved engine snapshot with
its original recipe semantics; early snapshots and current recipes were checked.
Native rendering additionally requires the user's held revision 239.1 cache and
existing Rust workbench executable; neither is distributed here.

## Validation and provenance

The isolated branch starts from reconstruction baseline
`df879b05ee1898cff0eacf0ea0b68d6d66563fc4`. The initial reusable tool revision is
`c140128`. The modeling baseline's original files and hashes remain intact.
Fresh native cache inspection matches the original definition/model payload
identities. All four selected models pass saved-Blend byte re-export, exact native
vertex/face/color/render-flag decode, six-view pixel-identical regeneration, and
fresh recipe-generation byte identity. Five synthetic diagnostic tests and the
headless surface tests pass. Original muzzle-parameter variation leaves the
body geometry unchanged. Early cow/wolf/goblin snapshot replay is byte exact.

Native export records integer-lattice rounding and dropped collapsed source
triangles; no degenerate triangles remain in selected native outputs. Open coat
layers and thin details mean manifold-physics or animation readiness is not
claimed. No rig or action generation is performed.

Only generic source, coarse aggregate findings and independently original
demonstration artifacts are published. The protected reference/reconstruction
package remains outside this Git worktree. ORACLE and FORGE branches are not
modified or merged.
