# Reference baseline and compatibility plan

## Provenance and isolation

Baseline: local Project Gielinor commit `cb6e66cc0f16a25b60ba6705d8f157cc22ab4869`, inspected on October 11, 2026 (America/Los_Angeles). This is the committed local baseline, not a claim that it matches pristine upstream RS Mod. The working checkout contains many local gameplay and engine changes; `work/reference-local-changes.txt` records their paths. These need a separate future parity track after the committed baseline is verified.

Agent 1's active workspace is `C:\Projects\OSRSRSPS\rust-client`. Only its top-level inventory, Cargo manifest, and build script were inspected. Its source and running jobs were not modified. The initial process snapshot showed Java PID 8428, started October 2, listening on 43597 and 43598. Treat it as an external live workload. No connection was made to it. No client source scan, cleanup, or process control was performed.

Agent 2 owns only this server directory. `work/target`, `work/cargo`, `work/gradle`, logs, fixture outputs, and the source-only reference snapshot are isolated. No production `.data` directory, account, private key, database, or game cache was copied. The shared cached Gradle artifacts and compiler binaries are read-only inputs; Gradle itself runs from an independent distribution copy in `work/gradle-dist`.

## Build and service dependencies

The reference declares Java 21, Gradle 9.0.0, Kotlin 2.2.0, Netty 4.2.2.Final, RSProt `1.0.0-ALPHA-20260630` (`osrs-239-api` and `osrs-239-shared`), OpenRS2 `0.1.0-SNAPSHOT`, Guice 7.0.0, Jackson 2.19.1, coroutines 1.10.2, SQLite JDBC 3.50.1.0, Flyway 9.22.3, Argon2 JVM 2.12, and Bouncy Castle 1.81. Versions come from the pinned `gradle/libs.versions.toml`, not current online package metadata.

SQLite uses `.data/saves/game.db` by default (`api/db/.../DatabaseConfig.kt`). `SqliteService` creates parent directories and applies Flyway migrations during startup. This requires an isolated working directory before startup. No separate database service is needed for this default configuration.

`server/shared/.../DirectoryConstants.kt` uses relative `.data` paths. `CacheStoreModule` opens OpenRS2 stores and copies the vanilla cache when derived stores are absent or empty. A symlink to a production cache is therefore not an acceptable isolation mechanism. Before full-server execution, provide a verified read-only cache adapter and disable packing/install tasks, or establish a safe isolated overlay. Do not download or duplicate the large cache casually.

The pinned `NetworkFactory` hardcodes port **43594**, which conflicts with the parent workspace rule. Never run that snapshot unchanged. A future reference-only runtime overlay must bind loopback on an unused dedicated port (proposed 43610), use disposable persistence and test-only keys, and preserve the original source snapshot for comparison. Port availability must be checked at launch time. The current working Kotlin checkout's configurable port is a local modification and is not silently assumed in the committed baseline.

## Architecture and behavior

`engine` provides entities, coordinates, routing, collision, events, coroutines, plugin and module abstractions. `api` assembles account management, cache codecs/enrichment, registries, network adapters, game processing, inventory transactions, combat, skills, and persistence. `content` supplies scripts and configuration; `server` supplies installation, dependency injection, logging, services, and application startup.

The baseline content directories cover areas, generic interactions, interfaces, travel, and skills including cooking, crafting, fishing, fletching, herblore, hunter, magic, mining, smithing, and woodcutting. Other modules include login, commands, shops, tower, skill campus, and special attacks/weapons. Directory presence does not establish completeness. The working tree includes additional skill modules and changes outside the pinned baseline.

`GameService` runs a single game executor at 600 ms intervals. It carries sub-millisecond elapsed time into the next calculation; overruns wait until the next interval boundary rather than executing an unbounded burst of catch-up ticks. Deterministic Rust tests should advance logical time directly and test scheduling separately from simulation.

`GameCycle` order is StartCycle event; world main; NPC pre-tick; player shuffle; player input; route requests; NPC main; controller main; player main; player logout; player login; map-clock increment; LateCycle event; world post-tick; player post-tick; EndCycle event. The working tree adds autosave after player post-tick; this is a separate delta. `PlayerInputProcess` updates each player's map clock and previous coordinates before reading input, and disconnects on processing errors.

RSProt owns wire framing, encryption and revision codecs. `MessageConsumerProvider` connects incoming movement, interface buttons, NPC/object/location/player operations, chat, dialogs, and client state to handlers. `ConnectionHandler` checks loader availability and JS5 CRCs before password authentication, account load/create, Argon2 verification, and TOTP. Token authentication returns InvalidLoginPacket and reconnect returns ConnectFail in this baseline; those are known reference limitations, not features to invent.

`RspCycle` initializes with RebuildLoginV2 and maintains coordinate, movement, appearance, animation, hits, and build-area state. Flush queues active-world context, NPC origin, world-entity info, player info, optional NPC info, and active-world context again. Protocol compatibility cannot be inferred from a successful login handshake alone.

## Incremental Rust boundaries

Implemented: `gielinor-world` contains pure coordinate primitives with no IO, wall clock, or global randomness. Raw signed packed values preserve Kotlin's NULL sentinel; coordinate construction rejects invalid axes; translations preserve JVM integer wrap semantics before validation. Chebyshev distance ignores level, matching the reference. `gielinor-protocol` contains ISAAC-32, verified against the exact cached RSProt crypto dependency for five seeds and 1,025 outputs per seed. This covers reverse output order, unsigned shifts, wrapping arithmetic, and repeated refill boundaries. It does not yet establish opcode tables, framing, session seeding rules, or complete encryption parity.

Next boundaries, added only with concrete behavior and fixtures: protocol (bounded codec errors, RSA/XTEA/ISAAC and exact 239.1 framing); sessions (authentication and authoritative player lifecycle); world (deterministic cycle phases, injected randomness, collision/routes and synchronization); gameplay (inventory transactions, equipment/bank, combat/skills/content); persistence (isolated account repository); diagnostics/testing (headless input, snapshots and reset confined to tests).

AI actions must use the same validated command path as players, with actor identity and versioned observation/action schemas. Keep observations read-only. Do not expose direct state mutation or network-accessible reset as production features. No external AI integration has been added.

## Compatibility gates

1. Complete isolated Kotlin dependency resolution, then full reference compilation and safe runtime configuration. Use disposable test accounts only after cache and persistence isolation is verified.
2. Extract RSProt/Java-client 239.1 fixtures for handshake, revision/subrevision, packet opcode/length tables, byte transforms, cryptography, CRC ordering, and login success/failure. The pinned module name establishes revision 239 only; subrevision 1 remains unverified.
3. Implement a minimal Rust listener, login, initialization and sessions only against those fixtures. Unknown messages must produce explicit diagnostics/errors rather than being ignored.
4. Add deterministic Kotlin/Rust comparisons for cycle ordering, movement/collision, synchronization, inventory, persistence, combat and each content module. Record source revision and inputs for every fixture corpus.
5. Test a stable exported client snapshot after headless protocol parity. Do not modify Agent 1's active client. Record any compatibility changes for later handoff; none are yet proven necessary.

No live Kotlin test session, Rust connectivity, graphical client integration, or full parity has been verified yet.
