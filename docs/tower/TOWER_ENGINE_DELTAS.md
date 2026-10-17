# Shared engine deltas

Initial baseline: `6a1ea78022bd1c9cbc7e259bcb3595652e5eaae5`.

No engine source changes in the initial publication checkpoint.

For each change record: files, motivating capability/defect, why existing interfaces are insufficient, new interface/behavior, verification, downstream dependencies, upstream conflict risk and whether Tower-specific or reusable. Keep content and engine integration commits separate where practical.

Known integration work includes successful-action objective credit, boss death attribution, live hazard scheduling, special location dispatch, atomic reward claims and an explicit completion/replay lifecycle. A hardcoded external historical combat catalog also needs an explicit supported configuration rather than a machine-local assumption.

## Reward transaction correction

`tower_chest::Config::claim` now stages grants using a clone of the existing inventory and commits only after every insertion succeeds. Previously a bank coin stack could satisfy the room precheck, allowing coins into the final free inventory slot before beer failed. The unpaid bounty could then be retried. Failure now preserves the complete inventory, emits only the waiting message and leaves diary payment pending. This intentionally departs from historical Kotlin partial-grant behavior.

Verification: all 62 Tower world tests pass, including three repeated failed claims with no inventory change and the successful paid-once claim. Runtime still applies inventory and diary sync in one simulation action; durable account-save crash behavior remains an acceptance requirement. No new inventory authority or storage framework was introduced. Conflict surface: `crates/world/src/tower_chest.rs`.

## Cinderworks foundation deltas

- `tower_layout.rs`: open service court replaces internal grid walls; collision now derives wall tiles from the actual shell. Exterior closure, room-centre connectivity and river preservation checks pass. Cache template publication and location-delta authorities remain unchanged.
- `tower_run.rs`, storage V12 and `tower_run.rs`, `accounts.rs`, `characters.rs`, `simulation.rs`: separate durable Direction B lifecycle, preserving legacy diary data. Existing character transaction includes run fields and inventory. Pending first-clear reward rolls back when a later inventory save fails. Transient encounters are intentionally excluded from persistence. Integration conflict surface includes the shared character projection and migration numbering; coordinate migration number during upstream integration.
- `runtime/movement.rs`, `runtime/tutorial.rs`: normal character creation remains mandatory; walking cannot close it while the character is hidden. After appearance/experience choice inside Tower, existing tutorial-completion effects hand off to Tower guidance instead of pointing to an absent island NPC. Tower handoff explicitly closes old tutorial guide overlays. This is production alternate onboarding, not an acceptance-only state edit.
- `tower_cinderworks.rs`, `simulation/cinderworks_actions.rs`, location dispatch: ordinary routed interactions start/recover a run, exit/return and claim rewards. Missing tools and rewards stage in existing inventory clones before commit. First-clear payment history survives replay. Source-only action tests cover insufficient space, repeated requests, exit/return and once-only payout.
- Successful smithing/cooking/offering callbacks credit the three services. No attempted-action or item-presence credit. New-run offerings do not invoke legacy forced walking; legacy login restoration is excluded once Direction B begins.
- Isolated client adapter patch: `client-validation.patch`, local client commit `4001176` against `95eed88159b06cebea24559a814a5111b54a59cd`. Optional environment directory; native and headless use ClientState menu actions and normal packets. Native WS_EX_NOACTIVATE prevents focus activation. Client remote remains disabled; patch is reviewable without modifying Agent 1's branch.

- `zones.rs`, protocol zone encoder, `ground_sync.rs`, runtime event dispatch: reusable transient tile graphics through the existing ordered zone observer stream (revision 239 enclosed record 5). Client already decodes these records. No location, collision, inventory or persistence authority changes. Wire test validates byte order and coordinate bounds; encounter use and native telegraph visibility are next.

## Encounter and service integration

- `cinderworks_encounter.rs`, `simulation/cinderworks_encounter_actions.rs`, ControllerMain hook: deterministic warning patterns, delayed queued hits, ownership bound to ActorId generation and run ID, phase changes, coolant windows and hero-attributed death credit. Reset clears transient hits/ownership and restores a surviving guardian without resetting durable services. Ordinary combat remains the damage/XP authority. NPC generic retaliation is suppressed for these scripted guardians. Shared conflict surfaces: simulation fields/phase hook, melee denial and NPC dispatch.
- `cinderworks_work.rs`, weak-queue dispatch and NPC handler: real net catches and range cooking use staged inventories and stats; no XP on failed output, no synthetic ingredients. Work checks current target identity and proximity on continuation. Fishing supports the authored shoreline approach. The old display furnace was replaced with the already-configured tutorial furnace after an actual native action exposed its missing binding.
- `smithing_actions.rs`: multiply existing global rate by the character's existing rate, matching other skill entry points. Regression check proves queued production at 1x and 5x. No retroactive XP edits are made to disposable characters.
- `journal.rs`, runtime and progress events: project current Cinderworks lifecycle on login, task-tab reopening/switching and real progress transitions. The legacy eleven-row state remains stored separately.
- Cache validation before admission requires positive guardian death-animation durations. Population gives Imp 18 HP and Warden 42 HP, no passive regeneration and an eight-tick respawn interval. These are pacing parameters awaiting native encounter review.

## Sanctuary and checkpoint integration

- `simulation/cinderworks_recovery.rs`: four-tick sanctuary rescue only for active Cinderworks characters inside the constructed floor. Normal death sequence, movement teleport and stat restoration; no item regrant, XP refund or alternate actor model. Zero-health saves reconstruct recovery; health comparisons preserve unsigned levels above 127.
- `simulation/cinderworks_checkpoint.rs`, world events and `runtime.rs`: guarded content checkpoints reuse `Character::save`, `Request::Save` and the account worker's SQLite transaction. Proposed reward inventory remains private until successful acknowledgement. World input and the runtime packet entry point reject gameplay during the bounded save barrier; logout waits. Required account retries preserve the barrier on uncertain writes. A runtime-local monotonic token replaces actor generation as request identity, retaining actor generation in public completion diagnostics. Observer synchronization stays active.
- Guardian victory aborts stale combat and checkpoints its unlock; The final readable timing uses four ticks for every guardian warning; the final Warden phase still accelerates cadence. Ground-level prototype stair removed; no upstairs continuation is exposed as the completion path.

- `tower_layout.rs`: mainland sources now use a cache-authored flat charcoal tile with a matching border (zone 364,1245); river sources retain actual bank/water clipping. Removes overlay-rich repeated gold seams without editing cache data or client terrain. Cinderworks arrival suppresses obsolete legacy kit restoration and uses the authored sanctuary introduction.

## Final presentation and legacy exclusion

- `journal.rs`, runtime configuration and `tower_run.rs`: Cinderworks-enabled servers show the authored introduction before run start without mutating run ID or legacy diary. Six compact rows and 14-pixel line spacing replace the old cramped header body for this route; normal legacy mode retains its original presentation.
- `tower_layout.rs`: audited flat, equal-height ninth-border cache sources distinguish slate, earth, moss and ash courts; existing river sources remain. The native ten-point tour verifies actual routing/rendering.
- `simulation/tower_login_actions.rs`: bypass the obsolete legacy ash timer entirely when Cinderworks is enabled. This removes unrelated movement XP, cinder rewards and automatic diary restoration; ordinary skill/combat callbacks remain authoritative. Strict client exit/return confirms XP preservation.
- Local client validation patch: listener dispatch acknowledgements reflect cache onOp submission even without a direct button mask. Automation readiness now matches the native UI's existing script-busy input barrier. No production client protocol or authority is replaced.

- `Runtime::configure_cinderworks` enables the authored journal after generic startup UI configuration, making configuration order explicit before admission. Every guardian telegraph now lasts four server ticks; final Warden cadence remains faster. This adjustment follows an actual native replay failure under frame stalls, not a fabricated success.

## Accepted validation boundary

The optional client patch now retries transient Windows observation-file sharing failures independently of action dispatch. Large debug dumps require `inspect`, ordinary observations stay lightweight, and native frame requests retain their own sequence until an atomic image write succeeds. A real read-only two-second file lock plus normal weapon swap/restore verifies continued native operation. These are local validation changes; production gameplay still uses the existing protocol and authoritative Rust engine.

Agent 2's final observed chemistry/queue APIs and the V12 migration collision are recorded in TOWER_INTEGRATION_CONTRACT.md; no integration or parity claim is made for that newer ref.

## Visual production reconstruction (2026-10-16, review in progress)

- `tower_layout`: supported straight/corner masonry edges, clipped-corner sanctuary, processional hall, enclosed service wings, fortified foundry and vestibule. Inherited clipping clears before wall application; real river terrain remains. Unreachable upper template is removed.
- `cache::tower_shell::stamp_shaped`: replace only the selected scene layer so wall decorations preserve supporting masonry and clipping. Plaque uses shape 4, portal shape 0.
- `server::tower_map`: exact construction-time deletion baseline for JS5-only conveyor frames 9109–9113 at source zone (242,620), offsets (2..6,7). Paired game map omits these decorations. Native read-only scene inventory proved the discrepancy; no protected cache or renderer edits.
- `combat_updates::NpcUpdates`: standard revision-239 initial name, combat-level and transform masks. Only the Tower Warden receives initial name `Ash Warden`, level 7 and visual transform 2075. Authoritative Warden remains 16295, preserving existing combat and XP metadata. Both cache footprints are two tiles.
- Cache-validated fire-giant charge/defend/death animations replace the Warden presentation; scoped melee defend animation mapping retains ordinary damage rules. Phase changes emit a native NPC flare; coolant emits a flinch and NPC steam effect. Four-tick tile warnings, hazard damage, ownership and account transactions remain unchanged.
- Coal braziers are ordinary placed cache assets with a harmless routed Investigate response; no progression or item grant.
- Native validation adapter adds opt-in, read-only `inspect` scene inventory. It writes only on request and retries transient Windows output errors. No scene mutations, renderer bypass, desktop input or gameplay state injection.
- Scenario controllers select the visible transformed Warden type (2075), update the portal route, and recover a rejected hidden inventory widget by reopening the normal inventory and reacquiring food. A rejected action sends no gameplay packet.

### Current readability and interaction refinement

Foreground southern room edges use supported low stone wall 5154 through `prepare_palette`; exterior, back walls and plaque support remain full-height 2341. Movement stays blocked, while range policy follows the actual cache asset. Native comparison at the foundry threshold shows the avatar remains visible. No renderer or input-readiness bypass is used.

Coolant uses stock rainwater barrel 30124 (`mistmyst_barrel_water`), whose genuine native cache has Search operation 1. Storage barrel 362 has no native Search action and is no longer a coolant fixture. Four coolant corners (10421,135), (10421,138), (10428,135), (10428,138) keep central cardinal/diagonal escapes clear. Obsolete lamps/stump inside hazard fields are removed or moved to perimeter recesses.

The shield eligibility rule is preserved, but automatic melee retries no longer flood chat with identical shell-denial messages. Initial instructions and phase/coolant cues remain. The first visible warning explains heat; subsequent waves use the actual animated telegraphs. The displayed aggressive dagger style (Combat Options → Lunge) trains Strength through ordinary combat, with no injected levels or XP.

The validation adapter now exposes existing normal Chat UIDs/cycles in JSON. This is observation only; it fixes acknowledgement ambiguity from repeated text and prevents old win messages from counting toward a new encounter. Native input readiness and all parity behavior remain unchanged. The current nine-patch handoff includes this observer change and the read-only scene inventory.
