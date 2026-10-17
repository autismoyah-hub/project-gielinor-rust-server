# Geometry, surface data and scale findings

The scene study adds evidence from actual placements to the earlier isolated-model library. Its twelve new native/Blender inspections preserve decoded source topology locally; only identifiers, bounds, counts, attributes and observations are published.

| Actual scene asset | Model(s) | Faces | Finding |
|---|---|---:|---|
| Keldagrim wall6098 |5884 |114 | Four colors, no texture, native smooth faces and non-flat scene shading. Its construction differs from a closed beveled-box kit. |
| Hanging lantern3474 |3511 |186 | Small elevated fixture, only46 native units tall in its body bounds; apparent detail is concentrated above the floor. |
| Cave top1434 |834 |2 | A four-vertex surface intended to work with adjoining scenery. Closed high-poly rocks are not a universal requirement. |
| Cave face1417 |782 |10 | Minimal face geometry includes a160-unit buried skirt; exposed height must be distinguished from total bounds. |
| Dungeon wall1658 |16340 |16 | Uses texture23. Its fine surface pattern cannot be explained by triangle count or constant face colors. |
| Skull torch206 |2256 |210 | Small identifiable fixture has ten color values and a distinctive silhouette, not an emitted-light requirement in these static frames. |
| Ground detail325 |1139 |18 | Only eight native units high; sparse low detail contributes to a complete environment. |
| Short grass4735 |4401 |42 | Natural ground detail uses a small three-color shape and definition ambient20. It is not merely a green floor tint. |
| Clothing6062 |5968 |133 | Occupation is conveyed at roughly one-tile width with hanging silhouette and several colors. |
| Shelves6155 |2133 |68 | Contents and vertical grouping matter at a compact human scale. |
| Chair6195 |5934 |129 | Recognizable silhouette and working height, not a uniformly scaled architectural primitive. |
| Furnace6189 |1192 |158 | Reappears in the actual industrial window;1.17-tile visible height is a useful check on ordinary workstations. |

These observations are sample-specific. They do not establish universal OSRS polygon limits, palettes or dimensions.

## Human scale check

An additional read-only native NPC assembly measured cached NPC3106, `Man`, using its real body model list, recolors and native resize. Its unanimated bounds are70 ×202 ×33 native units (X × height ×Z), or0.547 ×1.578 ×0.258 tiles. See `HUMAN_SCALE_REFERENCE.txt` for model IDs and exact bounds. This is a static NPC bind pose, not a universal player/animated standing height. No human mesh is exported or redistributed.

The sample's explicitly labeled1.72-tile figure is about9% taller than that static reference and remains a ruler convention, not an authentic actor. A0.70-tile work surface is roughly44% of the measured human height; the earlier1.485-tile custom bench was roughly94%, supporting the observed scale correction. Actual player appearance and idle-animation measurements remain a later integration check.

## Original sample assets

The retained candidate uses the preserved original Tower assets plus original wall lantern, coats, personal shelf, low grit, rest mat and threshold families. New model IDs are1100081–1100086 and object IDs63081–63086. Existing role IDs remain stable, including arch1100010/object63010 and rack1100045/object63045. No original model/object identifier is replaced. The original cache's known maxima are far below these custom assignments; the overlay's duplicate/original-ID checks remain in place.

The original archive has58 models and6,596 triangles, of which38 families appear in93 sample placements. Geometry remains opaque, untextured, constant face-HSL with explicit native flat/smooth material properties. Some natural rock/vessel parts use native smooth shading. Pale lantern panels and orange fire faces do not emit light. Blender metallic/roughness values are not native material fidelity claims.

Native coordinates are128 units per tile, Y negative upward. Blender uses `[nativeX,nativeZ,-nativeY]/128`. The reference mapping has determinant+1 and preserves winding. Custom glTF conversion is quantized to nearest native integer and rejects nonfinite/bounded coordinates, unsupported textures/animation/transparency, degenerate triangles, nonclosed or inconsistent edges, index/codec overflow and excessive native model radius. The closed-mesh requirement is a converter constraint; several authentic references are intentionally open.

The original arch is about2.58 tiles wide and2.54 high. Its ordinary scale is distinct from the preserved foundry landmark. The wall occupies a two-tile module and has116 triangles; this is a measurement, not proof that it matches the114-face source wall. The pressure fixture is about1.90 tiles high with268 triangles. Original geometry stays original; source models supply analysis, not copied vertices.

## Native shading and topology limitation

The native decoder normalizes an all-zero face-render stream to an absent array; the documented default is smooth type0. The independent validation oracle now accounts for that representation without changing the decoder. Flat type1 and mixed streams are checked face by face.

Blindly setting every wall face smooth rounds normals across closed end/cap geometry. Selectively smoothing front planes while using scene-normal baking made some directional faces too dark in this particular closed topology. Those experiments are preserved and rejected. The retained flat walls are a known approximation. A later focused wall revision should design appropriate hard edges/front surfaces and deliberate color placement; changing global light to conceal this would invalidate the comparison.

Native underlays5000–5004 and overlay5010 are original sample inputs defined in the scene. Their RGB values and explicit heights pass through native floor decoding/blending/lighting. They are not original cache floor definitions. The package-only source rejects malformed/duplicate floor IDs, cache textures and installation through an ordinary cache reader. The unchanged live map and ordinary asset-loading validation are not replaced by this experimental terrain source.

## Reproduction and validation

`original-assets/generation-manifest.json` records per-asset purpose, native IDs, bounds, orientation, collision requirements and source provenance. The original GLBs, native model/definition bytes, independent expected geometry/shading/definition contracts and editable original Blender projects are included. The sample additions are generated headlessly from the preserved original Tower GLBs; no reference cache file is read by the original-asset script.

All58 models pass the independent native decoder, including every vertex, triangle, packed color, render type, model binding, footprint and optional scene-normal flag. Package-only reopening and rejection of cache fallback also pass. A separate native terrain contract constructs sloped terrain and verifies definition/source boundaries. These checks establish technical integrity only.
