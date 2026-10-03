# Matching: the Ashta Koota and the ten considerations (the `matching` module)

Status: the Ashta Koota and the ten considerations `built`, 2026-10-04.
Written from the sources before any code. `sdk.chart().matching`
answers two charts' Ashta Koota and `sdk.chart().porutham` their ten
considerations; `matching_with` matches a batch against one partner's
birth under both. Every binding reads them as `chart.matching` and
`chart.porutham`.

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

## The ten considerations, decided

Each consideration is a catalogue `koota` member (C282), in
Kalaprakasika's order, and each answers whether it agrees and the facts
it read. `n` is the count from the bride's star to the groom's, the
bride's counting 1.

1. **Dhinam** (`TARA`, pp. 69–72). The 3rd, 5th and 7th of the first
   nine disagree (C267). In the second nine only a quarter of the
   groom's star disagrees: the 1st of the 12th, the 4th of the 14th
   and the 3rd of the 16th (C268). The third nine agrees except the
   22nd, *Vadha-Vainasika*, and the 27th unless the signs are one.
   - One star for both is read by p. 71's lists: excellent and neutral
     agree, the eight to avoid disagree. A star across two signs
     agrees when the groom's quarter is the earlier (C270).
   - Two stars in one sign agree when the groom's is prior. His may
     be the next one when hers is Ashvini, Krittika, Mrigashira,
     Magha, Hasta, Swati, Purva Ashadha or Shatabhisha. Bharani with
     Krittika, Dhanishtha with Shatabhisha and Pushya with Ashlesha
     are left out of the rule and read by the count.
   - The four happy pairs agree whoever is the bride (C269). The six
     unhappy pairs of p. 70 are all 7th counts, and a test holds that
     they disagree.
2. **Ganam** (`GANA`, p. 72). The same gana agrees, and Deva with
   Manushya either way round (C271). Deva with Rakshasa and Manushya
   with Rakshasa disagree. The reading says when the bride's star
   stands beyond the 14th from the groom's, which "diminishes" the
   Rakshasa and does not lift it (C279). The nine Manushya are v. 29's
   (C265).
3. **Mahendra** (p. 72): `n` is 4, 7, 10 and so on to 25.
4. **Sthree-Dheergham** (`STREE_DEERGHA`, p. 72): `n` beyond the
   13th; beyond the 7th is a knob (C272).
5. **Yoni** (p. 73) on Kalaprakasika's own table and eight enmities
   (C278): an enmity disagrees, the same yoni or any other agrees.
6. **Rasi** (`BHAKOOT`, pp. 73–74), the groom's sign counted from the
   bride's. The 7th, 8th to 12th and the 1st agree (C280); the 2nd to
   the 6th disagree. Two exceptions: the 2nd agrees when the groom's
   sign is even, and the 6th when the bride's is odd. Both read the
   bride's sign odd (C281).
7. **Rasyadhipathi** (`GRAHA_MAITRI`, pp. 74–75) on the chapter's own
   friendships. It agrees on one lord, or when each lord calls the
   other a friend. One way is a knob (C273).
8. **Vasyam** (`VASHYA`, p. 75) agrees when either sign is concordant
   to the other on p. 75's own table (C266). That table is not the
   Ashta Koota's `is_vashya`, which takes the human signs and Leo from
   the verse and makes a sign vashya to itself. A shared sign does not
   agree (C274).
9. **Rajju** (p. 75): the same division disagrees, and the reading
   names it. Each star's division is the folded rule (C275).
10. **Vedhai** (`VEDHA`, p. 76): a listed pair, or two stars of the
    Mrigashira–Chitra–Dhanishtha triple, disagree (C276).

**The exception** (p. 76, C277). One lord, friendly lords, or opposite
signs lift Rajju, Vedhai, Ganam and Rasi. Each of the three is reported,
and a lifted consideration agrees and says it was lifted.

**The answer** carries the ten rows, the count that agree, and the
chief five marked: Dhinam, Ganam, Yoni, Rasi and Rajju. The text's
"at least five" stays the reader's to apply, as the 36 points do.

**Not built here:** the p. 70 fruits of named pairs, and p. 77's sex
of the stars, gotra and the signs' castes. They are judgments beyond
the ten, a later step if a consumer asks.

## The surface

- `crates/matching`: `Native { nakshatra, pada, rashi, navamsha }`, read
  by `Native::of_moon`; `ashta_koota(bride, groom, KootaRules) ->
  AshtaKoota`, each row's `reading` naming its koota; and
  `porutham(bride, groom, PoruthamRules) -> Porutham`, ten rows of
  `{ agrees, lifted, reading }`, each reading naming its koota.
  `PoruthamRules` holds the three knobs (C270, C272, C273).
- `sdk.chart().matching(&bride, &groom, KootaRules)` and
  `sdk.chart().porutham(&bride, &groom, PoruthamRules)` on two founded
  charts. `sdk.chart().matching_with(&charts, &PartnerMatching)` matches
  a batch with one partner's birth, founded once, and answers a `Matched`
  a chart: both systems.
- At the boundary, `matching_json` is `{"partner", "partnerRole",
  "rules", "porutham"}`. The partner is synastry's record. `partnerRole`
  is `BRIDE` or `GROOM`, and every chart of the batch stands on the other
  side. The Ashta Koota crosses in `matchings`, a row a chart with every
  koota's reading, and `matching_kootas`, eight rows a chart with the
  points; the ten in `poruthams`, a row a chart with the counts, the
  exception's clauses and every reading, and `porutham_rows`, ten rows
  a chart with whether each agrees and was lifted. Every binding reads
  `chart.matching` and `chart.porutham`, with parity across the five
  runners.

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
- **The South's system is named Porutham.** "Dasha Koota" would sit
  beside `DashaSystem` and read as the periods, so the kernel's
  `porutham`, its `Porutham` answer and the binding's `chart.porutham`
  take the South's own word. Each reading class is named for the
  consideration (`DhinamPorutham` to `VedhaiPorutham`) and carries its
  catalogue koota, as the Ashta Koota's do.
- **Vasyam is not `is_vashya`.** The Ashta Koota's table takes the
  human signs and Leo from VI.23 and makes a sign vashya to itself.
  Read whole, p. 75's own table is neither, so the ten read it alone,
  as C266 asks of every table.
- **The same-sign rules are Dhinam's.** p. 71's "the groom's star
  prior" is the 26th or 27th count inside one sign, which the third
  round's "27th unless the signs are one" already excuses. Rasi is
  silent on one sign (C280), so reading the rule in Dhinam keeps both
  pages' clauses in one place.
- **Rasi's two exceptions are one rule.** The groom's 2nd is even
  exactly when the bride's sign is odd, and p. 74's six felicitous 6ths
  are every odd bride's sign. A test holds the printed pairs against
  the rule (C281).
- **The research's Rajju names were not the text's.** The feature list
  had Pada, Kati, Udara, Kantha and Shira; Kalaprakasika prints Padha,
  Ooroo, Nabhi, Kanta and Siro, and the boundary enum `TsRajju` spells
  the printed five.
- **One request, both systems.** The ten read the same Moons as the
  eight, so `matching_json` asks both and founds the partner once. The
  ten's knobs nest under `porutham`, so a typo is named
  `matching.porutham.deergha`.
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
- The ten against Kalaprakasika's printed lists: the Rajju rule star
  for star, the Vedhai pairs both ways and only as printed, the
  chapter's yoni (three cows, no mongoose), every lord's friendship
  line and the Vasyam table; Dhinam's rounds, quarters, common stars,
  two-sign stars and same-sign stars on hand-placed Moons; p. 70's six
  unhappy pairs disagreeing and four happy pairs agreeing both ways;
  the exception lifting four and saying so.
- The total over every pair of the 108 padas, which fix all a koota
  reads, is a measured page
  ([`matching-measured.md`](matching-measured.md), held by
  `check-matching`): the distribution of points, each koota's share, how
  often each exception lifts a dosha, and what each knob moves.
