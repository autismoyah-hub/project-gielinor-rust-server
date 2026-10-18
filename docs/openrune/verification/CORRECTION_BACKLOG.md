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

## User-directed execution deferral

Chemistry, potion and Herblore scenarios are deferred until explicitly resumed by the user. No historical failure or passing subcheck is erased. Ordinary food remains eligible without additional potion setup; Prayer, cooking, packs, assembly, smithing and gem cutting continue. See USER_DEFERRALS.json. Chemistry interrupt-002 had already failed a client count assertion when the deferral arrived; preserve it as an unresolved observer attempt, without rerunning or calling it a production defect. The proposed chemistry crash case was never executed.

## PG-INFRA-007 — corrected Prayer test discovery and read-only preflight setup

- Exact Prayer target: f6b0f6ad54fe7813bb6607827eeb8da0e60137fa. Initial existing integration invocation failed because the published world crate sets autotests=false. Five selected library tests had passed. A fresh owned external crate executed the unchanged, hash-recorded openrune_burial source: all3tests PASS, followed by production server compilation. No production checkout was edited.
- Read-only cache preflight first used64MiB, exceeding this decoder's24-bit group limit; corrected to0xffffff. Next attempt incorrectly required dummy=0, while absent ordinary target dummy metadata is-1; accept the decoder's ordinary nonpositive sentinel, retaining strict IDs/names/type/member/operation/template checks. Both raw failures and source snapshots remain historical verifier evidence. Corrected same-SHA preflight PASS: base fine rewards45/150/720for526/532/536. No gameplay defect attributed.
- Native selected burial plus stale replay and actual restart/relogin PASS with225fineXP/display22; full V1–V4 acceptance is not inferred from scoped subchecks.

## Food negative follow-up

Fresh negative-003 PASS at ae3d234c71764dd2364f8452fc1974a54a6783f0: full28cake selected replacement, stale whole-cake claim with untouched duplicate, actual brew10→13followed by legally delayed Eat preserving13and all XP. Real restart/native relogin preserves partial cake, dose and initial Stats. Negative-001 fixture setup and negative-002 immediate-after-drink timeout are retained verifier errors, not production findings. Additional potion-based scenarios are now user-deferred.

## PG-DEF-001 — native Close leaves chemistry policy dialog live

- Target: OR-MIG-0006, exact `e7ac3608e61893d80b2de6a1a2e512b3ab785505`.
- Severity: P2 ordinary isolated mismatch; not implementation-blocking. Continue unrelated FORGE work.
- Reproduction: worn Options, real Continue, reopen Options, genuine native Action::Close (95 empty), replay earlier real ResumePause115 with Stop choice. Local modal closes; authoritative saved policy changes Continue0 to Stop1. Charges2, inventory and all Stats unchanged; integrity ok, save acknowledged and graceful shutdown0.
- Expected: closed modal rejects stale response and retains `[2,0]`. Actual `[2,1]`.
- Root cause: exact target gameplay decoder falls through to Unhandled for95, leaving runtime chemistry dialog active. Route genuine Close into existing cancellation and reject stale held/worn replies. Reverify at explicitly published correction SHA; preserve historical failure.
- Evidence: `independent/deep-chem-close/close-001/independent-assessment.json`, raw native packet log, before/after SQL snapshots, frozen client/server hashes and launcher source. Raw launcher PASS means probe execution, while independent V2 verdict is FAIL. Earlier negative-002 corroborates; negative-001 was verifier misuse of Continue latch, not production evidence.
- Issue: https://github.com/autismoyah-hub/project-gielinor-rust-server/issues/123. User explicitly approved publication after automatic review initially rejected external disclosure. Current HEAD replay is not claimed.
- Status: OPEN correction; V2 historical FAIL. No FORGE stop requested.

## PG-INFRA-006 — resolved observer and capacity assumptions

- Retain negative-001: removing an actual Continue packet from outbound left the native Continue latch active, preventing real Cancel. Corrected observer dispatches real Cancel before replay and never strips generated packets. Fresh negative-002 and genuine Close case are separate immutable runs.
- Retain initial chemistry matrix: expected unchanged level5 despite legitimate XP crossing level6 was an oracle defect. Corrected 22000-combination matrix plus12 validation cases passes at unchanged e7ac3608.
- Capacity guard now matches project three-sample CPU70% policy while retaining stricter6GiB available RAM and restricting only owned children to two CPUs. Earlier30% deferrals remain measurements, not server failures.
- Food observer received all HP46 updates, full inventory22 and actual post-apply inventory snapshots; raw partial inventory18 was omitted. Assessment uses actual post-apply confirmation plus server trace and SQL, and independently decodes full22 on real restart. It does not claim raw18 capture. Initial assessor incorrectly expected a post-consumption full22; corrected without rerunning or altering successful campaign state.
- Observer compilation signed-position/unsigned-Walk mismatch corrected with checked conversions; failed source and build logs retained. No production correction needed.

## PG-INFRA-004 follow-up — blocker resolved for further execution

Fresh `deep-food-observer/observer-001` completed six actual native actions, including four formerly unexecuted bites, acknowledged save, real restart and native relogin. Raw received HP sequence2→3 before Eat, then3→6 from meat and6→10 from cake distinguishes regeneration and consumption. XP and unselected duplicate inventory remain conserved. Original failed driver and its missing packet observations remain historical; its exact missed-equality sequence is inferred, not retroactively fabricated. Full native negative/overheal/capacity and crash recovery remain open.

For each confirmed defect, record a stable finding ID, feature ID, full tested SHA, severity, expected/actual states, precise reproduction and scenario, evidence paths, affected dependency chain, implementation-blocking status, GitHub issue, and correction/retest history. Preserve historical failures after a fix. Search existing issues before filing a duplicate.
