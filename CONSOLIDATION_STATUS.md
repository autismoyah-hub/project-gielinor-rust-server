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

## Legacy bounty conservation

The audited legacy bounty fix stages the whole reward before publishing it.
A failed beer insertion cannot leave a partial coin grant and an unpaid bounty
that can be retried. All2048 original Kotlin cases were compared:2000 retain
original results;48 partial-grant cases now deliberately roll back, with three
retry checks each. The original fixture is unchanged. Seventeen actual location
handler tests and650 world library tests pass, with one preexisting ignored test.
This is an explicit reference-behavior correction, separate from the already
atomic ACK-gated Cinderworks claim path. Fresh native earned claims remain pending.
Evidence: `docs/consolidation/legacy-claim-validation.json`.

## Fresh native onboarding and first-plaque failure

A fresh ordinary account completed the actual visible creator and experience
selection. Its plaque action walked to10418,106 and durably saved run1 and six
recovery tools. The server then exited because the combined introduction/tool/
guide chat exceeded its byte-length packet limit; the client had not received
the tools. This failed attempt,42 client files and the database are preserved.
Raster captures are real; desktop presentation reported an invalid handle, so
this is not a native display pass. Initial Herblore3/Runecrafting2/5x policy is
preexisting login behavior and is not counted as earned activity.

The fixed producer keeps the full guide and uses a shorter introduction. Tests
exercise its real first/max-replay producers and ACK barrier. Queued smithing
now applies the character multiplier through its existing member-aware owner,
and Warden defence uses the standard shared animation catalog for its admitted
body. All652 world library tests pass (one preexisting ignored), as do12500
original smithing cases,28 shared combat tests and three TCP lifecycle tests.
The exact archived candidate passed real-cache authored configuration checking.
Same-save native tool-delivery/retry recovery and the earned chain continue.
Evidence: `docs/consolidation/shared-content-validation.json`.

## Scoped banking and first-checkpoint recovery

The authored bank booth now uses operation2 at10417,114 through the existing
full-bank request, reach checks and UI owner. The same cache object elsewhere
cannot open this scoped bank. Forty-five focused banking tests pass, including
normal routing, exact-site rejection and character-state conservation.

The first-plaque crash also exposed an authentication assumption: an existing
account without a previous logout was rejected even after its first durable
checkpoint. Authentication now recognizes the saved initialization marker after
ordinary password/OTP verification. A never-saved registration remains rejected.
A real SQLite/Argon2 worker-reopen diagnostic reproduced response13 before the
fix and now passes; wrong passwords remain rejected. No logout time was invented.

All654 world and289 server library tests pass, plus three encrypted TCP lifecycle
tests covering account reservation. The exact archived candidate is running
against the unchanged earned account for native recovery. The original saved
character, stats, inventory and run match the preserved first checkpoint. Both
failed client recovery attempts remain retained. Received-tool/retry acceptance
and the full services/encounters/reward/replay journey are still in progress.
Evidence: `docs/consolidation/scoped-bank-recovery-validation.json`.

## Native tool recovery and authored fishing shore

Native first-checkpoint recovery now passes: the client received the six saved
tools once each, completed a real plaque retry without duplicates and logged out
normally. The database preserved inventory, stats, chemistry and run1 with no
services or victories. Seventy-one client files and the matching save are retained.
Inventory packet reception is verified; visible inventory icons remain a client
layout investigation, and GDI display limitations remain explicit.

Two ordinary native fishing actions reached dry shoreline but failed the generic
NPC touch check. The audited three-tile shore approach is now scoped to configured
Cinderworks fishing NPCs, using the existing gathering queue without changing
water collision. A real ordinary NPC-action regression failed before the fix and
passes afterward. All656 world tests,39 shared NPC-action tests and three TCP
lifecycle tests pass; one existing world test remains ignored. The exact candidate
is in another same-save native run. Fish/service/encounter/reward acceptance is
still pending. The acceptance contract now lists exact ore and supply targets.
Evidence: `docs/consolidation/shore-fishing-validation.json`.

## Dedicated fishing operation binding

The geometry-only candidate still failed the native Net retry. Review found
that the live authored spot had no handler: ordinary tutorial fishing uses9478,
whereas the authored spot is3317. The previous unit fixture supplied a handler
manually and therefore did not reproduce real startup. That gap is now explicit.

Actual Runtime and the shore fixture now use one pre-admission content factory
that registers a dedicated net operation. It retains the normal tutorial owner
and rejects conflicting/live reconfiguration. Eight service tests pass, including
ordinary shore actions and real queued smelt/forge conservation with output-only
workshop credit. All658 world and289 server library tests, three TCP lifecycle
tests and real-cache application startup pass. Native fishing is still pending.

Separately, the corrected native765x503 client visibly rendered all six tools
and exercised the actual pickaxe Wield menu. Inventory removed that pickaxe,
equipment gained it once, and normal logout saved the change without XP or run
progress. Sixty-five files and the database are preserved. GDI/audio limitations
remain separate. Evidence: `docs/consolidation/fishing-binding-validation.json`.

## Authored range binding and earned Workshop

The native session caught23 shrimp, deposited them, withdrew five, mined copper
and tin, smelted a bronze bar, then forged and wielded a bronze dagger through
visible menus. Workshop was credited after output. The authoritative disconnect
save retains18 raw shrimp in bank, five in pack, all tools and run[1,1,0,0].
Ninety-four client evidence files and a read-only SQLite snapshot are preserved.

The range attempt returned Nothing interesting happens. Its authored9736 had no
ordinary cooking surface; the test fixture had masked this using its fire ID.
The explicit content factory now binds the canonical range recipes and existing
modifiers to that exact authored site. Ten focused tests and all289 server/660
world library tests pass (one existing ignored test); exact real-cache authored
startup passes. Native cooking remains pending on the new binary.

A later real chicken Attack ended the client with PendingAsset archive8/group2176.
The server logged damage, defeat and private drops, then saved on disconnect;
rendered combat and pickup are unaccepted. ORACLE owns the asset repair. No
Imp/Warden/reward acceptance is inferred. The owned old server stopped normally.
See `docs/consolidation/cooking-binding-validation.json`.

## Shared preparation order acceptance

All six Forge/Kitchen/Altar orders now pass through real queued smithing,
cooking and prayer outcomes in a deterministic fixture with explicitly synthetic
starting materials. Requests alone do not advance services. The fixture walks
between the configured altar and range, consumes ore/fish/bones, retains tools
and verifies repeated empty-slot offering cannot duplicate XP or inventory.
All11 service tests pass. An initial forbidden altar relocation in test setup
failed and is retained; the corrected test uses the actual spatial mapping.
Production code and the running native candidate are unchanged. This is headless
acceptance; native Kitchen/Altar and the earned encounter chain remain pending.
Evidence: `docs/consolidation/service-order-validation.json`.

## Partial standalone authoring-tool adoption

The original strict static GLB converter and classic diagnostic encoder now live
under `tools/tower/custom-assets/converter` with their original tests
and source hashes (CRLF text normalized to LF where present). All11 synthetic boundary tests pass at the destination. No
cache, private geometry, Blender process or runtime dependency was introduced.
The README documents the supported subset and limits. Native decoder/rendered
acceptance is not inferred from encoder tests. Blender/extended animation tools
remain recoverable for separate dependency review; historical packagers retain
stale ID assumptions and were not made current namespace authorities. T-CORE is
partially adopted. See `docs/consolidation/static-tools-validation.json`.

## Explicit shared-owner reconciliation

Twenty-one source hunks now have recorded ownership decisions against the exact
experimental handoff. The maintained files match committed main. These decisions
retain newer OpenRune precision, member/slot/consumable policies and location
menus while selecting scoped Tower character, presentation, admission, service
and onboarding adapters. Legacy tutorial cooking credit is not imported: Kitchen
uses actual shared cooking output. Existing chemistry state remains alongside
Tower saves; this review does not resume the paused independent chemistry work.
Other S090 paths remain under review. No discovery or path count is a gameplay
completion percentage. See `docs/consolidation/shared-owner-review.json`.

## Earned native preparation observed

The actual character completed Forge, Altar, then Kitchen on the pinned current
floor. Workshop and equipment were already verified in the prior durable save.
The repaired native client survived real chicken combat, displayed the private
drops and took bones through a visible menu. Using those bones at the altar
consumed them and completed Altar. The first range attempt burned shrimp into
7954 without XP or Kitchen credit; the next made315 and awarded150 Cooking XP,
completing Kitchen. Synthetic tests use a declared323 burn output; that is not
the cache recipe's native output. Immutable native snapshots/captures are hashed.

The session remains active: all-service durability and the guardian/reward chain
are pending. Composed frames show interaction/UI but flat gray ground and limited
visible shell; that client terrain/cutaway scope is under review. Desktop GDI and
audio remain separate limitations. Whole-floor visual acceptance is not inferred
from inventory/journal results. No prepared grants or forced outcomes were used.
See `docs/consolidation/native-services-validation.json`.

## Durable preparation and bounded JS5 read pipeline

The006 native session logged out normally. SaveACK608 at tick3820 persisted
run[1,7,0,0], all six tools, worn dagger and twelve cooked shrimp. All23 earned
raw shrimp are conserved as12 cooked and11 burnt. The635 client evidence files
and a read-only SQLite snapshot are preserved. The Imp attempt stopped outside
its court and was correctly refused; the actual narrow south entrance route has
been identified from production collision and still needs native traversal.

A separate JS5 candidate now queues up to four eligible disk reads behind a
cold response head without changing wire order or existing budgets. Two delayed
provider tests,291 server library tests, real socket/worker checks,2048 original
RSProt queue cases,three TCP lifecycle checks and exact authored startup pass.
The old native server remained unchanged during play. Native timing on the new
candidate is pending; a clean saved-session swap is coordinated with ORACLE.
See `docs/consolidation/js5-pipeline-validation.json`.

## Exact ordinary court walking and earned Imp

The read-only authored-floor diagnostic now executes World walking with all20
NPC footprints after its static checks. The direct10434,128 to10435,132 request
routes around the barrier in five ticks; each entrance waypoint takes one tick.
All eight functional anchors,1323 escape cases and2565 reachable tiles still pass.
A synthetic movement actor gains no possessions, XP or service progress.

Separately, the actual native character followed each entrance waypoint and
defeated the18HP Ash Imp. Its journal unlocked the Warden, HP remained10, and
four normal food commits reduced cooked shrimp from12 to8. Completion SaveACK607
at tick676 and a read-only SQLite snapshot confirm run[1,7,1,0]. The first partial
evidence copy expected an optional inspect276 before it appeared; that attempt is
retained, and immutable274 supplies the completed proof. Warden is still pending.

The same native client reached the world in63.847s on the pipelined server versus
153.405s on the prior server,10723 startup groups in each reported run. This is
a single observed pair, not a general benchmark. No active gameplay hotpatch or
floor/boundary change was made. See `docs/consolidation/authored-walk-validation.json`.

## Earned first clear and reward verified

The native character defeated the42HP Warden through ordinary attacks, movement,
food and two successful coolant interactions. Phase-two and final-phase messages
were observed. Its victory checkpoint persisted before the chest was used.
A full28-slot pack, filled through seven additional normal catches, rejected the
reward without payment. Banking nine burnt shrimp made space. The next search
granted500 coins and one beer; a repeated search changed no inventory or XP.

Safe-location stop and SaveACK609 at tick4950 preserve run[1,7,11,1], all tools,
the worn dagger and the single reward. All30 caught shrimp are accounted for:
7 raw,6 cooked,11 burnt and6 eaten through normal food transactions. The1082
closed client files and read-only final SQLite snapshot are preserved. The old
server exited normally; an unchanged-save restart is ready for native follow-up.

A test-only lifecycle comparison also passes: quiet disconnect saves its position
after17 world ticks; a live warning can continue through combat-prevented grace,
cause normal rescue and save sanctuary after28 ticks without refunding items or
resetting progression. Different native/server clock domains are not subtracted.
This reproduces a mechanism consistent with the earlier sanctuary return, rather
than claiming an unconditional relogin teleport or an uncaptured rescue trace.
No production combat/logout rule changed. Warden/reward state acceptance does
not imply complete visual hazard/shading/shell/GDI/audio parity.
See `docs/consolidation/first-clear-validation.json`.
