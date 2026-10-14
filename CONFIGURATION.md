# Runtime configuration

The executable takes command-line arguments; `config/example.ps1` is a commented, non-executing command template, not an automatically loaded server config. Use `--help` from the exact built executable.

Required external inputs for content checking are a matching revision-239.1 game cache, JS5 cache and symbol directory. Obtain and use these independently with appropriate rights. There is no bundled asset pack or cache download helper.

```powershell
./target/release/gielinor-server.exe --check --game-cache 'D:/external/game-cache' --js5-cache 'D:/external/js5-cache' --symbols 'D:/external/symbols'
```

`--check` validates decoded content before opening a socket or account database. It is not a cache-free self-test. Paths above are placeholders; this exact cache-backed command is not claimed run by Scout in this task.

To run a disposable local server after content checks:

```powershell
./target/release/gielinor-server.exe --game-cache 'D:/external/game-cache' --js5-cache 'D:/external/js5-cache' --symbols 'D:/external/symbols' --rsa-exponent 'D:/private/rsa-private-exponent.raw' --rsa-modulus 'D:/private/rsa-modulus.raw' --argon2 'D:/private/argon2.dll' --state-name 'my-disposable-alpha' --bind '127.0.0.1:0' --ticks 2
```

RSA inputs are raw big-endian bytes, not PEM text. Provision your own development keys and compatible client public key; never submit private material. Account authentication dynamically loads a compatible Windows Argon2 library; no DLL is shipped. Use the upstream Argon2 implementation/distribution under its applicable terms (https://github.com/P-H-C/phc-winner-argon2), not unpublished local binaries. OS SQLite and Windows crypto APIs are also runtime dependencies. No external binary is required merely to compile.

Only loopback bindings are accepted; port 43594 is rejected. Port 0 requests an unused local port. Default spawn is 3222,3218,0; override with `--spawn X,Z,LEVEL` only using matching content. Repeated `--symbols` adds overlays. `--stop-file PATH` and Ctrl+C request a draining shutdown. This alpha stores state beneath the compilation checkout's `work/state/NAME`; relocating only the executable is not a supported deployment method. State names are constrained; no production save is provided.

## Known data portability gap

`crates/server/src/kronos_combat.rs::DEFAULT_DIRECTORY` still points at the developer's Kronos combat-data directory. `startup.rs` attempts to load it and emits failures if absent, then continues with an empty catalog. This means NPC stat/XP/spawn/respawn bindings may differ on a clean machine. The data is not bundled or licensed by this release. Do not obtain an uncleared archive to fill this gap. Agent 2 must make an explicit independently approved data-source/configuration decision before full combat equivalence can be claimed. Scout has proposed that change without altering runtime algorithms.

`--elemental-source` is a private research override that parses external source files; it is unnecessary for the default public configuration and must remain unset unless independently permitted. Public compilation requires none of those inputs. No internet-facing deployment or production account security assessment is claimed.
