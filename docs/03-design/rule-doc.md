# Rule to prose: what a rule says, in words an astrologer reviews

Status: `built`, designed and built 2026-09-20; measured in
[`rule-doc-measured.md`](rule-doc-measured.md). It builds the review gate
[`rules-engine.md`](rules-engine.md) names under "Validation gates at pack
build" — *every rule renders to prose (`teistro rule-doc`) so an astrologer
can review a change without reading the schema* — and it is the first step of
Phase 6's interpretation layer ([`../07-roadmap/00-roadmap.md`](../07-roadmap/00-roadmap.md)).

## 1. Purpose and scope

The kernel ships 997 rules as JSON. The people who can say whether a rule is
*right* read verses, not `{"type": "planet-in-kendra-from", …}`. This page
decides how a rule becomes English: one sentence for a condition, one short
passage for a whole rule, from the kernel's own types, byte-stable, and held
by a gate so a phrasing that drops a field fails the build.

In scope: the vocabulary, how conditions compose, what of a rule must appear,
who else reads the renderer, the command, and the measurement.

Out of scope: **reading a chart's answer** to prose — a present rule with its
participants and its dasha windows — which is Phase 6's `interpret` and its
composers; and **other languages**, which wait for `teistro-intl migrate
baseline`. §7 says what each would need from this.

## 2. What the research found

- **Half the vocabulary is already written.** `crates/rules/src/reference.rs`
  has a section called *Prose*: `BodyRef` and `SignRef` render themselves as
  "the lord of house 7 from the upapada", "the pada of the 4th". The trace's
  `Resolved` composes them into the lines `Explanation`'s `Display` prints.
  What is missing is the predicate around the reference, and the rule around
  the predicate. This is an extension of an existing section, not a new one.
- **The trace prints the schema's own word, which is the thing being
  avoided.** `Step` today writes `holds planet-in-kendra`: an astrologer
  reading an explanation is reading the JSON's type tag. The renderer has a
  second reader waiting, and replacing that word is how the two stay one
  vocabulary rather than two that drift.
- **The language has 64 condition kinds. 51 appear in the shipped packs, 11
  more only in the corpus's own rules, and two — `planet-in-nakshatra` and
  `vipareeta-argala` — appear in no pack at all.** A renderer measured only
  over the packs would leave its rarest sentences unread, so the two unread
  kinds are named here and are carried by the golden test instead, and the
  golden test covers every kind rather than a sample.
- **A rule is much more than its conditions.** Of the shipped rules, 240
  carry an effect in words, 25 a class of life, 19 a span; some carry
  reference groups with labels, cancellations with a threshold, a severity
  rule, remedies, a scope, a timing and a citation. A rendering of the
  conditions alone would let a changed severity or a changed threshold pass a
  review unseen, which is exactly the failure the gate exists to prevent.
- **The corpus cannot say whether a sentence is *good* English, but it can
  say whether it is *faithful*.** No text records the prose of a rule, so
  nothing here is derived from a recording. What the corpus can falsify is
  collision: if two conditions that differ render alike, a change from one to
  the other is invisible to the reviewer. That is measurable over every
  condition in every pack, and it is this page's falsification.
- **Measured, "differ" turned out to mean *differ in meaning*, not in
  shape.** The first run found seven renderings shared by two spellings, and
  every one of the seven was one meaning written twice: a combinator holding a
  single condition, and `chara-karaka-in-house` beside `planet-in-house` of
  that karaka, which the evaluator resolves identically. Prose renders
  meaning, so two spellings of one meaning *should* read alike; §5 states the
  gate over meanings and reports the spellings that share a sentence beside
  it. Three of the seven were single-armed combinators in a shipped pack and
  were simplified rather than excused — the pass found a defect in the rules
  on its first run, which is what a pass is for.

## 3. The rendering

### A condition is one sentence

`impl Display for Condition`. A predicate names its subject as the existing
prose does, then says what is asked of it:

```text
MARS stands in the 1st, 4th, 7th or 10th house
the dignity of the lord of house 9 is exalted, mooltrikona or own sign
SATURN aspects the 7th from the upapada
at least three malefics stand in the 6th, 8th or 12th from the MOON
```

- **Bodies, signs, nakshatras and the rest are named by their catalogue
  keys** — `MARS`, `ARIES` — and not by display names. A reviewer is
  comparing prose against the rule's own JSON, and the key is the word both
  carry. Dignities are the exception: their catalogue documentation is
  already a phrase ("Own Sign"), so they read as words.
- **Houses are ordinals**: "the 1st, 4th, 7th or 10th house", "the 7th from
  the MOON". A list is joined with commas and a final "or"; "and" where every
  member is required.
- **Combinators compose in line, and nest with parentheses.** `and` joins
  with ", and", `or` with ", or", `not` prefixes "it is not the case that".
  A combinator inside a combinator is parenthesised, so precedence is never
  read wrongly.
- **Every field of a condition appears.** An orb, a `padas` list, an `except`
  list, an `atMost`, a `karakaScheme`, a `windowHours` — each has its clause.
  The gate in §5 is what holds this true.

### A rule is a short passage

`impl Display for Rule`. Its lines are always in this order, and a line is
absent only when the rule has nothing to say there:

```text
MAHAPURUSHA_RUCHAKA — mahapurusha. Saravali ch. 37 vv. 5-7 (rank 1).
Note: Mars in his own sign or his exaltation, standing in an angle from the ascendant. …
When:
  MARS stands in a kendra, and
  the dignity of MARS is exalted, own sign or mooltrikona
Then: long of face, … a ruler of the Vindhya and the Sahya and a life of 70 years.
```

and, where a rule carries them:

```text
AN_EXAMPLE — planetary, a match. BPHS ch. 33 v. 12 (rank 1).
When:
  the birth fell by day
Found from any of:
  LAGNA, "Mars in an afflicted house", weight 2:
    MARS stands in the 1st or 4th house
Cancelled by:
  "Jupiter aspects Mars": JUPITER aspects MARS
  MARS is retrograde
  2 of them cancel it fully; fewer cancel it in part.
Severity: 25 for each place it is found from, at most 100.
Then: trouble in marriage and a medium life, 64 years.
Felt in every period.
Remedies: MANGAL_SHANTI.
Computed in code as KUJA_DOSHA: the language does not say it.
```

The scope is named only where it is not a birth chart, and the timing only
where a rule is felt in every period rather than in the periods of the grahas
concerned: a line for a default is a line a reviewer learns to skip.

The passage is plain text with two-space indentation, as `Explanation` is: it
reads in a terminal, in a code block and in a review comment, and it needs no
Markdown escaping for a key that contains an underscore.

### One renderer, three readers

The kernel's prose module is the only place English is written about a rule.
The trace reads it (a step prints its condition's opening in place of the
type tag, the conditions inside it being the steps beneath), the command
reads it, and a consumer displaying a present rule reads it through
`teistro::rules`. A second phrasing of a predicate
anywhere else is a defect.

## 4. The command

`cargo xtask rule-doc [<pack>|<key>|<category>]` prints the passages to
standard output: with no argument every shipped rule, otherwise the pack
(`nabhasas`, `arishtas`, `gandantas`, `readings`, `doshas`, `yogas`, or
`corpus` for the corpus's own), the one rule with that key, or every rule of
that category. It is a workspace command
and not a shipped binary, because the SDK ships the renderer as a library and
the prose belongs wherever the consumer puts it — a page, a tooltip, a diff.

The full text of 997 rules is **not** checked in: it is some 400 KB of
derived words, and a generated page the size of the pack it derives from
buys a reviewer nothing that the command does not. What is checked in is the
measurement, which carries one rendered sentence for each of the 64 kinds —
so a change in phrasing shows up as a diff of 64 lines, in review, where it
can be argued about.

## 5. The measurement, and the gate

`cargo xtask rule-doc` (the pass) writes
[`rule-doc-measured.md`](rule-doc-measured.md) and `check-rule-doc`
regenerates and diffs it, as every other measured page is held. It reports:

1. **Collisions.** Every distinct condition in every shipped pack and in the
   corpus's own rules, rendered; any two that **differ in meaning** and read
   alike are named. **Zero is the gate**, and it holds: 5415 conditions are
   written 2600 ways, which say 2596 things, and the renderer gives those
   2596 sentences. Two spellings of one meaning are named in a section of
   their own rather than failed, and the identities that decide "one meaning"
   — a combinator with one condition in it, and the two spellings of a chara
   karaka's house — are named in the pass and nowhere else. A dropped field
   shows here as soon as two rules differ only in that field, which over 997
   rules is the usual case rather than the lucky one.
2. **Coverage.** Each of the 64 kinds with how often it occurs, and one
   rendered example for each: the table a reviewer reads to argue about the
   words themselves.
3. **Size.** The longest and the shortest passage, and the rules whose
   conditions nest deepest — the places where the prose is at its weakest and
   a reader should look first.

Beside the pass, in the crate: a golden test asserting the exact sentence for
**every** one of the 64 kinds, written by hand rather than captured, so a
phrasing cannot change silently; a test that each field a rule may leave out
changes the sentence when it is there, for the twelve fields rare enough that
no two rules might differ in exactly one of them; the `Display` of a whole
rule asserted for one rule carrying each of groups, cancellations, a
threshold, severity, remedies, an effect, a class, a timing and a computed
key; a golden for each of the five severities and the three outcomes, four of
which occur in no pack and would otherwise be read by nothing; and a test
that every shipped rule renders.

The list the pass walks is the language's own: `language::KINDS`, held
against the arms of `Condition::kind` both ways by `check-lints`'
`every-predicate-is-listed`, because a kind missing from a private list would
simply not be reported.

## 6. Order of work

1. `crates/rules/src/prose.rs`: the vocabulary, `Display for Condition`, and
   the golden test over all 64 kinds.
2. `Display for Rule`, its golden tests, and the trace reading the sentence
   in place of the type tag.
3. The `rule-doc` command and the pass, the measured page, `check-rule-doc`
   in `fast-check.yml`.
4. The façade. **Nothing to do**: `teistro` already re-exports the kernel as
   `teistro::rules`, and the prose is `Display` on the types a consumer
   already holds, so `println!("{rule}")` works wherever a `Rule` does — a
   re-export of its own would be a second name for one thing.

## 7. What this design does not settle

- **Reading an answer, not a rule.** "MARS stands in a kendra" is the rule;
  "Mars stands in the 10th, in Capricorn, exalted" is the chart's answer to
  it. The second needs the `RuleResult`'s participants and houses, and is
  `interpret`'s first composer. The vocabulary here is what it will compose
  from, which is why the subjects are rendered from `BodyRef` and `SignRef`
  rather than flattened into strings.
- **Other languages.** The renderer writes English from the catalogue's keys.
  A locale needs the keys mapped to names (`teistro-intl` has them for the
  entities already) and each predicate's sentence as a message with
  arguments. That is a translation of this module's sentences and not a
  second renderer; the golden test is the message list it would start from.
- **A rule's own prose, written by its author.** Some rules could carry a
  hand-written sentence better than any rendering. Nothing here forbids it,
  but a field for it is not added until a rule needs one: a hand-written
  sentence that no gate compares against the conditions is how a rule and its
  description drift apart.
