# Reusable owned construction

`tools/tower/asset-workbench` remains the CLI/tool root. `shape_families.py`
implements tapered 3D swept sections with a stable cross frame, closed stave
sector shells, radial voussoirs and chamfered/worn blocks. Integer native
coordinates are intentional: 128 units/tile; X right, Y down, forward -Z.

`construction.py` composes planks, battens, bands, rivets, curved diamond-section
blades, clipped shield boards/back framing/grip, anvils with a shaped horn,
masonry, radial arches, trestle benches, chests, picks and hammers. The barrel
uses separate bowed staves, hoops, head and bung. Wood end surfaces and edge
roles are described separately from lateral plank faces.

`creature_families.py` has separate wolf, bear and bovine parameter paths plus
a winged bat with membranes/ribs. Longitudinal torso/neck/muzzle, feet/claws,
ears/jaw/tail and explicit limb ownership replace one generic box quadruped.
`humanoid_families.py` adds face planes, lapped armor, a grip socket and biped
controls. Its goblin/imp results remain unapproved; new parts do not establish
convincing anatomy. No reference mesh is copied into these families.

`fit_semantic.py` creates a private development recipe from an explicit class,
low-dimensional dimensions and palette observations. It does not read the
original vertex/face payload to author candidate topology. Descriptor fitting
is disclosed, so these studies are not blind recognition. Curated bat skin
bounds exclude specific helper/shadow groups; that binding is private and is
not a universal rule about native label numbers.

`production_designs.py` exercises the libraries in four genuinely original
recipes: arcade, hoist, Kilnback and supporting terrain. Each component has
role/material/group metadata. `author_blender.py` builds editable owned meshes,
saves/reopens them and exports the actual edited data. It rejects reference
meshes masquerading as owned components. Blender custom controls author classic
groups; native poses are the authoritative deformed preview.

Opt-in `geometry_validation.remove_zero_area_faces` cleans triangles collapsed
by integer quantization before owned import. `mesh_validation.py` preserves
vertex order, rejects wholly collapsed components and reports removals. Edited
meshes with new degeneracies fail export rather than being silently rewritten.
Reference interchange never uses this cleanup. Existing classic vertex/group
identity and all 105 final Kilnback animation payload files remain unchanged.

Corners, terminations and repaired structures can be composed from these
components; there is no approved comprehensive architecture kit or universal
automatic wear generator. The original arcade includes a repair tie and modular
guide rails. Final production proportions still require intentional art decisions.
