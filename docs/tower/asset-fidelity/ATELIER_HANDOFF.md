# ATELIER handoff

This is a developing workbench, not production certification. The earlier asset
and animation pipeline remains preserved. Keep the staged cache read-only and
all reference derivatives outside the source repository. Do not modify authoritative
parity/migration branches or run against saved characters.

## Setup and commands

Use sibling `rust-server` and `rust-client-reference` checkouts. Pin the client to
23a94f75707f0503a9efe610fe99bfce5089b575. Build with the existing offline-render
Cargo manifest and `--release --bin asset_workbench`. Dependencies are the existing
vendored native library. Python needs Pillow; headless authoring is tested with
Blender 5.2.2 LTS and its slotted-action API. Supply all machine paths explicitly.

```text
python tools/tower/asset-workbench/bench.py --native NATIVE_EXE --cache CACHE --out PRIVATE_OUTPUT inspect
python tools/tower/asset-workbench/bench.py --native NATIVE_EXE --cache CACHE --out PRIVATE_OUTPUT --blender BLENDER_EXE roundtrip
python tools/tower/asset-workbench/analyze.py library PRIVATE_REFERENCES PRIVATE_ANALYSIS
python tools/tower/asset-workbench/verify_motion.py PRIVATE_TRAINING_DIRECTORY
python tools/tower/asset-workbench/motion.py PRIVATE_REFERENCES PRIVATE_TRACE_REPORT.json
python tools/tower/asset-workbench/test_workbench.py
```

Native selectors are `object:ID[:SHAPE]`, `npc:ID`, `item:ID`, `item:ID:worn`.
`inspect` accepts a batch. `animate SELECTOR SEQUENCE` inspects a specific action;
`discover objects` lists genuinely animated definitions. `render`, `package` and
`play` are standalone native diagnostic modes. They do not start a game service.

## Boundaries

The public branch contains general source and aggregate metadata only. Original
geometry, animation payloads, original imagery, Blender derivatives and direct
reconstruction recipes/output remain local. Complex texture reconstruction,
weighted cached animation authoring, live NPC overlay registration, audio/event
parity and production map integration are not established by these tests.

Use source manifests, strict Blender exits, matching-timestamp coverage and native
comparisons. Record exact source commits and binary hashes with every evidence
set. Technical B success must remain separate from independent C assessment.
