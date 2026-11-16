# Festival groups

Status: `built`, 2026-11-17. Design for the almanac app's SR-125
(its addendum 9, P1), written before the code and corrected
by it. Every number below was measured through the built call
(`crates/sdk/tests/festival_groups.rs`) under `nepali-committee` at
Kathmandu.

Some festivals are kept over several days, and each day is its own rule.
Tihar is the crows' day, the dogs' day, Lakshmi puja, Govardhan puja,
Mha puja and Bhai Tika. An app shows them as one festival: "Tihar,
7 to 11 November", with each day under it. Until now the app grouped
them in its own code, by key. A group is now **data in the pack**, and
the festival answer gives each group's **spans**.

## 1. The shape

A group is four fields (`FestivalGroup`, crux C2100):

| field | what it is |
|---|---|
| `key` | the group's key, `TIHAR`. A consumer's names and descriptions hang on it, as on a rule's key. |
| `members` | the member rules' keys, in the festival's order. Each must be a key the request carries, refused by its index otherwise (`groups[i].members[j]`). At most 32, none twice. |
| `main` | the member the group is reckoned around: Lakshmi puja for Tihar, Vijaya Dashami for Dashain. One of `members`. |
| `source` | the text and the notices that say which rules make the festival |

A pack carries its groups (`FestivalPack::groups`), and
`FestivalRequest` folds them in with the pack's rules. A request's own
group replaces a pack's of the same key, or is added after them
(`with_group`, or `groups` in the request's JSON). So a consumer that
ends Dashain at the Tika, or adds Gai Tihar to Tihar, says so as data.

Who keeps a member is the member rule's own meta. A group never states
it again. Mha puja is `NEWAR`'s, so a request filtered to the Newar
community keeps Tihar and not Dashain.
A group is answered when the filter keeps **any** of its members
(`keeps_group`); the span then lists the members it keeps.

### What the answer gives

The festival answer gains `groups`: every span that reaches the asked
range, in the order of their first days, with the groups' order on a
tie. It is left out when the request names no group, so an answer
asked without groups is the same as before. A span is:

| field | what it is |
|---|---|
| `group`, `main` | the group's key and its main member |
| `first`, `last` | the span's first and last days, inclusive |
| `members` | each member and its day, in the group's order. `day` is left out when the member falls on no day within reach of the main day (§4), or a published set withdrew it. |
| `days` | **every** day from `first` to `last`. Each lists the members that fall on it, and `gap: true` when none does. |

The gap is a marker on a day, not a missing row. An app that draws the
span draws every day and shades the gaps. One that lists members reads
`members`.

**Why spans and not a group key on each observance.** A key on each
observance would leave the app to find the first and last days, the
gaps and the occurrence each day belongs to. It would also be wrong for
a rule in two groups: Bihar's `MAHA_ASHTAMI` is in both the Durga Puja
and the Navaratri. A span is the thing the app draws. The observances
stay as they are, so nothing that reads them moves.

## 2. Nepal

`NEPAL_PANCHANGA` carries two groups.

**Dashain** (`DASHAIN`, main `VIJAYA_DASHAMI`): Ghatasthapana
(`NAVARATRA_ARAMBHA`), Phulpati, the Mahashtami, the Mahanavami, Vijaya
Dashami and Kojagrat Purnima. *Dharmasindhu* gives the Navaratra from
Ashvina's bright 1st (p. 65), the Mahanavami (p. 69), the visarjana and
the Vijaya Dashami (pp. 70–71) and the Kojagara on the full moon
(p. 74). The national panchanga prints each in VS 2082 (p. 13) and
VS 2083 (p. 15). The Ministry of Home Affairs' VS 2083 holiday notice
gives Ghatasthapana, and "दशैं" from Phulpati to the 12th.

**Kojagrat belongs** (C2101). The tika is given from the Dashami to the
full moon, and Dashain is said to end on Kojagrat Purnima. *Dharmasindhu*
treats the Navaratra and the Kojagara as two observances, and the
notice's holiday ends on the 12th. So the text does not make Kojagrat
part of the Navaratra, and the notice's holiday is not the festival.
The festival as Nepal keeps it runs to the full moon. A consumer that
ends Dashain at the holiday or the Tika regroups it (§1). Measured spans
run 13 to 15 days.

**Tihar** (`TIHAR`, main `LAKSHMI_PUJA`): Kag Tihar, Kukur Tihar,
Lakshmi puja, Govardhan puja, Mha puja and Bhai Tika (`YAMA_DWITIYA`).
*Dharmasindhu* gives Lakshmi's worship on the new moon (p. 77), the Bali
and Govardhan puja (p. 78) and the Yama dwitiya (p. 79). Nepal keeps
these days, Ashvina's dark 13th to Kartika's bright 2nd, as the
yamapanchaka (C613). The panchanga prints each in VS 2082 (pp. 14–15)
and VS 2083 (pp. 16–17). The notice gives "तिहार" from Lakshmi puja to
the day after Bhai Tika. Dhanteras falls outside the five days, and Mha
puja is the Newar member (C2102, C2103).

**BS 2083's gap.** Kartik 21 to 25, 2083 are 7 to 11 November 2026. The
new moon holds the 8th's evening and the 9th's sunrise. So Lakshmi puja
falls on the 8th, with the dogs' day, and Govardhan puja on the 10th:

```
7:KAG_TIHAR 8:KUKUR_TIHAR+LAKSHMI_PUJA 9:- 10:GOVARDHAN_PUJA+MHA_PUJA 11:YAMA_DWITIYA
```

The committee prints the cows' worship on Kartik 23 (the 9th). The pack
holds no Gai Tihar rule: *Dharmasindhu* puts the cows with the
Govardhan puja, and that day misses five of the consumer's seven
verified days (C1661, dropped). So the 9th is a gap of the span, and it
is one by the pack's rules, not a hole in the festival (C2111). A
consumer that keeps the cows' day writes the rule and adds it to the
group.

**Chhath** is not a Nepal group. The panchanga prints one day, छठपर्व,
with no Kharna and no Nahay-Khay, and the pack's `CHHATH` is that one
day (C2110).

## 3. India

| pack | groups |
|---|---|
| `INDIA_BIHAR` | `DURGA_PUJA` (Phulpati, the Mahashtami, the Mahanavami, Vijaya Dashami; main the Mahashtami); `NAVARATRI` (the Navaratra's first day, Phulpati, the Mahashtami, the Mahanavami; main the Mahashtami, C2105); `CHHATH` (Kharna, the evening arghya, the morning arghya; main `CHHATH`, C2106) |
| `INDIA_WEST_BENGAL` | `DURGA_PUJA` from the Shashthi (the bodhana) to the Dashami (C2104) |
| `INDIA_ASSAM`, `INDIA_ODISHA` | `DURGA_PUJA`, as Bihar's |
| `INDIA_UTTAR_PRADESH` | `DEEPAWALI`: Narak Chaturdashi, Lakshmi puja, Govardhan puja, the Yama dwitiya (C2102) |

The **Durga Puja** is the rite's days. In Bengal they run from the
bodhana on the Shashthi to the visarjana on the Dashami. West Bengal's
List II also calls its Chaturthi and Panchami holidays the Puja's, but
those are holidays before the rite (C2104). Elsewhere the notices list
the Saptami (the patrika day, `PHULPATI`) to the Dashami.

**Bihar's Chhath** runs from Kharna to the morning arghya. Nahay-Khay,
the day before Kharna, is kept, but no rule of the pack gives it, so it
is no member until one does (C2106). In 2026 the span is 14 to
16 November.

Uttar Pradesh carries no Navaratri group, because its pack has no rule
for the Navaratra's first day. Telangana's Dasara, Gujarat's Navaratri
and the other packs carry none yet: each needs its own notice read
first (§7).

## 4. Finding the spans

`group_spans` is given the answer's standing days: observances, fasts,
dated days, anniversaries and days a set added. A withdrawn day is not
a member's day.

- **A span for each main day.** Each day the main member falls on is one
  span. Each other member takes its day **nearest** the main day, the
  earlier on a tie, within **20 days** (`MOST_REACH_DAYS`). Dashain's
  first day is nine or ten days before Vijaya Dashami, the furthest any
  shipped group reaches. Twenty days is twice that, and still less than
  a lunation, so a member never takes another year's or another
  month's day (C2109).
- **First, last and gaps.** The span runs from the earliest day taken to
  the latest, and lists every day between. A day no member falls on is a
  gap.
- **A doubled tithi** puts a gap inside a span. In 2026 the 8th holds two
  sunrises, so 19 October lies between the Mahashtami (18th) and the
  Mahanavami (20th). Dashain runs 11 to 25 October, 15 days.
- **A lost tithi** puts two members on one day. In 2024 the 8th holds no
  sunrise, so the Mahashtami and the Mahanavami share 11 October.
  Dashain runs 3 to 16 October, 14 days, 9 of them gaps.

**A range that cuts a span** still answers the span whole. A request
for 9 November 2026 alone answers Tihar from the 7th to the 11th. The
reckoning already widens its days (`festival-rules.md` §4). When a span
or a member day left over lies within 20 days of the days reckoned, the
call reckons that side again with only the groups' members. It reckons
from 20 days past the edge up to the range's own edge, so a member on
the old edge is judged with its whole karmakala. At most two rounds
are run. The spans kept are those that reach the range.

**An adhika Kartika** splits Tihar (C2107). In 2028 (BS 2085), Lakshmi
puja falls in Ashvina's dark half, before the adhika month. Govardhan,
Mha puja and Bhai Tika fall in the nija Kartika, a month later and past
the reach. The span holds the crows' and dogs' days and Lakshmi puja,
the other three members have no day, and `is_whole` is false. The pack
has nothing that says how Nepal keeps Tihar in that year, so the answer
does not invent a span across the month.

## 5. The next span

`almanac.next` takes a `group` in place of a `kind` or a `rule`, with
the `festivals` that carry it (`next-occurrence.md`). The request is cut
to the group's members, together with what their days need: the rules
they follow, a parana's rite, and a rule that bars them
(`narrowed_to_group`). Each window answers the first span of the group
**whose last day is on or after the civil day of the instant**, so a
span under way counts (C2108). From noon on 9 November 2026, the next
Tihar is the one that began on the 7th.

The answer's `day` is the span's first day, and `keys` are the
members'. `festivals` holds the members' entries over the span, from a
reckoning over exactly those days. `group` is the span. A key that no
group of the request has is refused on `group`, as is a group asked
together with a kind or a rule.

## 6. Bindings

Every binding reads `groups` on the festival answer and `group` on the
next answer, and asks `group` in `next`: Node and wasm (`group`), Python
(`group=`), Dart (`group:`), Java (`nextGroup`), C (the JSON request of
`ts_almanac_next`) and MCP (`almanac.next`). A request's own groups are
JSON on every binding, as a request's own rules are. Each runner prints
the spans of a `grouped` case over *Dharmasindhu*'s pack, so the parity
check compares every binding.

## 7. What is left

- Groups for the India packs not listed in §3, each from the state's
  notice.
- Nahay-Khay as a rule of Bihar's pack, which would make it Chhath's
  first member.
- How Nepal keeps Tihar in an adhika Kartika (C2107).
