# Cinderworks modeling standard — reference study 1

Status: measured working standard, awaiting artistic review. The previous 68-asset floor remains a preserved concept. Neither this guide nor successful codec tests confer visual approval.

## Evidence and limits

Thirty actual revision 239.1 object compositions were decoded with the isolated native Rust client, rendered from three angles through `WorldMesh` and the software rasterizer, and imported headlessly into Blender from the same decoded arrays. See `REFERENCE_MANIFEST.json` for IDs, counts, bounds, shapes, definition transformations, lighting and local interchange hashes. Four Blender views per object and all native views remain local because they contain proprietary art. The scripts are public; extracted arrays and reference pixels are not.

This sample emphasizes the Keldagrim symbol families and industrial scenery; the symbol table does not prove a particular world placement. It is not a census of OSRS architecture. A complete original game room, animated machinery and terrain lighting were not reconstructed for this first calibration. Observations below describe this sample, and the proposed Cinderworks rules are design decisions informed by it.

## Scale and hierarchy

| Measured composition | Visible native extent X × height × Z, tiles | Faces | Consequence for the old kit |
|---|---:|---:|---|
| 1868, model 1486, complete arch | 1.72 × 2.38 × 0.47 | 260 | An ordinary portal need not be a four-tile wide, nearly four-tile tall monument. |
| 7016, model 7097, palace pillar | 1.00 × 1.88 × 1.00 | 214 | Shape the shaft and capital; five stacked beveled boxes are not equivalent detail. |
| 6189, model 1192, furnace | 1.72 × 1.17 × 1.80 | 158 | Old custom furnace is 3.8 wide and 4.0 high. Reserve that scale for the final foundry. |
| 9085, model 9049, stove | 2.00 × 2.25 × 1.00 | 334 | Complex machinery can remain a readable compact silhouette. |
| 6150, model 5918, anvil | 0.96 × 0.50 × 0.25 | 72 | Lower the old 1.32-tile custom anvil and tighten its footprint. |
| 6196, model 5952, table | 0.92 × 0.60 × 0.92 | 50 | Work surfaces and furniture should read as usable by people. |
| 5589, model 5516, wheel | 3.02 × 3.10 × 0.98 | 348 | Use a strong rim/paddle silhouette, not excessive rings, spokes and bright fittings. |
| 6147, model 727, rock column | 2.00 × 5.63 × 2.00 | 76 | Tall cave structure can use very little geometry; contour matters. |

Visible bounds differ from definition occupancy. Wall 6010 extends 160 native units below its placement datum; do not mistake that submerged skirt for a 3.125-tile exposed wall. Arch 2106 is an offset fragment, not a complete doorway. Pump 9090 is a thin elevated handle component, not the entire mechanism. Inspect assembled context before drawing proportional conclusions.

Working Cinderworks targets: domestic tables 0.60–0.80 tile high; anvils approximately 0.55–0.85 including support; regular masonry exposed height 1.8–2.5; ordinary portals 2.5–3.2 wide with a clear 1.6–2.2 opening. These are project targets, not universal OSRS limits. Sanctuary/foundry landmarks may exceed them deliberately. Use a static 1.72-tile figure as a consistent comparison convention and label it; it is not an original player measurement.

## Shape and topology

The native pillar has a broad polygonal capital, narrowing/twisting shaft and weight at its foot. Its recognition comes from changing cross sections, rather than separate rectangular courses. The stove combines a rounded body, a substantial face frame and a short chimney. The furnace uses a restrained shell with a localized orange mouth. The rock column bends and narrows dramatically without a high triangle count.

Model silhouette first. Use low-sided cross sections with purposeful changes in width, taper and lean. Add seams where construction changes or parts articulate; do not outline every surface with a uniform bevel. A wall run should have a continuous body, a credible foot and coping, interrupted by structural piers and a few repair seams. Provide a real corner/end model; do not hide an unfinished termination with another unrelated prop.

The sampled wall models use 18–105 faces, while the old one-tile wall used 264. The old 636-face arch, 396-face pillar and 564-face furnace spend geometry on repeated blocks and beveled boxes. Simplify before adding detail. There is no hard triangle budget inferred from these few assets; test the silhouette at actual player distance, then justify any added geometry by a visible improvement.

Original meshes may contain open boundaries and hidden undersides intended for terrain or neighboring pieces. Preserve them in reference import. The custom converter deliberately retains its stricter closed, consistently wound mesh contract; this is a tooling constraint, not a claim that original OSRS assets must be closed.

## Materials and native shading

Several Keldagrim architectural references use three closely related face colors, not many alternating stone swatches. Wall/pillar examples have packed-HSL lightness values around 18–26; these encoded values are not display luminance and cannot be compared directly with a Blender linear RGB component. Some references are much brighter, notably the textured arch. Avoid inventing one universal palette.

Use a narrow warm grey/charcoal masonry range, dark neutral iron, muted brown wood and small colored functional accents. Reserve warm orange for visible heat sources and a restrained teal for water/maintenance markings. Concentrate brighter wear at touched edges and repair sites. Do not alternate brass/stone bands on every structural member. The previous kit's bright brass collars and cyan surfaces repeated too evenly to convey a specific history.

Four distinct layers affect appearance:

1. Geometry establishes silhouette, faceting and normals.
2. Packed face HSL and texture identifiers carry authored surface information. Reference 1868 uses texture 15; its stone pattern is not mesh detail and is not recreated in Blender.
3. Native face render types, vertex-normal accumulation and definition ambient/contrast determine shading. The furnace mixes smooth and flat faces; walls also use `non_flat_shading` scene treatment. This is different from Blender `use_smooth` and PBR roughness.
4. The native palette at brightness 0.8, scene lighting, texture sampling, clipping and rasterizer produce final pixels. Blender studio lighting is only a topology aid.

The old converter explicitly encoded every face flat. Support deliberate native shading choices in manifests and validate decoded render types; do not assume that more bevels can substitute for smooth native shading. Neither original nor custom orange face colors emit dynamic light in these static studies. Metallic/roughness settings, transparency and emission are not supported by the custom opaque codec and must not be claimed in evidence.

## Terrain and placement

The current tiled plates expose a regular one-tile outline across every room. Reference models' submerged skirts and edge-aligned wall shapes demonstrate dependence on terrain and neighboring structure, but this first sample does not establish an authentic whole-room terrain recipe.

For Cinderworks, author continuous original ground surfaces with broad worn routes, irregular repair slabs and locally exposed bedrock. Keep the principal travel datum readable. Place silt at slow water corners, damp staining below joints and ash near exhaust; avoid uniform random noise. Retaining walls must meet the water surface and bridge abutments. Show a source, controlled headgate, wheel drop, return channel and maintenance access. Do not allow scenery to imply an inaccessible workstation or a bridge landing in water.

## Environmental authorship

The toolkit shelf (6055), hanging smith tools (6188), table (6196), crates (6178) and bed (6183) demonstrate compact occupational props with recognizable use. They do not by themselves prove a complete scene's storytelling. Group original equivalents into specific activities: incoming ore, sorting, furnace fuel, hot handling, quenching, finished stock and worker rest. Leave a working side open at each machine.

The sanctuary should occupy a converted service station: benches, stores, sleeping places, a ledger and patched utility lines around a protected water source. Contrast maintained routes and occupied pockets with failed bays. Foundry growth should be visible through more substantial structure, deeper heat scars and an increasingly clear central destination, rather than scaling every prop equally. Resource areas should connect to transport and drainage, not appear as generic annexes.

## Review contract

Compare arch, wall/corner and industrial fixture beside relevant original references at the same native scale, brightness, camera and normal scene lighting. Keep mixed comparison frames local. Publish original-only versions and written measurements. Inspect front, reverse, quarter, close player view and adjoining-kit context. Preserve before/after versions. A successful codec roundtrip validates compatibility, not artistic coherence.

Stage C is a focused review checkpoint. Continue useful iteration without treating silence as approval. Revisit rules when broader scene evidence contradicts them. The complete floor remains an offline proposal until the user explicitly approves promotion.
