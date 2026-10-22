# Tower gameplay gap analysis

Audit: 2026-10-21 (America/Los_Angeles). Source baseline:
`c32edfcb2d652f4b5f7f028000dfe504e5c10705`, fetched from maintained server
`origin/main`. Branch: `agent7/tower-core-gameplay-20261021`.

This is an implementation inventory against the reassignment brief, **not a
completed charter reconciliation**. The owner clarified that V1.8 supplements
V0.1–V1.7 and explicitly authorized confirmed persistent progression, floor
unlocks, reusable objectives and safe persistence foundations. V1.8's actual
contents and the earlier charter remain unavailable in this checkout. Earlier
root-level Tower reports describe an older experimental
baseline and must not override the maintained consolidation report.

## 1. Implemented with existing verification evidence

| Capability | Maintained implementation / evidence | Limits |
|---|---|---|
| Fresh arrival retaining possessions and stats | `simulation/cinderworks_entry.rs`; entry tests | Existing configured starter policy is retained; setup XP is not earned XP |
| Three preparation services, Imp, Warden, coolant and hazards | `simulation/cinderworks_services.rs`, `cinderworks_encounter_actions.rs`; `CONSOLIDATION_REPORT.md` | Native historical first clear, not current-branch native certification |
| Durable run and first/replay distinction | `tower_run.rs`; storage V13 and `storage/src/tower_run.rs` | Existing reward behavior retained |
| Atomic rewards and durable acknowledgements | `simulation/cinderworks_actions.rs`, `cinderworks_checkpoint.rs`; `storage/accounts.rs` | Keep inventory, run state and acknowledgement in existing transaction/worker |
| Return, rescue, relogin and restart | `cinderworks_recovery.rs`; `docs/consolidation/restart-replay-validation.json` | Historical native source/binary scope must stay attached to its original evidence |
| Equipment, bank, wealth, fine XP, queued skills and combat | Existing world actions and account save transaction | Reuse owners; do not create replacement engines |

The evidence above is inherited implementation-owner evidence, not a new
independent Agent 7 acceptance finding. Current regression results are recorded
separately in `TOWER_SYSTEM_ACCEPTANCE.md`.

## 2. Implemented but incomplete for the wider Tower

- `Run.clears` and `first_reward_paid` survive replay, but objective bits belong
  to the current attempt. New V14 and `tower_progression::Progress` now preserve
  floor-indexed solo conquest separately. No historical service accomplishments
  are inferred from reset bits. New functionality needs the acceptance below.
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

The new increment supplies content-owned floor identity/prerequisites, required
versus optional objectives, permanent completion and first/replay classification.
A synthetic second-floor contract exercises eligibility and disk persistence;
it is not a live Floor Two. Detailed milestones, later-floor World integration,
boss catalogue, Boss Credit ledger, repeatable assignments, challenge completion
history and durable group ascent/instance ownership remain unimplemented.

## 4. Explicitly exploratory or undecided

Boss Credit earning rules, scope, failed-attempt treatment and distribution;
group roster persistence/replacement, shared loot, failure rules and group boss
replay eligibility; any new mandatory skill gates or final economy rates.
No defaults or database policy are selected here.

## Next dependency-aware increment

Implementation `1c2d4d8e0dd4d761ef156588a05f669d788d202e` adds the floor ledger
inside the existing save transaction, backfills positive V13 clears without
grants, and stages its World visibility behind the existing acknowledgement.
FORGE review and exact-source native acceptance remain required before integration.
Pin the complete charter/addenda before broadening gameplay and economy rules.
