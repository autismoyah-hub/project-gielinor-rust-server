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

No new native client/server session was launched for that first increment.
No native acceptance for that source was claimed. Tests run by this implementation
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

## Continued Floor One systems increment

Production source: `c606b56b86eed52594734724940abce4b7bf0740`, incorporating
maintained main `074eb3adc8f9989b619eec49911c179eb4eb25c1`.

- `cargo check --offline --workspace --all-targets`: PASS.
- `cargo test --offline --workspace --lib`: **1,047 passed, 0 failed, 3 existing
  ignored**. Per crate: cache 26, protocol 21, server 304, storage 19, world 677.
- New cases cover objective/floor dependency cycles and optional gates, discovered
  bosses without replay admission, multi-item reward capacity/overflow, exact
  recovery across three containers, V15 backfill of incomplete/completed/replay
  state, invalid-V13 migration rollback, full 64-bit history masks, stale-history
  merge, character/floor isolation and history rollback with late inventory failure.
- Existing all-six-order real shared service tests now also assert permanent
  milestones. Existing acknowledgement tests assert the full historical mask is
  staged but not visible before victory-save acknowledgement.
- Library log: `work/agent7-gameplay-systems-regression-20261021.log`.
  All-target log: `work/agent7-gameplay-systems-check-20261021.log`.
  Native server build log: `work/agent7-gameplay-systems-native-build-20261021.log`.

Actual earned-kitchen/persistence native acceptance **passed** with server source
`c606b56b86eed52594734724940abce4b7bf0740`, binary SHA256
`873CDF5BBBC72F48905A21CBC9A30E56E60B9305842876775694A2335B0464A1`.
It used the unchanged preserved client
source `0ed67a5670272c17191a3f9f691db66c44750f4c`, binary SHA256
`7857F7E23E7BE31A4193427472ED086C690128E52B284F80597A6E091273BB42`.
No client source or parity behavior was changed. Driver source is tracked at
`scripts/tower_gameplay_native_check.py`; private accounts/passwords/resources,
raw actions and frames remain outside the public repository.

Earned attempts `-07` and `-08` caught nine real shrimp, cooked seven and burnt two
through native menus/packets. The first burn correctly earned no milestone. The
same character was resumed normally, with no account/database state edits. Its
final native Fishing XP is 450 and Cooking XP is 1,050; login starter stats are
not classified as earned XP. All six recovery tools remained one each.

Final continuation `work/agent7-floor-native-20261021-11` passed two full native
sessions, both client/server clean exits, normal logout, server restart, portal
departure/return and exact inventory/equipment/XP/journal comparisons. The empty
equipment state is preserved; equipped-item persistence remains headless coverage.
Final database: `run=1,services=2,flags=0,clears=0`, objective history `(floor=1,mask=2)`,
no permanent conquests and no reward claim. Final driver SHA:
`9aa63ea424eac8ff025081097225e1fc42fddbb3`. The driver changes camera only after
explicitly unsubmitted visible-menu rejections; every accepted interaction still
uses ordinary native picking and packets.

Machine-readable results and retained failure hashes:
`docs/gameplay/acceptance-floor-one-systems-20261021.json`. Exact measured binaries
were copied without overwriting to the private final attempt's `binaries/` folder.
No full native boss clear/replay payment, native Floor Two, group rendering or
broad visual/audio parity acceptance is claimed for this new source. Independent
review remains outstanding.

Preserved native attempts (not acceptance):

1. `work/agent7-floor-native-20261021`: driver waited for an observation before
   publishing its first observation request. No gameplay acceptance.
2. Suffix `-02`: driver raced the separately delivered experience-selection
   registration dialog. No progression credited.
3. Suffix `-03`: obsolete experimental interaction coordinates were rejected by
   the visible native menu. No unearned fallback or coordinate bypass.
4. Suffix `-04`: real run admission and missing-tool recovery passed; repeated
   plaque/chest actions conserved state, but fresh server reply identity was not
   asserted. The driver then tried to close a nonexistent dialog. This is bounded
   partial evidence, not completed scenario acceptance.
5. Suffix `-05`: ordinary movement and onboarding passed; an obsolete experimental
   fishing stance was too far from the maintained waterborne target and the
   visible menu rejected the action. No fish, XP or milestone credit.
6. Suffix `-06`: onboarding passed; the driver looked for a fishing NPC before
   approaching its retained-view range. No fish, XP or milestone credit.

The driver fixes change no game code or game rates. Suffix `-05` uses the
maintained floor-checker's exact collision edges to navigate short visible native
walking segments. Further retained attempts:

7. Suffix `-07`: real fresh onboarding, fresh duplicate/refusal replies, eight
   normally caught shrimp and one real burn. The driver still assumed the old
   experimental batch-cooking flow and timed out after one selected input. The
   incomplete earned character was saved normally, not reset or seeded.
8. Suffix `-08`: continued that character; caught one more shrimp and explicitly
   cooked each selected raw item. Seven successes, two total burns and the kitchen
   milestone were earned; portal departure preserved the record. The unchanged
   client then exited nonzero because its ordinary `work/live-native.bmp` shutdown
   diagnostic directory was absent. Server save acknowledgement and drain passed.
9. Suffix `-09`: correct earned state restored on a cold server/client start. The
   outside portal was occluded from the initial camera view; normal cleanup passed.
10. Suffix `-10`: normal camera rotation revealed the portal, and return preserved
    the earned state. The inside portal was then occluded from that changed view.
11. Suffix `-11`: driver supplies the expected diagnostic directory and resolves
    both portal directions with ordinary visible camera views. Full clean native
    persistence/return scenario **PASS**, using the same earned character.

All attempts, including their failures and raw receipts, remain preserved. No
test database or character state was edited to produce native progression. The
camera and driver corrections do not change gameplay, client parity or rates.
