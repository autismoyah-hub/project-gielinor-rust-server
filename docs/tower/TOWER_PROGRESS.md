# Tower Floor One development

Standing mandate: deliver a comprehensive, polished, playable Direction B Floor One. Milestones are continuation checkpoints, not completion.

## Baseline and publication

- Initial engine baseline: `6a1ea78022bd1c9cbc7e259bcb3595652e5eaae5`.
- Development branch: `agent5/tower-development`.
- Native client baseline: `95eed88159b06cebea24559a814a5111b54a59cd`.
- Latest observed Agent 2 ref: `924e929438c952d994287fad6f0c0bce749438ce`; not merged.
- Checkpoint history: [CHECKPOINTS.md](CHECKPOINTS.md).

## Initial verified foundation

82 isolated tests passed: 62 Tower world tests, nine available differential targets, two map-publication tests, eight storage tests and one native hashing test. Server offline build, client library check, cache/content validation and bounded loopback startup/shutdown succeeded. Private resources and detailed local evidence remain outside publication.

Native rendered acceptance has not yet been established. No complete player-driven Floor One route exists at this baseline: objective callbacks, boss credit, cinders and special transitions require integration. Final-chest partial payout must be corrected. Three historical differential fixtures are unavailable; no replacement expected data will be fabricated.

## Active work

1. Publish this branch and verify the remote SHA.
2. Authenticate the isolated native Rust client, render the existing Tower, and capture real baseline evidence.
3. Author the full floor identity, layout and dependency-aware gameplay plan.
4. Implement layout, ordinary progression, demi-boss, final boss, completion/replay and polish, validating both authoritative simulation and rendered interaction throughout.

Only the comprehensive criteria in [TOWER_ACCEPTANCE.md](TOWER_ACCEPTANCE.md) constitute overall completion.

## Programmatic native validation checkpoint

Published starting SHA verified: `2d8e1634683ceb0614fa7c01fa88e26d36089409`. The isolated native client authenticated to the local server and rendered the existing Tower, receiving the real arrival messages, NPCs and location changes. A local opt-in adapter uses `ClientState::menu_action`, with no desktop input injection and a window configured not to activate. Initial walk requests have not yet produced confirmed movement; this is under investigation and is not accepted gameplay evidence. Baseline frame also exposes camera/UI readability problems. Client validation changes remain on a separate local branch against pinned client baseline.

Reward transaction correction is implemented and all 62 Tower world tests pass. Full floor acceptance remains outstanding. Next: prove native packet-driven movement, retain frame/state receipts, then implement the authored floor layout and route.

## Native control and Cinderworks foundation

Native programmatic movement is confirmed in a live session, including after normal appearance confirmation and experience selection: `[0,10424,120]` to `[0,10425,120]`, server tick 529, with no desktop input or activation. Headless movement also confirmed independently. Frames and state receipts are retained privately. Login interface setup is slow; submission acknowledgements alone are not accepted as outcomes.

The authored Cinderworks design is recorded in TOWER_DIRECTION_B.md. The first layout revision removes internal nine-room wall partitions while retaining the exterior boundary and river. Actual native frames show the open service court and visible player. This is an early layout revision, not final visual acceptance.

Implemented foundation: durable run lifecycle and migration V12, alternate Tower onboarding after normal character choices, plaque run start/recovery tools, progress-preserving exit/return, atomic first/replay reward claim, and service credit at successful smithing/cooking/offering callbacks. Boss victory transitions exist as lifecycle functions but are not yet integrated into live encounters. No complete floor route is claimed.

Validation so far: 64 Tower world tests, three routed Cinderworks action tests, eight storage tests (one native test excluded by its existing ignored flag), cache shell test, two layout tests, server check/build and isolated native/headless sessions. A schema-upgrade fixture initially failed because it retained the newly added V12 table; the fixture now reconstructs the preceding schema correctly and passes.

Next: validate the configured live plaque/portal route, integrate scheduled visible vents and encounter ownership, implement the Imp and phased Warden, then complete authentic success/failure/recovery/replay sessions and polish.

Checkpoint validation broadened: full world library suite passes (389 passed, two existing ignored); full server library suite passes (52 passed); storage suite passes (eight passed, one existing ignored). Enclosed tile-graphic wire test passes. Live native plaque interaction routed from the court, began run 1, and delivered the recovery tools and next-step messages. Live native portal interaction reached normal-world `[0,3224,3218]`; return is being checked. This is continued development, not overall acceptance.
