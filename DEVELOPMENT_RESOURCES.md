# Development resources

Inventory date: 2026-10-15. Paths below describe private local resources, not permission to publish their contents. Base directory is C:\Projects\OSRSRSPS. No resource was moved or deleted.

## Reference source

OpenRune root: C:\Projects\OSRSRSPS\source-reference\OpenRune-Server-main\OpenRune-Server-main. All 2,948 inspected Kotlin, Java, Gradle and license/notice files were readable and hashed with zero errors. Build/cache/editor/generated directories were excluded. No .git identity was present: completeness relative to an upstream commit cannot be established. External openrune-source-manifest.json SHA256: 708a66a4cc4c8caa8559c4243cc21d9b5df6d784a1807bff9e52eb67922d5eb9. Retain this snapshot before obtaining any replacement archive.

game.example.yml specifies revision 241; GameServer.kt initializes the cache using that configuration. An AGENTS.md introduction mentions 240.2. Neither is authority for target 239.1. Root LICENSE.md is ISC (RSMod, 2025). api/drop-table/license.md has MIT-style terms PLUS a notification condition; review that exact condition before reuse. LineRouteFinding.kt and LineValidator.kt carry BSD-style notices. Root licensing cannot replace file-level provenance checks. No OpenRune source was modified or ported.

PluginScriptLoader uses PluginClasspathScan subclass discovery, filters abstract/interface classes and instantiates scripts via Guice. Content scripts use the script API; drop definitions use Kotlin/TOML, with cache/resource packing modules and gamevals.toml supporting registration data.

Content families include areas, bosses, drops, events, generic, interfaces, minigames, other, quest, skills and travel. Skills include cooking, crafting, firemaking, fishing, herblore, magic, mining, prayer, runecrafting, slayer, smithing, thieving and woodcutting. Combat/NPC leads include api/combat-maxhit, generic-npcs and special-attacks/special-weapons. Shops have api/shops, ShopRestockProcess, engine/game shop types and Wiki dumping tools. Items/equipment leads include generic-items, consumables and special-weapons; world interactions include generic-locs, travel, windmill and sawmill. Quests are under content/quest; a complete diary registry has not been established in this bounded readiness pass. Boss/minigame directories exist; existence is not an exhaustive implementation audit. Inspect settings/build scripts, content resources, or-cache and game.example.yml before attempting reference execution. Private .data is not publishable.

Original Kotlin: C:\Projects\OSRSRSPS\Project-Gielinor, observed Git HEAD cb6e66cc0f16a25b60ba6705d8f157cc22ab4869. This identifies the commit, not a clean-working-tree attestation. Local modified reference: rust-server\reference\kotlin; prior overlays: reference\pre-overlay; runtime overlays: work\integration\kotlin-overlay. Preserve these modifications and their classpath/source manifests. Rebuilding requires existing Java/Gradle dependencies and local cache/symbol inputs; original checkout alone cannot regenerate modified snapshots.

Kronos 184: C:\Projects\OSRSRSPS\source-reference\kronos-osrs-184-master\kronos-osrs-184-master. Non-authoritative historical reference with unresolved reuse provenance; do not publish its source or assume revision compatibility.

## Data and native acceptance

Wiki: C:\Projects\OSRSRSPS\osrs-wiki-dump\data\osrs-wiki.sqlite (1,187,098,624 bytes), pages.jsonl (799,595,743 bytes), sync-state.json and SQLite sidecars. Existing dump.ps1, requirements and README describe regeneration. A new dump is not the same pinned evidence; preserve revision IDs/hashes and local snapshot. No database was published.

Cache/symbols: rust-server\work\integration\a-kotlin\.data\cache\game, cache\js5 and symbols; original Kotlin .data also exists. These require the licensed/local revision-specific inputs and are not regenerated from Rust source alone.

Native fixtures: work\preparation\fixtures\basic-v1 and basic-kotlin-v1. Per-run mutable state: work\state, including accounts.sqlite. Preserve exact fixtures and manifests; use isolated clones for acceptance. Never publish character/account data, fixture credentials, work\integration\keys or private runtime configuration.

Client baseline: work\preparation\client-239.1-baseline-001\source. Integration pins, builds and drivers: integration\client-baselines, integration\client-builds and integration\scenario-drivers. Follow existing INTEGRATION_HARNESS and pinned-baseline instructions; do not modify Agent 1's workspace. Client executables and private fixture inputs are excluded from public commits.

Evidence: work\integration\runs, work\integration\server and work logs/manifests. Historical traces and content-addressed executables cannot be reconstructed merely by recompiling current source; preserve them. Kotlin libraries/classpaths: work\integration\kotlin-libs. Native Argon2: work\native\argon2.dll. Existing Cargo vendor sources retain their notices.

## Size, dependencies and retention

Original inventory: work/ 57,905 files / 41,972,973,851 bytes; integration/ 3,443 / 1,299,356,338; reference/ 10,189 / 73,953,261; crates/ 728 / 104,675,563. These counts include resources intentionally excluded from publication. Detailed private inventories are in the external recovery root.

Supported Rust toolchain is read-only at C:\Projects\OSRSRSPS\rust-client\work\rustup\toolchains\stable-x86_64-pc-windows-gnu. Server owns work\cargo and work\target. Preserve offline dependencies and native libraries. Toolchain use does not authorize changing Agent 1 processes. Regenerate build outputs only when guard permits; never substitute new fixtures, caches or Wiki snapshots silently for pinned acceptance inputs.
