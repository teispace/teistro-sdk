# Nepal's day, measured

Status: `generated` by `cargo xtask nepal-day`. Do not edit:
`check-nepal-day` regenerates this page and fails on any difference.

It measures the almanac's limbs at Kathmandu against Nepal's daily
panchanga: when each tithi, nakshatra and yoga ends, which days one of
them holds both sunrises or neither (`span.sunrises`), and the sunrise.
Nepal's national panchanga committee requires its makers to compute by
the Surya Siddhanta, and its printed Moon is the text's with a bija on
its apsis (`calendars/bikram-sambat.md`, R2; C28); this page asks
whether the daily print is that text, in the text's own zodiac, and what
the modern sky would print instead (C187).

## 1. The claims

| proposed rule | verdict | measured |
|---|---|---|
| the committee's sky (`nepali-committee`): the Surya Siddhanta with its bija, in its own zodiac, at a modern sunrise: every printed tithi end within 1.5 minutes | **holds** | 0 of 333 disagree; median -0.1 min |
| the committee's sky (`nepali-committee`): the Surya Siddhanta with its bija, in its own zodiac, at a modern sunrise: every printed nakshatra end within 1.5 minutes | **holds** | 0 of 326 disagree; median -0.1 min |
| the committee's sky (`nepali-committee`): the Surya Siddhanta with its bija, in its own zodiac, at a modern sunrise: every printed yoga end within 1.5 minutes | **holds** | 0 of 331 disagree; median -0.0 min |
| the same text at its own sunrise: every printed tithi end within 1.5 minutes | **holds** | 0 of 333 disagree; median -0.1 min |
| the same text at its own sunrise: every printed nakshatra end within 1.5 minutes | **holds** | 0 of 326 disagree; median -0.1 min |
| the same text at its own sunrise: every printed yoga end within 1.5 minutes | **holds** | 0 of 331 disagree; median -0.0 min |
| the text without a bija, at its own sunrise: every printed tithi end within 1.5 minutes | falsified | 316 of 333 disagree; median -0.1 min |
| the text without a bija, at its own sunrise: every printed nakshatra end within 1.5 minutes | falsified | 303 of 326 disagree; median -0.2 min |
| the text without a bija, at its own sunrise: every printed yoga end within 1.5 minutes | falsified | 303 of 331 disagree; median +0.1 min |
| the modern sky under Lahiri (`nepali-default`): every printed tithi end within 1.5 minutes | falsified | 333 of 333 disagree; median +41.4 min |
| the modern sky under Lahiri (`nepali-default`): every printed nakshatra end within 1.5 minutes | falsified | 323 of 326 disagree; median -12.5 min |
| the modern sky under Lahiri (`nepali-default`): every printed yoga end within 1.5 minutes | falsified | 326 of 331 disagree; median -53.9 min |
| the committee's sky (`nepali-committee`): the Surya Siddhanta with its bija, in its own zodiac, at a modern sunrise: every printed sunrise within 3 minutes | **holds** | 0 of 333 disagree; median -0.3 min |
| the same text at its own sunrise: every printed sunrise within 3 minutes | falsified | 231 of 333 disagree; median +0.3 min |
| the text without a bija, at its own sunrise: every printed sunrise within 3 minutes | falsified | 231 of 333 disagree; median +0.3 min |
| the modern sky under Lahiri (`nepali-default`): every printed sunrise within 3 minutes | **holds** | 0 of 333 disagree; median +1.0 min |
| the committee's sky (`nepali-committee`): the Surya Siddhanta with its bija, in its own zodiac, at a modern sunrise: every printed tithi flag, or a reason the pass checks | **holds** | 331 of 331 agree; the 0 others each for a reason in §5 |
| the committee's sky (`nepali-committee`): the Surya Siddhanta with its bija, in its own zodiac, at a modern sunrise: every printed nakshatra flag, or a reason the pass checks | **holds** | 333 of 333 agree; the 0 others each for a reason in §5 |
| the committee's sky (`nepali-committee`): the Surya Siddhanta with its bija, in its own zodiac, at a modern sunrise: every printed yoga flag, or a reason the pass checks | **holds** | 312 of 317 agree; the 5 others each for a reason in §5 |
| the committee's sky (`nepali-committee`): the Surya Siddhanta with its bija, in its own zodiac, at a modern sunrise: no flag is decided by the sunrise alone | **holds** | 0 ends fall between the reading's sunrise and the print's |
| the same text at its own sunrise: every printed tithi flag, or a reason the pass checks | **holds** | 329 of 331 agree; the 2 others each for a reason in §5 |
| the same text at its own sunrise: every printed nakshatra flag, or a reason the pass checks | **holds** | 331 of 333 agree; the 2 others each for a reason in §5 |
| the same text at its own sunrise: every printed yoga flag, or a reason the pass checks | **holds** | 307 of 317 agree; the 10 others each for a reason in §5 |
| the same text at its own sunrise: no flag is decided by the sunrise alone | falsified | 9 ends fall between the reading's sunrise and the print's |

## 2. The records

Makalukhabar's daily "आजको पञ्चाङ्ग" posts, 333
days from 2025-04-09 to 2026-09-30, each read for its Gregorian date,
its sunrise and its tithi, nakshatra and yoga with their ends; the page
holds them in `xtask/src/nepal_day/records.txt`, a day a line. The
source prints Pushya as "तिष्य", and names the 24th yoga,
Shukla, rightly 14 times and as Shubha 10 times: 5 times before Brahma
(§6), and 5 times as a Shubha lasting day and night where the text has
Shubha then Shukla (§5).

## 3. The ends

Each printed end against the end of the same member in the reading, in
minutes, the reading's less the print's. A member the reading does not
place within a day of the printed end is unpaired.

| reading | limb | ends | median | least | most | beyond 1.5 min | unpaired |
|---|---|---|---|---|---|---|---|
| the committee's sky (`nepali-committee`): the Surya Siddhanta with its bija, in its own zodiac, at a modern sunrise | tithi | 333 | -0.1 | -1.2 | +0.9 | 0 | 0 |
| the committee's sky (`nepali-committee`): the Surya Siddhanta with its bija, in its own zodiac, at a modern sunrise | nakshatra | 326 | -0.1 | -0.9 | +0.7 | 0 | 0 |
| the committee's sky (`nepali-committee`): the Surya Siddhanta with its bija, in its own zodiac, at a modern sunrise | yoga | 331 | -0.0 | -0.9 | +0.7 | 0 | 0 |
| the same text at its own sunrise | tithi | 333 | -0.1 | -1.2 | +0.9 | 0 | 0 |
| the same text at its own sunrise | nakshatra | 326 | -0.1 | -0.9 | +0.7 | 0 | 0 |
| the same text at its own sunrise | yoga | 331 | -0.0 | -0.9 | +0.7 | 0 | 0 |
| the text without a bija, at its own sunrise | tithi | 333 | -0.1 | -20.3 | +16.8 | 316 | 0 |
| the text without a bija, at its own sunrise | nakshatra | 326 | -0.2 | -18.7 | +15.5 | 303 | 0 |
| the text without a bija, at its own sunrise | yoga | 331 | +0.1 | -17.2 | +14.5 | 303 | 0 |
| the modern sky under Lahiri (`nepali-default`) | tithi | 333 | +41.4 | -360.3 | +339.2 | 333 | 0 |
| the modern sky under Lahiri (`nepali-default`) | nakshatra | 326 | -12.5 | -388.7 | +278.3 | 323 | 0 |
| the modern sky under Lahiri (`nepali-default`) | yoga | 331 | -53.9 | -412.0 | +249.2 | 326 | 0 |

## 4. The sunrise

Each printed sunrise against the reading's, in minutes, the reading's
less the print's.

| reading | days | median | least | most |
|---|---|---|---|---|
| the committee's sky (`nepali-committee`): the Surya Siddhanta with its bija, in its own zodiac, at a modern sunrise | 333 | -0.3 | -1.3 | +1.2 |
| the same text at its own sunrise | 333 | +0.3 | -11.0 | +19.6 |
| the text without a bija, at its own sunrise | 333 | +0.3 | -11.0 | +19.6 |
| the modern sky under Lahiri (`nepali-default`) | 333 | +1.0 | -0.1 | +2.4 |

## 5. The flags

Each day's printed flag for each limb — plain, a vriddhi
("दिनरात") or a kshaya (three in a day) — against the flag
each reading whose ends are the print's gives through `span.sunrises`. A
record that contradicts itself (§6) is left out. Every disagreement is
listed with the reason the pass checked for it.

| reading | limb | days | printed vriddhi | printed kshaya | agree | disagree |
|---|---|---|---|---|---|---|
| the committee's sky (`nepali-committee`): the Surya Siddhanta with its bija, in its own zodiac, at a modern sunrise | tithi | 331 | 7 | 9 | 331 | 0 |
| the committee's sky (`nepali-committee`): the Surya Siddhanta with its bija, in its own zodiac, at a modern sunrise | nakshatra | 333 | 12 | 5 | 333 | 0 |
| the committee's sky (`nepali-committee`): the Surya Siddhanta with its bija, in its own zodiac, at a modern sunrise | yoga | 317 | 9 | 23 | 312 | 5 |
| the same text at its own sunrise | tithi | 331 | 7 | 9 | 329 | 2 |
| the same text at its own sunrise | nakshatra | 333 | 12 | 5 | 331 | 2 |
| the same text at its own sunrise | yoga | 317 | 9 | 23 | 307 | 10 |

| reading | day | limb | printed | the reading's | why |
|---|---|---|---|---|---|
| the committee's sky (`nepali-committee`): the Surya Siddhanta with its bija, in its own zodiac, at a modern sunrise | 2026-05-08 | yoga | vriddhi | plain | Shubha then Shukla, which the source prints as Shubha |
| the committee's sky (`nepali-committee`): the Surya Siddhanta with its bija, in its own zodiac, at a modern sunrise | 2026-06-03 | yoga | vriddhi | plain | Shubha then Shukla, which the source prints as Shubha |
| the committee's sky (`nepali-committee`): the Surya Siddhanta with its bija, in its own zodiac, at a modern sunrise | 2026-06-28 | yoga | vriddhi | plain | Shubha then Shukla, which the source prints as Shubha |
| the committee's sky (`nepali-committee`): the Surya Siddhanta with its bija, in its own zodiac, at a modern sunrise | 2026-07-23 | yoga | vriddhi | plain | Shubha then Shukla, which the source prints as Shubha |
| the committee's sky (`nepali-committee`): the Surya Siddhanta with its bija, in its own zodiac, at a modern sunrise | 2026-08-18 | yoga | vriddhi | plain | Shubha then Shukla, which the source prints as Shubha |
| the same text at its own sunrise | 2025-11-10 | tithi | plain | kshaya | an end between the reading's sunrise and the print's |
| the same text at its own sunrise | 2025-11-11 | tithi | kshaya | plain | an end between the reading's sunrise and the print's |
| the same text at its own sunrise | 2026-07-12 | nakshatra | kshaya | plain | an end between the reading's sunrise and the print's |
| the same text at its own sunrise | 2026-07-13 | nakshatra | plain | kshaya | an end between the reading's sunrise and the print's |
| the same text at its own sunrise | 2025-05-05 | yoga | kshaya | plain | an end between the reading's sunrise and the print's |
| the same text at its own sunrise | 2025-10-26 | yoga | vriddhi | plain | an end between the reading's sunrise and the print's |
| the same text at its own sunrise | 2025-10-27 | yoga | kshaya | plain | an end between the reading's sunrise and the print's |
| the same text at its own sunrise | 2026-05-08 | yoga | vriddhi | plain | Shubha then Shukla, which the source prints as Shubha |
| the same text at its own sunrise | 2026-05-29 | yoga | vriddhi | plain | an end between the reading's sunrise and the print's |
| the same text at its own sunrise | 2026-05-30 | yoga | plain | vriddhi | an end between the reading's sunrise and the print's |
| the same text at its own sunrise | 2026-06-03 | yoga | vriddhi | plain | Shubha then Shukla, which the source prints as Shubha |
| the same text at its own sunrise | 2026-06-28 | yoga | vriddhi | plain | Shubha then Shukla, which the source prints as Shubha |
| the same text at its own sunrise | 2026-07-23 | yoga | vriddhi | plain | Shubha then Shukla, which the source prints as Shubha |
| the same text at its own sunrise | 2026-08-18 | yoga | vriddhi | plain | Shubha then Shukla, which the source prints as Shubha |

## 6. Records that contradict themselves

A day's members follow one another round the wheel, so a record that
skips or repeats one says two things; it is left out of §3 and §5.

| day | limb | printed members |
|---|---|---|
| 2025-05-27 | tithi | 30 → 2 |
| 2025-05-28 | tithi | 2 → 1 → 3 |
| 2026-03-24 | yoga | 2 → 4 |
| 2026-03-25 | yoga | 4 → 3 → 5 |
| 2026-03-26 | yoga | 5 → 6 → 6 |
| 2026-04-24 | yoga | 8 → 10 |
| 2026-04-25 | yoga | 10 → 10 → 11 |
| 2026-05-05 | yoga | 20 → 16 |
| 2026-05-06 | yoga | 16 → 22 |
| 2026-05-09 | yoga | 23 → 25 |
| 2026-06-04 | yoga | 23 → 25 |
| 2026-07-12 | yoga | 11 → 8 |
| 2026-07-13 | yoga | 8 → 13 |
| 2026-07-16 | yoga | 15 → 17 |
| 2026-07-17 | yoga | 17 → 16 → 18 |
| 2026-07-24 | yoga | 23 → 25 |
| 2026-08-19 | yoga | 23 → 25 |
| 2026-09-13 | yoga | 23 → 25 |

## 7. What the records decide

The daily print is the Surya Siddhanta with the committee's bija,
read in the text's own zodiac: its ends are the text's to the
minute the print is written in, where the text without the bija
parts from them by up to a quarter of an hour and the modern sky by
hours. So `nepali-committee` reads the text over `SURYA_SIDDHANTA`
with `SuryaBija::NepalCommittee` and the text's ayanamsha, and a
search over a provider that defines its zodiac reads the provider's
own longitudes (`ChartZodiac::searched`) rather than shifting the
tropical ones by the catalogue member of the same name, which stands
1.6° from the text today.

The print's sunrise is not the text's: the text's carries no equation of
time (C37), and the print's is a modern one, the upper limb on the
geometric horizon (C39). A flag decided within minutes of sunrise
differs for that alone, so the committee's sky takes its sunrise from a
modern ephemeris beside the text's limbs (`SuryaSunrise::Modern`,
`UPPER_LIMB_NO_REFRACTION`), and then every printed flag agrees but
where the source names Shukla as Shubha. The committee's five star
planets, modern positions under Lahiri (C38), are not read here: a day's
limbs need only the Sun and the Moon.
