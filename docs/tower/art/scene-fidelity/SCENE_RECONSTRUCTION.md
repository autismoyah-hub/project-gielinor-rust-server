# Authentic scene reconstruction and input identity

These are finite static reconstructions from the authorized, locally staged revision 239.1 paired cache. They are not arrangements of individually chosen props. The native client reads archive 5 terrain file 0 and delta-coded map-location file 1, loads the real object definitions, applies native scene insertion/collision bookkeeping and builds `WorldMesh` through the normal terrain/object paths.

| Reference family | Normal rebuild anchor X,Z | Native scene base X,Z | Camera focus X,Z | Region groups in archive 5 |
|---|---|---|---|---|
| Industrial underground architecture, Keldagrim symbol families | 2896,10176 | 2848,10128 | 2871,10205 | 11422, 11423, 11678, 11679, 11934, 11935 |
| Stone underground complex | 2880,9792 | 2832,9744 | 2854,9809 | 11416, 11417, 11672, 11673 |
| River terrain and settlement | 3232,3240 | 3184,3192 | 3232,3240 | 12593, 12594, 12595, 12849, 12850, 12851, 13105, 13106, 13107 |

Each native scene is 104 × 104 tiles. Industrial source chunks span X356–368/Z1266–1278; dungeon X354–366/Z1218–1230; river X398–410/Z399–411, with no rotations or fabricated replacement chunks. Terrain and original placements are loaded across four source planes; the review renders plane 0 with upper roofs hidden. Camera focus is selected independently inside the source window. The inventory's legacy `world_focus` label denotes the rebuild anchor, not the independently selected camera focus; camera sidecars record the latter.

`SCENE_REFERENCE_INVENTORY.json` records exact region IDs, raw disk-group SHA256, expected CRC32/version, original object IDs, source model IDs and placement counts. The complete paired-cache file identity is also preserved in the parent art reference manifest. Final source windows contain 8,483 / 9,287 / 7,925 static map-location entries respectively across all planes. All final frames report `missing_regions=[]` and `mesh_pending=false`. Those entry counts are not visible prop counts.

Initial survey windows centered directly on the industrial and dungeon camera foci requested source-absent peripheral groups 11680 and 11160. They remain recorded in the local survey logs. Final available source windows were chosen to contain the desired focus without inventing missing region content. This is a bounded reference window, not a claim that the entire original underground world was reconstructed.

## Native methodology

The offline renderer recognizes `N anchor_x anchor_z plane` and calls native `Scene::load_normal`. It does not export or re-author the reference placement list. The normal loader decodes actual terrain, flags, shape/rotation and locations, applies the ordinary scene bookkeeping, and builds native geometry, model/scene lighting, underlay blending and texture mapping. Original texture definitions and sprite pixels are loaded read-only. `TOWER_ASSET_PACKAGE`, package-only and network-recovery overrides are cleared for reference jobs.

The explicit offline legacy reader validates the staged source's stored u16 revision and full group CRC before expanding only its in-memory suffix for the stricter native decoder. Original disk bytes are never changed. This compatibility adapter does not prove revision upper bits absent from the legacy suffix; full source identities and CRCs are retained. Normal client validation and the original parity checkout are unchanged.

The full 50-file, 453,219,756-byte paired cache was rehashed after this calibration. Every SHA256 matches the preserved initial inventory; see `cache-integrity.txt`.

## What is and is not reproduced

Actual map terrain, location IDs, shape/rotation, definitions and native static model assembly are reproduced inside the selected windows. Upper roofs are deliberately hidden for both reference and custom comparisons. Native model/texture requests complete without a pending-asset fallback.

These are offline static views. They omit players, NPCs, ground-item spawn/state, live animation clocks, dynamic location packets, quest/account varps, UI, camera input/inertia/shake, skybox selection and the live scene lifecycle. Transforms use the native default variable state. Source-available geometry is not proof of current live quest or animation state. No new live client capture was made during this phase; older native-client pipeline evidence is a different experiment and is not presented as scene-fidelity acceptance.

## Geometry inspection

The earlier thirty-model reference library is retained. Twelve additional compositions were selected because their IDs actually occur in these reconstructed windows: 6098, 3474, 1434, 1417, 1658, 206, 325, 4735, 6062, 6155, 6195 and 6189. Their source-safe measurements are in `model-study/REFERENCE_MANIFEST.json`.

Native Rust decoding produced local-only interchange arrays, then the headless Blender importer generated four topology/proportion views per composition. All twelve imports preserved every face index and winding; maximum coordinate error was zero at the measured tile scale. Packed colors and texture references remain in the source data. Blender does not reproduce native lighting; unsupported textures are explicitly marked rather than replaced with guessed UVs. Complex texture mapping arrays omitted by the existing native decoder, animation, alpha/priority semantics and auxiliary metadata limitations are documented in the reference pipeline. No extracted mesh, reference Blender project, texture or reference screenshot is published here.

## Local evidence

Protected reference frames, raw BMPs, camera sidecars, source inventories and raw placement TSVs remain under `experiments/custom-model-pipeline/scene-fidelity/references`. The nine earlier 45° native reference rasters were reproduced into a separate local archive and verified against all nine original pixel hashes. The current review references use the shared 56.25° camera described in `RENDERER_CAMERA_COMPARISON.md`. Public material contains only safe measurements/identifiers and original Tower art.
