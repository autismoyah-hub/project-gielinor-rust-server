THE CINDERWORKS — INTEGRATED ENVIRONMENT REVIEW CANDIDATE
2026-10-18 / ASCENT experimental workspaces

Open index.html for the publishable original-only catalogue, planning and test
metadata. On the development machine, the complete PRIVATE native-world review is:
../cinderworks-environment-private/review/index.html (relative to the server root's
directory). It contains protected cache-world pixels and is NOT published.

This candidate is integrated into the actual game world. It is not artistically
approved or production ready. Read REMAINING_WORK.txt and validation/technical-report.json.
No full combat/skilling/reward-loop acceptance or smooth-play claim is made.

SOURCE AND ISOLATION
Server branch: experiment/cinderworks-environment-production
Server base: 6771dee2cdec2abca702b4504449f63ef76eadbc
Client branch: experiment/cinderworks-environment-production
Client base: 3049aa30268bc2c73da65536671dfcbb6ab429b6
The client addition is src/bin/cinderworks-review.rs, an offline native-scene review
adapter. Runtime uses the existing gielinor-client enhanced implementation. Core
renderer, cache decoders, gameplay rules and other agents' branches are unchanged.

Recovered authoritative sources: crates/world/src/tower_layout.rs, tower_run.rs,
tower_arrival.rs, tower_scenery.rs, tower_cinderworks.rs, cinderworks_encounter.rs;
crates/server/src/tower_map.rs; crates/cache/src/tower_shell.rs. The old eleven-task
Kotlin diary is historical, not the present three-service/Imp/Warden run contract.
The A/B/C concept studies and previous art/NPC research remain separate.

LOAD THE ORIGINAL OVERLAY INTO AN ISOLATED SERVER/CLIENT
Use an existing authorized local revision239.1 game/JS5 cache, symbols, RSA test
keys and Argon2 runtime. None of these protected or machine-specific inputs is in
this publication. Do not run against production accounts or port43594.

Both processes need:
  TOWER_ASSET_PACKAGE=<absolute path to this directory>/overlay
The isolated server additionally needs:
  TOWER_CINDERWORKS_ENVIRONMENT=<absolute path to this directory>/environment.tsv
The visible client additionally needs:
  TOWER_RENDERER=gpu
  TOWER_GPU_ADAPTER=<actual NVIDIA/desired hardware adapter index>
  TOWER_WINDOW_SIZE=1280x900
Choose TOWER_RENDERER=lite with the actual Intel adapter index for fallback, or
TOWER_RENDERER=classic for CPU rasterization. Indices vary by machine: use the
enhanced-benchmark probe command and verify the reported adapter. This launcher
and review default to GPU; the shared native core's global default is unchanged.
GPU/Lite require --features enhanced-renderer and the existing Windows D3D11 backend.

The environment flag is opt-in. Without it, the established gameplay map remains
the default. Do not leave the new environment flag enabled without its overlay.
Do not merge into FORGE/ORACLE branches as part of launching this review.

WORKSPACE REPRODUCTION
The following Python tools live in tools/tower/environment-production. They assume
the server/client sibling worktree arrangement used by this experiment. Private
generated output goes to ../cinderworks-environment-private, outside Git.
Python3.13 + Pillow/NumPy and Blender5.2.2LTS were used. The Blender executable was
C:/Program Files/Blender Foundation/Blender 5.2/blender.exe; edit the explicit tool
path for another installation. No network content downloads are required.

Fast launch in this workspace:
  python tools/tower/environment-production/install_review.py
  python tools/tower/environment-production/native_session.py start gpu
The launcher creates a NEW disposable account at the existing Lumbridge recovery
pad, a fresh isolated database, private password file and localhost server. Normal
Tower arrival places it in the floor. Complete ordinary appearance and experience
choices. Existing accounts and earned progression are never seeded or overwritten.
The current live review account completed those ordinary choices.
Native subprocesses on this host required permission to reach loopback. A sandbox
launch may log a listener while a sandboxed client cannot connect; use the normal
approved local launch context. Background server console stays hidden, game visible.
Stop only the session recorded in latest-native-session.json; use its ordinary
adapter 'stop' action and its server_args --stop-file. Server normal drain saves
accounts. Do not kill unowned processes. Direct reconnect admission failed during
the first mode switch; native_modes.py uses normal server drain/restart instead.

Build server from server root:
  cargo build --offline -p gielinor-server --bin gielinor-server --example cinderworks_environment_audit --example cinderworks_environment_account
Use CARGO_TARGET_DIR=../build/cinderworks-server for the launcher paths.
Build client from sibling client root:
  cargo build --offline --release --features enhanced-renderer --bin gielinor-client --bin enhanced-benchmark --bin cinderworks-review
Use CARGO_TARGET_DIR=../build/enhanced and the existing enhanced-state/cargo-home
offline dependency mirror. The ignored generated src/data/java-char-case.bin was
copied unchanged from enhanced-client for compilation. Its SHA256 is recorded in
validation/build-inputs.json; reproduce that normal generated input before building.

AUTHORED SOURCE AND CONVERSION
assets/ contains26 editable .blend sources, original geometry, conversion receipts
and native static models. Two walls also retain source-detail.blend: runtime walls
use440/204triangles instead of the earlier2456/876. IDs1220000..1220025 and64700..64725
are an experimental namespace, not a permanent global reservation. Check collisions
before any wider integration. Native models use128units/tile and quarter-turn native
placement; authored Blender axes are X lateral/Y depth/Z up.
overlay/registry.txt includes per-file CRCs for ordinary custom loading. No original
cache data is modified. No new NPC geometry, rig, animation or AI is included.

For a fresh authoring build, point TOWER_CINDERWORKS_ASSET_OUTPUT to an EMPTY private
directory. Run Blender headlessly with --python-exit-code1 and --python kit.py --,
then -- runtime-walls, -- refined-fit, -- utility-clearance, -- finishing in order.
The script refuses overwriting existing asset directories. Selected runtime versions
are recorded by package.py and the original-only asset catalogue. Saved-source export:
  blender -b assets/<name>/source.blend --python-exit-code 1 --python tools/tower/constrained/reexport.py -- <private-output>
verify_assets.py performs all26 saved-source byte comparisons, independent native
decoding and fresh normalized oriented-surface/material checks. Fresh model bytes
are not universally identical because Blender can reorder triangle tessellation;
the differences are explicitly listed in validation/asset-reproducibility.json.
No PBR normal/roughness transfer: native static vertices are integer and per-face
colors quantized into6/3/7-bit HSL. No skin labels or authored animation.

AUDIT AND REVIEW REPRODUCTION
Run cinderworks_environment_audit DATA_ROOT OUTPUT with the overlay/environment
variables above for the actual prepared scene, object inventory and movement edges.
DATA_ROOT is the authorized .data parent containing cache/game and symbols.
The audit example writes private prepared.scene, collision and object files.
navigation.py, check_milestones.py and traverse.py consume that exact server output;
the latter uses ordinary client walking and observed server positions, not teleport.
interactions.py uses ordinary existing object operations. It begins a disposable run,
supplies normal tools and tests plaque/bank/Prayer/reward denial/portal; it does not
fake services, boss victory, rewards or full-loop acceptance.
native_modes.py switches only the owned session and records real frame traces.
prepare_review_adapter.py derives the offline native review executable from the
existing native scene tool. render_floor.py records15GPUviews, matched Classic and
IntelLite comparisons. Overview requires the adapter's far30000review-only setting;
the earlier far-clipped blank outputs remain private failed evidence, not accepted
views. The final overview uses target56,54/yaw15300/distance9000/floor-iso in BOTH modes.
Native player camera limits and core far clipping remain unchanged.
finish_review.py curates safe public assets and PRIVATE world evidence separately.

VALIDATION AND LIMITS
13namedareas /48×48tiles /1940reachabletiles /25functional approach contracts.
105explicit original placement contracts plus the existing shell's original wall
palette replacement. All671expected placements are present in the native client.
55ordinary native walking segments passed; all named areas visited and sanctuary
reached again. Existing exit/return preserves inventory and journal. Account state
also survived normally drained server restarts for each rendering mode.
All55original solid footprints block in the actual native scene. All1656eight-way
movement cases in the two protected encounter regions match the source baseline.
14native/server collision flag differences remain; their cause and baseline status
are NOT proven. validation/native-scene-comparison.json records every local tile and
both flags. No global cache/client clipping rewrite was attempted.
Two environment guard unit tests passed. No original functional anchor was moved.
Optional external Kronos NPC combat metadata was absent (0IDs,1filefailure); the
floor-specific source rules were preserved, not reimplemented.

PUBLICATION SAFETY
Publish this directory and original source/tools only. Never publish the sibling
private directory, native cache-world screenshots, full cache, account databases,
passwords, RSA private keys, local dependency mirrors or built executables.
The two Git repositories have separate source commits on isolated branches.
