# Cinderworks: three complete map alternatives

**Offline design review · 17 October 2026 · user selection OPEN.** Gameplay remains at baseline `1153b4e7281435e83f60bff5131b4a3f0a8e0023`. None of these proposals is installed or gameplay accepted.

![Three original map diagrams at the same scale](comparison.png)

| Proposal | Spatial idea | Review |
|---|---|---|
| A — The Hearthring Citadel | Central refuge with four gates and a surrounding circulation ring | [Layout and anchors](candidate-a/layout.md) |
| B — The Broken Waterway | Two ruined banks joined by two crossings, with encounters at opposite ends | [Layout and anchors](candidate-b/layout.md) |
| C — The Foundry Procession | Compact bent progression spine with preparation branches and successive thresholds | [Layout and anchors](candidate-c/layout.md) |

**Recommendation: B**, for its memorable water landmark, route choice and opposing destinations. Its crossings must earn their travel cost. A offers the strongest refuge; C offers the clearest forward sequence. This recommendation does not select or combine them automatically.

## Authentic renders and publication limit

The genuine cache-based renders use the existing native client library. Eighteen candidate frames cover two oblique whole-floor angles, a plan view, sanctuary, waterworks and final encounter approach. Two baseline frames support comparison. All have been visually inspected. The local gallery is `work/floor-one-design-review/selected-review/gallery.html` in the Tower workspace; it switches candidates and viewpoints without starting a game server. Raw BMPs, lossless PNGs, camera sidecars and hashes remain alongside it.

Repository licensing does not establish permission to redistribute Jagex imagery. Following the brief's explicit fallback, **authentic cache-rendered images remain local**. The GitHub PNGs are our original geometric diagrams, with no cache imagery, icons, textures or models. They communicate layout, not the renderer's art quality. Expected cache-image names such as `sanctuary.png`, `waterworks.png` and `boss-approach.png` are intentionally local only. See [asset and rights notes](ASSET_NOTES.md).

## Evidence and scope

- [Current layout audit](CURRENT_LAYOUT_AUDIT.md) and [source-backed constraints](CONSTRAINTS.md).
- [Comparative assessment](COMPARISON.md), [reproduction guide](../../../../tools/tower/offline/design/README.md), and [hash/camera manifest](REVIEW_MANIFEST.json).
- Static checks use actual cache clipping, normal movement, real NPC sizes, workstation adjacency, four fishing positions and 1,512 sampled encounter escape cases per map. All three pass. This is feasibility evidence, not native gameplay acceptance or a pacing measurement.
- Proposed anchor coordinates remain isolated JSON and scene files. Production source, character state and existing evidence are preserved. Regular-tree gameplay remains an unresolved earlier binding limitation.

Review the full local renders before choosing a direction. Select A, B or C, or identify specific elements to combine; major live-map work waits for that decision.
