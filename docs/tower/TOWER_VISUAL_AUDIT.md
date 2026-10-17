# Cinderworks rendered spatial defect ledger

Status: visual production pass in progress; artistic acceptance remains with the user.
Baseline server: `34bb31d1b5a48449b3c960951ec43f93b4f17fd9`.
Baseline client adapter: `a32b66deeeb30a2be159fbd379a9ffb07f423b72`.
All coordinates below are absolute ground-plane coordinates. Evidence consists of genuine native composited frames held locally, never generated illustrations. Frame references use `work/native-visual-before/frame-N.bmp`.

| ID | Area / coordinates | Actual native evidence | Severity / cause | Intended correction / current state |
|---|---|---|---|---|
| V01 | Entire floor, 10400-10447 / 96-143 | Tour frames 51-364 | Major: internal partitions deliberately removed | Reconstruct actual masonry and thresholds; implemented, final native review pending |
| V02 | Southern court, 10424 / 104 | 91 | Major: goblins, bookcases and dining furniture repeat the original nine-room prototype without a coherent room | Replace with southern processional hall, side openings and useful stores; remove obsolete goblin population and staff crate |
| V03 | Sanctuary, 10424 / 120 | 183 | Major: unused mind altar, crate, goats and furnishings lack architectural setting | Remove obsolete gameplay props; clipped-corner sanctuary anchors bank, work branches, altar, portal and reward |
| V04 | Workshop, 10406 / 118 | 220 | Major: no enclosure; repeated earth bands and scattered props resemble a test area | Enclose forge wing, frame its resource/workbench axis, preserve normal service access |
| V05 | Grove, 10408 / 136 | 257 | Moderate: repeated trees and green floor abruptly meet ash | Retain managed resource court, define threshold and northern wing boundaries |
| V06 | Foundry, 10423 / 132 | 289 | Major: oversized cage-like braziers overlap approach and coolant; no arena enclosure | Remove all tomb braziers, bones clutter and incompatible props; construct real arena walls and clear entrances |
| V07 | Vent gallery, 10438 / 132 | 327 | Major: same oversized brazier obscures silhouette and route; no staging | Distinct smaller vent court, compact light sources and unobstructed combat field |
| V08 | Boss silhouettes, 10424 / 136 and 10440 / 136 | 289, 327 | Major user concern: weak identity. Audit finds existing definitions already differ (two-tile stone guardian versus one-tile red imp); different model bytes alone did not create convincing presentation | Improve arena composition, compatible animation and explicit rendered phase/quench cues; validate multiple angles |
| V09 | All masonry | Source and native perimeter | Major: `threshold_wall` has a full-tile model duplicated with shape 10; collision preparation runs after stamping and would erase new thin-edge walls | Use cache wall 2341 with supported straight/corner shapes; clear inherited clipping before applying actual edges. Test symmetric movement/range masks |
| V10 | Upper stub | Construction template and population | Moderate: unreachable plane-one bank/stair/portal scene remains allocated | Remove unused upper template, objects and NPC; no inaccessible prototype scenery |
| V11 | River, 10439 / 116 | 155 | Moderate: raw copied bank visibly interrupts stone court; valuable real fishing terrain | Preserve water clipping and fishing, frame waterworks entrance and quay with authored geometry; inspect seams after reconstruction |
| V12 | Boundary / arrival / exit | Extended audit captures indexed locally | Pending extended review: neighboring geometry, wall views, occlusion and camera clearance | Four-angle perimeter and arrival audit; repeat after reconstruction and lifecycle rebuild |

## Construction trace

`constructed::build_small` copies terrain and source locations. `cache::tower_shell::prepare` removes all copied locations inside the authored floor on planes zero and one, keeping the baseline for client zone deletion deltas. `server::tower_map::prepare` then stamps prototype-era furnishings back in. The principal remnants are therefore authored reintroduction, not a failure to delete copied cache objects. Publication loads/deletes the exact cache baseline before installing the final scene; this must remain stable through region reload.

No renderer bypass or Tower-specific occlusion change is proposed. Thin cache masonry uses existing native scene rules. The previous full-tile boundary convention is being replaced with actual wall edges, including corner models, and collision is installed in the same order.

## Root-cause corrections discovered during iteration

The first attribution of the cage-like frames to tomb braziers was incorrect. Read-only native scene export found 38 surviving conveyor pieces (9109–9113); the paired game map contains none at those anchors, while JS5 does. Exact construction baseline masks removed all of them in the next real native rebuild. Count: zero remaining, confirmed in `work/native-visual-production/scene-objects.csv`. No renderer alteration was needed.

Two additional hidden defects were found by comparing native and authoritative inventories: plaque 909 only supports wall-decoration shape 4; portal 57692 only supports wall shape 0. Both had previously been stamped as unsupported shape 10, so client-driven packet tests could interact despite missing scenery. The plaque now has supporting masonry; the portal is integrated in the eastern wall at (10432,124). Native inventory confirms both have real accepted scene entries. Plaque facing is being rechecked at cardinal angles.

Camera audit correction: yaw uses 16384 units per turn. The early extended baseline used 0/512/1024/1536, which were small angular changes, not four cardinal directions. The production walkthrough explicitly uses 0/4096/8192/12288. Recorded observed positions are retained even when an early perimeter destination was not reached.

The initial architectural regression passed normal services, lethal recovery and Imp victory. Its Warden attempt failed on a stale/hidden food widget and is not counted as a completion. The controller now reopens the ordinary inventory and reacquires food after a rejected dispatch; actual accepted consumption and full encounter outcomes remain required.

Further native combat review exposed blocked diagonal escape corners when coolant was placed directly beside melee tiles. Intermediate food-exhausted Warden attempts remain failures. Coolant is moved to four corner stations with a clear buffer lane, and the controller rejects diagonal moves whose cardinal corners are occupied. Ordinary aggressive dagger training is selected through the visible combat panel; Strength XP was observed increasing from real hits. No damage or hazard timing is changed to force acceptance.

The normal storage barrel had no native Search action. The selected rainwater variant 30124 has genuine Search operation 1 in the read-only client cache and accepted native scene entries. This closes a gap in earlier packet-only interaction evidence. Duplicate shell-denial chat was also causing unnecessary UI work and obscuring phase feedback; it is suppressed while the shield rejection itself remains authoritative.

Foreground high walls obscured the avatar at the foundry threshold in an intermediate capture. Low stock masonry on southern room edges fixes the view using ordinary architecture, with movement/range clipping taken from each actual cache definition. Native comparison: old threshold frame266 in `work/native-visual12-final`, readable threshold frame703 in `work/native-readable12`.
