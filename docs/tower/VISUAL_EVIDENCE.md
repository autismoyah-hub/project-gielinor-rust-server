# Visual evidence index

Private evidence is retained locally; no cache assets or binary captures are published here.

- Existing-floor baseline: authenticated native `live-view`, server starting source `2d8e1634683ceb0614fa7c01fa88e26d36089409`, client baseline `95eed88159b06cebea24559a814a5111b54a59cd` plus isolated local adapter. Frame records real server scene, not an offline mockup. Arrival at `[0,10424,120]`; 1754 dynamic location changes applied. Movement confirmation still pending. Camera framing and blank panel need correction. This is diagnostic evidence only, not full floor acceptance.

- Open-court revision: actual optimized native client, player visible after normal character creation. Live receipt records movement `[0,10424,120]` -> `[0,10425,120]`, tick 529. Server includes layout and onboarding revisions; client local adapter commit `4001176` atop pinned baseline. No mouse/keyboard injection or focus activation. Early frame confirms internal partitions removed and service court visible. Remaining issues: large foreground altar, uneven terrain/material seams, sparse landmarks, slow login interface setup and a stale tutorial guide panel (handoff close fix added for subsequent validation). This does not demonstrate completed encounters.

- Native completed services and Ash Imp: real inventory, forged dagger, cooked food, smoke/eruption graphics and ordinary combat. Accepted Imp frame sequences 315, 555, 699; native action receipts retain authoritative kill credit.
- Refined native Warden: sequences 1676, 2007, 2268 show warning/phase progression; 2604 records the confirmed kill message and pending sanctuary reward. State reports all three services, Imp and Warden complete, victory count one, first reward unclaimed. A subsequent normal walk moved to `[0,10424,128]` (frame 2615). The guardian can respawn globally; personal run credit does not reset with NPC respawn. These are genuine framebuffer captures from the isolated native renderer with no desktop input. Terrain seams remain a final polish item.

- Final material/journal revision: native character legitimately on run two after first-clear payout. The ten-point native tour records frames 42 (pasture), 80 (southern court), 121 (poultry), 152 (waterworks), 189 (sanctuary), 232 (workshop), 273 (grove), 305 (foundry entrance), 338 (vent gallery) and 369 (return). Reviewed actual frames show distinct flat court materials, intact outer shell, cache-authored landmarks and readable current journal. They are not offline renders.
- Crash/relogin native review retained 500 coins, beer, forged dagger and paid history; actual native durable frame 17 demonstrates the revised journal. Headless recovery and Warden files contain structured observations only. Final replay victory/payment frames remain pending.

## Accepted final evidence

- Native final replay Imp (readable four-tick timing): actual frames 457, 545, 686 in the private native-replay-final session. Phase two and authoritative victory are visible; no food was consumed.
- Native final replay Warden with resilient snapshot I/O: actual frames 69, 324, 607 and 844 in native-robust show the telegraph progression, crossed lanes, final phase and victory. All three phases were accepted, ending at 10 HP with all 18 prepared food retained. Actual frame 886 records the 25-coin replay claim, 525 total coins and two beers.
- Final post-restart native frame 35 in native-accepted-final shows all six journal rows complete, run two, two victories and first reward claimed. The genuine native window remains connected there; no desktop input or activation was injected.
- Clean headless guardian files explicitly declare `mode: headless`, an empty `frame_sequences` list and observation sequences only. No headless image is presented as native evidence.
- Forced read-only two-second snapshot-sharing fault passed with a 2.09-second ACK delay, normal axe/dagger swap and restoration, and continued real native rendering.

All listed frames were reviewed from the real composited raster. Binary captures and complete raw session data remain private and are not included in Git publication.

## Visual production pass: genuine intermediate diagnostics

Pinned pre-redesign server is 34bb31d; baseline native tour is `work/native-visual-before` frames 51/91/126/155/183/220/257/289/327/364. Source-only frame masks were verified against the real native scene inventory; there are zero remaining conveyor-frame objects after correction. Plaque and portal now have accepted shape-4 and shape-0 entries.

`work/native-visual-production/frame-92.bmp` shows the distinct spiked fire-giant Warden without inherited conveyor frames. Fresh native Imp victory is documented by `evidence/native-visual12-final/imp-production-summary.json`: full HP and 19 food retained; genuine frames 71/170/266. Warden frames 297/528/892 in that same native directory show lane/cross/diagonal phases but belong to a failed food-exhausted attempt and are **presentation evidence only**. They are not counted as a win.

Foreground masonry comparison: `work/native-visual12-final/frame-266.bmp` obscures the avatar; `work/native-readable12/frame-703.bmp` shows the normal avatar through lower southern masonry. Displayed Combat Options → Lunge is genuine native frame 684 in `work/native-readable12`; Strength XP was observed increasing from actual combat.

The final production walkthrough uses `tools/tower/visual_walkthrough.py`: twelve spaces, each at yaw 0/4096/8192/12288. Its full current-source gallery, corrected Warden victory, full reward loop and lifecycle regression are still pending at this checkpoint. Images, raw scene CSV and account/session details remain private.

## Reviewed reconstruction checkpoint 164b (superseded artistic baseline)

Actual native gallery: local `evidence/visual-gallery-164b/gallery.html`; machine-readable walkthrough lists all 48 original BMPs, twelve areas at yaw 0/4096/8192/12288. Arrival 815/817/819/821; sanctuary 843/845/847/849; plaque 866/868/870/872; workshop 915/917/919/921; moss court 955/957/959/961; hall 999/1001/1003/1005; poultry 1044/1046/1048/1050; waterworks 1077/1079/1081/1083; grove 1144/1146/1148/1150; vestibule 1189/1191/1193/1195; Warden 1214/1216/1218/1220; Imp 1258/1260/1262/1264. Original captures are in `work/native-clear12`. All four angles of all twelve areas were visually reviewed. The latest user mandate judges these spaces too sparse; this is the next reconstruction's before-reference.

Native current Warden win: phase views 69/283/518, victory in arena 638, retreat 651, same private native-clear12 directory. 13 HP, 19 food retained. First reward capacity/duplicate and exact portal results are in `evidence/native-clear12`. Headless current Warden requires a new normal coolant message UID (8); its first claim and exact exit/return passed in `evidence/headless-clear11`. Headless records are observations, never native images. New fresh headless/replay results will be indexed separately, and intermediate failures stay failures.


## Fallen Cinderworks draft evidence (not artistic acceptance)

Private current evidence: `evidence/fallen-refined-native-gallery/visual-walkthrough-summary.json` indexes fourteen locations and four settled cardinal angles, with normal journal UI; captures are in `work/native-fallen12b`. The gallery includes a close sanctuary hearth and sluice-wheel approach in addition to the twelve principal areas. Genuine first-draft wheel close-up is native-fallen12 frame302; its inland placement caused the river-contact refinement.

`evidence/headless-fallen16` contains actual ClientState scene CSV checks before services and after workshop, kitchen and altar. Observed IDs progress from [18525,18512,16810,16810,16811] through [18526,18512,16810,16810,16811], [18526,18511,16810,16810,16811], then [18526,18511,16811,16810,16811]. The central hearth is deliberately still unlit before final victory. The separate optional Search lesson preserves inventory/XP/health and grants no progress. Current full encounters and later lifecycle checks must be recorded separately; old 164b successes are not substituted.

Completed 164b native replay coolant is actual native-resume12 frame572; victory frame1140. Fresh headless 164b complete route is in evidence/headless-visual13. Exact no-logout crash survival and post-restart duplicate denial are in evidence/visual-164b-crash-relogin.json, with prior read-only database verification. Original failed diagnostics are retained. All raw captures, scene CSV, account/session data and logs remain private.


### Known defects in the current draft gallery

Native-fallen12b frame535/537 shows global peer warning/eruption effects in an otherwise completed Imp court. Native frame451 and corresponding cardinal views show the mismatched broken-wall models. Both are retained as failed visual diagnostics. The corrected source must produce new native evidence before these items can be marked fixed. Native-fallen15 service scene files prove kitchen-first ordering; stable-journal confirmation is separate from the original failed immediate aggregate log. Headless-fallen16's first Warden timeout is not a victory.
