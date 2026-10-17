# Cinderworks Floor One — current development status

Standing mandate: complete a comprehensive, polished Direction B floor. Overall acceptance remains in progress; individual milestones are continuation checkpoints.

## Ownership and baseline

- Server baseline: `6a1ea78022bd1c9cbc7e259bcb3595652e5eaae5`.
- Own published branch: `agent5/tower-development`; last verified remote checkpoint `0e2c54c7afd8ac1b097bf973dfce44a786bf0f82`.
- Client baseline: `95eed88159b06cebea24559a814a5111b54a59cd`; isolated validation branch now `6545790`. The source-only six-commit patch is supplied here; the client repository is never pushed.
- Last observed Agent 2 ref: `924e929438c952d994287fad6f0c0bce749438ce`, not merged. No other workspace, process or branch is modified.

## Implemented and demonstrated

The Cinderworks is an open sanctuary with a workshop, waterworks kitchen, creature yards, altar, grove, Ash Imp gallery and northern Ash Warden foundry. Equal-height cache-authored materials distinguish moss, earth, slate and ash courts. The river retains actual bank/water clipping; the outer shell stays closed. The ground stair into the old upstairs stub is removed.

Normal first-login appearance and experience choices lead to the authored introduction. The plaque starts a run and atomically supplies missing tools. Ordinary mining, smelting, dagger smithing, wielding, net fishing, item-on-range cooking, creature combat, bones pickup and item-on-altar offering restore three services. A readable six-step task journal gives concise next actions. Legacy diary state remains stored separately, while obsolete movement cinders and automatic kit restoration are excluded from this route.

The Imp escalates to three-tile warnings. The Warden has three health phases, visible timed smoke/eruption patterns, a closed shell during charges and routed coolant interruption/exposure. Ownership, prerequisites, equipment, authoritative hero credit, departure reset, damage and ordinary food healing are integrated into the existing simulation. Defeat animates and returns the player to the sanctuary without resetting equipment, services or pending rewards; consumed supplies remain spent.

Guardian victories and reward proposals use the existing asynchronous account worker and SQLite transaction. Rewards remain unspendable until inventory plus paid marker are committed. Logout waits for the checkpoint; failures retain the proposal and retry. Unique request tokens reject stale duplicate completions. First clear pays 500 coins plus a beer; a replay pays 25 plus a beer. Claim, exit, return and new-run rules preserve first-clear history.

Genuine native gameplay has completed all services, both guardians, all three Warden phases, full-pack chest denial/recovery, one first-clear payment and duplicate-search rejection. A fresh headless character independently completed all services, actual lethal vents and sanctuary recovery, Imp victory, deliberate server crash/relogin, three-phase Warden victory, first-clear claim and duplicate-search rejection. Both the native paid reward and fresh Imp unlock survived a deliberate crash before logout. Strict exit/return now preserves inventory, equipment, XP and the completed journal exactly.

The final native layout tour reached all ten review points with real normal movement and framebuffer captures. The native character has started run two through the plaque, retaining paid history while resetting services and boss credit. Current work runs a clean first-login headless route and the native replay through the public controllers.

## Validation and remaining work

Latest world suite: 400 passed, two existing ignored. Server suite: 52 passed. Storage checkpoint suite: eight passed, one existing ignored. Server and optimized native/headless builds pass. Actual action receipts, structured outcomes, process identities, read-only database checks and native frames are private; no protected resource or binary is published.

Remaining overall criteria: complete the final clean scripted route; complete and durably verify the replay payment; recheck relog with the latest adapter; consolidate final acceptance and visual evidence, then publish the completed source and limitations. Historical chronology is in CHECKPOINTS.md; the acceptance matrix is authoritative for completion.

Final clean character: normal onboarding, prerequisite denial, all services and actual lethal-vent recovery pass in single scenario invocations. The native replay completed its services but one Imp attempt exhausted food and ended in sanctuary recovery; no victory was credited. Four-tick warnings and safe adjacent Imp attacks are being validated in both modes, with supplies replenished only through normal skills.

The clean headless character has now defeated the four-tick Imp at full health without spending food. Its Warden and the native replay Imp are running in separate courts. Latest native and headless validation binaries both use adapter `6545790`; the introduction configuration-order correction is active before admission.
