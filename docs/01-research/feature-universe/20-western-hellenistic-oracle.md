# The Western and Hellenistic oracle: what exists, and what it costs

Status: research, 2026-09-11. Answers crux **C46**
([`19-verification-cruxes.md`](19-verification-cruxes.md)), which ADR-0025
calls "the largest new unknown in the plan" and which blocks Phase 7's
Western exit — not its build.

C46 states the problem and this page does not restate it: the astronomy
underneath these traditions is already covered, and **what has no oracle
is the doctrine above the astronomy**. What follows is what a survey
found, what it costs, and the one structural finding that changes the
shape of the work.

## The finding that matters

**Valens' worked examples decouple the oracle from the ephemeris**, and
that is the difference between a tractable fixture set and an intractable
one.

Vettius Valens' *Anthologies* carries **more than a hundred example
charts** worked through in the text. Each gives positions *and* the
result the technique yields from them. That is precisely the shape a
fixture takes here — a recorded input beside a recorded answer — and it
is the shape `fixtures/baseline/variants/*--geocentric.json` already
uses, which the project's own notes single out as the corpus's only
recorded *before* as well as after.

Read that way, a Hellenistic fixture records **the positions as inputs**
and the doctrine's output as the answer. Three things follow:

1. **No engine is the oracle.** The text is. That is what C46 asks for
   and what no implementation can supply, since the implementations
   disagree with each other.
2. **No ephemeris is exercised at all**, so nothing about the fixture
   depends on which provider is installed or on how accurate it is. The
   doctrine layer is tested as the doctrine layer.
3. **The 1800–2400 problem disappears.** This is the part that would
   have bitten. Valens wrote in the second century and his charts are
   dated to it; the built-in ephemeris covers **1800 to 2400** and would
   answer `OUT_OF_RANGE` for every one of them. Had the fixtures needed
   computed positions, the whole Hellenistic set would have been
   engine-only and unreproducible for a consumer with nothing but the SDK
   — the exact property Phase 3 spent itself establishing. Recording the
   positions as inputs avoids it rather than working around it.

## Rank 1: the texts, and whether they can be had

C46 lists the rank 1 texts as "unread here". They are obtainable, and
three of the six are outright public domain.

| text | edition surveyed | status | what it enumerates |
|---|---|---|---|
| Ptolemy, *Tetrabiblos* | Ashmand 1822 | **public domain**, Project Gutenberg #70850, plain text and HTML | the three term systems, essential dignity, aspect doctrine |
| Vettius Valens, *Anthologies* | Riley, released free by the translator | freely published; **not** a stated open licence | 100+ worked charts, lots, zodiacal releasing, sect |
| William Lilly, *Christian Astrology* | 1647 | **public domain** | dignity tables, orbs, horary considerations |
| Dorotheus, *Carmen Astrologicum* | Pingree; Dykes | in copyright | triplicity lords, lots |
| Firmicus Maternus, *Mathesis* | Rhys Bram; Holden | in copyright | terms, decans |
| Bonatti | Dykes | in copyright | later medieval synthesis |

**What "public domain" buys and what it does not.** It settles
redistribution of the *text*. It is not what makes a fixture safe — a
fixture records facts (a chart's positions, a technique's output), and
facts do not carry the copyright of the book that reports them. So a
citation under ADR-0018 is the requirement, and it is one the project
already imposes on every Vedic row. The in-copyright translations can be
*cited and read* like any other reference; what would be wrong is
reproducing their prose, which no fixture needs to.

## The terms are the place to start, and they are enumerable

C46's first step is "derive what the texts state where they enumerate",
on the model of the nakshatra tables. The terms are the clearest case:

- **Three systems, and they disagree by construction.** The Chaldean
  divides every sign the same way (8–7–6–5–4); the Egyptian and the
  Ptolemaic are irregular, with a different division per sign.
- **Ptolemy transmits the Egyptian set and argues against the Chaldean**,
  and then gives his own. One public-domain text therefore carries all
  three and says which it prefers and why — a source that records the
  disagreement rather than a source that has to be reconciled with
  another.
- They need **no ephemeris, no chart and no instant**: a term table is a
  static division of the zodiac, so the pass that derives it is pure
  reading, and its gate is a generated page like every other.

Sect and the lots are the same shape — enumerable, stated, and
disagreeing between authorities in ways the sources themselves name (the
day and night reversal of each lot is the standard example). Zodiacal
releasing is not: it is an algorithm, and its oracle is Valens' worked
examples rather than a table.

## Rank 2: the implementations, and why none can be a dependency

| implementation | licence | usable as a dependency? |
|---|---|---|
| `flatlib` (Python) | MIT source, but rides on Swiss Ephemeris | **no** — Swiss Ephemeris is AGPL, which `deny.toml` refuses (ADR-0019) |
| `immanuel-python` | AGPL | **no** — refused outright |
| Astro Gold, Solar Fire, Delphic Oracle | commercial | no |

This is less of a loss than it looks, because C46 already establishes
that **no implementation is rank 1**: they disagree with each other, so
adopting one would be choosing a school rather than finding an oracle.
What they remain useful as is *witnesses to the disagreement* — running a
program and recording what it answered is not linking against it — and
the disagreement is itself something C46 asks be **measured rather than
resolved**.

`immanuel-python` is worth one specific note: its dignity scores follow
Astro Gold's. That is a school, stated openly, and exactly the kind of
thing that should become a **cited row** rather than a default.

## What this recommends

C46's three steps, in its own order, with the first two now costed:

1. **Derive the enumerable doctrine from Tetrabiblos** — the three term
   systems, essential dignity, sect — as a falsification pass writing a
   generated, gated page. No ephemeris, no fixtures, no engine. This is
   the cheapest work in Phase 7 and it is available now.
2. **Build the Hellenistic fixture set from Valens' worked examples**,
   positions recorded as inputs. Larger, and the reading is the cost
   rather than the coding.
3. **Register every school as a cited row**, with unsourced rows shipping
   `UNSUPPORTED (unsourced)` under ADR-0018 — the same contract a Vedic
   row gets, which is what keeps "Western support" from meaning "whatever
   one implementation happened to do".

**The Western side is thinner and the gap is real.** Orb tables per
aspect and per school, which progressions and by which arc, whether a
return is precession-corrected and whether it is relocated: Lilly gives
orbs and the older doctrine, but the modern apparatus — secondary
progressions, solar arc directions, harmonics — is twentieth-century and
its sources are in copyright. Those rows will be cited to modern authors
or they will ship unsourced, and this page does not pretend otherwise.

## What this does not establish

**That the texts say what the survey says they say.** Nothing here has
been read against the primary sources; this is a survey of what is
obtainable, at what licence, in what form. The reading is step 1 and its
output is a gated page, which is where a claim about Ptolemy's terms
becomes checkable rather than asserted.

**Whether Valens' examples reproduce.** His positions were computed with
ancient tables and are known to differ from a modern ephemeris for the
same instants. That does not affect a fixture that records them as
inputs — the doctrine is what is under test — but it does mean a fixture
set that tried to *verify* his positions would be measuring the accuracy
of second-century arithmetic. It is the doctrine that is the oracle, not
the astronomy, and this is the sharpest form of C46's own point.

**Sources.** [Tetrabiblos, Ashmand 1822 (Project
Gutenberg)](https://www.gutenberg.org/ebooks/70850) ·
[Tetrabiblos at the Internet
Archive](https://archive.org/details/ptolemystetrabib00ptol) · [Valens,
*Anthologies*, Riley translation
(Skyscript)](https://www.skyscript.co.uk/valens_riley.html) · [Riley
translation reformatted by Jane
Griscti](https://www.skyscript.co.uk/valens_janegca.html) · [The
Anthology of Vettius Valens (The Astrology
Podcast)](https://theastrologypodcast.com/2022/10/17/the-anthology-of-vettius-valens/)
· [flatlib on PyPI](https://pypi.org/project/flatlib/) · [flatlib
FAQ on its Swiss Ephemeris
licensing](https://flatlib.readthedocs.io/en/latest/faq.html) ·
[immanuel-python](https://github.com/theriftlab/immanuel-python) ·
[Terms, Astrodienst
Astrowiki](https://www.astro.com/astrowiki/en/Terms) · [The Ptolemaic
bounds](https://www.taniadaniels.com/ptolemaic-bounds-revealed/)
