# What the bindings exercise, measured

Status: `generated` by `cargo xtask exercised` over each binding's
hand-written layer and its own tests and examples, 2026-09-22. Do not
edit: `check-exercised` regenerates this page and fails on any
difference.

`entry-point-is-reachable` holds that every boundary function is
**placed** in a binding — exposed, declared, callable. It says nothing
about whether anyone has ever called it, and that difference cost a
whole corpus: `loadPack` was generated into three bindings and executed
from none, so nothing showed that a record's forms were being dropped on
the way out. This page asks the other question of the layer a generator
does **not** own.

A member counts as exercised when its name appears after a dot anywhere
in that binding's tests or examples, so a property read counts as much
as a call and the count errs towards exercised. A member named here is
therefore one nothing touches.

| binding | declares | exercised | untouched |
|---|---:|---:|---:|
| Node | 32 | 28 | 4 |
| Dart | 24 | 21 | 3 |
| Python | 117 | 95 | 22 |

**29 members nothing names**, of 173 members the three layers declare.
They are listed rather than counted, because a member that stops being
exercised has to change this page and one that starts has to as well.

- **Node**: `dashaName`, `pack`, `range`, `unpack`
- **Dart**: `callJson`, `standing`, `twelve`
- **Python**: `ayana`, `ayanamsha_offset_deg`, `brahma`, `call_json`, `canonical`, `chalit`, `day_elapsed`, `day_lagna_deg`, `day_part`, `disha_shool`, `keeps_its_sign`, `last_error`, `manifest_json`, `moon_signs`, `muhurta_yogas`, `pack`, `panchaka`, `place`, `range`, `sun_signs`, `unpack`, `window`

| proposed rule | verdict | measured |
|---|---|---|
| every binding's hand-written layer is read and its members found | **holds** | 0 of 3 disagree |

