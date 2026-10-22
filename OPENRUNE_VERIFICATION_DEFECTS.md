# Independent verification findings

Current maintained-main target: `ec00738fbd97d9564c7711859fc509dae3d66a4e`. Original Agent7 findings remain in [the preserved correction backlog](C:/Projects/OSRSRSPS/agent7-verification/docs/openrune/verification/CORRECTION_BACKLOG.md). No historical failure is deleted or silently relabelled fixed.

## PG-DEF-003 — confirmed on maintained main

Eight independent immediate battlestaff/limestone life-refusal scenarios still change inventory at zero HP or after an actual unresolved lethal hit. Applicable limestone XP is also awarded. The living control test passes all four recipe/direction combinations. Expected behavior is refusal before inventory/XP publication. See [exact reproduction and evidence](docs/openrune/verification/independent/integration-external-immediate-life-ec00738fbd97d9564c7711859fc509dae3d66a4e-1792639733372574300/independent-finding.json).

The first current-main fixture did not compile because Recipe gained `skill` and `output_count`. Its unchanged raw failure is preserved and classified as a harness API mismatch. The corrected fixture states Crafting12 and one output explicitly; refusal expectations are unchanged. No production implementation was edited.

## Historical findings requiring distinct current-main checks

PG-DEF-001: Close/chemistry policy epoch; independent chemistry remains user-deferred. PG-DEF-002/004/005: dead-actor assembly/pair/tanning paths. PG-DEF-006: worn hammer Swap operation routing. PG-DEF-007: variable rune-stock normalization. PG-DEF-008: native Toad Eat6 versus server5. Their original affected SHAs, reproductions and native qualifications remain in Agent7 evidence. None is claimed repaired by consolidation without a relevant current-main retest.

Existing harness findings include empty/stale test selection, mistaken synchronous submit refusal, native UI/X-input focus, startup contention, incomplete fixture population and timer overlay oracle errors. All4,729 indexed Agent7 evidence hashes matched during recovery. Native vendor population and Runecrafting boost dependencies remain separate from gameplay defects.

## PG-DEF-003 extension — spade unresolved death

Independent four-case lifecycle tails at both exact173 and maintained main have2PASS/2FAIL. Visible HP restoration after actual DeathRequested allows new Dig or delayed completion. Generic Dig gives no item/XP, but the authoritative lifecycle refusal is absent. [Exact finding and correction](docs/openrune/agent5/spade-life-finding.json). Normal positive/cache/cancellation suites remain valid; complete V1 acceptance is withheld.
