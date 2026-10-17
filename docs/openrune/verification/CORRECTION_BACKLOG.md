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

All contracts require V0–V4. Current checks are partial V0/V1 only. Complete registration/definition/action coverage, independent modifier scenarios, native gameplay, disposable persistence/restart/reconnect campaigns and supported reference comparisons remain queued. These are coverage gaps, not alleged implementation defects. Where a concrete fixture, dependency or specification prevents execution, append a precise blocked attempt and a contract request rather than substituting another SHA.

The handoff's `ba455a6` identifier is an informational mismatch: the fetched original contract commit is `ba455a0278f3f0b5ac6cbe377b6a8b83f0e95348`. This did not block inspection or testing, and it is not a gameplay defect.

## Finding format

For each confirmed defect, record a stable finding ID, feature ID, full tested SHA, severity, expected/actual states, precise reproduction and scenario, evidence paths, affected dependency chain, implementation-blocking status, GitHub issue, and correction/retest history. Preserve historical failures after a fix. Search existing issues before filing a duplicate.
