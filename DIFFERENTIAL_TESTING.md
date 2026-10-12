# Kotlin-versus-Rust differential observations

Kotlin is the behavioral reference. A difference is an observation requiring attribution, not automatic proof of a Rust defect.

## Workflow

1. Verify the client pin and shared cache/runtime hashes. Run A then B with the same mode and canonical initial fixture. Never compare different binary versions or historical physical cache copies as if equivalent.
2. The preparation wrapper exports initial/final state and imports bounded JSONL. Preserve failed-run evidence; use a new run ID for retries.
3. Compare using the same interpreter documented in `PINNED_CLIENT_BASELINE.md`:

```powershell
& $py scripts/integration_telemetry.py validate work/integration/runs/prep-kotlin-actions-002/events.jsonl
& $py scripts/integration_telemetry.py validate work/integration/runs/prep-rust-actions-001/events.jsonl
& $py scripts/integration_telemetry.py compare work/integration/runs/prep-kotlin-actions-002/events.jsonl work/integration/runs/prep-rust-actions-001/events.jsonl work/preparation/review-actions-diff-001.json
```

Output is machine-readable JSON plus a Markdown discrepancy report alongside it. Existing outputs are never overwritten. Reproduction runs use the exact two commands in `PINNED_CLIENT_BASELINE.md` with new IDs. Imports can also be invoked directly with `import RUN OUTPUT --environment kotlin|rust --scenario MODE --baseline ID --fixture HASH --cache HASH --detail checkpoints|packets`; unknown provenance must remain unknown, not guessed.

## Deterministic comparison rules

Scenario, baseline ID, executable hash, fixture, cache, input mode and revision must match and be present; otherwise BLOCKED. Compare available state/action/interface/NPC/object/decision/lifecycle checkpoints in observed order. Report the first differing checkpoint with full expected Kotlin and actual Rust observations, event ID/tick, suspected subsystem and source evidence path/line. Check relative action timing only when both have ticks. Current adapter coverage is enumerated in `TELEMETRY_SCHEMA.md`.

PASS means covered checkpoints match and both runs have clean lifecycle plus initial, action and persisted-state coverage. Equal failures and missing required coverage are BLOCKED. A mismatch is FAIL with classification `observed difference; attribution required`. NOT TESTED is used for scenarios or dimensions without evidence. `scenario-result.json` is a separate deterministic driver verdict: targeted assertions and exit status can pass while the cross-server comparison fails.

## Actual preparation result

`work/preparation/actions-differential-001.{json,md}`: **FAIL**, first meaningful captured difference `state.persisted`, checkpoint ordinal 2. Both runs authenticated, built the world with zero missing regions, confirmed a walk to 3225,3218 and opened dialogue 217 using the same executable, cache and initial state. Kotlin additionally logged the Tower arrival and a rebuilt constructed world. Kotlin's exported saved position remained 3224,3218; inspect the full report for Rust state and other retained differences.

The comparator correctly keeps these differences. It does not yet observe every intermediate dialogue or relocation event, so this is the **first captured comparable divergence**, not necessarily the first real-time cause. Investigate reference login hooks, Tower/realm initialization, delayed activity, shutdown save timing and server behavior before assigning a defect. Do not fix gameplay during this preparation milestone.

Historical native Continue fails and other earlier results retain their actual client/cache provenance. Older A/B drivers sometimes differ by diagnostics; do not label those an exact matched executable test. Direct protocol Continue probes are separate from native/visible Continue acceptance.

## AI consumption

Consume `scenario-result.json`, the comparison JSON, initial/final state and their referenced logs. Summaries expose status, expected markers, first failing action, limitations, run IDs, executable hash and reproduction command. Use actual first-divergence events for expected/actual state. Unknown acceptance/timing/NPC data remains unknown. AI may suggest attribution after Git setup; no model inference, gameplay planner, or outcome invention is part of this harness.
