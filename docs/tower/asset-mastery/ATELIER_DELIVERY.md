# ATELIER delivery and reproduction

Use TOOLING_HANDOFF.md and ../asset-fidelity/ATELIER_HANDOFF.md for workspace
bootstrap, pinned sibling checkouts and the offline Rust toolchain. Tested:
Python 3.13 + Pillow; Blender 5.2.2 LTS d13f752e3b9c; isolated client
23a94f75707f0503a9efe610fe99bfce5089b575. Build asset_workbench in
tools/tower/offline/render. No manual Blender scene or UI dependency.

Set explicit NATIVE, CACHE, BLENDER, PRIVATE, BOUNDS and IDENTITIES paths.
CACHE is an authorized revision-239.1 JS5 directory kept read-only. BOUNDS
records actual original namespace maxima from inspection. IDENTITIES is a
persistent single-writer ledger. Copy public asset_key/identity pairs from the
delivery manifest into its assets map for exact binary-ID regeneration. Preserve
other private reservations. Check IDs against the future shared destination;
experimental reservations are not globally assigned production IDs.

Coordinates: X/Y-down/Z, 128 units per tile, original models at scale 128.
Quarter-turn rotation precedes scene translation. Common native light: 64/768,
direction [-30,-50,-30]. Native HSL and classic animation are authoritative.
No PBR shader or alternate animation system determines final appearance.

## Author and check an original

```text
python tools/tower/asset-workbench/original_boss_designs.py PRIVATE/recipes
python tools/tower/asset-workbench/author_workflow.py --recipe PRIVATE/recipes/basalt-hearthbearer.json --out PRIVATE/fresh-boss --native NATIVE --cache CACHE --blender BLENDER --bounds BOUNDS --identities IDENTITIES --asset-key mastery-original-boss:basalt-hearthbearer
python tools/tower/asset-workbench/owned_motion_review.py PRIVATE/fresh-boss PRIVATE/previews
python tools/tower/asset-workbench/contact_diagnostics.py PRIVATE/fresh-boss PRIVATE/contact
```

The workflow constructs, saves and reopens a real .blend; exports native models,
classic skeleton/frame/sequence payloads and a diagnostic object package; renders
five views; compares native poses to independent tracing. Recipe parameters own
geometry/material/rig/action inputs. Always use fresh outputs and retain failed
iterations. Use a new asset key when static/animated identity semantics change.
Native per-section byte lengths and combined vertex counts have separate u16
limits; split real geometry into chunks before overflow.

## Reproduce the whole floor

art/tower/advanced-mastery contains original recipes, native models, classic
payloads, five views, identity manifests and a relative scene submission:

```text
NATIVE CACHE PRIVATE/overview scene art/tower/advanced-mastery/floor/scene.tsv art/tower/advanced-mastery/floor/overview-detail.camera
NATIVE CACHE PRIVATE/sanctuary scene art/tower/advanced-mastery/floor/scene.tsv art/tower/advanced-mastery/floor/sanctuary-a.camera
```

Regenerate delivered recipes with their asset_key/identity pairs. Build an
ASSET_ROOTS JSON mapping asset names to the fresh output folders, then:

```text
python tools/tower/floor-one-visual/design_assets.py PRIVATE/scenery-recipes
python tools/tower/floor-one-visual/design_assets.py PRIVATE/refined-recipes --refinement
python tools/tower/floor-one-visual/compose_floor.py --assets ASSET_ROOTS --out PRIVATE/new-floor --native NATIVE --cache CACHE --blender BLENDER --bounds BOUNDS --identities IDENTITIES
python tools/tower/floor-one-visual/render_motion.py --floor PRIVATE/new-floor --actor PRIVATE/original-crown --out PRIVATE/new-motion --native NATIVE --cache CACHE
```

Initial scenery supplies walls, bell, bench, cart, vessel, rock and lantern.
Refinement supplies raised sluice, retort, threshold, stock, barrel, resting bench
and clothed figure. Delivered recipes identify exactly which version is placed.
The earlier original pump/threshold/Kilnback recipes are included, removing any
dependency on private old output folders. layout.py owns polygons, connectors,
water, crossings, elevations and routes; compose_floor.py owns terrain,
supported architecture, placements and cameras. Every ground chunk passes
through actual Blender save/reopen and native decoding.

render_motion submits every native action tick, including terminal holds.
Previews combine identical image holds using original durations. Declared world
walking travel is not collision or gameplay pathfinding. All original source,
recipes and lossless rendered PNGs are distributable; no authentic cache mesh,
frame or texture occurs in the published floor.

## Private evaluation and known limits

workbench-mastery/review/index.html is protected local evidence. Do not publish
it or its media. exam-first-attempt is frozen; exam-development-v2 is explicitly
post-exam. Raw first-attempt command-success labels require the movement failure
audit described in CURRENT_STATUS. Native play-complete covers all custom
classic ticks; animate-complete covers native source clips. Independent cached
weighted authoring/retargeting remains unsupported.

preserve_authored_groups keeps classic ownership without an authored rig;
preserve_face_groups=false avoids inventing alpha membership when absent from a
source. Original interchange retains its untouched path. Component feedback
requires curated semantics. Section reconstruction uses target-derived coarse
observations: it is a private derivative, not automatic anatomical understanding
or original art. Reference-derived material brushes also stay private.

Footprint, body-projection and capsule diagnostics are proxies. Repair stance
drift and floating feet before integration. Floor connectivity ignores scenery
blocking; AABBs overestimate arches, raised gates and articulated bodies.
Cutaway architecture, static face-colored water and emissive-looking lantern
values have no production roof, liquid simulation or dynamic point lights.
The environment needs user artistic approval and further floor/rock refinement.

Future integration needs destination registry checks, region placement,
terrain/elevation/roof/occlusion work, authored clipping and real player clearance.
Do not infer acceptance from these offline manifests. No gameplay work is included.

Run the 43-test workbench suite. Reproduce native model/classic payload bytes
and five views against delivery hashes. Render the relative scene and compare
pixels to the lossless PNGs. Authorized private fixtures are needed to repeat
the 51-static/26-classic preservation suite; none is embedded in this repository.
Retain tool/input hashes, source revision and failed iterations at every step.

## Additional development and gallery entry points

```text
python tools/tower/asset-workbench/adaptive_reconstruction.py PRIVATE/reference/geometry.json PRIVATE/new-static-proposal --metadata PRIVATE/reference/metadata.json
python tools/tower/asset-workbench/review_static_mastery.py --private PRIVATE/workbench-mastery --server SERVER_CHECKOUT
python tools/tower/asset-workbench/build_private_mastery_review.py --private PRIVATE/workbench-mastery
python tools/tower/asset-workbench/build_mastery_review.py --private PRIVATE/workbench-mastery --server SERVER_CHECKOUT
```

The last command assembles already packaged original art and safe aggregates.
It does not publish protected comparison media. The private builder requires the
sealed evidence root outside the source checkout. Gallery links depend on the
documented evidence layout, not a hidden running session. The original-only
package_mastery.py exports an explicit allowlist after checking provenance and
rejecting cache-textured assets. Its output must stay outside the original cache.
The static proposal's inputs, output recipes and images remain protected; no
weighted animation support or faithful artistry is inferred from bone partitions.
