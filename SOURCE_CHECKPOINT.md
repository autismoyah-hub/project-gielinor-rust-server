# Source-recovery checkpoint — rust-server

Captured 2026-10-14T00:49:44.420Z through 2026-10-14T00:52:18.644Z (UTC), with matching selected-file hashes before, during and after copying and verified separate recovery. This is observed byte stability, not an atomic snapshot, semantic consistency guarantee or build verification. Later active development is intentionally outside this checkpoint.

Parent: 3c05ae173714514ba1ce2634fb9b60216169c8cd. Local checkpoint and one guarded upload authorized by the October 13 request. No automatic synchronization. Active development workspaces were read only. Original snapshots, SHA-256 manifests and independent recovery copies are stored outside this repository.

This is not an independently build-verified or fully reproducible release. No builds, servers or tests were run during checkpoint preparation. Source preserves incomplete work. Progress documents contain historical agent reports; they are not independent verification by this checkpoint.

Full differential/integration reproduction depends on excluded expected fixtures, oracle/reference material, immutable client baselines and local cache/runtime data. Existing and new gameplay source remains preserved, including incomplete implementations. Source availability does not establish gameplay acceptance.

Vendored dependencies and license notices are retained. Excluded proprietary references, caches, credentials, databases, binaries, raw telemetry and held fixtures are not repository contents. Machine-specific paths and cache assumptions in existing scripts/docs require local configuration. Existing license notices do not certify unresolved provenance; private visibility does not resolve inclusion rights.

## Active-file deferrals

Three complete capture attempts observed active edits. The accepted source subset excludes these unstable paths:

- crates/protocol/src/zone.rs
- crates/server/src/ground_sync.rs
- crates/server/src/runtime.rs
- crates/world/src/simulation.rs
- crates/world/src/zones.rs
- reference/SoundAreaWireOracle.java
- scripts/sound-area-wire-parity.ps1

Previously committed tracked versions are retained for 5 paths; 2 unstable new paths are omitted. Attempted newer bytes remain only in diagnostic snapshots and are not claimed consistent. This repository combines source from different times and is NOT asserted semantically consistent or buildable. No source deletions were inferred from these deferrals.
