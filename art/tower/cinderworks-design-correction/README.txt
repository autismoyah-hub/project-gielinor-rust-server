CINDERWORKS DESIGN CORRECTION / REVIEW CHECKPOINT

Open index.html. This is one proposed connected floor plan and THREE developed
representative scenes. The previous native prototype stays unchanged. These are
not an approved final level, a full-floor implementation or a playability claim.

All new published imagery is original-only: existing authored machinery/arches,
new scene construction and owned native terrain colours. No protected cache-world
pixels, textures, archives, NPC models, credentials, local accounts or executables
are published. Runtime/cache diagnostics remain private; only safe metadata is here.

tools/tower/design-correction/plan.py builds the source-grounded master plan and
explicit relocation contracts. It must precede detailed scene construction.
studies.py runs in Blender5.2.2LTS and assembles original architecture plus the
preserved art/tower/cinderworks-environment library. It refuses overwriting an
existing study. The editable study.blend contains shell and reused object meshes;
native terrain is authored separately in study.scene/ground.json. The shell's
conversion.json records export component order. reexport_study.py and verify.py
prove exact saved-source native bytes and independent geometry/material decoding.
The first construction revision is preserved privately for before/after review.

Reproduction in the sibling worktree setup:
  python tools/tower/design-correction/plan.py
  blender -b --factory-startup --python-exit-code 1 --python tools/tower/design-correction/studies.py
  python tools/tower/design-correction/render.py
  python tools/tower/design-correction/plan_validation.py
  python tools/tower/design-correction/verify.py
The Blender path in verify.py is explicit; adjust to the local installation.
Python3.13/Pillow and the preserved native codec/mesh utilities are used.
Do not rerun studies.py over the curated studies; use a fresh isolated output tree.

render.py builds a PRIVATE package from original assets and the3review shells,
then uses the existing native cinderworks-review executable. Enhanced GPU/NVIDIA
adapter1 is the verified default on this machine. Classic comparisons use identical
geometry/camera. IDs64800..64802 are review shell objects; models1231000..1231002.
The current live environment guard excludes these IDs: this is not a live overlay.
Original underlay definitions5801..5807 use native stored IDs+1. No cache material,
texture or palette is loaded in the original-only mode. Shadows/AO/AA come from the
existing native backend; no new PBR, dynamic lighting, water simulation or NPC art.

prepare_diagnostics.py derives an isolated metadata-only collision executable and
opt-in timing instrumentation in the separate experimental client worktree.
Build with the preserved offline CARGO_HOME mirror and --features enhanced-renderer:
  cargo build --offline --release --bin cinderworks-collision-review --bin cinderworks-profile-live
Use the existing build/enhanced target directory. Source core behavior is unchanged;
the only viewport additions are timings and a bounded optional log, enabled by
TOWER_CORRECTION_PROFILE. The normal allocator and ordinary native actions remain.
The metadata collision adapter inspects final prepared.scene against an authorized
local cache; it writes safe flags/definition metadata, not cache-world screenshots.
Its manual chunk path omits terrain FLOOR installation: restrict the classification
to the14reported sites, as documented. The128water omissions in that offline adapter
are not new live defects. diagnostics.py checks13lifecycle sites and1bank policy site.

profile_live.py uses only the recorded OWNED isolated server/client. It normally
drains the old server and restarts the SAME preserved prototype map/database with
the timing-only client; it never loads a redesign scene. No account progress is
seeded/rewritten. It waits for actual ready state, then a post-ready frame window,
and compares saved position, inventory, XP and journal. Loopback launch on this host
requires an approved local context. Do not launch against production accounts.
Logs, credentials, accounts, raw BMPs and runtime/cache evidence stay in the sibling
cinderworks-design-correction-private directory, outside Git.

review.py curates this HTML review, editable diagram and source/integration handoff.
Raw native BMPs and duplicate sidecars are retained privately; publish PNGs and
source-safe metadata only. No whole-floor construction follows this checkpoint
until the user has reviewed the architectural direction.
