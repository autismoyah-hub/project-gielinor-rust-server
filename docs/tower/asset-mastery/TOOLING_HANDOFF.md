# Advanced tooling: reproducible entry points

Use the setup in ../asset-fidelity/ATELIER_HANDOFF.md: sibling server/client
checkouts, Python + Pillow, Blender 5.2.2 and the actual offline Rust renderer.
The original cache is read-only. Reference derivatives and examination outputs
must stay in a private directory outside source and cache. T below is
tools/tower/asset-workbench. Substitute explicit local paths for capitalized
arguments. The preserved identity allocator is single-writer.

## Owned geometry and materials

primitives.make now accepts section_mesh, shell and gear in addition to the
previous forms. section_mesh uses consistent authored section/ring vertex
correspondence with optional per-ring profiles and hem offsets. shell makes
closed fitted armor surfaces with explicit thickness and optional open angular
range. gear makes a real toothed profile with an optional bore. All return real
native-unit geometry for Blender. No screenshot geometry or alternate renderer
is involved. advanced_geometry.topology reports winding and boundary problems.

A deformation_cage supplies bounds and eight XYZ corner offsets in bit order
(X bit 0, Y bit 1, Z bit 2). A sampled Jacobian rejects local foldovers before
integer quantization; this is a conservative authoring guard, not proof against
every possible fold between samples. Owned degenerate triangle cleanup remains
explicit and preserves vertex order. Source/reference interchange is unchanged.

Native materials can opt into native_texture_id, texture_lightness and either
explicit native_texture_triangles/native_texture_coords or a component-owned
native_uv_projection with two coordinate axes. Mapping vertices and skin arrays
remain consistent through saved/reopened Blender. Type-0 mapping is supported;
complex cached mappings and independent weighted-bone encoding are not. Actual
texture resources are read locally by the native renderer, never copied by the
authoring tools. Reference-textured study imagery is protected, even if its
geometry is independently constructed. Per-face fallback can produce bad grain;
the native comparison must judge the projection. PBR shaders are not authoritative.

## Rigging and motion

mastery_creatures supplies parameterized biped, winged_biped, segmented_crawler
and refined_quadruped constructors. They use named body landmarks, native group
ownership and the existing classic compiler. The biped has fitted shells, hands,
named equipment ownership and seven authored clips including a second attack
and walk-entry bridge. The crawler uses a serial axial hierarchy and declares
body contact. Wings have independent owned groups and secondary classic motion.
These are unapproved family exercises, not faithful reference bosses by default.

rig_contracts validates parent-first ownership, pivots, attachment components
and optional native-angle limits. advanced_motion reviews actual native pose
landmarks, extension/recovery, cadence, static steps and declared capsule pairs.
Capsule overlap is a review proxy, not a mesh collision assertion. Repeated
native ticks can be normal cadence/terminal hold, not automatically ineffective
motion. Explicit biped support_weight_transfer bakes authored double-support
weight shifts through the same IK/classic path. sole_footprints support mode
uses the actual pad polygon; historical centroid results remain preserved.

```text
python T/mastery_construction.py PRIVATE/original-recipes
python T/mastery_creatures.py PRIVATE/creature-recipes
python T/author_workflow.py --recipe ORIGINAL_RECIPE --out FRESH --native EXE --cache CACHE --blender BLENDER --bounds BOUNDS --identities IDENTITIES --asset-key UNIQUE_KEY
python T/contact_diagnostics.py FRESH PRIVATE/contact-review
python T/advanced_motion.py FRESH PRIVATE/explicit-landmarks.json PRIVATE/advanced-review
python T/owned_motion_review.py FRESH PRIVATE/motion-previews
```

Native render scales are explicit: a private raw-coordinate reconstruction can
set native_render_scale to three positive native definition values and use an
explicit reference camera. Static renders, classic playback tracing and object
package definitions apply the same scale. Standalone original designs remain
in native units at scale 128; global scaling does not conceal modeling errors.
NPC bind lighting precedes coordinate scaling, while the native object factory
scales before lighting. Nonuniformly scaled object registration therefore need
not match NPC pixels, even with exact world coordinates. Preserve that native
factory distinction; do not change the renderer to make the comparison pass.

## Actionable iteration and private reconstruction

component_feedback takes source/candidate geometry plus curator-approved semantic
bindings and names proportion, position, material and construction defects.
guided_refinement runs targeted real Blender edits, fixed native views and
preserved candidate/history directories. Pareto retention across views and RGB
is a declared development policy, never artistic approval. A rejected edit is
preserved and stops that proposal chain.

```text
python T/component_feedback.py PRIVATE/A.json PRIVATE/C.json PRIVATE/bindings.json PRIVATE/feedback.json
python T/guided_refinement.py --baseline PRIVATE/C --reference PRIVATE/A --bindings PRIVATE/bindings.json --out PRIVATE/fresh-history --native EXE --cache CACHE --blender BLENDER --bounds BOUNDS --identities IDENTITIES --asset-key STUDY_KEY
python T/reconstruction_regions.py PRIVATE/geometry.json PRIVATE/section-recipe
```

section_reconstruction observes 3–12 coarse cross sections of curated regions
and constructs new, consistent ring topology. reconstruction_regions proposes
native skin-label/material partitions and omits zero-area/fully invisible faces.
Partitions are not automatically inferred anatomy. Descriptor observations are
target-derived, so these are data-assisted private reconstructions; they are
neither a round trip nor independently original art. Do not publish those
recipes, geometry or renders as original Tower content. Original counterparts
must be separate authored designs without target-derived sections or topology.

## Sealed examination

Before reading the six advanced cases in detail, record the source revision,
all Python authoring module hashes, native binary hash, rules and variant
selection policy. Catalog-only identity resolution follows the freeze.

```text
EXE CACHE PRIVATE/catalog catalog-npcs all
EXE CACHE PRIVATE/catalog catalog-sequences all
EXE CACHE PRIVATE/exact-definitions catalog-npcs ID...
EXE CACHE PRIVATE/full-action animate-complete npc:ID SEQUENCE
python T/advanced_examination.py --freeze PRIVATE/freeze.json --cases PRIVATE/cases.json --case CASE_KEY --out PRIVATE/exam --native EXE --cache CACHE --blender BLENDER --bounds BOUNDS --identities IDENTITIES
```

Catalogs contain protected local metadata and remain private. NPC definitions
identify idle/walk and transform variants; they do not by themselves identify
every combat action. Additional attacks/specials require actual local script or
definition evidence. Do not guess sequence IDs or relabel an unrelated animation.
animate-complete renders every native tick through the full source duration.

The examiner rejects changed frozen tools/binary and nonempty attempt directories.
It preserves source metadata/hashes, reconstruction recipes, new geometry,
multi-angle silhouette/color/internal-edge diagnostics and complete supported
native motion. Classic source controls may be applied privately to the new
group-compatible topology; reports explicitly separate that modeling test from
independent animation authorship. Authentic cached weighted motion can render,
but reconstruction weighted retarget/playback is unsupported. Record that failure
and continue eligible cases. A later fix requires a separately labeled retest.

Original counterparts use original recipes and the normal author_workflow path.
Publish only genuinely original assets, source-safe provenance/aggregates and
wholly original scene imagery. Keep authentic references, reconstructed derivatives,
source frames, cache textures and mixed reference scenery reviews local.
