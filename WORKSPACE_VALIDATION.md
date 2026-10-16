# Workspace validation — Git CLI migration accepted, 2026-10-15

Current status: PASS for active-workspace Git migration and normal CLI publication/recovery. OpenRune implementation has not started. The earlier blocked report below is retained as historical evidence and is superseded by this section.

## Approved execution and index repair

Default execution identity was desktop-ne0e5bh\codexsandboxonline; approved require_escalated execution ran as desktop-ne0e5bh\backup. Git 2.55.0.windows.3 and credential.helper=manager were available. No NTFS ACL, credential, Git installation or security setting was changed. The repository-specific ownership exception is passed per command/session, not installed globally:

    $env:GIT_CONFIG_COUNT='1'
    $env:GIT_CONFIG_KEY_0='safe.directory'
    $env:GIT_CONFIG_VALUE_0='C:/Projects/OSRSRSPS/rust-server'

Use approved elevated execution for subsequent Git mutations and network operations. Default sandbox writes are not claimed to work. Explicit user approval remains subject to the application's policy. No TLS override was required for the successful elevated Git CLI operations.

Before mutation, all 11,406 pre-migration snapshot hashes and all 731 recorded source/build hashes passed. The index was confirmed absent. git read-tree HEAD then initialized the index without a worktree update; git diff --cached --exit-code passed. The prior mass staged deletions disappeared. Actual unstaged status contains 16 modified tracked files and 16 absent public packaging/documentation files, matching prior reconciliation. Those local differences were neither staged nor overwritten. The 212 local Rust paths outside public history remain preserved under existing exclusions. Ignored resources were inventoried before repair and compared afterward: 68,059 paths remained present with unchanged sizes. Eleven Cargo incremental metadata.rmeta files exposed timestamp differences; all other recorded timestamps matched. No build or resource mutation was performed by this task. This inventory is not a whole-resource content-hash attestation; see cli-resource-preservation.json and cli-resource-changes.json for the limitation.

## Exact normal CLI verification

Working directory: C:\Projects\OSRSRSPS\rust-server.
Base main and origin/main: b811af5de9e10d73989070ec318601c05ddbe9f5.
Origin: https://github.com/autismoyah-hub/project-gielinor-rust-server.git.

Executed successfully (exit 0):

    git switch -c verify/agent2-cli-20261015
    git add -- GIT_CLI_VERIFICATION.md
    git diff --cached --check
    git commit -m 'docs: verify normal Git CLI from active server workspace'
    git push -u origin verify/agent2-cli-20261015
    git ls-remote origin refs/heads/verify/agent2-cli-20261015
    git clone --single-branch --branch verify/agent2-cli-20261015 https://github.com/autismoyah-hub/project-gielinor-rust-server.git cli-recovery

Only the harmless marker was staged; its diff was reviewed before commit. Local HEAD, upstream, remote HEAD and fresh independent recovery HEAD all matched 406e962e902c4b0f0c98bb24c5be2d872d2ccd40. Marker SHA256 matched 03d18a32223ca1a5a0cf3762f3f3221d902bad7efd66f12052c64c5c0968f48c.

The independent recovery checkout is C:\Projects\OSRSRSPS\git-staging\agent2-migration-20261015\cli-recovery. The verification branch tracks origin/verify/agent2-cli-20261015. A subsequent explicit documentation-only commit/push records this report and the updated workflow/recovery instructions; its exact final local/remote/recovery SHA is recorded in cli-completion-receipt.json outside the worktree to avoid a self-referential commit hash. Main is not modified. The active workspace intentionally retains uncommitted gameplay and packaging differences.

## Build result retained, not rerun

PASS: cargo check --offline --locked -j 1 -p gielinor-server, exit 0, 13.64 seconds, rustc 1.99.0 (b940084d7, 2026-09-28), x86_64-pc-windows-gnu. Prior launch guard: 3,089 MiB available, 79.4% commit, 1,255 pages-in/sec, 2.2 ms disk latency, 8.9% CPU; no competing compiler at launch. Vendored num-bigint/libbz2 warnings only. Exact prior wrapper error: argument '--jobs <N>' cannot be used multiple times; the later direct command passed. Source manifest remains 8a9dfc465e36851a6e30b3c25494e107d5d52cf2dfbff7308fd71ee7c22fa4dc. Hash reconciliation confirmed the validated source was unchanged, so no repeated build was needed. This does not establish runtime acceptance or full gameplay parity.

## Historical preparation report

# Workspace validation, 2026-10-15

Overall: PARTIAL / BLOCKED. Source preservation, bounded current-source compilation, OpenRune readability and GitHub API publication/recovery passed. Active-workspace Git migration and ordinary CLI push did not pass. No OpenRune port began.

## Source and Git status

Public main observed at b811af5de9e10d73989070ec318601c05ddbe9f5. Backup branch backup/pre-grok-issues-2026-10-15 was present at 3559f2ec19b821364f4bcb2a7cd5490e4ab70df9. No main or backup history was rewritten.

Active workspace started without .git, hence no original branch/stash/worktree metadata. After metadata copy, main points to the public base but index/ref writes are denied. Do not interpret its staged-deletion status as real source changes. Isolated verification clone has an ordinary single worktree and no stashes.

Reconciliation found 16 normalized differences against public history, including local manifests, smelting, smithing, simulation and password handling. There are 212 local Rust paths absent from the public tree, many intentionally private tests/examples; this is not 212 new production modules. Sixteen public packaging/documentation paths are absent locally. Do not indiscriminately synchronize or publish these differences.

Every original snapshotted file remained present; only .gitignore changed during setup, to the existing remote source-only exclusion policy. The four new readiness documents are additional files. Existing implementation, including unfinished iron-smelting work, was preserved. External backup scope and repair instructions are in WORKSPACE_RECOVERY.md.

## Current source compilation

PASS: cargo check --offline --locked -j 1 -p gielinor-server.
Actual supported compiler: rustc 1.99.0 (b940084d7, 2026-09-28), x86_64-pc-windows-gnu, LLVM 23.1.1.
Source identity: 731-file manifest SHA256 8a9dfc465e36851a6e30b3c25494e107d5d52cf2dfbff7308fd71ee7c22fa4dc.
Guard at launch: 3,089 MiB available, 79.4% commit, 1,255 pages-in/sec, 2.2 ms disk latency, 8.9% CPU; no competing compiler at launch. Completed exit 0 in 13.64 seconds, checking world/protocol/cache/storage/server. Warnings were in vendored num-bigint and libbz2 code. No compiler errors.

The first wrapper attempt failed because -j 1 duplicated build.ps1's own jobs argument. A competing compiler caused another attempt to defer. The final direct command ran after Assert-ServerBuildCapacity with the supported server environment. The PowerShell transcript work/workspace-migration-check-003.log did not capture native Cargo output; completion/output were observed in the execution tool result. Do not call that transcript a complete compiler log.

This is compilation checking, not a full test-suite run, runtime acceptance or functional parity. Historical Alpha results do not attest current source. No native acceptance was run during setup.

Resource policy: measure Windows Available MBytes immediately before builds. Above 4 GiB permits bounded compilation; 2-4 GiB max two jobs; 1-2 GiB one job; below 1 GiB defer. Inspect commit pressure, sustained paging, disk, competing compilers and shared services. Prefer one job. Do not count standby cache as unavailable or terminate another agent. Final disk observation: 9,117,540,352 bytes free. A later Win32_Process CIM query was denied; it is not proof of process absence. No owned build or server was left running by this task.

## GitHub and reference verification

The authenticated connector published a harmless marker on verify/agent2-api-20261015. Local fetched HEAD, remote HEAD and independent recovery HEAD all matched d62a03d4e7e041063c6e91672c9c340994821edd; marker content hashes matched. Ordinary CLI push attempts returned exit 128 without diagnostic text and remain unverified. Read-only Git transport worked with per-command OpenSSL instead of the failing Schannel backend.

Issue search/read, native parent/sub-issue links, labels and historical comments passed. Repository permissions advertise write/triage/admin capability. Actual evidence-comment, close and reopen writes were not exercised and remain unverified. No gameplay issue was created or closed.

OpenRune: 2,948 inspected source/license files readable, zero read errors, no upstream commit identity; revision 241 configuration and mixed file-level license terms documented in DEVELOPMENT_RESOURCES.md. Complete upstream archive equivalence and exhaustive content/diary coverage are not established.

## Blocking next steps

Repair the app/environment's writable Git metadata access for the active server directory, then initialize its index without changing worktree files and review true differences. Explicit grants did not overcome Windows sandbox deny ACLs; this session did not bypass them. Resolve and retest ordinary CLI push if it is required in addition to the verified connector workflow. Do not start OpenRune migration until workspace readiness is accepted. No gameplay acceptance gaps were reclassified as complete.
