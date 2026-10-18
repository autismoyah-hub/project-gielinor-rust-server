# Reference selection and provenance

[REFERENCE_CATALOG.json](REFERENCE_CATALOG.json) records inspected selectors, names,
original model IDs, source payload/definition hashes and measured counts.
The executable selection manifest is
`tools/tower/asset-workbench/reference_catalog.json`. Its training/held-out split
is declared before tuning the authoring utilities on held-out assets.

Training includes marble/stone arches and Keldagrim architectural pieces,
furnace/anvil/furniture, cave walls and rock columns, ground detail and clothing,
three animated machines, four humanoids and four non-humanoid creatures,
two swords, two shields, a platebody and a helmet. Bronze and iron swords share
model 2604: they are useful color controls, not independent geometry examples.
There are 32 distinct component-model sets among the 33 records.

Original IDs and variant choice are decoded by the existing native factories.
NPC assembly, recoloring and definition scale are explicit. Equipment can be
inspected as inventory or male-worn geometry; these are not interchangeable.
Object selectors may specify a shape. Missing definitions, models, animations
and textures fail the batch rather than being silently replaced.

Each private package includes geometry, definition text, source hashes, raw model
payloads/interchange, definition transform metadata, native cameras, five native
views, sequence/skeleton/frame data and available native poses. The cache is
read-only and remains the genuine staged revision-239.1 source. Unavailable
semantic landmarks and artist intent are not inferred from IDs or triangle counts.

Held-out selectors are objects 362,366,2097,436; equipment 1323,1171,1163
(worn); and NPCs 106,2834. Their payload overlap with training must be checked
before interpreting generalization. Held-out descriptor-driven fitting is distinct
from blind class prediction and must be reported honestly.
