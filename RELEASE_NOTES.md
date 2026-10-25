# 2026-10-25 server sync

Source: private monorepo main `cd3603315a443b821b75f339ada1d0508c50cacc` (path `rust-server/`), replacing the server code imported from public `1aa20582`. Development snapshot, not a tagged release; not build- or test-verified in this repository (the monorepo "Workspace checks" CI compiled the server workspace).

Notable server changes from merged monorepo PRs:

- Gameplay: ordinary OpenRune fishing core for 239.1 (#5); ordinary firemaking with a tinderbox (#89); mining gem pre-roll and gem rocks (#103); clay rocks and infinite rune essence rocks (#125), with the selected essence named in full-pack refusals (#136); PvN Dragon longsword special attack (#142).
- NPCs and drops: NPC combat stats loaded from the cache instead of Kronos (#36); NPC aggression from cache hunt modes (#66); shared drop sub-tables and tertiary rolls (#83); goblin drop table 2 and Goblin Village drops (#106); Man and Woman drop tables (#112); ring-of-wealth shared drop context (#145); five-profile ordinary respawn, part A (#152); Goblin 3028 effective hitpoints corrected to 5 (#161); typed NPC attack geometry framework (#162).
- Social and session: MESSAGE_PUBLIC decoding and public-chat echo (#97); logout-panel button routed through protected logout (#98); interrupted first registrations resume instead of failing with login status 13 (#22); disconnect grace and x-log cap pinned (#28); explicit `--ordinary-world` mode (#132); free accounts refused pickpocketing explicitly (#129).
- Tower: Floor One banker Talk-to and NPC examine (#55); solo attempt and life-mode model (#116); attempt records, validated restore and recovery (#122); V14 attempt storage and request ledger (#137); sealed snapshot attempt commits (#150).
- Infrastructure: pure resource and throttle guard policy (#184); selfsuite terminal binding and verified source imports (#187); test suite made to compile in a clean worktree (#8); the server authentication test now uses an independent public test-only RSA fixture instead of private reference data (#191).

Packaging: `.cargo/config.toml` now sets `jobs = 2` and `target-dir = "work/target"`, so build outputs go to `work/target` (git-ignored) and not `target/`. Public root documents, licenses, `config/` and test-discovery settings are retained. Private integration/reference material, cache-derived fixtures, private tests/examples and parity scripts remain excluded. See SOURCE_CHECKPOINT.md.

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
