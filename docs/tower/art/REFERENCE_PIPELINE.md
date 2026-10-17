# Offline art reference and calibration reproduction

Use two independent sibling checkouts named `rust-server` and `rust-client-reference`, both on `experiment/fallen-cinderworks-design`. Do not change the production Tower checkout or the client parity checkout. Install an ordinary Rust toolchain and Blender 5.2.2 LTS; the scripts also require Python with Pillow for review PNGs. All paths passed to Blender should be absolute.

Resources: authorized revision 239.1 paired `cache/js5` and `symbols/loc.sym`. Keep resources and all extracted geometry outside either repository. Create `local-reference` and `art-redesign` beside the two checkouts. Those are local resource/evidence directories, not publication destinations. No gameplay server, account, database, login or network service is involved.

## Native decoding

From the client checkout, build `cargo build --release --example art_reference`. Run the example with `CACHE_JS5 LOCAL_REFERENCE comma-separated-object-IDs`. Omitting the last argument produces a candidate TSV only. The selected thirty IDs are recorded in `REFERENCE_MANIFEST.json`; retain that manifest as the sample identity.

`DiskCache::open_legacy_reference` is an explicit offline input-format adapter. The staged legacy disk containers hold only a u16 revision suffix even when the reference table revision exceeds 65535. The adapter verifies the original full group CRC and the stored low revision bits, expands only the in-memory suffix, then passes the result through the ordinary native decoder. `raw_group` still returns original bytes. Default client `open`, recovery validation and ORACLE's checkout are unchanged. This is compatibility with a known source representation, not repair or proof of the missing upper revision bits. A corrupt CRC or low revision mismatch fails closed. Contract checks exercise both failures and verify normal strict decoding still rejects the truncated revision.

The example exports the raw model and the native object composition after shape selection, mirror, recolor/retexture, scale and offsets. JSON preserves triangle indices, native coordinates, colors, face render types, priorities, alpha, skins/bone weights, texture IDs/coordinates/triangles and face depth flags. Vertex ordering and winding are not cleaned up. Source model metadata not exposed by this interchange and complex texture mapping arrays omitted by the existing revision decoder are limitations; no replacements are invented. Animation, world terrain contouring and live transformed-object state are not applied in this static composition study.

## Blender inspection

Run headlessly with `tools/tower/custom-assets/blender/inspect_osrs_reference.py -- --input ABS_LOCAL_REFERENCE --output ABS_LOCAL_REFERENCE/blender`.

Mapping: native `[X,Y,Z]` → Blender `[X,Z,-Y]/128`, determinant +1. No normals are recalculated, vertices merged or triangles changed. The importer verifies face indices exactly and coordinate error below 0.000001 tile. It writes four standardized orthographic views and a `.blend` per reference. The complete native source JSON remains in a Blender text block for inspection.

Face HSL is previewed through the native palette at brightness 0.8 converted to Blender linear RGB. This does not reproduce native lighting. Unsupported texture mapping is explicitly purple; texture references remain in metadata. Alpha, draw priority, face depth bias and skeletal attributes are retained as metadata, not given guessed Blender behavior. The native renderer is the appearance authority.

## Native reference rendering

Build `cargo build --release --manifest-path tools/tower/offline/render/Cargo.toml` in the server checkout. It depends on the sibling isolated client. Run `design/render_references.py RENDERER CACHE_JS5 LOCAL_REFERENCE`. It sets only the explicit offline legacy reader, clears recovery/package overrides, renders three angles and stores proprietary output in `LOCAL_REFERENCE/rust`. Source cache access is read-only.

Generate source-safe analysis with `design/analyze_reference.py LOCAL_REFERENCE LOC_SYM docs/tower/art`. It copies no geometry or images. The source-safe manifest contains IDs, counts, bounds, encoded lightness ranges, edge-incidence summaries, definition lighting and local interchange hashes. Cache/source hashes are retained in the checkpoint resource manifest. Symbol labels identify families but are not proof of original world placement.

## Original calibration

Generate with `blender --background --factory-startup --python ABS_CREATE_CALIBRATED_KIT -- --output ABS_GENERATED --iteration 2`.

Convert with `converter/package_cinderworks.py GENERATED PACKAGE`. This accepts only supported opaque static geometry, verifies closed edges and winding, rejects quantized degenerate triangles and bounds/codec overflows, and converts twice to check determinism. Original material custom property `native_render_type` explicitly selects smooth 0 or flat 1; absent property preserves the older flat default. New material behavior is not inferred from Blender appearance. Model and object IDs remain stable through iteration and must be noncolliding.

Build/run client `custom_kit_validate` against PACKAGE. Its independent native decoder checks every expected vertex, triangle, packed color and render type. Package-only reopening and rejection of cache fallback are also checked. Use `design/render_calibration.py RENDERER CACHE_JS5 PACKAGE REVIEW` for native comparison scenes. `REVIEW/private-reference` contains protected original art; keep it local. `REVIEW/original-only` contains only original Tower assets and can be published. The renderer retains raw BMPs and camera sidecars; PNG headers are added outside native raster pixels.

The three representative families are an original arch, a wall with bonded corner and a pressure fixture. A static 1.72-tile figure is a consistent ruler convention, not an authentic NPC. Mixed scenes use the same scale, brightness and lighting, but perspective causes pixel-size differences with depth. Original-only close views use different framing and cannot be used as equal-pixel-scale comparisons.

## Publication and handoff

Publish scripts, source-safe analysis, original GLBs/models/definitions/manifests and selected original-only Rust renders on the isolated branch. Do not publish local reference JSON, `.blend` reference projects, extracted textures, cache archives or mixed reference screenshots. No distribution permission is assumed. Keep earlier concept studies and stable ID assignments intact.

Each future asset manifest must describe role, origin/rotation, footprint, native material behavior, bounds, IDs and collision requirements. Offline nonblocking definitions are not gameplay integration. Before any later promotion, explicitly author collision, terrain heights/planes, access/actions, resource ownership, encounter safe space and completion interactions. Those systems and production maps remain outside this assignment. Stage C is a review checkpoint; artistic approval remains with the user. ATELIER can reproduce and extend these independent scripts without private account or database state.
