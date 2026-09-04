# Products surveyed

Status: `research`, 2026-09-04. Each entry records what was read, what the
product claims, and what it means for the SDK.

## Jagannatha Hora (JHora)

Source: the features page at vedicastrologer.org/jh/features.htm (fetched
2026-09-04), the Grokipedia summary and third-party guides.

Free Windows software by P.V.R. Narasimha Rao; the de facto reference for
Vedic computational depth. Claims: date range 5400 BCE to 5400 CE; drik and
Surya Siddhanta models; 23 divisional charts with named variants (six horas,
four drekkanas, three navamshas, three trimshamshas, two each of D-4, D-5,
D-8, D-11, D-81, D-108) plus generic D-N to 300 and D-m of D-n; nine planets
plus three outer, Gulika, Mandi, nine upagrahas, eleven-plus lagnas, Bhrigu
bindu, eleven arudhas, Chandra and Surya arudhas, graha arudhas, Kunda,
Yogi points, sphutas; taras and special taras, latta, nakshatra aspects,
special tithis; chara karakas with seven and eight schemes; Brahma, Rudra,
Maheshwara; 36 sahamas; mrityu and pushkara bhagas; BAV, sodhita, PAV,
sodhya pindas; seven ayanamshas plus custom; topocentric; true and mean
nodes; three sunrise definitions; Shadbala, ishta and kashta, Vimshopaka
over four sets, Vaiseshikamsas, five avastha families, Tajika balas; 184
yoga types; around 45 dasha systems including conditional ones, all rashi
dashas over all vargas, levels to deha-antardasha, dasha pravesh charts;
transits from four references with tara and murthi classification, vedha,
Ashtakavarga and kakshya scoring, transit search; eight chakras; Tajika
charts at six resolutions plus tithi, yoga and nakshatra pravesha; mundane
charts (ingresses, lunations, conjunctions, varga change, eclipses,
swearing-in with compression); prashna by 108, 249 and 1800 numbers; KP with
five levels; daily and monthly panchanga; ten Indian languages plus English.

For the SDK: the completeness bar for areas E, I, D and K. PyJHora
re-implements most of it in Python with 5,600 tests, which is the best
available cross-check.

## Parashara's Light 9 (GeoVision)

Source: parashara.com and parasharaslight.com pages via search (the edition
table page did not load).

Commercial; Windows, macOS, iOS. Claims 5,000 calculations, 300 worksheets
with 100 customizable, interpretive reports, point-and-click
interpretations, options for true or mean nodes, karakas with or without
Rahu, sunrise by centre or edge, a world atlas, Anka Jyotish numerology and
Vastu companions, many Indian languages. Known from general use to cover
shodashavarga, Ashtakavarga, Shadbala, KP, Jaimini, Tajika, muhurta,
prashna, rectification, compatibility and around ten dashas.

For the SDK: the reference for interpretation reports and the
astrologer-facing option set (sunrise convention, karaka schemes). Also the
name the baseline engine was built to replace.

## Kala (Ernst Wilhelm)

Source: vedic-astrology.net/kala pages.

Commercial Windows software focused on the essential Parashara and Jaimini
calculations. Claims: Jaimini Sutras support, Tajika, prashna module,
muhurta module and helper, compatibility module, yoga judgment screen and
yoga searches, Ashtakavarga, avasthas, Shadbala, Sudarshana chakra,
"Transits Hit List" with exact timing, transits calendar, interpretive
reports, PDF printing; English, German, Russian, Spanish, Hungarian; version
Kala 2023.

For the SDK: the model for the transit hit list as a first-class feature and
for Jaimini presentation; the second tool the baseline engine replaces.

## Shri Jyoti Star 10

Source: vedicsoftware.com and vendor pages.

Commercial Windows software claiming the most dashas and highest accuracy,
election tools, prashna by KP and Prashna Marga, rectification,
astromapping, oracles, a custom yoga module with search and time search,
Parashara, Jaimini, Tajika, KP, Systems Approach and Western astrology,
custom report building, multi-device sync.

For the SDK: the custom yoga builder is the strongest argument for a
consumer-authored rule pack format, and the "time search" (when does a
yoga form by transit) maps to the rule engine over a positions grid.

## Sky Vision (Nepal)

Source: software listing sites; no vendor feature page found.

Nepali Windows software: kundali generation, predictions, remedies (gems,
numbers, colours), life event insights, compatibility, Nepali calendar
integration. The third tool the baseline engine replaces; its distinctive value is the
Nepali calendar and language, which the SDK makes a locale and calendar
pack.

## Astro-Vision

Source: indianastrologysoftware.com.

Indian commercial suite (LifeSign, SoulMate, StarClock, GemFinder, DigiTell,
NameFinder, Panchapakshi, Ming Sign, marriage-bureau tooling) in ten Indian
languages, with Gun Milan, Kerala and Tamil matching methods, muhurta,
panchang, Jaimini, yogas, remedies and Vastu. For the SDK: regional matching
variants and the breadth of Indian-language localisation to plan for.

## Maitreya

Source: saravali.github.io.

Free, cross-platform (wxWidgets). Vedic and Western chart styles, dashas in
text and graphical views, Ashtakavarga, Shadbala, yogas with predictions
from classical texts, solar charts, transits and progressions, solar arc,
Uranian astrology, partner charts, ephemeris, eclipses, hora. For the SDK:
the one open-source tool spanning Vedic and Uranian, useful as a second
cross-check for solar arc and Uranian points.

## Solar Fire 9 (Astrolabe)

Source: alabe.com/solarfireV9.html (fetched in full).

The Western reference: 30 house systems, 26 predefined aspects with full orb
control, 50 standard points plus user points, 1,081 extra asteroids, 290
fixed stars with parans, secondary, tertiary and minor progressions,
solar-arc and primary directions with several keys, every kind of return,
firdaria, profections, zodiacal releasing, primary directions, Lilly horary
with considerations, dignity tables and almutens, Arabic parts editor with
100 parts, midpoints and dials, harmonics, synastry and composites to 15
people, astro-mapping with parans and local space, eclipse paths, dynamic
hit lists and TimeMap, graphic ephemeris, void-of-course Moon (two methods),
Ashtkoot matching and Vedic divisional charts. For the SDK: the complete
specification of the Western module family (`15-western-modern.md`).

## Delphic Oracle (Zoidiasoft)

Source: astrology-x-files.com pages via search.

The Hellenistic reference: time-lord systems (zodiacal releasing from Fortune
and Spirit, profections, circumambulations, decennials), Greek lots and
Arabic parts, time wheels, transit animator, graphic ephemeris, based on
Project Hindsight translations of Valens, Antiochus, Hephaistio, Dorotheus,
Porphyry and Rhetorius. For the SDK: the specification of
`16-hellenistic-medieval.md`.

## PyJHora

Source: the GitHub README (fetched in full).

Open-source Python re-implementation of JHora's scope with about 5,600
tests: exhaustive panchanga including regional muhurtas and yogas, 22 vargas
plus custom and mixed, 60-plus dashas in graha, rashi, annual and other
families with tribhagi and year-length variants, about 284 yogas, eight
doshas, a long list of balas, Jaimini sphutas and karakas, KP levels,
Ashta Koota and 10 porutham, Tajika with the yoga set and sahamas, 13
chakras, transits, experimental rectification, a Vedic calendar with
festivals and vratas, six languages. For the SDK: the primary open
reference implementation for golden vectors beyond the baseline engine, and the best
enumeration of variants to name.

## VedAstro

Source: vedastro.github.io.

Open-source .NET platform with a 706-method REST API, Python library,
MCP server, classical-text RAG and predictions; dasas to eight levels,
Ashtakavarga, panchang, yogas, matching, muhurtha, transits. For the SDK:
the closest thing to a developer-facing astrology API today, and evidence
that a flat method catalogue without settings snapshots or batch shapes is
what developers currently have to work with.

## jyotishganit

Source: GitHub via search.

Python library on Skyfield with JPL DE421: D1 to D60, panchanga, Shadbala,
Vimshottari. For the SDK: an example of a non-Swiss ephemeris backend,
relevant to the port's capability model.

## The baseline engine

See `../baseline-engine/`.
