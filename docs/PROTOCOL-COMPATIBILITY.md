# Revision 239.1 protocol evidence

Protocol authority is the pinned RSProt artifact `1.0.0-ALPHA-20260630` used by the isolated Kotlin server. The active Rust client workspace remains read-only. These notes identify integration checks; no client code was changed or client compatibility claimed.

## Player appearance and facing

`scripts/appearance-parity.ps1` executes the original RSProt `PlayerAppearanceEncoder` and `PlayerFacingEncoder`. The Rust comparison passed 512 appearance cases and 512 facing cases, including transformed NPC appearance and all four facing kinds. Fixture: `crates/protocol/tests/fixtures/appearance.expected`; evidence: `work/appearance-parity.log`.

Appearance uses update mask `0x20`, a byte length transformed as `128 - length`, and payload bytes transformed by adding 128. Its raw payload contains equipment followed by a second identikit-only array, colors, seven animations, name/combat/skill/hidden fields, customization flags, three name extras, and pronoun. Item look offset is 2048; identikit offset is 256. Customization payloads beyond the refresh flag remain unimplemented in Rust.

Facing is the **unified `0x1` block**, preceding movement speed in the extended-info writer. It is not the old `0x10` three-byte entity-index block. The first byte is negated and combines walk mode in bits 0–2, kind in bits 3–4, and instant in bit 6. Kind values: entity 0, location 1, angle 2, reset 3. Entity data contains entity type (smart 1/2), index (nullable smart 2/4), and fallback angle (smart 1/2). Location data contains x, z, and packed dimensions as smart 1/2 values. Angle uses one smart 1/2; reset has no additional data.

The provisional Rust encoder's legacy action-string and face-entity blocks were removed after this comparison. Name extras belong inside appearance. Future stable-client integration should explicitly verify the unified facing decoder and this field order.

## Other verified boundaries

The original oracle also covers 1,024 inventory full/partial, rebuild-normal, and stat payloads, plus 5,125 ISAAC outputs. The encrypted login fixture validates RSA/XTEA parsing, session transition and all 23 cache CRCs. Real loopback tests cover encrypted movement through server-authoritative routing and persistence.

Four-pass player-info tracking currently has Rust unit tests; a complete stateful RSProt player-info differential corpus remains outstanding. The per-observer player synchronization adapter is connected to runtime output when Presentation is configured; its encrypted TCP test passes. NPC low-resolution additions and retained movement match 1,024 original RSProt cases. Root-world/NPC runtime integration passed the 139-test workspace suite, including actual ISAAC-framed TCP admission/removal. Context/run/stop-transmit payloads match 1,032 original RSProt cases; inventory transmission ordering/output/weight matches 1,024 original Kotlin processor scenarios. Inventory, stat, weight and energy updates are connected to sessions. Interface state, button/drag payloads and outgoing interface packets have independent Kotlin/RSProt comparisons and an encrypted session test. Food chat/sequence payloads match 2,048 RSProt cases; consumption, healing, delay expiry and dragging are session-tested. Dynamic world entities, zones, complete login scripts, full stateful synchronization comparisons and playable client/server integration remain unfinished.


## Bank and variable session evidence

Immediate varp and varbit writes, reset/resync, and mixed client-script arguments match 2,049 RSProt cases. Banking TCP checks cover open/event refresh, authoritative deposits/withdrawals, count prompts, signed responses, prompt abort on main-modal closure, and stop-transmit. The 139-test full run is recorded in work/bank-dialog-workspace-tests.log. A further 2,048 signed count decoder cases match RSProt (work/count-dialog-parity.log). Settings/grid-drag additions after this batch remain under validation.

## Login and interface continuation

The latest complete suite has 164 passing tests (`work/journal-workspace-tests.log`). Revision 239 login initialization passes 1,024 actual LoginScript variable cases, 1,538 RSProt packet cases and encrypted TCP ordering checks. Gameframe selection passes 2,048 actual Kotlin cases. Open callbacks, prayer/settings mutations, journal opening and tab switching have additional compiled Kotlin/RSProt comparisons and TCP coverage. Remaining resize/move output is under validation; complete content, JS5 service, zones and playable client integration are still outstanding. Earlier test counts above identify historical batches, not the current total.
