# Tajika and Varshaphala (annual charts) and other pravesha charts

Status: `research`, 2026-09-04. Checked against the baseline engine's Tajika (whole-sign
solar return), JHora (annual, monthly, 2.5-day, 5-hour, 25-minute, 2-minute
charts, true or mean solar motion, sunrise charts, Tithi/Yoga/Nakshatra
Pravesha) and PyJHora (Muntha, balas, Ithasala family, Mudda, Patyayini).

| feature | inputs | variants | baseline | field | tier |
|---|---|---|---|---|---|
| Varsha Pravesh: solar return to the natal sidereal Sun longitude | crossing search | true solar motion (default) or mean (365.2564 days × n); precession-corrected in Western usage | yes | all | P0 |
| Maasa Pravesh (monthly, every 30° of Sun) and finer charts (60-hour, 2.5-day, 5-hour ...) | crossing search | | no | JHora, PyJHora | P1 |
| Muntha (progressed lagna one sign per year) and its lord | natal lagna, age | | partial | all | P0 |
| lord of the year (Varshesha) from the five candidates (Muntha lord, lagna lord of annual chart, natal lagna lord, Trirashi lord, Dinaratri lord) by Pancha Vargeeya Bala | | | partial | all | P0 |
| Pancha Vargeeya, Dwadasha Vargeeya and Harsha balas | see strengths | | partial | JHora, PyJHora | P0 |
| Tajika aspects: conjunction, sextile, square, trine, opposition with deeptamsha orbs per planet; applying versus separating by speed | annual chart | orb table (Sun 15, Moon 12, Mars 8, Mercury 7, Jupiter 9, Venus 7, Saturn 9) — **verified 2026-09-23** from Charak's Table X-1, together with the rule that a pair takes the *mean* of its two | partial | all | P0 |
| the sixteen Tajika yogas (Ikkavala, Induvara, Ithasala kinds, Ishrafa, Nakta, Yamaya, Manau, Kamboola, Gairi Kamboola, Khallasara, Radda, Duphali Kutta, Dutthotha Davira, Tambira, Kuttha, Durapha) | aspects, balas | | partial | JHora, PyJHora | P0 |
| sahamas (36) with day and night formulas | annual chart | | no | JHora, PyJHora | P1 |
| Mudda (Varsha Vimshottari) and Patyayini dashas | annual chart | | partial | all | P0 |
| Tithi Pravesha, Yoga Pravesha, Nakshatra Pravesha annual and monthly charts | Sun–Moon composite crossing | | no | JHora | P1 |
| Tajika sunrise charts | | | no | JHora | P2 |
| Western solar and lunar returns, planet returns, precession-corrected, demi and quarti returns | crossing search | tropical | no | Solar Fire | P1 (`western`) |

## Closing checklist

- ~~Confirm deeptamsha orbs~~ — **done** (2026-09-23): the table above is
  the source's, and a pair is governed by the mean of its two. The
  **applying rule for retrograde bodies** is still open: the source ranks
  the seven by speed as a fixed order and never says whether retrogression
  reverses it, so `BY_SPEED` is a ranking and says so.
- The Varsha Pravesh instant depends on the ayanamsha and the natal Sun's
  exact longitude, so it is a good cross-provider conformance case.

## A Tajika source, read (2026-09-22)

The rows above were written from the references' *feature lists*. This
section is the first reading of a Tajika **text**: K.S. Charak, *A
Textbook of Varshaphala* (UMA Publications, 3rd ed.), in the archive.org
item the Raman balas come from. It is **rank 2** — a modern textbook and
not the Tajika Neelakanthi — so every rule below wants the classical text
before its crux closes. Its tables come out of the OCR as noise, so each
one here was read off the rendered page.

Three of the four tables the technique needs turn out to have a
**generating rule**, and deriving them rather than pasting them is what
lets a test fail both ways.

### The Muntha

The birth lagna's sign advanced one sign for each **completed** year:
add the completed years to the lagna's sign number, divide by twelve, and
the remainder is the sign, a remainder of zero meaning Pisces. The book's
worked example is Leo rising (5) with forty years complete — 45 mod 12 =
9, Sagittarius — for the chart it calls the *forty-first* year's.

That last clause is the trap. The book numbers a chart by the year of
life it **opens** and does its arithmetic in years **completed**, and the
two are one apart. Over the twelve lagnas and 120 years the two readings
never once agree on the Muntha's sign, and they agree on that sign's
lord in 120 of 1440 cases — only where Capricorn meets Aquarius, because
Saturn rules both. `Pravesha.year` counts returns for this reason.

The book also progresses the Muntha **within** the year, at 2°30′ a month
and 5′ a day, which implies it begins the year at its sign's first degree
rather than carrying the natal lagna's degree. It does not say so
outright, and the two readings differ by up to a whole sign at the year's
end (crux C107).

### The five office-bearers, and the year lord

The Muntha's lord, the natal lagna's lord, the annual lagna's lord, the
Tri-Rashi lord and the Dina-Ratri lord — the lord of the Sun's sign if the
pravesha falls by day, of the Moon's sign if by night. One planet may hold
several portfolios.

The **Varshesha** is the strongest of the five by Panchavargiya bala
*that also aspects the annual lagna*; failing that, the strongest that
does; ties go to the holder of most portfolios; and if none aspects the
lagna, or all are under five units, the Muntha's lord takes it. So the
year lord needs the Panchavargiya bala **and** the Tajika aspects, and
neither is built.

### The Tri-Rashi lords: 24 cells, one rule

The printed table is the Dorothean triplicities with a positional rule,
and every one of its 24 cells follows it: a sign takes its triplicity's
**day and night** lords in that order if it is the first of its
triplicity, **swapped** if it is the second, and the *participating* lord
by day and night alike if it is the third.

### The Panchavargiya bala

Griha (Kshetra) 30 units, Uchcha 20, Hudda 15, Drekkana 10, Navamsha 5 —
eighty at most, divided by four for a Vishwa Bala out of twenty. Each is
reduced to three quarters in a friend's division, a half in a neutral's
and a quarter in an enemy's, where friendship is **positional and
Tajika's own** — friends at houses 3, 5, 9 and 11 from each other, enemies
at 1, 4, 7 and 10, neutrals at 2, 6, 8 and 12 — and not the natural
friendship the Parashari strengths use.

### The Tajika Drekkana: 36 cells, one line

Its lords are not the Parashari decanate lords, and the book gives the
rule in prose: the cycle **Mars, Mercury, Jupiter, Venus, Saturn, Sun,
Moon** — the weekday order begun at Mars — indexed by
`(sign + 5 × decanate) mod 7`, zero-based from Aries. That single
expression reproduces all 36 printed cells and all seven of the book's
worked values.

### The Hudda: 60 cells, and no rule — but not no pattern

The book says the Huddas follow no regular pattern and calls them "an
interesting area for further research". Read off the page, they are the
**Egyptian terms**: every one of the twelve degree columns is the Egyptian
bound widths exactly, and 57 of the 60 lords agree too. Three signs
transpose two adjacent lords while leaving the degrees untouched —
**Gemini** (Venus 6 then Jupiter 5), **Sagittarius** (Mars 5 then Saturn
4) and **Aquarius** (Venus 7 then Mercury 6).

The book's own worked example touches seven cells and **none of those
three**, so nothing in the book can tell a transposition from a printing
fault (crux C109). Two invariants do hold over all sixty and belong in a
test: every sign's five widths sum to thirty, and every sign carries
Mars, Mercury, Jupiter, Venus and Saturn exactly once each.

### The worked year, read back through the SDK

The source works one birth through to its forty-first year's chart —
Bombay, 20 August 1944, 07:11 IST, Chart III-1 — and the SDK reproduces
all of it (`03-design/muntha-measured.md` §5). Two things the text does
not say outright came out of the reading back.

**Its return is the mean one.** It casts the year by adding a
**Dhruvanka** to the birth — 1d 6h 6m 29s for forty years, 1d 6h 9m 10s
for one — and both are whole mean sidereal years modulo a week, to the
second. So its 13:17:29 IST is `Reading::Mean`, which the SDK lands on to
1.5 s. It names the true return as the reference and the difference as "a
few minutes … due to the disturbance of the Sun's longitude by the
planets", which is 0.83 minutes here on a mean ayanamsha. That supports
the SDK's default rather than moving it: the text uses the mean year as an
approximation to the true one, and says so.

**Its positions are geocentric.** Under a topocentric frame its Moon is
58′ out, which is the Moon's parallax at Bombay; under the geocentric
default it is 3′. A consumer comparing against a Tajika text should found
the chart geocentrically, and the profile's centre is reported on every
answer so that this is a thing one can check.

### The aspects, read (2026-09-23)

The Tajika aspect is a relation between **signs** — friendly at 3, 5, 9
and 11, inimical at the kendras, and nothing at 2, 6, 8 and 12, each
aspecting kind halved into the open and the secret — while the deeptamsha
is a distance between **planets**. Two planets inside the mean of their
orbs are in **Ithasala** when the faster is behind the slower and
**Ishrafa** when it is past; a planet at 29° or more acts from the next
sign as well, which can make an Ithasala it would otherwise be too far
advanced for.

"Behind" is **degrees within the sign**, the completed signs deleted, and
not longitude: the source's own worked pair has the Sun at Leo 3°50′
behind Mars at Scorpio 7°42′, three signs further on. It is the one thing
here an implementer would naturally get backwards.

The speed order is fixed by the tradition — Moon, Mercury, Venus, Sun,
Mars, Jupiter, Saturn — and is a ranking rather than a measurement.
