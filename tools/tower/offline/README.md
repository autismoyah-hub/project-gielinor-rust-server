# Cache-backed offline composition studies

These two standalone tools export the current server-authored cold shell and render candidate compositions through the existing native client library. They never connect to a game server, read an account database, or modify protected cache files. Run from the Tower workspace containing sibling `rust-server` and `rust-client-reference` checkouts and the verified `resource-staging` inputs. The native client checkout remains authoritative; this tool introduces no replacement renderer or client changes.

```powershell
cargo run --manifest-path rust-server/tools/tower/offline/export/Cargo.toml -- . work/scene-study
cargo run --release --manifest-path rust-server/tools/tower/offline/render/Cargo.toml -- resource-staging/server/work/integration/a-kotlin/.data/cache/js5 work/scene-study/baseline.scene work/scene-study/sanctuary.bmp 56 56 0
```

Use the workspace's isolated toolchain and build directories when required by its setup instructions. The exporter also writes a sanctuary furnishing-entry audit and a tab-separated audit of every final prepared hub object. Use the final records to distinguish visible props from authoring entries already replaced on the same scene layer.

The exporter is compiled against the current authored server source. Its output is a design snapshot, not a capture of a running player's personal restoration state. Preserve a snapshot under a new name before generating another.

## Scene format

For three isolated complete-floor proposals, original layout diagrams, actual-cache feasibility checks and batch camera presets, see the [design workbench](design/README.md). The renderer now accepts optional `[DISTANCE] [detail|overview|plan]` after yaw and writes a `.camera.txt` sidecar. The default detail preset retains the existing 765×503 camera geometry; overview and plan provide larger whole-floor frames. These are composition cameras, not live-controller equivalence.

Each whitespace-delimited record uses original local Floor One coordinates:

- `T destination-zone-x destination-zone-z destination-plane source-zone-x source-zone-z source-plane rotation` copies an authentic eight-tile cache chunk through the client's existing `Scene::add_chunk`.
- `L local-x local-z plane location-id shape rotation` places a final prepared location through native `PlacedObject` and `WorldMesh`.
- Lines beginning with `#` are comments. Destination floor tiles are local32..79; rendering uses a64-tile scene with an eight-tile margin.

Copy a snapshot to compare alternative layouts, materials, supported wall shapes and prop arrangements. Render all four cardinal yaw values0/4096/8192/12288 and inspect entrances from player approach positions. Outputs are raw BMP files; label any contact sheets **OFFLINE**, keeping the originals intact. Cache bytes and generated images remain private workspace inputs and evidence, not repository content.

## Limits and acceptance

The tool calls the unchanged native library's cache decoder, scene assembly, `WorldMesh`, texture loader and software rasterizer. It omits actors, live object animation, hazard graphics, UI, gameplay, personal progression, zone reconstruction and the native viewport's camera/controller behavior. It assembles final runtime objects directly, so baseline object identity and baked/dynamic lighting history may differ from a live cache rebuild followed by zone changes. Its look-at camera and empty scene margin also differ from the gameplay viewport. There is no pixel-equivalence claim.

The renderer prints the four ground vertices around the requested pose. Check these before choosing a playable stance: a raised quay beside lower water can place a player on a sloping edge even when a wide scene overview looks convincing. Protect unsupported edges through genuine authored collision and verify visible footing in native play.

A preview is a rapid selection aid. Check actual source cache clipping and every required interaction approach independently. Every retained production revision still needs genuine native captures from multiple player positions, relevant normal-action gameplay and reconstruction/persistence checks, with before/after evidence scoped to the tested source and executable. Offline images never replace that evidence.

## Static encounter route comparison

`export` also provides `arena_routes`, which compares standing-tile clipping and every directed cardinal/diagonal movement inside the existing Imp and Warden bounds between two exported scenes. It applies the real cache location definitions with the world's normal collision implementation. Both courts have their copied terrain cleared by the authored floor; the water channel is outside these bounds. The check permits visual changes behind the court while detecting changes to playable lanes.

```powershell
cargo run --manifest-path rust-server/tools/tower/offline/export/Cargo.toml --bin arena_routes -- . work/before.scene work/after.scene
```

This is a static geometry check, not a boss, NPC occupancy, telegraph, damage, reward or persistence test. Keep the full actual-cache floor connectivity/hazard-escape audit and genuine native/headless encounter regressions alongside it.
