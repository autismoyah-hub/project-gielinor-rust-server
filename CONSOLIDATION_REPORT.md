# Server consolidation report

`main` is the maintained Project Gielinor Rust server authority. The OpenRune
branch was promoted normally at `7421b89d5e458a7022ff7c5afc51ea79b02406fe`.
The accepted code/test checkpoint for this report is
`bce0e0683a3913ac46e046d693e727504e556434`; subsequent report publication adds
documentation. The [commit sequence](docs/consolidation/integrated-commits.tsv)
records the verified increments.

## Runtime and persistence

The exact enclosed Cinderworks floor, sparse resource registry, spatial bindings,
constructed scene, collision and 20 authored actors now run through the maintained
server. Services use shared gathering, cooking, smithing and prayer operations.
Combat, inventory, equipment, banking, ticks, queues and account storage retain
their existing owners. These World actions remain the integration point for
future testing agents.

Tower progression uses additive **V13**. Existing chemistry **V12** and its
checksums remain unchanged. Chemistry state, Tower run state and ordinary account
data coexist. Divergent experimental V12 databases are refused under the documented
import policy; any import needs an explicit reviewed copy rather than automatic
rewriting. Checkpoints gate tools, victory and rewards on the existing worker's
durable acknowledgement. Original-source legacy-claim comparisons preserve the
documented atomic rollback corrections.

The admitted runtime package is
`630bbb1bd738d4a105493f510b7273c51a6cfaca789bb7ae2d95c40d2bdb32d0`;
its original pair is
`8efa117834c574946e00405d4349cdb650e880d93e80659f18a8020d9666f7cb`.
Both contain 95 model and 70 object entries. The pinned floor hash is
`f1ef5627db2e9ef152515647e61e9bdc5c3a97b18e8f03a38cba91c5d6572027`.
The same identities were coordinated with ORACLE. Installation remains out of
band; long-term distribution/version admission remains open.

## Verification

The disposable account used the existing Floor One 5× and starter-stat policy.
Initial minimum Hitpoints, Runecrafting and Herblore are setup rather than earned
service outcomes. The actual native character completed preparation in Forge–Altar–Kitchen order,
defeated the Ash Imp and the 42 HP Warden, used coolant twice and reached the later
encounter phases. A full 28-slot pack blocked payment. After normal banking, the
chest paid 500 coins and one beer. A repeated claim added nothing. The final save
accounted for all 30 caught shrimp: 7 raw, 6 cooked, 11 burnt and 6 normally eaten,
with every tool and the worn dagger retained.

An actual same-binary server restart restored the paid clear. Native portal
departure and return worked. Replay admission reset objectives while retaining
the first-paid marker, clear count, possessions and XP; an unearned claim paid
nothing. The final durable projection was `run=2,services=0,flags=8,clears=1`.
Receipts distinguish successful native cooking from real burns, failed earlier
attempts, prepared fixtures and headless tests.

The shared library regression passed **1014 tests**, with one existing ignored
test. Two existing private Argon2/logout opt-ins were enabled locally; public
default tests retain their ignore guards. The final additional replay test passed
within eight focused action tests: 25 coins plus beer are acknowledgement-gated,
full capacity cannot leak partial stacked coins, and duplicate claims conserve
525 total coins/two beers across two synthetic completed-run prerequisites.
This payment test is headless; a second native earned clear was not run.

Other scoped evidence includes all six preparation orders, actual World walking
over the pinned collision and all 20 footprints, eight functional anchors, 1323 hazard
escape cases, real disk/socket/TCP checks, and original RSProt zone/NPC/queue and
smithing comparisons. Headless two-player ownership, per-observer restoration and
receiver-private/no-replay graphics pass. Simultaneous native peer rendering was
not exercised under the one-renderer protection.

Native gameplay used immutable server
`A9B70BD12E3830B0FF8003313D50266D10F73D8E0654668C733505250F0E7A63`
and client
`0A506755D2845CA50655A419ED114215C9D12106E9CF780F530FBD321AA5D8C7`.
Later mainline changes are diagnostic/tests/documentation. Full composed-scene
shading, shell/cutaway, hazard visual parity,desktop GDI and audio remain client
follow-ups. The gray altar underlay has an authored data/palette basis.

## Source and recovery

Adaptation was reviewed against current handoff
`f9180b23ae1e158a296489a3730a8ae697a71de2`, runtime source
`c78e2a83a39a3cfd7ad9039de4b0b685207dab2d` and historical Tower
`f9e4433f2451235805f85fe379d351d095b7535b`. The source ledger classifies the
positive shared hunks and retains newer OpenRune owners. The asset/tool ledger
records every audited entry; its inventory is not a gameplay completion measure.
Selected static converter/diagnostic tools are on main with eleven passing
synthetic tests. Blender-dependent and historical authoring/research tools remain
preserved for separate ownership/native acceptance rather than becoming a second
server or client engine.

Original histories, all audited source heads, the 11,406-file pre-migration snapshot,
the 26 pre-existing pending paths, private fixtures, pinned resources and four ignored
Blender masters remain preserved. Hash-verified native files and SQLite snapshots
include all failed attempts. Recovery copies are on this host; independent
off-host recovery of private resources remains open. Restricted source/assets
were not added to the public repository. Git commits and pushes use the approved
normal CLI route under the backup identity/Git Credential Manager.

[The retirement review](docs/consolidation/BRANCH_RETIREMENT_REVIEW.md) lists
source identities, accepted destination and preserved resources. Refs have not
been deleted. Owner approval is required for any retirement.

The OpenRune migration continues. Regular-tree gathering remains GR10-gated and
the requested independent chemistry verification campaign remains paused.
Detailed receipts are indexed in [CONSOLIDATION_STATUS.md](CONSOLIDATION_STATUS.md).
