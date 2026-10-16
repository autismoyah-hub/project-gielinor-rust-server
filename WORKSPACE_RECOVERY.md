# Workspace recovery

Status 2026-10-15: migration blocked by active .git write permissions. Do not begin the OpenRune port.

## Preserved state

Active directory: C:\Projects\OSRSRSPS\rust-server.
External recovery root: C:\Projects\OSRSRSPS\git-staging\agent2-migration-20261015.

pre-migration/ contains 11,406 individually SHA256-verified files totaling 181,759,005 bytes. pre-migration-manifest.json records paths, sizes and hashes. The snapshot includes implementation and local Kotlin reference source. It excludes top-level work/, integration/, target/, .git and Python __pycache__. Those resources remain in place; this is NOT a complete disaster-recovery backup of ignored resources. Do not publish this snapshot or overwrite Agent 3 backups.

resource-size-inventory.json records the original directory inventory. current-source-manifest.json identifies 731 source/build files with SHA256 8a9dfc465e36851a6e30b3c25494e107d5d52cf2dfbff7308fd71ee7c22fa4dc. source-reconciliation.json records original-file preservation and differences from public history.

## Incomplete Git initialization

The active directory initially had no Git repository. A no-checkout clone's metadata was copied into .git after the verified snapshot. HEAD remains main at b811af5de9e10d73989070ec318601c05ddbe9f5, with the expected public origin. Ref creation and index initialization then failed with Windows permission denied, even after explicit write grants for the root and .git. ACL inspection showed sandbox deny entries.

The active index is absent/uninitialized. Status showing staged deletion of remote files is a metadata artifact, not source deletion. Do not commit that status. The proposed agent2/development branch was not created. No active-workspace ahead/behind or clean-state claim is valid yet.

Have the app/environment establish writable Git metadata for the authoritative server workspace. Do not remove sandbox ACLs as a workaround. After repair, verify ownership and permissions, then run git read-tree HEAD to populate the index WITHOUT overwriting working files. Inspect the complete actual diff and ignored paths, and create a development branch. Never use reset --hard, clean, force checkout or force push. A per-command safe.directory for this exact directory may be necessary because the sandbox user differs from its owner.

Sixteen public packaging/documentation paths were absent locally, including .gitattributes, provenance/notices, build/release/configuration documents and licenses/. They remain available in the verified remote checkout. Restore only reviewed absent files; do not replace existing local source or LICENSE indiscriminately.

## Independent recovery test

Branch: verify/agent2-api-20261015.
Local fetched, remote and independent recovery HEAD all equal:
d62a03d4e7e041063c6e91672c9c340994821edd.

verification/ and recovery/ are isolated checkouts. The marker's SHA256 is 61d467c956c16f5d4dd25af647cefa288052e0eb46064e1daacb8b0561b4acd5. push-recovery-result.json records the successful GitHub API publication and Git retrieval. Main was unchanged.

The earlier local verify/agent2-workspace-20261015 branch retains failed CLI publication candidates; it was not pushed. Do not confuse those SHAs with the verified API commit. Keep these records until active-workspace repair and a subsequent ordinary Git push are verified.
