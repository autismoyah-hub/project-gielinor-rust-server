# Modeling analysis and actionable diagnostics

Exact decoded records include counts, topology, coordinates, HSL face colors,
texture/render/priority/alpha/depth metadata, model assembly and skin labels.
Calculated records include indexed/welded connectivity, areas, edge incidence,
dihedral estimates, mirror coverage, principal axes, profile envelopes and
area-weighted palettes. Semantic regions, material names, joints and intended
features are curated annotations; they are never automatically asserted as facts.

Two measurement failures materially changed the toolkit. Nearest-vertex height
bands alias small collars into large plates; triangle-plane cross sections now
measure the envelope. Decoded bounds can include invisible helper geometry.
Native inspection records submitted-face indices from the actual lighting result
(`light[face][2] != -2`), and analysis separately computes that surface's bounds
and profiles. Exact decoded geometry remains intact. Submitted surfaces include
translucency and do not claim camera visibility, occlusion or semantic identity.

Object 2097 has 12 alpha-255 faces that inflate bounds. The failed descriptor fit
was oversized even though the visible anvil was small. Orientation-only correction
did not help. Submitted-surface fitting increases mean five-view silhouette overlap
from 0.341 to 0.752; this is development evidence for a general measurement fix.

The component workbench connects errors to owned Blender objects and parameters.
Curated goblin proxies identify excessive head width, torso thickness/height,
equipment attachment, shield-plane orientation and hand/foot coverage. PCA avoids
mislabeling a rotated shield as simply too thick. Proposals are bounded local
affine/rotation/material edits and preserve untouched component hashes. Region
centroids are not joints, and different proxy coverage can mislead an optimizer.

Archetypes remain limited. Low-dimensional bounds/profile/palette descriptors
lose spiral construction, joinery, species anatomy, facial planes and material
placement. A high box overlap does not reproduce a crate's planks. The current
cow lacks horns, ears, hooves and convincing coat regions; scimitar curvature and
full helmets/winged creatures need additional authored semantic structure.
These are authoring failures, not reasons to alter global renderer behavior.
