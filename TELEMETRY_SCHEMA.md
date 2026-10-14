## Client invocation diagnostics

Optional `client.invocation` records the launcher's UTC invocation start/end and elapsed milliseconds from `client-invocation.json`. Its timestamp is the observed invocation start. It includes asset loading and is not a socket-connect timestamp, login duration, or game tick. `transport_connect_utc` and `game_tick` stay null. It is diagnostic metadata outside gameplay checkpoint comparison; persisted varps/playtime differences remain strictly compared. Historical runs without the file receive no invented timing event.

Bones-offered state is now included in diary evidence. Kotlin reads the canonical bones flag; Rust reads V11 nullable bones_offered. NULL or a pre-V11 schema is UNKNOWN and blocks parity even when all diary counters match. Explicit known-empty fixtures initialize false; existing saves remain untouched.

## Tower diary scenario observations

Exact driver assertions map to `action.effect` actions `tower.tanning`, `tower.chest_paid` and `tower.chest_repeat`. Parameters preserve observed hides/leather/axe totals, unchanged Crafting XP, bounty message and repeated-empty result. `client_dispatch_result` is `effect_observed`; `server_acceptance` remains null. This records client inventory/chat observations, not an instrumented server decision or graphical input acceptance. Known/unknown initial and persisted diary records retain the existing strict comparison rules. New adapter regression passes; live A/B remains pending.

# Gameplay telemetry contract v1

Resumed development adds `persistence.completion` (observed new offline logout
save or unavailable), `action.target` (NPC definition), `client.continue`
(handled flag and emitted opcodes), and `dialogue.response` (observed text).
Comparison now requires completed logout observations on both sides; historical
reports without them are retained but cannot establish current persistence
parity. Response text remains unnormalized: GenericPerson has random branches,
so a different response requires attribution and is not automatically a defect.

Implementation: `scripts/integration_telemetry.py`; tests: `scripts/test_integration_telemetry.py`. This shared contract currently has an offline evidence adapter for both environments. It is not comprehensive live server instrumentation.

Each line is one JSON object with exactly these envelope keys:
`schema_version`, `event_id`, `run_id`, `scenario_id`, `environment`, `source`, `timestamp`, `game_tick`, `actor_id`, `kind`, `observation`, `data`, `evidence`.

Example records (illustrative, not claimed test evidence):

```jsonl
{"schema_version":1,"event_id":"example:000001","run_id":"example","scenario_id":"actions","environment":"rust","source":{"implementation":"evidence-adapter","version":"sha256-of-adapter"},"timestamp":null,"game_tick":null,"actor_id":"player:fixture","kind":"action.effect","observation":"observed","data":{"action":"walk","parameters":{"position":[3225,3218]},"client_dispatch_result":"effect_observed","server_acceptance":null},"evidence":{"file":"client.log","line":6}}
{"schema_version":1,"event_id":"example:000002","run_id":"example","scenario_id":"actions","environment":"rust","source":{"implementation":"evidence-adapter","version":"sha256-of-adapter"},"timestamp":null,"game_tick":null,"actor_id":"player:fixture","kind":"server.decision","observation":"unavailable","data":{"action":"walk","accepted":null,"reason":"server decision hook not instrumented"},"evidence":{"file":"client.log","line":6}}
```

`observed` means directly present in named evidence, `inferred` is a separately labeled analysis, and `unavailable` means unknown. Missing timestamps/ticks remain null; they are never fabricated. IDs use run plus monotonically increasing sequence. The actor is a disposable fixture pseudonym. `source.version` hashes the adapter; context includes the actual client executable hash, baseline, revision, cache and fixture identity. Rust launch evidence includes server binary hash; Kotlin launch/overlay/resource manifests identify its runtime.

## Current observations and limits

- `context`: baseline, client hash, server hash when recorded, fixture/cache, input mode and revision.
- `state.initial`, `state.persisted`: SQLite checkpoint position/plane, varps, energy, XP rate, all stats with visible/base levels and fine XP, inventories with container/slot/item/count/vars. Equipment and bank are preserved as their actual container types. Account credentials and session metadata are excluded.
- `tower.initial`, `tower.persisted`: narrow arrival projection (`kit_given`, `staff_taken`, `diary_progress`) from Kotlin's disposable-account sidecars or Rust's character-owned table. Absent rows in an owned pre-V9 Rust fixture use the migration default; this is not evidence that an older executable implements the hooks. Full diary counts, restoration effects and crash timing are outside this projection. Existing runs without these files retain their original coverage.
- `player.position`: known position log format with a real tick if provided. Coverage is partial; other prose formats are not silently interpreted.
- `client.dispatch`: component/child, operation when logged, and outgoing opcode list when the owned driver observes it. `packet_queued` means a native packet was enqueued; older logs retain `attempt_logged` and null opcode coverage. Server acceptance stays null.
- `action.effect`: recognized walk, equip/remove and banking confirmation markers. These are **client-observed effects**, not server decision records. Action parameters are included only when logged.
- Arrival effects use `tower.first_arrival` with observed destination and `tower.first_kit_clear` with containers 93/94. An absent client container is not accepted as an empty container by the scenario driver.
- `inventory.restore` requires the full native assertion: sword1277 in worn container94 slot3,1000 coins,two lobsters and appearance equipment value3325. This is appearance-update state, not rendered-pixel verification. A partial log prefix does not create restore coverage.
- `interface.hidden`: logged widget visibility evidence, not all interface transitions.
- `prayer.dispatch` and `prayer.continue`: native outgoing opcode observations with null server acceptance. `prayer.client_state` records the observed4104 bit, which may be predicted by client scripts. `prayer.requirement_notice` and `prayer.notice_closed` are separate `action.effect` observations. The strict comparison includes all three prayer event kinds and preserves notice text; this does not establish server acceptance of activation or rendered appearance.
- `prayer.server_mirror`: the owned driver observed bit4104 and confirmed its entire base varp equals the pinned client's separate server mirror via `server_differences()`. This strengthens toggle-state evidence without changing the client library. It does not expose a server decision reason or verify rendered pixels; server_acceptance remains null. Mirror observations participate in strict comparison and require matching driver hashes.
- `packet`: incoming game opcode, payload length, ordinal and direction from the bounded R239CAP1 capture. Full payloads, handshakes and session keys are not exported. Outgoing opcode/dispatch coverage remains incomplete.
- `session.end`: actual client/server exit codes, forced flag and server-exited observation, with the recorded shutdown timestamp.
- Contract kinds `npc.state`, `object.state`, and `server.decision` are comparable but **not yet emitted by this adapter**. NPC/object transitions, full dialogue transitions, per-action server rejection reasons, complete widget updates and error detail remain coverage gaps. Raw logs and separate Tower scene assertions remain supporting evidence. No absent observation is treated as successful acceptance.

## Bounds and privacy

Writer creates a new file exclusively, appends one validated record at a time and flushes. No asynchronous queue grows unbounded. Records are limited to 64 KiB, strings to 16 KiB, captures/comparison inputs to 100,000 events, packet payload reads to 65,535 bytes. Comparison buffers bounded event lists; it is not a streaming unlimited analytics store.

`--detail checkpoints` omits packet metadata; `--detail packets` includes available packet metadata and is the default. Neither mode dumps a full world each tick. The adapter only runs after a scenario, minimizing gameplay overhead.

Known sensitive key names and private-key markers are rejected recursively. Adapters whitelist source fields. This is defense in depth, not a guarantee for arbitrary caller-supplied free text. Raw client/server logs, captures, databases and local keys stay excluded from Git; manually inspect any evidence selected for sharing.

## Normalization

Ignore absolute wall-clock values, run/event IDs, environment label, source build identity and explicitly ephemeral session identity only after checking scenario/baseline/client/fixture/cache/revision compatibility. Never normalize inventory quantities, item IDs, coordinates, stats/XP, varps, interface visibility, action order or logout results. Preserve deterministic relative action tick offsets when all are known; otherwise timing is NOT TESTED. No gameplay randomness is currently permitted for normalization.

## Diary persistence observations

New runs additionally record `diary.initial` and `diary.persisted` from `before-diary-state.json` and `diary-state.json`. These include six counts, seven action flags, eleven paid flags, pending bosses, bounty, schema and imp-bones state. The stage is persisted fields before session synchronization. Skill levels and staff ownership remain separate authoritative inputs. Kotlin file decoding follows restoreDiaryCounts on the canonical post-migration character directory; legacy global-file migration and gameplay side effects are not represented. Rust absent rows/tables are UNKNOWN. Any included unknown diary record blocks strict parity; two unknowns are not equivalent empty states. Historical runs without these records retain absent coverage. Existing arrival projection is unchanged.
