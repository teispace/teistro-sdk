# Festivals in an adhika or a kshaya year (SR-93)

Status: `built`, 2026-11-16. Written from the texts first; §5 measures
the built rule.

A lunisolar year sometimes doubles a month's name (an **adhika** month,
then the **nija** month of the same name) and, far more rarely, joins two
names in one lunation (a **kshaya** month, which holds two sankrantis). A
festival rule names a month and a tithi; this page says where such a rule
falls in those years, why, and how an answer says so.

The consumer's finding (addendum 8, SR-93): over BS 2085 (August 2028 to
April 2029), a year with an adhika Kartika and a kshaya lunation joining
Margashirsha and Pausha, `nepali-committee` and `nepali-drik` differed on
85 observances, and seven or more Pausha festivals were missing from the
committee's answer: the kshaya lunation was named by its first name only,
so a rule asking for Pausha found no Pausha.

## 1. The texts

Ranked as `LANE-RULES.md` asks; page numbers are of the page images read.

| rank | source | what it says |
|---|---|---|
| 1 | *Dharmasindhu* (1888 Nirnaya Sagar edition), p. 3 | the principle: a rite with no other time (*ananyagatika*) is kept even in an adhika or a kshaya month; one with another time (*sagatika*) is not. A kshaya lunation is "a single month made of the two, Margashirsha and Pausha" |
| 1 | *Dharmasindhu* p. 4 | among what an adhika month bars: the festivals ("शयनपरिवर्तनाद्युत्सवः"); in a kshaya year the Magha bathing begins on the full moon of the kshaya month that holds the Sun's entry into Makara, and the annual shraddhas of both months are kept in it "without dividing the tithi" |
| 1 | *Dharmasindhu* p. 31 | the year's beginning (*samvatsarārambha*) on the adhika Chaitra's first day |
| 1 | *Dharmasindhu* p. 38 | Dashahara kept in the adhika Jyeshtha when Jyeshtha is doubled |
| 1 | *Nirnayasindhu* (Dhakal's edition, VS 2071), p. 28 | the *Kaladarsha*: what is barred is kept in the second month, and the gods' rites too; Hemadri's halves of a tithi in a kshaya lunation, for a death or a birth |
| 2 | Kane, *History of Dharmaśāstra* V.1 pp. 88–89 | the year's beginning on the adhika's first day by some, on the shuddha's by the *Mayukha*, which puts every festival in the shuddha month; the samsarpa (the adhika before a kshaya) barred like any adhika |
| 1 | the committee's prints, VS 2082 p. 15 and VS 2083 p. 17 | the Nepal Sambat new year printed on Kartika's bright first, beside the Mha puja |
| 1 | the Nepal Gazette's holiday notices, VS 2076 to 2081 | the beginning of Nepal Sambat on one day with the Mha puja or Govardhan puja, so the year keeps the nija Kartika with them (C2418) |

No printed patro for 1963–64 or 1982–83, the two kshaya years in the
SDK's tested span before 2028, was found (C2045).

## 2. The rule

Every case follows from p. 3's principle, read per rule:

- **A doubled month.** A festival of the month has another time, the nija
  month, and is kept there. The clause is `NIJA`.
- **A rule's own exception.** A rule whose source keeps it in the adhika
  month says so with its `inAdhika` (`festival-rule-language.md` §3.7):
  `FIRST` (Dashahara, p. 38),
  `ONLY` (Padmini and Parama ekadashi) or `ALSO` (a rule kept every
  month). The clause is `ADHIKA`.
- **A kshaya lunation.** It is both months (p. 3), so a festival of either
  has no other time and is kept in it on its own tithi: `KSHAYA_FIRST`
  under the lunation's own name, `KSHAYA_SECOND` under the name it passes
  over. Nothing is moved and nothing dropped. The halves of a tithi
  (Hemadri) name the month of an instant, a death or a birth
  (`shraddha.md` §2.3); no text applies them to a festival (C2042).
- **The samsarpa**, the adhika month before a kshaya one, is an adhika
  month like any other and keeps no festival a rule does not name for it
  (Kane; p. 4 bars the yajna festival in it, C2044).

So the setting where schools differ is the rule's own `inAdhika`, which a
consumer replaces by key like any field of a shipped rule
(`festival-rules.md` §4). The texts agree on the kshaya clauses, so they
take no setting. A rule is never special-cased by its name in the engine:
the Nepal Sambat new year is `FIRST` because its rule says so.

## 3. The days an answer reads

Whether a nija day's month is doubled is read from the lunation beside the
day's own: an adhika month runs new moon to new moon under both
conventions, so it comes just before the amanta nija month of its name.
A range that starts in a nija month would not see the adhika before it, so
the SDK widens the days it founds into the lunation before the first day,
and the lunation after the last, **when that lunation is an adhika month
of the name next to it** (`Almanac::beside_doubled`). One day of each is
founded to ask; the rest only when the answer is yes. The provenance's
`festival.days` convention says the days reach that far. A short range
then answers as a year does: `crates/sdk/tests/festival.rs` holds a
fortnight of BS 2085 to the year's own answer.

## 4. The answer

Every observance a month rule placed carries
`malamasa: {clause, source}`, the clause one of `NIJA`, `ADHIKA`,
`KSHAYA_FIRST`, `KSHAYA_SECOND`, and the source the texts that clause is
read from; an observance in an ordinary month carries none, so an answer
in an ordinary year keeps its bytes and its hash. A day counted from
another rule's (`FollowingRule`) inherits its base's clause. The field is
the reason a day differs from the one an ordinary year would give, and is
written to serve `almanac.compare` (SR-94) as its `MONTH_DECISION` reason.

It reaches every binding: Rust `teistro::festival::Malamasa`, the C
boundary's JSON, Node's `Malamasa`/`MalamasaClause`, Python's
`Malamasa`/`MalamasaClause`, Dart's `Malamasa`/`MalamasaClause`, Java's
`Malamasa`/`MalamasaClause`, wasm and MCP through the schema; each has a
test, and the parity runners print the clause on every observance line,
with a `festivals-malamasa` case on the test provider's doubled 2023 month.

## 5. Measured

`NEPAL_PANCHANGA` at Kathmandu under `nepal_official`:

- **BS 2085** (2028-08 to 2029-04): no rule an ordinary year keeps is
  missing. The Nepal Sambat new year falls on 18 October 2028, the adhika
  Kartika's first; the Mha puja and Govardhan on 17 November and Bhai
  tika on 18 November, the nija month's; the Pausha festivals keep their
  tithis in the kshaya lunation, the Magha bathing and the Swasthani vrata
  beginning on its full moon, 31 December 2028 (p. 4). Against
  `nepali-drik` the year now differs on 74 observances, none of them a
  dropped festival.
- **1963–64**, *Dharmasindhu*'s own example year: the lunation from 16
  December 1963 holds both sankrantis; Gita jayanti keeps 26 December
  under Margashirsha and the Magha bathing begins on 30 December, its full
  moon.
- **1982–83**: the lunation from 14 January 1983 is Pausha and Magha;
  Vasanta panchami (19 January) and Shivaratri (11 February) keep their
  tithis under Magha; Holika waits for the nija Phalguna after the adhika
  that follows (28 March).

Each is held in `crates/sdk/tests/festival.rs`, with the check that every
rule an ordinary year keeps is kept.

## 6. Cruxes

C2040 to C2045 in `01-research/feature-universe/19-verification-cruxes.md`.
