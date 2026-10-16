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
