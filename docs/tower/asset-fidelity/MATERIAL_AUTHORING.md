# Semantic native materials

`semantic_materials.py` shares material families and component roles with Blender
authoring and native export: wood, stone, metal, bronze, organic, ceramic and
leather. Recipes describe native packed HSL relationships for broad surfaces,
end grain, edges, shaded undersides and controlled per-component variations.
Face roles depend on component axes and geometry, not unrelated random colors.
Saved Blender face attributes preserve actual native color/alpha/priority data.

These are native face-color descriptions, not claims that Blender PBR materials
match the software client. Native texture mapping and palette lighting remain
distinct paths. Geometry is simplified deliberately; material boundaries help
planks, hardware, masonry courses, shells and body parts read at gameplay scale.

The revision 239.1 barrel object 362 uses texture 16. A read-only native
`texture-stats 16` observation yields mean RGB approximately (101.064,68.656,13.884)
and nearest palette code 6934. Only those aggregate observations guide the
owned palette. No reference texture pixels, grain image, UV derivative or mesh
are published or copied. Broad bands cannot reproduce all textured grain.

Frozen tests exposed gray tool shafts and a red bear nose: unconstrained palette
matching selected semantically unrelated regions. Later development restricts
wood to warm chromatic candidates, uses area-dominant body color and a neutral
keratin/eye treatment. Those changes do not replace frozen results. Palette
inference remains imperfect for unannotated multi-material targets.

The original hoist has bowed wooden staves, metal bearings/ratchet/pawl and
counterweights. The arcade uses divided masonry, keystone and iron repair ties.
The Kilnback separates dark organic anatomy, fired ceramic plates and bronze
mandibles/claws. Water is a simple authored flat native face color, not animated
liquid or a physically based shader. Context imagery is protected local evidence.
