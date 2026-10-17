# Original Blender arch pipeline experiment

This package is original procedural geometry. It contains no extracted cache
models, textures, fixtures, account data or game screenshots. The editable blend
and Blender preview remain local; the generator and80KiB GLB are distributable
original source. The9KiB final model is native-client-consumable mesh data.

Use only the isolated experiment/custom-model-pipeline server and client branches.
The authoritative Tower gameplay and client parity branches are unchanged.

1. Run the installed Blender executable with `--background --factory-startup
   --python blender/create_archway.py -- --output OUTPUT`.
2. Run Python `converter/convert_glb.py OUTPUT/archway.glb PACKAGE`.
3. Run Python `converter/package_archway.py PACKAGE`.
4. Run `converter/test_converter.py`. Use a writable temporary directory.
5. Run the client `custom_model_validate` example with the final model and
   expected-geometry.txt. This compares against the unchanged native decoder.
6. Run `custom_asset_contract PACKAGE NEW_TEST_DIRECTORY ORIGINAL_CACHE` against
   the isolated server's read-only JS5 service. It verifies original bytes/CRCs,
   immutable registry behavior and rejection cases. The example currently uses
   the dedicated experiment loopback port63225.
7. Set TOWER_ASSET_PACKAGE to PACKAGE for both isolated processes. Start a
   disposable server and a normal native live-view client with the existing
   TOWER_ACTION_DIRECTORY gateway. No desktop input automation is used.
8. Run the native offline renderer with registry/archway.scene and the same
   package, optionally using TOWER_JS5_READTHROUGH_ADDR for the legacy cache.

The normal offline presets are retained; asset-plan and asset-passage add close
model inspection cameras without replacing scene assembly or the rasterizer.

See docs/tower/custom-assets for format limits, publication and integration
contracts, exact tested revisions, evidence and remaining production gaps.
