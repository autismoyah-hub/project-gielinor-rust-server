# Tower system acceptance

2026-10-21. Current source under test:
`c32edfcb2d652f4b5f7f028000dfe504e5c10705`. Documentation-only gameplay branch.

## Current execution

Workspace library regression launched with:

```powershell
cargo test --offline --workspace --lib --target-dir C:/Projects/OSRSRSPS/Tower-Development/work/target-agent7-gameplay
```

Result pending completion. No new native client/server session was launched.
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
| V13 upgrade | Existing storage consolidation tests | Additive migration, pending reward, replay, invalid-state rollback |
| Permanent floor unlocks/re-entry | Existing Cinderworks replay only | Floor-indexed eligibility, first conquest versus replay |
| Independent floor state | Not implemented | Minimal clearly synthetic second-floor fixture plus later native slice |
| Credit accounting/contracts | Not implemented | Approved policies, ledger conservation, duplicate/stale spend refusal |
| Solo/group isolation | Existing solo court ownership only | Durable group identity and instance destruction/recreation |

Synthetic completion prerequisites in unit tests must stay labelled synthetic;
they cannot stand in for earned native gameplay. Each future native claim must
pin full source SHA, binary hashes, account provenance, scenario and retained
receipts, including failures. Independent review is required after implementation.
