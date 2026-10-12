# Gielinor Rust server

Active server rewrite targeting OSRS 239.1. The workspace includes TCP login/session management, world simulation, account persistence, interfaces, inventory/equipment/banking, combat state, rune validation, synchronization, and a read-only JS5 service. The development executable has passed startup validation and a bounded loopback smoke test against the existing cache. Complete gameplay parity and a playable client demonstration remain outstanding.

The initial Kotlin baseline is Project Gielinor commit `cb6e66cc0f16a25b60ba6705d8f157cc22ab4869`. Its source snapshot is in `reference/kotlin`; production saves, keys, caches, and uncommitted changes are excluded. See `docs/BASELINE.md` for architecture and compatibility gaps, and `docs/PROGRESS.md` for validation status.

Run from a fresh PowerShell process in this directory:

```powershell
.\build.ps1 -Action test
.\scripts\coordinate-parity.ps1
.\scripts\isaac-fixtures.ps1
.\scripts\reference-build.ps1
```

The first command runs Rust tests offline, including 4,452 coordinate cases and 5,125 ISAAC outputs from checked-in reference fixtures. The coordinate script compiles the original Kotlin map module with cached Kotlin 2.2.0 jars, executes the cases, then builds and compares Rust output. The ISAAC script regenerates RSProt outputs under `work/isaac-oracle`; compare these with the checked-in fixtures before intentionally updating them. Kotlin and Rust compilation run sequentially. The reference-build script attempts isolated Gradle map compilation, without installation, cache packing, or server startup.

Use `-CargoArgs @('-p', 'gielinor-world')` for package selection. Build scripts check available RAM, CPU samples, and compiler processes, lower their own priority, and restrict their own CPU affinity. Cargo uses two jobs and independent storage under `work`. Launch scripts in a dedicated shell because process priority, affinity, and environment changes last for that shell's lifetime.

Compiler binaries currently come from the existing client-local Rust toolchain, read-only. Cargo home and target output are server-local. Do not update that toolchain from this workspace. Kotlin jars are read-only inputs; Gradle's distribution copy and writable home are server-local. The Gradle copy under `work/gradle-dist/gradle-9.0.0` avoids a Windows/JDK ZIP-filesystem access failure with the shared distribution. It was copied from the existing Gradle 9.0.0 installation without network downloads.

If the snapshot is missing, `.\scripts\snapshot-reference.ps1` recreates it from the pinned local Git object. It deliberately refuses to overwrite an existing snapshot. Preserve upstream ISC attribution in `LICENSE`.

The executable validates configured content before it opens a socket or account database:

```powershell
.\build.ps1 -Action run -CargoArgs @('-p','gielinor-server','--','--check',
  '--game-cache','C:/Projects/OSRSRSPS/Project-Gielinor/.data/cache/game',
  '--js5-cache','C:/Projects/OSRSRSPS/Project-Gielinor/.data/cache/js5',
  '--symbols','C:/Projects/OSRSRSPS/Project-Gielinor/.data/symbols')
.\scripts\startup-smoke.ps1
```

The smoke script uses checked-in test keys, creates isolated storage under `work/state/executable-smoke`, binds an ephemeral loopback port, runs two ticks, and drains shutdown. It never starts Kotlin or writes its cache/saves. For an interactive development run, use `--help` for explicit key, Argon2 DLL, state-name, and bind options; omit `--ticks` and use Ctrl+C to log players out and drain saves. The executable currently loads only the nine map squares around the configured spawn. Do not use the public fixture keys outside isolated testing.
