# Matching: the Ashta Koota and the ten considerations (the `matching` module)

Status: the Ashta Koota `built`, 2026-10-04; the ten considerations
`designed`. Written from the sources before any code.
`sdk.chart().matching` answers two charts' Ashta Koota, and
`matching_with` a batch against one partner's birth. Every binding reads
it as `chart.matching`.

Phase 8 opens with matching, P0 in `01-research/feature-universe/10-matching.md`.
Two charts are compared through the Moon's nakshatra, pada and sign at
birth. The North reads eight kootas worth 36 points. The South reads
ten considerations, each agreeing or not. Both are tables over the
catalogue the SDK already holds: the nakshatras carry their gana, yoni
and nadi, and the grahas their natural friends. So the work is in the
doctrine: which table, which points, and which exceptions.

## What the sources decide

**Muhurta Chintamani** (Daivajna Rama, 1600), *Vivaha prakarana* VI.21–35,
is rank 1 for the Ashta Koota. It was read in the 1954 printing with the
*Piyushadhara* commentary, on the Internet Archive's DLI scan
`in.ernet.dli.2015.326601`, every verse on the page image (printed pp.
245–260 = leaves n257–n272). The points are not in the verses. The
commentary gives them from *Daivajna-manohara*, and both are rank 1 here.

- **The eight and their weights** (v. 21, p. 245): Varna 1, Vashya 2,
  Tara 3, Yoni 4, Graha Maitri 5, Gana 6, Bhakoot 7 and Nadi 8, each
  "stronger than the one before".
- **Varna** (v. 22): Cancer, Scorpio and Pisces are Brahmin; Aries, Leo
  and Sagittarius Kshatriya; Taurus, Virgo and Capricorn Vaishya;
  Gemini, Libra and Aquarius Shudra. A bride of higher varna than the
  groom is not praised. 1 point when the groom's varna is the same or
  higher, 0 when lower; "some say" half for the same (C259).
- **Vashya** (v. 23): leaving Leo, every sign is vashya to the human
  signs, and the water signs are their food. Every sign is vashya to
  Leo except Scorpio. "The rest, by usage." The verse does not finish
  the table (C260). Points: friendship 2, enmity or food 0, vashya with
  enmity 1, vashya with food a half.
- **Tara** (v. 24): count from the bride's nakshatra to the groom's, and
  from his to hers, and take each remainder of nine. The 3rd, 5th and
  7th are bad. Good both ways is 3 points, one way 1½, neither 0.
- **Yoni** (vv. 25–26): fourteen animals, Uttarashadha with Abhijit the
  mongoose. Seven pairs are great enemies: horse–buffalo,
  elephant–lion, sheep–monkey, serpent–mongoose, deer–dog, cat–rat,
  tiger–cow. Points: the same yoni 4, friend 3, neutral 2, enemy 1,
  great enemy 0. Only the great enmities are listed (C261).
- **Graha Maitri** (vv. 27–28): the natural friendships, which are the
  catalogue's own for the seven sign lords, and a test holds them.
  Points: one lord or mutual friends 5, friend and neutral 4, both
  neutral 3, friend and enemy 1, neutral and enemy ½, mutual enemies 0.
- **Gana** (vv. 29–30): Deva, Manushya and Rakshasa, nine stars each.
  The same gana is best, Deva with Manushya middling, Rakshasa with
  Manushya death, Rakshasa with Deva enmity. Points: the same gana 6;
  groom Deva with bride Manushya 5; bride Deva with groom Manushya "four
  or three"; groom Rakshasa with bride Deva 2 and with bride Manushya 1;
  otherwise 0 (C262).
- **Bhakoot** (v. 31): the signs 6/8 apart are death, 5/9 loss of
  children, 2/12 poverty; 1/1, 3/11, 4/10 and 7/7 bring happiness. 7
  points or none.
- **Its exceptions** (vv. 32–33, pp. 251–257). A bad bhakoot is lifted
  by:
  1. one lord for both signs;
  2. the lords' friendship;
  3. the navamsha lords' friendship;
  4. the taras' purity;
  5. the signs' vashya.

  The nadi must be pure in every case. The commentary reports Garga:
  the 6/8 needs three of these, and the 2/12 and 5/9 two. It closes
  with "as tradition holds" (C263).
- **Nadi** (v. 34). Adi is Ashvini, Ardra, Punarvasu, Uttara Phalguni,
  Hasta, Jyeshtha, Mula, Shatabhisha and Purva Bhadrapada. Madhya is
  Bharani, Mrigashira, Pushya, Purva Phalguni, Chitra, Anuradha, Purva
  Ashadha, Dhanishtha and Uttara Bhadrapada. Antya is the other nine.
  The same nadi is bad, and the middle one is death. 8 points or none.
  The commentary (p. 260) reports four nadis in Ahichhatra and five in
  Panchala, and a reading where only the middle nadi kills (C264).

The catalogue's gana, nadi and yoni of all 27 nakshatras were checked
against vv. 25–34: every one agrees. Pushya's and Krittika's `GOAT` is
the commentary's *aja* for the verse's *meṣa*.

**Kalaprakasika**, chapter XIII, is rank 1 for the South's ten. It is N.
P. Subramania Iyer's 1917 translation, on the Internet Archive scan
`in.ernet.dli.2015.45999`, printed pp. 69–77, read on the page images
for the lists.

- **The ten**: Dhinam, Ganam, Mahendhram, Sthree-Dheergham, Yoni, Rasi,
  Rasyadhipathi, Vasyam, Rajju and Vedhai. Each agrees or does not, and
  "at least five" must agree (p. 76). They carry no points.
- **Its gana list prints five Manushya stars** (p. 72). Muhurta
  Chintamani v. 29 names all nine, so the four the page omits are
  Manushya on a second rank 1 text (C265).
- **Its own tables.** The Yoni puts Uttarashadha with the cow, not the
  mongoose (p. 73). The Vasyam is a complete sign table (p. 75). The
  Rasyadhipathi is its own friendship list (pp. 74–75), and the varna
  of the signs differs (p. 77). Each is kept as its source prints it,
  and none is mixed into the other system (C266).

## What is decided

- **Clauses, not a verdict.** Each koota answers its points, its
  maximum and the facts it read (the two nakshatras' count, the two
  ganas, the two lords' relation). The total is a sum. Bhakoot and
  Nadi report their dosha, and each exception reports whether it holds.
  No total is turned into "match" or "reject": the texts leave that to
  the reader, and Kalaprakasika's "at least five" is a count the answer
  carries.
- **The bride and the groom are named.** Every rule above is directional
  (varna, gana, the count). A request names who is whom rather than
  "first" and "second".
- **The Moon at birth** gives each native's nakshatra, pada, sign and
  navamsha. Both come from founded charts in their own zodiac, so a
  binding asks it of two charts like a synastry.
- **Every variant is a knob with its source as the default:**
  - the equal varna (C259): 1;
  - the gana where the text gives two readings (C262): 4;
  - the nadi count (C264): three nadis, any shared nadi a dosha.

  Each knob names the alternative's source.
- **Vashya (C260)** ships the verse's own relations where it decides,
  and Kalaprakasika's sign table for the rest. The pair's relation is
  reported, so a consumer with another table sees which cells it
  differs on.

## The surface

- `crates/matching`: `Native { nakshatra, pada, rashi, navamsha }`, read
  by `Native::of_moon`; `ashta_koota(bride, groom, KootaRules) ->
  AshtaKoota`, each row's `reading` naming its koota; and, still to come,
  `dasha_koota(bride, groom) -> TenConsiderations`.
- `sdk.chart().matching(&bride, &groom, KootaRules)` on two founded
  charts. `sdk.chart().matching_with(&charts, &PartnerMatching)` matches
  a batch with one partner's birth, founded once.
- At the boundary, `matching_json` is `{"partner", "partnerRole",
  "rules"}`. The partner is synastry's record. `partnerRole` is
  `BRIDE` or `GROOM`, and every chart of the batch stands on the other
  side. The answer crosses in `matchings`, a row a chart with every
  koota's reading, and `matching_kootas`, eight rows a chart with the
  points. Every binding reads `chart.matching`, with parity across the
  five runners.

## What building it found

- **The record names the partner's side, never the chart's.** A batch's
  charts all stand on one side, so one word says both. The side is
  required: a match with no `partnerRole` is refused rather than
  assuming the partner is the bride.
- **The rules nest under `rules`.** Laid flat beside the partner, a
  misspelt rule would be named by the record alone, as `flatten` names
  it. Nested, `matching.rules.nadi` names the typo.
- **A reading names its koota.** The encoder checks the eight against
  the verse's order before writing, so every reading column holds one
  cell a chart. A kernel that read them out of order fails as an
  internal error, never as shifted columns.
- **No dosha is a member at the boundary.** `TsBhakootDosha` carries
  `NONE`, held to serde with the other three as `TsBrahmaOutcome`'s
  `FOUND` is, and every binding reads it back as null.
- **`TaraReading` was taken.** Python and Dart already had a tarabala
  `TaraReading`, so each koota's reading is named for the koota:
  `VarnaKoota` to `NadiKoota`.
- **A lift is a clause, not points.** The measured page proposed that
  Bhakoot gives its 7 when a dosha is lifted, and the kernel said no on
  3 790 pairs: *Daivajna-manohara* scores the signs' distance alone, and
  the verse's exceptions say the bad Bhakoot is auspicious without
  restoring a point. So the Bhakoot lift and the middle-nadi reading
  change what the answer says and never a total, and the page counts
  both. Garga's count is far stricter: it lifts 1.6% of the 6/8 doshas
  where any one exception lifts 54.8%.
- **A birth matched with itself is a fixed point.** One sign and one
  nakshatra give every koota but Nadi its whole points and the shared
  nadi none, 28 whatever the Moon. Each binding's test starts from it.

## The acceptance tests

- Every table against its verse: the gana, the nadi, the yoni and the
  great enmities, and the friendships for the seven lords.
- Hand-worked pairs from the text's own clauses: a 6/8 bhakoot lifted
  by one lord (Aries–Scorpio, Mars), the same nadi, the madhya nadi.
- Symmetry where the rule is symmetric and asymmetry where it is not.
  Tara, Yoni, Maitri, Bhakoot and Nadi are symmetric in their points,
  while Varna and Gana change when bride and groom swap. A test swaps
  every pair of the 729 nakshatra pairs and holds both.
- The total over every pair of the 108 padas, which fix all a koota
  reads, is a measured page
  ([`matching-measured.md`](matching-measured.md), held by
  `check-matching`): the distribution of points, each koota's share, how
  often each exception lifts a dosha, and what each knob moves.
