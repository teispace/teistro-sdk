# `teistro-interpret`

Status: `building`, 2026-09-21: the plan and four composers. The design
is [`docs/03-design/interpret-composers.md`](../../docs/03-design/interpret-composers.md),
measured in [`docs/03-design/interpret-measured.md`](../../docs/03-design/interpret-measured.md).

The composers: what the SDK computed, turned into a **narrative plan** — an
ordered list of message keys and their slots. A plan holds no words, so one
plan renders in every locale the engine carries, and a golden plan tests the
composer rather than the translator.

| module | what it settles |
|---|---|
| [`lib`](src/lib.rs) | `Item` (a message key and its slots) and `Plan` (what a composer says, in order): ordered, serialisable both ways, and the same bytes for the same input |
| [`placements`](src/placements.rs) | where each of the nine grahas stands and who shares a sign, from the rules kernel's own `RuleChart` |
| [`readings`](src/readings.rs) | what each rule a chart held says: its verse's statement, who took part, whether a cancellation moved it, how grave it is |
| [`strength`](src/strength.rs) | each graha's Shadbala in rupas, the strongest first, over `Document.shadbala` — the first composer over a *section* |
| [`houses`](src/houses.rs) | the lord of each of the twelve bhavas, first house first, over `Document.houses` — the first to say what a graha *rules* rather than where it stands |

```rust
use teistro_interpret::{houses, placements, readings};

// A chart the SDK founded, read as the rules kernel reads it.
let mut plan = placements(&chart);
plan.items.extend(readings(held));
// A section composer reads the document instead, and concatenates: a
// report says what it wants in the order it wants, which is what a flat
// plan is for.
plan.items.extend(houses(document.houses.as_ref().unwrap().all()));
for item in &plan {
    println!("{}", sdk.intl().render(&item.key, &item.params).text);
}
```

## What the research found

- **The vocabulary was already there.** `i18n/*/sdk.reason.json` carries
  `grahaInRashi`, `grahaInBhava` and `occupants` in both strict locales,
  translated by hand. The first composer therefore adds **no** message and no
  translation debt, which matters because a strict locale cannot carry a
  missing key and the project forbids machine-translated stubs.
- **The corpus has no oracle for prose.** It records numbers, keys and flags
  and not one composed sentence, so a plan is measured by whether it can be
  *said*: every item of every recorded chart's plan renders in each strict
  locale with no fallback and nothing to warn about (19 488 renderings).
- **A composer says what a locale can say for itself.** `sdk.reading`'s six
  messages are mechanical — a span, a class, a list with an agreeing verb, a
  status, a severity. The verse's own statement crosses as a slot in the
  words the rule cites and is not translated, because a machine translation
  of a cited text would be worse than the visible seam.
- **What a composer cannot say is counted.** The lagna stands in every chart
  and in no plan, because these messages read a graha and the lagna is
  `point.LAGNA`; whether a graha reaches its required rupas has no message
  either, nor has a bhava's sign, its class or a body shifted by the chalit.
  The measured page counts each rather than letting it be missed, and a
  machine-translated sentence for any of them is the stub the project
  refuses.

## Tests

`cargo test -p teistro-interpret`, and `cargo xtask interpret` for the
measurement (`check-interpret` holds the page, its counts and the rendered
snapshot in both languages).
