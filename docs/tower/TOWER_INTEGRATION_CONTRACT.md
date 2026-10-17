# Tower integration and ownership

The standing development mandate supersedes the initial research-only restrictions. Agent 5 may change simulation, runtime, application and other necessary engine files inside this isolated clone. Intermediate Agent 2 approval is not required. Keep changes bounded, explicit and reviewable in the engine-delta ledger.

Agent 2's workspace, processes and `agent2/openrune-content` branch remain untouched. Agent 1 owns ongoing native client development; use the isolated reference checkout for testing, keeping any validation-only changes separate. Agent 3 remains undisturbed and Agent 4 paused.

Push only `agent5/tower-development`, using ordinary non-force updates. Publish only independently authored source, tests with approved synthetic inputs and safe metadata. Never publish cache assets, game binaries, keys, credentials, account state, reference archives or confidential fixtures.

Fetch upstream at deliberate checkpoints, record exact refs, review relevant APIs and integrate only deliberately. Acceptance against one baseline does not validate another. Preserve an exact restart checkpoint before execution limits.

Direction B uses migration V12 `tower_run`, saved with inventories through Accounts::save. Absent state remains unstarted; legacy diary is retained separately. Native validation adapter source is supplied as `client-validation.patch` for the pinned client baseline. It is optional, local and uses the existing packet/action path; it is not required by production clients. Test credentials, private keys, frame captures and compiled clients are excluded from this branch.

## Reviewed handoff at completion

Read-only upstream fetch observed Agent 2 `521e2e047071a7e7d481ab458829bb88a57b76e2`, with common ancestor `6a1ea78022bd1c9cbc7e259bcb3595652e5eaae5`. It is not merged or modified. Relative to the earlier observed ref, reviewed shared changes add chemistry dialogs, potion upgrades and combat-mixing queues across application/runtime/simulation. Preserve those fields, cancellation hooks, queue arms and save projection alongside Tower fields; repeat combined validation after integration.

**Known migration collision:** Agent 2 now owns `V12__chemistry_state.sql`; this pinned Tower branch owns `V12__tower_run.sql`. Append/renumber the Tower migration in the integrated migration sequence, preserving both tables and schema-upgrade fixtures. Do not blindly overlay this branch's migration list or an existing database. This branch's isolated schema and private disposable database remain internally consistent.

Checkpoint requests intentionally reuse the existing account worker/save transaction; the actor stays reserved and observable until real logout. Integrators should preserve upstream account metadata and any additional saved content fields when combining `Character::save` and worker APIs. No global account-transfer crash guarantees or acceptance of unrelated engine parity are implied by the bounded Tower scenarios.

The client mailbox is exported as raw Git bytes and verified against the exact tested tree. Its baseline contains CRLF blobs: apply with `git am --keep-cr`. The repository already stores files with `-text`; do not normalize the mailbox through a text-output pipeline.
