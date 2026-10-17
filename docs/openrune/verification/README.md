# OpenRune implementation verification handoff

FORGE publishes implemented gameplay and bounded development checks. PROVING GROUND independently verifies exact implementation SHAs. Queue integrity or historical native evidence does not certify gameplay acceptance.

## Files and validation

- `IMPLEMENTATION_QUEUE.jsonl`: append-only implementation revisions, one coherent testable feature per record.
- `IMPLEMENTATION_QUEUE.schema.json`: JSON Schema2020-12 for a single record.
- `scripts/validate-openrune-queue.py`: standard-library structural and cross-record validation, revision links, dependency cycles and optional Git path/symbol existence checks.
- `IMPLEMENTATION_STATUS.md`: current throughput, coverage boundaries and remaining obligations.

Run `python scripts/validate-openrune-queue.py docs/openrune/verification/IMPLEMENTATION_QUEUE.jsonl --git` from the repository. `--git` checks every implementation commit and listed Rust/dispatch path at that SHA. Finding a symbol is an integrity check; V0 must independently exercise reachability through the documented entrypoint. The validator enforces additional invariants beyond the schema, including all five distinct levels, an enabled V0 and append-only revision links.

## Publication and revision policy

Compile the affected implementation, check shared transactions/registration and publish its source commit first. Append a complete contract with that full published40-character SHA in a following metadata commit; never use HEAD, a future/self-referential SHA, or a build-success placeholder. Required dependencies must be another stable implementation ID or an explicit ENV requirement.

Keep stable OR-MIG IDs. A correction appends revisionN+1 with `supersedes` linking revisionN and its exact implementation SHA. Never rewrite an older contract or a verification attempt. Independent findings belong in GitHub Issues owned by PROVING GROUND and identify implementation ID, tested SHA, severity/class, expected/actual behavior, reproduction and artifact references. A fix publishes a new SHA and requests retesting; FORGE does not turn an independent failure into acceptance.

Critical defects (build failure, corruption, unsafe transactions, systemic security or widespread regressions) are fixed immediately. Dependency-blocking defects are fixed before dependent work. Ordinary isolated mismatches remain explicit correction backlog while unrelated migration continues. Deferred means unresolved, never accepted.

## Verification levels

- V0: exact-SHA compilation, registration, definition resolution and dispatch eligibility.
- V1: authoritative normal/negative actions, exact items/fineXP/state, timing, atomicity and interrupts.
- V2: actual native client actions, UI and authoritative confirmations.
- V3: logout/relogin, reconnect, interruption, capacity, crash recovery and relevant persistence.
- V4: supported original/OpenRune observations and pinned game rules. Unavailable compiled Kotlin is an explicit limitation, not invented evidence.

Every record lists all five levels with required/excluded reasons. Legacy FORGE evidence is marked `independent_verification:false`; independent conclusions must reference an exact tested SHA and remain separate. Protected local fixtures and source archives are not published by this queue. Resolve access or prepare independent fixtures before claiming coverage.

## Provenance and source inventory

OpenRune file SHA256 values match the preserved discovery inventory and local archive. Current records use independently authored behavior adaptation: root ISC is not blanket file/dependency clearance, and no source expression/assets were copied. Direct translation requires separate clearance and notices. Source-observed facts, pinned Wiki rules, deliberate adaptations and unresolved inference are separate fields.

The3508 discovered source/configuration records are not a gameplay-module denominator. Classification of a TOML/YAML/properties file is structural only. A source file/table linked to contracts can still contain unimplemented obligations. Unknown records remain `not_yet_assessed`; initial contracts do not reconcile all earlier implementations or all eligible gameplay.
