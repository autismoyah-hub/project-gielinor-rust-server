# FORGE implementation status

<!-- forge-current:start -->
## Current implementation checkpoint

OR-MIG-0045: bear, rat and rabbit cooking, plus cooked-rabbit consumption. Published source [461424d](https://github.com/autismoyah-hub/project-gielinor-rust-server/commit/461424d9c7f1ab00ffdbc264f583088b77d978ef). Affected libraries compiled and focused registration/transaction checks passed: 1 server test, 401 source hashes unchanged. Previous shared-engine regression remains 83 server/443 world, one existing ignored test; it predates this adapter.

The manual subset has 25 fire and 38 range profiles. All three meats use selected-slot transactions, Cooking1/30XP, canonical outputs and cutoff34. Rabbit uses live membership and heals5HP through the existing food transaction. Fire timing uncertainty, deliberate burning, sinew and roasting are explicitly excluded.

There are45 implementation units across49 append-only records:23 historical contracts and22 new gameplay/shared units, all pending independent verification. Discovery remains3,508 records:30 partially contracted eligible files/tables,656 configuration and2,822 unassessed; this is not a completion percentage.

Remaining work includes Anglerfish PvP-area/recent-combat consumption eligibility, further content, quantity/Cook menus, modifiers/special surfaces, cake tin returns, inventory reconciliation and independent V0–V4 acceptance. Source and failed/historical evidence are preserved.
<!-- forge-current:end -->


Resumed2026-10-17 under the implementation/independent-verification handoff mandate. Branch `agent2/openrune-content`; baseline local/remote SHA `8d3e691c667698f9fbbb6f1102a44374832cbb17` matched before work. Historical evidence and unrelated source changes are preserved.

Initial backfill:17 coherent recent Herblore implementation contracts with exact published source SHAs. All are `implemented_pending_verification` / `queued_independent`. Prior FORGE native and persistence results retain their original scope and are not PROVING GROUND acceptance.

Inventory:3508 discovered records;7 source files/tables linked to partial behavior contracts,656 structurally classified configuration records,2845 not yet assessed. No eligible-module completion percentage is claimed. Earlier smithing, pack, cooking, gathering and other implemented batches still need contract backfill; this is explicitly outstanding.

Current throughput:17 contracts backfilled;0 new gameplay units in the initial metadata checkpoint. Source compilation is not repeated for unchanged backfill. Queue format/Git identity and validator negative tests are the bounded checks for this checkpoint. New implementation batches will record their own compile/transaction checks and published SHAs.

No PROVING GROUND defects have been received in this resumed session. Existing broad GitHub issues remain open. Independent findings are triaged using the correction policy in README.md.

Next: publish queue/schema/validator, continue dependency-aware eligible gameplay implementation with bounded checks, append exact-SHA contracts, and continue reconciling older implemented batches and unassessed inventory. Learned super-antifire/barbarian gates, alchemist storage, automatic cleaning timing, acquisition and drinking behavior remain unresolved where documented; those gaps do not block unrelated implementation.

## Implementation checkpoint OR-MIG-0018

Source71289e622cdfac55d5f11e1c857f325929ebff3f published: selected-slot atomic potion consumption is reachable through existing held Drink/Empty -> Action::DrinkSlot -> consume_slot. Bounded3server38world potion-filtered testsPASS; legacy reference API/tests retained. Independent V0-V4 acceptance queued; no new native driver built.

Current throughput:17historical contracts backfilled plus1new shared gameplay implementation;18queue units. Inventory now8partially contracted eligible source files/tables,656configuration records,2844not-yet-assessed. Full source/module reconciliation and earlier-batch backfill remain outstanding. No PROVING GROUND findings received. Continue next eligible implementation without waiting for acceptance.

## Implementation checkpoint OR-MIG-0019

Published source d02633e45576cbea0ba4e14e7d3cb8c164886950 implements all eight Ancient/Forgotten brew drinking doses through existing held DrinkSlot dispatch, using shared overrestore/current-drain primitives and selected-slot atomic transactions. Bounded4server41world potion-filtered testsPASS;397source hashes unchanged. Independent V0-V4 acceptance queued.

Throughput:17historical contracts plus2new gameplay units,19queue units. Inventory now12partially contracted eligible source files/tables,656configuration records,2840not-yet-assessed. No PROVING GROUND findings received. Full earlier-batch backfill, eligible-module reconciliation and remaining potion/gameplay obligations remain outstanding.

## Checkpoints OR-MIG-0020 and historical OR-MIG-0021

Saradomin brew source cf75fa343a0fc86c50f4451a22600cc10a0ee500: four registered doses, capped HP/Defence boosts, current offensive drains, atomic dose exchange and foul-liquid message. Focused4server43world testsPASS;397hashes unchanged. Explicit V4 discrepancy: Wiki minimum0 selected over donor helper default1.

Backfilled historical smithing c8b9aef260db3101dafb2b591ebbdfa4add69f5b as OR-MIG-0021 using its existing compile/test evidence and actual queue/registration paths; no completed build/native work repeated. Scope is queued live requirements and ordinary iron outcome safety, not full smithing parity. All independent levels remain queued.

Throughput:18historical contracts plus3new implementation units;21queue units. Inventory14partially contracted eligible source files/tables,656configuration records,2838not-yet-assessed. Remaining prior-batch backfill and eligible-gameplay reconciliation continue. No PROVING GROUND findings received.

## Checkpoints OR-MIG-0022 and historical OR-MIG-0023

Ordinary Bastion/Battlemage source71fa140f48a1860c90b294344ddb3bdcde2e8781:8registered doses reuse shared boosts and selected-slot atomic dispatch. Focused4server44worldPASS;397source hashes unchanged. Historical17profile pack adapter caefcf424869d2a54d711acd22a279351f098be0 backfilled with actual registration, transaction and repeat boundaries. Preserved native/same-save history is not independent acceptance.

Throughput:19historical contracts plus4new gameplay units;23queue units. Inventory15partially contracted eligible source files/tables,656configuration records,2837not-yet-assessed. No completion percentage or complete eligible-module denominator claimed. Earlier content backfill and remaining gameplay obligations continue; no independent finding delivered to FORGE in this session.

## Checkpoint OR-MIG-0024

Common ordinary food gameplay now uses exact selected-slot transactions and safely permits eating above baseHP without extra healing/no-op update. Source0132d0737f7bcbab7f2387dc3151f238a055922d;66server426worldPASS/1existingignored. First malformed-fixture failure and final-source evidence retained. Legacy reference API/tests and configured multipart profiles preserved. Independent V0-V4 queued.

Throughput:19historical contracts plus5new gameplay/shared safety units;24queue units. Inventory16partially contracted eligible source files/tables,656configuration records,2836not-yet-assessed. Full remaining food stages/special effects, earlier content backfill and eligible-module reconciliation continue. No independent finding delivered to FORGE this session.

## Checkpoint OR-MIG-0025 and OR-MIG-0024revision2

Sourcea560034a59e766d1faf400b61187fec861d13475 registers15additional chocolate-cake/topped-pizza/pie stages and preserves claimedIDs for all ordinary held Eat packets. OR-MIG-0024revision2 appends a link to revision1/0132d0737f7bcbab7f2387dc3151f238a055922d; earlier evidence/history is unchanged. OR-MIG-0025 explicitly follows Wiki healing where donor raw pizza values disagree. Focused3server12worldPASS;397hashes unchanged;16target cache definitionsPASS. All independent levels remain queued.

Throughput:19historical contracts plus6new gameplay/shared units;25units/26append-only records. Inventory17partially contracted eligible source files/tables,656configuration records,2835not-yet-assessed. No complete eligible-module denominator or broad migration completion claim. Earlier content backfill, full source reconciliation and remaining gameplay obligations continue.

## Checkpoint OR-MIG-0026

Source5b47d7fe42703ade7b97e10636c53abb35bd6e0e registers ten missing ordinary food healing profiles through actual startup and existing selected-slot dispatch. Focused4server12worldPASS;397hashes unchanged;10target definitionsPASS. All ten donor rows and Wiki values checked,including Caviar5. Cosmetic Onion/Ugthanki messages are explicit remaining obligations. Independent V0-V4 queued.

Throughput:19historical contracts plus7new gameplay/shared units;26units/27append-only records. Inventory remains17partially contracted eligible source files/tables,656configuration records,2835not-yet-assessed. Full remaining source/module reconciliation, prior-batch backfill and gameplay behavior remain incomplete.

## Historical foundation backfill OR-MIG-0027..0029

Added exact-SHA scoped contracts for four-vessel water filling f1f0032d54405e6509a6250a94368531df78bb18,ordinary dairy harvesting/packed approach231f33710596f8a9b4159f332e0e99a26d182b14,and nine dough choices d8978ede56778fc7821fbb85fa0617d5b8f0c683. Existing tests/native/archive/failure evidence preserved; no completed build/native work repeated. Well cups remain outside selected four-vessel policy; dough lossless capacity refusal/return completeness and milk variant/quest limits explicit. V0-V4 acceptance queued independently.

Throughput:22historical contracts plus7new gameplay/shared units;29units/30append-only records. Inventory20partially contracted eligible source files/tables,656configuration records,2832not-yet-assessed. No full eligible-module denominator or migration-completion claim. Remaining source reconciliation/content implementation and historical backfill continue.

## Checkpoint OR-MIG-0030

Sourceeeb270e37b44388f784f2ac2bd40c34fb8c2df2e registers nine prepared fruit portion foods through shared ordinary binding and actual held Eat dispatch. Focused5server12worldPASS;397hashes unchanged;9target definitionsPASS. Pinned Food/Allfood15318406 explicitly supplies four heal fields omitted from individual pages. Existing ordinary/multipart regressions retained; native/lifecycle acceptance queued independently.

Throughput:22historical contracts plus8new gameplay/shared units;30units/31append-only records. Inventory20partially contracted eligible source files/tables,656configuration records,2832not-yet-assessed. Remaining content/backfill/source reconciliation continues; no complete migration claim.

## Checkpoints OR-MIG-0031 and historical OR-MIG-0032

Manual Admiral source0fe7d767e630ace7d5518be979adf094bb285026 registers three stages with optional shared1/2/2clock timing;70server427worldPASS/1existingignored;397hashes unchanged;7target definitionsPASS. Recipe ABI/player schema/default27timing preserved. Make-X and baking remain excluded. Historical27recipe sourcefe04a83de2b6f7f3edefc92c7f7031290a5908b0 backfilled with actual registration/atomicity/reward/requirement limits and preserved native/failure evidence;no old validation repeated. Independent V0-V4 queued.

Throughput:23historical contracts plus9new gameplay/shared units;32units/33append-only records. Inventory22partially contracted eligible source files/tables,656configuration records,2830not-yet-assessed. No complete eligible-module denominator or migration-completion claim;remaining source reconciliation,content and prior backfill continue.

## Checkpoint OR-MIG-0033 and OR-MIG-0024revision3

Sourceae3d234c71764dd2364f8452fc1974a54a6783f0 registers14bites across7skill-boost pie families and stages shared secondary boosts with food/HP. Effective member-food definitions gate actual Eat using trusted account membership.71server429worldPASS/1existingignored;397hashes unchanged;15target item definitions/7stat symbolsPASS. Shared food contract revision3 links revision2/a560034a59e766d1faf400b61187fec861d13475 without overwriting evidence. Dragonfruit dish return follows Wiki over omitted donor stage. All independent levels queued.

Throughput:23historical contracts plus10new gameplay/shared units;33units/35append-only records. Inventory23partially contracted eligible source files/tables,656configuration records,2829not-yet-assessed. Full remaining food/creation/cooking/skills/source reconciliation and earlier backfill remain incomplete.

## Checkpoint OR-MIG-0034

Source97b25c57d4c6dc0606adf91dd00e41ec43600a27 registers Summerwhole/half andpapaya with shared food/boost/energy preparation and existing run-energy projection.72server430worldPASS/1existingignored;397hashes unchanged;4target definitionsPASS. Exact consumption energy is separated from passive recovery;invalid later effects cannot award energy. Summerdish return follows Wiki. Independent V0-V4 queued.

Throughput:23historical contracts plus11new gameplay/shared units;34units/36append-only records. Inventory24partially contracted eligible source files/tables,656configuration records,2828not-yet-assessed. Remaining source reconciliation,content/creation/cooking/skills and prior backfill continue;no complete migration claim.

## Checkpoint OR-MIG-0035

Source1446167370dece10751c4c332929af880259ee23 registers four live-baseHP food profiles through existing selected food/stat pipeline.73server432worldPASS/1existingignored;397hashes unchanged;5target definitionsPASS. Explicit Strawberrytable/sourceceil6% andWatermelonWiki floor5%+1 decisions preserve conflicting prose/donor rules for V4. Bowlreturn follows donor stage. Independent acceptance queued.

Throughput:23historical contracts plus12new gameplay/shared units;35units/37append-only records. Inventory25partially contracted eligible source files/tables,656configuration records,2827not-yet-assessed. Full eligible-module reconciliation,remaining content/skills/creation/cooking and older backfill remain incomplete.

## Checkpoint OR-MIG-0036

Publishedsource 8065a96c23c16c7b7cfd90e402acd015e356d706 registers3manual ordinaryfire recipes through actual reached-location/weakqueue route. Final74server437worldPASS/1existingignored;399hashesunchanged. Active/completedtutorialboundary covered;testfixturecompilefailure002preserved. Source/Wiki/cacheprobabilityandtiming differencesexplicit;independentV0-V4pending.

Throughput23historical+13newgameplay/sharedunits;36units/38records. Inventory{"configuration": 656, "eligible_gameplay_behavior": 28, "not_yet_assessed": 2824};3508discoveryrecords arenotmodulecompletiondenominator. Continueeligiblecontent;ordinaryfishnext.

## Checkpoint OR-MIG-0037

Sourcefbf10373608cb22b03c89ce9f6bfcb12491b994e registers9manualordinaryfirefish onexistingcookingadapter. SpecificWiki probability/timing/firestopprofilesincludingPike54 areexplicit;sharedburntIDsattested. Cookingfilter1server6worldPASS plus explicitnewbinding1server0worldPASS;399identicalsourcehashes;24targetdefsPASS. IndependentV0-V4queued.

Throughput23historical+14newunits;37units/39append-onlyrecords. Inventory28eligiblepartiallycontracted/656configuration/2824unassessed;3508discoveryrecords arenotmoduledenominator. RemainingCookingquantity/surfaces/modifiers,content/skillsandfullreconciliation/backfillcontinue.

## Checkpoint OR-MIG-0038

Sourcecb8975bc8fd6524e1e322f8da28a1e30244d81ee registers6eitherorderStewassemblypairs,current25/2clocks/0XP;manualfirecooking117success/burn0;StewEat11HP/retainedbowl. Existingenginesonly. Final2server0worldPASS/399hashesunchanged;10ordinarydefsattested. Firemanual1timingexplicitlyinferredfromrangequalifiedWiki recipe/fireinstructions,notacceptednativeparity. IndependentV0-V4queued.

Throughput23historical+15newunits;38units/40append-onlyrecords. Inventory29eligiblepartiallycontracted/656configuration/2823unassessed;3508discoveryrecords arenotmodulecompletiondenominator. Continueeligiblecontent/reconciliation/backfill.

## Checkpoint OR-MIG-0039

Sourcef79a54671dfd48534d1a20e9bb1dc2aac2d4b85c registers2chocolatetoppings(bar/dust),current50/30baseXP/2clocks usingexistingAssembly. Focused1server0worldPASS/399hashesunchanged;4targetdefsPASS. Wiki gates/reward/dust intentionallyextenddonor. IndependentV0-V4queued;4materialbatter/bakingremainunimplemented.

Throughput23historical+16newunits;39units/41append-onlyrecords. Inventory30eligiblepartiallycontracted/656configuration/2822unassessed;3508discoveryisnotmodulecompletiondenominator. Continuefoundationalmulti-inputassemblyforcakebatter andremainingeligiblecontent.

## Checkpoint OR-MIG-0040

Sourced63c0258dd6960bffc3ef43736f4140efb3b757a extends sharedassemblytransactions/reward withcapturedmulti-inputjobsand3actualcake-tinpairbindings. Fouringredients->uncookedcake/bucket/pot,current40/2clocks/0XP. Openingmodalcancelssharedpair/batchassembly;closedmodaltestpreventsdelayedrelease. Final79server439worldPASS/1existingignored;399hashesunchanged;7targetdefsPASS. IndependentV0-V4queued;baking/range/tinreturn/quantityremainseparate.

Throughput23historical+17newunits;40units/42append-onlyrecords. Inventory30eligiblepartiallycontracted/656configuration/2822unassessed;3508discoveryisnotmodulecompletiondenominator. Continueeligiblecontent;standardrangescandidateinspectionpreserved,butnotregistered.

## Checkpoint OR-MIG-0041

Source902e55c7dd619ab5af4dd5f8bba0284ed4f75dca addsordinaryrange26181/896throughsameCookingqueue,13priorprofiles+range-onlyBread40XP/1clock. Swordfishrangestop80/30/310 differsfire86/18/292. Final80server440worldPASS/1existingignored;399hashesunchanged;actualbreadroute/generation/refusaltested. LegacyroundingBread37chance1/cutoff38explicitV4precisionobligation;donor34notadopted. IndependentV0-V4queued.

Throughput23historical+18newunits;41units/43append-onlyrecords. Inventory30eligiblepartiallycontracted/656configuration/2822unassessed;3508discoveryisnotmodulecompletiondenominator. Continueordinaryrangepie/pizzabaking;Cook-op1/quantities/modifiers/cake-tinreturnsremainseparate.

## Checkpoint OR-MIG-0042

Source052d24c0b0d85989785f94ffe0910a6b39af1412 registers4range-onlymanualbakingprofiles:Redberry/Meat/Applepie/Plainpizza. Wikiindividuallevels/XP/stops/curves/1clock intentionallyreplaceuniformdonorpolicy. Focused1server0worldPASS/399hashesunchanged;10freshordinaryF2PdefsPASS. Sharedcooking0041regressionsretained. IndependentV0-V4queued;EmptyDish/quantity/modifiers/native/lifecycle remainseparate.

Throughput23historical+19newunits;42units/44append-onlyrecords. Inventory30eligiblepartiallycontracted/656configuration/2822unassessed;3508discoveryisnotmodulecompletiondenominator. Current13fire/18rangeprofilesaremanualsubset,notcompleteCooking. Continueeligiblecontentandremainingobligations.

## Checkpoint OR-MIG-0043

Source8805474ede2cbbb80e9167c300b3bbbefe845c83 registers8memberrangepieswithlivecookingadmission/revocationgate,recomputedrawmembershipmetadata andOptioncutoffs. Dragonfruit220XP;Dragonfruit/Wild/SummerNonepreservesburnat99. Final82server441worldPASS/1existingignored;399hashesunchanged;17freshordinarydefsPASS. IndependentV0-V4queued;modifiers/quantity/EmptyDish/native/lifecycle remainopen.

Throughput23historical+20newunits;43units/45append-onlyrecords. Inventory30eligiblepartiallycontracted/656configuration/2822unassessed;3508discoveryisnotmodulecompletiondenominator. Current13fire/26rangeprofilesaremanualsubset. Continueeligiblecontentwithnewmember/Nonecutofffoundation.


## Corrected dependency revisions

OR-MIG-0037r2 and OR-MIG-0041r2 use source 864d439132b33395773116d57347f9caefdfcf1c. Five burnt-fish identities are corrected using exact Wiki variant pages. Prior tests passed against their own recipe metadata and did not prove semantic identity. Focused correction002passed1server/0world with399unchanged source hashes, including exact IDs through both configuration paths. No historical source, queue record, character file or private evidence was rewritten. Independent retesting remains queued. Stable unit count stays43; append-only record count becomes47.


## Checkpoint OR-MIG-0044

Published source 30509633a6d1e0b96eb9530275040b46e43fabe4 adds nine member fish through a modular adapter and an optional base fine-XP override into the existing Stats transaction. The affected suite passed83server/443world,1existing ignored,400unchanged source hashes. Exact variants, method rules, five99burns and6authoritative fractional-rate cases are covered; independent acceptance remains queued. Current44units/48records; source inventory30eligible partial/656configuration/2822unassessed. Continue special consumption and remaining content; no complete Cooking parity claim.


## Implementation checkpoint OR-MIG-0045

Published source 461424d9c7f1ab00ffdbc264f583088b77d978ef. Canonical meat cooking and ordinary cooked-rabbit consumption are registered through existing startup dispatch. Bounded compile/transaction check PASS1server0world/401same hashes. Queue45units/49records, all independent acceptance pending. Continue with other eligible content; Anglerfish requires shared PvP eligibility before faithful special consumption.
