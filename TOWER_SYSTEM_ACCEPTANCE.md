# Tower system acceptance

2026-10-21. Maintained baseline:
`c32edfcb2d652f4b5f7f028000dfe504e5c10705`.
Implementation: `1c2d4d8e0dd4d761ef156588a05f669d788d202e`.
Additional storage acceptance assertions:
`97eb70e0bf64e7a9d9372a6105b881d636fe2249`.

## Current execution

Workspace library regression launched with:

```powershell
cargo test --offline --workspace --lib --target-dir C:/Projects/OSRSRSPS/Tower-Development/work/target-agent7-gameplay
```

The initial baseline library run completed successfully (tool execution exit 0).
The new implementation regression **passed: 1,040 passed, 0 failed, 3 existing
ignored** (cache 26, protocol 21, server 304, storage 16, world 673). Production
code is the implementation SHA above. The storage-only assertion strengthening
was committed during this run, then separately recompiled/retested at `97eb70e`:
**16 passed, 0 failed, 1 existing ignored**. This is not additional unique tests
to add to the workspace total. The first implementation
build failed because an edit changed the encoding of an existing non-ASCII
character literal; corrected to the original bytes before the implementation
commit. Retained failed log: `work/agent7-gameplay-regression-20261021.log`.
Corrected regression log: `work/agent7-gameplay-regression-20261021-02.log`.

Real account-worker/SQLite diagnostic **passed** on a new disposable
directory, using the existing `cinderworks_checkpoint_check` example extended
with synthetic victory prerequisites and completion/pending-reward assertions.
Log: `work/agent7-gameplay-worker-20261021.log`. It uses actual maintained worker,
database, Argon2 authentication and close/reopen; no client or earned gameplay.
The example binary SHA256 is
`450B69D59C8982BE2E28579F5F374417589E6BCFC23D11BAAA59E993C5201671`.
Read-only inspection of its actual final SQLite database found exactly floor 1
completed and `run=1,services=7,flags=7,clears=1`: victory and pending reward, no
paid marker. Duplicate saves/reopen retained its synthetic 10 coins without a
new reward. Source is `1c2d4d8` (the subsequent `97eb70e` changes storage tests
only). Log hashes, binary hash and result totals are in
`docs/gameplay/acceptance-20261021.json`. Local logs and disposable accounts stay
outside the repository; no credentials or account database are published.

No new native client/server session was launched.
No current-source native acceptance is claimed. Tests run by this implementation
owner are not independent third-party verification.

## Inherited native evidence — limited to its original scope

`CONSOLIDATION_REPORT.md` and
`docs/consolidation/restart-replay-validation.json` describe historical first
clear, full-pack refusal, first reward, duplicate refusal, same-binary restart,
portal return, replay admission and unearned replay reward refusal. The restart
receipt identifies base main `2634178df160dd190f31370455bc5ab6ac38b019`, server SHA256
`A9B70BD12E3830B0FF8003313D50266D10F73D8E0654668C733505250F0E7A63` and client SHA256
`0A506755D2845CA50655A419ED114215C9D12106E9CF780F530FBD321AA5D8C7`.
The receipt's base-main field alone does not establish the exact native binary's
complete source provenance. These receipts do not certify the fetched current source. A second earned native
replay victory/payment and simultaneous native peer rendering remain unverified.

## Required future acceptance matrix

| Scenario | Existing coverage to retain | Additional evidence needed |
|---|---|---|
| Fresh onboarding and preparation | `cinderworks_entry`, `cinderworks_services` tests | Exact-source native new character |
| Boss eligibility and outcomes | `cinderworks_encounter_actions` tests | Earned first and replay victories |
| Reward anti-duplication/capacity | `cinderworks_actions` tests, historical native first reward | Cross-floor and boss-only claim attempts |
| XP/equipment/wealth persistence | Account transaction and inherited native restart | New ledger plus unchanged ordinary possessions |
| Death/logout/interrupted save | `cinderworks_recovery`, `cinderworks_checkpoint` tests | Cross-floor interrupted ack and restart |
| V13 upgrade | New V14 backfill/rollback tests and existing schema tests | Exact-source native upgraded account |
| Permanent floor unlocks/re-entry | New conquest/prerequisite/acknowledgement tests | Live later-floor entry and native first/replay acceptance |
| Independent floor state | New synthetic two-floor objective and disk-reopen tests | Live later-floor slice |
| Credit accounting/contracts | Not implemented | Approved policies, ledger conservation, duplicate/stale spend refusal |
| Solo/group isolation | Existing solo court ownership only | Durable group identity and instance destruction/recreation |

Synthetic completion prerequisites in unit tests must stay labelled synthetic;
they cannot stand in for earned native gameplay. Each future native claim must
pin full source SHA, binary hashes, account provenance, scenario and retained
receipts, including failures. Independent review is required after implementation.
