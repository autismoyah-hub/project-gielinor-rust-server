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
