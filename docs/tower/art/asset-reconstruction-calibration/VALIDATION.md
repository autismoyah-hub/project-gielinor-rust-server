# Validation record

* Release native diagnostic build passes against client source
  23a94f75707f0503a9efe610fe99bfce5089b575. Existing vendor warnings are unchanged.
* Wall original versus saved/reopened Blender round trip: 7 native views, every
  pixel exact; geometry, face indices/winding, packed colors and effective metadata
  agree. The only scalar exception is the unused global priority fallback when
  per-face priority arrays are present.
* Goblin original versus saved/reopened Blender round trip: 560 native renders,
  every pixel exact. Every compared posed coordinate and alpha value also agrees.
  Counts include three bind cameras, four draw orientations, 504 single-action
  timestamps and 49 masked action/walking timestamps.
* Geometry/color/action edit witness passes after a fresh Blender reopen. Changes
  to actual Blender mesh, material and action channels reach exported interchange.
* Seven new codec boundary/determinism tests pass: explicit quantization boundary,
  skin-label bounds, unsupported weighted skeleton rejection, texture rejection,
  triangle bounds, signed-smart overflow and deterministic output.
* Complete workflow reproduction: 1,930 native image/model/animation files agree
  byte-for-byte. This checkpoint precedes the final independent material/motion
  refinements. Final independent authoring is separately regenerated in a fresh
  Blender process: all 111 package/definition/frame/skeleton/sequence/registry files
  agree byte-for-byte. Blender file byte identity is not claimed.
* The independent death action's maximum Y is zero at inspected falling samples
  0,30,60,87,90. At 91 and 390 all vertices are zero. Both original and independent
  references enter the terminal disappearance at tick 91: native clock transitions
  use `elapsed > length`, not `elapsed >= length`. The retained prior failure had
  a suspended corpse and no terminal disappearance.
* Final read-only cache audit: all 50 files / 453,219,756 bytes have unchanged
  SHA-256 values. Protected baseline server/client heads remain unchanged.
* Private review link check passes for all 18 media/report references.

The independent wall's diagnostic silhouette intersections are about 0.920,
0.980 and 0.921 in the three matched views. The independent goblin's are about
0.602,0.679,0.598 after correcting equipment orientation, compared with
0.585,0.611,0.564 before it. These are mask overlap measurements, **not artistic
acceptance scores**. Color, surface treatment and choreography remain visibly
different. In particular, a close silhouette is not proof of convincing OSRS art.

Native binary SHA-256:
`c1acb0fca33773ea91f284bb4c3a55aeb61bfb0e52538318d9d8405ded2b7892`.
The aggregate JSON provides counts, bounds and comparison results without raw
reference geometry or screenshots. Full source payload hashes, native frame
arrays, Blender sources, raw renders, GIFs and failure evidence remain private.

This is a tested conversion checkpoint and an unapproved artistic study.
It is not completion of general animated asset support, artistic acceptance,
full-floor approval or gameplay integration.
