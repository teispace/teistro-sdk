# Amrita kalam, the day's lagna table, and the day page's names

Status: `design`, 2026-10-21. §1, the amrita kalam, is **built** with its
default decided on a published panchanga (§1.5); §2 and §3 are not yet.

Three things a printed panchanga's day page carries that the almanac does
not yet answer: the *amṛta ghaṭikā* (amrita kalam) beside the varjyam it
already answers (`omen::varjyam`, C165), the table of lagnas rising
through the day, and names for every catalogue member such a page
prints.

## 1. Amrita kalam

### 1.1 What was missing

C399 recorded the amrita kalam as not built: neither Muhurta Chintamani
(the 1906 edition's OCR and index), Raman's *Muhurtha* nor Kalaprakasika
prints a table of its starts. The table is in the *Praśna Mārga*, which
gives the visha ghatis of each nakshatra and, in its commentary, the
amrita ghatis.

### 1.2 Sources

| tag | work | edition | read |
|---|---|---|---|
| PM | *Praśna Mārga* (Kerala, c. 1649), ch. 2 v. 21 and its Sanskrit commentary, rank 1 | the 1926 Sanskrit edition with commentary, archive.org `prasnamarga035823mbp`, book p. 20 = PDF p. 38 | the page image read |
| PM-R | B. V. Raman's translation, *Prasna Marga* part I, ch. 2 v. 21, pp. 45–47, rank 2 | — | **secondhand**: its table as quoted in a published essay on the Kerala muhurta tables; the book itself was not read |
| MV | *Mādhavīyam* v. 26 (Nampoothiri's edition, 2005, p. 71), rank 2 | — | **secondhand**, as PM-R |
| MC | Muhurta Chintamani, the varjyam table the SDK ships (`VARJYAM_MUHURTA_CHINTAMANI`) | 1906 | in the SDK (C165) |

### 1.3 Three readings

**(a) Adjacent.** The PM commentary, read off the page image: "अमृतघटिकाः
सर्वेष्वपि नक्षत्रेषु विषघटिकाभ्योऽनन्तराश्चतस्रो घटिकाः" — in every
nakshatra the amrita ghatis are the four ghatis **immediately after the
visha ghatis**. So amrita starts at visha + 4 and runs four ghatis.

**(b) Thirty plus a word numeral.** The same commentary then gives a
*matāntara* (another view) as a verse: "सम्कृत्याकृतिनागसायकजिनैरिन्द्रोत्कृतिभ्यां
जिनैर्मन्वर्कैस्तिथिशक्रनागवसुभिः वेदाष्टशक्रैः क्रमात् । …ैर्युता नाडीस्त्रिंशदतीत्य
भेषु परतो नाड्यश्चतस्रोऽमृतम्" — in each star, past thirty nadis and the
numbers named in order, four nadis are amrita. The numerals are
*bhūta-saṃkhyā* words (*saṃkṛti* 24, *ākṛti* 22, *nāga* 8, *sāyaka* 5,
*jina* 24, *indra* 14, *utkṛti* 26, *manu* 14, *arka* 12, *tithi* 15,
*śakra* 14, *vasu* 8, *veda* 4). Read as 30 + n they give Ashwini 54,
Bharani 52, Krittika 38, Rohini 35, Mrigashira 54, Ardra 44, Punarvasu
56, Pushya 54, Ashlesha 44, then Purva Phalguni 45, Uttara Phalguni 44,
Hasta 38, Chitra 38, Swati 34, Vishakha 38, Anuradha 44: exactly the
*Mādhavīyam* column MV prints for those sixteen. The verse's second half
is damaged in the scan and **not decoded**; the remaining eleven are MV's
printed values only.

**(c) Visha plus twenty-four.** The table printed in Raman's translation
(PM-R) and in most almanacs. Its starts, Ashwini to Revati:

```
amrita (PM-R): 42 48 54 52 38 35 54 44 56 54 44 42 45 44 38 38 28 38 44 48 44 34 34 42 40 48 54
visha  (PM-R): 50 24 30 40 14 11 30 20 32 30 20 18 22 20 14 14 10 14 20 24 20 10 10 18 16 24 30
visha  (MC)  : 50 24 30 40 14 21 30 20 32 30 20 18 21 20 14 14 10 14 56 24 20 10 10 18 16 24 30
```

Amrita = visha + 24 for 23 of the 27. The four exceptions: Ashwini (42,
where 50 + 24 passes sixty) and Rohini (52, likewise from 40); Hasta (45,
which fits MC's 21 rather than PM-R's 22, so 21 + 24); Anuradha (28,
where 10 + 24 = 34, read as a misprint). PM-R's visha differs from MC in
Ardra (11 against 21), Hasta (22 against 21) and Mula (20 against 56).

The *rule* behind (c) is not stated in any text read; it is inferred from
the table, and its two wrapped stars (Ashwini, Rohini) show it is not a
plain sum mod 60. MV (column b) agrees with neither.

### 1.4 Design

```rust
pub const AMRITA_PRASNA_MARGA_ADJACENT: Setting  // (a): visha + 4, on the varjyam table chosen
pub const AMRITA_MADHAVIYA: [f64; 27]            // (b): MV's starts
pub const AMRITA_RAMAN: [f64; 27]                // (c): PM-R's printed starts, as printed
pub const AMRITA_GHATIS: f64 = 4.0;
```

- `Omens.amrit_kalam: Vec<Interval>` beside `varjyam`, computed by the
  varjyam's own arithmetic (`omen::varjyam`): each start and the four
  ghatis are sixtieths of the star's span, so a star of 64 ghatis has an
  amrita of 4.27 ghatis (C679, the varjyam's own scaling).
- A setting `panchanga.amrit_kalam`, members `PRASNA_MARGA` (a),
  `MADHAVIYA` (b), `RAMAN` (c) and `NONE`, a key of its own beside
  `panchanga.varjyam`, so a consumer pairs either table with either
  reading. Under (a) the start follows whichever varjyam table is chosen
  (C678); under (b) and (c) the table is fixed.
- The tables carry their sources as catalogue sources do, with (b)'s
  eleven undecoded starts marked `unverified` and (c) marked secondhand.
- **Default (C677, maintainer):** proposed (c), `RAMAN`, because it is
  what the almanacs print and therefore what a reader compares against;
  (a) is the only reading verified on a primary page, so it is offered
  and named. If the maintainer prefers the verified reading, (a) on MC's
  table.

C399's action changes from "not built" to this page.

### 1.5 Built, and the default measured

A week of a published panchanga decides the default. Drik Panchang's day
pages for Delhi, 15 to 21 June 2025, print an amrita kalam each day and
a varjyam on six. Read as sixtieths of each star over the SDK's own
nakshatra spans, every amrita start is reading (c)'s: Dhanishtha 34,
Shatabhisha 42, Purva Bhadrapada 40, Uttara Bhadrapada 48, Revati 54,
Ashwini 42, and Shravana's 34. Over the default context (Lahiri, the
built-in ephemeris) the SDK's windows are the printed ones, every edge
within the printed minute: seven of seven, and the varjyam's seven of
seven on Muhurta Chintamani's table (Uttara Bhadrapada 24, where Raman's
varjyam table has 30), including Bharani's before the sunrise of 22
June, which the 21st's page prints unlabelled (`crates/sdk/tests/printed_day.rs`).

So the almanacs pair Muhurta Chintamani's varjyam with Raman's amrita
table, and that is the default: `panchanga.amrit_kalam` is `RAMAN`
(C677 decided). Built as designed, with two changes:

- **`MADHAVIYA` is not shipped.** Eleven of its starts are MV's printed
  values with the verse that would check them undecoded, and no
  almanac read prints it. It joins when the verse is read.
- **Under `PRASNA_MARGA`, a star whose varjyam ends it has none.** The
  commentary's four ghatis "immediately after" the visha fall past the
  star's end where the visha starts after 52 (Muhurta Chintamani's Mula
  at 56). The next star's span would scale them differently and no text
  says which, so the reading gives none there rather than guess (C678).

`Omens.amrit_kalam` is beside `varjyam`, by the same arithmetic
(`omen::ghati_windows`), and crosses to every binding as the day's
`amrit_kalam` span section (`amritKalam` in Node, Dart and Java).

## 2. The day's lagna table

A day page lists each sign rising from sunrise to the next sunrise with
its start and end.

### 2.1 Definition

The table for a day at a place is the list of `LagnaSpan { sign, from,
to, whole }` covering [sunrise, next sunrise): the cuts of the sidereal
ascendant through each sign boundary, found by `muhurta::window::lagna_cuts`
(minute sampling, bisection to about 1e-8 days) over `lagna_at` in the
context's chart zodiac (`muhurta::sources`, `chart::foundation::ascendant_at`).
The first and last spans are clipped to the day, and `whole: false` says
so (C681): the sign rising at sunrise began before it.

- Under a Surya Siddhanta profile (the Nepal committee's), the lagna is
  the Siddhanta's own: the Sun's sayana longitude and `siddhanta::lagna`'s
  `RisingTimes::at(model, latitude)` (the rising times in asu, corrected
  by the place's ascensional difference), as a patro printed in Nepal
  computes it (C680). Elsewhere, the context's own ascendant.
- `sdk.almanac().day(...)` gains `lagnas: Vec<LagnaSpan>` behind the
  section `lagnas`, asked for only when wanted (a day's cuts cost about
  1,440 ascendant evaluations plus bisection; measure first, `price a
  pass`).
- Polar latitudes: a sign may not rise in a day, or rise twice; the list
  is whatever the cuts give, never assumed to hold twelve (the premise
  `an-equivalence-has-a-premise` names).

### 2.2 Acceptance

No published lagna table has been read into a fixture yet. The test is a
week of a printed panchanga's lagna table at its own place and on its own
sky (`a-printed-day-follows-the-printers-sky`): one printed in Nepal under
the Surya Siddhanta profile, an Indian one under Lahiri, each span's ends
to the printed minute. Until then the test is internal: the spans tile
the day without gaps, and each span's `sign` equals `chart::moment::rising`
at its midpoint.

## 3. The day page's names

### 3.1 What the audit found

The brief said masa and direction have no names. **That is stale**: both
are named in all five packs since `a0d890ad` (2026-09-14), and
`crates/intl/src/validate.rs` asserts 12 of 12 and 8 of 8. Only the
comment in `crates/sdk/examples/almanac.rs` (lines 48–54) still says they
are unnamed, and should be corrected when the example is next touched.

An audit of the five packs against `catalogue/catalogue.json` (65 kinds)
counts 314 members with no en-Latn name. 72 of them are listed in
`xtask/src/intl.rs`'s `UNNAMED` (the chart document's scope); **242 are
not**. By kind:

| kind | en missing | | kind | en missing |
|---|---|---|---|---|
| star | 128 | | ayanamsha_category | 7 |
| koota | 12 | | muhurta_nature | 7 |
| state | 12 | | chart_layout | 6 |
| deity | 10 | | tithi_class | 5 |
| dasha_family | 10 | | body_class | 4 |
| point_family | 8 | | auspiciousness | 4 |
| modality, guna, gender, rising, star_class, blackout_kind | 3 each | | bala_scheme, parity, sex, degeneracy, solar_eclipse_kind | 2 each |
| lunar_eclipse_kind | 1 | | | |

blackout_kind's three are KHARMAS, ECLIPSE_STAR and ECLIPSE_VEDHA; the
eclipse kinds' are PENUMBRAL, ANNULAR and HYBRID. hi-Deva-IN also lacks
members en names: varga 15, dasha_system 11, point 11, era 9,
avastha_sayanadi 12, vaiseshikamsa 27, avastha_cheshta 3, blackout_kind
9, nature 2, and the eclipse kinds 2 each. ne-Deva-NP has no gap beyond
en's.

### 3.2 Proposed names for the day page's kinds

**Unvetted**: these are proposals for a reviewer of each language
(`hand-the-work-out-dont-guess-it`), not names to ship. The Sanskrit
terms are the texts' own; the en forms follow the packs' existing
spelling (unaccented Latin).

| kind | key | en-Latn | ne-Deva-NP | hi-Deva-IN |
|---|---|---|---|---|
| tithi_class | NANDA, BHADRA, JAYA, RIKTA, PURNA | Nanda, Bhadra, Jaya, Rikta, Purna | नन्दा, भद्रा, जया, रिक्ता, पूर्णा | नन्दा, भद्रा, जया, रिक्ता, पूर्णा |
| modality | CHARA, STHIRA, DWISWABHAVA | Movable, Fixed, Dual | चर, स्थिर, द्विस्वभाव | चर, स्थिर, द्विस्वभाव |
| muhurta_nature | DHRUVA, CHARA, UGRA, KSHIPRA, MRIDU, MISHRA, TIKSHNA | Fixed, Movable, Fierce, Swift, Soft, Mixed, Sharp | ध्रुव, चर, उग्र, क्षिप्र, मृदु, मिश्र, तीक्ष्ण | ध्रुव, चर, उग्र, क्षिप्र, मृदु, मिश्र, तीक्ष्ण |
| guna | SATTVA, RAJAS, TAMAS | Sattva, Rajas, Tamas | सत्त्व, रजस्, तमस् | सत्त्व, रजस्, तमस् |
| gender | MALE, FEMALE, NEUTRAL | Male, Female, Neuter | पुरुष, स्त्री, नपुंसक | पुरुष, स्त्री, नपुंसक |
| rising | SIRSHODAYA, PRISHTODAYA, UBHAYODAYA | Head-rising, Back-rising, Both-rising | शीर्षोदय, पृष्ठोदय, उभयोदय | शीर्षोदय, पृष्ठोदय, उभयोदय |
| blackout_kind | KHARMAS, ECLIPSE_STAR, ECLIPSE_VEDHA | Kharmas, Eclipse star, Eclipse vedha | खरमास, ग्रहण नक्षत्र, ग्रहण वेध | खरमास, ग्रहण नक्षत्र, ग्रहण वेध |
| lunar_eclipse_kind | PENUMBRAL | Penumbral | उपच्छाया | उपच्छाया |
| solar_eclipse_kind | ANNULAR, HYBRID | Annular, Hybrid | कङ्कणाकार, मिश्रित | वलयाकार, संकर |

The keys and their order are the catalogue's. Once vetted, each
kind's members leave the audit's unlisted count; the 128 stars and the
other chart-scope kinds stay for `entity-names.md`'s own pass.

## 4. Forks

| # | fork | proposed default | the other readings |
|---|---|---|---|
| C677 | the amrita's start | **decided**: (c) Raman's printed table, the published almanac's to the minute (§1.5) | (a) visha + 4, the PM commentary, verified on the page and shipped as `PRASNA_MARGA`; (b) 30 + n, the *matāntara* verse and MV, not shipped |
| C678 | which visha table (a) rides on | the setting `panchanga.varjyam` already chosen | MC fixed |
| C679 | scaling to the star's span | as the varjyam: sixtieths of the span | fixed ghatis of 24 minutes |
| C680 | the lagna table's lagna | the context's sidereal ascendant; the Surya Siddhanta's own rising times under that profile | the context's ascendant always |
| C681 | the first and last spans | clipped to [sunrise, next sunrise), with `whole: false` | the whole span of each sign touching the day |
| C682 | the unlisted unnamed members | name the day page's kinds (§3.2) once vetted; list the rest in `UNNAMED` with a reason, so the gate's count reaches zero | leave the count as it stands |

## 5. Order of work

1. The amrita tables and `Omens.amrit_kalam` with its setting, each
   reading tested on a star of 60 and of 64 ghatis.
2. `LagnaSpan` and the `lagnas` section, the tiling and midpoint tests,
   then a printed week as the fixture.
3. The names, after review, with the audit made a gate.
