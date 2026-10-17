# Isolated asset package integration

Set TOWER_ASSET_PACKAGE to a trusted original-art package directory in BOTH
experimental server and client processes. Unset means the existing baseline.
The text registry starts `TOWER_ASSET_PACKAGE 1`, followed by rows of
`model|object DECIMAL_ID SAFE_BASENAME HEX_CRC32`. Files are bounded to16MiB;
path traversal, symlinks, duplicate IDs and incorrect checksums are rejected.
CRC is corruption detection, not a signature or authenticity boundary. SHA256
provenance belongs in the conversion and evidence manifests. The mount directory
must be locally trusted and immutable for a session; package signing and atomic
version negotiation are future production requirements.

The native DiskCache layer adds new model group resolutions and appends object
definition files in memory. Mounting first verifies all IDs against original
reference/config records. It never rewrites disk bytes, raw_group, reference
tables, the master index or login CRCs. Reopened scene loaders retain the same
immutable mounted package. The server independently validates registry data,
rejects original-ID collisions and appends object definitions to its in-memory
Definitions map. tower_map publishes the two experimental arches through normal
location deltas only in this isolated branch when the package is enabled.

The current paired legacy cache carries truncated16-bit disk group revision
suffixes. The latest client validates full revisions. This experiment preserves
that validation and uses its existing read-only JS5 recovery, which validates
network CRCs and appends full revisions in memory. The offline workbench can
explicitly opt in with TOWER_JS5_READTHROUGH_ADDR=127.0.0.1:63225. It still renders
a static authored scene without logging in or requesting gameplay. This content
provider dependency is recorded; a production offline tool should accept a
separately verified full-version source or a bounded read-only source adapter.
No protected cache suffix is rewritten or quietly accepted by relaxed checking.

The package itself is preinstalled out of band. The original JS5 service does
not distribute custom model1000000, change the master index or negotiate its
identity. Clients without the matching package are unsupported for this POC.
Production requires a versioned registry handshake before admitting players,
separate namespace management, signed content manifests and streaming support.

Modular walls/arches and static damaged variants fit the present subset. Multi
part machinery needs authored pivots, collision and placement composition.
Textures need UV-to-native texture mapping and material tests. Animated scenery
needs skeleton/sequence/frame export. NPCs/bosses additionally need definition,
rigging, deformation, animation and protocol contracts; none are claimed here.
