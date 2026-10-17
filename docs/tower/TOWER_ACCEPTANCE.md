# Floor One acceptance — functional baseline; visual acceptance pending

Production redesign mandate (2026-10-16): the user rejected the current environmental and artistic quality. Earlier comprehensive completion declarations were premature. Gameplay evidence below remains a regression baseline; visual production acceptance and release-candidate acceptance are **not granted**. Final artistic acceptance belongs to the user.

Pinned pre-redesign baseline: server `34bb31d1b5a48449b3c960951ec43f93b4f17fd9`; client validation `a32b66deeeb30a2be159fbd379a9ffb07f423b72`.

## Historical functional acceptance record

The authored Cinderworks Floor One is complete and accepted on the isolated Tower branch. Gameplay engine source is `420729798ffc3b0d31c7aff7497d1841597c2104`; final validation uses the pinned native client plus local adapter `a32b66deeeb30a2be159fbd379a9ffb07f423b72`. This report and the added diagnostic controllers do not change that tested server implementation.

| Criterion | Confirmed result |
|---|---|
| Normal onboarding | Real appearance and experience choices; authored pre-run introduction, empty inventory and no implicit run before the plaque |
| Layout and navigation | Real native movement through all ten review points; distinct workshop, moss, slate and ash courts, preserved river, closed exterior shell and no ground stair to the old upstairs stub |
| Ordinary progression | Mining, smelting, dagger smithing/wielding, net fishing, item-on-range cooking, creature combat, bones pickup and item-on-altar offering; all three services credited by successful callbacks |
| Rejection and capacity | Pre-service guardian denial with no combat XP; full-pack fishing/pickup handling and normal recovery; full-pack chest pays nothing until space is made |
| Ash Imp | Native and clean headless victories, visible phase escalation and real movement/combat; final native replay and clean headless attempts used no food |
| Ash Warden | Native and clean headless victories through all three phases, real smoke/fire, damage, food rules and routed coolant interruption |
| Defeat and interruption | Actual lethal vents reach zero HP, then sanctuary recovery preserves items/equipment/services without an unlock; native failed attempt also recovered; disconnect releases transient court ownership |
| Transactional rewards | Proposed inventory and paid marker use the existing account transaction, with gameplay/logout barrier until ACK; first clear is 500 coins/beer, replay is 25/beer; repeated searches add nothing |
| Crash durability | Deliberately killed only the owned server before logout; committed native first reward and fresh Imp unlock survived real relogin |
| Exit and return | Native replay and clean headless completion preserve exact inventory, equipment, XP and journal through normal portals |
| Replay | Normal plaque begins run two, retains paid history, resets only objectives; second full clear pays 25 rather than 500 |
| Final restart/relogin | Native has run two, two victories, 525 coins and two beers; clean headless has run one, one victory, 500 coins and one beer; duplicate searches still grant zero |
| Native rendering | Genuine Windows Rust client and composited framebuffer captures, no desktop input or focus activation; completed six-step journal remains visibly open |
| Gateway reliability | Forced two-second Windows snapshot-sharing violation delays ACK by 2.09 seconds; live client survives normal weapon swap and restore, with no gameplay redispatch; large diagnostics are opt-in |
| Client patch reproducibility | All seven source patches apply from the pinned baseline in a separate index and reproduce the exact tested client tree `d623dddb94ba47d16c08d73fb37d6dc68526612a`; preserve CR bytes with `git am --keep-cr` |
| Automated/headless evidence | Reusable normal-action controllers and structured assertions; headless observations are never described as native frames |

The clean service route earns 93 smithing XP from its bar and dagger and no obsolete movement agility XP. Final world suite: **400 passed, two existing ignored**. Final server suite: **52 passed**. Server and optimized native/headless builds pass. Earlier storage validation passed eight tests with one existing ignored; persistence is additionally checked through actual account transactions, crash, restart and relogin. [VALIDATION_RESULTS.json](VALIDATION_RESULTS.json) contains safe aggregate outcomes; detailed session state and frames remain private.

Failed diagnostics remain failed: the early Warden retreat, controller capacity/widget timing errors, a food-exhausted native replay Imp, and a native Warden session interrupted by a snapshot sharing violation are not counted as wins. Subsequent named attempts independently passed after their fixes. No save edits, injected items, developer gameplay shortcuts, physical input emulation or generated gameplay images were used.

## Integration boundary and existing limitations

Agent 2's latest observed ref is `521e2e047071a7e7d481ab458829bb88a57b76e2`; it is not merged. Its V12 chemistry migration collides in version number with this branch's V12 Tower migration. Agent 2 must append/renumber the Tower migration and preserve both state projections during integration. Shared runtime/player/queue changes also require review; acceptance against that newer branch is not claimed. See [TOWER_INTEGRATION_CONTRACT.md](TOWER_INTEGRATION_CONTRACT.md).

Three historical prototype differential expected files (tower-bones, tower-chest, tower-kit) were absent. They were not fabricated or reported as passing; Direction B intentionally redesigns that prototype and is accepted through its own policy, integration and authentic client scenarios. The generic combat-catalog loader retains its documented missing-source warning; configured Tower guardians and ordinary creature combat were demonstrated. Whole-client/server parity, other floors, per-party instances and cooperative boss credit are outside this Floor One acceptance. Private caches, binaries, keys, credentials, databases, raw sessions and captures are excluded from publication.
