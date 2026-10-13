# Native Continue visibility reproduction

## Fresh matched driver003 evidence

Kotlin `resume-kotlin-menu-008` PASS: native visible menu hit at [215,450],
handled=true, outbound=[115], NPC response "Are you asking for a fight?".
Save completion35.516seconds, clean unforced exit. This confirms the input-path
correction against Kotlin. The random response is retained verbatim.

Rust `resume-rust-menu-006` with the same driver and new archive1AAA46F4 failed
before target selection: no Talk-to was submitted. It authenticated, loaded the
world, received1356packets by the last progress record, saved, and stopped cleanly.
This is not evidence of a Continue rejection. Final position remained3224,3218.
Driver004 adds native walking to3230,3218 before target selection, and logs nearby
person definitions/positions. This tests whether visibility at the wander-range
edge explains the absent target; that cause is not yet established. Only owned
scenario code changed; pinned client libraries and Agent1 remain unchanged.

Earlier Rust driver002 Continue PASS remains separate historical evidence.

## 2026-10-12 resumed-development correction

The findings below are historical and must not be generalized to the current
visible client. New pin `client-239.1-baseline-001-build-001` has visible Continue
layout in `prep-rust-dialogue-001`; the driver still returned handled=false.
Source tracing established that the driver called `primary_interface_action`,
an IF3-operation-only helper. The actual viewport uses `interface_menu`, drag
handling and `perform_default_menu`/`menu_action`; its menu supports Continue.
Thus the old helper result does not establish failure of the actual native
viewport. The earlier hidden-ancestor evidence remains valid for those old runs.

A corrected owned scenario driver `dialogue-menu-001` uses the same non-menu-
opening, non-drag default-menu branch, a visible hit-test point, and fixed NPC
definition 3108. It never injects opcode 115 in UI mode. Driver SHA-256:
`2714D9484C84B75F4E6E274A40DC6FFD6C18725D9D0AF0094EA25D20B9A0AF47`.
It was built against unchanged pinned client library sources in an owned working
copy. Immutable client pins and Agent 1's active workspace are unchanged.
The first driver stalled before Kotlin authentication. Driver `dialogue-menu-002`
also defers gameplay interface ticking until authentication, avoiding queued
listeners blocking handshake processing. Its executable SHA-256 is
`EDE93F8F6D596E5DB8920F0C9960A3DC5306F85D474B58E04B1734E43B4E2F99`.
Fresh Rust `resume-rust-menu-005` passes: visible Continue menu at [215,450],
handled=true, outbound=[115], followed by NPC response "How can I help you?".
Logout persistence completed after 9.797 seconds, with clean server shutdown.
Matched Kotlin `resume-kotlin-menu-002` did not reach Continue. Its chosen man
was inside the Lumbridge recovery pad and the saved character reached Tower,
with a new kit flag. The run also showed queued timer listeners delaying packet
dispatch, slow reference ticks, and forced shutdown. It is not evidence of a
server Continue rejection. Driver source now leaves a packet-processing poll
between timer refills; that correction remains unvalidated.

This is a reproducible harness finding for future Agent 1 review, not a request
to change client code. No inter-agent message has been sent. Headless menu-path
success, if observed, still requires separate visible-window acceptance.

## Historical evidence

Latest matched evidence: driver004 passes native Continue on Kotlin
`resume-kotlin-menu-010` and Rust `resume-rust-menu-007`. Both confirm the approach
walk before selecting the eastern man2, emit opcode115 through the normal menu,
observe a reply, finish saving and stop unforced. Kotlin replies "Are you asking
for a fight?"; Rust replies "How can I help you?". Both are implemented random
branches, but exact response parity remains FAIL in dialogue-differential-002.
The successful menu path does not prove the historical pointer/layout defect
fixed. The pinned client library and Agent1 workspace remain unchanged.

Historical pointer-path evidence below:

The isolated native client reaches GenericPerson's greeting on both servers but
cannot click Continue. This is not unique to the Rust replacement server. Exact
client-versus-driver attribution remains unresolved; neither client source nor
widget visibility was changed to bypass it.

- Rust: `work/integration/runs/rust-dialogue-ui-006`, server117DA9F8,
  driverD635057D. Player3224,3218; man3108; greeting217:6correct.
- Kotlin: `work/integration/runs/kotlin-dialogue-ui-002`, pinned reference runtime
  and per-run overlay hashes/server arguments recorded there; driver
  `5A12BE571229B8556DB494A589E7E80ED4D29951FC74F4E8B8CF9FF8951F570D`.
  This driver adds read-only ancestor logging. Same starting position/man/greeting.
- Both Continue217:5 layouts are hidden with matching geometry at25,442.
  Kotlin's trace identifies hidden ancestor162:566 (`chatbox:chatmodal`).
  Its ancestors162:55,162:34,162:0,548:11,548:2,548:0are visible.
  The button's own hidden flag isfalse. No RESUME_PAUSEBUTTON115is emitted.
- Both owned servers stopped normally, exit0, unforced. Kotlin reached its
  greeting despite earlier startup-load issues. This result is not a timeout.

Reproduce with `scripts/integration-{rust,kotlin}-login.ps1 -Mode dialogue-ui`
using a new run name and the existing completed-tutorial disposable fixture.
The driver drains both active and queued interface scripts, ticks native
interfaces, waits one second after the greeting, then uses native pointer
dispatch. UI mode refuses protocol injection. Ancestor logging lives only in
`reference/NativeDialogueScenario.rs`, copied into the owned client workspace.

Rust same5A12driver ancestor capture remains pending after a compiler deferral.
Native Continue success and subsequent dialogue branches remain unverified.
Earlier explicit protocol-probe success is separately labeled and is not UI proof.

Read-only inspection of the pinned client snapshot found a candidate mechanism:
`src/interface_state.rs::open` loads the child group and runs its on-load scripts,
but does not dispatch ancestor sub-change listeners. The generic script listener
setter accepts1424; no1424dispatch call was found in the snapshot. This is a
diagnostic lead, not a confirmed root cause. The exact live listener/script chain
still needs inspection. No active Agent1source or process was changed.
