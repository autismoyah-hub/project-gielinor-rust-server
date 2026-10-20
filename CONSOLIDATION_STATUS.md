# Consolidation checkpoints — 2026-10-19 to 2026-10-20

The maintained OpenRune server is now the mainline. Promotion used a normal
fast-forward to `7421b89d5e458a7022ff7c5afc51ea79b02406fe`; GitHub main and
remote HEAD were independently checked. The former `agent2/openrune-content`
branch remains a recovery checkpoint at that commit.

## Recovery and source ownership

The authoritative pre-consolidation source was `82888b93d619cd03facf1bf022d79078ab97f1e3`.
Tower adaptations are reviewed against the enclosed-floor server handoff
`f9180b23ae1e158a296489a3730a8ae697a71de2` and its audited catalogs.
The older experimental shared engine is not copied over the newer OpenRune engine.
S-001 foundations and ordinary handlers are retained. The existing S-016 worker
remains the single owner, with the additive checkpoint request documented below.

Both histories were bundled and verified. The 26 pre-existing dirty paths,
private fixtures, ignored resources and deleted-in-worktree documentation were
preserved separately. Those unrelated pending changes are excluded from this
integration commit. No experimental branch or Agent 5 worktree was removed.
The same-host shared resource path is an access route, not an offsite backup.

The complete restricted backup manifest contains 17,254 entries; its SHA-256 is
`a190a578df0230c558457f3853f864c4840344e8651e877f80f1acc8bb76d323`.
A restore to a new directory recovered 1,560 current-floor dependencies from
1,099 distinct payloads and reverified floor/master/package/metadata identities.
`tools/restore_audited_resources.py` performs the restore and refuses an existing
output directory. Private manifests, asset payloads, caches and accounts are not
published with this code. A GitHub checkout alone does not contain the floor art.

## Batch A — storage and package contracts

Chemistry V12 remains byte-for-byte unchanged. Tower state is additive V13.
Accounts retain chemistry, legacy arrival/diary, stats, varps, energy and
inventories alongside the new run state. All account fields use the existing
transaction and save worker. Run/service/claim transitions have independent
validation; no gameplay reward path is enabled by this batch.

An experimental database with Tower at V12 is rejected before migration and
without changing its version or data. It must never be relabeled in place.
The safe import policy is: retain an immutable original snapshot; identify its
exact schema/source; create a fresh maintained database; explicitly map account,
character, inventory and Tower state through reviewed fields; validate foreign
keys and run invariants; then compare a disposable native login/save/reopen.
Chemistry absent from that source receives its maintained default. No automatic
experimental importer or production-database migration is supplied here.

The package loader is explicit and immutable. It bounds input sizes, rejects
path traversal, duplicate/unowned IDs and stock collisions, checks per-file CRCs
and the agreed full-content SHA-256, and publishes definitions only after all
checks succeed. It does not rewrite stock JS5, auto-load environment paths,
negotiate a new protocol, or claim publisher authentication. Client geometry
decoding remains client-owned. Operator admission is out of band.

The paired package reserves exactly 95 models and 70 objects with no remapping.
The runtime identity is
`630bbb1bd738d4a105493f510b7273c51a6cfaca789bb7ae2d95c40d2bdb32d0`.
The original identity is
`8efa117834c574946e00405d4349cdb650e880d93e80659f18a8020d9666f7cb`.
Exact IDs, source hashes and the independent framing formula are recorded in
`docs/consolidation/floor-package.json` and the paired ORACLE contract.
Vendored hash dependencies retain their original licenses and file provenance.

Read-only package diagnostic (with the operator's private game-cache directory):

```
cargo run --offline -j 2 -p gielinor-server --example cinderworks_package_check -- GAME_CACHE RUNTIME_PACKAGE
```

## Verification

Before promotion, the complete Rust suite passed 1,357 tests, with zero failures
and two pre-existing ignored tests. Private harness repairs were scoped against
the unchanged reference; the only public production repair was malformed
Survival Tutor punctuation. Native baseline bank actions conserved all ten coins;
same-save relogin preserved inventory/bank/stats; walking and a real NPC Talk
interface succeeded. These were headless native-client actions on the baseline
floor, not graphical acceptance or a current Cinderworks earned victory.

Batch A's rebuilt library suite passed 960 tests, zero failures, one pre-existing
ignored test. It covers fresh/V12 upgrades, chemistry preservation, persisted run
state, late SQL failure rollback, retry/duplicate claim markers, immutable bytes,
namespace/path rejection and a CRC collision rejected by full content identity.
An earlier run reused a stale world-test artifact because a preparation copy
retained old timestamps; that run is retained but not accepted as current-world
validation. Fresh timestamps and a rebuilt world artifact corrected the check.

The actual restored runtime package admitted 95 models and 70 objects against
62,194 stock definitions, producing 62,264 definitions. Stock cache SHA-256
remained `06f14e1e68a294b057b3b7c2b534422e746017f7a650eb838e6276d492c81425`.
The complete Batch A regression suite passed **1,365 tests, zero failures and two
pre-existing ignored tests**. Native V12-to-V13 login/save and subsequent server
restart/relogin passed against a new disposable copy; the original V12 bank save
remained unchanged. Every checked permanent varp and inventory/bank/stat/chemistry/
arrival/diary field was preserved. Session date/clock/playtime/special-energy
updates were identified through the preserved symbols and existing timer code.
The Tower run reloaded as its expected default. The native binary SHA-256 is
`be54fb386befa3dae8d60dec0ea04facaa7f59d2e753260c9b37315ca3a0635a`.

Detailed local receipts are retained under `work/consolidation-*` and in the
restricted consolidation recovery directory. The sanitized receipt is
`docs/consolidation/batch-a-validation.json`.

## Next integration boundaries

Batch B must publish the exact 128×128 authored floor at `10368,64,0`, preserving
functional anchors, spatial overrides, collision and constructed-scene behavior.
Durable checkpoints must not expose rewards before a matching save acknowledgment.
Subsequent batches adapt services to existing OpenRune engines, then encounters,
rewards, recovery and observer-specific restoration.

ORACLE owns client rendering and presentation. Its coordinated package identity
must match before paired-floor acceptance. A distinct maintained Floor One launch
receipt is still required. The complete earned preparation → Imp → Warden → claim
journey on the current enclosed floor has not yet been accepted. Historical
performance/art limitations remain open; older 61/73 ms GPU results miss 33 ms.
S-015 regular-tree policy remains gated by GR-10; no timer is invented here.
The user-paused chemistry-specific independent/native campaign remains paused;
generic storage preservation tests do not resume it. No branch retirement is
approved by this checkpoint.

## Batch B1 — exact authored scene

The explicit content loader now pins floor, spatial overrides, visual bindings
and cutaway metadata alongside the runtime package. No process-global environment
switch selects geometry. The default legacy template remains unchanged.

The current floor prepared and published all 16,384 cells, 74 authored objects
and 103 final locations. Production collision/reach found 2,565 reachable tiles
and verified exact bank/plaque/chest/altar/portal/furnace/anvil/range anchors.
The spawn is 10420,110,0. These are server preparation/navigation checks, not
claims that a player performed each interaction. No rewards were generated.

Validation: 966 library tests passed, zero failed, one pre-existing ignored;
both original Kotlin layout/progression comparisons passed. The newly archived
server also passed the maintained native login/walk/NPC Talk baseline. Stock
cache bytes retained their earlier SHA-256. The sanitized receipt is
`docs/consolidation/floor-scene-validation.json`.

Read-only diagnostic with operator-installed private inputs:

```
cargo run --offline -j 2 -p gielinor-server --example cinderworks_floor_check -- GAME_CACHE SYMBOLS ART_TOWER
```

ART_TOWER contains interior-production and floor-handoff/runtime-package.
Application opt-in serving, entry/recovery, save acknowledgments, content hooks
and earned current-floor acceptance follow as separate verified increments.

## Batch B2 — checkpoint persistence kernel

Proposed inventory and Tower run snapshots stay behind a player barrier until
the matching account-worker acknowledgment. Pending snapshots block further
actions, queues and logout snapshots; interrupted shutdown waits and then saves
the acknowledged state. Internal save tokens are unique across requests. Public
save-completion generation remains the world actor generation, preserving the
existing native observer contract. Old/duplicate completed tokens are ignored.

The original bounded account worker remains the only writer. An additive
Checkpoint request shares the existing account transaction while preserving
last_logout. Ordinary Save continues to stamp logout metadata. This corrects
an experimental snapshot behavior that would otherwise fabricate a logout.
Chemistry V12 and the additive V13 schema are unchanged.

Validation: 969 library tests passed, zero failed, one pre-existing ignored;
three focused final-source tests and all three encrypted TCP lifecycle tests
passed. The actual worker acknowledged two checkpoints and one ordinary save,
preserved checkpoint logout metadata, conserved repeated-snapshot inventory,
and reopened the saved state. A newly archived server passed the native
login/walk/Talk/logout baseline with the original outward acknowledgment fields.
The receipt is `docs/consolidation/checkpoint-validation.json`.

Operator-only diagnostic, refusing an existing output directory:

```
cargo run --offline -j 2 -p gielinor-server --example cinderworks_checkpoint_check -- NEW_OUTPUT_DIRECTORY ARGON2_DLL
```

The diagnostic uses synthetic disposable state and does not demonstrate earned
content. Entry, tool and claim producers, services, encounters and paired current-
floor acceptance remain separate integration work. The library checkpoint
producer is intentionally not yet exposed through a gameplay handler.

## Gameplay kernel — authored entry, services and encounters

Opt-in entry preserves earned possessions and permanent progress. Plaque tool
collections, new runs, guardian unlocks and chest rewards stage full snapshots;
items and run flags become visible only after the matching save acknowledgment.
Full packs and repeated pending claims leave live inventory unchanged.

The maintained gathering, cooking, smithing and offering owners perform resource
changes and XP. New hooks credit successful preparation only. Burned shrimp does
not restore the kitchen. The authored Imp/Warden bounds, telegraphs, coolant
interrupts and rescue scheduler are integrated behind explicit configuration.

Standard revision-239 MapAnim records carry owner-filtered transient hazards.
The run journal and eight restoration landmarks use committed player state and
explicit spatial overrides; scene resets reconstruct each observer independently.

Validation: the pre-journal workspace passed 991 tests, zero failures, one
pre-existing ignored test. The journal/scenery server run passed 285 tests and
exposed one old-layout visibility fixture assumption; the corrected final six
scene tests passed, including all eight exact authored landmark coordinates.
All three original-RSProt zone comparisons passed (1,024 MapAnim cases). The
archived final server passed the maintained native walk/Talk/logout baseline.
Detailed evidence and the preserved failure are recorded in
`docs/consolidation/gameplay-kernel-validation.json`.

Application package mounting, authored population/display/Attack settings and
quiet guardian visibility are next. The default launch remains unchanged.
These tests do not constitute earned current-floor native acceptance; the full
service/Imp/Warden/reward/replay/restart/observer scenario remains pending.
Chemistry verification stays paused and the GR-10 tree-policy gap stays explicit.

## Authored launch and paired native entry

`--cinderworks-content ART_TOWER` now mounts the exact admitted package and
floor before any account database or listener opens. The default launch retains
its existing layout. The authored population contains 20 actors at the pinned
positions. Warden presentation uses standard NPC name/combat/transform blocks;
Attack initialization transmits an absent default while preserving saved Hidden.
Restoration pairs must have identical clipping and no gameplay operations.

The full maintained workspace passed 1,407 tests, zero failures and two previously
ignored tests. Original RSProt NPC presentation comparisons cover 1,024 cases;
existing combat and spot-animation composites also pass. Real-cache collision
checks cover eight anchors and 1,323 hazard escape cases with NPC footprints.
The adapted audit example additionally exports scene, objects, flags, walk edges,
combat steps and NPC footprints to a new private directory; overwrites are refused.

Paired native entry/viewing/logout passed with 428 frames, followed by a final-
source same-save restart with 469 frames. Both preserved 10420,110,0, inventory,
stats, XP rate and the unstarted run; neither sent action input or granted tools
or progress. Original screenshots and immutable client/server identities are
retained. These runs verify entry and persistence, not the earned content chain.

Image review found a restart HUD discrepancy (HP/prayer displayed as zero).
A separate pinned native packet capture received all 25 cache-defined stats,
including HP10/XP1154, prayer1 and energy10000, with no later zero updates.
Client application/readiness and packet-order investigation remains open; HUD
parity is not asserted. Two shorter startup timeouts are preserved; independent
startup recovery completed 10,695 requests in about 231 seconds without failures.

Receipts: `docs/consolidation/native-launch-validation.json` and
`docs/consolidation/added-source-disposition.csv`. The concrete remaining journey
is specified in `docs/consolidation/earned-acceptance-contract.json`: genuine
services, Imp, Warden, atomic claim/full-pack/replay, restart and observer separation.
C018 native action tooling is being coordinated with ORACLE. No branch retirement
or full content acceptance follows from this launch checkpoint.

## Creator route and native readiness follow-up

The restart HUD question above was resolved by a strict-ready native observation.
HP10, prayer1, energy100 and total XP1411 appeared after initial queued loading
completed; prior zero HUD frames predated readiness. The saved inventory, stats,
chemistry and unstarted run remained unchanged. Startup latency remains recorded.

A selective original-source adaptation preserves the mandatory character creator
against movement before Confirm. After real appearance and experience selection,
explicit in-floor Cinderworks accounts can proceed to the plaque; ordinary and
out-of-floor accounts keep the existing guide route. Three TCP lifecycle tests,
including nine route/experience cases, and all289 server library tests pass.
The route matrix preserves inventory, equipment, bank, stats, run and position.
The exact candidate is archived. Fresh-account native ordinary-action acceptance
is underway without seeded progress or items; the earned chain remains pending.

Evidence: `docs/consolidation/onboarding-readiness-validation.json`. Earlier
failures and private captures remain retained; no branch retirement is approved.
