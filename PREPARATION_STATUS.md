# Preparation status — 2026-10-12

**Ready for private Git setup review, with explicit integration acceptance gaps.** Preparation tooling, isolated fixtures, a consistent rebuilt client pin, structured observations and a differential comparator are in place. This is not complete Rust server parity, a fully accepted client baseline, or authorization to begin broader gameplay work. No Git initialization, commit or push was performed.

## Deliverables

All seven requested files are in `C:/Projects/OSRSRSPS/rust-server`:

- `PINNED_CLIENT_BASELINE.md`: immutable source/build identity, hashes, evidence and reproduction.
- `INTEGRATION_HARNESS.md`: isolated A/B environments, fresh fixtures, launch/stop ownership and scenario commands.
- `TELEMETRY_SCHEMA.md`: bounded JSONL contract, privacy, observed/unknown distinctions and actual coverage gaps.
- `DIFFERENTIAL_TESTING.md`: comparison rules, actual first captured divergence, attribution and reproduction.
- `INTEGRATION_MATRIX.csv`: 15 scenario/dimension rows separating new-baseline evidence from historical results.
- `GIT_READINESS.md`: source inventory, proposed ignores, exclusions, backup risks and future setup checklist.
- `PREPARATION_STATUS.md`: this readiness report and stopping point.

## Verified during preparation

1. Consistent read-only snapshot of Agent 1's source: 3352 manifest files, hash verification PASS. Agent 1's active files and processes were not modified.
2. Independent snapshot build: PASS, offline single-job build in an owned working copy. Executable pin hashes verified. Rebuilt live_actions SHA-256 is `8C2162F69497F358911AEA83ED751F7E6A680ED4A37D22B0D49BC8E829F44EAC`.
3. Same rebuilt client, same owned cache and canonical initial gameplay state on A/B: login, initial world (zero missing regions), one confirmed walk and NPC dialogue opening PASS. Runs: `prep-kotlin-actions-002`, `prep-rust-actions-001`.
4. Kotlin fixture repair: a separate copied Kotlin schema preserves eight real successful Flyway migrations; canonical gameplay export exactly equals the Rust template. Failed original run `prep-kotlin-actions-001` remains BLOCKED evidence. No migration bypass and no existing/live DB reset.
5. Differential comparison ran on real equivalent-provenance observations. It correctly reports FAIL at `state.persisted`; Kotlin also logged Tower arrival. This is an observed difference requiring attribution, not an automatically diagnosed Rust defect.
6. Fresh Rust native Continue test: FAIL (`prep-rust-dialogue-001`). The widget had visible layout, but primary action returned false and emitted no opcode 115. This differs from older hidden-ancestor evidence and must not be described as the same proven cause.
7. All four preparation-launched servers exited without forced termination. Three completed test servers exited 0; the failed initial Kotlin schema run exited 1. Session lock released. No preparation server is intentionally left running.
8. Fifteen lightweight telemetry/comparison/result-assessor tests PASS; seven Python files compile; six PowerShell scripts parse; client pins and fixture equivalence check PASS. Logs: `work/preparation-unit-checks.log`, `work/preparation/consistency-checks.json`, `work/preparation/powershell-checks.log`.

Runtime evidence is under `work/integration/runs/`. Actual differential report: `work/preparation/actions-differential-001.json` and `.md`. Supplemental client validation: `integration/client-validation-001.json`. Immutable source/build manifests keep their original creation-time UNVERIFIED state; the supplement records subsequent partial validation without silently changing a pin.

## Remaining gaps and blockers

- Matched Kotlin native Continue retry was **BLOCKED before launch** by the existing CPU >30% guard (`work/prep-kotlin-dialogue-001.log`). No resource limits were relaxed. Earlier Kotlin Continue failures are historical, not a fresh matched driver result.
- Dedicated continuous connected-duration/long soak and fresh visible GUI acceptance are NOT TESTED. A bounded client loop or clean action disconnect alone does not prove sustained connection stability. Overall client baseline remains partially verified.
- Multi-waypoint/bank/object modes now have fresh per-run Tower fixtures and a hashed reference scene. Current-baseline runtime validation of those modes is NOT TESTED. Inventory/equipment/re-login drivers are rebuilt and runnable; their historical passes are kept separate from fresh validation.
- Fresh character creation is NOT TESTED. Canonical fixtures are tutorial-completed. Tower login hooks, initial-world relocation and persistence timing need explicit investigation after authorization.
- Telemetry currently adapts available logs, SQLite checkpoints and optional incoming packet captures. Direct server decisions, complete NPC/object/dialogue transitions, outgoing packet coverage and comprehensive ticks/timestamps remain unavailable. Null is not acceptance. Comparison cannot prove parity for unobserved dimensions.
- Drivers/server wrappers have bounded lifecycles, but there is no independent client watchdog for every possible hung native/GUI call. New floor-mode runtime validation remains outstanding.
- Current Rust source includes frozen, unfinished prayer book/drain/timer work from the prior assignment. Focused world tests passed earlier (100), but pending Kotlin oracle compilation and missing expected fixtures prevent claiming a clean current full-workspace test run. Preparation used the separately archived, previously validated server executable instead. No broad gameplay fix was attempted here.
- Existing local caches, binaries, databases, keys and reference material require exclusion and rights review before Git. Current `.gitignore` is insufficient for the new pins. Ignore additions are proposed, not silently applied.

## Resource and isolation policy retained

Agent 1 remains highest priority. At least 4 GiB free memory, CPU at most 30% for prebuilt runs, no concurrent compiler, sequential A/B execution, BelowNormal priority and two CPUs; one Cargo job for builds. Existing build guards remain. No production account, live service, Agent 1 environment or shared process was modified. No repeated resource polling or indefinite Gradle wait was used to finish this milestone.

## Next steps requiring authorization

Stop here. The next phase is approval of private repository roots/hosting, backup policy, explicit source allowlist and ignore rules in `GIT_READINESS.md`, then authorized Git setup. No additional credentials are needed for this report. Hosting/remote configuration and authorization to initialize/commit/push are still required before taking those actions.

After Git setup and separate authorization for integration development: run matched Kotlin Continue, fresh inventory/re-login and Tower sequences; add explicit connected-duration assertions and native visual acceptance; then investigate the preserved differential evidence and extend server-side observations. Do not treat those follow-on steps as started or complete.
