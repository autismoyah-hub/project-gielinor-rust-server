# Evidence, isolation and reproduction

## Preserved baseline

Local source started at `070b8e9f1144a4eb4e82fbac788d257dfe299c14`, the concept-review commit. Functional production code remains at `1153b4e7281435e83f60bff5131b4a3f0a8e0023`. The prior implementation receipt's five production source hashes still match. There are no production crate, migration, gameplay-rule, client-source or account changes in this study. No gameplay process was started. The full A/B/C review directory remains unchanged, preserving the unapproved concept studies and their evidence.

## Rendering evidence

The final preferred treatment has 15 views: 12 close frames, two oblique overviews and a plan. Plain and city treatments each retain five matched frames. All 25 final treatment frames are visually reviewed, alongside 25 asset/material boards, four wheel camera studies and three paired lighting comparisons. Raw BMPs, compressed lossless PNGs, camera sidecars, stdout logs and hashes stay under `work/environment-quality-study/`.

[Earlier lighting and camera comparison scene sources](comparison-sources/index.json) retain the exact authored geometry, mode, poses and raw-frame hashes for these additional comparisons. The paired lighting scene inputs are byte-identical.

The original renderer default remains `runtime`; an earlier Candidate A sanctuary BMP remains byte-identical with the updated executable. Optional `authored` assembly sets initial object identities and invokes existing native static lighting and terrain shadow code. No client file or substitute renderer was introduced. The object mode is recorded in every new frame's sidecar and manifest. Camera settings are composition presets, not live viewport-controller equivalence.

## Access audit

The existing real-cache audit was run on the final preferred scene and reports **960 loaded / 802 reached dry / 64 blocked water / 0 issues**. Its metadata records entry, recovery reservation, required services, tour points, all 12 close-view stances and four fishing reserves. The scene contains no arena metadata, so **hazard_escapes=0**. No new combat or progression acceptance is claimed. The earlier production encounters and evidence are preserved rather than rerun or modified for this visual assignment.

Initial authoring and audits failed on a contradictory wall edge, an occupied hearth waypoint and an accidentally retained water chunk at the new northern crossing. Those failed logs/scenes are retained. Corrections changed detached geometry/markers only. A future implementation must still validate real actors, rescue alternatives, action binding, source-object masking, restoration states, native camera obstruction, reconstruction and persistence.

## Reproduce from the Tower workspace

Use the existing protected local caches, sibling native-client checkout at `ff9cef17fd1ce3ec2ebf8d1181fee31411999c4c`, isolated Rust toolchain and Python/Pillow. Nothing here grants access or distribution rights to the cache dependencies.

```powershell
cargo build --release --manifest-path rust-server/tools/tower/offline/render/Cargo.toml
cargo build --manifest-path rust-server/tools/tower/offline/export/Cargo.toml --bin design_audit
cargo build --manifest-path rust-server/tools/tower/offline/export/Cargo.toml --bin environment_assets
python rust-server/tools/tower/offline/design/environment_study.py --config rust-server/docs/tower/design-review/environment-quality/composition.json --output work/environment-reproduction --renderer work/target-client/release/tower-scene-render.exe --cache resource-staging/server/work/integration/a-kotlin/.data/cache/js5
work/target/debug/design_audit.exe . work/environment-reproduction/environment.scene > work/environment-reproduction/audit.log
python rust-server/tools/tower/offline/design/environment_diagram.py --config rust-server/docs/tower/design-review/environment-quality/composition.json --audit work/environment-reproduction/audit.log --output work/environment-reproduction/diagrams
```

The workspace Cargo wrapper may be required to select its established toolchain and target directories. Inspect each process exit code. Change `--treatment` to `austere` or `city`, optionally restricting `--views` to the matched comparison list in the manifest. `--assembly runtime` retains the older assembly for a same-geometry lighting comparison. Each published asset-board scene can be passed directly to the renderer with its indexed camera and explicit `runtime` mode. Segoe UI fonts are used only to draw original diagrams; no font files are redistributed.

The study compiler does not install configurations or read accounts. Source chunk unions and actual placed objects are reproducible from [composition.json](composition.json). Public diagram colors and furniture boxes are original symbols, not cache-rendered asset thumbnails. The exact source, scene, camera and pixel hashes are in [REVIEW_MANIFEST.json](REVIEW_MANIFEST.json).

## Known visual limits

Actors, active interfaces, moving machinery, live flames, hazards and personal restoration transitions are absent. The authored lighting mode does not reproduce the present floor's copied-source/removal/runtime history. Sloping terrain seams at the basin and empty scene margins are visible; no inaccessible elevation or underpass is presented as implemented. The standard is still subject to the user's close-view judgment.
