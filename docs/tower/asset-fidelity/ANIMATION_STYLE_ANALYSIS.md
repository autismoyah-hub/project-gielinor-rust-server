# Animation analysis and authoring

The reference corpus exercises 19 classic action sets: idle and walk on Goblin655,
Man3106, Skeleton70, Zombie26, Chicken1173, Cow2790, Rat2854 and Spider3019;
plus animated objects 5589,11010,13560. Exact sequence IDs and sample counts are
in [ANIMATION_VALIDATION_SUMMARY.json](ANIMATION_VALIDATION_SUMMARY.json).
The prior goblin attack/hit/death and masked-blend evidence remains preserved.

Classic skeletons are ordered operations on vertex/face label groups. Pivots are
computed from currently transformed vertices. A conventional Blender armature
does not reproduce these semantics automatically. The workbench preserves
actual Blender action channels, sequence cadence and masks, then writes new
classic skeleton/frame/sequence payloads. Original and converted native poses,
alpha and pixels agree at 335 matching timestamps.

The Python transform tracer is a diagnostic and authoring compiler, not a renderer.
It is checked against native Rust at those same timestamps. Native float32 values
must be compared as float32: shortest decimal serialization otherwise introduced
apparent sub-millionth-unit differences on scaled rat poses. No renderer change
was made to hide these differences.

For independent work, explicit joint origins and group ownership are compiled to
native centroid-compensating pivot offsets. Parent transformations propagate to
child joint control points deliberately. Contact, anticipation, swing arcs,
recovery and termination still require action-specific comparison. Matching total
duration alone does not establish choreography. Selected landmark trajectories
require supplied semantic selectors and record both absolute and bind-offset-
removed motion errors.

Cached weighted animations can be decoded/rendered for inspection where available;
their Blender authoring/export remains unsupported. Audio events and non-rendering
sequence fields are not certified by geometry/motion round-trip tests.
