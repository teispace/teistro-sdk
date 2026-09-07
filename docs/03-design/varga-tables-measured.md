# The divisional charts' tables, measured

Status: `generated` by `cargo xtask vargas` over the conformance
corpus's `vargas` sections, 2026-09-07. Do not edit: `check-vargas`
regenerates this page and fails on any difference. The design it tests
is [`varga-kernel.md`](varga-kernel.md).

A divisional chart's answer is a function of one sidereal longitude, and
the corpus records both the longitude and the answer. So unlike almost
everything else the SDK measures, this is not a comparison against an
implementation's numbers — it is a **derivation of the engine's own
tables**, and it either reproduces them or it does not.

93 fixtures carry divisional charts: 55 recorded days under the
engine's default profile and 38 of the variants, which is the same
day under another ayanamsha, another node, another centre or the
tropical zodiac. That is 19 530 placements over 21 charts, ten bodies at
a time, and the lagna is one of them.

## 1. One evaluator, twenty-one rows

The design proposes a single form: sort the sign into a group, take the
part of the sign the longitude falls in, and either step through the
signs from somewhere or read the answer off a list. Written out,
`(a·sign + step·part + offset) mod 12`, where `a` is nought, one or
the number of divisions.

| proposed rule | verdict | measured |
|---|---|---|
| `D1` is 1 part, by all: the sign itself, unchanged | **holds** | 0 of 930 disagree |
| `D2` is 2 parts, by parity: [LEO, CAN]; [CAN, LEO] | **holds** | 0 of 930 disagree |
| `D3` is 3 parts, by all: 4·part + 0 from the sign itself | **holds** | 0 of 930 disagree |
| `D4` is 4 parts, by all: 3·part + 0 from the sign itself | **holds** | 0 of 930 disagree |
| `D5` is 5 parts, by parity: [ARI, AQU, SAG, GEM, LIB]; [TAU, VIR, PIS, CAP, SCO] | **holds** | 0 of 930 disagree |
| `D6` is 6 parts, by parity: 1·part + 0 from a fixed sign; 1·part + 6 from a fixed sign | **holds** | 0 of 930 disagree |
| `D7` is 7 parts, by parity: 1·part + 0 from the sign itself; 1·part + 6 from the sign itself | **holds** | 0 of 930 disagree |
| `D8` is 8 parts, by modality: 1·part + 0 from a fixed sign; 1·part + 4 from a fixed sign; 1·part + 8 from a fixed sign | **holds** | 0 of 930 disagree |
| `D9` is 9 parts, by all: 1·part + 0 from continuously (parivritti) | **holds** | 0 of 930 disagree |
| `D10` is 10 parts, by parity: 1·part + 0 from the sign itself; 1·part + 8 from the sign itself | **holds** | 0 of 930 disagree |
| `D11` is 11 parts, by parity: 1·part + 0 from the sign itself; 1·part + 10 from the sign itself | **holds** | 0 of 930 disagree |
| `D12` is 12 parts, by all: 1·part + 0 from the sign itself | **holds** | 0 of 930 disagree |
| `D16` is 16 parts, by modality: 1·part + 0 from a fixed sign; 1·part + 4 from a fixed sign; 1·part + 8 from a fixed sign | **holds** | 0 of 930 disagree |
| `D20` is 20 parts, by modality: 1·part + 0 from a fixed sign; 1·part + 8 from a fixed sign; 1·part + 4 from a fixed sign | **holds** | 0 of 930 disagree |
| `D24` is 24 parts, by parity: 1·part + 4 from a fixed sign; 1·part + 3 from a fixed sign | **holds** | 0 of 930 disagree |
| `D27` is 27 parts, by element: 1·part + 0 from a fixed sign; 1·part + 3 from a fixed sign; 1·part + 6 from a fixed sign; 1·part + 9 from a fixed sign | **holds** | 0 of 930 disagree |
| `D30` is 30 parts, by parity: [ARI, AQU, SAG, GEM, LIB] over spans of 5/5/8/7/5°; [TAU, VIR, PIS, CAP, SCO] over spans of 5/7/8/5/5° | **holds** | 0 of 930 disagree |
| `D40` is 40 parts, by parity: 1·part + 0 from a fixed sign; 1·part + 6 from a fixed sign | **holds** | 0 of 930 disagree |
| `D45` is 45 parts, by modality: 1·part + 0 from a fixed sign; 1·part + 4 from a fixed sign; 1·part + 8 from a fixed sign | **holds** | 0 of 930 disagree |
| `D60` is 60 parts, by all: 1·part + 0 from the sign itself | **holds** | 0 of 930 disagree |
| `D150` is 150 parts, by all: 1·part + 0 from the sign itself | **holds** | 0 of 930 disagree |

0 of 19 530 placements disagree, and 0 cells are pinned
twice with different answers — which would mean the *part* rule was
wrong, not the map. 18 of the 21 charts step through the signs;
the rest list them, and §2 gives their lists.

## 2. Three charts list their signs instead of stepping through them

A rule that steps says "the next part is so many signs on". Three charts
do not step at all — their signs are a fixed sequence — and one of
the three is the reason the design keeps an explicit form at all.

| chart | group | the signs, in order |
|---|---|---|
| D2 | odd signs | Leo, Cancer |
| D2 | even signs | Cancer, Leo |
| D5 | odd signs | Aries, Aquarius, Sagittarius, Gemini, Libra |
| D5 | even signs | Taurus, Virgo, Pisces, Capricorn, Scorpio |
| D30 | odd signs | Aries, Aquarius, Sagittarius, Gemini, Libra |
| D30 | even signs | Taurus, Virgo, Pisces, Capricorn, Scorpio |

The pairing is the finding. D5 and D30 send their five parts to the
**same five signs**, in the same order, and differ only in how wide the
parts are: D5's are six degrees each and D30's are not (§3). So the two
are one table under two span rules, which is why the design keeps spans
and the map as separate fields of a group rather than folding either
into the other.

## 3. D30's parts are not equal, and the corpus draws them

Every other chart cuts a sign into equal parts. D30 does not, and the
boundaries below are read off the corpus rather than assumed: each is
bracketed by the last recorded placement that gave the sign before it
and the first that gave the sign after, and a boundary is settled when
exactly one whole degree lies in that bracket.

| group | boundary | bracket | signs either side |
|---|---|---|---|
| odd | 5° | 4.625° to 5.103° | Aries then Aquarius |
| odd | 10° | 9.936° to 10.145° | Aquarius then Sagittarius |
| odd | 18° | 17.771° to 18.281° | Sagittarius then Gemini |
| odd | 25° | 24.854° to 25.007° | Gemini then Libra |
| even | 5° | 4.843° to 5.070° | Taurus then Virgo |
| even | 12° | 11.884° to 12.184° | Virgo then Pisces |
| even | 20° | 19.856° to 20.080° | Pisces then Capricorn |
| even | 25° | 24.916° to 25.006° | Capricorn then Scorpio |

| proposed rule | verdict | measured |
|---|---|---|
| D30's spans are whole degrees | **holds** | odd 5, 5, 8, 7, 5; even 5, 7, 8, 5, 5 |
| they sum to thirty | **holds** | both groups |
| the even group's spans are the odd group's reversed | **holds** | exactly |

So the spans are a per-group field and not a per-chart one: the two
groups of one chart have different ones. The design's schema had `spans`
beside `map` at the top of a `VargaDef`, which cannot say that, and the
kernel moves it into the group.

Every span is a whole number of degrees, so a part index is an
integer comparison against a cumulative table and never a division
(`03-design/exact-arithmetic.md`, ADR-0016). The engine computes it
as `floor(deg / (30/N))` in floating point, and none of 30/7, 30/11,
30/27 or 0.2 is representable, so a body exactly on a part boundary
can land either side of it depending on the platform.

## 4. What the corpus pins, and what only a rule can carry

A table has twelve signs by its parts. The corpus pins a cell only when
some recorded body actually fell in it, and the larger the chart the
fewer of them it reaches — nine hundred and thirty bodies cannot fill
eighteen hundred cells.

| chart | cells | pinned | share |
|---|---|---|---|
| D1 | 12 | 12 | 100% |
| D2 | 24 | 24 | 100% |
| D3 | 36 | 36 | 100% |
| D4 | 48 | 48 | 100% |
| D5 | 60 | 60 | 100% |
| D6 | 72 | 72 | 100% |
| D7 | 84 | 84 | 100% |
| D8 | 96 | 96 | 100% |
| D9 | 108 | 108 | 100% |
| D10 | 120 | 119 | 99% |
| D11 | 132 | 131 | 99% |
| D12 | 144 | 141 | 98% |
| D16 | 192 | 183 | 95% |
| D20 | 240 | 217 | 90% |
| D24 | 288 | 250 | 87% |
| D27 | 324 | 273 | 84% |
| D30 | 60 | 60 | 100% |
| D40 | 480 | 344 | 72% |
| D45 | 540 | 371 | 69% |
| D60 | 720 | 417 | 58% |
| D150 | 1800 | 556 | 31% |

The corpus pins every cell of D1, D2, D3, D4, D5, D6, D7, D8, D9, D30,
and a fraction of the largest charts'. **This is the argument for
shipping rules rather than tables.** A rule answers a cell no recorded
body ever fell in; a table derived from the corpus would have holes in
exactly the charts where a hole is hardest to notice.

It is also why the kernel's own tests generate all twelve signs by all N
parts for every chart and check the rule against its materialised table,
rather than resting on these rows.

## 5. Vargottama is the grahas' only

A body is vargottama when its navamsha sign is the sign it already stood
in — the corpus records the flag beside every body, so the definition
can be checked rather than assumed.

| proposed rule | verdict | measured |
|---|---|---|
| a body is vargottama when its navamsha sign is its rashi sign | falsified | 2 of 930 disagree |
| the same, over the grahas alone | **holds** | 0 of 837 disagree |
| the lagna is ever marked vargottama | falsified | 0 of 93, where 2 qualify |

It holds for every graha and fails for the lagna, and it fails in one
direction: on 2 of the 93 recorded lagnas the two signs *are* the same
and the engine marks it false anyway. A vargottama lagna is a standard
term, so this is the engine restricting a general property to the grahas
rather than a different definition of it. The SDK computes it for
whatever it is asked about, and the difference joins the
deliberate-difference registry.

## 6. What this decides

1. **There is one evaluator.** 19 530 placements over 21 charts
   and two zodiacs, and not one of them needs a rule of its own. The
   design's central claim survives the strongest test the corpus can
   put to it.
2. **A group carries its own spans.** D30's two groups have
   different ones, so `spans` belongs beside `map` inside a group
   and not above both — a correction to the design's schema.
3. **Not every chart steps.** 3 of them list their signs
   instead, and two of the three share one list under different
   spans. There is no second family, only two ways to fill one
   table.
4. **Ship rules, not tables.** The corpus cannot pin the larger
   charts' cells, and a rule answers the cells it never reached.
5. **Vargottama is a property of a body, not of a graha.** The
   engine restricts it; the SDK does not, and says so.
