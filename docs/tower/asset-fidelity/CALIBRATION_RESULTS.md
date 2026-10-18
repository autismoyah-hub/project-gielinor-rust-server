# Development calibration

The previous pillar, waterwheel, industrial anvil, goblin combat and cow studies
are preserved, including failures. This cycle targets the exposed construction
weaknesses. Exact revision 239.1 reference model IDs/payload/definition hashes
and five-view aggregate results are in PRODUCTION_RESULTS.json. All are unapproved.

| Development reference | Prior selected mean IoU | Revised mean IoU | Interpretation |
|---|---:|---:|---|
| item 1323 scimitar | .331 | .549 | Curved blade improves; fittings remain poor |
| object 2097 anvil | .752 | .769 | Horn/waist construction modestly improves |
| object 362 barrel | .893 | .905 | Better volume and separate staves; grain mismatch |
| object 366 crate | .928 | .885 | More joinery, measurable silhouette regression |
| item 1171 shield | .739 | .711 | New board/back structure, overlap regression |
| NPC 106 wolf | .638 | .622 | New anatomy still inadequate; regression |
| NPC 2834 giant bat | unsupported; first new .143 | .411 | Wing family supported, not convincing art |
| NPC 655 goblin | .672 | .634 | Revised face/armor/grip remains a negative result |

The bat's .143 failure fitted a full envelope including helper/shadow geometry.
Alpha/zero-area filtering was insufficient because an opaque helper triangle
still reached ground. Explicit private anatomical skin bounds fixed gross
scale; the old failure remains preserved. This is a known-asset development
annotation, not automatic unseen asset understanding. Arch object 1868 exercises
radial masonry and remains an unapproved study.

Internal material/edge/component results are intentionally separate from IoU.
Scimitar blade region .633 but fittings .189, anvil horn .577 and barrel wood
.605 identify remaining contour and surface failures. Goblin gait controls and
contact tools improve authoring reach; they do not establish authentic humanoid
choreography. The original native source corpus still passes exact preservation.

These are selective improvements with real regressions. They justify further
controlled authoring and review, not mass production. All five native angles,
A/B/C and earlier retained versions remain in the protected local review.
