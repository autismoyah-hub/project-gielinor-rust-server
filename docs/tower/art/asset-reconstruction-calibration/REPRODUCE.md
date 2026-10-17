# Reproduction and publication boundary

Use the experimental server tools with the sibling client source pinned to
23a94f75707f0503a9efe610fe99bfce5089b575. Build the server's
`tools/tower/offline/render` binary `asset_calibration` in release mode using the
existing isolated toolchain wrapper. No production service needs to run.

The complete local workflow is
`tools/tower/custom-assets/design/run_asset_calibration.ps1` with explicit
`CacheRoot`, `OutputRoot`, `Blender`, `Renderer`, and `IndependentAuthoringScript`
arguments. The output directory must be outside the source repository and cache.
The private authoring recipe is
`asset-calibration/authoring/calibration_recreate.py` beside the evidence in the
parent experiment workspace. It takes reference timing only, not original meshes
or frame transforms. It is deliberately excluded from publication with the
reconstructions it generates. Blender 5.2.2 LTS is used headlessly.

The workflow exports native references, saves and reopens Blender, exercises the
edit witness, generates and encodes independent studies, validates custom native
definitions and identifiers, renders A/B/C and blends, compares geometry/pixels,
and produces the private review. It records source identities and the renderer
hash. Its output is **private reference material**, not a publishable Tower kit.

The native tool's standalone modes are:

```text
asset_calibration CACHE OUTPUT export
asset_calibration CACHE OUTPUT render static original
asset_calibration CACHE OUTPUT render static CUSTOM_MODEL
asset_calibration CACHE OUTPUT render npc original original
asset_calibration CACHE OUTPUT render npc CUSTOM_MODEL CUSTOM_PACKAGE
asset_calibration CACHE OUTPUT compare static original ROUNDTRIP_MODEL INDEPENDENT_MODEL
asset_calibration CACHE OUTPUT compare npc original ROUNDTRIP_MODEL INDEPENDENT_MODEL
```

`compare_asset_calibration.py FINAL_DIRECTORY` performs exact geometry, pose,
alpha and pixel comparison. `review_asset_calibration.py FINAL_DIRECTORY`
builds private media and a separately named `SOURCE_SAFE_SUMMARY.json`.
`test_calibration_codec.py` runs seven boundary/determinism tests. The full cache
hash audit is an independent read-only check against the pre-existing 50-file
manifest. A full native-image/model/frame reproduction check is retained locally.

Only the following belong in the experimental Git branch: general-purpose source,
these documents and explicitly selected aggregate metadata. Never stage the
evidence tree recursively. In particular, do not publish original binaries,
Blender derivatives, original or reconstructed screenshots, direct model
reconstructions, their specific recipe, character state or authentication data.

## Next gate

Review the independent object's architectural treatment and the creature's
anatomy/materials/motion separately from the exact conversion result. The smaller
environment scene and style standards can be reassessed after that review. There
is no automatic return to complete-floor production or gameplay integration.
