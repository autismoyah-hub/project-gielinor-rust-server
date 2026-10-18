# Contact-aware classic animation

The existing client-compatible group runtime is retained. Native classic
transform ordering, centroid pivots, integer translations, Z-X-Y rotations,
8-bit angle channels and strict clock boundaries remain authoritative.
`motion_authoring.py` solves two-bone joint targets, applies authored bend
preferences, refines integer controls and cancels ankle world orientation.
Parent/limb ownership and guide controls are explicit. No new game runtime is
introduced. Saved/reopened Blender actions are genuine editable control channels;
they are not a conventional armature claiming live viewport/native equivalence.

Native original choreography observations remain in the preserved motion review
and ANIMATION_STYLE_ANALYSIS.md: 26 unique source action sets, 468 matching
timestamps. Head/torso/foot/hand/equipment proxies compare arcs, coordination,
recovery and cadence at identical timestamps without time warp. Previous goblin
attack/hit/death and independent equipment remain visibly different. Matching
duration or technical frame reconstruction does not establish choreography.

The new Kilnback uses six articulated three-joint legs with actual foot-pad
support correction. Stance/swing goals, stride, lift and crouch bake to classic
controls. `support_surface_iterations` is opt-in; the old analytic-only variant
is preserved. Pad correction changes mean drift from 2.234 to 2.599 native units
but eliminates its 14 floating stance observations and five support failures.
The corrected walk's maximum drift is 7.672: sliding is not solved.

| Original action | Native duration / samples | Authored intent |
|---|---|---|
| idle | 60 / 61 | Breathing/body movement, planted feet |
| walk | 48 / 49 | Alternating tripod stance/swing with explicit travel |
| attack | 42 / 43 | Anticipation 0–12, execution 12–22, recovery 22–42 |
| hit | 30 / 31 | Fast recoil and early neutral recovery |
| death | 84 + 90 hold / 175 | Grounded persistent visible corpse |
| walk_entry | 8 / 9 | Explicit idle tick 0 to walk tick 0 bridge |

All 368 poses agree with the independent native integer tracer. Attack/hit
recover to neutral. The bridge's two endpoint meshes and pixels are exact.
`bridge_controls` evaluates already-authored source clips; it is an explicit
authoring operation, not arbitrary client runtime blending. Death no longer
scales the creature to zero. Grounded final geometry is evidence of a visible
terminal state, not actor despawn/event/physical collision behavior.

Contact diagnostics use actual exported native frame phase and declared world
travel -1.008064516 Z units/tick. Across 49 poses and 189 stance observations:
mean drift 2.598552, max 7.671998; zero sampled penetration, floating stance feet,
wholly unsupported frames and body-centroid-proxy outside support. Tolerance is
3 native units. The previous creature, stress-tested under that same *assumed*
travel/stance protocol, measured 12.775/32.304 and had failures. The old protocol
is not inferred historical gameplay contact state.

`contact_diagnostics.py` emits CSVs, action recovery/terminal reports, rigid
component edge-length errors and contact sheets. It does not infer mass or
perform triangle-triangle equipment/self-collision. `owned_motion_review.py`
labels full native BMP sequences and old/new key poses at matching timestamps;
no interpolation, retiming or image fitting is used. Scene playback additionally
shows walk, attack and death beside authentic native scenery.

Weighted cached animation authoring, audio/events, arbitrary blend transfer,
comprehensive collision and artistic acceptance remain unsupported/open.
