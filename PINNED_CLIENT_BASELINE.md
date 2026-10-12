# Pinned client baseline — preparation, 2026-10-12

Source baseline: `client-239.1-baseline-001`, protocol revision 239.1.
Location: `integration/client-baselines/client-239.1-baseline-001`.
Executable baseline: `integration/client-builds/client-239.1-baseline-001-build-001`.

## Provenance and immutability

The source was read from Agent 1's active workspace without modifying it. A whole selected-tree hash before copying, the copied tree hash, and a second active-tree hash matched on attempt 1. The snapshot manifest covers 3352 source, dependency, executable, and evidence files. It excludes target directories, game caches, and the original proprietary reference tree. Snapshot files and the independently built executable pin are read-only; each run verifies hashes. Read-only attributes are accidental-edit protection, not a security boundary.

Snapshot manifest SHA-256:
`0B57972892416B9C875C82A54541C1D998A2A77845BFF4AEA12D4A04A1A0B256`.

Agent 1 evidence was copied, including `evidence/native-music.log` and related build/test logs. The newest source also contained subsequent transfer/reconnect work. Therefore copied release executables were **not assumed to correspond to this source**. Their original manifest remains UNVERIFIED.

An owned working copy was compiled offline, with one Cargo job, below-normal priority, two CPUs, debug information disabled, and incremental compilation disabled. Original source/vendor/Cargo.lock hashes were checked against the snapshot. The only build overlays were an empty `[workspace]` and three explicit scenario examples. `build-attestation.json` records these overlays and binary hashes. Build evidence: `work/preparation-build-001.log` and `work/preparation/client-239.1-baseline-001/{build.log,build.err}`. Never build inside either immutable pin or Agent 1's workspace.

Important rebuilt binary SHA-256 values:

- GUI: `E48807845051C5F90058B29724238EFB71C36352424C8925409B1602562348FB`.
- live_actions: `8C2162F69497F358911AEA83ED751F7E6A680ED4A37D22B0D49BC8E829F44EAC`.
- live_session: `CDCC96D5A0A67E15B3790FBEDA4D94EF236DB0CFC44C9132AA261F76122D9148`.
- server_dialogue: `ACF14360B3066B05CB4E9019E124643402D3810A10594C01BF70EC0A32D64C6E`.
- server_inventory: `0D74E7B35FE11CE2FD1DC77FFD07C2833AF0F917B46FEF5FF512BD86EB92792F`.
- server_floor: `8CD58C88DDAD3DAA1F638AA2FA7375DF12E019ADBA758397FE76845EDC0B5A76`.

## Validation scope

Build/source correspondence: PASS. Revision/cache compatibility, authentication, initial world construction (zero missing regions), one confirmed walk, and NPC dialogue opening: PASS on both isolated environments in `prep-kotlin-actions-002` and `prep-rust-actions-001`. Both use exactly the same live_actions executable and canonical initial player state. Both shut down cleanly.

The differential result is **FAIL: observed persisted-state difference**, not a parity pass. Kotlin also relocated the player into the Tower during the run. See `DIFFERENTIAL_TESTING.md`. Action-driver success only means its targeted assertions passed. It does not prove the exact tanner response, full packet coverage, general character initialization, or graphical acceptance.

Baseline acceptance remains **PARTIALLY VERIFIED / UNVERIFIED for sustained graphical use**. Dedicated connected-duration/soak evidence and fresh visible GUI acceptance are absent. Native Continue has an explicit test; latest results are in `INTEGRATION_MATRIX.csv`. Immutable original manifests intentionally retain the validation state at creation. Subsequent run evidence does not silently mutate them.

## Reproduction (PowerShell 7, server root)

```powershell
Set-Location C:/Projects/OSRSRSPS/rust-server
$py = 'C:/Users/backup/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe'
& $py scripts/integration-baseline.py verify client-239.1-baseline-001
& $py scripts/integration-resources.py verify --environment kotlin
pwsh -NoProfile -File scripts/integration-preparation-run.ps1 -Environment kotlin -Run review-kotlin-actions-001 -Mode actions
pwsh -NoProfile -File scripts/integration-preparation-run.ps1 -Environment rust -Run review-rust-actions-001 -Mode actions
```

Use new run IDs each time. Existing pins, fixtures and runs are never overwritten. Future client upgrades require a new baseline ID, consistent snapshot, separate build, and explicit validation. The rebuild script is `scripts/integration-baseline-build.ps1`; its existing working directory is evidence, not a reusable clean target.
