# AC-24: a dragged position is remembered. Re-proved live, 2026-08-30

Build: commit `5ffc7f2`, clean tree.

## What was done

The pill was dragged by its grip a hand's width across the screen and
released, during the AC-27 test.

## What was read back

`dictation_setting` for the signed-in account, read straight out of the live
`echoscribe.sqlite3` in `%APPDATA%\com.echoscribe.app\`:

Before the sitting:

```
user_3IXiPaRho7Jkw48Yq8MMHCwCLyC|double_tap_ctrl|1|0.2862|0.6497|2026-08-30T05:32:29Z
```

After the drag:

```
user_3IXiPaRho7Jkw48Yq8MMHCwCLyC|double_tap_ctrl|1|0.4945|0.4365|2026-08-30T09:50:19Z
```

Columns: `account_id`, `hotkey`, `sounds_enabled`, `pill_x`, `pill_y`,
`updated_at`.

Both fractions changed and `updated_at` moved to the moment of the drag. The
values are fractions of the working area, between 0 and 1, as the record's
data model requires. The row belongs to the signed-in account.

## What was confirmed on screen

At the next open, in step 2 of the sitting, the user reported the pill appeared
"where i last closed it from". The stored position is where the pill opens.

## The clause that is still not proved

AC-24 also promises the remembered spot carries "in the same relative spot when
I next dictate on a different screen". Not observed. The machine had only one
monitor connected during this sitting. Blocked, for the second run running.

The across-restarts clause was met on 2026-08-29 and is not re-proved here;
see `AC-24-dragged-position-is-remembered.md`.

## Outcome

AC-24, the remembering and re-opening clauses: **met**.
AC-24, the cross-screen clause: **blocked**. One monitor only.
