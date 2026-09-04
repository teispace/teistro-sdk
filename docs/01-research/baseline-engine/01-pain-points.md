# baseline engine: pain points and defects the SDK must not inherit

Status: `research`, 2026-09-04. Observed during the full read; each item
names where it lives so it can be verified, and what the SDK does instead.

## Structural

| pain | where | what the SDK does instead |
|---|---|---|
| Internationalisation is threaded through computation: `EntityNames` blocks (four languages) ride inside engine results, serialisers carry name resolution, and locale is applied in several places (engine registry, backend projection services, serializers) with different rules | `core` registry, `chart` results, backend `panchanga-day.serializer` (1,410 lines), `full-chart-localizer.util`, `section-locale.resolver` | engine emits keys; one localisation layer with data packs; formatting at the edge only (Principle 2) |
| Separation of concerns is thin: packages are separate but backend feature services contain astrology (day-level Chandrabalam and invented ghati timings in `balam.calculator.ts`, samvat offsets in `date-conversion.service.ts`, anga-span recomputation in `anga-span.util.ts`) | `src/modules/panchanga/services/` | every computation lives in the SDK; the application only orchestrates, caches and projects |
| Node-only: the engine can only be used from Node; Flutter and web clients cannot compute on device | whole `packages/` | one core, generated bindings |
| Global ephemeris state: twelve `set_sid_mode` and four `set_topo` call sites, no `await` allowed between setters and `calc_ut`, threading impossible | `core/node` | provider port with request-scoped frames; no global state |
| Duplicate projections of the same thing (`BirthData` built in three places: chart compute, milan compute, birth-data mapper) | backend | one input model in the SDK |
| Two ayanamsha wire vocabularies (`suryasiddhanta` versus `surya-siddhanta`) needing alias maps | `chart-compute.service.ts`, `milan.service.ts` | one key catalogue |
| Rashifal pinned to Kathmandu by design; location is not part of the model | rashifal orchestration | location is an explicit input to the SDK; the pinning becomes an application choice |

## Correctness findings

| defect | where | evidence | SDK consequence |
|---|---|---|---|
| Bhava-Chalit cusps are Vehlow (equal from the midpoint), not Sripati, while documented as Sripati | `chart` houses, `docs/house-system-policy.md` open finding | measured by the baseline engine team | named variants: `bhava-chalit.sripati`, `bhava-chalit.vehlow`, `bhava-chalit.porphyry`; a profile chooses |
| Samvat era numbers in date conversion: Kali Yuga as BS + 1049 (3130 for BS 2081, should be about 5125) and Nepal Sambat as BS − 1173 (908, should be about 1144); the unit spec asserts the same wrong offsets | `src/modules/panchanga/services/date-conversion.service.ts` and its spec | the engine's own `calendar.service.ts` gives AD + 3101 for Kali | eras are computed in one calendar module with sankranti-aware rollover, tested against published almanacs |
| Cached panchanga samvat numbers are year-precise only (fixed offsets from the AD year) | `panchanga-api.service.ts` `deriveSamvatNumbers` | comment admits it | same |
| Day-level Chandrabalam approximates the weekday-lord sign with the Sun sign; bhagyat, srudhakat and bhumika markers multiply indices by 38 or 37 ghatis without a source | `balam.calculator.ts` | comments admit both | classical definitions or nothing; every table cited |
| KP computed under the caller's ayanamsha with only a warning; sub-lords are arcminute-sensitive | `kp-chart.service.ts` | code | KP profile binds the KP ayanamsha; override is explicit |
| Tithi and other limb boundaries found to a tolerance of 0.001° (up to 7.6 s); Teimeris's harness showed 0.0000 s disagreement at 1e-9 | `panchanga-timing.service.ts` (from Teimeris's `PLAN_MIGRATION.md` step 0.3) | measured by Teimeris | root finders converge to a stated tolerance in time, gated |
| Retrograde detection by finite difference (two extra ephemeris calls) rather than the returned speed | `transit.service.ts` (Teimeris step 0.2) | measured | speeds are part of every position |
| Rectification computes nine planets per one-minute candidate and reads only the ascendant | `kp-rectification.service.ts` (Teimeris step 0.1) | 4.5x measured | candidate stages request only what they need through batch house calls |
| Per-chart ephemeris call count: 703 calls, 593 of them one scalar method | Teimeris `PLAN_MIGRATION.md` | measured | batch grids; the ephemeris is 8.2% of a chart, so the SDK's own loops matter more than the backend |

## Process findings

- Some claims in code comments are not measured (the ghati markers, the
  Chandrabalam approximation). The SDK adopts Teimeris's "measure, do not
  assert" rule and gates every documented number.
- Interpretation text and rule data are the most valuable assets and are
  well structured (citations, four languages); they should be exported
  mechanically into the SDK's pack formats, not rewritten.
