# What each consumer surface exercises, measured

Status: `generated` by `cargo xtask exercised` over each surface's own
declarations and its own tests and examples, 2026-09-22. Do not edit:
`check-exercised` regenerates this page and fails on any difference.

`entry-point-is-reachable` holds that every boundary function is
**placed** in a binding — exposed, declared, callable. It says nothing
about whether anyone has ever called it, and that difference cost a
whole corpus: `loadPack` was generated into three bindings and executed
from none, so nothing showed that a record's forms were being dropped on
the way out. This page asks the other question of the four surfaces a
generator does **not** own — the three bindings' layers and the Rust
areas they mirror, which is the one to read the others against.

A member counts as exercised when its name appears after a dot anywhere
in that surface's own tests or examples, so a property read counts as
much as a call and the count errs towards exercised. A member named here
is therefore one nothing touches.

| surface | declares | exercised | untouched |
|---|---:|---:|---:|
| Node | 32 | 32 | 0 |
| Dart | 22 | 21 | 1 |
| Rust | 58 | 58 | 0 |
| Python | 118 | 116 | 2 |

**3 members nothing names**, of 230 members the four surfaces declare.
They are listed rather than counted, because a member that stops being
exercised has to change this page and one that starts has to as well.

- **Node**: every member is touched.
- **Dart**: `callJson`
- **Rust**: every member is touched.
- **Python**: `call_json`, `manifest_json`

| proposed rule | verdict | measured |
|---|---|---|
| every surface is read and its members found | **holds** | 0 of 4 disagree |

