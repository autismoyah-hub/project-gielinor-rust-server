# Production fidelity and generalization — current status

Three original Cinderworks assets and a small native mixed environment are ready
for artistic review. Consistent production art quality is **not established**.
The live map and authoritative client/server branches remain unchanged.

Authoring source: `ba9ed6c77555fc4085d33c8182dccf93330c1301`. Frozen held-out authoring:
`13acb2bdae6c25310d5434e502d69154fb03b6e8`. Preserved baseline:
`1c8b9b7169c2092475d713695007a29ae5e30e61`. Native client:
`23a94f75707f0503a9efe610fe99bfce5089b575`.

The workbench adds swept curved forms, separate staves, masonry/arches, joinery,
species-specific anatomy paths, semantic native materials, component masks,
contact-baked classic actions and an offline mixed-scene command. Scimitar and
anvil calibration improve; crate, shield, wolf and goblin silhouette results
regress despite new construction capabilities. Bat authoring is now supported
but remains visually weak. Seven sealed held-out cases are all unapproved.

Preservation: **51 unique static variants / 255 native views**, including seven
new static fixtures, are exact after saved/reopened Blender. The original
**26 classic action sets / 468 matching timestamps** remain pixel, float32
coordinate and alpha exact. The independent original creature has **368** native
pose checks across six clips; these are not added to original-source fidelity.
All **26 unit/boundary tests** pass. Protected cache: **50 files / 453,219,756 bytes**,
all original SHA256 values unchanged at final audit.

The Kilnback has complete idle/walk/attack/hit/death and an explicit walk-entry
clip. Its 49-pose walk has mean stance drift 2.599 / maximum 7.672 native units;
sampled penetration, floating stance feet and support failures are zero under
the declared geometric protocol. Residual sliding, collision and artistic
choreography require review. No automatic artistic approval is inferred.

Read [components](AUTHORING_COMPONENTS.md), [materials](MATERIAL_AUTHORING.md),
[animation](ANIMATION_AUTHORING.md), [diagnostics](FIDELITY_DIAGNOSTICS.md),
[calibration](CALIBRATION_RESULTS.md), [sealed evaluation](HELD_OUT_RESULTS.md),
[original assets](ORIGINAL_TOWER_ASSETS.md), [showcase](ENVIRONMENT_SHOWCASE.md)
and [ATELIER handoff](ATELIER_HANDOFF.md). Exact reference identities/hashes,
tool hashes and aggregate results are in [PRODUCTION_RESULTS.json](PRODUCTION_RESULTS.json).
Original publishable assets are in [art/tower/production-fidelity](../../../art/tower/production-fidelity/MANIFEST.json).

The protected local evidence root is the private `workbench-production` directory
beside the experimental server checkout. Open `review/index.html` there for the
overview, player views, all motion clips, A/B/C comparisons and retained failures.
That gallery and authentic mixed-scene imagery are deliberately not published.
All prior Floor One concepts, original probes, source fidelity fixtures and failed
studies are preserved. Full Floor One production remains suspended.
