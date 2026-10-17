## Current guarded waterworks and furnace composition

Implementation `c7e4230b897d93ba2cb83fd7b9a60eaf16097ffc`, executable `6D94F65F360C95B789506EDCBCCA03F416A3433DC4EA15218A566E9934ABA5A9`. Floor origin remains 10368,64. The four existing passive3317 fish NPCs stand at local72,50/52/54/56; ordinary net fishing reaches them from flat dry local70,50/52/54/56. The wheel landmark moves to local72,58 and retains Water state/angle0/dormant18512/restored18511. Continuous low masonry on local70,z48..63 and two endcaps protect the height seam; ordinary through-crossings are local69/70,z48/64 in both shell and opening declarations. Quay chunks use actual377,545 rotations0/1; genuine water uses187,566. No terrain-byte or interaction-reach override.

Middle Foundry landmark local55,78 now uses18526/18525 angle1, matching the two neighboring rotated forge mouths; the extra mouth tile has a real shell opening. Piers local54/59,78 and boiler connections local69/74,76 remain beyond combat bounds. Imp and Warden bounds, coolant coordinates, authoritative encounter rules, service flags, inventory, rewards and persistence format are unchanged. Current victory palette is18526,18511,16811,16811,18525,18525,18525,18525. See `WATERLINE_ART_AND_LIFECYCLE_20261017.json` for exact evidence scope and preserved failures. Older coordinate/palette statements below describe historical checkpoints.

# Tower architecture

Continue from the pinned engine baseline using its authoritative world simulation, inventory, combat, networking and transactional account saves. Do not introduce competing engine authorities.

Prefer Tower-specific modules for floor definitions, run lifecycle, objective rules, encounter decisions, hazard patterns, presentation and reward policy. Shared-engine hooks may be implemented in this clone when necessary; record each in [TOWER_ENGINE_DELTAS.md](TOWER_ENGINE_DELTAS.md).

The current constructed region is shared while progress is personal. Each guardian court has one ActorId/generation/run owner; authoritative hero attribution grants only that character credit. Leaving, disconnecting or defeat resets transient ownership and health. Do not imply per-party instances or cooperative credit already exist.

Confirmed successful gameplay outcomes should advance objectives. Enemy death credit must use authoritative attribution. Hazards must use world ticks and validated movement/damage. Completion and reward claims must commit consistently with inventory and account state. Unknown historical saves must remain distinct from known fresh profiles.

Use the same implementation for native play and deterministic headless scenarios. Tests must cover rejected and interrupted actions, actor identity, death, restart, full inventory, logout/relogin and replay. A helper test or screenshot alone does not establish gameplay acceptance.

## Implemented modules and lifecycle

- `tower_run.rs` and V12 `tower_run` store personal services, Imp unlock, victory/pending claim, paid history and clear count separately from the legacy diary.
- `tower_layout`, server `tower_map`, `tower_cinderworks`, `simulation/cinderworks_actions` and `cinderworks_work` publish the authored world and route successful existing actions into its services.
- `cinderworks_encounter` and `simulation/cinderworks_encounter_actions` decide deterministic patterns on world ticks, then use existing zone graphics, damage and combat authorities.
- `simulation/cinderworks_recovery` implements bounded sanctuary rescue with ordinary animation, movement and stat authority; `cinderworks_checkpoint` stages durable proposals without exposing reward items early. Runtime and the existing single account worker own save submission/ACK/retry; no separate persistence engine exists.
- The optional client adapter uses normal `ClientState` actions/packets in both native and headless modes. Receipt retries cannot redispatch actions; native frames come directly from the rendered raster. Read-only observations and file-lock diagnostics never mutate authoritative gameplay state.

The complete accepted paths and exact integration boundary are in TOWER_ACCEPTANCE.md.

## Current visual production reconstruction

The previous open-court architecture is superseded by masonry rooms, a clipped-corner sanctuary, a southern hall and fortified northern foundry. `tower_layout::shell` is the shared scene/collision source; wall shapes 0/2 use stock wall 2341. `prepare_collision` now precedes wall application. `Shell::stamp_shaped` supports real wall decorations without destroying their supporting wall. The plaque remains (10422,115), now shape 4. The wall portal moves to (10432,124), shape 0.

The server game's map and native JS5 map differ for a narrow conveyor-decoration set. A native scene inventory identified the exact remaining ids and coordinates. `tower_map::prepare` registers that client-only baseline so ordinary persistent zone deletes remove it; final geometry is still published through the existing constructed registry.

The Warden retains authoritative type 16295 and the same run, combat, reward and owner rules. Standard initial NPC transform/name/level masks present cache type 2075 as Ash Warden. Cache fire-giant sequences provide compatible presentation; no Tower renderer exists. The native validation adapter's `inspect_scene` is read-only provenance, opt-in and local-only.

### Current readability and interaction refinement

Foreground southern room edges use supported low stone wall 5154 through `prepare_palette`; exterior, back walls and plaque support remain full-height 2341. Movement stays blocked, while range policy follows the actual cache asset. Native comparison at the foundry threshold shows the avatar remains visible. No renderer or input-readiness bypass is used.

Coolant uses stock rainwater barrel 30124 (`mistmyst_barrel_water`), whose genuine native cache has Search operation 1. Storage barrel 362 has no native Search action and is no longer a coolant fixture. Four coolant corners (10421,135), (10421,138), (10428,135), (10428,138) keep central cardinal/diagonal escapes clear. Obsolete lamps/stump inside hazard fields are removed or moved to perimeter recesses.

The shield eligibility rule is preserved, but automatic melee retries no longer flood chat with identical shell-denial messages. Initial instructions and phase/coolant cues remain. The first visible warning explains heat; subsequent waves use the actual animated telegraphs. The displayed aggressive dagger style (Combat Options → Lunge) trains Strength through ordinary combat, with no injected levels or XP.


## Authoritative environmental response

The new `tower_scenery` palette and `tower_scene::Observer` derive personal decorative views from the existing durable Run. There is no second set of restoration flags or client-side quest state. Inactive/active cache counterparts must match width, length, movement/range clipping and routing behavior, and expose no unimplemented operation. Canonical collision therefore stays authoritative for every player. Normal interactive scenery is never visually replaced with an operation-incompatible ID.

Baseline zone snapshots precede the per-restorer decorative additions. A Run change refreshes visible landmarks; zone reset or region rebuild refreshes affected landmarks even when Run is unchanged. Relog initializes a new observer from the saved Run. Replay resets service fixtures and guardian presentation while the first-clear beacon retains paid history. NPC death animation remains visible before a credited restorer stops receiving the healthy shared respawn. This uses ordinary NPC synchronization rather than a client renderer exception.
