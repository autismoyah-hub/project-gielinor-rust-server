# Diagnostic difference ledger

| Difference | Classification | Evidence / disposition |
|---|---|---|
| Previous GLB subset cannot encode skin/face labels, priorities, depth flags or animation | Conversion limitation | Preserved old contract; new explicitly bounded diagnostic interchange/codec carries these channels; native comparisons pass |
| RGB conversion could change packed HSL values | Color conversion risk | Integer native HSL is stored in Blender materials and exported directly; all source face colors agree exactly |
| Per-face priority encoding changes unused global fallback | Format constraint | Effective priority arrays agree; matched native pixels agree; not represented as a byte-identical source payload |
| Initial independent wall's extrusion axis was wrong | Modeling / orientation | Retained v1; corrected by an independently drawn X/Y profile extruded along Z |
| Initial independent geometry contained inward faces | Geometry authoring | Retained failure; corrected normals on independent meshes only; source and round-trip faces are never cleaned or rewound |
| First custom static-definition registration used an incompatible shape terminator | Definition encoding | Native decoder rejected trailing bytes; changed to explicit shaped opcode 6; native binding and collision checks now pass |
| First survey selected old goblin combat clips | Reference-selection error | Final equipped NPC uses matching 618x family and armed attack 6185; old survey retained privately and excluded from final reference evidence |
| Independent wall has different molding, topology and vertex-normal distribution | Modeling / shading | Matched native views retain the visible discrepancy; changing lighting would conceal it. No artistic approval asserted |
| Independent goblin differs in anatomy, armor, equipment and facial treatment | Modeling | New independently authored geometry is compared against the original at identical scale/camera; appearance remains for artistic review |
| Independent actions have different poses and pivots | Animation authoring | Compact classic group rig and authored actions work in native code; matching durations do not imply matching choreography |
| First independent death pose remained elevated and did not disappear | Animation authoring | Retained failed native frames; final action falls forward with authored ground correction and a terminal native scale-to-zero operation. The original reference's terminal vertices are also zero; no sprite or renderer hide flag is substituted |
| Original vs converted native frames | No measured mismatch in tested scope | Wall 7/7 views and NPC 560/560 renders pixel-exact; posed-coordinate and alpha comparison also exact |
| Genuine asset vs larger Cinderworks environment | Scene context | Not retested by this asset study; prior floor work remains preserved and unapproved |

No renderer adjustment was made to improve these comparisons. Shared rendering
eliminates some variables but is not independent proof of the Rust client's parity
with Jagex. Technical round-trip success is not artistic reconstruction acceptance.
