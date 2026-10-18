# PROVING GROUND — current verification status

This is an ongoing verification campaign. No contract is yet fully accepted across all applicable V0–V4 levels. Historical results and current integration checks are separate.

## Intake

Latest observed FORGE HEAD: `6e469bc1dacab463190917e8189f8add83c08d8f`.
Exact queue SHA256: `b33764a1aa21d43cbc4d1ba84ca16ca1df4130206448063780354cfa367a7983`.
Published queue: 57 records / 53 stable implementation IDs. Published validator: PASS.
The copied queue is refreshed only from fetched immutable blobs after append-history checks. Current coverage and every contract revision are retained in [WORKLIST.json](WORKLIST.json). Synchronization observations are append-only in [SYNC_OBSERVATIONS.jsonl](SYNC_OBSERVATIONS.jsonl).

## Level coverage

- V0: PASS 3, FAIL 0, BLOCKED 0, IN_PROGRESS 5, NOT_STARTED 45, NOT_APPLICABLE 0.
- V1: PASS 0, FAIL 0, BLOCKED 0, IN_PROGRESS 8, NOT_STARTED 45, NOT_APPLICABLE 0.
- V2: PASS 0, FAIL 1, BLOCKED 0, IN_PROGRESS 2, NOT_STARTED 50, NOT_APPLICABLE 0.
- V3: PASS 0, FAIL 0, BLOCKED 0, IN_PROGRESS 3, NOT_STARTED 50, NOT_APPLICABLE 0.
- V4: PASS 0, FAIL 0, BLOCKED 0, IN_PROGRESS 3, NOT_STARTED 50, NOT_APPLICABLE 0.

PASS applies only to the named level, target and scope. IN_PROGRESS retains passing subchecks while required acceptance cases remain open. The old coarse ledger is retained; [LEVEL_RESULTS.jsonl](LEVEL_RESULTS.jsonl) is authoritative for the new level-specific program.

## Deep evidence

- Super-combat at `5a260c1b`: six genuine native inventory/menu combinations; exact item/quantity packets and authoritative two-tick transitions; six outputs and 45,000 fine XP; acknowledged save, actual process restart and native relogin without replay. [Assessment](independent/deep-combat/independent-assessment.json).
- Chemistry at `e7ac3608`: native Options→Stop→Break→Confirm; selected amulet removal, five next-amulet charges and durable stop policy; actual restart/native relogin. [Assessment](independent/deep-chemistry/independent-assessment.json).
- Historical worn chemistry at `e7ac3608`: separate real native equipment Options→Stop, acknowledged save, process restart and native relogin. Worn amulet, charges, stop policy, coins and XP conserved. [Assessment](independent/deep-chemistry-worn/hist-worn-002/independent-assessment.json).
- Reference comparison: seven independently hashed source/pinned Wiki records for super-combat, plus chemistry control source inspection. Documented dose, timing, boost and transaction adaptations remain distinct from literal donor behavior. [Comparison](independent/deep-combat/reference-comparison.json).
- Food revisions 2 (`a560034a`) and 3 (`ae3d234c`): the four independently authored tests passed separately at both exact SHAs, with 319 synthetic fixtures per target through actual `World::submit`/`World::tick`; stale claims, duplicate/replay input, full bags, remainders, malformed inputs and overheal. Initial verifier assumptions and failed source/logs are preserved. [Test source](../../../verification/ordinary-food/tests/authoritative.rs).
- Food revision3 native: meat confirmed by client; cake commit confirmed by server trace and real restarted native inventory capture (2,061 packets). The six-action driver timed out at an intermediate-HP equality check; four remaining bites were unverified in that original run; fresh instrumented observer-001 completed all six actions and real restart/relogin. Passive-regeneration diagnosis and raw failure are retained. [Partial assessment](independent/deep-food/independent-assessment.json).
- Item packs at `caefcf42`: atomic selected-slot/overflow helpers, server definition bindings and authoritative repeat/movement-cancellation/stale-slot scenarios passed. Native, persistence and reference acceptance remain open.
- Dynamic food at `14461673`: independent action matrix passed across 1,584 base-level/HP fixtures plus live-base/stale replay and seven rollback cases. Independently hashed three protected sources and four pinned Wiki records; strawberry prose/table and watermelon donor/Wiki differences remain explicit. [Comparison](independent/reference-food-1446167370dece10751c4c332929af880259ee23/comparison.json).
- Shared batch assembly at `d63c0258`: 13 independently executed existing tests and an independently authored 1,512-case full-bag/captured-slot/direction/late-failure matrix PASS. Pair/timed routing and cancellation regressions are retained; native, persistence and full reference acceptance remain open. [Independent test source](../../../verification/assembly-batch/tests/authoritative.rs).

No rendered-window, original-executable differential or crash-atomicity acceptance is claimed. Native packet evidence uses the real pinned Rust client library/binaries and actual server transport, without desktop automation. Client binary/source correspondence relies on the preserved, hash-verified baseline/driver manifest; fresh observer campaigns use an independent rebuild of 1,674 hash-verified pinned source/vendor files with observation-only logging and owned controllers.

## Integration regression

- `4ef5f5c1ebc50b81e56de4f3e2421e2bba49de03`: raw attempt PASS; potion 44, chemistry 12, combat_mixing 4, inventory 27, queues 9 test executions passed.  [Manifest](independent/integration-4ef5f5c1ebc50b81e56de4f3e2421e2bba49de03/manifest.json).
- `d63c0258dd6960bffc3ef43736f4140efb3b757a`: raw attempt FAIL; 0 test executions passed. Verifier assumed which unselected duplicate was captured; corrected selected-slot/no-fallback matrix passed. See PG-INFRA-005. [Manifest](independent/integration-external-assembly-batch-d63c0258dd6960bffc3ef43736f4140efb3b757a/manifest.json).
- `d63c0258dd6960bffc3ef43736f4140efb3b757a`: raw attempt PASS; assembly-batch 1 test executions passed.  [Manifest](independent/integration-external-assembly-batch-d63c0258dd6960bffc3ef43736f4140efb3b757a-1792281175590139100/manifest.json).
- `e7ac3608e61893d80b2de6a1a2e512b3ab785505`: raw attempt FAIL; chemistry-matrix 1 test executions passed.  [Manifest](independent/integration-external-chemistry-matrix-e7ac3608e61893d80b2de6a1a2e512b3ab785505/manifest.json).
- `e7ac3608e61893d80b2de6a1a2e512b3ab785505`: raw attempt PASS; chemistry-matrix 2 test executions passed.  [Manifest](independent/integration-external-chemistry-matrix-e7ac3608e61893d80b2de6a1a2e512b3ab785505-1792282956704102000/manifest.json).
- `97b25c57d4c6dc0606adf91dd00e41ec43600a27`: raw attempt PASS; energy-food 2 test executions passed.  [Manifest](independent/integration-external-energy-food-97b25c57d4c6dc0606adf91dd00e41ec43600a27/manifest.json).
- `ae3d234c71764dd2364f8452fc1974a54a6783f0`: raw attempt BLOCKED; 0 test executions passed. Measured-resource deferral; no substitute target: Measured resource guard blocked; no other process altered [Manifest](independent/integration-external-food-regeneration-ae3d234c71764dd2364f8452fc1974a54a6783f0/manifest.json).
- `ae3d234c71764dd2364f8452fc1974a54a6783f0`: raw attempt FAIL; 0 test executions passed. Rejected same-clock diagnosis; refined later-clock reproducer passed. This failed hypothesis is retained, not attributed to gameplay. [Manifest](independent/integration-external-food-regeneration-ae3d234c71764dd2364f8452fc1974a54a6783f0-1792280887389358300/manifest.json).
- `ae3d234c71764dd2364f8452fc1974a54a6783f0`: raw attempt PASS; food-regeneration 1 test executions passed.  [Manifest](independent/integration-external-food-regeneration-ae3d234c71764dd2364f8452fc1974a54a6783f0-1792280932175111100/manifest.json).
- `8ad59b108e1ecd55185570c88d1be8be06b39756`: Native current-integration worn Options -> Stop, equipped-item/coins/XP conservation and actual restart/native relogin; complete V2/V3 acceptance remains open. [Assessment](independent/native-chemistry-worn-8ad59b108e1ecd55185570c88d1be8be06b39756/independent-assessment.json).

An older integration PASS is not a PASS on a newer FORGE HEAD. Change-impact lists and the last executed integration scope are recorded separately per feature.

## Corrections and constraints

- Last-charge chemistry full-bag native production and stop-on-crumble with one pair remaining plus real restart PASS. Independent 22000-case chemistry modifier matrix PASS; complete V1-V4 obligations remain separate.
- Confirmed critical/dependency-blocking gameplay findings: none in completed scoped tests.
- Confirmed ordinary gameplay defect PG-DEF-001: genuine Close95 leaves chemistry policy dialog live at e7ac3608; stale Stop changes saved policy. V2 FAIL; ordinary P2, not implementation-blocking. GitHub issue123.
- Verifier defects: initial filter, food trace/dirty-slot assumptions and dynamic-food types/labels corrected. PG-INFRA-004 no longer blocks execution: fresh observer captured HP2->3 regeneration before Eat, all six actions and real restart/relogin. Original unrecorded missed-equality sequence remains inferred. [Backlog](CORRECTION_BACKLOG.md).
- Resource deferrals used measured CPU/RAM; later builds used one job and private targets. No unrelated process was changed.
- Optional third-party JSON Schema engine is unavailable; the actual published standard-library validator passes. This is not counted as a gameplay failure.
- Protected source/cache, account templates, credentials, native executables and databases stay local. Published material contains synthetic tests, hashes, scoped state snapshots and logs.

## Active work and continuation

- Lane B — OR-MIG-0006 at `e7ac3608e61893d80b2de6a1a2e512b3ab785505`: Complete native negative/cancelled held and worn dialogs, last-charge production/modifiers and interrupted production persistence; retain completed positive controls/restart evidence.
- Lane C — OR-MIG-0006 at `6e469bc1dacab463190917e8189f8add83c08d8f`: Inspect shared changes since the last recorded native integration campaign; select affected current-build regression scopes without inheriting older PASS results.
- Lane A — OR-MIG-0024 at `ae3d234c71764dd2364f8452fc1974a54a6783f0`: Expand runtime packet and genuine native ordinary/remainder food acceptance after completed revision-3 authoritative tests; item-pack native/lifecycle remains separate.

[RESUME_QUEUE.json](RESUME_QUEUE.json) contains exact targets and all outstanding feature levels. Do not infer that work runs after the active session ends.

## Current contract coverage

- **OR-MIG-0001 r1** `d8978ede5677` — Manual ordinary herb cleaning; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0002 r1** `a5e21e03c483` — Unfinished potions: ordinary water and three alternate bases; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0003 r1** `f3b4b984af8f` — Initial twenty-nine finished-potion profiles; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0004 r1** `613b29ac4e3d` — Herb tar batch production; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0005 r1** `87c4a3def092` — Seven ordinary tool-preserving grinding routes; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0006 r1** `e7ac3608e618` — Ordinary chemistry charges and Options/Break controls; V0 PASS, V1 IN_PROGRESS, V2 FAIL, V3 IN_PROGRESS, V4 IN_PROGRESS.
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
- **OR-MIG-0024 r3** `ae3d234c7176` — Ordinary selected-slot food transaction and potion-overheal integration safety; V0 PASS, V1 IN_PROGRESS, V2 IN_PROGRESS, V3 IN_PROGRESS, V4 NOT_STARTED.
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
- **OR-MIG-0036 r1** `8065a96c23c1` — Manual chicken beef and shrimp cooking on a generation-validated ordinary fire; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0037 r2** `864d439132b3` — Nine ordinary manual fire-fish recipes with corrected Wiki burnt variants; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0038 r1** `cb8975bc8fd6` — Ordinary stew both preparation orders manual fire cooking and retained-bowl consumption; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0039 r1** `f79a54671dfd` — Manual chocolate bar and dust cake topping with live level and XP; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0040 r1** `d63c0258dd69` — Shared captured multi-input assembly and four-ingredient cake batter with container returns; V0 IN_PROGRESS, V1 IN_PROGRESS, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0041 r2** `864d439132b3` — Ordinary range and bread baking with corrected inherited burnt-fish variants; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0042 r1** `052d24c0b0d8` — Manual ordinary range-only redberry meat apple pie and plain pizza baking; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0043 r1** `8805474ede2c` — Eight member range pies with live cooking membership and optional forced burn cutoffs; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0044 r1** `30509633a6d1` — Nine member fish with method-specific cooking and fractional base XP; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0045 r1** `461424d9c7f1` — Bear, rat and rabbit manual cooking and cooked-rabbit consumption; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0046 r1** `138d95d184e6` — Cave- and lava-eel manual production with no-burn lava guarantee; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0047 r1** `3f49323fa71a` — Shared atomic random healing and eight ordinary foods with skewer return; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0048 r1** `3a57d318bd42` — Fourteen ordinary manual meal, quest-item and holiday foods; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0049 r1** `8d8dff9c3f52` — Thirteen retained-bowl foods and containerless egg potato; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0050 r1** `78f603fd516a` — Six Wiki-corrected Dorgesh-Kaan foods with two Cooking boosts; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0051 r1** `f6b0f6ad54fe` — Selected-slot and atomic actual network Prayer burial; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0052 r1** `8389805afedc` — Five demonic ash Scatter profiles through selected Prayer transactions; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
- **OR-MIG-0053 r1** `0d9a03c8b97c` — Seven absent ordinary Bury reward profiles with configured-policy preservation; V0 NOT_STARTED, V1 NOT_STARTED, V2 NOT_STARTED, V3 NOT_STARTED, V4 NOT_STARTED.
