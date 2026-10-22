# Rust-owned persistent state

These identifiers are server extensions, not claims about revision 239.1 cache identifiers. They never transmit to the client. Do not add them to protected cache symbol inputs or remap existing state.

- **62400 — `rust_ring_of_forging_used`**. Whole permanent Varp, `Transmit::Never`, bit protected, `client_code=-1`. Values 0–139 count successful ordinary iron bars since the last ring broke. Absent/zero means 140 charges remain. The 140th bar removes the currently worn ring, resets this value to zero and stops that smelting job. Held Break removes the selected ring and resets this player counter. Swapping or banking rings does not reset it. Invalid saved values refuse the modifier and item action without overwriting state.

The forging-ring factory refuses startup if target varp symbols, varp definitions or any varbit base already occupy 62400. This extension is installed in the existing definition registry before character admission, so the maintained character save path persists it without a new schema or item-instance charge field. Existing item, equipment, recipe, packet and interface identifiers remain unchanged.

The actual local target symbols and client source were searched for a forging counter and 62400 before use. No client reservation acknowledgment is claimed. FORGE sent ORACLE the proposed server-only reservation for coordination; no client change or reply is required to read or persist this never-transmitted state. A future cache collision must be explicitly migrated, never silently reassigned.
