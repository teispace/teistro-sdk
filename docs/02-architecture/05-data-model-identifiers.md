# Data model and identifiers

Status: `draft`, 2026-09-04. Depends on Q13 (numeric policy).

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
| dasha year length | 365.25, 360, sidereal, lunar |
| dasha depth defaults | per system |
| chara karaka scheme | 7, 8 |
| ekadhipatya method | classical, zero, transfer |
| Rahu and Ketu aspects | none, 5/7/9, 3/7/11 |
| combustion orb table | named table |
| lunar month system | amanta, purnimanta |
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
    sdk_version, module_versions, profile_id, settings_hash,
    provider: { name, version, data_version, precision, flags_used },
    packs: [ {id, version} ],
    fallbacks_used: [ ... ],
    warnings: [ {code, key, slots} ],
    content_hash
  }
}
```

Warnings are keys with slots, rendered by `intl` when needed.

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

Decided as ADR-0011: `f64` throughout with the hygiene that delivers its
full precision (split Julian days, one normalisation routine, compensated
summation where measured necessary, convergence with caps, no fast-math),
and an explicit rounding contract applied only at serialisation:
longitudes to 1e-9 degrees, instants to the millisecond, scores to the
stated decimals per field. No decimal arithmetic in the core. All
comparisons through tolerance-aware helpers; tolerances are part of the
results schema.

## Errors

A closed enum of statuses (`INVALID_ARG`, `OUT_OF_RANGE`, `CAPABILITY`,
`PROVIDER`, `NOT_CONVERGED`, `UNSUPPORTED`, `PACK`, `LIMIT`, `INTERNAL`) with
a detail code, a message in English naming the field and range, and
optionally a message key with slots for localisation. Messages are never
produced on success.
