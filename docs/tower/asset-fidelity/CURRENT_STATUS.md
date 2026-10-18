# Asset authoring and fidelity workbench

Floor production and gameplay integration remain suspended. The previous
calibration checkpoint 75e611a4a97c7141c837de46102766711e9cdf22 is preserved.
Work continues on `experiment/asset-fidelity-workbench`, using unchanged client
source 23a94f75707f0503a9efe610fe99bfce5089b575.

The reference platform now inspects 33 training assets: 19 objects, eight NPCs
and six equipment variants. All 33 survive a saved/reopened Blender round trip
with pixel-identical native results across 165 views. Three cases preserve raw
models plus native definition transforms rather than rounding fractional evaluated
vertices. Nineteen classic action sets match across 335 native timestamps. The
independent classic transform tracer also matches those 335 native samples.

These results establish technical preservation, not independent artistic fidelity.
Nine held-out assets are declared separately and are not used in training metrics.
The goblin remains the first independent authoring target. Component diagnostics,
material-region diagnostics and explicit joint-to-native-group authoring are under
development; held-out and original-asset results must be recorded separately.

| Milestone | Current evidence |
|---|---|
| 1: reference analysis | Tested batch inspection and numeric reports; native views and classic motion |
| 2: modeling toolkit | Active development; owned components, sockets, saved Blender edits and joint compiler |
| 3: independent static fidelity | Pending controlled comparative iterations |
| 4: independent animation fidelity | Technical preservation passes; independent choreography remains unapproved |
| 5: generalization | Predeclared held-out cohort; evaluation pending |
| 6: original Tower assets | Pending bounded authoring studies; no production promotion |
| 7: ATELIER handoff | Commands and limits being documented; not production certification |

See [calibration results](CALIBRATION_RESULTS.md), [reference catalog](REFERENCE_CATALOG.md),
[architecture](PIPELINE_ARCHITECTURE.md) and [handoff](ATELIER_HANDOFF.md).
Raw reference meshes, Blender derivatives, cache imagery, specific reconstruction
recipes and animation payloads stay outside Git in the private workbench output.
