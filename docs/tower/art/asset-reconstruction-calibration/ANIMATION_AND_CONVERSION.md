# Deliberate classic-animation translation

Revision 239 supports two paths in this client. Classic sequences identify frames
as `(archive-0 group << 16) | file`. Those frames reference archive-1 skeletons.
Skeleton entries describe ordered operations and lists of vertex/face labels.
The selected goblin uses this classic path. Cached animations instead use
archive 22 curves and skeleton bone matrices with weighted skinning. A conventional
Blender armature is not a replacement for either representation.

## What passes through Blender

Native X,Y,Z map to Blender X,Z,-Y at 128 native units per tile. This mapping has
positive determinant. Round-trip mesh vertices are read back from the actual
Blender mesh; no merging, retessellation or normal recalculation is allowed.
Triangles retain their vertex order. Native packed HSL values are integer material
properties, avoiding a lossy HSL-to-RGB-to-HSL conversion. Face-domain integer
attributes preserve shading type, priority, alpha, face labels and depth flags.
Blender vertex groups preserve the classic one-label-per-vertex assignments.

Animation imports create a named ordered-transform control rig and genuine Blender
actions. Control channels store the three native values for each operation.
Constant keyframes reflect classic stepped playback. Export samples evaluated
Blender action channels. The rig is intentionally not represented as an ordinary
parented armature. This version does not offer a native-equivalent deformed mesh
preview inside Blender; native rendering remains the appearance authority.

Native operations are: 0 compute a current-vertex centroid pivot plus offset;
1 translate; 2 rotate around that pivot in Z-X-Y order; 3 scale around the pivot;
5 alter alpha on face-label groups. Native integer pivot averaging, trig tables,
rounding and operation order matter. The export preserves the active operation
list, including implicit pivot entries exposed by the decoder. A new skeleton
and frame payload is encoded from those channels, not copied from original bytes.

Blender source is saved, reopened in a fresh headless process, and exported again.
An edit witness changes one actual vertex, a material's HSL property and a walking
action key. All three changes appear in the resulting interchange. This guards
against an exporter merely returning its original JSON inputs.

The diagnostic model codec writes SkinnedLegacy data with explicit skin labels,
per-face priorities and optional face-depth data. New payloads are independently
decoded by the existing native Rust model decoder. When per-face priorities are
present, the format has no meaningful separate global priority: the source's
unused global fallback can differ after decoding. All effective face priorities
and every compared native pixel agree. Native coordinates are integral for these
references; the actual round-trip error is zero native units. Non-integral input
is rejected rather than silently quantized.

## Identifiers and native execution

Round-trip models use 1100200/1100201, object 63200 and NPC 60000.
Independent models use 1100202/1100203, object 63201 and NPC 60001.
Both live in separate diagnostic packages, not the active asset overlay.
Sequence IDs are 36181,36180,36185,36183,36182. Round-trip frame groups are
60000..60004 with skeletons 64000..64004. The independent rig uses frame groups
60100..60104 and skeletons 64100..64104. Reuse of sequence IDs is confined to
separate standalone diagnostic packages; these are never simultaneously installed.

The native diagnostic loader decodes custom NPC/object definitions, checks model,
object, NPC, skeleton and frame ID collisions against the source cache, and binds
the new data in its private process. It does not add an NPC/animation overlay to
the ordinary client cache. Native `Clock`, `Sequence`, `Frame`, `Skeleton`,
`Model::animate_classic`, mask operations, lighting and software rasterization
are used. Original A uses `Animations::apply`/`apply_blended`. Converted B uses
newly decoded custom frames with the same native deformation operators.

Static lighting is evaluated before actor deformation, matching NPC assembly.
Masked primary action and secondary walking passes use the native mask operator
in native order. Independent actions share reference clip durations, but their
poses are newly authored and are not claimed identical.

## Supported scope

This is a reusable proof for untextured classic assets. The old opaque static GLB
converter remains unchanged. The new diagnostic interchange is not a claim of
general GLB armature support. Weighted cached rigs, textures and live registration
need their own controlled tests before production adoption. No live session was
started and no game account, actor or world was altered.
