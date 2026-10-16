# Shared engine deltas

Initial baseline: `6a1ea78022bd1c9cbc7e259bcb3595652e5eaae5`.

No engine source changes in the initial publication checkpoint.

For each change record: files, motivating capability/defect, why existing interfaces are insufficient, new interface/behavior, verification, downstream dependencies, upstream conflict risk and whether Tower-specific or reusable. Keep content and engine integration commits separate where practical.

Known integration work includes successful-action objective credit, boss death attribution, live hazard scheduling, special location dispatch, atomic reward claims and an explicit completion/replay lifecycle. A hardcoded external historical combat catalog also needs an explicit supported configuration rather than a machine-local assumption.

## Reward transaction correction

`tower_chest::Config::claim` now stages grants using a clone of the existing inventory and commits only after every insertion succeeds. Previously a bank coin stack could satisfy the room precheck, allowing coins into the final free inventory slot before beer failed. The unpaid bounty could then be retried. Failure now preserves the complete inventory, emits only the waiting message and leaves diary payment pending. This intentionally departs from historical Kotlin partial-grant behavior.

Verification: all 62 Tower world tests pass, including three repeated failed claims with no inventory change and the successful paid-once claim. Runtime still applies inventory and diary sync in one simulation action; durable account-save crash behavior remains an acceptance requirement. No new inventory authority or storage framework was introduced. Conflict surface: `crates/world/src/tower_chest.rs`.
