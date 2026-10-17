# Floor One offline design workbench

This tool compiles isolated candidate JSON into native-library scene input and original geometric review diagrams. No server loads these configurations. It never opens accounts, changes cache files or installs a map. Review sources live in [the Floor One review](../../../../docs/tower/design-review/floor-one/CURRENT_REVIEW.md).

## Reproduce

Use a Tower workspace with sibling `rust-server` and `rust-client-reference` checkouts, Python with Pillow, verified private `resource-staging` inputs and the established isolated Rust toolchain. Baseline server source is `1153b4e7281435e83f60bff5131b4a3f0a8e0023`; native library source is `ff9cef17fd1ce3ec2ebf8d1181fee31411999c4c`. The review commit supplies tooling/configurations and their hashes. Build both standalone tools using the workspace's existing Cargo wrapper, or equivalent isolated Cargo configuration:

```powershell
cargo build --release --manifest-path rust-server/tools/tower/offline/render/Cargo.toml
cargo build --manifest-path rust-server/tools/tower/offline/export/Cargo.toml --bin design_audit
python rust-server/tools/tower/offline/design/generate.py --config rust-server/docs/tower/design-review/floor-one/candidate-b/config.json --output work/design-b
# Use the executable path returned by your isolated target directory:
work/target/debug/design_audit.exe . work/design-b/candidate.scene > work/design-b/audit.log
python rust-server/tools/tower/offline/design/generate.py --config rust-server/docs/tower/design-review/floor-one/candidate-b/config.json --output work/design-b --diagram work/design-b/diagram.png --audit work/design-b/audit.log
python rust-server/tools/tower/offline/design/render_review.py --renderer work/target-client/release/tower-scene-render.exe --cache resource-staging/server/work/integration/a-kotlin/.data/cache/js5 --configs rust-server/docs/tower/design-review/floor-one --private-output work/design-renders
```

Commands run from the Tower workspace, one directory above the server checkout. Resolve the renderer path if necessary. The actual audit uses the local game cache; renderer uses JS5. These protected dependencies are not distributed by this review. Python diagrams currently use Windows Segoe UI fonts. Substitute an available local font on another platform; font substitution changes diagram hashes.

Inspect the audit exit code before proceeding. A diagnostic log containing zero issues does not replace successful process completion. `--audit` draws the checked tour's real clipped path. Without it, the gold line is only the configuration's route intention. `T` and `L` are native scene records; `P`, `A`, and `F` are audit metadata ignored by the renderer. `composition.json` records source/config/scene hashes and references. Raw BMP originals, lossless PNGs, camera sidecars, stdout and render manifest are retained privately.

## Cameras and review

Each candidate receives the same three whole-floor cameras: centre (56,56), yaw 2048/10240, distance 9000, `overview`; centre (56,56), yaw 0, distance 1, `plan`. Detail views use per-candidate functional approach coordinates, distance 1800 and `detail`; exact values are in JSON and the manifest. Detail is 765×503, overview 1600×1200, plan 1600×1400. The four ground vertices are printed for stance inspection. Explicit presets support fast composition switching; no native camera/controller equivalence is claimed.

The renderer uses the unchanged native cache decoder, Scene, WorldMesh, textures and rasterizer. It omits actors, active animation, hazards, interfaces, personal state and live zone lifecycle. Detached final locations differ from source-copy removal plus live deltas. Lighting history, look-at camera and empty margins also differ. Static collision/hazard checks do not validate moving bosses, timing, rescue alternatives, persistence or real play duration.

## Publication

Only our original schematic geometry/typography is published. Genuine cache-rendered pixels remain local because no redistribution permission was established. Do not stage output folders, raw assets, caches, credentials or account fixtures. The software license is not an asset-media grant. User direction selection remains open; never copy a candidate into production simply because it passes static checks.
