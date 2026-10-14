# Clean-clone build

Prepared platform: Windows x86-64, GNU Rust toolchain 1.99.0 (rustc b940084d7). This is the validation baseline, not an asserted minimum supported compiler. Install Rust independently through its official distribution. No absolute compiler path is embedded in Cargo configuration. Vendored path dependencies and Cargo.lock permit an offline build once the compiler is installed.

```powershell
git clone https://github.com/autismoyah-hub/project-gielinor-rust-server.git
Set-Location project-gielinor-rust-server
$env:CARGO_BUILD_JOBS = '1'
$env:CARGO_HOME = Join-Path $PWD 'work/public-build/cargo-home'
$env:CARGO_TARGET_DIR = Join-Path $PWD 'work/public-build/target'
$env:TEMP = Join-Path $PWD 'work/public-build/tmp'
$env:TMP = $env:TEMP
New-Item -ItemType Directory -Force $env:CARGO_HOME,$env:CARGO_TARGET_DIR,$env:TEMP | Out-Null
cargo metadata --offline --locked --format-version 1
cargo build --release --offline --locked -j 1
cargo test --offline --locked -j 1
& "$env:CARGO_TARGET_DIR/release/gielinor-server.exe" --help
```

Authentication is needed to clone while the repository is private. Public unit tests use synthetic inputs; private Kotlin/RSProt fixture-based integration tests and development examples are excluded from automatic Cargo discovery. The original private development tree retains them. No differential fixture regeneration is performed here. Windows runtime tests may exercise OS-provided SQLite/cryptography; test outputs belong only to this checkout and its private TEMP.

Do not infer complete gameplay parity from cargo test. Cache-backed --check and live acceptance require independently supplied runtime data. On a shared development host, use Scout's resource-guarded wrapper: no build while another cargo/rustc exists or available RAM is below 4 GiB; one job and below-normal priority. A deferral is not a build pass.

Two retained unit tests are explicitly ignored by default: storage's external Argon2 DLL check and world's private Kotlin pending-logout comparison. They require inputs not distributed here. An ignored test is not a pass; do not run `--ignored` without supplying independently permitted inputs.
