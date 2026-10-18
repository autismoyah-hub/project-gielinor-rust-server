# Pipeline architecture

The workbench lives under `tools/tower/asset-workbench`, with native entry point
`asset_workbench` in the existing offline-render crate. Client library source is
unchanged. All model pixels use its model factories, deformation, lighting,
textures, submission and rasterizer. Python produces diagnostics, contact sheets
and UI only; it is not a second renderer.

1. Native inspect decodes definitions/assembly and hashes source payloads.
   `inspect-static` omits action extraction. Submitted-surface diagnostics use
   native lighting eligibility while preserving full decoded data.
2. Analysis separates exact fields, calculated descriptors and curated semantics.
   Same-camera masks, row profiles, PCA and material-region reports guide edits.
3. Blender interchange stores actual geometry, HSL/face metadata, labels, texture
   maps and action channels. A separate process reopens the saved file.
4. Owned authoring builds named primitives/sockets and explicit native joint
   ownership. `edit_blender.py` changes selected saved components; the action
   replacement option is explicit and preserves previous owned Blender actions.
5. The native codec rejects fractional coordinates, unsupported complex textures
   and weighted bone export. Fractional factory results use raw B meshes plus
   their native custom definitions instead of quantization.
6. Immutable object packages and stable identity manifests allocate new IDs.
   Static and animated counters are separate. Static keys cannot silently become
   actors; use an explicit new variant key. Manifests are single-writer, not a
   distributed allocator.
7. Native rendering supports fixed-reference views, owned standalone views and
   actual shared `beside` scenes with common camera/light and an 80-unit gap.
   It preserves dimensions and never fits one shared-scene asset to the other.
8. Strict exit codes, fresh output directories, expected action/timestamp coverage
   and native coordinate checks reject stale or incomplete success reports.

No original cache is rewritten. The legacy-reference cache adapter is explicit
and read-only; normal strict client loading remains unchanged. No live NPC
overlay, map integration, service launch or game account operation is included.
The earlier GLB converter and calibration executable are preserved.
