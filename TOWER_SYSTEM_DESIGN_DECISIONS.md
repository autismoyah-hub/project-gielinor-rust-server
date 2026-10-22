# Tower system design decisions

2026-10-21. Baseline `c32edfcb2d652f4b5f7f028000dfe504e5c10705`.

## Binding constraints from the assignment

- The Tower is the game; character progress is persistent. Preserve the current
  Cinderworks loop and additive V13; future migrations must also be additive.
- Use shared World actions, combat, inventory, queues and the existing account
  transaction/save worker. Chemistry/Herblore independent verification stays paused.
- Keep permanent character development, durable ascent/floor records and transient
  encounters separate. Solo and group progress must not accidentally share credit.
- Do not choose unresolved economy, group or skill-gate rules. Coordinate shared
  engine changes with FORGE and client requirements with ORACLE. Do not change
  original-client parity code independently.

## Unresolved design dependencies

| ID | Decision needed | Implementation boundary while unresolved |
|---|---|---|
| D01 | Complete charter V0.1–V1.8 and subsequent approved decisions | Owner explicitly authorized confirmed progression/objective/persistence foundations; broader policies remain open |
| D02 | Later-floor completion versus reward-claim admission requirements | Do not equate paid reward with conquest without approval |
| D03 | Boss Credit earning, scope, spend, failure and payout policies | No live credit minting, spending or assumed rates |
| D04 | Boss catalogue unlock and boss-only replay eligibility | Keep existing full-floor replay behavior |
| D05 | Assignment/challenge catalogue and repeat/expiry rules | No arbitrary contract generation or mandatory objectives |
| D06 | Group identity, membership replacement, failure, loot and replay | Document ownership contract; no roster/economy schema guesses |
| D07 | Approved second-floor scaffolding and client presentation | No finished Floor Two world or parity modifications |

## Confirmed foundation increment

Owner clarification authorizes permanent character development, permanent floor
unlocks, reusable objectives and safe persistence without waiting for the earlier
charter. V1.8 is an addendum, not a replacement. Its exact contents have not yet
been supplied to this checkout; do not invent its skilling details.

Implemented at `1c2d4d8e0dd4d761ef156588a05f669d788d202e`: solo floor identity,
permanent conquest set, content-owned prerequisites and required/optional objective
contracts. A conquered floor stays conquered during replay and before claiming
its reward. No later live floor admission policy is enabled. `Conquest::Replay`
only classifies an already-conquered floor; it is **not** a reward authorization,
attempt counter or Boss Credit grant. Objective proofs are trusted server content
inputs, not accepted directly from a client packet.

V14 adds one monotonic completion row per character/floor, preserving V13 and
backfilling only Floor One where `clears > 0`. Saving merges rows inside the
existing account transaction. Failed transactions cannot expose a new completion;
the existing World acknowledgement barrier publishes the staged ledger on success.
Known Floor One completion is reconciled at load/save as a compatibility boundary.
No dates, repeated-clear counters, group IDs or missing accomplishments are invented.

## Further proposed technical contracts — not yet implemented

Each configured floor should supply stable identity, prerequisites, objective
identities, completion contract and explicit replay/reward policy references.
Unknown floor IDs or missing policy must deny admission rather than guess.
Stable identifiers must survive configuration changes.

Durable completion is independent of transient guardian state and pending rewards.
A new reward claim must identify the owner, floor and attempt, stage the complete
grant, save it atomically with its consumption record, then publish only after
the existing worker acknowledges it. Retries must not mint another reward.

Any legacy migration must preserve the exact V13 fields and pending rewards.
Positive `clears` is evidence of an earned historical victory even during a fresh
replay; current objective bits are not a historical accomplishment ledger.
Do not invent missing dates, achievements, credits or paid rewards during backfill.

A future durable group ascent must have a stable identity separate from temporary
instance identity. Destroying an instance must not delete the ascent. Character
solo records and group-owned records require explicit ownership keys. These are
proposed boundaries, not a decision on membership or eligibility.

## Coordination

FORGE: review storage/World checkpoint extraction and transaction boundaries.
ORACLE: confirm presentation requirements only after floor contracts are agreed.
Independent reviewer: replay, interrupted-save, migration and group isolation
acceptance on exact published implementation SHAs. No coordination message has
been sent: these agent identities were not found among accessible active chats.

## Floor One systems increment — 2026-10-21

Owner requested continued implementation of applicable Floor One/lobby/later-floor
systems. Maintained main was synchronized through
`074eb3adc8f9989b619eec49911c179eb4eb25c1`; this upstream
increment changed verification metadata, not gameplay. Production implementation:
`c606b56b86eed52594734724940abce4b7bf0740`.

- The existing five objectives now have one content definition and dependency
  graph. Workshop/kitchen/altar remain independent; Imp requires all three and
  Warden requires the Imp. Run mutations, entry eligibility and catalogue queries
  use these shared contracts. Optional objectives cannot become dependencies of
  required objectives. Unknown floors, duplicate content IDs and cycles fail
  catalogue validation. No new skill gate or balance rule was added.
- Floor identity 1 is Cinderworks. Its permanent objective bits are workshop 1,
  kitchen 2, altar 4, Imp 8 and Warden 16. Preserve these meanings; do not recycle
  retired bits. Boss catalogue identities 1 and 2 are stable Tower identities,
  independent of cache NPC IDs.
- Additive V15 stores historical objective masks as exactly eight little-endian
  bytes. This supports all 64 bits without SQLite signed-integer loss. Observed
  accomplishments merge by union inside the existing account transaction; stale
  snapshots cannot remove them. Current attempts remain V13-owned, independently
  of the permanent record. No second active-run engine was introduced.
- Migration batches validated V13 records. An incomplete run proves only its
  committed flags; an existing clear proves the five required objectives. Invalid
  legacy lifecycle state aborts migration and its version marker together.
  Unknown historical masks remain preserved even if content is later removed.
- Permanent objective marks are accomplishments, with no new item, XP, currency
  or achievement bonus. The existing task overlay shows their count and a
  read-only boss catalogue both inside and outside the floor. Catalogue discovery
  does not authorize boss-only replay or mint Boss Credits.
- Reward bundles and missing-tool recovery now stage through a reusable helper
  backed by the normal inventory engine. Floor One keeps its established 500/25
  coin plus beer rewards, eligibility flags and acknowledgement barrier. Recovery
  checks carried, worn and banked items, and deduplicates configured tool IDs;
  it does not fabricate combat equipment or resources.

Boss Credit policies, boss-only replay, assignments, group economics, new equipment
tiers and authored later-floor content remain open. These changes are on the
gameplay branch for FORGE review; maintained main and client sources are untouched.
