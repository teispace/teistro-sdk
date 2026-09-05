# Data model and identifiers

Status: `draft`, revised 2026-09-05: the design pages
`03-design/core-types-and-catalogue.md` (kinds, keys, ids, the catalogue,
newtypes, the envelope and status codes) and
`03-design/settings-and-profiles.md` (the knobs, profiles, coherence and
the hash) settle what this page sketches (ADR-0016, ADR-0020, ADR-0023).

## Identifiers

- **Keys** are strings with namespaces (see the localisation page) and
  dense numeric ids per namespace for the ABI; both generated from one
  catalogue file (`catalogue/*.yaml`) that also carries attributes (a
  graha's exaltation sign and degree, a nakshatra's lord, span, gana,
  nadi, yoni, padas and navamsa signs, a tithi's deity). The catalogue is
  the successor to the baseline engine's `data/*.data.ts` files with names removed.
- **Stable forever**: a key or id is never reused or renumbered; deprecated
  keys stay resolvable with a deprecation flag.
- **Open sets** (custom vargas, consumer dasha systems, consumer rule keys)
  use a reserved id range and a registration handle.

## Settings

A `Settings` value is complete (every knob present) and is built by
applying overrides to a **profile**. Profiles are named, versioned records
shipped by the SDK (`nepali-default`, `kp-default`,
`western-tropical-default`) or defined by consumers. Knobs (initial list,
superset of the baseline engine's):

| knob | values |
|---|---|
| zodiac | tropical, sidereal |
| ayanamsha | catalogue id or custom (epoch, value, rate); mean or nutated |
| node type | mean, true |
| topocentric | on, off |
| siddhanta | drik, surya (with bija on/off) |
| house system for placements | one of the registered systems |
| house system for bhava bala and chalit | Sripati, Vehlow, Porphyry, KP |
| house system policy overrides per module (KP forces Placidus) | |
| sunrise convention | centre-no-refraction, upper-limb-refraction, custom altitude |
| dasha balance | spatial, temporal |
| dasha year length | per system from a named table: 365.25, savana 360, sidereal 365.2564, tropical 365.2422, lunar 354.367, nakshatra 324 (verify) |
| dasha depth defaults | per system |
| chara karaka scheme | 7, 8 |
| ekadhipatya method | classical, zero, transfer |
| Rahu and Ketu aspects | none, 5/7/9, 3/7/11 |
| combustion orb table | named table |
| lunar month system | amanta, purnimanta |
| polar policy for house systems undefined at the latitude | error, fallback-whole-sign, fallback-porphyry, clamp |
| day boundary for calendar dates | midnight, sunrise, sunset, noon (with the reference place) |
| convention for unattested divisional charts (arbitrary D-N) | cyclic (parivritti) default, or a named scheme |
| overflow for a seed outside a conditional dasha's cycle | wrap-to-start (flagged), reject |
| calendar for civil dates | registered calendar id |
| DST policy | gap and overlap choices |
| precision and rounding contract | per output field family |
| limits | batch sizes, ranges, iteration caps, cache memory |

The settings value has a canonical serialisation and a hash; the hash is
part of every result and every cache key.

## Result envelope

```
Result<T> {
  value: T,
  provenance: {
    sdk_version, module_versions, calculation_version, profile_id, settings_hash, input_hash,
    provider: { name, version, data_version, data_hashes, tier, precision, flags_used },
    packs: [ {id, version, hash} ],
    time: { delta_t_model, leap_table_version, tzdb_version, time_basis_applied, time_uncertainty? },
    calendar: { resolution: tabular | computed | divergent { tabular, computed, followed } }?,
    deviation?: { model, from_drik },              // when a classical siddhanta answered
    applied_conventions: [ {knob, value, reason} ], // arbitrary D-N, seed overflow, unattested requests
    confidence: verified | unverified,
    fallbacks_used: [ ... ],
    warnings: [ {code, key, slots} ],
    content_hash
  }
}
```

Warnings are keys with slots, rendered by `intl` when needed.
`calculation_version` bumps on any change to numeric output for identical
input (ADR-0020); the cache key is `(input_hash, settings_hash,
calculation_version)`. Every field above is typed in every binding
(ADR-0023).

## Chart model

- `BirthData`: name key (opaque string, never used in computation), gender,
  instant, place (lat, lon, alt), zone resolution, time accuracy and
  uncertainty, birth order, civil date in the input calendar.
- `Foundation`: positions of every requested point in both frames
  (tropical and sidereal longitudes, latitude, distance, speeds,
  declination, RA), cusps for the placement system and for the chalit
  system, angles, sunrise window, the local day, the lunar month state,
  hora and abda lords, and a settings hash. Computed once and memoised.
- `Placement`: sign index, degree in sign, nakshatra and pada, navamsa
  sign, house by span and by whole sign, dignity, state flags.
- Slices are separate typed results keyed by the foundation's hash.
- `ChartKind` distinguishes natal, transit, event, prashna, return,
  relocated and composite so rules can be scoped.

## Numeric policy

Decided as ADR-0011 and amended by ADR-0016. Astronomy is `f64`
throughout with the hygiene that delivers its full precision (split
Julian days, one normalisation routine, compensated summation where
measured necessary, convergence with caps, no fast-math). Every angle
that becomes data is converted once, in `core::angle`, to a canonical
`i64` nanoarcsecond value (`Nas`); every classification (sign, nakshatra,
pada, varga part, KP sub, koota lookup) is exact integer arithmetic on
that value with half-open lower-inclusive boundaries; dasha spans are
exact rationals as fractions of the parent span, materialised to
instants only at presentation. Serialisation carries the canonical
integer (`lon_nas`) and the derived degrees; instants to the
millisecond; scores to the stated decimals per field. No decimal
arithmetic in the core. Comparisons between floating-point quantities go
through tolerance-aware helpers; tolerances are part of the results
schema. Design: `03-design/exact-arithmetic.md`.

## Errors

A closed enum of statuses (`INVALID_ARG`, `OUT_OF_RANGE`, `CAPABILITY`,
`PROVIDER`, `NOT_CONVERGED`, `UNSUPPORTED`, `PACK`, `LIMIT`, `SCHEMA_VERSION`,
`INTERNAL`) with a stable numeric code, a detail code (`UNSUPPORTED` carries
`unsourced` for a registered variant without an implementation,
ADR-0018), a message in English naming the field and range, and
optionally a message key with slots for localisation. Messages are never
produced on success. Degenerate astronomical outcomes are not errors:
they are typed states on the result (`undefined { reason }`), and the
conventions the SDK had to choose are listed in `applied_conventions`.
