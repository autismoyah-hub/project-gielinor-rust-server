# Reference catalog and selection

The source manifest `tools/tower/asset-workbench/reference_catalog.json` declares
33 base training variants (19 objects, eight NPCs, six equipment variants) and
nine held-out variants. `equipment_mode_catalog.json` adds two explicitly separate
worn-mode training inspections. Thus technical B coverage is 35 training variants
plus nine held-out variants, not 44 independently authored successes.

Native definition IDs, model IDs, source-payload hashes, definition hashes and
numeric geometry records are in the safe catalog/data files. Raw decoded meshes,
textures, source sequence payloads, .blend derivatives and native reference images
stay private. Selectors distinguish inventory and worn assembly; those coordinate
frames are not interchangeable. The source identity is the paired immutable local
revision-239.1 cache and symbols, not an inferred asset name.

The library covers architecture, arches/walls/pillars, cave/rock/soil details,
furniture, industrial machinery, humanoids, chicken/cow/rat/spider, weapons,
shields, worn armor and animated waterwheel/furnace/pump objects. The 33-asset
base includes all model/definition/skin/action measurements available from decoding.
Object 1868 also exercises simple native textured geometry.

The held-out selectors were declared before authoring tuning: objects 362, 366,
2097 and 436; items 1323, 1171 and 1163:worn; NPCs 106 and 2834. All nine have
zero source-model payload overlap with both the base training cohort and the
supplementary worn equipment. This does not establish semantic independence:
barrels/crates remain familiar archetypes. After evaluation, their failures are
development information and cannot be relabeled as fresh held-out results.

Technical classic actions include idle/walk for eight base NPCs, three animated
objects, goblin attack 6185/hit 6183/death 6182, and held-out wolf/bat idle/walk.
See the source-backed IDs and counts in WORKBENCH_RESULTS.json. Native shared
scenes and independent C tests keep A/B/C identities explicit.
