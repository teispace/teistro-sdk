# Competitive matrix

Status: `research`, 2026-09-04. Sources are the product pages and README files
fetched on that date and listed in `01-products.md`. A cell says what the
source states; `?` means the source did not say. Counts are the products'
own claims and are not independently verified.

Legend: `●` present, `◐` partial or limited, `○` absent, `?` unknown.

| capability | baseline | JHora | PL 9 | Kala | SJS 10 | Sky Vision | Astro-Vision | Maitreya | Solar Fire 9 | Delphic Oracle | PyJHora | VedAstro |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| ephemeris | sweph via Node | Swiss | Swiss | Swiss | Swiss | ? | ? | Swiss | Swiss | Swiss | Swiss (pyswisseph) | Swiss |
| siddhanta option | ● drik, Surya | ● drik, Surya (Makaranda) | ○ | ○ | ○ | ? | ○ | ○ | ○ | ○ | ○ | ○ |
| ayanamshas | 47 | 7 + custom | many | many | many | ? | ? | many | Lahiri, KP, others | ? | ? | ? |
| house systems | 22 | 4 to 17 | ? | ? | ? | ? | ? | ? | 30 | ? | 17 | ? |
| vargas | standard set | 23 + custom D-N + mixed | shodasha | ● | ● | ● | shodasha | ● | Vedic divisional (basic) | ○ | 22 + custom | ● |
| dasha systems | 18 | ~45 | ~10 | Parashari + Jaimini core | "most" | ? | Vimshottari | ● | Vimshottari | ○ | 60+ | up to 8 levels |
| Shadbala, Ashtakavarga, Bhava bala | ● | ● | ● | ● | ● | ? | ● | ● | ○ | ○ | ● | ● |
| Vimshopaka, Vaiseshikamsa | ◐ | ● | ● | ? | ? | ? | ? | ? | ○ | ○ | ● | ? |
| yogas | 562 rules | 184 types | ● | shubha/ashubha, searches | custom builder + search | ● | ● | ● with text | configurations | ○ | ~284 | 8 methods |
| doshas | 62 rules | ● | ● | ● | ● | ● | ● | ? | ○ | ○ | 8 | ? |
| Jaimini | ◐ | ● | ● | ● (core focus) | ● | ? | ● | ? | ○ | ○ | ● | ? |
| KP | ● 4 levels | ● 5 levels | ● | ○ | ● | ? | ? | ? | ○ | ○ | ● | ? |
| Tajika | ◐ | ● (annual to 2-minute) | ● | ● | ● | ? | ? | ● (solar) | returns | ○ | ● | ? |
| panchanga | ● full day | ● daily and monthly | ● | ● | ● | ● Nepali | ● | ● | ○ | ○ | ● extensive | ● |
| muhurta search | ● | ◐ | ● | ● module + helper | ● election tools | ? | ● | ○ | electional searches | ○ | ◐ | ● (1 method) |
| prashna | ● | ● numbers | ● | ● module | ● KP + Prashna Marga | ? | ? | ○ | horary (Lilly) | ○ | ● | ? |
| matching | ● Ashta Koot + extended + Mangal | ○ | ● | ● | ● | ● | ● Gun Milan, Kerala, Tamil | partner charts | Ashtkoot + synastry | ○ | ● Ashta + 10 porutham | ● |
| transits | ● gochar, Sade Sati, calendar | ● from 4 references, vedha, tara, murthi, search | ● | ● hit list + calendar | ● | ● | ● | ● | ● dynamic reports, TimeMap | animator | ● | ● |
| rectification | ● Bayesian cascade | ○ | ● | ◐ | ● | ? | ? | ○ | astro-lines, events | ○ | ◐ experimental | ○ |
| longevity | ● Ayurdaya, Maraka, windows | ◐ | ● | ● three pairs | ? | ? | ? | ○ | hyleg variants | ○ | ? | ? |
| remedies, gems | ● | ○ | ● | ○ | ● | ● | ● (GemFinder, Parihara) | ○ | ○ | ○ | ○ | ● |
| numerology | ● | ○ | ● (Anka Jyotish) | ○ | ○ | ● | ● DigiTell | ○ | ○ | ○ | ○ | ● |
| Lal Kitab | ● | ○ | ● | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ |
| Pancha Pakshi | ● | ○ | ? | ○ | ○ | ? | ● | ○ | ○ | ○ | ● | ○ |
| chakras (Sudarshana, Sarvatobhadra, Kota ...) | Sudarshana | 8 | ● | Sudarshana | ● | ? | ? | ? | ○ | ○ | 13 | ? |
| mundane and pravesha charts | ○ | ● extensive | ◐ | ○ | ◐ | ○ | ○ | ○ | ingress, eclipse | ○ | ● | ○ |
| Western: aspects, progressions, directions, returns, synastry, fixed stars, mapping | ◐ synastry only | ○ | ○ | ○ | ● (Western listed) | ○ | ○ | ● Uranian, solar arc | ● the reference | ● traditional | ○ | ○ |
| Hellenistic time lords | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ZR, profections, firdaria | ● the reference | ○ | ○ |
| interpretation text | ● 4 languages, cited, composers | learning aids | ● reports | ● reports | ● custom reports | ● reports | ● reports | ● from texts | ● large libraries | ● reports | ○ | ● predictions |
| languages | ne, en, sa, hi | 10 Indian + en | many Indian | en, de, ru, es, hu | en | ne, en | 10 Indian | en, de, ... | en | en | en, ta, te, hi, kn, ml | en |
| calendars | AD, BS + eras | ? | ● Indian | ? | ? | ● Nepali | ● | ? | Gregorian | Gregorian | Vedic, Islamic | ? |
| platform | Node backend | Windows | Windows, macOS, iOS | Windows | Windows | Windows | Windows, mobile | cross-platform desktop | Windows | Windows, macOS | Python | .NET, Python, REST |
| API for developers | internal packages | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ● library | ● REST 706 methods, MCP |
| atlas and timezone | geo-tz + tzdb | 2.5M places | world atlas | ● | "best atlas" | ? | ? | ● | ACS atlas | ● | ○ | ○ |

## What the matrix says

1. **Depth belongs to JHora and PyJHora** (dashas, vargas, chakras, mundane
   charts). Any claim of completeness for a Vedic SDK is measured against
   JHora's feature list, and PyJHora is the open reference implementation to
   cross-check against.
2. **Breadth belongs to Solar Fire** for everything Western, and to Delphic
   Oracle for Hellenistic time lords. A general astrology SDK needs their
   feature families, which no Vedic tool has.
3. **The baseline engine is already ahead of the field** in three places: the rule
   corpus size (562 yogas versus JHora's 184 types), the rectification
   method (interval-first Bayesian cascade versus manual tools), and
   cited four-language interpretation with deterministic composers. These
   are the assets to preserve.
4. **Nobody offers an SDK across platforms.** VedAstro is the closest (REST
   and Python, 706 methods) but is one ephemeris, one language of text,
   .NET-centric and monolithic. The position is open.
5. **Product-level features that are not engine features** (worksheets,
   atlases, printing, chart art, databases) stay out of scope; the SDK
   provides the data those features render.
