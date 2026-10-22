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
| D01 | Complete charter V1.8 and all subsequent approved decisions | No new progression rules or migration until reconciliation |
| D02 | Later-floor completion versus reward-claim admission requirements | Do not equate paid reward with conquest without approval |
| D03 | Boss Credit earning, scope, spend, failure and payout policies | No live credit minting, spending or assumed rates |
| D04 | Boss catalogue unlock and boss-only replay eligibility | Keep existing full-floor replay behavior |
| D05 | Assignment/challenge catalogue and repeat/expiry rules | No arbitrary contract generation or mandatory objectives |
| D06 | Group identity, membership replacement, failure, loot and replay | Document ownership contract; no roster/economy schema guesses |
| D07 | Approved second-floor scaffolding and client presentation | No finished Floor Two world or parity modifications |

## Proposed technical contracts — not yet implemented or approved

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
