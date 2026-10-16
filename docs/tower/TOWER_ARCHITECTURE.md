# Tower architecture

Continue from the pinned engine baseline using its authoritative world simulation, inventory, combat, networking and transactional account saves. Do not introduce competing engine authorities.

Prefer Tower-specific modules for floor definitions, run lifecycle, objective rules, encounter decisions, hazard patterns, presentation and reward policy. Shared-engine hooks may be implemented in this clone when necessary; record each in [TOWER_ENGINE_DELTAS.md](TOWER_ENGINE_DELTAS.md).

The current constructed region is shared while progress is personal. Initial work must define encounter participation and reset ownership explicitly. Do not imply per-party instances or cooperative credit already exist.

Confirmed successful gameplay outcomes should advance objectives. Enemy death credit must use authoritative attribution. Hazards must use world ticks and validated movement/damage. Completion and reward claims must commit consistently with inventory and account state. Unknown historical saves must remain distinct from known fresh profiles.

Use the same implementation for native play and deterministic headless scenarios. Tests must cover rejected and interrupted actions, actor identity, death, restart, full inventory, logout/relogin and replay. A helper test or screenshot alone does not establish gameplay acceptance.
