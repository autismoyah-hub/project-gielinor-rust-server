# PROVING GROUND independent verification

Verification branch: `agent7/openrune-verification`.

This clone and its detached `../agent7-targets/<implementation SHA>` checkouts are independent of FORGE's working tree. Each build has a private `work/<SHA>/target`, Cargo home and temporary directory. The installed GNU compiler is a read-only tool reference. No live world, shared database, game port, client process or protected reference fixture is used by this initial batch.

## Intake and provenance

The requested short SHA `ba455a6` does not resolve in the fetched repository. The published initial 17-contract schema/validator commit is `ba455a0278f3f0b5ac6cbe377b6a8b83f0e95348`. Its actual validator was independently executed against its original queue, including Git checks. Its nine validator regression tests also passed. This establishes contract integrity only.

The first fetched checkpoint was `aa36efb30cfa1003e46177d3b9dfe5a4e1d9ae3d` with 18 records. A subsequent fetch consumed `b323ebdee860a052b2c42a32542e61bfd8340a7d` with 21 records while the original exact-SHA batch continued. Both snapshots and ancestry checks are preserved in `independent/`. New intake cannot replace an active test target.

FORGE's `legacy_evidence` is historical developer evidence. Independently rerunning FORGE-authored tests is recorded as independent execution of existing tests; it does not turn their design into an independently specified oracle. The separate external Rust test crate contains PROVING GROUND's own synthetic adversarial scenarios and links directly against the unchanged pinned production crate. No substitute gameplay engine is used. These scenarios test contract transaction invariants and retained arithmetic, not original-source or Wiki parity.

## Initial risk order

1. OR-MIG-0018: shared potion inventory/stat/energy/event transaction and selected-slot identity.
2. OR-MIG-0006: character chemistry charges, equipment loss, random draw order and stale dialog actions.
3. OR-MIG-0010: four selected ingredients, output/XP atomicity and secondary-saving modifiers.
4. Newly consumed OR-MIG-0019/0020: shared drinking effects, overboost caps, current-level drains and late-effect rollback.
5. OR-MIG-0021 and dependent production families: queued requirement revalidation, resource loss, fractional XP and persistence.

All initial 17 contracts and newly consumed records retain explicit intake rows. Unselected features are `NOT_ASSESSED`, never implicitly passed. Dependency ancestry was checked for every declared feature dependency. An unresolved environment, unavailable dependency, or later required correction must produce a `BLOCKED` attempt at the recorded SHA, with the exact reason; testing a newer build requires its own contract and attempt.

## Completed initial checkpoint

- OR-MIG-0018 at `71289e622cdfac55d5f11e1c857f325929ebff3f`: 14 existing world tests, three existing server binding tests, and four independent adversarial tests passed. The external tests exercise all 28 selected slots with duplicates/full bags, Drink/Empty, stale replay, malformed counts/metadata, late-effect rollback, stack redirection and overflow refusal.
- OR-MIG-0006 at `e7ac3608e61893d80b2de6a1a2e512b3ab785505`: ten world and three server tests passed, including authoritative chemistry action paths and packet-decoder/binding tests.
- OR-MIG-0010 at `5a260c1bd6efa5f8a221bed7c96e6ac72fd428bc`: four world and one server test passed after correcting the verifier's empty test selection. The earlier zero-test attempt remains explicitly blocked in the ledger.

Total: 39 Rust test functions passed across these exact targets, plus nine validator tests. Three features have partial V0/V1 coverage; the other 18 consumed features remain unassessed for gameplay. No feature is fully verified, no confirmed gameplay failure was found, and no critical/dependency-blocking escalation is required. There are no currently blocked feature campaigns among these three; one historical verifier-infrastructure attempt was resolved at its original SHA. V2/V3/V4 remain open for all 21 consumed features.

## Repeatable execution

From the verification repository, using Python 3 and Git:

```text
git fetch origin agent2/openrune-content
python scripts/proving-ground-sync.py
python scripts/proving-ground.py --batch OR-MIG-0018 OR-MIG-0006 OR-MIG-0010
python scripts/proving-ground.py --batch OR-MIG-0018 --independent
python scripts/proving-ground-sync.py
python scripts/proving-ground-finalize.py
```

`proving-ground.py` currently supports only the three reviewed suite mappings above; new contract intake is generic but requires a reviewed scenario mapping before execution. The default batch uses the latest published revision of the requested stable ID and pins that record's full implementation SHA. It does not build moving branch HEAD. The initial host compiler path is explicit in the runner; port it to another host by changing that read-only tool path, not the implementation target.

One build job, below-normal priority, at least 4 GiB available RAM, no observed active cargo/rustc, and a five-minute command timeout bound each attempt. A resource deferral is a blocked infrastructure attempt, not a build failure. Timeouts terminate only the runner's own process tree. Every subsequent attempt gets a separate evidence directory. Zero matching tests cannot count as acceptance. Source identity and tracked-file cleanliness are checked before and after execution.

## Coverage boundaries

The initial runs cover compilation and bounded synthetic headless execution: partial V0/V1. Passing filters do not establish every runtime definition, all registered dispatch paths, complete gameplay eligibility, or all shared regressions. In particular, the potion server filter initially covers recipe bindings, not a native potion action. Chemistry packet-decoder unit tests also remain headless tests.

V2 native-client actions, V3 real logout/relogin/database/restart durability, and V4 reference parity remain unassessed. No native acceptance, original Kotlin execution or Wiki reconciliation is claimed. Next campaigns need disposable runtime fixtures and databases, isolated process identities and ports, and exact-SHA manifests before launching. Synthetic inventory-capacity and replay checks are not persistence evidence.

`VERIFICATION_RESULTS.jsonl` is append-only. Intake, failed or blocked attempts, later corrections and successful reruns remain separate. `CORRECTION_BACKLOG.md` distinguishes gameplay defects from verification infrastructure gaps. Ordinary gameplay findings go into that backlog; only critical or dependency-blocking findings are escalated promptly to FORGE. No production implementation change is included in this verification branch.
