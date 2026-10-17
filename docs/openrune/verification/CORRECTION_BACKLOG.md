# OpenRune correction backlog

## Gameplay findings

No confirmed gameplay defect from the completed initial scoped tests. This is not acceptance of untested behavior. No critical or dependency-blocking finding currently requires FORGE to stop or change its implementation sequence.

## PG-INFRA-001 — empty super-combat test selection

- Class: test-infrastructure blocker; verification harness only.
- Feature: OR-MIG-0010.
- Exact implementation SHA: `5a260c1bd6efa5f8a221bed7c96e6ac72fd428bc`.
- Reproduction: `cargo test --offline --locked -j 1 -p gielinor-world --lib super_combat`.
- Expected: execute four-input mixing and its authoritative action tests.
- Actual: Cargo compiled successfully but selected zero tests. This supplies build evidence and no gameplay acceptance.
- Cause: initial PROVING GROUND filter used a feature name instead of the module name.
- Correction: use `combat_mixing` for both world and server filters; count selected tests and reject zero-test runs.
- Evidence: `independent/OR-MIG-0010-5a260c1bd6ef/` and its subsequent attempt directory.
- Implementation blocking: no. No FORGE production correction or GitHub issue is appropriate for this verifier-owned defect.
- Status: resolved. The corrected filter reran at the same exact SHA: four world tests and one server test passed. The initial empty-selection logs and ledger row remain preserved. The original runner's raw manifest reported `PARTIAL` from exit codes alone; the ledger explicitly classifies that attempt as `BLOCKED`, and the corrected runner rejects zero-test selections.

## Open coverage work

All contracts require V0–V4. Native super-combat and chemistry controls, acknowledged-save/restart/relogin recovery, and source/reference subcomparisons now have independent evidence. Complete negative/modifier matrices, interrupted production, applicable crash recovery and remaining content still need acceptance. See DASHBOARD.md for current levels. Coverage gaps are not alleged implementation defects. Where a concrete fixture, dependency or specification prevents execution, append a precise blocked attempt and a contract request rather than substituting another SHA.

## PG-INFRA-002 — food verifier trace and dirty-slot assumptions

- Class: resolved verification-infrastructure defect; no confirmed gameplay defect.
- Feature/revision: OR-MIG-0024 revision2, `a560034a59e766d1faf400b61187fec861d13475`.
- Initial attempt: `independent/OR-MIG-0024-a560034a59e7-independent/` preserves the failed test source, hash, binary identity and logs.
- Actual: ordinary food does not always emit the optional multipart `FoodTransition`; a normal authoritative tick can update inventory synchronization metadata while item slots remain identical.
- Expected correction: assert exactly one real food animation for successful queued consumption and compare actual inventory slots for stale-input item conservation. Keep HP, all XP, delay and replay assertions.
- Retest: corrected independent external crate passed four test functions covering319 fixtures through actual `World::submit` and `World::tick` at the same SHA. The failed attempt remains historical verifier evidence, not an implementation failure.
- FORGE impact: none; no production fix or GitHub issue requested.

## Measured execution deferrals

Chemistry build attempts were deferred during observed compiler contention, including53% CPU. Later one-job builds proceeded with spare CPU and over6GiB available RAM; no other agent was paused or altered. Resource measurements are preserved with the deep campaign. A deferral is not a failed server build or a gameplay defect.

The handoff's `ba455a6` identifier is an informational mismatch: the fetched original contract commit is `ba455a0278f3f0b5ac6cbe377b6a8b83f0e95348`. This did not block inspection or testing, and it is not a gameplay defect.

## PG-INFRA-003 — dynamic-food verifier argument types and synthetic labels

- Class: resolved verifier-only defect, no confirmed production defect.
- Target: OR-MIG-0035 at `1446167370dece10751c4c332929af880259ee23`.
- The first external harness used unsigned variables for signed stat setter arguments. Cargo rejected the test crate; the unchanged production world library compiled. Checked conversions now cover the bounded 1–99 base and 0–104 current values.
- Independent pinned-reference inspection also corrected swapped watermelon/sweetcorn labels in synthetic fixtures (5984 and 5988 respectively). This affects test identity; these fixtures never claimed configured runtime binding acceptance. Earlier source snapshots and logs remain preserved.
- The intermediate corrected policy suite passed three test functions; the final label-corrected suite is a separate attempt. A measured 35.55% CPU deferral between attempts is recorded as execution contention, not a build or gameplay failure. Current completed outcomes are in WORKLIST.json and the attempt manifests.
- No FORGE source edit, stop request or gameplay issue is appropriate.

## PG-INFRA-004 — native food observation timeout; diagnosis remains open

- Scope: OR-MIG-0024 revision3 at `ae3d234c71764dd2364f8452fc1974a54a6783f0`; pinned `food-001` driver. No later build substituted.
- Raw actual result: the six-action driver exited1 at its finite150-second timeout. It confirmed meat2142 selected-slot consumption and HP2→5, then sent the cake1891 action at slot3. Server trace records tick99 selected replacement1893, HP5→9 and unchanged11540 fineXP. The durable database subsequently contains HP10. Four later cake/pizza actions were never sent and remain untested.
- Independent follow-up: actual graceful save/shutdown, restart, genuine native authentication, and independent decoding of2061 captured server packets confirm the durable partial-cake inventory, preserved earlier identical slots, coins and displayed XP. This proves partial durable state, not a passing six-action driver verdict.
- Diagnosis: the pinned driver requires the exact intermediate HP9 while waiting for scripts/packet state. An independent unchanged-World test reproduces a valid food5→9 transition followed by passive regeneration to10 without another food or XP award. The initial same-clock regeneration hypothesis was rejected by its test; refined subsequent-clock evidence passes. Both sources/logs are retained in `independent/integration-external-food-regeneration-*`.
- Status: OPEN verification observation blocker. The original production-session client does not log each received HP/inventory update, so the exact missed-observation cause is still inferred, not conclusively demonstrated. No confirmed gameplay defect or FORGE stop request.
- Next: instrument a new owned observer to record each received inventory/HP update and script-busy state, correlate exact server traces and regeneration, and execute the remaining four bites using a fresh disposable run. Do not overwrite the failed campaign or replay its initial fixture against its post-consumption database.

## PG-INFRA-005 — unselected assembly duplicate admission assumption

- Class: resolved independent-test oracle defect; no confirmed production defect.
- Exact target: OR-MIG-0040 at `d63c0258dd6960bffc3ef43736f4140efb3b757a`.
- The initial1,512-case matrix assumed all four captured material slots matched the fixture's nominal rotation. Only the two explicitly clicked slots have that guarantee; the other ingredients may validly be captured from the first matching ordinary duplicate at admission. The initial test failed at its admission assertion before attempting a transaction in that fixture.
- Correction: assert both clicked slots exactly, validate each other captured ingredient, and preserve/assert all uncaptured slots. Late stale tests remove the actual captured material while leaving a matching duplicate present, so the no-fallback requirement remains independently tested.
- Retest: corrected matrix PASS at the same unchanged SHA. Both harness snapshots, source/executable hashes and logs remain in `independent/integration-external-assembly-batch-*`. All three refused returns, live level, membership, invalid positive-XP rate, repeated captured slot and completed-job replay cases remain strict.
- No FORGE edit, stop request or gameplay issue requested.

## Finding format

For each confirmed defect, record a stable finding ID, feature ID, full tested SHA, severity, expected/actual states, precise reproduction and scenario, evidence paths, affected dependency chain, implementation-blocking status, GitHub issue, and correction/retest history. Preserve historical failures after a fix. Search existing issues before filing a duplicate.
