# PROVING GROUND — current verification status

This is an ongoing verification campaign. No contract is yet fully accepted across all applicable V0–V4 levels. Historical results and current integration checks are separate.

## Intake

Latest observed FORGE HEAD: `1efda0299062de44fd7196e6555c19b87f02d12b`.
Exact queue SHA256: `17040ee9bd9695e52145e32fb1d26e9d9302b9f2b6b541d314c3d3bdcdeef187`.
Published queue: 37 records / 35 stable implementation IDs. Published validator: PASS.
The copied queue is refreshed only from fetched immutable blobs after append-history checks. Current coverage and every contract revision are retained in [WORKLIST.json](WORKLIST.json). Synchronization observations are append-only in [SYNC_OBSERVATIONS.jsonl](SYNC_OBSERVATIONS.jsonl).

## Level coverage

- V0: PASS 2, FAIL 0, BLOCKED 0, IN_PROGRESS 5, NOT_STARTED 28, NOT_APPLICABLE 0.
- V1: PASS 0, FAIL 0, BLOCKED 0, IN_PROGRESS 7, NOT_STARTED 28, NOT_APPLICABLE 0.
- V2: PASS 0, FAIL 0, BLOCKED 0, IN_PROGRESS 2, NOT_STARTED 33, NOT_APPLICABLE 0.
- V3: PASS 0, FAIL 0, BLOCKED 0, IN_PROGRESS 2, NOT_STARTED 33, NOT_APPLICABLE 0.
- V4: PASS 0, FAIL 0, BLOCKED 0, IN_PROGRESS 3, NOT_STARTED 32, NOT_APPLICABLE 0.

PASS applies only to the named level, target and scope. IN_PROGRESS retains passing subchecks while required acceptance cases remain open. The old coarse ledger is retained; [LEVEL_RESULTS.jsonl](LEVEL_RESULTS.jsonl) is authoritative for the new level-specific program.

## Deep evidence

- Super-combat at `5a260c1b`: six genuine native inventory/menu combinations; exact item/quantity packets and authoritative two-tick transitions; six outputs and 45,000 fine XP; acknowledged save, actual process restart and native relogin without replay. [Assessment](independent/deep-combat/independent-assessment.json).
- Chemistry at `e7ac3608`: native Options→Stop→Break→Confirm; selected amulet removal, five next-amulet charges and durable stop policy; actual restart/native relogin. [Assessment](independent/deep-chemistry/independent-assessment.json).
- Historical worn chemistry at `e7ac3608`: separate real native equipment Options→Stop, acknowledged save, process restart and native relogin. Worn amulet, charges, stop policy, coins and XP conserved. [Assessment](independent/deep-chemistry-worn/hist-worn-002/independent-assessment.json).
- Reference comparison: seven independently hashed source/pinned Wiki records for super-combat, plus chemistry control source inspection. Documented dose, timing, boost and transaction adaptations remain distinct from literal donor behavior. [Comparison](independent/deep-combat/reference-comparison.json).
- Food revisions 2 (`a560034a`) and 3 (`ae3d234c`): the four independently authored tests passed separately at both exact SHAs, with 319 synthetic fixtures per target through actual `World::submit`/`World::tick`; stale claims, duplicate/replay input, full bags, remainders, malformed inputs and overheal. Initial verifier assumptions and failed source/logs are preserved. [Test source](../../../verification/ordinary-food/tests/authoritative.rs).
- Item packs at `caefcf42`: atomic selected-slot/overflow helpers, server definition bindings and authoritative repeat/movement-cancellation/stale-slot scenarios passed. Native, persistence and reference acceptance remain open.
- Dynamic food at `14461673`: independent action matrix passed across 1,584 base-level/HP fixtures plus live-base/stale replay and seven rollback cases. Independently hashed three protected sources and four pinned Wiki records; strawberry prose/table and watermelon donor/Wiki differences remain explicit. [Comparison](independent/reference-food-1446167370dece10751c4c332929af880259ee23/comparison.json).

No rendered-window, original-executable differential or crash-atomicity acceptance is claimed. Native packet evidence uses the real pinned Rust client library/binaries and actual server transport, without desktop automation. Client binary/source correspondence relies on the preserved, hash-verified baseline/driver manifest; a fresh independent client rebuild has not been performed in this campaign.

## Integration regression

- `4ef5f5c1ebc50b81e56de4f3e2421e2bba49de03`: PASS; potion 44, chemistry 12, combat_mixing 4, inventory 27, queues 9 test executions. [Manifest](independent/integration-4ef5f5c1ebc50b81e56de4f3e2421e2bba49de03/manifest.json).
- `97b25c57d4c6dc0606adf91dd00e41ec43600a27`: PASS; energy-food 2 test executions. [Manifest](independent/integration-external-energy-food-97b25c57d4c6dc0606adf91dd00e41ec43600a27/manifest.json).
- `8ad59b108e1ecd55185570c88d1be8be06b39756`: Native current-integration worn Options -> Stop, equipped-item/coins/XP conservation and actual restart/native relogin; complete V2/V3 acceptance remains open. [Assessment](independent/native-chemistry-worn-8ad59b108e1ecd55185570c88d1be8be06b39756/independent-assessment.json).

An older integration PASS is not a PASS on a newer FORGE HEAD. Change-impact lists and the last executed integration scope are recorded separately per feature.

## Corrections and constraints

- Confirmed critical/dependency-blocking gameplay findings: none in completed scoped tests.
- Confirmed ordinary gameplay defects: none in completed scoped tests.
- Verifier defects: initial super-combat filter, food trace/dirty-slot assumptions, and dynamic-food argument types/synthetic labels corrected; failed attempts retained. [Backlog](CORRECTION_BACKLOG.md).
- Resource deferrals used measured CPU/RAM; later builds used one job and private targets. No unrelated process was changed.
- Optional third-party JSON Schema engine is unavailable; the actual published standard-library validator passes. This is not counted as a gameplay failure.
- Protected source/cache, account templates, credentials, native executables and databases stay local. Published material contains synthetic tests, hashes, scoped state snapshots and logs.

## Active work and continuation

- Lane B — OR-MIG-0006 at `e7ac3608e61893d80b2de6a1a2e512b3ab785505`: Complete native negative/cancelled held and worn dialogs, last-charge production/modifiers and interrupted production persistence; retain completed positive controls/restart evidence.
- Lane C — OR-MIG-0006 at `1efda0299062de44fd7196e6555c19b87f02d12b`: Inspect shared changes since the last recorded native integration campaign; select affected current-build regression scopes without inheriting older PASS results.
- Lane A — OR-MIG-0024 at `ae3d234c71764dd2364f8452fc1974a54a6783f0`: Expand runtime packet and genuine native ordinary/remainder food acceptance after completed revision-3 authoritative tests; item-pack native/lifecycle remains separate.

[RESUME_QUEUE.json](RESUME_QUEUE.json) contains exact targets and all outstanding feature levels. Do not infer that work runs after the active session ends.

## Current contract coverage

- **OR-MIG-0001 r1** `d8978ede5677` — Manual ordinary herb cleaning; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0002 r1** `a5e21e03c483` — Unfinished potions: ordinary water and three alternate bases; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0003 r1** `f3b4b984af8f` — Initial twenty-nine finished-potion profiles; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0004 r1** `613b29ac4e3d` — Herb tar batch production; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0005 r1** `87c4a3def092` — Seven ordinary tool-preserving grinding routes; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0006 r1** `e7ac3608e618` — Ordinary chemistry charges and Options/Break controls; V0 PASS, V1 IN_PROGRESS, V2 IN_PROGRESS, V3 IN_PROGRESS, V4 IN_PROGRESS.
- **OR-MIG-0007 r1** `693dd370dac7` — Stamina dose upgrades; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0008 r1** `03bffb1fab1d` — Extended anti-venom+ dose upgrades; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0009 r1** `29d9751117da` — Thirty-two divine dose upgrades; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0010 r1** `5a260c1bd6ef` — Four-input super combat production; V0 PASS, V1 IN_PROGRESS, V2 IN_PROGRESS, V3 IN_PROGRESS, V4 IN_PROGRESS.
- **OR-MIG-0011 r1** `289981e0efb1` — Anti-venom four-dose family; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0012 r1** `289981e0efb1` — Extended antifire dose family; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0013 r1** `03ce46c812d4` — Forgotten brew nonlinear dose awards; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0014 r1** `da62d6154b74` — Ancient brew selected dust stack production; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0015 r1** `337cbd9e7209` — Nihil shard stack crushing; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0016 r1** `b1cade8a86f3` — Menaphite remedy production; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0017 r1** `d09659ad9265` — Four-dose Anti-venom+ with both Torstol forms; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0018 r1** `71289e622cdf` — Selected-slot atomic potion consumption; V0 IN_PROGRESS, V1 IN_PROGRESS, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0019 r1** `d02633e45576` — Ancient and Forgotten brew drinking effects, all eight doses; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0020 r1** `cf75fa343a0f` — Saradomin brew drinking, four doses; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0021 r1** `c8b9aef260db` — Queued smithing live requirements and ordinary iron outcome safety (historical backfill); V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0022 r1** `71fa140f48a1` — Ordinary Bastion and Battlemage potion drinking, eight doses; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0023 r1** `caefcf424869` — Seventeen ordinary item-pack profiles, atomic opening and weak repeat queue (historical backfill); V0 IN_PROGRESS, V1 IN_PROGRESS, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0024 r3** `ae3d234c7176` — Ordinary selected-slot food transaction and potion-overheal integration safety; V0 IN_PROGRESS, V1 IN_PROGRESS, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0025 r1** `a560034a59e7` — Fifteen additional ordinary multipart food stages: chocolate cake, topped pizzas and pies; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0026 r1** `5b47d7fe4270` — Ten missing ordinary food healing profiles; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0027 r1** `f1f0032d5440` — Four ordinary water-source vessels with checked selected-slot filling (historical backfill); V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0028 r1** `231f33710596` — Ordinary dairy-cow harvesting with selected bucket and packed approach flags (historical backfill); V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0029 r1** `d8978ede5677` — Nine ordinary dough choices with selected container returns and quantity repeats (historical backfill); V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0030 r1** `eeb270e37b44` — Nine prepared fruit portion healing profiles; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0031 r1** `0fe7d767e630` — Manual three-stage Admiral pie assembly and optional shared timing; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0032 r1** `fe04a83de2b6` — Twenty-seven ordinary pie and pizza assembly recipes (historical backfill); V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0033 r1** `ae3d234c7176` — Seven skill-boost pie families, fourteen bites and dish returns; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0034 r1** `97b25c57d4c6` — Summer pie and papaya energy restoration through shared food effects; V0 IN_PROGRESS, V1 IN_PROGRESS, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0035 r1** `1446167370de` — Four live-base-HP food healing profiles with explicit rounding rules; V0 IN_PROGRESS, V1 IN_PROGRESS, V2 NOT_STARTED, V3 NOT_STARTED, V4 IN_PROGRESS.
