# ASCENT controlled asset reconstruction calibration

Floor production remains suspended. This study separates technical conversion
from independently authored likeness. Neither independent reconstruction is
approved Tower art, and no production map, cache identifier or account is changed.

The private review is at
`experiments/custom-model-pipeline/asset-calibration/final/review/index.html`
in the parent development workspace. The repository contains tools and aggregate
measurements only. Extracted models, animation payloads, screenshots of originals,
Blender derivatives, independently reconstructed game assets and their specific
authoring recipe remain outside Git.

## Findings

| Stage | Evidence | Decision |
|---|---|---|
| 1: static Blender round trip | Wall 6098, shape 0, model 5884; 90 vertices, 114 faces; three cameras and four draw orientations | Technical PASS: every compared pixel and meaningful decoded channel agrees |
| 2: independently authored static object | New profile, topology and moldings; matched native views, three-way native scene and authentic human scale comparison | Rendered and measured; artistic approval is not asserted |
| 3: NPC static geometry round trip | Goblin 655, eight original component models assembled by native NPC code; 936 vertices, 1,910 faces | Technical PASS: geometry, topology, colors, skin labels, per-face priorities and depth flags preserved |
| 4: NPC animation round trip | Five classic sequences; native clock, native frame/skeleton decoders, native integer deformation and rasterizer; masked attack/walking blend | Technical PASS for this bounded classic reference; 560 matched renders including bind/rotation views agree pixel-for-pixel |
| 5: independent creature | New anatomy/equipment, a compact classic vertex-group rig, an authored ordered-transform rig and five Blender actions, plus a masked blend | New geometry and motion work natively; likeness and choreography differ and need artistic review |

The results do **not** certify the full art pipeline or authorize Floor One
production. Exact conversion is separate from visual authorship. In particular,
the independent wall's molding/normal distribution and the creature's armor,
anatomy and motion differ from the originals. Those differences must not be
hidden by changing the reference camera or lighting.

## References and control conditions

The static reference is the shape-0 composition of object 6098, not all of that
definition's shape variants. Its other model IDs are 5885 and 5886. The selected
composition is untextured. The native object factory applies definition transforms
before export; the custom model contains that evaluated geometry.

Goblin 655 uses models 24433, 24443, 24450, 24464, 24476, 24484, 24502 and
24506. Native assembly/recoloring precedes Blender export. Its scale is 128/128,
ambient offset 15 and contrast offset 0. Clips are idle 6181, walk 6180,
armed attack 6185, hit 6183 and death 6182. Idle and walk come from the NPC
definition. Combat clips are selected from the matching local symbol family and
their actual dependencies are decoded. This does not verify live combat AI linkage.

The initial survey's older goblin combat clips and an intermediate unarmed attack
are retained privately as rejected reference selections, not ground truth.

Every individual view uses the same 700 x 650 native framebuffer, focal length
850, camera distance 630, eye height -340, target height -95, palette brightness
0.8 and light direction [-30,-50,-30]. Object light is 64/768, NPC light 79/850.
These are controlled asset-inspection cameras, not claimed normal gameplay cameras.
The native three-way scene uses a wider shared view with fixed placements.
Per-panel views are the exact camera comparison; perspective differs across
placements within the single scene. Original Man 3106 supplies an authentic
static human-scale reference, not an animated live player.

Original cache access is explicit, read-only and CRC/revision checked. The existing
offline legacy suffix adapter remains unchanged. All 50 protected cache files
(453,219,756 bytes) retain their original SHA-256 values. Protected server/client
heads remain f9e4433f2451235805f85fe379d351d095b7535b and
ff9cef17fd1ce3ec2ebf8d1181fee31411999c4c.

## Evidence and limits

`SOURCE_SAFE_SUMMARY.json` contains measurements and status without geometry or
reference pixels. The private `diagnostics.json` adds input hashes and temporal
measurements. The exact oracle compares posed vertices and alpha as well as
pixels; silhouettes alone are insufficient. Independent mask comparisons are
diagnostic measurements, not artistic scores.

Native classic clips use discrete frames. Samples cover every tick through 256,
all frame boundaries and the final timestamp; the death clip's remaining long
terminal hold is sampled at its end. Animation clips have durations 63, 48, 36,
95 and 390 ticks respectively. At the client's 20 ms tick these are 1.26, 0.96,
0.72, 1.90 and 7.80 seconds. The masked blend has 49 timestamp samples.
Review GIFs contain native raster frames and repeat for inspection. They are
neither the animation implementation nor substituted sprite assets.

The bound tests do not establish cached weighted-skeleton conversion, textures,
complex texture mappings, audio events, live actor lifecycle transitions, network
movement or equivalence with another client's renderer. The current animation
tool deliberately rejects cached animation selection and unsupported weighted or
textured model encoding. Native rendering code and ORACLE's branch are unchanged.

Reconstruction-specific authoring code remains private alongside the derivatives;
the general import/export, codec, native diagnostics and comparison tools are
published. No claim of a right to redistribute Jagex's reconstructed assets is made.
