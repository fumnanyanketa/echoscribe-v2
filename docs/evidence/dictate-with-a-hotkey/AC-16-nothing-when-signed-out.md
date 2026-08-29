# AC-16: the hotkey does nothing at all when nobody is signed in

Live run, 2026-08-29.

## Outcome: met

## What was observed

Step 1. The app started signed in as fumnanya nketa. The user clicked
Sign out, waited for the window to change, clicked away so EchoScribe was
not the focused app, then double tapped Ctrl three times, using the left
Ctrl and the right Ctrl.

User: "nothing visible and nothing audible."

No pill on either monitor, and neither sound played.

## Scope of this evidence

The user confirmed they followed the step as written, which included
waiting for the window to show the signed out screen. This session did not
independently see that screen. If sign out had silently failed, the same
"nothing happened" would instead have been an AC-1 failure. The user was
asked to confirm and said the instructions were followed.
