# Festival traditions: whose reading of a festival an answer keeps

Status: `built`, 2026-11-17 (SR-116). Written from the shipped packs and
their sources before any code; §6 says what the building settled.

A family follows one tradition. Where the texts give a festival two
days, one for the Smartas and one for the Vaishnavas, the SDK's packs
ship both readings as two rules, and today an app has to know each pair
to show the family one day. This page decides the setting that does it
for the app: `festivals.observer`.

## 1. What the packs ship today

A *reading* here is one tradition's rule for a festival another rule of
the same request also reads. Every shipped pair, read off the packs
(`crates/sdk/src/festival_request.rs`, `crates/panchanga/src/festival/`):

| festival | reading | key | pack | source |
|---|---|---|---|---|
| Ekadashi | Smarta householder | `EKADASHI_SMARTA` | `DHARMASINDHU`, `NEPAL`, `NEPAL_PANCHANGA` | *Dharmasindhu* pp. 11–12, vedha at sunrise (`festival-rules.md` §8.3) |
| Ekadashi | Smarta renunciant | `EKADASHI_SMARTA_RENUNCIANT` | the same | the same pages, the renunciant's column |
| Ekadashi | Vaishnava | `EKADASHI_VAISHNAVA` | the same | the same pages, vedha at arunodaya |
| Ekadashi | Gaudiya | `EKADASHI_GAUDIYA` | `GAUDIYA` | *Hari Bhakti Vilasa* 12–13: the Vaishnava's vedha and table and the eight mahadvadashis (§8.9) |
| Janmashtami | Smarta | `JANMASHTAMI` | `DHARMASINDHU`, `NEPAL` | *Dharmasindhu* pp. 49–50, nishitha (C175) |
| Janmashtami | Vaishnava | `JANMASHTAMI_VAISHNAVA` | `GAUDIYA` | *Hari Bhakti Vilasa* 15.358–374, sunrise (§8.10) |
| Rama Navami | Smarta | `RAMA_NAVAMI` | `DHARMASINDHU`, `NEPAL` | *Dharmasindhu* p. 33, madhyahna |
| Rama Navami | Vaishnava | `RAMA_NAVAMI_VAISHNAVA` | `GAUDIYA` | *Hari Bhakti Vilasa* 14.249–253 |
| Nrisimha Chaturdashi | Smarta | `NARASIMHA_JAYANTI` | `NEPAL_PANCHANGA` | *Dharmasindhu* p. 37, sunset |
| Nrisimha Chaturdashi | Vaishnava | `NRISIMHA_CHATURDASHI` | `GAUDIYA` | *Hari Bhakti Vilasa* 14.455–461 |

Only the Ekadashi's readings sit in one pack. The other three meet when a
request names a Smarta pack beside `GAUDIYA` (`["NEPAL", "GAUDIYA"]`),
which `festival-rules.md` §8.10 already allows: "a request may hold both".
`NextKind::EKADASHI` reads the three *Dharmasindhu* observers and answers
the earliest when they part (`next-occurrence.md` §2).

Three other kinds of difference are **not** readings of this kind:

- **Regional days.** Holi in the hills and in the Terai (`HOLI_HILLS`,
  `HOLI_TERAI`) are two days kept in two places, both printed; Mithila's
  Jitiya replaces Nepal's when a request names `MITHILA`; each Indian
  state's list is its own pack. The place chooses these, through the pack
  and `only.regions` (`festival-metadata.md` §4), not the family's
  tradition (C2144).
- **The Jain gacchas.** Paryushana and Samvatsari fall on the bright 4th
  for the Tapa and Kharatara gacchas and on the 5th for the Achala gaccha,
  the Sthanakvasis and the Terapanthis, and `JAIN` ships both, each naming
  its gacchas in `communities`. The split rests on a rank-4 relay (C1112),
  and a consumer already keeps one gaccha's days with `only.communities`.
  Not on this setting yet (C2145).
- **A government's day.** `INDIA_CENTRAL`'s `JANMASHTAMI` is the
  Vaishnava day because the notice prints it (C851). A holiday list is
  one day per festival, whoever's reading it is, and carries no reading.

## 2. The decision: one observer per context (C2140)

Two shapes were weighed:

- **A map** from festival to reading (`{"EKADASHI": "VAISHNAVA",
  "JANMASHTAMI": "SMARTA"}`). It says everything, but the app must know
  each festival that has readings, which is what the request asks to be
  spared, and a pack that gains a pair later is silently unset in every
  stored map.
- **One value**: the tradition the family follows, applied to every
  festival that has readings. It is what the request describes ("a family
  follows one tradition"), and a pack that gains a pair is read under it
  at once.

One value, then: `festivals.observer`, a closed set.

| member | keeps |
|---|---|
| `EVERY` (default) | every reading the request carries, as before this setting |
| `SMARTA` | the Smarta householder's |
| `SMARTA_RENUNCIANT` | the renunciant's, else the householder's |
| `VAISHNAVA` | the Vaishnava's |
| `GAUDIYA` | the Gaudiya's, else the Vaishnava's |

The name is `observer`, not the request's `traditions`: a festival rule's
`tradition` is already its religion (`FestivalMeta.tradition`, the
catalogue's `Tradition`, `only.traditions`), and one word for two things
would let `{"only": {"traditions": ["HINDU"]}}` and the setting be
confused. *Dharmasindhu*'s Ekadashi is written as who observes it, and
the SDK has called the three rules "the three observers" since §8.3. A
consumer who wants two readings at once keeps `EVERY`, or asks the
observance finder, which compares named traditions side by side
(`observance-finder.md`).

## 3. Which rule is whose: the rule says so (C2141)

A rule says which festival it is a reading of, and whose, in its own
values: `FestivalMeta.variant`, `{"festival": "EKADASHI", "observer":
"SMARTA"}`. It is not read off the key, because a key does not say it:
`INDIA_CENTRAL`'s `JANMASHTAMI` is the Vaishnava day under the Smarta
rule's key. The shipped packs set it from the table in §1 (the texts'
own rules: `FestivalMeta::of_text` and `FestivalMeta::nepal`, and the
`EKADASHI` kind's rules); a government's list sets none. A consumer's
own rule may carry one, and is chosen among the others like a shipped
one; `EVERY` is no observer a rule can be the reading of, and a request
naming it is refused at `variant.observer`.

## 4. How the answer is chosen

Under any observer but `EVERY`, for each festival the request carries
two or more readings of:

1. the readings whose observer is the setting's are kept;
2. else, where the setting names a tradition it follows (§5), that
   tradition's readings are kept;
3. else every reading stands.

A festival the request carries in one reading stands whatever the
observer (C2143): the setting chooses between readings and never hides a
festival. A Smarta family that asks for `GAUDIYA` alone is shown
`GAUDIYA`'s Janmashtami, because it asked for it; and the observance
finder, which narrows each tradition's request to its one rule, answers
as before.

The choice is made on the answer, as `only` is (`festival-metadata.md`
§4): every rule is still reckoned, so a day counted from a reading left
out stands, and a published set's `SET` on it still checks. It is read
wherever a festival request is reckoned: `almanac.festivals`, a range's
festivals, the month grid, the feed and digest, and `almanac.next`, whose
`EKADASHI` kind then answers the observer's day rather than the earliest
of three.

## 5. The two traditions that follow another (C2142)

- **The renunciant follows the householder** where no renunciant
  reading is shipped. *Dharmasindhu* gives the renunciant a column of his
  own only in the Ekadashi table, and even there, where a row does not
  name him (S5 and S7, §8.1), he keeps the householder's day (§8.3). So
  outside the Ekadashi a renunciant keeps the Smarta day.
- **The Gaudiya follows the Vaishnava.** *Hari Bhakti Vilasa*'s Ekadashi
  is the Vaishnava's vedha and table with the mahadvadashis added
  (12.359's commentary, §8.9), and its other festivals are the
  Vaishnava's rule of a viddha tithi (15.358, 12.352's commentary,
  §8.10). Where a request carries no Gaudiya reading, as `DHARMASINDHU`'s
  Ekadashi does not, the Gaudiya keeps the Vaishnava's.

No other member follows one: a Smarta reading is not a Vaishnava's, nor
the other way.

## 6. Defaults, profiles and what moves

`EVERY` is the root's value and every shipped profile keeps it (C2147),
so no shipped profile's answer moves. Nepal's committee prints both
observers' days where they part ("स्मार्तहरूको" on the first, "वैष्णवहरूको"
on the second, VS 2082 p. 6, VS 2083 p. 8; C688), so `nepali-committee`
keeping every reading is the committee's own practice, and no other
profile is a sect's.

What moves when a consumer sets it: which of a festival's readings the
answer lists. No day moves, no rule is reckoned otherwise, and no reading
is added. Every settings hash moves once, the document gaining the group
`festivals`, and the request hash of a pack whose rules gained `variant`
(`DHARMASINDHU`, `NEPAL`, `NEPAL_PANCHANGA`, `GAUDIYA`, and the `EKADASHI`
kind's request).

**Acceptance.** *Dharmasindhu*'s S8 row (`festival-rules.md` §8.1) on a
sky that happened: Bhadrapada's bright 11th of 2023 at New Delhi began
after the 25th's sunrise and ended before the 26th's, pierced at sunrise
and holding no sunrise. The almanac gives the Smarta fast the 25th and
the renunciant's and the Vaishnava's the 26th, as the text does. Under
`SMARTA` the answer is one fast on the 25th, under `VAISHNAVA` and
`GAUDIYA` one on the 26th, under `SMARTA_RENUNCIANT` one on the 26th, and
under `EVERY` the three as before (`crates/sdk/tests/festival_observer.rs`).

## 7. Not built

- **Nepal's named ekadashis** (`KAMADA_EKADASHI` and the rest of
  `NEPAL_PANCHANGA`'s 26) are the Smartas' sunrise 11th, as the print
  names them (C688). The print also names the Vaishnavas' day where the
  observers part, but the pack ships no Vaishnava reading of each name, so
  under `VAISHNAVA` the named day stays the print's and `EKADASHI_VAISHNAVA`
  gives the Vaishnava's fast (C2146, open).
- **The Jain gacchas** as members (C2145, open): a gaccha's own calendar
  in hand would settle the split and let them join; they would be new
  members, an append.
- **A per-festival map** (C2140): if a consumer needs one, it is a second
  knob laid over this one, an append.
