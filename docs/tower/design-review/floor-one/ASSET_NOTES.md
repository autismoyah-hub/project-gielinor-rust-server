# Assets, materials and publication boundary

The scenes use the existing locally verified cache and unchanged native-client library at `ff9cef17fd1ce3ec2ebf8d1181fee31411999c4c`. These are asset references, not redistributed assets. Source zone aliases in the design compiler are stone (377,545), dark (364,1245), forge (362,1211), moss (212,571), earth (321,649), ash (242,620), water (187,566). Authentic chunks retain their own surface characteristics; material boundaries operate at chunk scale.

| Cache reference | Role | Actual footprint / caveat |
|---|---|---|
| 18491 | Bank | 1×1; Bank operation |
| 409 | Altar | 2×1 before rotation; Pray operation |
| 37947 | Work furnace | 2×3 before rotation; Use operation |
| 2097 | Anvil | Real cache collision, adjacent access audited |
| 11360 / 10943 | Ore | Resource references; placement approaches audited |
| 9736 | Range | Cooking approach audited |
| 16811 | Ceremonial fire | 2×2, no operations; must not occupy camera/recovery marker |
| 18511 | Waterwheel | 1×3, no operations; visual machinery |
| 18526 / 9116 / 9120 | Foundry / Imp infrastructure | Staged behind encounter reserves; not new mechanics |
| 30124 | Coolant marker | Four per Warden court, translated with preserved offsets |
| 6143 / 6145 | Masonry | Actual boundary-shape clipping; low/open walls emphasize B's ruin |
| 7016 / 7017 / 568 | Piers and dwarf statues | Threshold framing and refuge orientation |
| 6915 | Rubble | 3×1 before rotation; only B, outside encounter reserves |
| 1276 | Tree | 2×2; visual reserve. Cached Chop-down operation does not establish the unresolved server binding. |
| 57692 / 57694 / 909 | Portal / chest / plaque | Required service types reserved and accessible |

Complete curated definition summaries and NPC sizes are in `REVIEW_MANIFEST.json`. No archive, raw cache geometry, texture, credential, account fixture or extracted definition dump is published.

The repository [README](../../../../README.md), [provenance](../../../../SOURCE_PROVENANCE.md) and [license](../../../../LICENSE) distinguish software licensing from Jagex content rights. No rendered-media redistribution grant was established. Authentic BMP/PNG evidence therefore stays local under the explicit fallback in the assignment. Public PNGs are newly drawn geometry and typography; they never open cache image/model data. Do not treat this review as permission to upload private renders elsewhere.
