# Ashhorn reference reconstruction

This isolated static study reconstructs the existing original Ashhorn turnaround. It does not generate another concept, replace production content, or establish artistic approval.

The review package is at `art/tower/ashhorn-reference-reconstruction/index.html`. Its full assessment records the remaining facial, armor and anatomical gaps. The prior model and original drawings remain immutable comparisons.

## Inputs and coordinates

Runtime inputs live outside the repository at the sibling `ashhorn-reference-private` directory. `baseline.py` records the original sources and hashes **once, before authoring**. Do not rerun it to replace the frozen manifest. `prepare.py` extracts the existing art; `constraints.py` estimates cameras and landmarks; `workspace.py` creates the initial reference workspace. These stages precede creature geometry.

Author coordinates are X viewer-right in the front drawing (anatomical left), Y rearward, Z up; ground Z0 and128 units per tile. The native exporter maps this to `[X,-Z,Y]`. Laterality follows the illustration. Uniform perspective cameras are estimated and frozen; the sheet is not an exact orthographic turnaround. Hidden landmarks and conflicting poses have explicit uncertainty. Local modeled refinements such as the final knee/hock adjustment are recorded in the build snapshot rather than asserted to be measured ground truth.

The runtime reference folder must contain the published `technical/constraints.json`, `technical/reference-workspace.blend`, and prepared original images under `references/`. The frozen preservation audit additionally requires the original baseline files and other recorded workspaces. No cache resources are supplied by this package.

## Build and inspect

From this server worktree, with Blender5.2.2 installed at the path in `run.py`:

```powershell
python tools/tower/reference-reconstruction/run.py 18 unique-review-label
```

Candidate output directories must be new; the script refuses to overwrite preserved versions. `build.py` creates independently authored anatomical surfaces, facial topology, true solid plates, paws and claws. The saved `.blend` includes packed original image guides and static measurement controls, excluded from export. It has no armature or animation actions.

`paint.py` assigns native face colors using anatomical material classes and restrained visibility-aware source color variation. It does not use a photo texture, billboard or inverse-light bake. The rejected unrestricted projection and global color-boundary cut remain in the iteration history. Final face ordering is canonicalized after Blender triangulation, preserving oriented triangles and their materials.

The frozen `asset_workbench.exe` supplies six standard views. The small `reference_mesh_view` adapter adds viewport offsets through the existing native camera API for four drawing views. The native rasterizer, lighting implementation, model codec and production client remain unchanged. All model views use ambient72, contrast850 and light `[-30,-50,-30]`.

Build the camera adapter with existing offline dependencies:

```powershell
$env:CARGO_TARGET_DIR='../build/concept-ab'
cargo build --offline --release --manifest-path tools/tower/offline/render/Cargo.toml --bin reference_mesh_view
```

`evaluate.py` compares the frozen previous model and selected reconstruction using common cameras, uniform-gray models, five component close views and estimated silhouette/contour measurements. `verify.py` checks preserved hashes, other workspace states, saved-source re-export, fresh generation, exact native decode and ten pixel-identical rebuild renders. Its validation folder must also be new. `review.py` curates original-only source and evidence, verifies image integrity and local links, and writes a file-hash manifest. It does not open or operate a browser.

No caches, texture archives, protected environment screenshots, executables or palettes are published. The native cache is a read-only dependency for the existing renderer. No live NPC definition, animation, gameplay, AI, enhanced renderer or map integration is included.

## Review result

Eighteen preserved candidates culminate in 4,331 vertices and 8,037 native triangles. The previous mesh has 3,621 triangles. Projected silhouette IoU improves from 0.631→0.790 front, 0.592→0.792 side, 0.535→0.689 rear and 0.493→0.661 quarter. These approximate outline measurements do not measure artistic quality. The face, armor detailing, muscle planes and interpretation of inconsistent drawings remain unresolved artistic limitations.

The selected source re-exports byte for byte; a fresh build and ten native renders reproduce exactly. All 23 frozen baseline files and five recorded neighboring workspaces remain unchanged. Four hundred source triangles collapse on the integer lattice; 252 of those are small eye/iris triangles. The exported model has no degenerate triangles or nonmanifold index edges, but 443 boundary edges. It is not claimed watertight or production-ready.
