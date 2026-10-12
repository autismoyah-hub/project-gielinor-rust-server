# Native Continue visibility reproduction

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
