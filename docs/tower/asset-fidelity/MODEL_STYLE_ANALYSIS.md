# Geometric and material analysis

[MODEL_STYLE_DATA.json](MODEL_STYLE_DATA.json) groups training measurements by
asset family. These are observations of a small source-backed sample, not hard
limits defining all OSRS art. Role, shape variant and assembly matter.

The analyzer separates decoded fields from calculated descriptors: bounds,
coordinate/triangle counts, face flags, alpha, priorities, skin labels, texture
presence; then connectivity with and without coordinate welding, edge incidence,
triangle-area quantiles, dihedral-angle distributions, symmetry coverage, surface
centroid estimates and palette distributions. A surface centroid is not physical
mass. Dihedral changes describe geometry, not automatically intentional hard edges.

Native vertex seams, packed HSL and face metadata must survive technical round
trips. Materials are not inferred from RGB screenshots. Textured-face color codes
can be lighting placeholders and must not be interpreted as texture albedo.
Neither manifoldness nor a particular polygon count proves artistic fidelity.

Character component labels require explicit curated selectors. Native group
centroids are geometric proxies, not guaranteed anatomical joints. Authored
component names and parameter links allow a diagnostic to point to the actual
Blender part. PCA plane estimates help distinguish a rotated shield from one that
merely needs more depth. Material-region descriptors can expose missing color
placement, but visual review remains necessary.
