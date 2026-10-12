# Gameplay telemetry contract v1

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
- `player.position`: known position log format with a real tick if provided. Coverage is partial; other prose formats are not silently interpreted.
- `client.dispatch`: component/child and attempt observation. Server acceptance stays null.
- `action.effect`: recognized walk, equip/remove and banking confirmation markers. These are **client-observed effects**, not server decision records. Action parameters are included only when logged.
- `interface.hidden`: logged widget visibility evidence, not all interface transitions.
- `packet`: incoming game opcode, payload length, ordinal and direction from the bounded R239CAP1 capture. Full payloads, handshakes and session keys are not exported. Outgoing opcode/dispatch coverage remains incomplete.
- `session.end`: actual client/server exit codes, forced flag and server-exited observation, with the recorded shutdown timestamp.
- Contract kinds `npc.state`, `object.state`, and `server.decision` are comparable but **not yet emitted by this adapter**. NPC/object transitions, full dialogue transitions, per-action server rejection reasons, complete widget updates and error detail remain coverage gaps. Raw logs and separate Tower scene assertions remain supporting evidence. No absent observation is treated as successful acceptance.

## Bounds and privacy

Writer creates a new file exclusively, appends one validated record at a time and flushes. No asynchronous queue grows unbounded. Records are limited to 64 KiB, strings to 16 KiB, captures/comparison inputs to 100,000 events, packet payload reads to 65,535 bytes. Comparison buffers bounded event lists; it is not a streaming unlimited analytics store.

`--detail checkpoints` omits packet metadata; `--detail packets` includes available packet metadata and is the default. Neither mode dumps a full world each tick. The adapter only runs after a scenario, minimizing gameplay overhead.

Known sensitive key names and private-key markers are rejected recursively. Adapters whitelist source fields. This is defense in depth, not a guarantee for arbitrary caller-supplied free text. Raw client/server logs, captures, databases and local keys stay excluded from Git; manually inspect any evidence selected for sharing.

## Normalization

Ignore absolute wall-clock values, run/event IDs, environment label, source build identity and explicitly ephemeral session identity only after checking scenario/baseline/client/fixture/cache/revision compatibility. Never normalize inventory quantities, item IDs, coordinates, stats/XP, varps, interface visibility, action order or logout results. Preserve deterministic relative action tick offsets when all are known; otherwise timing is NOT TESTED. No gameplay randomness is currently permitted for normalization.
