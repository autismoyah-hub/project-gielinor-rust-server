Latest development checkpoint: see [SOURCE_CHECKPOINT.md](SOURCE_CHECKPOINT.md). Earlier Alpha validation applies to its recorded release source.

# Project Gielinor Rust — Alpha 0.1.0

A native Rust server targeting OSRS revision 239.1, built from the Project Gielinor/RS Mod foundation with RSProt-informed networking. This is a development alpha, not complete OSRS gameplay parity or a production-hosting release.

The workspace separates `gielinor-protocol` (wire/crypto/session codecs), `gielinor-cache` (read-only cache/symbol decoding), `gielinor-world` (deterministic simulation and content), `gielinor-storage` (account persistence/authentication primitives) and `gielinor-server` (TCP runtime and configuration).

Implemented source paths cover login, player/NPC synchronization, inventory/equipment/banking, movement/interaction, combat/runes/prayer, tutorial/Tower content and persistence. Ordinary gathering is under active Wiki-driven development: source presence is not live acceptance. Read [RELEASE_NOTES.md](RELEASE_NOTES.md) for exact validation and limitations. No parity percentage or comparative performance result is claimed.

## Build

Windows x86-64 with Rust 1.99.0 GNU is the preparation toolchain; the native application currently requires Windows account storage. Other OS support is not established. Dependencies are vendored and compilation does not require Kotlin, a gamepack, cache files, external DLL downloads or another development checkout.

```powershell
cargo build --release --offline --locked -j 1
cargo test --offline --locked -j 1
./target/release/gielinor-server.exe --help
```

See [BUILDING.md](BUILDING.md) for a clean isolated build and [CONFIGURATION.md](CONFIGURATION.md) for external cache, symbol, authentication and runtime prerequisites. No Jagex cache, gamepack, assets, account database, credentials, Kotlin reference snapshot or compiled binary is included. No unauthorized asset download is automated.

## Contribute through the Wiki backlog

[Open Issues](https://github.com/autismoyah-hub/project-gielinor-rust-server/issues) contain stable GR task identifiers, Wiki revisions, existing Rust/Kotlin assessments, research provenance, design guidance and acceptance criteria. Unknown coverage is not absence; implemented code is not verified gameplay. Read [ISSUE_WORKFLOW.md](ISSUE_WORKFLOW.md), [CONTRIBUTING.md](CONTRIBUTING.md) and [SECURITY.md](SECURITY.md). The owner reviews implementation and merges; auto-merge is not enabled by this release process.

## Client relationship and licensing

The server is developed alongside a native Rust client. That client remains private; no client executable or source is included here. Historical native-client interaction evidence against Kotlin does not establish acceptance against this Rust server snapshot.

[LICENSE](LICENSE) retains RS Mod's ISC notice and the owner-confirmed ISC declaration for new Rust contributions. [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) preserves RSProt MIT and dependency-specific terms. These licenses do not grant rights to Jagex assets or uncleared third-party material. [SOURCE_PROVENANCE.md](SOURCE_PROVENANCE.md) records source identities and boundaries. This is an unofficial project, not endorsed by or affiliated with Jagex; no official Jagex service compatibility is promised.
