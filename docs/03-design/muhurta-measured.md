# Muhurta, measured

Status: `generated` by `cargo xtask muhurta`. Do not edit:
`check-muhurta` regenerates this page and fails on any difference.

The design this measures is `muhurta.md` (§5). Everything is read
over the built-in ephemeris at Kathmandu (27.7172° N, 85.324° E,
1400 m, UTC+5:45), under the default profile, the place both the
baseline engine and the published almanac read. The searches run
over 2026-09-01 to 2026-11-30 and cut **every** open day, so every window is
measured.

## 1. The regression

The baseline engine's marriage (`ActivityRules::baseline_marriage`)
under its own ranking (`Ranking::Baseline`). 80 days are closed and
11 judged; the judged days are cut into 2062 windows, 821 of them open
and 351 of those in a choghadiya the engine offers. 8 windows fell
inside a blackout that did not cover their day, and were left out.
The best window opens on 2026-11-25, scored 100.

| proposed rule | verdict | measured |
|---|---|---|
| a marriage search under the baseline's rules vetoes 2026-09-21, 2026-10-19 and 2026-11-05 | **holds** | 0 of 3 disagree |
| nothing is open before Devuthani Ekadashi, 2026-11-20 | **holds** | 0 of 80 disagree |
| every day closed is closed by Chaturmas, and says so | **holds** | 0 of 80 disagree |
| 2026-11-25 is kept | **holds** | 0 of 1 disagree |
| an open window is scored exactly when the engine would offer it (Amrita, Shubha or Labha) | **holds** | 0 of 821 disagree |

## 2. The asta against the published almanac

The Nepal Panchanga Nirnayak Samiti's BS 2083 windows, which an
Indian almanac published independently: Guru asta Asar 32 to
Shrawan 24, Shukra asta Ashwin 28 to Kartik 11. Each is set
against the SDK's asta at Kathmandu, from the last sighting's
evening or morning to the next first one, by local date. A veto
a day wider than the almanac blocks a day nobody would use; one a
day narrower offers a day the country treats as closed, so
**holding the published window whole** is the property claimed. A
negative count is a window that begins later or ends earlier than
the almanac's.

| criterion | asta | found | published | begins earlier by | ends later by |
|---|---|---|---|---:|---:|
| Surya Siddhanta | Guru | 2026-07-13 to 2026-08-12 | 2026-07-16 to 2026-08-09 | 3 days | 3 days |
| Surya Siddhanta | Shukra | 2026-10-10 to 2026-10-29 | 2026-10-14 to 2026-10-28 | 4 days | 1 day |
| the combustion orbs | Guru | 2026-07-13 to 2026-08-14 | 2026-07-16 to 2026-08-09 | 3 days | 5 days |
| the combustion orbs | Shukra | 2026-10-18 to 2026-10-30 | 2026-10-14 to 2026-10-28 | -4 days | 2 days |
| Ptolemy | Guru | 2026-07-05 to 2026-08-17 | 2026-07-16 to 2026-08-09 | 11 days | 8 days |
| Ptolemy | Shukra | 2026-10-12 to 2026-10-29 | 2026-10-14 to 2026-10-28 | 2 days | 1 day |

| proposed rule | verdict | measured |
|---|---|---|
| under Surya Siddhanta, each asta holds the published window whole | **holds** | 0 of 2 disagree |
| under the combustion orbs, each asta holds the published window whole | falsified | 1 of 2 disagree |
| under Ptolemy, each asta holds the published window whole | **holds** | 0 of 2 disagree |

## 3. A window is constant

A window is cut wherever a clause can change, so what is judged at
its middle holds over all of it. Each window is read a second
inside either end (a quarter of the way in when it is shorter than
four): the nine grahas' signs, the lagna's and the Moon's navamsas,
and every instant clause the rules read. Windows are as short as the
sky makes them — two cuts a second apart are two real changes — and
the engine, which offers nothing under six minutes, would not offer
the shortest. Neighbours **judged alike** meet at a cut no clause
the rules read changed at — a hora's edge, a graha's ingress into a
sign nothing weighs — so the answer holds more windows than
judgements; they are counted rather than merged, because which cuts
matter is the rules' to say.

| search | windows | unsteady | clauses over part of a window | neighbours judged alike | shortest | under six minutes |
|---|---:|---:|---:|---:|---:|---:|
| the baseline's | 2062 | 0 | 0 | 706 | 0.79 s | 855 |
| Raman's | 13 411 | 0 | 0 | 4516 | 0.01 s | 5485 |

| proposed rule | verdict | measured |
|---|---|---|
| every window of the baseline's search reads the same sky a moment inside either end | **holds** | 0 of 2062 disagree |
| every clause of the baseline's search's windows covers its window whole | **holds** | 0 of 10115 disagree |
| every window of Raman's search reads the same sky a moment inside either end | **holds** | 0 of 13411 disagree |
| every clause of Raman's search's windows covers its window whole | **holds** | 0 of 121022 disagree |

## 4. Where the engine's sampling parts from the SDK

The engine keeps or drops a whole day by the Moon's star **at
sunrise**; the SDK bars the windows in a star the rite does not
take and keeps the rest. A day the engine keeps the SDK keeps too,
since the star at sunrise runs in the day; the parting is one way.
Of the 11 days the baseline search judged, the engine drops 3 days that the
SDK keeps for an allowed star rising after sunrise. And 149 open
windows the SDK scores are shorter than the six minutes under which
the engine drops a fragment.

## 5. The panchaka remainder against the star (C159)

Raman makes a panchaka's kind the remainder by nine of the tithi,
vara, star and lagna numbers; the baseline engine makes it one kind
per star of the last five. Of Raman's search's windows, 1573 lie in a
panchaka star; the remainder names a panchaka in 851 of them, and
the star's kind in 167. The remainder is the SDK's reading and this
is how far the engine's would part from it.

C158, where panchaka begins, is not counted: it is closed at rank 1
(Dhanishtha's latter half), and the rival start parts from it at
every panchaka by construction, so a count would be the number of
lunar months.

## 6. What ships

The searches above are the façade's (`sdk.almanac().muhurta`),
read back through the muhurta crate wired by hand, because the
re-reads need its sources. The façade takes the chart zodiac at the
range's middle and names that instant in its provenance, and the
hand-wired sources take the instant it names, so the two are one
search and are held to one answer.

A consumer electing a time shows the almanac beside the windows.
`muhurta_with_days` founds the range once and serves the search
its days. Counted in calls that reach the provider behind a fresh
context's cache, over Raman's search:

| asked | provider calls |
|---|---:|
| the search alone | 116 284 |
| the almanac alone | 5718 |
| the two apart | 122 002 |
| the two together | 117 329 |

Together they are spared 4673 calls, 3.8% of the two apart: the
almanac beside the search adds 0.9% to the search's own.

| proposed rule | verdict | measured |
|---|---|---|
| the façade answers the baseline's search as the crate wired by hand does | **holds** | 0 of 1 disagree |
| asked beside its days, Raman's search answers as it does alone | **holds** | 0 of 1 disagree |

