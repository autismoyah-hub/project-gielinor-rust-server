# Static reconstruction fidelity reassessment

Four revision 239.1 cases were independently rebuilt from visual observations in an isolated continuation of the completed Sol modeling checkpoint `3f1b9ea`. New direct reconstructions, authentic imagery, editable sources and the comparison gallery remain private. This commit publishes only general tools and a source-safe factual summary.

Technical delivery is complete. Artistic fidelity remains incomplete. Cow, wolf and scimitar develop visibly beyond their historical generic blockouts; the goblin is a mixed result and still regresses in some silhouette views. No numerical metric or successful conversion constitutes artistic approval.

## Cases

| Study | Reference composition | New native geometry | Assessment |
|---|---|---|---|
| Armored goblin | npc:655; model IDs 24433, 24443, 24450, 24464, 24476, 24484, 24502, 24506 | 2053 vertices, 3676 triangles | Mixed; front/gameplay silhouette remains weaker |
| Cow | npc:2790; model IDs 23889 | 1497 vertices, 2593 triangles | Improved over historical blockout; fidelity incomplete |
| Wolf | npc:106; model IDs 26266 | 1484 vertices, 2626 triangles | Improved over historical blockout; fidelity incomplete |
| Iron scimitar | item:1323; model IDs 2373 | 230 vertices, 290 triangles | Improved over historical blockout; fidelity incomplete |

## Evidence and verification

The private review contains authentic / historical / new comparisons for front, side, rear, quarter, above and gameplay cameras, plus an asset detail camera. Every column uses the same camera, native light, origin and scale. Common-crop panels retain relative scale and position; no per-asset registration is applied. Full native frames, overlays and retained Blender round-trip controls remain accessible in the local gallery.

All four saved/reopened authored Blender sources re-export byte for byte. Private reference re-encodes reproduce all 20 fresh native inspection images exactly. Historical definition and component hashes match. The retained cache-to-Blender round trips match all 24 common native views. Fresh author-script runs reproduce the same vertices and oriented material triangles and all 24 common native images. Blender may reorder equivalent triangles in goblin/cow generation, so their fresh-run native payload bytes can differ even though saved-source re-export is exact. The renderer is unchanged. GPU Cycles beauty and equivalent native-camera projection renders are supplementary.

Integer export losses are recorded per study rather than hidden: micro triangles can collapse and source normals do not guarantee native shading parity. Retained iterations include rejected frame, coat, union, face and equipment treatments. The all-body cow union was rejected for weakening its deliberate planes and coat boundaries.

No animation, rig, live-map, gameplay, cache or enhanced-renderer work was performed.

## Reusable tools

`tools/tower/reassessment/anatomy.py` adds explicit longitudinal and cranial contours, parallel-transport limb frames, material-region cuts, solid surfaces, ray-fitted facial markings and a variable diamond-section curved blade. These consume authored parameters, not proprietary vertex/face arrays.

The frame regression crosses the world-up singularity: the old frame reverses direction (dot product -1), while the parallel-transport frame remains continuous (dot product 1). A failed facial surface intersection raises an error instead of silently placing an eye in empty space.

Run the geometry regressions with Blender in background mode:

```text
blender -b --factory-startup --python-exit-code 1 --python tools/tower/reassessment/verify_helpers.py -- PRIVATE_OUTPUT.json
```

`compare.py` takes a private JSON configuration containing renderer/cache paths, owned model paths for authentic/historical/roundtrip controls, per-case camera dimensions and native lighting. The renderer and cache paths must point at authorized local resources. Native camera settings and output directories are recorded with each image.

```text
python tools/tower/reassessment/compare.py PRIVATE_CONFIG.json reference reserved
python tools/tower/reassessment/compare.py PRIVATE_CONFIG.json render VERSION ASSET
```

Protected per-asset recipes are deliberately excluded. The owner's local review retains the common authoring script, editable Blender files, conversion reports and reproduction evidence.

## Remaining art deficiencies

- **Armored goblin:** Mixed result. The nose, jaw and brow remain stylized; torso armor is too simple, greaves too cylindrical, and shield/weapon pose still differs. Front and gameplay silhouettes can regress against the older study. This is not an accepted goblin reconstruction.
- **Cow:** Head pitch, muzzle and horn shape remain different. The brisket is too shallow, limb/body transitions still show, and the coat pattern is approximate. Native faceting and cheek planes do not yet reproduce the authentic surface.
- **Wolf:** Ruff and cheek masses remain too simple, body shading too smooth, and facial expression insufficiently close. Paws, leg plane changes, tail section and eye placement require more art direction. The profile is recognizably canine but is not an exact reconstruction.
- **Iron scimitar:** Blade belly, the kink approaching the tip, guard silhouette, pommel and facet shading remain approximate. The final cleanup removes coplanar guard surfaces; small source details still collapse during integer export.
