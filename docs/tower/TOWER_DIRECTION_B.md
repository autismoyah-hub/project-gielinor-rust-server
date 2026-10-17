# Direction B — The Cinderworks

Floor One is a damaged forge sanctuary. The player restores its three services, learns to move through vented heat, defeats the Ash Imp that is feeding the vents, then quenches the Ash Warden and restores the sanctuary. The first visit should take roughly 20–30 minutes; repeat runs emphasize encounter mastery. Timing is a design target, not an acceptance result.

## Route and physical identity

Keep the supported constructed-region publication and real cache assets, but replace the uniform nine-box experience with a clear route: southern arrival court, western workshop and grove, eastern waterworks, northeastern vent gallery and Imp court, northern Warden foundry, then return through the restored central sanctuary. The workshop and waterworks are branches visible from the sanctuary; the Imp and foundry form the escalation route. Remove decorative wall centrepieces that obscure players at normal camera height. Use continuous floor materials, sparse low furnishings, wide doorways and a central landmark. Preserve the river as the waterworks boundary. No decorative objects may obstruct approach tiles for required interactions.

## Progression and services

The plaque is the durable route guide and gives the current next step. Ordinary resource and skill actions remain useful: gather and prepare a small supply of food, forge a bronze weapon, and restore the sanctuary altar. Replace repeated eleven-row bounty grinding with three service objectives, a vent traversal, the Imp, and the Warden. Validate objectives on successful existing world actions, never attempted clicks or inventory presence alone. Replenishable supplies and recovery tools are available through clearly described normal interactions, including after death or abandonment. Preserve legacy diary state explicitly rather than silently treating it as a fresh run.

## Encounter teaching and escalation

The vent gallery teaches a warning followed by a delayed eruption on marked tiles. The safe route is visible in-world and warnings also appear in chat. Leaving the warned tiles avoids damage; there is no hidden random hit. The Ash Imp targets the player's tile, then scatters heat across three tiles. Safe adjacent positions and recovery between eruptions permit ordinary melee. Its defeat grants the durable foundry unlock to eligible participants. No unrelated player receives credit.

The Ash Warden has three health phases: deliberate single-lane strikes, alternating crossed lanes, then alternating diagonal vents with a faster cadence and clear recovery interval. Heat warnings precede damage and distinguish danger from safe lanes. Quench interactions interrupt a charged attack and expose the boss; they are positional normal object interactions, not test commands. Combat, hitpoints, food, movement and damage use existing engine authorities. Arena exit, defeat, logout, disconnect and ownership loss reset transient encounter state while preserving completed services and unlocks. Shared-world ownership is explicit; another player's presence cannot steal credit or leave an encounter permanently occupied.

## Resolution, rewards and replay

Victory restores the sanctuary and enables an explicit claim and exit. First-clear reward eligibility and inventory delivery save in the same character transaction. A full inventory leaves a pending claim; retries cannot duplicate restricted rewards. Replay begins through the plaque, retains first-clear payment history, resets only run-specific state and provides a modest repeatable reward. The upstairs stub is not an ending. A normal-world exit and return interaction must be reachable from both completed and abandoned runs, without re-triggering recovery-pad stripping.

## Implementation order and evidence

1. Native/headless action gateway, observable state and genuine frame capture.
2. Open, readable layout and route objects; review actual native views from arrival and both branches.
3. Durable run state and guide/entry/exit/recovery interactions, then ordinary-action service callbacks.
4. Scheduled vent hazards and Ash Imp lifecycle, then phased Warden and quench interactions.
5. Durable completion, transaction/replay checks, full authentic success/failure/relogin runs and visual polish.

Each checkpoint continues into the next. This document records the intended complete experience; unimplemented behavior must not be described as shipped.

Implemented and accepted; see TOWER_ACCEPTANCE.md. All warning windows allow four server ticks; final Warden cadence accelerates independently of that readable reaction interval.

## Visual reconstruction supersedes the open-court prototype

The current production pass replaces the open nine-court composition with enclosed service rooms, a clipped-corner sanctuary, a processional hall and a tighter fortified foundry with vestibule. The vent gallery uses lighter slate, while the final foundry uses charcoal ash. Precise JS5-only conveyor-frame deletions prevent inherited industrial remnants from reappearing. Obsolete mind altar, vial/staff crates, goats, goblin training and upper stub scenery are removed.

The Ash Imp retains its small red silhouette. The Warden receives a distinct spiked fire-giant body through a normal NPC transform, with compatible fire-giant movement, charge, defend and death sequences. Authoritative identity and combat/XP metadata remain unchanged. Coolant adds visible steam/flinch feedback, and phase changes flare at the boss. The inside portal is a real wall opening at (10432,124); the plaque is a supported wall decoration. User artistic acceptance remains pending.

### Current readability and interaction refinement

Foreground southern room edges use supported low stone wall 5154 through `prepare_palette`; exterior, back walls and plaque support remain full-height 2341. Movement stays blocked, while range policy follows the actual cache asset. Native comparison at the foundry threshold shows the avatar remains visible. No renderer or input-readiness bypass is used.

Coolant uses stock rainwater barrel 30124 (`mistmyst_barrel_water`), whose genuine native cache has Search operation 1. Storage barrel 362 has no native Search action and is no longer a coolant fixture. Four coolant corners (10421,135), (10421,138), (10428,135), (10428,138) keep central cardinal/diagonal escapes clear. Obsolete lamps/stump inside hazard fields are removed or moved to perimeter recesses.

The shield eligibility rule is preserved, but automatic melee retries no longer flood chat with identical shell-denial messages. Initial instructions and phase/coolant cues remain. The first visible warning explains heat; subsequent waves use the actual animated telegraphs. The displayed aggressive dagger style (Combat Options → Lunge) trains Strength through ordinary combat, with no injected levels or XP.
