# The four microphone error kinds: where each stands after the 2026-08-31 sitting

Asked for by the sitting's backlog, item 2. Nothing here was re-run; this
file says plainly what stands proved and what remains blocked, so the verdict
table has one place to cite.

| Kind | Standing | Where it was proved |
|---|---|---|
| `microphone_blocked_by_windows` | proved live 2026-08-30 | `AC-15-28-29-microphone-errors-live.md`: machine-wide toggle off, then desktop-apps toggle off, each read `MICROPHONE_BLOCKED_BY_WINDOWS` with Open Windows settings |
| `no_microphone_found` | proved live 2026-08-30 | same file: every input device disabled, the screen read `NO_MICROPHONE_FOUND` with Try again |
| `microphone_in_use_by_another_app` | **blocked, machine limitation** | nothing on this machine takes the microphone exclusively; shared-mode capture is the Windows default and every app tried on 2026-08-30 used it. Cannot be forced here. |
| `microphone_unavailable` as the visible catch-all | **blocked, machine limitation** | with the consent switches allowing and a device present, no failure that is neither busy nor missing could be forced on 2026-08-30, and nothing about the machine has changed since |

The two blocks are the same two the 2026-08-30 sitting recorded, restated
rather than rounded up. Both kinds are unit tested (the mapping and the
sentences), and the catch-all's classification path was seen live once, on
2026-08-30, when it wrongly caught a privacy block before step 2b landed.
What has never been seen is either kind arriving live through the fixed
build's screen on this machine, and this sitting does not claim otherwise.

AC-29's two remaining clauses, the privacy page actually opening and a
successful Try again, stay as the user left them on 2026-08-30: their
decision to stop, recorded as a block, not a defect. Confirmed unchanged in
this sitting by the user's own instruction.
