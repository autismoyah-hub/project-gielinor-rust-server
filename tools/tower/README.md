# Cinderworks normal-action acceptance tools

These Python controllers consume the opt-in client adapter in `docs/tower/client-validation.patch`. The native window and headless example both use the same `ClientState` action encoding and real loopback server. No desktop input, focus activation, save editing, item spawning or world mutation is used.

Use only an isolated server, disposable accounts, owned action directories and private output directories. Apply the four-commit client patch to its documented baseline, build the native binary and `tower_actions` example, then set `TOWER_ACTION_DIRECTORY` when launching either client. Supply its cache path, loopback address, username, password-file path and public-key path through the client's existing launch arguments. The server's normal first-login registration path creates a fresh profile; a pre-created account without an initial logout is intentionally rejected by authentication.

A new action directory needs an initial `command.txt` containing `1 observe`. Commands are sequence-numbered and atomically replaced. Receipts acknowledge dispatch only. Every scenario additionally waits for server-delivered position, items, messages or journal state. Cache onOp listeners can submit actions without a direct IF_BUTTON mask; the controller sends such a request once and verifies its outcome.

Run `services.py --directory ACTIONS --output EVIDENCE --onboard --gate-check` for ordinary appearance/experience choices, plaque, prerequisite rejection, mining, smelting, forging, wielding, net fishing, item-on-range cooking, creature combat, bones pickup and item-on-altar offering. `--stage kitchen` or `--stage altar` resumes only a deliberately interrupted route with its actually earned items. Food consumption includes its normal action delay before a pickup retry.

Run `recovery.py` with the same directory/output options to intentionally stand in real Imp vents until lethal damage, then assert sanctuary rescue preserves equipment, items and services without granting an unlock. This is an intentional defeat scenario; it does not accept a retreat as victory.

Run `guardian.py imp` and then `guardian.py warden` with the same directory/output options. The bounded controller reads real smoke/eruption graphics, moves, eats and searches coolant barrels. It records authoritative phase messages and victory. `--label` distinguishes attempts; failures remain failed. A victory stops combat with normal movement out of the court. Pattern geometry is deterministic; ordinary combat rolls vary, so bounded timeouts can fail and must never be silently classified as success.

Run `claim.py --directory ACTIONS --output EVIDENCE --full-pack` after the first victory to fill the pack by normal fishing, prove atomic claim denial, drop two raw shrimp, claim exactly 500 coins plus a beer, and prove repeated searches pay nothing. Replay uses `--coins 25` without `--full-pack`. The claim message is emitted only after account-worker save acknowledgement. Retain before/after observations across real disconnect/relogin and restart separately.

All IDs and coordinates are specific to the pinned OSRS239.1 cache and this authored origin. Logs and actual native framebuffer captures remain private. These scripts cannot replace final visual review or be used to claim acceptance when an assertion fails.
