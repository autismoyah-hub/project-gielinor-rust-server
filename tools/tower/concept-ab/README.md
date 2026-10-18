# Ashhorn Stalker: concept-guided modeling pilot

Start with `art/tower/concept-ab/index.html`. Evaluate the randomized A/B models
before using its explicit technical-reveal control. The private local review
also contains authentic cache-backed courtyard views; those images and cache
resources are deliberately excluded from this repository.

This is one original creature, one frozen specification and two independently
authored static meshes. The text-only run was frozen before image generation.
The image-guided run began from an empty scene. Neither imports a previous
creature mesh, competing model or original cache geometry. Native integer units
are128 per tile. No renderer, live cache, gameplay, animation or NPC definition
was changed.

## What is established

* A SHA-256-frozen original design and a pre-concept text-run freeze.
* Five successful text-run modeling passes; six concept-run passes and one
  rejected out-of-surface eyebrow attempt. Every candidate remains local.
* Matching native front, side, rear, quarter, elevated and gameplay views;
  matched face/armor close-ups and twelve supplementary Cycles views.
* Original sources, recipes, initial/intermediate/final native checkpoints,
  original generated hero and both turnaround variants, correction prompts,
  reference calibration, explicit conflicts and measured effort.
* Both final saved Blends re-export byte-exactly. Fresh recipe generation
  reproduces the native bytes and all six original native view pixels exactly.
  Native decoding preserves vertices, triangles, colors and render flags.
* Two isolated static-model registry entries and diagnostic object proxies
  decode exactly. These IDs are not installed NPC definitions or globally
  reserved production identifiers.

The modeling/codec libraries remain at baseline
`8b85f19ce2d70988b0724f1d03e241c29a45daf6`. The experiment-specific generators
use its semantic surfaces, transported limbs, masked sculpting, union, local
relaxation, mesh construction, solidify and face-color exporter. The concept
run additionally authors interior-sampled armor cages with the same low-level
mesh/solidify helpers; this extra effort is explicitly a comparison confound.

## Findings and limits

The image-guided model has more deliberate angular facial planes and layered
armor. Both models still have simplified profile-cage anatomy and lack the
painting's sculptural specificity. This run does **not** establish that concept
generation solves the artistic quality gap or justify mandatory pipeline
adoption. User visual preference and artistic approval remain pending.

The image-guided run received more elapsed effort and one extra successful
iteration. Reported time is subprocess execution and UTC authoring envelopes,
not human labor or precisely measured active work. There is one sequential
trial per workflow. The generator automatically displayed its imagery, so the
final randomized review reduces attribution bias but cannot be fully blinded.

Generated images disagree about paw counts, width and tail shape. The second
turnaround corrects visible toes, and a recorded canonical interpretation gives
the frozen numeric dimensions priority. The frozen design itself contains a
coordinate/laterality-label conflict: both meshes preserve identical explicit
horn coordinates; final native renders are never mirrored to conceal it.
Reference image planes were packed after initial concept-run landmark work,
rather than before all modeling. This ordering limitation is recorded.

Source-to-native quantization drops179 and201 collapsed triangles. No
degenerate triangles remain in final native outputs. Open/detail seams,
simplified joints and static-only sources are reported; animation readiness,
watertight physics and final artistic approval are not claimed.

## Rebuild the original meshes

Requires the held baseline source dependencies, Blender5.2 (tested5.2.2LTS,
build`d13f752e3b9c`), and Python with NumPy/Pillow for evaluation. Geometry
generation needs no game cache and uses factory-empty scenes. Find the text
and concept label assignments in the technical reveal **after** blind review.
Use fresh output directories; source authoring refuses overwrites.

```powershell
# Text-only recipe: substitute the label disclosed in the technical reveal.
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' -b --factory-startup --python-exit-code 1 --python tools/tower/constrained/surface.py -- - art/tower/concept-ab/models/<TEXT_LABEL>/recipe.json work/text-rebuild

# Concept-guided recipe, including its original packed image-guide arrangement.
& 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe' -b --factory-startup --python-exit-code 1 --python tools/tower/concept-ab/build_concept.py -- art/tower/concept-ab/models/<CONCEPT_LABEL>/recipe.json work/concept-rebuild art/tower/concept-ab/technical/original-concept
```

`method_text.py` and `method_concept.py` produce the recipes from the same
frozen specification. `build_concept.py` constructs the concept-specific plate
cages from its own independently authored anatomy and adds non-exported image
empties and orthographic cameras. All exported creature surfaces are actual3D
meshes; native conversion samples no image textures or billboards.

`run.py`, `bootstrap.py`, `prepare_reference.py`, `verify.py`,
`evaluate_experiment.py`, `package_assets.py` and `build_review.py` preserve the
controlled local experiment. They expect the existing sibling workspaces and
held read-only cache for native rendering; they do not download resources.

## Native environment inspection

`tools/tower/offline/render/src/bin/concept_ab_scene.rs` is a read-only scene
submission adapter. It loads authentic revision239.1 terrain and scenery,
places either original static mesh at the same tile center/terrain height,
then calls the existing native renderer. No render feature or renderer source
was changed. Native standalone views use the frozen existing workbench binary.

```powershell
$env:CARGO_TARGET_DIR='work/concept-ab-build'
cargo build --offline --release --manifest-path tools/tower/offline/render/Cargo.toml --bin concept_ab_scene
# Keep these cache-backed outputs private:
work/concept-ab-build/release/concept_ab_scene.exe <HELD_JS5_CACHE> <ORIGINAL_MODEL> <PRIVATE_OUTPUT> 3222 3218 quarter
```

The normal gameplay comparison uses765×503, focal308, pitch2000, yaw14500,
orbit600. Five close inspection views use1000×900/focal1000 at the same fixed
origin and target height84. Earlier distant gameplay views at orbit1008 are
preserved separately. Ambient72, contrast850, light[-30,-50,-30] and native
palette brightness0.8 are identical for both. Cycles32/AgX is supplementary.

The public package contains original meshes/art, source-safe manifests and
standalone original renders only. Runtime palettes, extracted reference
geometry, cache textures and authentic courtyard screenshots remain local.
