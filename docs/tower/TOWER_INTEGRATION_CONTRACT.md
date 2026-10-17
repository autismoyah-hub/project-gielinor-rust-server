## Current guarded waterworks and furnace composition

Implementation `c7e4230b897d93ba2cb83fd7b9a60eaf16097ffc`, executable `6D94F65F360C95B789506EDCBCCA03F416A3433DC4EA15218A566E9934ABA5A9`. Floor origin remains 10368,64. The four existing passive3317 fish NPCs stand at local72,50/52/54/56; ordinary net fishing reaches them from flat dry local70,50/52/54/56. The wheel landmark moves to local72,58 and retains Water state/angle0/dormant18512/restored18511. Continuous low masonry on local70,z48..63 and two endcaps protect the height seam; ordinary through-crossings are local69/70,z48/64 in both shell and opening declarations. Quay chunks use actual377,545 rotations0/1; genuine water uses187,566. No terrain-byte or interaction-reach override.

Middle Foundry landmark local55,78 now uses18526/18525 angle1, matching the two neighboring rotated forge mouths; the extra mouth tile has a real shell opening. Piers local54/59,78 and boiler connections local69/74,76 remain beyond combat bounds. Imp and Warden bounds, coolant coordinates, authoritative encounter rules, service flags, inventory, rewards and persistence format are unchanged. Current victory palette is18526,18511,16811,16811,18525,18525,18525,18525. See `WATERLINE_ART_AND_LIFECYCLE_20261017.json` for exact evidence scope and preserved failures. Older coordinate/palette statements below describe historical checkpoints.

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

## Visual production pass additions

Preserve the original authoritative Warden id 16295. Its visual is selected with the standard NPC_INFO transform block to 2075, plus standard name/level blocks on initial observer admission. Do not replace its combat metadata with a normal fire giant's statistics or XP multiplier. Cache footprints must match and required sequence durations must validate before admission. Retain the scoped defend/death animation selection for the transformed skeleton.

Construction removes JS5-only conveyor frames that are absent from the paired server game-map baseline. Keep these exact normal zone-delete records through login, replay, rebuild and restart. Do not replace this fix with cache edits or a Tower-specific renderer. Apply collision clearing before authored walls. Preserve wall and decoration layers separately. The inside portal is now (10432,124); outside remains (3225,3218).

No persistence schema changes are added by this visual pass. The previously documented Tower V12 / upstream chemistry V12 collision still requires deliberate integration reconciliation. No newer Agent 2 code has been merged.

### Current readability and interaction refinement

Foreground southern room edges use supported low stone wall 5154 through `prepare_palette`; exterior, back walls and plaque support remain full-height 2341. Movement stays blocked, while range policy follows the actual cache asset. Native comparison at the foundry threshold shows the avatar remains visible. No renderer or input-readiness bypass is used.

Coolant uses stock rainwater barrel 30124 (`mistmyst_barrel_water`), whose genuine native cache has Search operation 1. Storage barrel 362 has no native Search action and is no longer a coolant fixture. Four coolant corners (10421,135), (10421,138), (10428,135), (10428,138) keep central cardinal/diagonal escapes clear. Obsolete lamps/stump inside hazard fields are removed or moved to perimeter recesses.

The shield eligibility rule is preserved, but automatic melee retries no longer flood chat with identical shell-denial messages. Initial instructions and phase/coolant cues remain. The first visible warning explains heat; subsequent waves use the actual animated telegraphs. The displayed aggressive dagger style (Combat Options → Lunge) trains Strength through ordinary combat, with no injected levels or XP.


## Fallen Cinderworks integration additions

Review the scoped `tower_scene` observer hook in runtime synchronization and the personal guardian visibility filter separately from the content palette. Keep standard zone baseline ordering, clone-before-send semantics, actor-specific Run selection and generation-safe NPC synchronization. Decorative variants are deliberately operation-free and collision-identical. Do not apply one player's restoration to the global Locations registry or another player's progress.

No schema migration is added by the environmental pass. The previously recorded Tower V12 / Agent 2 chemistry V12 collision still requires renumbering at eventual integration; preserve Agent 2 chemistry/save projections. No broad upstream merge is performed. Client validation is now ten commits, with an exact byte-preserved roundtrip; apply using `git am --keep-cr`. Private cache, database, credentials, native frames and raw logs remain excluded.
