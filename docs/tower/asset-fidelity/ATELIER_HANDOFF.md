# ATELIER technical handoff

Technical source: `2a6e44ebf5e3c671a889dd04d05f1ea815a15d3b`; client: `23a94f75707f0503a9efe610fe99bfce5089b575`. The
workbench is usable for controlled authoring/diagnosis. It is not certified for
production art, hundreds of consistent assets or live Tower integration. Read
CURRENT_STATUS.md and the failed generalization results before using the outputs.

## Setup

Use sibling server/client checkouts and the existing offline-render Cargo manifest.
Build `cargo build --release --bin asset_workbench` in
`tools/tower/offline/render`. Native dependencies are the existing vendored client
library. Python needs Pillow. Blender 5.2.2 LTS, hash d13f752e3b9c, is tested; its
slotted-action API matters. Supply machine paths explicitly. Keep PRIVATE_OUTPUT
outside the source repository and immutable cache. No service/account is needed.

## Reference and B preservation

```text
python tools/tower/asset-workbench/bench.py --native EXE --cache CACHE --out PRIVATE_OUTPUT inspect
python tools/tower/asset-workbench/bench.py --native EXE --cache CACHE --out PRIVATE_OUTPUT --blender BLENDER roundtrip
python tools/tower/asset-workbench/verify_motion.py PRIVATE_OUTPUT/training
python tools/tower/asset-workbench/motion.py PRIVATE_OUTPUT/training/reference PRIVATE_TRACE.json
```

Selectors are object:ID[:SHAPE], npc:ID, item:ID or item:ID:worn. Pass
`--catalog tools/tower/asset-workbench/equipment_mode_catalog.json` for the two
worn supplements. Pass `--split held_out` for the declared held-out cohort. Do not
retune on it and then call it newly held out. Native `animate SELECTOR SEQUENCE`
extracts a specific supported action; `inspect-static` omits actions.

## Independent C authoring and local edits

```text
python tools/tower/asset-workbench/analyze.py library PRIVATE_REFERENCES PRIVATE_ANALYSIS
python tools/tower/asset-workbench/instantiate.py pillar PRIVATE_ANALYSIS/object-ID.json PRIVATE_RECIPE.json
python tools/tower/asset-workbench/author_workflow.py --recipe PRIVATE_RECIPE.json --out FRESH_OUTPUT --native EXE --cache CACHE --blender BLENDER --reference PRIVATE_REFERENCE_ASSET --bounds PRIVATE_REFERENCES/original-id-bounds.json --identities PRIVATE_IDENTITIES.json --asset-key STUDY_KEY
```

The output must be fresh; previous runs are never silently reused. The workflow
builds, saves, reopens, exports actual Blender data, validates native model/pose
coverage and creates a separate custom object package. `--blend SAVED_BLEND
--patch PATCH.json` performs explicit owned-component edits; `iterate.py` adds
bounded proposals, five-view retention and preserved history. See `--help` for
its required paths. `--replace-actions` explicitly regenerates actions from a
new recipe and keeps previous owned Blender actions; ordinary reopen preserves
saved artist channels.

`diagnose.py` requires explicit A/C semantic bindings. `motion_review.py` accepts
a manifest with reference geometry/action folders, named candidate roots and
curated bindings; it produces matched-timestamp trajectories, GIFs and contact
sheets. Bindings and direct reconstruction recipes stay private. Inspect actual
native images whenever a numeric improvement is retained.

## Original authoring, registration and native scenes

```text
python tools/tower/asset-workbench/original_designs.py PRIVATE_ORIGINAL_RECIPES
python tools/tower/asset-workbench/author_workflow.py --recipe ORIGINAL_RECIPE.json --out FRESH_OUTPUT --native EXE --cache CACHE --blender BLENDER --bounds ORIGINAL_ID_BOUNDS.json --identities PRIVATE_IDENTITIES.json --asset-key ORIGINAL_KEY
EXE CACHE PRIVATE_SCENE beside object:REFERENCE_ID FRESH_OUTPUT/asset.model
```

Omitting `--reference` makes standalone owned native views; it does not compare
against an automatically fitted A camera. `render MODEL CAMERA AMBIENT CONTRAST`,
`render-own MODEL AMBIENT CONTRAST`, `package ...` and `play MODEL CAMERA AMBIENT
CONTRAST SEQUENCE_DIRECTORY` are additional native modes. `native_codec.classic`
rebuilds sequence/frame/skeleton payloads from published original actions/*.json.
Object packages use the existing native package reader; model/NPC/sequence/skeleton/
frame identities are isolated and allocated above original maxima. The original
probes reserve models 1101514–1101516 and objects 63414–63416. Preserve the shared
identity manifest, use a single writer, and use a new key to turn a static design
into an actor. These diagnostics do not provide a live NPC overlay.

## Review, tests and publication

`build_review.py MANIFEST.json PRIVATE_REVIEW` builds the local evidence viewer
from explicit native-image paths; no rendering or public upload occurs. The local
review manifest records source commits, action timestamps and A/B/C identities.
`evaluate.py` requires prewritten protocol/tool hashes before frozen evaluation.
The archived v1 Python fixture preserves exact tested inputs, but the old native
binary and protected fixtures remain local; byte-identical cross-machine rebuilding
of that old executable is not claimed. Current authoring source is pinned above.

Set WORKBENCH_TEST_TEMP to an explicit task-owned temporary directory, then run
`python tools/tower/asset-workbench/test_workbench.py`. Fourteen tests cover codec
rejection, sockets, hierarchy/pivots, quantized IK, PCA, submitted surfaces and
stable allocation including 200 static assets. Cleanup checks the resolved target
stays under that directory. Native A/B tests and authored pose checks are separate.
Always use Blender `--python-exit-code 1` before `--python`; a zero default exit
can mask script failure.

Keep cache models, textures, frames, screenshots, .blend derivatives and direct
reconstruction recipes out of Git. Publish only general original tools, safe
metadata and truly new owned Tower assets. Shared scenes containing genuine
assets stay private. Do not change parity/migration branches, overwrite source
IDs, integrate into gameplay or equate B preservation with independent C quality.

## Remaining production work

Semantic anatomy/facial planes, coat/material-region authoring, curved weapons,
full helmets, winged creatures and independent combat choreography remain weak
or absent. Weighted cached animation authoring, complex texture reconstruction,
audio/event parity, arbitrary blend fidelity, live NPC registration and floor
production are not certified. Evaluate a new sealed cohort after strengthening
these tools. The user retains final artistic acceptance.
