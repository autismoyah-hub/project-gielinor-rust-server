# Calibration results

Work was tested from source checkpoint 2a6e44ebf5e3c671a889dd04d05f1ea815a15d3b
against unchanged client 23a94f75707f0503a9efe610fe99bfce5089b575. The earlier
reference platform checkpoint dc5e2ca9829fb4dddd192bde05ffdcfffc989a47 is preserved.

Technical preservation passes for 35 training variants / 175 native views plus
nine held-out variants / 45 views: **44 variants / 220 views total**. Twenty-six
classic action sets match across 468 unique native timestamps, including pixels,
coordinates and alpha; independent tracing agrees. Adding submitted-surface
analysis leaves all 165 base training native views byte-identical. Fourteen
meaningful boundary/ownership/IK/identity tests pass.

Independent static calibration, mean IoU over five unchanged native cameras:

| Case | Initial | Revised | Interpretation |
|---|---:|---:|---|
| object:7016 | 0.789 | 0.870 | Independent geometry; artistic acceptance false |
| object:6150 | 0.729 | 0.768 | Independent geometry; artistic acceptance false |
| object:5589 | 0.546 | 0.703 | Independent geometry; artistic acceptance false |

The pillar's collar/profile problem and waterwheel rim thickness improve; the
spiral shaft, joinery and material placement remain different. Worn weapon/shield
studies use their own native coordinate frames with matching A/B tests. Generic
cow anatomy is inadequate despite successful native export and five-action playback.

Four bounded goblin edits increase mean static IoU from 0.620 to 0.672. Thirteen
owned components change while 26 untouched component hashes remain exact per
local-edit report. A subsequent fresh-output loop smoke run passes. All retained
edits remain unapproved. The humanoid compiler matches 505 native poses; walk
proxy-arc error improves from 22.44 to 19.82 native units, while independent combat
choreography and visible anatomy remain inadequate.

Failures are preserved: aliased vertex-band pillar, thin-rim wheel, oversized
held-out anvil, ineffective orientation-only anvil correction, generic cow and
straight scimitar. A Blender Python exception once left older outputs available
because the default exit code was zero. That run is excluded; strict
`--python-exit-code 1`, fresh output directories and coverage checks now prevent
that path from reporting success. Tests certify the supported pipeline, not art.
