# Tower gameplay gap analysis

Audit: 2026-10-21 (America/Los_Angeles). Source baseline:
`c32edfcb2d652f4b5f7f028000dfe504e5c10705`, fetched from maintained server
`origin/main`. Branch: `agent7/tower-core-gameplay-20261021`.

This is an implementation inventory against the reassignment brief, **not a
completed charter reconciliation**. The complete charter V1.8 and subsequent
addenda were not found in the workspace or accessible Pages search. Requested
from the owner. Earlier root-level Tower reports describe an older experimental
baseline and must not override the maintained consolidation report.

## 1. Implemented with existing verification evidence

| Capability | Maintained implementation / evidence | Limits |
|---|---|---|
| Fresh arrival retaining possessions and stats | `simulation/cinderworks_entry.rs`; entry tests | Existing configured starter policy is retained; setup XP is not earned XP |
| Three preparation services, Imp, Warden, coolant and hazards | `simulation/cinderworks_services.rs`, `cinderworks_encounter_actions.rs`; `CONSOLIDATION_REPORT.md` | Native historical first clear, not current-branch native certification |
| Durable run and first/replay distinction | `tower_run.rs`; storage V13 and `storage/src/tower_run.rs` | One Cinderworks record per character, no general floor ledger |
| Atomic rewards and durable acknowledgements | `simulation/cinderworks_actions.rs`, `cinderworks_checkpoint.rs`; `storage/accounts.rs` | Keep inventory, run state and acknowledgement in existing transaction/worker |
| Return, rescue, relogin and restart | `cinderworks_recovery.rs`; `docs/consolidation/restart-replay-validation.json` | Historical native source/binary scope must stay attached to its original evidence |
| Equipment, bank, wealth, fine XP, queued skills and combat | Existing world actions and account save transaction | Reuse owners; do not create replacement engines |

The evidence above is inherited implementation-owner evidence, not a new
independent Agent 7 acceptance finding. Current regression results are recorded
separately in `TOWER_SYSTEM_ACCEPTANCE.md`.

## 2. Implemented but incomplete for the wider Tower

- `Run.clears` and `first_reward_paid` survive replay, but objective bits belong
  to the current attempt. There is no reusable floor-indexed completion/unlock
  history; do not infer historical service accomplishments from reset bits.
- Replay currently restarts the Floor One preparation loop. Boss-only catalogue
  admission is absent. Preserve current first/replay payment behavior pending an
  explicit superseding design decision.
- Guardian court ownership and receiver-private scenes isolate existing solo
  encounters. They do not establish durable parties or group accomplishments.
- Recovery and acknowledgement APIs are Cinderworks-specific; extracting their
  contracts requires shared-engine coordination, not a second persistence worker.
- Existing inventory/equipment/skills support permanent development, but there is
  no accepted cross-floor equipment/content progression in this audit.

## 3. Requested but not implemented in the inspected maintained Tower paths

These requirements come from the reassignment brief; charter approval remains
unverified: reusable floor definitions, per-floor entry eligibility, permanent
completion and milestone records, separate later-floor state, a minimal second
floor test slice, boss catalogue, Boss Credit ledger, repeatable assignments and
challenge completion history, durable group ascent/instance ownership.

## 4. Explicitly exploratory or undecided

Boss Credit earning rules, scope, failed-attempt treatment and distribution;
group roster persistence/replacement, shared loot, failure rules and group boss
replay eligibility; any new mandatory skill gates or final economy rates.
No defaults or database policy are selected here.

## Next dependency-aware increment

Obtain and pin the complete charter/addenda; reconcile each approved requirement
against this inventory. Then propose an additive floor ledger saved in the
existing account transaction and backfill proven Cinderworks completions without
granting rewards. Specify first conquest, pending reward and replay independently.
Review the extraction with FORGE before engine edits. Use a clearly identified
headless second-floor fixture before any authored Floor Two environment.
