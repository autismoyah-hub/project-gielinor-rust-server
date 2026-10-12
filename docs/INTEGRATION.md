# Live native-client integration

The primary strategy is now a vertical slice using an isolated native Rust client
against the Kotlin reference and Rust development servers. Full parity is not
verified. Existing deterministic tests and ranged-combat drafts are preserved.

Run from `C:/Projects/OSRSRSPS/rust-server`:

```powershell
./scripts/integration-prepare.ps1
./scripts/integration-kotlin-prepare.ps1
./scripts/integration-kotlin-build.ps1
./scripts/integration-rust-login.ps1 -Run rust-login-003
```

Preparation is idempotent and does not replace pinned client binaries. It copies
four existing release binaries and contemporaneous source from Agent 1 read-only.
`work/integration/versions.json` records binary hashes and source limitations.
Use exactly those binaries for both environments. The exact release source tree
is not attested; the byte-identical executable is the comparison identity.
No command builds, edits, cleans, or executes inside Agent 1's workspace.

Environment A lives at `work/integration/a-kotlin`, with independent `.data/cache`,
symbols, RSA key, SQLite database and runtime working directory. Its application
classes come from the existing isolated reference build; copied external
dependencies exclude original application jars. The only new runtime overlay
requires an explicit non-43594 port and binds loopback. A wrapper permits shutdown
through an owned stop file or a four-minute timeout using the existing JVM hook.
The first startup reached map loading but exceeded 75 seconds. A bounded 180-second
retry is prepared. The map-loader overlay selects exactly the nine Lumbridge
squares used by Rust; loaded map decoding and gameplay logic are unchanged.
The lifecycle wrapper runs the real migrations and sets the disposable realm to
spawn `(3222,3218,0)`, 1x XP, automatic registration/display names, and normal
password validation. This matches Rust's configured test policy. Live A/B
comparison remains pending.

Environment B uses `work/integration/b-rust/cache`, symbols and the same test RSA
key. The Rust executable uses disposable `work/state/integration-b/accounts.sqlite`
and an ephemeral loopback port. It stops after a bounded tick count, drains saves,
and is never left running intentionally. Each run records its server hash, port,
arguments, logs and native-client packet capture under `work/integration/runs`.

The native client must use the **JS5** cache. The enriched server `game` cache has
different archive CRCs (currently archives 2 and 5); using it caused code 68 in
`rust-login-001`. This was harness configuration, not grounds to disable CRC checks.

`rust-login-002` reached account admission but the native client rejected the
login-success length. Rust advertised 34 instead of revision 239's 37 bytes
(34 account bytes plus the first game opcode and two-byte length). The correction
is in `crates/protocol/src/session.rs`; regression coverage includes authenticator
ISAAC consumption and first rebuild header alignment. All three protocol session
tests pass. Native `rust-login-003` passed: 730 packets, one player, 22 NPCs,
interface root 548, and position `(3222,3218,0)`. No decode/script failures were
reported; `WORLDENTITY_INFO_V7` remains an explicitly unhandled client report.
The server drained accounts on shutdown and the read-only SQLite snapshot records
25 stats and the matching location.

Native `rust-render-001` passed and its BMP was visually inspected: Lumbridge,
player/NPCs, chat, HUD and minimap are visible. The native client built 6,636
objects and 550,402 triangles with zero missing regions; 299 frames, 5,138 loaded
widgets, 96 drawn widgets, seven ground piles and zero queued zone records.
The native report retains a minimap cross-world-entity limitation.

The render exposed a missing `SERVER_TICK_END` (83): the client tick stayed zero.
Runtime now queues this delimiter after the player update batch, matching Kotlin
PlayerPostTickProcess.flushClient. Socket tests now validate delimiter placement,
reject duplicates/missing delimiters, and separately check initial arrival.
All three socket tests pass (work/live-tick-socket-tests.log). Native rust-render-002 reports server tick 31, 312 frames, and zero missing regions; the owned server drained accounts and stopped.

Builds retain the original conservative resource guard. Prebuilt, bounded,
sequential headless runs have a separate gate requiring 4 GiB free RAM and at most
30% sampled system CPU, with below-normal priority and two CPU cores. No process
outside the exact children of a run is terminated or modified.

`integration-compatibility.json` is the machine-readable compatibility matrix.
Protocol tests, mock login, actual client login, rendered world, gameplay and full
parity are separate states. Failed startup/login runs are evidence, not passes.

Before this transition the full workspace retest passed 272 tests after adding an atomic
temporary-directory suffix to parallel JS5 worker tests. The original failure was
Windows timestamp collision during cleanup, not a JS5 shutdown deadlock. Logs:
`work/npc-death-workspace-tests.log`, `work/npc-death-workspace-retest.log`.

Next: validate tick delimiters and replay rendering; run native movement; start environment A and
compare equivalent captures, then fix gameplay mismatches with regressions.


`rust-actions-001` confirmed the requested walk from (3222,3218) to (3223,3218).
Talk-to selected man3 (3108), routed to (3220,3218), then exposed the unimplemented
GenericPerson handler: no interface change, native exit 1. SQLite after graceful
shutdown confirms the final location. This is a dialogue failure, not movement failure.

Kotlin `kotlin-login-002` failed because TypeReferencesLoader rejects private
Kronos shop reference objects. The isolated overlay now changes visibility only
for 13 ObjReferences objects. `kotlin-login-003` passed that stage and the real
reference repacked its disposable cache before restarting. The bounded startup
expired during the restart. No original or Agent 1 cache was modified. Cache
identity must be rechecked before A/B payload comparisons; a new bounded retry
can reuse the now-prepared disposable cache.

GenericPerson dialogue implementation is in progress: conversation branches,
suspended world state, validated continuation tokens, UI heads/animations, and
reference font/page wrapping. The interface oracle passed 4,608 rows including
1,536 new head/animation cases. Dialogue compilation, cancellation regressions,
font differential tests, and native replay remain pending. Do not count this as
verified gameplay dialogue yet.

Dialogue validation update: all eight NPC action tests pass, including duplicate
continuations, movement/close cancellation, recycled NPC slots and close-facing
reset. Font decoding and TextAlignment match 384 actual Kotlin oracle cases,
including tags, hyphens, page limits and signed kerning. The updated Rust server
build passes (work/person-dialogue-server-build.log). Native rust-actions-002
has not started yet: its first attempt was deferred by CPU capacity.

Kotlin run 004 reached gameplay-script startup, timed out before listening and
stopped normally (shutdown.json). Its startup thread dump showed class loading.
The next run uses archive copies of 138 original class/resource directories, in
unchanged classpath order. This packages existing binaries only; no compilation
or gameplay modification. The package manifest records archive hashes.

scripts/integration-seed-inventory.py prepares the offline inventory-v1 fixture
only in the two explicit disposable databases. It refuses online characters or
different existing contents. Do not seed before capturing the empty-account A/B
baseline. Seeding is fixture setup, not a verified gameplay transaction.

rust-actions-002 stopped before admission due to a startup symbol alias mistake:
Kotlin property bobs_axe_flyer resolves cache symbol flier (956). The server now
uses that actual cache symbol, and the rebuilt executable passes compilation.
The failed run's shutdown.json confirms the owned process exited normally.

Additional owned native scenario modes are prepared (not yet built or run):
`dialogue-ui`, `dialogue-protocol`, `inventory`, `inventory-verify` on both A/B
runners. Inventory scenarios use the copied library's interface dispatcher,
confirm equip/unequip through received containers, preserve item totals, then
leave the sword equipped for a new-login persistence check. Observations tolerate
the distinct inventory/equipment packets arriving in separate TCP fragments.
No native client implementation is modified. The scenario source/hash manifest
will be recorded by integration-client-build.ps1; outputs are isolated from all
Agent 1 Cargo state. Existing pinned live_actions remains the first replay target.

rust-actions-003 PASSED using the unchanged pinned live_actions binary: login
restored saved position 3220,3218, then server confirmed requested 3221,3218,
routed to man3/3108, sent the exact generic greeting and opened interface 217.
The native log confirms 26 subinterfaces and an actual interface change. This
verifies dialogue opening only, not Continue/choices. A subsequent database read
confirms final position 3219,3218 with 25 stats and world_id null.

Run 003 shutdown.json records forced cleanup of its exact owned server child:
the runner's old fixed wait was shorter than its 100-tick lifetime. Do not call
that a graceful server shutdown. Account logout had already saved successfully.
The runner now waits to the recorded tick budget plus 15 seconds before forced
cleanup, still bounded and always recording forced/normal status. Kotlin retry
005 has not started yet (memory preflight deferred; no run directory).

Kotlin run 005 was stopped through its owned stop file after detecting a harness
packaging defect: .NET ZIP creation omitted nonempty directory entries, so Flyway
could not discover db/migration in archives. It is not a reference gameplay pass.
The packager now adds every source directory entry explicitly and creates a fresh
versioned generation. The previous archives remain intact for run provenance.

### Kotlin A native capture 006

The pinned `live_session.exe` authenticated against the isolated actual Kotlin server (PoW completed) and captured 1,983 packets. The wrapper shut down normally (`shutdown.json`: exit 0, forced=false). Initial GPI/rebuild coordinates were Lumbridge 3222,3218,0; the saved account ended at the constructed Floor 1 hub 10424,120,0, with XP rate 500, Runecrafting 2 and Herblore 3. Inventory was empty. The different NPC count (six at the end versus Rust's Lumbridge population) is not a like-for-like NPC sync comparison.

The capture contains one REBUILD_NORMAL_V2 and zero REBUILD_REGION_V2 despite the saved relocation. This is a reference-side behavior to investigate: RspCycle.rebuildArea skips when knownBuildArea is NULL because a login rebuild has already been queued, while FloorOneScript can teleport during that login. Do not silently patch the reference or label this a Rust packet defect without tracing timing. Floor geometry/rebuild support remains necessary for the Rust implementation.

Observed native client failures: on-load script 8310 (widget 44498949) reaches unported opcode 106; MIDI_SONG_V2 and WORLDENTITY_INFO_V7 are unhandled. Exact client log, packet capture, comparison and saved state are in `work/integration/runs/kotlin-login-006/`. Capture comparison uses the older Rust login003 before SERVER_TICK_END was added; its tick-packet difference is historical, not a remaining Rust defect. A's cache was repacked during reference startup; archive identities must be reconciled before stronger visual parity claims.

Prepared follow-up observability: `integration-kotlin-login.ps1 -CaptureWorld` (after rebuilding the owned wrapper) writes startup Floor1 location, collision and NPC TSVs before game ticks begin. Collision includes NPC occupancy and must be normalized when comparing static geometry. Snapshot mode retains the actual installation, injector, prepareGame and startupGame calls; it adds observation between preparation and startup. Default mode still invokes the original main. This code has not yet been compiled/run; no snapshot result is claimed.

Generic inventory scenario placement: the offline fixture can explicitly place both disposable accounts at3224,3218,0, outside FloorOneScript's Lumbridge arrival-pad rectangle (x3219..3223,z3216..3220). This isolates ordinary equip/unequip and appearance behavior from Floor1's intentional tool banking/first-arrival stripping. The fixture record stores original and setup positions; placement is not evidence of movement or teleport parity. The original Floor1 relocation gap stays tracked, and the Kotlin006 saved-state snapshot is retained. No live account or reference gameplay hook is modified by setup.

UI investigation update: `scripts/integration-widget-cache.py` reads both private JS5 caches without a compiler/client. `work/integration/widget-cache-comparison.json` records identical unpacked group hashes for 149, 217, 162 and 548. Inventory 149 is an IF3 container; inventory type93 bytes are `02 00 1c 00` (28 slots) in both. Script6007 creates its dynamic children using that capacity. Historical A006/B003 captures also attach inventory149 to fixed-frame548:84. These observations narrow attribution but do not prove a client fault or successful native transactions.

Offline Floor 1 comparison preparation: `scripts/tower-snapshot.ps1` builds/runs an isolated cache-only Rust example with no account DB or listener. `scripts/tower-snapshot-compare.py KOTLIN_DIRECTORY RUST_DIRECTORY NEW_RESULT_JSON` compares exact location/NPC multisets and all32768 collision cells (including NPC occupancy), reporting bounded mismatch samples and counts. Outputs are evidence of startup state only, never a live-gameplay verdict. Startup snapshot-002 now matches Kotlin A002 exactly for all 788 locations, 27 NPCs and 32768 collision cells. Publication is wired into the accepting server source after content configuration, with a regression for cache deletions/replacements; the new executable and native floor runs remain pending.

First-login limitation observed in `kotlin-inventory-002`: client inventory driver C873D3171DC047A8F5AA5F41E8BC81CDD799A827FBEBA04D0285D6F1C5465B47, unchanged copied client core, reports `interface script 8310 widget Some((44498949, -1)): unported client script opcode 106`. Kotlin opens player_design679 because tut2_completed=false, and the tutorial locks the side tabs. Rust currently lacks that tutorial bootstrap, an independent server parity gap. To isolate ordinary equip/unequip, `integration-character-stage.py` records explicit OFFLINE tut2_completed varp2690=1 fixtures on both disposable accounts. This is setup, not proof of tutorial completion or character creation. Original fresh-account runs and their limitations remain retained.


Native inventory result: C873 driver against rust-inventory-006 completed equip, remove and reequip through native interface dispatch. rust-inventory-verify-001 restored the worn sword and its appearance on fresh login, with 1000 coins and two lobsters conserved. Both owned servers exited normally without forced cleanup. Kotlin inventory-004 authenticated/rendered but lost connection during startup-script processing and timed out before any item action; A/B transaction parity is not established.

Floor-native fixture: integration-floor-fixture.py backs up the logged-out disposable Rust inventory account into separate work/state/integration-floor and sets the copy to10424,120,0. It preserves the primary A/B inventory state and records fixture-only placement. Run integration-rust-login.ps1 with -StateName integration-floor and -Mode render or floor after building the new server/scenario. The floor scenario uses native walk/NPC action encoders and interface dispatcher operations for deposit-one/withdraw-one, without fallback injection. It has not yet run.

Subsequent native results: floor-actions002 and floor-content001 passed exact784ground-planeKotlin locations, doorway movement, banking and repeat vialSearch. floor-altar001 restored an explicitly drained Prayer fixture. rust-burial001 passedregular/batboneburial andpersistedfineXP490. Everyoneoftheseownedruns shut down unforced/exit0. TheseareRustlive results; Kotlin-nativefloor/altar/burialparityisnotclaimed. Kotlin-burial001 neverauthenticatedwithin90seconds andleftthe2bonefixtureunchanged. SeePROGRESS.md/currentmatrixforversions.

DialogueContinue failure now reproduces onKotlin: docs/NATIVE_DIALOGUE_FINDING.md recordswidget162:566hiddenandthematchinggreeting/layout. Nativeclientcoreisunchanged. ThisfindingisnotproofthatallRustdialoguepacketsmatchKotlin, andprotocolprobe115successisnotnativeUIproof.
