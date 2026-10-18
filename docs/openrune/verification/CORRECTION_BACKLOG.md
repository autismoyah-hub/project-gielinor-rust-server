# OpenRune correction backlog

## Gameplay findings

No confirmed gameplay defect from the completed initial scoped tests. This is not acceptance of untested behavior. No critical or dependency-blocking finding currently requires FORGE to stop or change its implementation sequence.

## PG-INFRA-001 — empty super-combat test selection

- Class: test-infrastructure blocker; verification harness only.
- Feature: OR-MIG-0010.
- Exact implementation SHA: `5a260c1bd6efa5f8a221bed7c96e6ac72fd428bc`.
- Reproduction: `cargo test --offline --locked -j 1 -p gielinor-world --lib super_combat`.
- Expected: execute four-input mixing and its authoritative action tests.
- Actual: Cargo compiled successfully but selected zero tests. This supplies build evidence and no gameplay acceptance.
- Cause: initial PROVING GROUND filter used a feature name instead of the module name.
- Correction: use `combat_mixing` for both world and server filters; count selected tests and reject zero-test runs.
- Evidence: `independent/OR-MIG-0010-5a260c1bd6ef/` and its subsequent attempt directory.
- Implementation blocking: no. No FORGE production correction or GitHub issue is appropriate for this verifier-owned defect.
- Status: resolved. The corrected filter reran at the same exact SHA: four world tests and one server test passed. The initial empty-selection logs and ledger row remain preserved. The original runner's raw manifest reported `PARTIAL` from exit codes alone; the ledger explicitly classifies that attempt as `BLOCKED`, and the corrected runner rejects zero-test selections.

## Open coverage work

All contracts require V0–V4. Native super-combat and chemistry controls, acknowledged-save/restart/relogin recovery, and source/reference subcomparisons now have independent evidence. Complete negative/modifier matrices, interrupted production, applicable crash recovery and remaining content still need acceptance. See DASHBOARD.md for current levels. Coverage gaps are not alleged implementation defects. Where a concrete fixture, dependency or specification prevents execution, append a precise blocked attempt and a contract request rather than substituting another SHA.

## PG-INFRA-002 — food verifier trace and dirty-slot assumptions

- Class: resolved verification-infrastructure defect; no confirmed gameplay defect.
- Feature/revision: OR-MIG-0024 revision2, `a560034a59e766d1faf400b61187fec861d13475`.
- Initial attempt: `independent/OR-MIG-0024-a560034a59e7-independent/` preserves the failed test source, hash, binary identity and logs.
- Actual: ordinary food does not always emit the optional multipart `FoodTransition`; a normal authoritative tick can update inventory synchronization metadata while item slots remain identical.
- Expected correction: assert exactly one real food animation for successful queued consumption and compare actual inventory slots for stale-input item conservation. Keep HP, all XP, delay and replay assertions.
- Retest: corrected independent external crate passed four test functions covering319 fixtures through actual `World::submit` and `World::tick` at the same SHA. The failed attempt remains historical verifier evidence, not an implementation failure.
- FORGE impact: none; no production fix or GitHub issue requested.

## Measured execution deferrals

Chemistry build attempts were deferred during observed compiler contention, including53% CPU. Later one-job builds proceeded with spare CPU and over6GiB available RAM; no other agent was paused or altered. Resource measurements are preserved with the deep campaign. A deferral is not a failed server build or a gameplay defect.

The handoff's `ba455a6` identifier is an informational mismatch: the fetched original contract commit is `ba455a0278f3f0b5ac6cbe377b6a8b83f0e95348`. This did not block inspection or testing, and it is not a gameplay defect.

## PG-INFRA-003 — dynamic-food verifier argument types and synthetic labels

- Class: resolved verifier-only defect, no confirmed production defect.
- Target: OR-MIG-0035 at `1446167370dece10751c4c332929af880259ee23`.
- The first external harness used unsigned variables for signed stat setter arguments. Cargo rejected the test crate; the unchanged production world library compiled. Checked conversions now cover the bounded 1–99 base and 0–104 current values.
- Independent pinned-reference inspection also corrected swapped watermelon/sweetcorn labels in synthetic fixtures (5984 and 5988 respectively). This affects test identity; these fixtures never claimed configured runtime binding acceptance. Earlier source snapshots and logs remain preserved.
- The intermediate corrected policy suite passed three test functions; the final label-corrected suite is a separate attempt. A measured 35.55% CPU deferral between attempts is recorded as execution contention, not a build or gameplay failure. Current completed outcomes are in WORKLIST.json and the attempt manifests.
- No FORGE source edit, stop request or gameplay issue is appropriate.

## PG-INFRA-004 — native food observation timeout; diagnosis remains open

- Scope: OR-MIG-0024 revision3 at `ae3d234c71764dd2364f8452fc1974a54a6783f0`; pinned `food-001` driver. No later build substituted.
- Raw actual result: the six-action driver exited1 at its finite150-second timeout. It confirmed meat2142 selected-slot consumption and HP2→5, then sent the cake1891 action at slot3. Server trace records tick99 selected replacement1893, HP5→9 and unchanged11540 fineXP. The durable database subsequently contains HP10. Four later cake/pizza actions were never sent and remain untested.
- Independent follow-up: actual graceful save/shutdown, restart, genuine native authentication, and independent decoding of2061 captured server packets confirm the durable partial-cake inventory, preserved earlier identical slots, coins and displayed XP. This proves partial durable state, not a passing six-action driver verdict.
- Diagnosis: the pinned driver requires the exact intermediate HP9 while waiting for scripts/packet state. An independent unchanged-World test reproduces a valid food5→9 transition followed by passive regeneration to10 without another food or XP award. The initial same-clock regeneration hypothesis was rejected by its test; refined subsequent-clock evidence passes. Both sources/logs are retained in `independent/integration-external-food-regeneration-*`.
- Status: OPEN verification observation blocker. The original production-session client does not log each received HP/inventory update, so the exact missed-observation cause is still inferred, not conclusively demonstrated. No confirmed gameplay defect or FORGE stop request.
- Next: instrument a new owned observer to record each received inventory/HP update and script-busy state, correlate exact server traces and regeneration, and execute the remaining four bites using a fresh disposable run. Do not overwrite the failed campaign or replay its initial fixture against its post-consumption database.

## PG-INFRA-005 — unselected assembly duplicate admission assumption

- Class: resolved independent-test oracle defect; no confirmed production defect.
- Exact target: OR-MIG-0040 at `d63c0258dd6960bffc3ef43736f4140efb3b757a`.
- The initial1,512-case matrix assumed all four captured material slots matched the fixture's nominal rotation. Only the two explicitly clicked slots have that guarantee; the other ingredients may validly be captured from the first matching ordinary duplicate at admission. The initial test failed at its admission assertion before attempting a transaction in that fixture.
- Correction: assert both clicked slots exactly, validate each other captured ingredient, and preserve/assert all uncaptured slots. Late stale tests remove the actual captured material while leaving a matching duplicate present, so the no-fallback requirement remains independently tested.
- Retest: corrected matrix PASS at the same unchanged SHA. Both harness snapshots, source/executable hashes and logs remain in `independent/integration-external-assembly-batch-*`. All three refused returns, live level, membership, invalid positive-XP rate, repeated captured slot and completed-job replay cases remain strict.
- No FORGE edit, stop request or gameplay issue requested.

## Finding format

## User-directed execution deferral

Chemistry, potion and Herblore scenarios are deferred until explicitly resumed by the user. No historical failure or passing subcheck is erased. Ordinary food remains eligible without additional potion setup; Prayer, cooking, packs, assembly, smithing and gem cutting continue. See USER_DEFERRALS.json. Chemistry interrupt-002 had already failed a client count assertion when the deferral arrived; preserve it as an unresolved observer attempt, without rerunning or calling it a production defect. The proposed chemistry crash case was never executed.

## PG-INFRA-007 — corrected Prayer test discovery and read-only preflight setup

- Exact Prayer target: f6b0f6ad54fe7813bb6607827eeb8da0e60137fa. Initial existing integration invocation failed because the published world crate sets autotests=false. Five selected library tests had passed. A fresh owned external crate executed the unchanged, hash-recorded openrune_burial source: all3tests PASS, followed by production server compilation. No production checkout was edited.
- Read-only cache preflight first used64MiB, exceeding this decoder's24-bit group limit; corrected to0xffffff. Next attempt incorrectly required dummy=0, while absent ordinary target dummy metadata is-1; accept the decoder's ordinary nonpositive sentinel, retaining strict IDs/names/type/member/operation/template checks. Both raw failures and source snapshots remain historical verifier evidence. Corrected same-SHA preflight PASS: base fine rewards45/150/720for526/532/536. No gameplay defect attributed.
- Native selected burial plus stale replay and actual restart/relogin PASS with225fineXP/display22; full V1–V4 acceptance is not inferred from scoped subchecks.

## Food negative follow-up

Fresh negative-003 PASS at ae3d234c71764dd2364f8452fc1974a54a6783f0: full28cake selected replacement, stale whole-cake claim with untouched duplicate, actual brew10→13followed by legally delayed Eat preserving13and all XP. Real restart/native relogin preserves partial cake, dose and initial Stats. Negative-001 fixture setup and negative-002 immediate-after-drink timeout are retained verifier errors, not production findings. Additional potion-based scenarios are now user-deferred.

## PG-DEF-001 — native Close leaves chemistry policy dialog live

- Target: OR-MIG-0006, exact `e7ac3608e61893d80b2de6a1a2e512b3ab785505`.
- Severity: P2 ordinary isolated mismatch; not implementation-blocking. Continue unrelated FORGE work.
- Reproduction: worn Options, real Continue, reopen Options, genuine native Action::Close (95 empty), replay earlier real ResumePause115 with Stop choice. Local modal closes; authoritative saved policy changes Continue0 to Stop1. Charges2, inventory and all Stats unchanged; integrity ok, save acknowledged and graceful shutdown0.
- Expected: closed modal rejects stale response and retains `[2,0]`. Actual `[2,1]`.
- Root cause: exact target gameplay decoder falls through to Unhandled for95, leaving runtime chemistry dialog active. Route genuine Close into existing cancellation and reject stale held/worn replies. Reverify at explicitly published correction SHA; preserve historical failure.
- Evidence: `independent/deep-chem-close/close-001/independent-assessment.json`, raw native packet log, before/after SQL snapshots, frozen client/server hashes and launcher source. Raw launcher PASS means probe execution, while independent V2 verdict is FAIL. Earlier negative-002 corroborates; negative-001 was verifier misuse of Continue latch, not production evidence.
- Issue: https://github.com/autismoyah-hub/project-gielinor-rust-server/issues/123. User explicitly approved publication after automatic review initially rejected external disclosure. Current HEAD replay is not claimed.
- Status: OPEN correction; V2 historical FAIL. No FORGE stop requested.

### PG-DEF-001 correction intake while chemistry is deferred

FORGE published candidate `b11260e0f4d1b3eaad0fefc18112c2e8bdf5843e` as OR-MIG-0061 and OR-MIG-0006 revision2. Published decoder/composed tests are historical FORGE evidence. The user's chemistry deferral includes the held/worn policy and charge-state retest; no independent native retest or issue closure is claimed. Nonchemistry Close/quantity cases remain eligible separately. Original `e7ac3608` FAIL and its disposable state are immutable.

### PG-META-001 — inconsistent packet numbers in Close acceptance prose

Ordinary P3, nonblocking metadata correction at queue HEAD `0c8c7f2b389d77f9f04517869c2003573d94b1d1`: OR-MIG-0061 V2 reason says native Close129/staleResume134, while the same contract's normal/negative behavior, exact candidate decoder and original independently captured239.1 reproduction use Close95(empty)/ResumePause115. Correct that acceptance-reason prose to95/115; do not change packet implementation to match the typo. No gameplay defect or FORGE stop inferred. Chemistry retesting remains user-deferred.

## PG-INFRA-008 — glass observer and additional admission scope corrections

- Target remains `5ce50aede9164cdee8e4008257d0298ea77289e7`. Initial independent glass matrix failed to compile because `close_dialogs` returns bool and its mutation match arm expected unit; wrapper corrected. Fresh1344positive/72negative/96late scenarios PASS. Original source/logs retained.
- First native eight-choice campaign and actual restart PASS. Resume115component uses InverseMiddle order, not big-endian; independent decoder corrected against exact target protocol and immutable captured bytes. No successful database was reset or native scenario replayed to fix the assessor.
- `quantities-001` lookup missed selected1text containing colour markup. `quantities-002` then required a matching operation before actual pointer handling. Both ended before Make and preserved inventory/Stats; raw FAIL verdicts, frozen drivers, acknowledged shutdown and databases are retained. Later lookup normalizes markup and binds exact observed logical quantity controls7/8/9/11/12; child14is decorative keyboard-hint text and must not be confused with a quantity button. Actual emitted Resume115count remains the acceptance oracle.
- Additional admission test initially mutated already-attested host `Catalog.notes` for output1919and assumed live refusal. Generic insertion accepts that noted-ID map and commits the exact output/XP; contract ordinary metadata rejection applies at startup. This is preserved as an unsupported trusted-host-mutation observation, not a player-reachable defect or a claim of live catalogue revalidation. The unexecuted analogous input-placeholder mutation was removed too. Corrected112legitimate selected-item/count/actor/policy cases PASS; invalid startup metadata is assigned to the separately authored, still-pending glass-bindings suite.
- Native eight-choice, Close and restart evidence does not finish quantity labels/1/5/10/All/X, captured batch interruption or all V1–V4 obligations. No chemistry-specific Close retest or issue123closure is implied.

## PG-INFRA-009 — native X keyboard sends an unintended product continuation

- Exact server `5ce50aede9164cdee8e4008257d0298ea77289e7`, client `4a6cee5ecad91dd5b0f15fa655ab12cbe4638d5302afee45ad14bfa788d36f17`; owned `deep-glass-remaining/remaining-002`. P2 native-test dependency; no server defect established, no FORGE stop requested.
- Actual read-only transport trace: keyboard/script-generated Resume115at tick154component270:17/count1occurs before intended manual270:18/count3at tick157. Server saves one selected Emptyoil lamp4525and1250fineXP, matching the first frame. The consumed continuation cannot then make three Vials229. Separate digit/Enter frames did not resolve it.
- Native X remains BLOCKED pending correct focused-input dispatch or a pinned native-client correction. Do not suppress outbound packets, directly overwrite local quantity state, substitute newer server code, or declare an X PASS from a headless count API. Exact callback/focus cause remains unproven; original-client reproduction is not claimed.
- Preserve earlier uninstrumented remaining-001wrong-three-oil state separately. Preserve quantities-003raw FAIL-at-Xand passing1/5/10subchecks plus actual saved-state/native recovery. Continue fresh All-only and captured-batch interruption tests immediately.
- OR63exact `143dab04f28c3daed844da0af33d6af5edcd962d` fresh `deep-production-gem-x/x10-001`, client `7adb96fb284d4ed8837550c918c4774ed84df232e60394055b5d4114f3878249`: single-option X3passed in separate immutable five-quantity gem/shell campaigns, but multi-digit X10 fails. Actual transport at294submits270:15/count1while typing10; intended Make10emits no packet and immediate-packet observer assertion fails. SQL saves exactly one selected Sapphire and2500fineXP, matching the actual frame. New `independent-blocker.json` records hashes. Thus passing a noncolliding X3does not complete V2. Full pointer press/release frames and actual custom-input popup/focus need a fresh controlled retest; no outbound suppression, direct quantity mutation or newer server substitution is allowed. No server defect or FORGE stop claimed.
- Controlled `deep-production-gem-x-focus/x10-focus-001`, same server143dab, client `b877fed6986b26f49143e89eac094f644c68e3cd322a3667ae135bf1e9df999f`: separate pointer press/release frames and visible native input prompt162:43`Enter amount (1 - 23):` were confirmed before typing10. Transport still sends270:15/count1at244; SQL again saves one Sapphire/2500fineXP, then the intended product widget is unavailable. Missing prompt/short pointer-frame timing alone is disproven. Investigate keyboard/event routing while the custom amount input is active, including underlying production hotkeys and observer versus ordinary client loop. Keep OR63V2BLOCKED; earlier X3/native built-in/recovery PASS evidence is preserved. Do not change native core functionality to force a verification PASS.

## PG-INFRA-010 — bank deposit controller and native dispatch blocker

- Exact server `7fc1306bb5c01540038ce5ff175b577294ade432`, client `ba67ee03633c6d7e5531f64a1969d4b87d43d3a9463820c18ed912e8ea8e7d53`, `deep-pouch/pouch-001`. P2 verifier defect; no server defect or FORGE stop request.
- Native Open-one/Open-all and stale selected-slot refusal passed. Actual Banker tutor operation opened the bank, and native menu offered Coins Deposit-All. The controller then asserted one immediate outbound packet, but this native menu schedules a client script; zero immediate packets caused the observer panic before polling the script. No deposit frame was actually submitted, so no bank restriction acceptance is inferred.
- Preserve raw FAIL, acknowledged saved opening results and the original database. Correct the observer to capture a readonly copy after successful gameplay transport submission and wait for the real bank script/deposit; require a successful coin deposit and the specific no_bank refusal message. Rerun in a fresh disposable campaign at the same implementation SHA. Original dispatch, scripts and transport are unchanged.
- Fresh `pouch-002`, client `70cea3bb316c6a2def0912e361c343ccd4c38b38af182dbed1727f6f24b61106`: immediate-packet assertion removed and readonly post-send capture installed. Native Deposit-All was offered but no normal interface47deposit was submitted during60seconds; sampled script queues were idle while server clocks advanced. Coins1009remain in inventory, bank empty, all Stats unchanged. The specific restriction was never reached. Thus asynchronous waiting alone does not resolve the blocker; the earlier asynchronous explanation was a hypothesis, not an established cause. Preserve `independent-blocker.json`; exact widget content/listener/mask and client/server ownership need a new bank-only diagnostic. Continue unrelated amethyst/shared-production tests; no FORGE stop or server defect claim.

## PG-INFRA-006 — resolved observer and capacity assumptions

- Retain negative-001: removing an actual Continue packet from outbound left the native Continue latch active, preventing real Cancel. Corrected observer dispatches real Cancel before replay and never strips generated packets. Fresh negative-002 and genuine Close case are separate immutable runs.
- Retain initial chemistry matrix: expected unchanged level5 despite legitimate XP crossing level6 was an oracle defect. Corrected 22000-combination matrix plus12 validation cases passes at unchanged e7ac3608.
- Capacity guard now matches project three-sample CPU70% policy while retaining stricter6GiB available RAM and restricting only owned children to two CPUs. Earlier30% deferrals remain measurements, not server failures.
- Food observer received all HP46 updates, full inventory22 and actual post-apply inventory snapshots; raw partial inventory18 was omitted. Assessment uses actual post-apply confirmation plus server trace and SQL, and independently decodes full22 on real restart. It does not claim raw18 capture. Initial assessor incorrectly expected a post-consumption full22; corrected without rerunning or altering successful campaign state.
- Observer compilation signed-position/unsigned-Walk mismatch corrected with checked conversions; failed source and build logs retained. No production correction needed.

## PG-INFRA-004 follow-up — blocker resolved for further execution

Fresh `deep-food-observer/observer-001` completed six actual native actions, including four formerly unexecuted bites, acknowledged save, real restart and native relogin. Raw received HP sequence2→3 before Eat, then3→6 from meat and6→10 from cake distinguishes regeneration and consumption. XP and unselected duplicate inventory remain conserved. Original failed driver and its missing packet observations remain historical; its exact missed-equality sequence is inferred, not retroactively fabricated. Full native negative/overheal/capacity and crash recovery remain open.

## PG-REF-001 — semiprecious reference conflicts require explicit adjudication

- OR-MIG-0064, exact `0c489b46b552ad012e630810c1218dbaeb437a51`. P2 reference/contract clarification; gameplay defect not established, not implementation-blocking, no FORGE stop. No external issue created. Affects OR64 V4; does not invalidate the separate OR65 limestone/zero-XP facts.
- Independently hash-verified Opal revision15315120 says failure3.8XP, while Crushed gem15273740 creation row says3.7. Current source/implementation follow3.8; the contract should explicitly acknowledge/select its reference priority rather than imply all pinned documentation agrees.
- Jade15280645 explicitly gives a floor-only formula, yielding118/256 at current13. The generic Skilling success rate15267313 rounds with+0.5, yielding implemented119/256; endpoints100/245 agree. Declared rounded behavior passed the controlled-byte matrix, but that does not resolve the Jade-specific documentation conflict.
- Reproduce with `scripts/proving-ground-reference-fallible-gems.py`; source-safe evidence: `independent/reference-fallible-gems/comparison.json`, twelve pinned identities and three independent profile comparisons. FORGE action: document the selected formula/reward adaptation or provide stronger observed reference evidence and an exact correction SHA. Keep V4 PARTIAL pending resolution; continue native, authoritative and unrelated verification. No expectations silently changed and no newer server substituted.

## PG-DEF-002 — precise assembly credits at zero HP or after queued death

- OR-MIG-0067 revision1, exact `e3ced82211ee438fd6a74a110d435e6746295a16`; P2 authoritative gameplay boundary failure, not implementation-blocking. No FORGE stop requested; no external issue created.
- Public World water-orb571/staff1391 held-use consumes selected slots5/19, inserts1395at free slot0 and awards1250fineXP at rate125 even with zeroHP. The same occurs after public `queue_hit` proves `DeathRequested`, followed by visibleHP10 restoration without respawn. Both fresh action and pending completion fail. Retained duplicates stay untouched, so this is not a stale-slot fallback or rounding issue.
- Four test functions fail at their first water/forward case; remaining death profiles/directions are unexecuted. Modal-after-admission cancellation independently passes8cases. Native dead-player reachability and current FORGE HEAD remain unproven; historical normal native/persistence evidence stays separate.
- Reproduce: `python scripts/proving-ground-integration-external.py e3ced82211ee438fd6a74a110d435e6746295a16 --suite battlestaff-lifecycle`. Exact immutable attempt `independent/integration-external-battlestaff-lifecycle-e3ced82211ee438fd6a74a110d435e6746295a16-1792309680874394800/` contains source, binary identity and1PASS/4FAIL log. Structured finding: `independent/battlestaff-life-finding.json`.
- Source diagnosis: assembly held-use admission checks membership/current skill; deferred completion calls the atomic item/reward API without authoritative player life. Require zeroHP/death-queued refusal at both admission and completion, including restored visibleHP without respawn. Review shared consumers, preserve integer/precise/immediate behavior and publish exact correction SHA/revision. Retest historical failure separately; never substitute a newer target silently.
- OR67 V1 remains FAIL and full acceptance is withheld. Continue native restart scopes and unrelated contracts while FORGE implements. The initial resource-deferred attempt and all failures remain preserved.

## PG-INFRA-011 — concurrent native startup deadline, resolved by sequential runs

- OR68 exact `894646a79fa4ba7e3e8b46af6ec000dc6b796514`. Native test dependency; implementation defect not established, nonblocking, no external issue.
- Concurrent `deep-crab-interrupt/interrupt-001/produce` and `deep-crab-carving/native-001/relogin` reached the180second ready deadline at receive clock0/HP0with initialization scripts pending. Both emitted zero gameplay sends and retained whole inventory/Stats; acknowledged graceful shutdown/resource identities passed. Original raw FAIL results and databases stay immutable.
- Identical frozen normal client `c39fdf017af33517e98bd0b826ad726da1373d007ab223f5c9950c9fa8183b1f` alone passes `relogin-002`; independent packet/SQL oracle preserves all24outputs/14625fineXP/Craft18and proves retry-before equals failed-reconnect after-state. Optional immutable reconnect-attempt output label is restricted to a prior failed crab reconnect and the same frozen client; no original evidence overwrite or successful production repeat.
- Fresh `interrupt-002` alone passes native partial All/restart with1broken shell/0XPand22raws. No deadline or client-core change was required. Run owned native clients one at a time under existing resource guard; contention is the supported operational diagnosis, exact root cause unproven. See `independent/crab-native-startup-blocker.json` for identities/history.

## PG-INFRA-009 scope — remaining OR68 native custom input

- OR68 exact `894646a79fa4ba7e3e8b46af6ec000dc6b796514` has V0/V1/V3/V4PASS, including310272rolls/288World fixtures/79cache refusals,39normal native products/restarts, partial zero-XP broken save and owned unsaved-state recovery. Native1/Allboth profiles and5/10shell packets pass.
- Remaining custom-Xnative acceptance is BLOCKED on the unchanged pinned client focus/input dependency. No exact68Xfailure is alleged and no older server result is inherited. Older62/63X10failures retain their exact identities; no functional client correction is currently attested. Preserve full evidence and continue OR69 rather than force quantities or use a newer server. See `independent/crab-custom-input-dependency.json`.

For each confirmed defect, record a stable finding ID, feature ID, full tested SHA, severity, expected/actual states, precise reproduction and scenario, evidence paths, affected dependency chain, implementation-blocking status, GitHub issue, and correction/retest history. Preserve historical failures after a fix. Search existing issues before filing a duplicate.

## PG-DEF-003 - immediate tool crafting ignores authoritative life

OR-MIG-0065 r1 at `4f4aacb2929e8ad775d091527e2d80070100ee69`: eight separate World refusal cases fail across both recipes, directions, zero HP and queued death with visible HP restored. Selected raw is replaced immediately; limestone awards 300 fine XP at rate500. Four alive controls pass. Native reachability/current HEAD remain untested. Apply shared life checks before immediate admission/credit and publish correction SHA; ordinary P2, no FORGE stop. Aggregate acceptance is withdrawn append-only; prior successful evidence remains intact. See [finding](independent/immediate-life-finding.json) and [withdrawal](independent/immediate-acceptance-withdrawal.json).

## PG-REF-002 - qualify Rabbit-foot source routing in OR69

At exact `fb348dda184b36ec2d503bfe0f1d5393cf8ae151`, thirteen wool rows belong to donor AmuletStringing2, while Rabbit-foot belongs to Combining1/COMBINE with no per-row tick override. Target first1/repeat2 is supported by its pinned Wiki2 and independent World checks, but must not be described as inherited donor AmuletStringing2 for every row. Clarify the reference basis and explicit adaptation; no gameplay defect, code change or FORGE stop alleged. V4 remains partial pending the qualified final comparison. Preserve the original39reference artifact and all completed tests. See [source-route addendum](independent/pair-reference-routing-addendum.json).

## PG-DEF-004 - immediate pair wiring ignores authoritative life

OR-MIG-0070 at `c7ddebe844820260252fb37df518f8c271f37243`: four separately named both-direction zeroHP/queued-death refusals fail. Empty orb10980 and wire10981 synchronously become10973 with1300fineXP at125%; four helm life controls pass. Refuse synchronous assembly admission/credit using authoritative life state and preserve the whole pair/Stats. P2 local correction, no FORGE stop or external issue. Native reachability, current HEAD and durability are untested. The separate corrected384case normal matrix passes without replacing this failure. See [finding](independent/crafting-finishes-life-finding.json).
