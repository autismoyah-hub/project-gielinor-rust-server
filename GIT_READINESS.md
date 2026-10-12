# Private Git readiness — proposal only

No repository was initialized, committed or pushed. Proposed repository root is `C:/Projects/OSRSRSPS/rust-server`; keep Agent 1's client in its own separately authorized repository. Do not initialize the common parent or import the entire local workspace by default.

## Source inventory and review boundary

- `crates/{cache,protocol,server,storage,world}`: Rust source and tests. `Cargo.toml`, `Cargo.lock`, `.cargo`, `build.ps1`: build configuration; inspect machine-specific settings first.
- `scripts`: build, Kotlin oracle, integration, comparison and fixture tooling. Inspect all legacy scripts for machine paths or old disposable credentials before staging; this preparation replaced a hardcoded password in `integration-prepare.ps1` with generate-once random material.
- `reference`: owned oracle/scenario sources mixed with protocol material, isolated Kotlin source, overlays and generated fixtures. Classify individual files by provenance/license; do not bulk-stage it.
- `vendor`: dependency source, requires license/provenance audit and size decision. Preserve reproducible dependency versions and notices; Git LFS is not a workaround for distribution restrictions.
- `docs`, root Markdown and matrix: architecture, progress and preparation reports; inspect absolute paths and copied evidence excerpts before publishing even privately.
- `integration`: immutable client source snapshots, copied third-party code, executable pins, manifests and local resource paths. Keep large snapshots/binaries outside the initial source repository. A sanitized manifest can reference an access-controlled local artifact backup by digest.
- `work`: compiled outputs, caches, runtime jars/classes, private keys, password files, DLLs, captures, logs, screenshots, SQLite state and build experiments. Exclude entirely.

## Proposed ignore additions (review later; not applied here)

```gitignore
/work/
/target/
/reference/kotlin/
/integration/client-baselines/
/integration/client-builds/
/integration/resource-manifest.json
**/__pycache__/
*.pyc
*.exe
*.dll
*.pdb
*.class
*.jar
*.db
*.db-*
*.sqlite
*.sqlite-*
*.key
*.pem
*.p12
*.password
.env
.env.*
```

Existing `.gitignore` only excludes work, reference/kotlin and target. These proposed patterns are a first defense, not a complete secret scanner. Use an explicit initial source allowlist; inspect any binary/cache/reference file outside ignored paths. Add narrowly named approved fixture exceptions only after provenance review. Do not ignore Cargo.lock. Do not commit local absolute runtime classpaths or secrets embedded in otherwise legitimate source.

## Never push

Game caches, game assets or other proprietary reference material without distribution rights; test/private RSA keys; password files or password hashes; SQLite account saves; production credentials; raw packet captures; unreviewed local logs; active user/account data; compiled copied executables; JVM caches and temporary runtime libraries. A private repository still transmits data to a third party. Never bulk `git add .` before the inventory/ignore/secret review.

## Backup and current risks

The workspace is not yet versioned. Immutable pins protect against ordinary accidental edits but are not independent backups. Keep an encrypted, access-controlled backup of owned source and irreplaceable manifests/evidence separately from Git, preserving SHA-256 manifests. Large local caches may be reproducible; keys and account saves need a distinct restricted backup policy. No external backup was created in this phase.

Current Rust source is newer than the archived integration server: prayer book/drain/timer work is incomplete. A focused 100-test world batch passed earlier, but two pending differential expected files are absent and a Kotlin prayer-drain oracle compile failed. Do not claim a clean full-workspace build or merge-ready source. Preserve this work explicitly; do not reset it to make Git setup look cleaner. The integration server pin has separate source-hash evidence and prior 319-test regression evidence.

## Authorized next steps after user approval

1. Confirm separate private repository roots and hosting/ownership, licensing and backup policy.
2. Back up the unversioned source, then review proposed ignores and an explicit source allowlist.
3. Scan staged candidates for secrets, absolute machine configuration, oversized and proprietary files; review the exact staging list.
4. Initialize only the approved server root, commit reviewed source and documentation with the actual incomplete-state notes, and configure the approved private remote. Push only after authorization.
5. Document how excluded artifacts are obtained and hash-verified. Establish a new validated server build from source before treating current work as reproducible parity evidence.

These are future instructions, not actions taken by this preparation task.
