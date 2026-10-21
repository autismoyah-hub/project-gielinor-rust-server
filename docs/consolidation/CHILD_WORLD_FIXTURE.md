# C-008 child-world test fixture

This optional fixture exercises a genuine revision239.1 child view with stock
terrain and an independently synchronized NPC. It is a wire/rendering test, not
a gameplay instance service. Build with `--features integration-fixtures`, then
explicitly pass `--child-world-fixture c008-v1`, pinned `--cinderworks-content`,
`--ticks` between1 and1200 and an owned `--stop-file`. The normal build rejects
the fixture option. A feature build without the option keeps root-only behavior.
Existing loopback-only binding and resource/session guards still apply.

Use a disposable snapshot of the previously verified Floor One save. No account
coordinates, items, stats, Tower run or namespace reservations are edited. The
fixture's NPC is presentation-only in the child stream; it is not inserted into
the gameplay World and cannot award drops or XP. Do not send interaction or
movement input during this test. Root build-area changes are refused.

## Exact packet and spatial contract

- Root WORLDENTITY_INFO_V7 admits ID1,16×16tiles, configuration-1, owner0,
  plane0, yaw512. Its fixed fine-coordinate anchor is
  `[10423*128+64,0,110*128+64]` in root-world coordinates.
- For root base`10368,56`, admission payload is
  `00000100de80ffffa21bc01b400200`. Following cycles retain it with`010100`.
- SET_ACTIVE_WORLD_V2 selects child1 with`000100`.
- REBUILD_WORLDENTITY_V4 has base`0,0` and four level0 chunks sourced from
  zones`400..401,400..401` in stock map square12850. Upper levels are empty.
  This packet uses a26-bit chunk stream and no XTEA tail.
- The child has no descendants. NPC update origin is`8,8`. NPC_INFO_SMALL_V5
  admits index30000, using the stock `chicken` definition resolved from symbols,
  at child tile`10,8`, facing6, stationary. Later cycles retain the actor.
- Every cycle restores the root active context and NPC update origin before
  continuing root zone updates and the ordinary tick delimiter.
- The actor's size1 centre is child fine`1344,1088`. Around child centre`1024,1024`,
  yaw512 maps that point to anchor plus`64,-320`, approximately root tile
  `10424,108`. Definition offsets and vertical terrain sampling are separate
  from this horizontal check. Floating-point comparisons need an appropriate tolerance.

`tools/child_world_fixture/ChildWorldFixtureOracle.java` calls the original local RSProt239
encoders and a real WorldEntityInfo/Avatar repository. It generates independent
admission, next-cycle retention, active-context and rebuild controls. RSProt
rebuild sizes are zone counts, not tile counts. The oracle initializes the root
build area and desktop extended-info writer, and advances allocation identity
for the retained cycle. `scripts/child-world-fixture-parity.ps1` reproduces the
control without starting Kotlin or Rust servers and compares it with the synthetic
packet controls in the Rust source. Original library JARs remain operator-provided
private test dependencies. Encrypted TCP tests additionally use the preserved
private login fixture; that fixture and its RSA material are not published.

## Acceptance boundary

Server tests check original-library bytes, independent observers, retained
child/NPC state, invalid inputs, encrypted TCP packet order and unchanged root
behavior without opt-in. A native lease is prepared only after server validation,
and starts only after archived client hashes and explicit BUILD READY.

Native acceptance must record actual child admission, generation-specific scene
loading, decoded NPC state and known transforms alongside real frame captures.
Admission or actor decoding alone does not establish rendered pixels. Keep a
root-only control, save acknowledgments, logical account-state comparisons and
graceful shutdown evidence. Performance acceptance remains a separate question.
