# Isolated integration harness

Run from `C:/Projects/OSRSRSPS/rust-server` with PowerShell 7. This is preparation infrastructure, not proof of full server parity.

## Environments and ownership

- A: isolated Kotlin reference source at `reference/kotlin`, upstream reference pin `cb6e66cc0f16a25b60ba6705d8f157cc22ab4869`, with documented integration overlays. Existing packaged runtime has 244 classpath files; hashes and overlay classes are recorded in `integration/resource-manifest.json`. Dedicated loopback port 43728. New per-run environments live in `work/integration/preparation/RUN`, with independent saves, temp directories and logs. The original Kotlin checkout and Agent 1's test environment are read-only to this harness.
- B: archived Rust executable `work/integration/server/57917C5CE4204CDC52329F1395ED1B84250AE96090FE34DF39DA9CF84B09AAE7/gielinor-server.exe`. The hash is checked before every run. Dedicated ephemeral loopback port; no port 43594. Per-run database `work/state/integration-prep-RUN/accounts.sqlite`. Current source contains later unfinished prayer work and is deliberately not substituted for this executable.
- Same rebuilt client pin on both sides. Both use the existing **owned Kotlin cache/symbol bundle**, checked before and after each run. Historical A/B physical cache copies differ; historical runs must not be treated as equivalent-cache evidence. Cache junctions are local owned inputs; no junction targets Agent 1. Kotlin runtime and overlay hashes are also checked.
- `work/preparation/session.lock` prevents concurrent preparation runs. A stale lock requires checking the recorded owned runs/processes before removing it; never remove it blindly. The harness does not kill shared processes or operate on production accounts.

## Disposable fixtures

The account `integration239` is a dedicated local test identity. Its generated password file and test RSA material are local-only, never printed into telemetry. Existing secrets are preserved. Provisioning changes only new SQLite copies.

`work/preparation/fixtures/basic-v1/state.sqlite` is the Rust schema template. `basic-kotlin-v1/state.sqlite` preserves the real isolated Kotlin Flyway history and schema. Both export to the same canonical initial-state hash:
`d54d496a937d013df93e841f6e747be11e4013d81b2cf6b9ebaeffee2231737b`.

Initial state: logged out, plane 0 at 3224,3218, run energy 10000, tutorial-completed varps, fixed stats, inventory sword 1277, 1000 coins and two lobsters, empty equipment and bank. This is **fixture setup, not a test of fresh character creation**. Templates are checked before cloning; existing databases are never overwritten. The initial failed Kotlin run used a Rust schema without Flyway history; it is retained as `prep-kotlin-actions-001`, BLOCKED. No fabricated history or migration bypass was used to fix it.

`floor` and `floor-content` clone the same templates and set initial coordinates to 10424,120,0 in the new DB only. They use the captured Kotlin Tower scene file pinned by `integration/floor-scenario-input.json`. This is an explicit constructed-world fixture, not a gameplay teleport assertion or general overworld test. New floor-mode plumbing has syntax/fixture checks but current-baseline runtime validation remains NOT TESTED.

Fixture creation commands (one-time only; refuse existing outputs):

```powershell
& $py scripts/integration-preparation-state.py fixture
& $py scripts/integration-preparation-state.py kotlin-fixture
```

`$py` is defined in `PINNED_CLIENT_BASELINE.md`. These templates already exist; do not run creation again to reset them.

## Scenario commands

Replace each run ID with a new lowercase ID. Run environments sequentially. The wrapper handles fresh state, process launch, collection, shutdown, exports and assessment.

```powershell
pwsh -NoProfile -File scripts/integration-preparation-run.ps1 -Environment rust -Run trial-login-001 -Mode login
pwsh -NoProfile -File scripts/integration-preparation-run.ps1 -Environment rust -Run trial-actions-001 -Mode actions
pwsh -NoProfile -File scripts/integration-preparation-run.ps1 -Environment rust -Run trial-continue-001 -Mode dialogue-ui
pwsh -NoProfile -File scripts/integration-preparation-run.ps1 -Environment rust -Run trial-inventory-001 -Mode inventory
pwsh -NoProfile -File scripts/integration-preparation-run.ps1 -Environment rust -Run trial-relogin-001 -Mode inventory-verify -ReuseStateFrom trial-inventory-001
pwsh -NoProfile -File scripts/integration-preparation-run.ps1 -Environment rust -Run trial-floor-001 -Mode floor
pwsh -NoProfile -File scripts/integration-preparation-run.ps1 -Environment rust -Run trial-objects-001 -Mode floor-content
pwsh -NoProfile -File scripts/integration-preparation-run.ps1 -Environment rust -Run trial-visible-001 -Mode render
```

Use `-Environment kotlin` and different run IDs for A. Re-login deliberately reuses the prior run's DB and requires that same environment. The predecessor must have a PASS verdict, completed save and clean shutdown. Its continued fixture ID hashes the original fixture, scenario, client executable and baseline; it excludes environment/run names so matched Kotlin/Rust procedures are comparable. This ID does not attest equal resulting state: strict comparison still compares every initial gameplay field and reports discrepancies. Fresh and continued fixtures remain distinct.

Use explicitly versioned `-InventoryDriver inventory-002` for matched inventory and inventory-verify runs. This immutable owned driver is already built, with executable SHA256 `130AAABCEC14465D55451AB5694B39AD4C9FF9B4AD54B8ED36220561683C414F`. Historical90/120-second drivers exhausted their deadlines after reference startup consumed81/105seconds. Driver002 bounds total time to150seconds and actions to30seconds after readiness, and asserts observed native IF_BUTTONX47 dispatch. The build verified the original client library/vendor/lockfile hashes; only the scenario example changed. Both runs must use the same driver hash. Container and appearance-update checks do not establish rendered-pixel correctness.

- `login`: authentication, player/NPC initialization, captured packets, bounded 120-second client loop. It can exit early on disconnect and therefore is **not by itself a connected-duration assertion**. Assessor rejects logged script failures even with exit code zero. Dedicated long-duration stability is NOT TESTED.
- `actions`: one confirmed walk and NPC Talk-to interface opening. Headless direct world-action dispatch, no claim of pointer acceptance.
- `dialogue-ui`: native widget layout and primary interface action must emit opcode 115 and reach the NPC response. No protocol fallback in this mode. It does not draw an OS window; visible acceptance still requires `render` plus manual interaction/evidence.
- `inventory`: native inventory tab/interface operations, equip, remove, conservation, re-equip. `inventory-verify` confirms saved equipment after a new login.
- `floor`: recorded scene assertion, four walking waypoints, bank deposit and withdraw with conservation. `floor-content`: same bank sequence plus repeated vial-crate Search and item count assertions. Source: `reference/NativeFloorScenario.rs`.
- `render`: separate visible native window using `--smoke`; saves `world.bmp` when emitted. Automated frame smoke is not manual graphical acceptance. Never infer visible success from headless results.

## Outputs and bounded lifecycle

Every launched run lives under `work/integration/runs/RUN`: server/client logs, launch metadata, result/shutdown JSON, before/final state, events JSONL, deterministic scenario-result JSON and preparation metadata. If startup fails, lifecycle evidence is preserved where available and the verdict is BLOCKED. A preflight deferral may only produce the caller's redirected log; it is not a scenario failure.

Rust is bounded to 300 ticks plus startup/save grace; the stop file normally ends it earlier. Kotlin has a bounded integration wrapper and startup deadline. Driver deadlines are versioned: historical90/120seconds, inventory002150seconds total with30seconds for actions. Finally blocks stop only the recorded child server, report forced termination, and close the lock. These are driver/server bounds, not an independent watchdog for an arbitrarily hung GUI/native call; that limitation remains documented.

Resource guards require at least 4 GiB free, CPU at most 30% for runs, and no active compiler. Owned processes use BelowNormal/two CPUs. Builds use one Cargo job and never overlap expensive Gradle work. Resource deferral is a reason to do documentation, not to change Agent 1's priority or processes.
