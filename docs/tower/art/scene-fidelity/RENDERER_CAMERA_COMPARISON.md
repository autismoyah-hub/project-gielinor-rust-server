# Camera, terrain and native rendering comparison

The old whole-floor pictures were design diagrams. They were unsuitable as direct comparisons with normal gameplay views. Matching the camera reduces that presentation difference, but it does not erase the remaining artistic differences.

| Setting | Earlier custom close preset | Shared scene review preset |
|---|---|---|
| Projection path | Free `Camera::look_at` | Native fixed-angle `Camera::angles` |
| Viewport | 1200 × 900 | 765 × 503 |
| Focal | 1000 | 308, calculated by native `viewport_shape::Settings::default().shape` |
| Pitch | About 31.43° for horizontal distance1800/elevation1100 | 56.25° (2560 native units), within normal client limits |
| Orbit | Arbitrary horizontal distance/elevation | Native `camera::position`, orbit1560, zoom[256,320] |
| Focus height | Fixed authored target, typically −220 | Actual native terrain elevation minus50 |
| Brightness | 0.8 | 0.8 for both cache and original assets |
| Far clip | Tool default3500; free-camera bounds path differs | Native default device option: 25 × 210 =5250 |
| Roof choice | Build plane0 | Explicit hidden-upper-roof plane0 for both reference and custom views |

The shared viewport gives horizontal field of view about102.3° and vertical about78.5°. The earlier close preset was approximately61.9° ×48.5°. This explains a material part of the difference in apparent prop size and surrounding context. It is not evidence that the old models themselves were correctly proportioned.

The preliminary matched run used pitch45°/orbit1368. The final focused review uses56.25°/orbit1560 **for every authentic reference, baseline and candidate frame**. It is a normal permitted camera configuration, not a custom-only lighting or camera adjustment. Additional low-pitch views are explicitly diagnostic. Every frame stores actual focus, eye, native angles, zoom, focal, orbit, scene/registry/renderer hashes and the camera constructor's parameters.

## What is shared with the live native client

The matched preset calls the same native viewport-shape, orbit-position and fixed-angle projection functions used by `live_runtime`. It uses the same `WorldMesh`, face lighting, native palette, rasterizer and default device far clip. Real references use `Scene::load_normal`; authored sample terrain uses native `Region`/`Scene::add_chunk`, not an alternative floor renderer. The sample also passes placements through normal scene insertion bookkeeping. No global light, palette, exposure, background or rasterizer override is introduced for custom art.

This is source-backed use of native rendering, not a claim of full live-client parity. A live interface's CS2 scripts may change viewport settings; live roof selection, actors, stateful transforms, animation, input and skybox selection are omitted. The explicit roof cutaway and fixed camera are controlled study conditions. The old free camera omits fixed-angle model-frustum state, another reason not to treat broad overview pictures as gameplay views.

## Terrain diagnosis

The unapproved complete floor used flat-shaded ground *models*. Its face colors and lighting did not follow native terrain underlay blending or per-corner terrain lighting. The sample now authors original RGB underlays5000–5004, overlay5010 and explicit height deltas. These source numbers are original project data, not extracted cache floor definitions. They are accepted only by the package-only offline study source; an ordinary cache reader cannot install them or request a cache texture through them.

Native terrain then performs its existing neighborhood HSL blending, height-gradient lighting, model-cast terrain shading and tile triangulation. The original package model codec/rasterizer are unchanged. This is a rendering-input correction, not a lighting trick or an assertion that a flat grey surface automatically establishes OSRS art quality.

The controlled `uniform-material` scene retains height and placement records while replacing terrain materials with one underlay and no overlay. The `flat-height` scene retains material and placement records while flattening heights. Native terrain lighting changes with height, and props attached to raised terrain follow their normal base-height calculation; the latter is therefore not an isolated color experiment. Both controls are rendered at the identical native camera configuration.

## Background and missing context

The native frame's default dark background is retained for both reference and custom renders. No sky or black area is painted over. Under the final matched front camera, the exact background-color fractions are about1.44% industrial,4.26% dungeon,0% river and1.07% unchanged Tower baseline. The candidate's quarter view is about0.31%. These are descriptive exact-color counts, not quality/similarity scores; an opaque surface could also contain that color.

Consequently the dark background does **not** explain most of the residual mismatch in matched close views. The huge blank margins of the earlier whole-floor diagrams were a separate framing issue. The initial48-tile sample lacked enough surrounding terrain; the reviewed64-tile sample adds authored native-height boundaries and keeps the inhabited suite compact. Camera and buffer size both changed between preliminary and final studies, so that before/after must not be used to isolate either effect by itself.

## Preserved failures

One previous complete-floor connecting-ground model decoded correctly but exceeded the native renderer's model-radius bound and disappeared. The preserved complete-floor study was fixed by stable22-tile partitions; the converter now rejects radius>=2900, below the native3000 limit. The renderer was not weakened.

The all-smooth wall experiment rounded closed masonry edges into an undesirable padded appearance. Selective smooth planes plus native scene shading also produced excessively dark directional faces. Both pass technical validation and are rejected artistically for this candidate. Their images remain in the review appendix. The retained sample uses the clearer flat wall treatment and lists a more faithful wall-topology/normal study as an unresolved task. Copying a native render-type value from a different topology is not sufficient.
