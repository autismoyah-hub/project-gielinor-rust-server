# Alpha 0.1.0 release preparation

Target: OSRS 239.1. Windows x86-64 GNU Rust 1.99.0. Source snapshot 78b3fcd2e73c777ba4dc4684bbd9400ab7736b80b5f95bea10c0621f64a2e2b8, captured 2026-10-14 UTC (October 13 Pacific). This is a development alpha, not complete OSRS parity.

## Verified preparation results

- Clean isolated `cargo metadata --offline --locked --format-version 1`: exit 0.
- Clean isolated `cargo build --release --offline --locked -j 1`: exit 0, using a fresh Cargo home, target and TEMP, one compilation job and below-normal priority. No game cache, reference source or unpublished binary was needed to compile.
- Fresh isolated `cargo test --offline --locked -j 1 -- --test-threads=1`: exit 0; 354 tests passed, 2 explicitly ignored, no failures. The ignored tests require an external Argon2 DLL and a private Kotlin logout fixture. Integration fixture suites and private development examples are excluded from default discovery. Ignored/excluded tests are not claimed passed.
- Optimized executable `--help`: exit 0. Missing required inputs: exit 1 as expected, before a session is started. This is CLI validation, not cache-backed startup validation.

The first test attempts exposed the two unpublished-input dependencies; packaging now marks them explicitly. Production Rust behavior remains unchanged from the captured source. The optimized build preceded those two test-only attributes; the final successful test run compiled the attributes. Final source/manifests used for compilation are hash-linked to the validated checkout. Later changes are release documentation only. Original failed logs are retained in private release evidence, not hidden.

## Implemented source, with bounded evidence

The source contains login/network processing, player/NPC synchronization, world simulation, movement, inventory/equipment/banking, combat/magic/prayer, tutorial/Tower interactions and persistence. The public tests exercise selected deterministic cases; they do not demonstrate complete content or live graphical behavior. Ordinary gathering remains under active Wiki-driven development. Refer to the existing GR Issues for per-feature acceptance criteria.

No fresh live Rust-client/Rust-server session, cache-backed `--check`, broad multiplayer soak, cross-platform run or performance comparison was executed for this release. Historical native-client interaction records against Kotlin are separate evidence and are not a pass for this server snapshot.

## Known limitations

Windows runtime only; Rust 1.99.0 GNU is the validated baseline, not a declared minimum compiler. Runtime requires independently supplied matching cache/symbol data, raw RSA parameters and a compatible Argon2 DLL. Data, assets and credentials are not bundled. Bindings are loopback-only and port 43594 is rejected. State uses the compilation checkout path; distributing only the executable is unsupported.

NPC combat data still uses a developer-specific optional external directory. Missing data produces an empty catalog and potentially different bindings; this portability defect is documented, with a narrow proposal supplied to the implementation owner. Unlicensed external data is not a suggested workaround.

First-party ISC contributor attribution was confirmed by the owner. RS Mod ISC, RSProt MIT and all vendor notices remain separate. The exact RSProt artifact-to-source-commit mapping remains unverified; no statement of asset redistribution rights is implied. SOURCE_PROVENANCE.md records the boundary.

Proposed future tag: `v0.1.0-alpha.1`. No tag, public Release, binary upload, visibility change or forum announcement was made by preparation.
