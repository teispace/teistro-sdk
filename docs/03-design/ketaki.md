# Ketaki: the plan

Status: `plan`, written 2026-11-17 from `01-research/ketaki.md`. Phase K1 is built
(2026-11-17, lane `ketaki-k1`): the `KETAKI` member, the `ketaki` and `tilak-revati`
profiles, T1 to T6 (`crates/astro/tests/ketaki.rs`, `crates/sdk/tests/ketaki.rs`, the
settings tests), and T10 from K2 (both Tilak dates reproduced at Pune). The rest of the page is not built yet. Cruxes C2180 to C2199 hold what is still open. The framework that Ketaki is the
first new member of is `03-design/reckonings.md`. Read that page first if you are adding
a reckoning other than Ketaki.

## 1. Purpose and scope

The SDK reckons today by drik (the Builtin engine, Teimeris) and by the Surya Siddhanta
(`crates/siddhanta`, opened as `SURYA_SIDDHANTA` and used by `nepal_official`). The
maintainer asked for a third reckoning, Ketaki. The research settles what that word
means in practice (`01-research/ketaki.md` §3.3), and the answer has two layers:

1. **Ketaki as the almanacs compute it** is a modern sky with three things on top: the
   Chitra-paksha ayanamsha of Ketkar's *Ketakī Grahagaṇitam* (1899), the sidereal year,
   and Ketkar's conventions (amanta months, the Shaka year, sunrise-based days, Bhaskara's
   adhika rule). This is what Date's, Nirnaysagar, Vaijayanti and the 1955 survey's
   sixteen almanacs mean by it, and it is what a consumer wants first.
2. **Ketaki as the 1899 text computes it** is a karana with modern constants: the
   *Ketakī Grahagaṇitam*'s verse rules and ten-degree tables. It reproduces Ketkar's own
   almanac and the karana's worked examples to the digit. It is a historical layer, like
   `mithila-makaranda` beside `surya-siddhanta`.

A third, smaller item falls out of the research. The **Revati** origin that the addendum
attributed to Ketaki belongs to Ketkar's withdrawn first edition and to the Tilak
panchang. It gets a profile of its own so that the dispute the consumer meets in
Maharashtra (Tilak against Date, about four days) can be computed. It is not Ketaki.

Out of scope: the tables of *Jyotirgaṇitam* (1898) as a provider, deferred (§9, phase
K6); festival rules from the dharmashastra digests (C2191); a Ketaki interpretation of
anything.

## 2. Placement

| layer | what | where | why |
|---|---|---|---|
| ayanamsha | a new catalogue member `KETAKI` | `crates/core` ayanamsha catalogue; Teimeris gains nothing (the SDK computes it, like Lahiri's fallbacks) | a linear rule needs no engine. A named member carries a citation, appears in `ayanamshas` listings and a chart document, and is the spelling a consumer types. `CUSTOM` with the same three numbers is exactly equal (§4.1) and is the test of the member |
| profile | `ketaki` | `crates/core/src/settings/profiles.rs` | the conventions as knobs over the root (§3); runs over any modern engine |
| provider | `ketaki-grahaganita` profile + a karana provider in `crates/siddhanta` | `crates/siddhanta/src/karana/ketaki.rs` (new) behind the framework's classical entry | the 1899 karana, phase K4 |
| profile | `tilak-revati` | `profiles.rs` | ζ Piscium at 0°, for the Tilak panchang (C2183, C2184) |

So Ketaki is **both** an ayanamsha with a profile and, later, a provider. The modern profile
needs no new provider and no framework change, so it ships first and on its own.

## 3. Settings: the `ketaki` profile

Base: the root. Only the knobs below change. Every value cites `01-research/ketaki.md`
and, through it, the rank-1 text.

| knob | value | root value | citation | crux |
|---|---|---|---|---|
| `frame.ayanamsha` | `KETAKI` | `LAHIRI` | KGG spashta v. 6; JG1937 p. 7 | C2180 |
| `frame.ayanamsha_basis` | `TRUE` (unchanged) | `TRUE` | the value is mean (JG1937 p. 346, research §2.2); a mean value on the true basis gives star-referred longitudes | C2181, C2182 |
| `frame.zodiac` | sidereal (unchanged) | sidereal | KGG v. 3 | — |
| `frame.node` | `MEAN` (unchanged) | `MEAN` | KGG madhyama vv. 17, 23 | — |
| `frame.centre` | `GEOCENTRIC` (unchanged) | `GEOCENTRIC` | KGG chs. 6–7 | — |
| `calendar.lunar_month` | `AMANTA` (unchanged) | `AMANTA` | JG1937 ch. 1 p. 7; RCRC Annexure VI | — |
| `calendar.eras` | Shaka (Shalivahana), elapsed, Chaitradi, first | the root's | KGG madhyama v. 1; RCRC | C2186 |
| `calendar.samvatsara` | the southern (luni-solar, Chaitradi) cycle | the profile's own | the Maharashtra almanacs; to confirm on a printed page | C2186 |
| `day.day_boundary` | `SUNRISE` (unchanged) | `SUNRISE` | JG1937 ch. 1 pp. 7–8 | — |
| `day.sunrise` | `UPPER_LIMB_REFRACTION` (unchanged, provisionally) | the same | neither text names the limb. JG1937 applies refraction (pp. 312–314). The modern heirs print the upper limb with refraction (rank 3) | C2189 |
| `time.zone` | the place's civil zone (unchanged) | the same | a modern almanac prints IST; Ujjain mean time is the karana's own (§5) | C2190 |
| adhika and kshaya masa | the root's rule (no sankranti → adhika; two → kshaya) | the same | JG1937 p. 7 quotes Bhaskara's rule, which the root already encodes | — |

The profile's `Citation`s name KGG and JG1937 for the ayanamsha and RCRC for the
conventions. The profile carries `Mark::Traditional`. The `tilak-revati` profile is the
same patch with `frame.ayanamsha` = `ANCHORED { star: REVATI, longitude_deg: 0 }` until
a rate is found (C2184).

`ketaki-grahaganita` sets `frame.siddhanta` to the new text value for the karana (the
framework's `Siddhanta::Text { reckoning: KETAKI_GRAHAGANITA, … }`, `reckonings.md` §3),
with `sunrise: TEXT` (mean sunrise at Ujjain carried by chara and udayantara, the centre on
the geometric horizon, KGG spashta vv. 7–9) and `planets: TEXT`.

## 4. Data model

### 4.1 The `KETAKI` catalogue member

```text
KETAKI = linear {
    epoch_jd_tt: 2407086.5,          // Mesha sankranti, Shaka 1800: 12 April 1878
    value_deg:   22.1425,            // 22°08′33″
    rate_deg_per_julian_year: 0.013952138,   // 50.228571″ per sidereal year
}
```

The member evaluates exactly as `CUSTOM` does (`astro-custom-ayanamsha.md` §4): the value
plus the rate times Julian years from the epoch. The rate is the text's 50.228571″ per
sidereal year converted with the text's own sidereal year, 365.256361 days (JG1937 p. 7):
0.013952138° = 50.228571 × 365.25 / 365.256361 / 3600. The member's doc comment holds the
derivation, so that the generated docs carry it.

**Continuous, not stepped (C2180).** The text gives the value at each year's Mesha
sankranti. A computer reading the karana keeps that value for the whole year. The modern
almanacs use the continuous value (the survey prints seconds for a year, not a step). The
member is continuous. The karana provider steps by year, as its text does.

### 4.2 The karana's data (phase K3)

The karana's constants are transcribed from the scan into `crates/siddhanta/data/ketaki/`
as typed Rust tables (no runtime parsing). Each entry carries its page and verse.

- the meridian's cities (vv. 6–7), the circumference (v. 8);
- the epoch mean places (*kṣepaka*, vv. 3–9) and the motion per 6940-day cycle
  (*dhruvaka*, vv. 10–14);
- the rational daily-motion rules (vv. 16–20) as exact fractions: `Ratio { num, den }`
  with arcminutes and arcseconds kept apart, so the karana's arithmetic is bit-exact
  integer arithmetic until the table look-up;
- the secular terms (vv. 21–27);
- the six tables (pp. 81–88): Sun's manda, its daily motion, the Moon's cyuti, tithi,
  manda and reduction, the planets' manda and radius vectors, at ten-degree steps.

Transcription is double-keyed. Two people (or one person twice, a week apart) key each
table from the page image, and a script diffs them. The Wikisource text layer is a third
reading and is never the source. The unlicensed `wall-clock` code is never read during
transcription (`CLEAN_ROOM.md`). C2185 records every disagreement.

## 5. Algorithms

**The modern profile.** No new algorithm. The engine gives tropical positions. The
`KETAKI` member gives the ayanamsha. The root's panchang, calendar and festival code read
them. That is the point of a profile.

**The karana provider.** Implements the ephemeris port as `SiddhantaProvider` does, with
`Astronomy::Classical` and `ayanamshas: [KETAKI]` (C2192):

1. Ahargana from Chaitra shukla 1, Shaka 1800 at Ujjain mean sunrise, in whole days, split
   into 6940-day cycles and a remainder (madhyama vv. 1–2).
2. Mean places: kshepaka + cycles × dhruvaka + the remainder rule, then the secular terms.
3. Sun: manda equation by table and linear interpolation.
4. Moon: cyuti, tithi correction, manda, the annual equation, the reduction to the
   ecliptic, in the text's order.
5. Planets: heliocentric mean place, manda, radius vectors, the geocentric rule.
6. The place's time: the longitude difference from the Ujjain line, the ascensional
   difference from the noon shadow, the equation of time. This is the provider's `TEXT`
   sunrise, and the provider answers sunrise through it (C2190).
7. Instants between sunrises by linear motion, as the text does (p. 22). The solver's
   root-finding over the provider reproduces this exactly when the provider presents
   linear motion within a day (the same arrangement the Surya provider uses).

The provider declares `deterministic: true`, `speeds: true` (the table daily motions),
`distance_unit: MeanDistances` and its `jd_range`: Shaka 1700 to 2200 (the text's
secular-term example reaches 2200; C2193).

## 6. API

No new API for the modern layer: `Context::builder().profile("ketaki")` over any engine.
The bindings and the MCP list the profile and the member from the generated catalogues,
with no per-binding code.

The karana is reached through the framework's builder
`Context::builder().reckoning("ketaki-grahaganita", modern)` (`reckonings.md` §3.4) and
through the classical chain entry. Until the framework lands, it is not reachable, and the
karana phases wait for it.

## 7. Errors

Nothing new for the modern layer. For the karana:

- an instant outside `jd_range` is the port's existing out-of-range refusal;
- a request for a body the text does not give (Uranus, Neptune, Pluto) is the
  existing unsupported-body refusal, with the hint to open a modern entry after it in
  the chain (the `ModernParts` arrangement);
- `frame.ayanamsha` other than `KETAKI` with the karana's planets is the existing
  `classical-provider-drik-settings` warning, generalised (`reckonings.md` R4).

## 8. Performance budget

The member is two multiplications. The karana is cheaper than the Surya provider (table
look-ups, no sines). Budgets go in the existing bench sections: `ayanamsha/ketaki` and
`provider/ketaki_grahaganita`, each within 10% of the Surya provider's counts.

## 9. Tests and acceptance

| test | asserts | tolerance | source |
|---|---|---|---|
| T1 member equals CUSTOM | `KETAKI` at 200 instants 1800–2400 equals `CUSTOM{…}` with §4.1's numbers | bit-identical | research §2.5 |
| T2 the text's example | Mesha, Shaka 1815 → 22°21′06″ | 0.5″ | KGG p. 12 (A1) |
| T3 Ketkar's epoch and checks | 22°08′33.04″ at Mesha 1878; 22°13′36″ at Chaitra shukla 15, Shaka 1806; 22°20′02.9″ on 1 January 1892 | 0.5″, 2.5″, 1″ | JG1937 pp. 7, 296, 346 (A2) |
| T4 the 1955 survey | 23°12′06″ to 23°12′10″ at Mesha, Shaka 1876 | inside the range ±1″ | RCRC (A3) |
| T5 the gap to Lahiri | Ketaki − Lahiri in −52″ to −67″ over 1878–2100, and Ketaki − (Spica − 180°, mean, geometric) within ±6″ | as stated | research §2.4, measured with the SDK's own star catalogue |
| T6 profile round trip | `ketaki` resolves, serialises and hashes stably; every knob in §3 cites | exact | settings tests |
| T7 karana step by step | the worked example of Shaka 1815 Chaitra shukla 15 (ahargana 5477; each intermediate the text prints; Sun 11s 19°8′11″ at Ujjain mean sunrise) | the text's last printed digit | KGG pp. 3–21 (A4) |
| T8 karana panchanga | the vyatipata of Shaka 1805 Shravana krishna 4 at Bagalkot: 45.7, 53.0, 0.3 ghatis | 0.1 ghati | KGG pp. 84–86 (A5) |
| T9 karana against the sky | the karana's Sun and Moon against the modern engine, 1878–2100 | Sun within 1′, Moon within 5′ (to measure; the claim in the text's preface) | a measured page |
| T10 Tilak against Ketaki | Shaka 1848 adhika masa under `tilak-revati` and the Grahalaghava (when it exists); Ganesh Chaturthi 2015 on 18 August under `tilak-revati`, 17 September under `ketaki` | the day | PAM (A7), Lokmat (A8) |
| T11 almanac pages | a printed Ketaki almanac's tithi, nakshatra and sankranti times | 1 minute | not found yet (C2188) |

T1 to T6 gate phase K1. T7 and T8 gate the karana. T9 is a measured page
(`ketaki-measured.md`, like the Surya Siddhanta's). T11 waits on C2188.

The conformance corpus gains a `ketaki` profile case set in phase K2: the corpus's existing
place-and-date grid computed under `ketaki`, recorded by the SDK and read back elsewhere
(`read-the-answer-back-elsewhere`). Its value is the regression guard, not an external
check, until T11's pages exist.

## 10. Phases and estimates

| phase | contents | estimate | depends on |
|---|---|---|---|
| K1 | the `KETAKI` member, the `ketaki` profile, T1–T6, the generated catalogues and docs | 2–3 days | SR-112 merged (for T1's CUSTOM) |
| K2 | `ketaki-measured.md`: Ketaki against Lahiri over 1900–2100 (sankranti shifts, nakshatra ends, adhika flips, C2187); the corpus case set; `tilak-revati` with T10 | 2–3 days | K1 |
| K3 | transcribe the karana from the page images, double-keyed; resolve C2185 | 3–4 days | the page images (in hand) |
| K4 | the karana provider, T7–T9, the `ketaki-grahaganita` profile | 5–7 days | K3 and the framework's F1–F4 (`reckonings.md` §6) |
| K5 | almanac pages (C2188) and the Date Panchang question (C2188, C2186): acquire, compare, record | open, 1–2 days of work once pages exist | a library or the publisher |
| K6 | *Jyotirgaṇitam*'s tables as a provider | 3–4 weeks; deferred until a consumer needs a pre-1950 almanac reproduced | — |

K1 and K2 are independent of the framework and can ship first.

## 11. Risks

| risk | likelihood | effect | mitigation |
|---|---|---|---|
| the consumer meant the Tilak (Revati) panchang when it wrote "Ketaki" | medium | the wrong 4° | ship both profiles and say which is which in the reply row; T10 shows the four-day difference |
| today's "Ketaki" almanacs use Lahiri, not Ketkar's value | medium (rank 3 says Date uses Lahiri) | the member differs from what they print by 1′, about 24 minutes at a sankranti | K5 settles it; until then the profile names its value and the docs give the gap (T5) |
| a transcription error in the karana | medium | a wrong place in a historical layer | double-keying, T7's step-by-step example, T9's bound |
| the karana's limb and refraction differ from the modern profile's | certain | sunrise minutes differ between the two profiles | intended; each profile states its sunrise (C2189) |
| festival dates in Maharashtra follow digest rules the SDK does not encode | high | a festival a day off in edge years | C2191; out of scope here |

## 12. Open questions

C2180 to C2199 in `01-research/feature-universe/19-verification-cruxes.md`. The ones that
gate a phase: C2185 (K4), C2188 (K5), C2192 (K4, the kit), C2189 (whether the profile's
sunrise stays the root's).
