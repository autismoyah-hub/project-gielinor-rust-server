# Cinderworks Floor One — functional acceptance; visual production pass required

Production redesign mandate (2026-10-16): the user rejected the current environmental and artistic quality. Earlier comprehensive completion declarations were premature. Gameplay evidence below remains a regression baseline; visual production acceptance and release-candidate acceptance are **not granted**. Final artistic acceptance belongs to the user.

Pinned pre-redesign baseline: server `34bb31d1b5a48449b3c960951ec43f93b4f17fd9`; client validation `a32b66deeeb30a2be159fbd379a9ffb07f423b72`.

## Current production pass

Architecture is reconstructed with real masonry edges and collision, a clipped-corner sanctuary, southern hall, enclosed service wings and fortified foundry. Exact JS5-only conveyor-frame masks remove the source scenery missing from the paired game map. Plaque and portal now use real supported scene shapes; the inside portal is integrated at (10432,124). Original obsolete staff/vial/mind-altar/goat/goblin/upper-stub content is removed.

The Warden is visually transformed to a spiked fire giant with compatible cache sequences while retaining authoritative type 16295 and its original combat/XP metadata. Charge, phase and coolant cues use standard native animation/spot packets. Four visibly searchable rainwater barrels sit at (10421,135), (10421,138), (10428,135) and (10428,138), outside the central escape lanes. The controller excludes blocked diagonal corners and evades before seeking coolant; warnings and damage are unchanged. The Imp gallery uses lighter slate.

Against published server `164b01c9dcf5306d08dd1044f2b03ad9ded4dc64`, native Warden passes all three phases at 13 HP with 19 food retained. Native and headless first-clear claims pass full-pack atomicity, exactly 500 coins/beer, duplicate denial and exact exit/return preservation. Headless Warden additionally records a new coolant acknowledgement. All four affected library suites pass: cache 24, protocol 20, server 52, world 401 (two existing ignores). The 48-view cardinal gallery is captured and reviewed. A fresh exact-source headless playthrough and native replay/lifecycle regression continue; no artistic or release-candidate acceptance is claimed.

Read-only native scene provenance is local client commit `8276b5737f8e3beed17eba28eeb50e6ed051135d`. The nine raw-byte patches reproduce exact tree `740e2a56e84c30eb1ef79aa3e2c438c4066baf1b` from the pinned baseline; apply with `git am --keep-cr`.

## Standing continuation: The Fallen Cinderworks

The latest user mandate explicitly rejects the sparse reviewed reconstruction as artistically incomplete. Adopt an ancient dwarven forge fortress overtaken by supernatural ash: stronger sanctuary composition, structural rhythm in principal corridors, believable workshop/waterworks, staged boss chambers and visible restoration derived from authoritative progress. The 164b gallery is the next before-reference, not an accepted endpoint. Preserve exact source-frame masks, supported shapes, low foreground walls, compatible Warden sequences, real Search coolant and transactional rewards.

Next: finish outstanding exact-164b native/headless regressions, then reconstruct major compositions and implement selected progression-driven environmental changes. Every changed build requires its own native, headless, rebuild, relog, replay and durability evidence. User artistic acceptance remains outstanding. The earlier gallery approval request is superseded by this concrete new direction.

## Historical functional acceptance record

The standing comprehensive Floor One mandate is satisfied on the isolated `agent5/tower-development` branch. The native Rust window remains connected at the restored sanctuary with the completed six-step journal. [TOWER_ACCEPTANCE.md](TOWER_ACCEPTANCE.md) is the final acceptance report; [VALIDATION_RESULTS.json](VALIDATION_RESULTS.json) records safe aggregate outcomes.

Implemented: coherent open courts and river boundary; normal onboarding and plaque/recovery tools; ordinary workshop, kitchen and altar progression; escalating Ash Imp and three-phase Ash Warden with readable hazards and coolant; real defeat recovery; durable completion and account-transaction reward barriers; full-pack recovery; normal exit/return; safe replay preserving first-clear history.

Verified native and clean headless gameplay, actual lethal vents, deliberate server crash/relogin, exact exit/return state preservation and final restart/relogin. Native two-clear balance is 525 coins/two beers; clean first-clear balance is 500/one beer. Repeated chest searches grant nothing. The native replay's final guardians used no food; failed earlier diagnostics remain recorded separately.

Final world tests: 400 passed/two existing ignored. Server tests: 52 passed. Optimized native/headless builds and the forced Windows snapshot-sharing regression pass. Client validation source is local commit `a32b66deeeb30a2be159fbd379a9ffb07f423b72` atop baseline `95eed88159b06cebea24559a814a5111b54a59cd`, supplied as a source-only seven-commit patch. The client repository is not pushed.

Gameplay implementation is tested at `420729798ffc3b0d31c7aff7497d1841597c2104`, based on server engine `6a1ea78022bd1c9cbc7e259bcb3595652e5eaae5`. Agent 2 ref `521e2e047071a7e7d481ab458829bb88a57b76e2` was fetched read-only and reviewed, not merged. Its V12 chemistry migration requires migration-order reconciliation with Tower V12 during Agent 2's eventual integration. This is a handoff constraint, not an unimplemented Floor One feature. Other agents' branches, workspaces and processes remain untouched.

Final publication contains authored source, reusable scenario controllers, safe metadata and integration documentation only. Protected resources and complete private evidence remain local. Historical milestones and failed/repaired attempts are retained in CHECKPOINTS.md and the private evidence index.

### Current readability and interaction refinement

Foreground southern room edges use supported low stone wall 5154 through `prepare_palette`; exterior, back walls and plaque support remain full-height 2341. Movement stays blocked, while range policy follows the actual cache asset. Native comparison at the foundry threshold shows the avatar remains visible. No renderer or input-readiness bypass is used.

Coolant uses stock rainwater barrel 30124 (`mistmyst_barrel_water`), whose genuine native cache has Search operation 1. Storage barrel 362 has no native Search action and is no longer a coolant fixture. Four coolant corners (10421,135), (10421,138), (10428,135), (10428,138) keep central cardinal/diagonal escapes clear. Obsolete lamps/stump inside hazard fields are removed or moved to perimeter recesses.

The shield eligibility rule is preserved, but automatic melee retries no longer flood chat with identical shell-denial messages. Initial instructions and phase/coolant cues remain. The first visible warning explains heat; subsequent waves use the actual animated telegraphs. The displayed aggressive dagger style (Combat Options → Lunge) trains Strength through ordinary combat, with no injected levels or XP.
