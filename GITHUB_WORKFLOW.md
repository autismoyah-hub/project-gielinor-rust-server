# Agent 2 GitHub workflow

Verified 2026-10-15. Repository: https://github.com/autismoyah-hub/project-gielinor-rust-server. Target runtime: OSRS 239.1.

Agent 2 owns server commits, branches, publication, evidence and accepted issue status. Agent 3 is no longer the routine publisher; Agent 4 remains stopped. GitHub is authoritative for committed history; local uncommitted development must be preserved.

## Current gate

Active workspace Git metadata is incomplete and not writable by this session despite permission grants. Do not stage or commit there until WORKSPACE_RECOVERY.md is followed. Source files remain intact. Ordinary CLI pushes failed; authenticated GitHub connector publication and independent Git recovery succeeded. This is a verified alternate publication workflow, not proof that CLI push works.

Use a new topic branch from an inspected base; never rewrite history or force push. Preserve existing main and backup branches. Prepare an explicit path list in an isolated clean checkout while active metadata is blocked. Review staged diff, whitespace, file types, licenses and private-data exclusions. Never use blanket staging of this development directory.

The validated fallback creates a Git tree from the exact reviewed text on an existing base, creates a single-parent commit, then updates the topic ref with expected old SHA and force=false. Fetch that commit and verify its tree, content and remote SHA. Connector commits can differ from local candidate commit IDs because author/time differ; compare trees before publishing and final fetched commit IDs afterward. Stop on any lease mismatch.

For HTTPS reads, this host required per-command git -c http.sslBackend=openssl; TLS verification stays enabled. Default Schannel returned SEC_E_NO_CREDENTIALS. Two CLI pushes returned exit 128 without diagnostics. No credentials were extracted or changed. Do not assume this identifies the authentication failure.

## Issue workflow

Search/read works. Native issue #82 has parent #12; #12 exposes children #77-83, #113, #115, #117 and #118. Labels and historical comments were readable. #82 was locked and open. Dependency links in issue bodies are readable; native blocking-edge endpoints were not validated.

The connected repository permissions advertise admin, maintain, push, triage and pull. Comment creation and close/reopen tools are available, but their write endpoints were not exercised during setup. Thus evidence-posting and close/reopen execution remain unverified, especially on locked issues. No gameplay issue was created, closed or reopened.

Select an eligible existing sub-issue, inspect current Rust, Wiki evidence and OpenRune reference, resolve licensing and revision differences, implement, compile/test, run required native acceptance, then record commit and immutable evidence. Close only fully accepted scope; reopen when regression evidence invalidates acceptance. Never infer acceptance from a compile or use deleted experimental work as requirements.

## Public safety

Keep work/, integration/, reference/, cache/gamepacks, databases/accounts, keys, private configuration, unapproved binaries and third-party trees private. The active .gitignore now matches the remote source-only policy; the prior version is backed up. Explicitly audit each proposed test/fixture and its provenance before changing exclusions. Preserve LICENSE, THIRD_PARTY_NOTICES.md and licenses/ from public history. Do not overwrite local manifests with public packaging variants merely to make a diff disappear.
