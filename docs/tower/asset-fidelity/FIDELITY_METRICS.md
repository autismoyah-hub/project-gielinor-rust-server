# Fidelity metrics and their limits

Exact B preservation compares saved/reopened native geometry channels, decoded
poses, alpha and all requested native pixels. Missing timestamps or actions fail
the run. Optional unused fallback/texture fields are distinguished from effective
per-face data. Reference camera/lighting and actual native texture handling remain
fixed. The independent tracer uses float32 canonicalization for decoded values.

Independent C diagnostics include foreground IoU, raw bounding displacement,
height-row width profiles, component bounds/centroids, PCA, HSL/area palettes,
height-region color placement and native motion trajectories. Comparisons do not
rescale or register C to improve overlap. Close front/side/rear/quarter views and
a fixed gameplay camera are all retained.

IoU is useful for large proportional errors but ignores internal construction.
Whole-frame RGB MAE is diluted by background and confounds material, shape and
lighting; the reports explicitly label it whole-frame error. It is not a color
fidelity score. Mask PNGs and actual native views remain necessary. PCA axes can
be ambiguous on nearly symmetric volumes. Vertex/face counts cannot certify
style; submitted-surface bounds cannot recover semantics or hidden design intent.

The local loop retains a proposal only when mean five-view IoU improves by more
than 0.002, no individual view falls by more than 0.04, and whole-frame RGB error
does not exceed 1.06 times the parent. These are bounded regression safeguards
for this experiment, not universal OSRS quality thresholds. Every retained result
still has artistic_approval=false. Rejected proposals and failed measurement
versions remain evidence. Curator annotation errors can make a numeric improvement
artistically worse.

Motion errors use explicit annotated component centroids and no time warp. Arc
errors remove initial bind offsets; absolute errors retain them. Foot contact
measurements use native Y-down surface positions, not collision or inferred joints.
The user remains the final artistic judge.
