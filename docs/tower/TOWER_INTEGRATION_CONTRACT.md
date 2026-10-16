# Tower integration and ownership

The standing development mandate supersedes the initial research-only restrictions. Agent 5 may change simulation, runtime, application and other necessary engine files inside this isolated clone. Intermediate Agent 2 approval is not required. Keep changes bounded, explicit and reviewable in the engine-delta ledger.

Agent 2's workspace, processes and `agent2/openrune-content` branch remain untouched. Agent 1 owns ongoing native client development; use the isolated reference checkout for testing, keeping any validation-only changes separate. Agent 3 remains undisturbed and Agent 4 paused.

Push only `agent5/tower-development`, using ordinary non-force updates. Publish only independently authored source, tests with approved synthetic inputs and safe metadata. Never publish cache assets, game binaries, keys, credentials, account state, reference archives or confidential fixtures.

Fetch upstream at deliberate checkpoints, record exact refs, review relevant APIs and integrate only deliberately. Acceptance against one baseline does not validate another. Preserve an exact restart checkpoint before execution limits.

Direction B uses migration V12 `tower_run`, saved with inventories through Accounts::save. Absent state remains unstarted; legacy diary is retained separately. Native validation adapter source is supplied as `client-validation.patch` for the pinned client baseline. It is optional, local and uses the existing packet/action path; it is not required by production clients. Test credentials, private keys, frame captures and compiled clients are excluded from this branch.
