# ATELIER handoff — production fidelity workbench

Authoring revision `ba9ed6c77555fc4085d33c8182dccf93330c1301`; frozen test revision
`13acb2bdae6c25310d5434e502d69154fb03b6e8`; native client
`23a94f75707f0503a9efe610fe99bfce5089b575`. Read CURRENT_STATUS.md and
HELD_OUT_RESULTS.md first. The workbench supports controlled creation/review;
production art and full Floor One integration are not approved.

## Setup and private resources

Keep the server and `rust-client-reference` as sibling checkouts. The offline
Cargo manifest resolves the client and vendored sha2 via that sibling path.
Build from `tools/tower/offline/render` with
`cargo build --release --bin asset_workbench`. It uses the unchanged real client
library. Python 3.13 + Pillow and Blender 5.2.2 LTS (hash d13f752e3b9c) were tested.
Blender slotted actions matter. Pass `--python-exit-code 1` before `--python` to
make script failures fail the process. No service, login or account is required.

In commands below, replace EXE, CACHE, BLENDER, PRIVATE, BOUNDS and IDENTITIES with
local absolute paths. CACHE is an authorized read-only revision 239.1 JS5 directory.
PRIVATE must be outside both source repository and cache. BOUNDS is the native
inspection's `original-id-bounds.json`; IDENTITIES is the preserved private shared
custom allocator manifest. Do not start an empty allocator alongside existing
custom IDs. It checks original maxima, but separate manifest copies do not
coordinate independent writers. Keep allocation single-writer and merge explicit
reservations before parallel production. NPC reservations are not live definitions.

## Reference inspection and saved Blender preservation

Run from the server root; T below denotes `tools/tower/asset-workbench`.

```text
python T/bench.py --native EXE --cache CACHE --out PRIVATE inspect
python T/bench.py --native EXE --cache CACHE --out PRIVATE --blender BLENDER roundtrip
python T/verify_motion.py PRIVATE/training
python T/motion.py PRIVATE/training/reference PRIVATE/motion-analysis.json
```

The CLI's --catalog selects a declared cohort; --split held_out selects its
held-out records. --reference-root reuses existing private fixtures. Selectors:
object:ID[:SHAPE], npc:ID, item:ID, item:ID:worn. Native inspect-static omits
source actions. Native animate SELECTOR SEQUENCE extracts supported classic
motion. References, decoded meshes/frames/textures, .blend derivatives and direct
reconstruction recipes remain private. Hash/provenance summaries can be published.
Run `python -m unittest discover -s T -p "test*.py"`; set WORKBENCH_TEST_TEMP to
an owned temporary directory if sandbox restrictions require it.

## Owned design, conversion and native validation

```text
python T/production_designs.py PRIVATE/original-recipes
python T/author_workflow.py --recipe PRIVATE/original-recipes/kilnback-rummager.json --out PRIVATE/fresh-kilnback --native EXE --cache CACHE --blender BLENDER --bounds BOUNDS --identities IDENTITIES --asset-key production-original:kilnback-rummager
python T/contact_diagnostics.py PRIVATE/fresh-kilnback PRIVATE/fresh-diagnostics
python T/owned_motion_review.py PRIVATE/fresh-kilnback PRIVATE/fresh-motion-review --previous PRIVATE/previous-kilnback
```

Choose a genuinely fresh --out directory. The workflow builds owned components,
saves/reopens Blender, exports actual artist data, verifies geometry/model bytes,
allocates above original IDs, compiles classic actions, validates all sampled
poses and renders five native views. Omitting --reference makes standalone owned
views without fitting an authentic A camera. --blend SAVED_BLEND --patch PATCH
edits owned data; --replace-actions deliberately regenerates authored actions.
Normal reopen preserves existing Blender channels. Use edit_blender.py's explicit
patch interface, not cache mesh replacement. Native play validates posed results;
the saved custom control rig is not an ordinary armature/native-live-preview claim.

Native commands use `EXE CACHE OUTPUT COMMAND ...`:

```text
render MODEL CAMERA AMBIENT CONTRAST
render-own MODEL AMBIENT CONTRAST
play MODEL CAMERA AMBIENT CONTRAST ACTION_DIRECTORY
package OBJECT_PACKAGE OBJECT_ID SHAPE CAMERA AMBIENT CONTRAST
mask MODEL CAMERA FACE_SELECTION [SCALE_X SCALE_Y SCALE_Z]
scene SCENE_TSV CAMERA [TICK]
texture-stats TEXTURE_ID...
```

Read --help/source signatures for optional native arguments. Files emitted by
original-owned recipes/actions are publishable; native source/reference payloads
are not. Stable IDs reproduce within this experiment only; carry its allocator.

## Calibration, component masks and sealed evaluation

`analyze.py library` extracts descriptors. `fit_semantic.py` is an importable
builder consumed by the sealed evaluator; it has no generic CLI. `instantiate.py`
is the older archetype CLI. `component_review.py MANIFEST --native EXE --cache CACHE
--out FRESH` requires explicit private A/C geometry/model/image/camera paths and
semantic bindings. `motion_review.py MANIFEST OUT` compares matching source/owned
trajectories; inspect its manifest schema. Geometric contact and component masks
do not prove collision or artistic correctness. Diagnostic inputs stay private.

production_eval_catalog.json is the declared new cohort. Before target inspection,
freeze identities, rules, source revision/module hashes and native hash. To
reproduce the historical sealed study, use a separate checkout of 13acb2b for
--authoring-root, with the same native binary and original fixtures:

```text
python T/production_evaluate.py --authoring-root FROZEN_TOOL_DIRECTORY --native EXE --cache CACHE --blender BLENDER --out PRIVATE/new-sealed-run --bounds BOUNDS --identities IDENTITIES --prior-reference PRIVATE/prior-reference-root
```

Repeat --prior-reference for every prior corpus (base/worn/combat/old-heldout),
not just the convenient subset. The evaluator rejects changed hashes and existing
results and records mesh-payload overlap, reference IDs/hashes, technical outcomes
and UNAPPROVED/UNSUPPORTED/BLOCKED status. Never fix and overwrite a frozen
candidate. Later development goes to a separately named directory; a new held-out
claim needs a newly sealed cohort. This cycle has no post-fix fresh cohort.

## Mixed showcase and review

Create a private JSON object mapping sluice-keeper-arcade, counterweight-hoist,
kilnback-rummager and showcase-terrain to their workflow output roots.

```text
python T/showcase.py --native EXE --cache CACHE --assets PRIVATE/assets.json --out PRIVATE/fresh-scene
python T/build_production_review.py PRIVATE/review-manifest.json PRIVATE/review
```

showcase.py calls the actual native scene renderer for five cameras and three
motion clips. build_production_review.py takes intro/notice/sections containing
notes, cards with media/link URLs, and optional tables. URLs are relative to the
output index.html. It only assembles a local review; it does not publish a site.
build_review.py remains available for the previous interactive A/B/C/time slider
manifest. Keep both galleries and all failed studies private and immutable.

The source-safe release is art/tower/production-fidelity: original recipes,
models, geometry, object packages, native classic actions, standalone views/GIFs
and hashes. No protected fixture or mixed screenshot is included. Do not replace
live maps, deploy, or mass-produce art from a numeric overlap score.
