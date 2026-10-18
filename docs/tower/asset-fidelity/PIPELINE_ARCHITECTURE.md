# Pipeline architecture

The workbench is a set of reproducible command-line tools under
`tools/tower/asset-workbench`, with native entry point `asset_workbench` in the
existing offline-render crate. It depends on the unchanged client library and
uses its model/definition factories, animation operators, lighting, textures and
software rasterizer. There is no new renderer or modification to global shading.

1. Native inspection decodes selectors and assembly, hashes original payloads,
   and renders matched close and gameplay-scale views.
2. Source-safe analysis derives numeric descriptions; semantic annotations remain
   explicit and separate from decoded facts.
3. Blender interchange stores actual mesh topology, HSL materials, face attributes,
   vertex groups, texture-map indices and action channels. A fresh process reopens
   the saved file before export.
4. The extended codec preserves supported static/classic channels and simple
   texture mappings. Complex texture reconstruction and weighted bone encoding
   are rejected. Fractional evaluated objects retain raw meshes plus their native
   custom-definition transforms instead of being silently rounded.
5. Immutable packages bind new model/object IDs through the existing native package
   reader. Standalone motion uses new classic frame/skeleton IDs and existing
   native deformation. It is not a live NPC overlay or gameplay integration.
6. Comparison reports separate exact technical preservation from independent
   silhouette, component, material and choreography diagnostics.

The earlier GLB converter and calibration tests remain unchanged. General owned
authoring uses explicit native-unit component recipes and sockets, with local
Blender edits and parent-aware native joint compilation. Private reconstruction
outputs are never automatically copied into publishable Tower packages.
