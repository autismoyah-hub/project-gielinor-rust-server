# Source synchronization checkpoint

## 2026-10-25 monorepo sync

Source: private monorepo `autismoyah-hub/osrs-workspace`, path `rust-server/`, main commit `cd3603315a443b821b75f339ada1d0508c50cacc` (merge of PR #192). Public parent: `1aa20582`, the commit the monorepo server was originally imported from. Exported with `git archive`; no working-tree files were used.

Code folders (`crates`, `Cargo.toml`, `Cargo.lock`, `vendor`, `tools`, tracked `scripts` and `docs`, `.cargo`) now match the source commit. Public root documents, `LICENSE`, `THIRD_PARTY_NOTICES.md`, `licenses/`, `config/example.ps1` and the per-crate `autotests = false` / `autoexamples = false` discovery settings are retained. `integration/`, `reference/`, `build.ps1`, private cache/fixture-dependent tests and examples, cache-derived `*.expected` fixtures, parity scripts and unpublished docs stay outside this tree under the existing `.gitignore` policy. The public test-only authentication fixture in `crates/server/tests/fixtures/authentication/` is published deliberately.

This sync was not build- or test-verified in this repository; the monorepo "Workspace checks" CI compiled the server workspace for the source changes. Release audit: AUDIT-2026-10-25.md (kept with the release preparation, outside this tree).

## Earlier checkpoint

Authoritative source captured 2026-10-16T01:28:41.507Z through 2026-10-16T01:31:29.500Z UTC. Parent public commit: dc17b90e768603e22555be991c35f8811ac4fc28. Matching selected-file hash scans and verified independent recovery establish observed byte stability, not an atomic or semantically consistent snapshot. Later active edits belong to a future checkpoint.

This development checkpoint is not independently build- or test-verified. No builds, gameplay suites or servers were run. Previous Alpha 0.1.0 validation documents apply to that published source, not automatically to these newer changes. Incomplete development work is preserved.

Existing public release documentation, notices, dependency files, public test-discovery settings and external-input test ignores are retained. Private development tests/examples/fixtures, reference source, integration research, scripts, local state, secrets, assets and build outputs remain outside this public tree. Reviewed source additions and changes are recorded in local SHA-256 manifests and independent recovery.

No active workspace, GitHub Issues, backup branch, tags, visibility, Actions settings or release candidate was modified. No automatic synchronization enabled.
