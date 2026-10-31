# The death-time check

Status: `built`, 2026-10-31. Design for the application's SR-26
(`consumers/addendum-3.md`). The ashauch answers how long a period runs;
this page answers what stood at the death and at the burning, and which
rites the texts tie to that. Both read the same *Dharmasindhu*.

## 1. Sources

| tag | work | rank | edition read | where |
|---|---|---|---|---|
| DS | *Dharmasindhu*, third pariccheda, uttarārdha, §§49 and 62 | 1 | 1888 Nirnaya-sagara edition (`in.ernet.dli.2015.513461`), pp. 316, 321, 334–335; pp. 306 and 335 read off the page images | the archive's OCR and leaves |
| NS | *Nirṇayasindhu*, third pariccheda, uttarārdha, "atha vidhānāni" | 1 | ed. Narayan Ram Acharya, 1949 (`in.ernet.dli.2015.326320`), pp. 434–435 | the archive's OCR |
| MC | *Muhūrta-cintāmaṇi*, nakṣatra-prakaraṇa II.48–49, with its commentary, which quotes Vasiṣṭha, Nārada, Śrīpati and the *Brahma* and *Garuḍa* purāṇas | 1 | the 1954 print, pp. 84–87 | `notes/texts/mc1954.txt` |

No published almanac's death-time calculator was read, so nothing here is
fitted to one. A search of Kane IV's OCR found no treatment of the panchaka in its *Śuddhi*
chapter.

## 2. What the texts say

- **The panchaka.** DS p. 334: "पञ्चकं नाम धनिष्ठोत्तरार्धमारभ्य रेवत्यन्तं
  सार्धनक्षत्रचतुष्टयम्", four and a half nakshatras from the second half of
  Dhanishtha to the end of Revati. NS p. 434 (*Garuḍa* through the
  *Madanaratna*): "आदौ कृत्वा धनिष्ठार्धमेतन्नक्षत्रपञ्चकम्". MC II.48 names it
  by the Moon, "the Moon in Aquarius or Pisces", and Vasiṣṭha, Nārada and
  Śrīpati there name it by the half. The two are one span: Dhanishtha's
  second half begins at 300°, the first of Aquarius. So the check reads the
  Moon's sign and no pada. (`panchaka_start` in the settings governs the
  day's printed `omens.panchaka`; a profile set to the whole of Dhanishtha
  does not move this answer, because these texts do not read so.)
- **Which instant counts** (C951). MC's commentary sets out three views (p.
  85) and takes the third: a death in *any* nakshatra is not burnt in the
  panchaka; a death in it is itself a fault too (*Brahma-purāṇa*). DS p.
  335 gives the working rule, as does MC p. 86 in almost the same words:

  | death | burning | owed | words |
  |---|---|---|---|
  | outside | outside | nothing | — |
  | outside | in the panchaka | the figures, no shanti | "नक्षत्रान्तरे मृतस्य पञ्चके दाहप्राप्तौ पुत्तलविधिरेव न शान्तिकम्" |
  | in it | outside (DS: "in Ashwini") | the shanti, no figures | "पञ्चकमृतस्याश्विन्यां दाहप्राप्तौ शान्तिकमेव न पुत्तलविधिः" |
  | in it | in it | both | DS pp. 334–335; NS p. 434 |

  The texts do not bar the burning: "the panchaka runs long and a body is
  not kept" (MC p. 85), so it goes ahead with the figures.
- **The figures.** Five, of darbha, wound with wool, smeared with barley
  paste, burnt with the body (DS pp. 334–335, NS p. 434, *Brahma-purāṇa* in
  MC p. 85). The SDK names the rite and its clause and gives no procedure.
- **Tripada and dvipada.** DS p. 335 and NS p. 435 list Punarvasu, Uttara
  Ashadha, Krittika, Uttara Phalguni, Purva Bhadrapada and Vishakha as
  tripada, and Mrigashira, Chitra and Dhanishtha as dvipada. MC p. 86 gives
  the rule: a nakshatra in which a sign ends after the first or third foot,
  after the second foot. So the lists are arithmetic, and
  `mc_249_the_lists_are_the_sign_arithmetic` derives them.
- **Tripada alone.** DS p. 335 "त्रिपादर्क्षेऽप्येतदेव शान्तिकम्"; NS p. 435
  the same: the same shanti. Aparārka in NS p. 435 adds gold in the mouth and
  three flour figures. No text read gives a bare dvipada nakshatra a rite
  (C952): the answer reports it and owes nothing.
- **Pushkara.** The bhadrā tithis (2, 7, 12) with Sunday, Tuesday or
  Saturday and a tripada nakshatra are a tripushkara; with a dvipada, a
  dvipushkara (DS pp. 316, 335; MC II.49; Garga in NS p. 434). Then three
  penances and three barley figures, and the shanti (DS p. 335). The verse
  "दहने मरणे त्रिद्विपुष्करे त्रिगुणं फलम्" names the burning beside the death,
  so a pushkara at either owes them. MC p. 87 gives Vasiṣṭha's gift of three
  cows, or two, as the remedy.
- **Other readings.** "कश्चित्": Thursday for Sunday; and the two limbs alone
  a dvipushkara (DS p. 316; Garga's "द्वयोर्योगे द्विपुष्करः", NS p. 434).
  Both are a table's values (C953).

## 3. The call

`almanac.deathTime` (`ts_almanac_death_time`, every binding, the agent
tool) takes the place, the clock, `at`, optional `crematedAt` and optional
`table`, and answers `{death, cremation, rites, table}`, each reading
`{at, nakshatra, tithi, vara, moon, panchaka, feet, pushkara}`. Each instant
is read as a sankalpa reads it (C954). With no `crematedAt`, the cremation
is taken to fall as the death did; a family that buries or waits gives the
instant. `DeathTimeTable` holds the lists; a table is data.

## 4. Cruxes

| # | item | what is known | what is missing | rank | action |
|---|---|---|---|---|---|
| C950 | Where the panchaka begins for a death | DS p. 334, NS p. 434 and MC II.48 agree: Dhanishtha's second half, the Moon in Aquarius | — | 1 | closed by the texts; the check reads the Moon's sign whatever `panchaka_start` says |
| C951 | Past Ashwini | DS p. 335 and MC p. 86 name Ashwini as the burning past a panchaka | whether a burning later still is the same | 1 | built: any burning past it answers as Ashwini does |
| C952 | A bare dvipada nakshatra | DS p. 335 and NS p. 435 give a tripada the shanti and say nothing of a dvipada alone; DS p. 321 bars gathering the bones in either | a clause | 1 | open: reported, no rite owed |
| C953 | What makes a dvipushkara | DS pp. 316, 335 and MC II.49: tithi, vara and a dvipada; "कश्चित्" and Garga in NS p. 434: any two; Thursday for Sunday, "कश्चित्" | which a family keeps | 1 | default the three limbs and Sunday; `twoMakeDvipushkara`, `pushkaraVaras` |
| C954 | The tithi and vara of an instant | the texts name the yoga of a day; they do not say whether a tithi is read at the instant or at sunrise | a clause, or a dated case | 1 | open: the tithi at the instant and the vara of the sunrise day, as a sankalpa |
