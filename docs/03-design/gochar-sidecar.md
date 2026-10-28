# The gochar sidecar: a chart's natal facts, kept for its transits

Status: `built`, 2026-10-28. `teistro_serial::sidecar::GocharSidecar`,
opened by `sdk.chart().natal(&sidecar)`; measured on
[`gochar-sidecar-measured.md`](gochar-sidecar-measured.md) (`cargo xtask
dossier`, held by `check-dossier`). Derives from
[`serial-and-the-envelope.md`](serial-and-the-envelope.md) §1, which
named the sidecar as the module's later row, and from the parity
checklist's serialiser row
([`02-parity-requirements.md`](../01-research/baseline-engine/02-parity-requirements.md)).

## 1. What it is

A chart is stored as text that never changes: the date-free dossier
([`dossier.md`](dossier.md)), whose bytes are the same for the same chart
on every machine. A transit cannot be stored that way, because it is a
question about *now*. So an app that keeps a chart's dossier needs a
second thing beside it: what a transit reading asks of the natal chart,
in a form it can keep with the dossier and read back years later, without
the chart's input and without founding the natal chart again.

That is the **gochar sidecar**: the natal facts every transit reading in
the SDK reads, cut from a sealed chart document, written as one canonical
JSON object, and tied by hash to the chart it was cut from and the
settings it was computed under. It holds no transit and no instant other
than the birth's own, so it is date-free in the dossier's sense: the
same chart gives the same sidecar bytes on any day.

## 2. What the baseline engine did, and what is kept

The baseline engine's serialiser (1.3.0 onwards) writes a sidecar beside
its date-free text: the natal Moon's and lagna's signs, the planets, and
the Ashtakavarga's tables, a format number `1`, persisted as a JSON
column on the chart's row. At read time a transit overlay is computed
from it at the conversation's instant, and a row without one is an
integrity error rather than a slow fallback.

Kept: the purpose (fresh transits over stored natal facts), the format
number, and the posture that a sidecar which does not belong is refused
rather than read. Changed, each for a reason:

- **The fields are the SDK's own read set, not a projection.** The
  baseline picked fields by reading its gochar pipeline. Here every
  transit operation already takes a natal `Document` and reads only some
  of its sections; the sidecar carries exactly those sections whole, so
  the existing operations read an opened sidecar unchanged and there is
  no second set of readers to keep in step. §3 lists what reads what.
- **Tables are not stored when they are derived.** The baseline stored
  its Ashtakavarga tables because its read path had no engine. The SDK's
  Ashtakavarga transit judgement recomputes the natal prastara from the
  foundation's signs, which is arithmetic over eight signs, so storing
  the tables would be a second copy of a fact that can disagree with the
  first.
- **It is tied to its chart and its settings by hash.** The baseline's
  row ties sidecar to text by the row. The SDK's sidecar says which chart
  it was cut from and under which settings, so it can be checked against
  either wherever it is read (§4).

## 3. What it holds

Every operation that reads a natal chart for its transits, and the
sections it reads:

| operation | reads |
|---|---|
| `chart().gochar` (and its Ashtakavarga judgement) | the foundation: the Moon's sign, the lagna's sign, the seven's and the lagna's signs for the prastara, the place |
| `chart().sade_sati`, `sade_sati_many` | the foundation: the Moon's (or the lagna's) longitude, the place |
| `chart().hits`, `hits_many` | the foundation: every natal point, the place; the midheaven from its angles |
| `chart().sarvatobhadra`, `sarvatobhadra_many` | the foundation (the Moon's star and sign, the place, the kind) and the **panchanga** (the birth's tithi and weekday) |

So the sidecar is the document's **foundation**, and its **panchanga**
when the document carries one. A sidecar cut from a chart founded
without a panchanga has none, and its Sarvatobhadra is refused exactly
as the document's would be, naming `panchanga`.

```json
{
  "chart": "…64 hex digits…",
  "format": 1,
  "foundation": { "…": "the document's foundation" },
  "natal_hash": "…64 hex digits…",
  "panchanga": { "…": "the document's panchanga, when it has one" },
  "settings": "…64 hex digits…"
}
```

| key | what it is |
|---|---|
| `format` | the sidecar's own shape, `1`; another number is refused with `SCHEMA_VERSION` |
| `chart` | the content hash of the sealed document it was cut from: the hash an app already stores with the chart's provenance |
| `settings` | the settings hash that document was computed under |
| `natal_hash` | the content hash of the natal document the sidecar opens to (the foundation and the panchanga as a `Document`) |
| `foundation`, `panchanga` | the sections themselves, in the document's own shape and schema |

The foundation and the panchanga keep the document's shape so the JSON
Schema describes them once (§7); the sidecar's own keys are the four
above them.

## 4. Its key, and what is refused

Three hashes, each answering one question:

- **Is this the sidecar of that chart?** `chart`. An app stores the
  chart's provenance; `sidecar.check_chart(provenance.content_hash)`
  refuses a sidecar cut from another chart (`INVALID_ARG`, field
  `sidecar.chart`). This is the stale case: a chart recomputed (a correction to
  its birth time, a new SDK) has a new content hash, and the old sidecar
  beside it is refused rather than read.
- **Is it read under the settings it was cut under?** `settings`.
  `sdk.chart().natal(&sidecar)` refuses a sidecar whose settings hash is
  not the context's (`INVALID_ARG`, field `sidecar.settings`). A transit is
  founded in the context's zodiac and read against the natal signs in
  the sidecar's; under another ayanamsha or frame the houses would be
  counted between two zodiacs. The whole settings hash is the check and
  not a narrower one, because it is the one every provenance already
  carries and a second recipe would be a second thing to keep right. A
  caller who means to read a natal chart under other settings has the
  sections themselves (`sidecar.foundation()`, `sidecar.panchanga()`)
  and builds the document by hand, which says so in the code.
- **Are these the facts it was cut with?** `natal_hash`. Reading the
  sidecar recomputes it from the sections read and refuses a mismatch
  (`INVALID_ARG`, field `sidecar.natal_hash`). That catches an edited
  row, and a sidecar written by a build that reads a section's values
  otherwise than this one does. A key this build does not know, at any
  depth, is refused by its path before the hash is taken, which is
  stricter than the document's reader (permissive to read, exact to
  hash, `serial-and-the-envelope.md` §8): a sidecar is read to compute
  with, not kept for display, and a section read short is a wrong
  transit rather than a missing line.

The check is on the canonical hash and **not on the stored bytes**. A
database that keeps JSON in a binary column (PostgreSQL's `jsonb`) gives
it back with its keys reordered and its whitespace gone; the reader takes
any JSON spelling of the object, and the hash, which is over the
canonical form, is what decides. A key the sidecar does not have is
refused by its path (`sidecar.extra`), as every record the SDK reads is,
and a key given twice by its path too.

What is not checked: the SDK version. A sidecar from an earlier release
whose sections read back whole is the natal chart that release computed,
which is also what the dossier stored beside it says; refusing it on the
version alone would make every stored chart unreadable after an upgrade.
An app that wants the current release's natal chart recomputes it and
cuts a new sidecar, and the new chart hash retires the old one.

## 5. Determinism and size

The sidecar is written in the canonical hash form
(`serial-and-the-envelope.md` §4): keys in code-point order, no
whitespace, the number grammar with no exponent. The same chart under
the same settings gives the same bytes, and a sidecar read back writes
the same bytes again. Its sections are the document's, so the bytes of
`foundation` and `panchanga` inside it are the bytes they have inside
the document.

The measured page holds the size against the document's over the
corpus: 0.14 of a document with every section, one sidecar between 15
and 91 KB (the panchanga's size follows its day's). The foundation and
the panchanga are all of it. It is larger than the baseline engine's sidecar, which kept signs
and tables rather than positions, and it is the price of one set of
readers: a narrower record would need every transit operation to read
two shapes.

## 6. The API

```rust
// teistro_serial::sidecar
pub struct GocharSidecar { /* private */ }
impl GocharSidecar {
    pub const FORMAT: u32 = 1;
    pub fn cut(document: &Document, provenance: &Provenance) -> Result<GocharSidecar, Error>;
    pub fn read(text: &str) -> Result<GocharSidecar, Error>;
    pub fn to_text(&self) -> String;
    pub fn check_chart(&self, chart: Hash) -> Result<&GocharSidecar, Error>;
    pub fn chart(&self) -> Hash;
    pub fn settings(&self) -> Hash;
    pub fn natal_hash(&self) -> Hash;
    pub fn foundation(&self) -> &ChartFoundation;
    pub fn panchanga(&self) -> Option<&Panchanga>;
    pub fn natal(&self) -> Document;   // unchecked against a context
}

// teistro (the façade)
impl ChartArea<'_> {
    pub fn natal(self, sidecar: &GocharSidecar) -> Result<Document, Error>;
}
```

`cut` takes the document and the provenance it was published with
(`Envelope`'s two fields, or `Sealed`'s `value()` and `provenance()`),
and refuses a provenance whose content hash is not the document's: a
sidecar that claimed a chart it was not cut from would defeat its own
key. It is the only constructor, and `read` the only reader, so a
sidecar whose `natal_hash` is not its sections' is not representable.
`natal()` builds the document a transit operation reads; `chart().natal`
is the same after the settings check, and the one an app should call.

The flow an app follows:

```rust
// When the chart is computed: keep the dossier, the provenance, the sidecar.
let read = sdk.chart().reading(birth, &request)?;
let text = teistro::dossier::dossier(&read.value, &DossierOptions::default(), &[])?;
let sidecar = GocharSidecar::cut(&read.value, &read.provenance)?.to_text();
// Later, on any day: read it back, check it, open it, read the transits.
let sidecar = GocharSidecar::read(&stored)?;
sidecar.check_chart(stored_provenance.content_hash)?;
let natal = sdk.chart().natal(&sidecar)?;
let today = sdk.chart().gochar(&natal, &GocharRequest::at(now))?;
```

The `GOCHAR` dossier section is written from the opened natal document
the same way, so the stored date-free dossier and a fresh dated `GOCHAR`
block compose into the prompt the baseline engine composed.

## 7. The schema

The sidecar's JSON Schema is generated from its type by `schemars`, as
the document's is ([`document-schema.md`](document-schema.md)), and
checked in as `crates/serial/schema/gochar-sidecar.schema.json` beside
it; `teistro_serial::schema::GOCHAR_SIDECAR` is the text and
`GOCHAR_SIDECAR_ID` its identifier
(`urn:teistro:schema:gochar-sidecar:<version>`). `cargo xtask
document-schema` writes both schemas and `check-document-schema` holds
both. The crate's tests validate a cut sidecar against it.

## 8. What a binding sees

Nothing yet, decided. A binding's transits are asked on the chart
request itself (its `gochar`, `sadeSati` and `dossier` records), which
founds the natal chart and reads its transits in one call from the
stored input, so a binding has no read path on which the natal chart is
missing. The migration guide's advice for a binding is the same as
before: store the input and the provenance, keep the date-free dossier,
and ask for the transits with the chart. Carrying the sidecar across the
C boundary (a chart request writing it, a transit request reading it in
place of the input) is a later row of the boundary, to be built when an
app measures the natal founding as a cost worth removing.

## 9. Tests

- `crates/serial/tests/sidecar.rs`: the same chart cut twice is the same
  bytes; a round trip writes the bytes it read; a reordered, re-spaced
  spelling (as `jsonb` returns it) reads back to the same sidecar; a
  sidecar of another chart, an edited foundation, another format and an
  extra key are each refused by their field; the text validates against
  the schema.
- `crates/sdk/tests/sidecar.rs`: a context of other settings refuses to
  open it; the transits, Sade Sati, the hit list and the Sarvatobhadra
  read from an opened sidecar are those read from the whole document, in
  bytes; a chart without a panchanga opens and its Sarvatobhadra is
  refused naming `panchanga`.
- The measured page, over the corpus: every birth's sidecar reads back
  as itself, its gochar with the Ashtakavarga's judgement and its
  Sarvatobhadra at four instants are the document's, and its bytes
  against the document's.
