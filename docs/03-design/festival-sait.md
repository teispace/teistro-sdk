# Festival saits: the rite's time on a festival day

Status: `built`, 2026-10-26. Written from the sources and from the
committee's printed times before any code (2026-10-25); §5 says what the
building corrected. Not the muhurta days of `saait-measured.md`, which
are the days a rite may be done in a year; a sait here is the time on a
festival's own day.

A festival answer says which day. Many festivals also have a **sait**,
the time a rite is done: the kalash set at Ghatasthapana, the tika at
Vijaya Dashami, Bhai Tika's tika, the Rakshabandhan thread. Almanacs
print these, and an app needs to compute them for any year and any
place. This page decides how.

## 1. The finding

The committee's printed saits are **not** the tithi's rite window
(karmakala). They are **lagna elections inside that window**. Of 15
printed times read against the sky at Kathmandu, 14 fall in a navamsa
of a dual sign (dvisvabhava: Mithuna, Kanya, Dhanu, Mina), and the
fifteenth in Dhanu lagna, also dual. When the day forces midday, the
committee takes Dhanu lagna inside Abhijit. Five printed times fall
within a minute of the chosen navamsa's midpoint. For Bhai Tika 2025
the committee named the parts itself: Dhanu lagna, Kanya navamsa, the
chara choghadiya, Abhijit and the vijaya muhurta.

No rank-1 text was found for the dual-sign preference. The nearest is
Raman's *Muhurtha* (rank 2), ch. XVI, which allows a common sign for a
female deity's installation. So the window is the text's, and the
lagna preference is a **reading** (crux C745), shipped as an option
and on by default in the Nepal packs.

| Rite | Year | Printed | The sky there (modern, Lahiri) |
|---|---|---|---|
| Ghatasthapana | 2023 | 11:29 | Chitra all day; Abhijit ∩ Dhanu lagna, Mithuna navamsa 11:26–11:40 |
| Ghatasthapana | 2024 | 9:16 | Mithuna navamsa 9:07–9:23, inside the first 10 ghatis |
| Ghatasthapana | 2025 | 9:13 | Mina navamsa 9:05–9:21 (mid 9:13); an earlier one lay in Rahu kaal |
| Ghatasthapana | 2026 | 11:47 | Chitra and Vaidhriti all day; Abhijit ∩ Mithuna navamsa 11:41–11:55 |
| Phulpati | 2024 | 7:11 | Dhanu navamsa 7:07–7:23 |
| Vijaya Dashami tika | 2023–2026 | 11:02, 11:36, 11:53, 10:26 | a dual navamsa or Dhanu lagna each time |
| Bhai Tika | 2025 | 11:39 | Abhijit ∩ Dhanu lagna ∩ Kanya navamsa |
| Rakshabandhan | 2024 | after 13:30 | bhadra ends 13:48 on the modern sky; aparahna from 13:24 |

The comparisons read the modern sky; the committee reads the Sun and
the Moon on the Surya Siddhanta's, which can move a tithi's end by tens
of minutes. The lagna and navamsa readings agree within about four
minutes.

## 2. The rules

DS is the *Dharmasindhu* (Nirnaya Sagar), NS the *Nirnayasindhu*
(Nirnaya Sagar, 1926), each cited by its own page.

**Ghatasthapana**, Ashvin Shukla Pratipada.
- The day (DS p. 65): the pratipada lasting three muhurtas after
  sunrise; failing that two, and some accept one. One lasting less than
  a muhurta after sunrise, or not touching it, gives the earlier,
  Amavasya-joined day. NS pp. 120–121 says the same, from the Devi
  Purana and the Rudrayamala.
- The window (DS p. 65; NS p. 124): within ten ghatis of sunrise; if
  that cannot be had, at midday in Abhijit; never in the afternoon,
  never at night.
- Avoided inside it: the pratipada's first 16 nadis (NS p. 120 gives
  12 as a variant), Chitra, Vaidhriti. DS p. 65 keeps these only where
  they fit inside the window: the forenoon and the pratipada are never
  given up for them.
- Where Chitra or Vaidhriti holds, the variants are (NS pp. 123–124):
  - Katyayana: drop only their first quarters;
  - Bhavishya: if they fill the pratipada, drop three quarters;
  - the Devi Purana: at their end;
  - the Rudrayamala: at midday in Abhijit.

  The default is DS's.

**Phulpati**, Ashvin Shukla Saptami (NS pp. 127–128): a Mula-joined
Saptami, else the Saptami touching sunrise; the puja in the forenoon
(the first six day muhurtas). The committee printed a time in 2024 and
said none was needed in 2025: optional.

**Vijaya Dashami.**
- The day (DS p. 71; NS pp. 140–141): the Dashami holding aparahna.
  Shravana decides only between two otherwise equal days (DS); NS lets
  it decide only when the later day has it in aparahna.
- The vijaya muhurta (DS p. 72): just after dusk, when a few stars show,
  or the 11th day muhurta.
- The tika the committee prints is a separate election: Abhijit or the
  forenoon ∩ a dual navamsa, Dhanu lagna preferred.

**Laxmi Puja**, Kartik Amavasya. DS p. 77 takes the Amavasya holding
pradosha, the later day when both do. NS pp. 148–149 (the Divodasiya)
makes pradosha and midnight together the principal case. In 2025 the
committee chose the day holding both (20 October) over the later one
holding 42 minutes of pradosha: NS's rule. **Default NS.** No rank-1
text fixes the Vrishabha or Simha lagna; it ships as an option.

**Bhai Tika**, Kartik Shukla Dvitiya. The day (DS p. 79): the earlier
only if it alone holds aparahna, else the later. The sait the committee
prints is Abhijit ∩ Dhanu lagna ∩ a dual navamsa, which is Hemadri's
madhyahna (NS p. 150) rather than aparahna.

**Rakshabandhan and Janai Purnima** (DS pp. 47–49): a Purnima free of
bhadra lasting more than three muhurtas after sunrise; the thread in
aparahna or pradosha, never in bhadra. Under three muhurtas, the earlier
day's pradosha without bhadra.

**Holika dahan** (DS pp. 94–95): a Purnima holding pradosha, free of
bhadra, never by day. Failing that, in order:
1. the later day's pratipada pradosha, when the Purnima lasts three and
   a half yamas and the pratipada grows;
2. after bhadra ends, if it ends before nishitha;
3. in bhadra's puchha;
4. in bhadra, avoiding its mukha.

Puchha is the last 3 ghatis of bhadra's third quarter, and mukha the
first 5 of its fourth. The quarters scale with the tithi; the ghatis
do not.

**Shivaratri** (DS p. 90): the Chaturdashi holding nishitha, the 8th
night muhurta. The later day when only it does, or when neither does.
When both do, the later (Madhava, NS); Kaustubha takes the earlier.

**Haritalika Teej** (DS p. 54): the later Tritiya even if it lasts a
muhurta or less. This is a fast's day, with no rite window.

## 3. What is built from what

The SDK already has:
- the day's fifths;
- `PRADOSHA` and `NISHITHA`;
- the day and night muhurtas (Abhijit is the 8th);
- the kaalas, Rahu kaal among them;
- the choghadiya;
- the lagna and navamsa cuts of `window.rs`;
- the tithi, nakshatra and yoga spans;
- vishti, which is bhadra, in the catalogue.

New pieces:
1. a window of **N ghatis from sunrise**;
2. a **tithi-relative exclusion**: the first N nadis of a tithi;
3. **nakshatra and yoga spans as exclusions**, whole or by quarters;
4. **bhadra with its mukha and puchha**;
5. a **sign-class filter** on the lagna or navamsa (chara, sthira,
   dvisvabhava), and a **choice rule** naming one instant: the midpoint
   of a whole navamsa inside the window. Which navamsa, when several
   fit, was open: "the latest whole one before the window ends" fits
   2024 and 2025 but rests on two years, so it was a crux to measure,
   not a rule (C746, measured in §5);
6. a **post-dusk window** for the vijaya muhurta, from a twilight.

A sait is a field of the festival day's answer, not a separate request:
the day, its window (the text's), and the elected instant with each
part it was chosen by, so an app can show "Dhanu lagna, Kanya navamsa,
Abhijit" as the committee does.

## 4. The gate

Every printed time in §1, recomputed on the committee's own sky, must
fall in the elected navamsa. The elected midpoint's distance from the
printed minute is measured and published, not asserted, until more
years are in hand. `festival-measured.md` §8 is that gate: it holds the
six printed times shipped rules cover on both skies, and measures each
reading below against the one it was chosen over.

## 5. What the building found

- **The rising follows the lagna table.** The committee's printed times
  fall in navamsas of the sphere's ascendant in Lahiri, not of its own
  Surya Siddhanta chart: over that, 0 of the 6 lie in a candidate, as its
  navamsas run about one navamsa (13–17 minutes) late. Its profile already
  says the same of its printed lagna table (`LagnaTable::Sphere`, C680),
  so a sait reads the ascendant `Founder::lagna_table_ascendant` gives,
  the table's own, and the two cannot part.
- **The lagna holds no Sun (C747).** Every dual navamsa the committee
  passed over for a later one lay in the lagna the Sun stood in (Virgo,
  the Sun in Virgo, on the mornings of Ghatasthapana 2024 and 2025 and of
  Phulpati 2024). Barring that lagna is the muhurta texts' rule that the
  lagna be free of a malefic, read for the Sun alone: barring Mars,
  Saturn and the nodes too loses Ghatasthapana 2025, printed in Libra
  with Mars there. `SaitRising.freeOf` names the grahas; the Nepal saits
  name the Sun. A reading, as C745's is.
- **Which navamsa (C746).** With the Sun's lagna barred, Ghatasthapana
  takes the latest whole one in its window and Phulpati the earliest in
  its forenoon; each turned round loses its printed time. Where the
  window cuts every candidate short (Abhijit in 2023, Chitra holding the
  forenoon), the longest stretch is taken, not a stub at the edge.
- **Laxmi Puja ships its window.** `LAKSHMI_PUJA` is the text's pradosha,
  the three night muhurtas from sunset (DS p. 77, NS pp. 148–149), with
  no rising: no printed lagna for it was found in the committee's prints
  in hand (C824), so none is chosen.
- **Not shipped.** The Vijaya Dashami tika: the tiered rule (Dhanu lagna
  with a dual navamsa, else Dhanu lagna, else a dual navamsa, the
  earliest from sunrise to Abhijit's end) gives three of the four printed
  times and not 2026's 10:26, printed in Scorpio lagna (C823).
  Rakshabandhan and Holika: their windows are the text's, but the
  elections the committee prints are not yet measured.
