# Observations established by the selected reconstruction tests

These observations supplement the earlier modeling guide; they do not approve
the existing floor assets or authorize another production pass.

* Native packed HSL values and per-face shading metadata are part of the authored
  appearance. A visually plausible Blender material is not a fidelity guarantee.
* Preserve seams, duplicate coordinates and original triangle topology in a
  technical round trip. Automatic welding and normal cleanup can change smooth
  shading even if the silhouette survives. Cleanup is permitted on new independent
  geometry, where its effect must still be judged in native rendering.
* The selected wall is 132 by 240 by 128 native units. Its 114 faces include open
  edges; watertightness is not a universal authenticity criterion. The independent
  closed extrusion has different normal weighting and visible shading differences.
* The selected goblin is 191 by 158 by 95 units in its unanimated base composition.
  Body-only height is not a universal player scale. Equipment changes the bounds;
  compare anatomy and equipment separately before changing overall scale.
* Correct equipment orientation matters. Turning a broad axe through the wrong
  plane can inflate model width while leaving a plausible-looking screenshot.
  The independently corrected equipment improved all three measured silhouettes;
  that does not prove artistic likeness or material fidelity.
* Sample actual palette assignments by part when investigating a color mismatch.
  Reusing a color found elsewhere on the same creature does not make it suitable
  for boots, armor or metal. The selected boots and axe use different palette
  families from the initial independently authored guesses.
* Classic animation pivots derive from the currently transformed vertices. A
  newly modeled rig with larger rigid groups changes pivots and therefore motion,
  even if some angle values or action timings are similar. Authoring convincing
  movement needs its own review after the converter is known to preserve data.

The confirmed reusable standard is to preserve channels explicitly, reopen the
actual Blender file, export evaluated data, decode it independently and compare
multiple native poses/cameras. Broad Floor One generator changes remain deferred.
