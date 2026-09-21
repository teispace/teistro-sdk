# The reading corpora

Two roots of **entity records** the SDK does not embed: a reading for each
rule a chart can hold, and a reading for what a chart *is* without holding
anything. They are localisation sources, in the same shape as `i18n/`, and
they become `.tpack` files a consumer loads at runtime.

| root | what it reads | records a locale |
|---|---|---:|
| [`readings/`](readings) | every yoga and dosha the SDK ships | 649 |
| [`states/`](states) | a graha in a bhava, a nakshatra, a tithi, a lagna, a graha's condition, an inauspicious kaala, a dosha's timing | 350 |

Five locales each: `en-Latn`, `ne-Deva-NP`, `hi-Deva-IN`, `sa-Deva`, and
`sa-Latn`, which is **derived** from `sa-Deva` by transliteration rather
than translated — `check-state-readings` re-derives it and fails if the
checked-in files differ.

## Why they are not in `i18n/`

`crates/sdk`'s build script compiles `i18n/` into **every artefact the SDK
produces**. These corpora are several times that root's size, so a consumer
computing a Julian day would carry every Sanskrit passage on Jupiter in the
first house to do it. They are loaded, not embedded
([`interpretation-records.md`](../docs/03-design/interpretation-records.md)
§3).

The built packs are **not committed** either: they are barely smaller than
their sources, so storing both would double the corpus in the repository to
save a few per cent. Build them.

## Building them

One pack a locale a namespace, which is the granularity a consumer wants: a
Nepali application loads Nepali and its fallback, not five languages' worth
of prose.

```sh
cargo run -p teistro-intl --bin teistro-intl -- \
  --root packs/readings build --out dist
```

```
| locale | namespace | entries | source bytes | pack bytes |
|---|---|---:|---:|---:|
| en-Latn | sdk.entity | 649 | 476686 | 453786 |
| ne-Deva-NP | sdk.entity | 649 | 777152 | 754549 |
…
```

`--locales` and `--namespaces` narrow it; `--bundle` writes one `.tbundle`
a locale instead. The same command over `packs/states` builds the other
corpus.

## Loading one

The bytes are the whole interface — a file beside your binary, a download,
an asset in your application bundle — and every binding takes them:

| | |
|---|---|
| Rust | `sdk.intl().load_pack(&bytes)?` |
| Node | `ctx.intl.loadPack(bytes)` |
| Dart | `ctx.intl.loadPack(bytes)` |
| Python | `ctx.intl.load_pack(data)` |

A record **merges** with the one already loaded rather than replacing it,
so a pack carrying one form adds that form and leaves the rest of the
record standing: `nakshatra.ASHWINI` ends with its `name`, its `iast`, its
`phala` and its `namakarana` together. The record returned counts what was
kept (`merged`) beside what was not (`replaced`).

**Read a corpus form through `forms`.** A record's forms are an open set,
so each binding's entity carries named fields for the ones `i18n/`
guarantees — `name`, `prose`, `iast` — and a `forms` map of every one it
has, which is where a reading a pack brought will be:

| | |
|---|---|
| Rust | `record.form("phala")` |
| Node | `entity.forms.phala` |
| Dart | `entity.forms['phala']` |
| Python | `entity.forms["phala"]` |

Each binding's tests load a pack and read a form back out of it, so the
path above is exercised in every language rather than described.

Loading one changes what the composers say without changing how they are
called: `readings` says a rule's own reading where the base locale carries
one, and `phala` says nothing at all until a states pack is loaded. The
whole path runs in [`readings.rs`](../crates/sdk/examples/readings.rs) and
[`phala.rs`](../crates/sdk/examples/phala.rs).

## Where they came from, and what is measured

The records are the baseline engine's own Sanskrit, Nepali, English and
Hindi, imported by `teistro-intl migrate readings` and `migrate states`,
which map the engine's keys onto this catalogue's by a written table and
refuse anything they cannot place.

The design is
[`interpretation-records.md`](../docs/03-design/interpretation-records.md)
and [`state-readings.md`](../docs/03-design/state-readings.md); what holds
of them is measured on
[`interpretation-records-measured.md`](../docs/03-design/interpretation-records-measured.md)
and
[`state-readings-measured.md`](../docs/03-design/state-readings-measured.md),
which are generated and gated: every pack builds, every pack loads into an
engine that never read these sources, and every record answers from it
afterwards.

**Neither corpus has been reviewed by a native reader.** It is the engine's
own text and not a machine translation, but the roadmap's `ne`/`hi` sign-off
covers it.
