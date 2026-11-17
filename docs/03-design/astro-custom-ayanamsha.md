# Astronomy: custom ayanamshas beyond a linear rate

Status: `built`, 2026-11-17 (SR-112, cruxes C2160 to C2166). This page derives from
`03-design/astro-ayanamsha-catalogue.md` (its construction, its fitted-model correction,
mean and true, and its §10.3 open question), `03-design/astro-star-table.md` (the star
table an anchor is read from) and `03-design/settings-and-profiles.md` (the
`frame.ayanamsha` knob). It answers the consumer's SR-112 (addendum 8, §3): a value at
an epoch carried by a named precession model, a star held at a sidereal longitude, and
the basis stated per ayanamsha. The acceptance is that an epoch-plus-precession
definition equal to Lahiri reproduces Lahiri to 0.01″ over 1800 to 2200.

## 1. Purpose and scope

Until SR-112, `AyanamshaChoice::Custom` was the only definition a consumer could write.
It is a value at an epoch plus a linear yearly rate. A linear rate drifts from the
precession. The general precession's `t²` term (1.105″ a century squared in P03's
`p_A`) alone puts a rate fixed at J2000 4.4″ out by 1800 and by 2200. Most schools define their zodiac in one of two other ways: by an epoch and the
value there, carried by precession, or by a star at a sidereal longitude. The catalogue
already computes both kinds for its own members. This page lets a settings document
name its own of either kind. The linear form stays as it was, so a stored document
reads back unchanged.

## 2. The forms

`frame.ayanamsha` (and a lagna table's `SPHERE` zodiac) takes one of four tagged
objects:

| `kind` | fields | meaning |
|---|---|---|
| `CATALOGUED` | `id` | a member of the catalogue |
| `CUSTOM` | `epoch_jd_tt`, `value_deg`, `rate_deg_per_year` | the value plus the rate times the Julian years since the epoch (unchanged) |
| `PRECESSED` | `epoch_jd_tt`, `value_deg`, `basis` (`MEAN`, `TRUE`), `precession` (`VONDRAK_2011`, `IAU_2006`, `IAU_1976`, `NEWCOMB`) | the value at the epoch carried to the date by the named model (§3) |
| `ANCHORED` | `star` (a key of the star table), `longitude_deg` | the star's apparent longitude of date less the longitude it is held at (§4) |

```json
{"frame": {"ayanamsha": {"kind": "PRECESSED", "epoch_jd_tt": 2435553.5,
  "value_deg": 23.250182778, "basis": "TRUE", "precession": "IAU_2006"}}}
{"frame": {"ayanamsha": {"kind": "ANCHORED", "star": "SPICA", "longitude_deg": 180.0}}}
```

The form is part of the settings document, so it is part of the hash and reaches every
surface that takes a settings patch: Rust, the C ABI, Node, Python, Dart, Java, wasm
and the MCP. A chart under a custom form reports `ayanamsha_kind` 2 (`ayanamshaCustom`
in the bindings), and the settings hash pins which definition it was. Adding the two
variants moves no hash, because no default changes.

## 3. A value carried by a named precession (C2160, C2161)

**The construction** is the one every catalogued epoch member takes, and the one the
Swiss Ephemeris's `SE_SIDM_USER` takes (Teimeris's `TM_AYANAMSHA_USER`,
`core/src/tm_ayanamsa.c`). The vernal point of the date, a unit vector along x in the
mean equator of date, is carried back to J2000.0 and on to the epoch, rotated onto the
ecliptic of the epoch by the model's own mean obliquity there, and read as a longitude
λ. The value is `value_at_epoch − λ`. The named model carries it, whatever
`frame.precession` says for the rest of the sky, and no fitted-model correction is
applied. A user definition has none in Swiss either.

**The models** are the four the SDK implements: Vondrák, Capitaine and Wallace (2011,
*A&A* 534, A22); IAU 2006, i.e. P03 (Capitaine, Wallace and Chapront 2003, *A&A* 412,
567); IAU 1976 (Lieske et al. 1977, *A&A* 58, 1); and Newcomb's in Kinoshita's form. A
model a later build adds but this one lacks is refused as `UNSUPPORTED` on
`frame.ayanamsha`.

**Why not the `p_A` polynomial.** P03's general precession in longitude,
`p_A = 5028.796195 t + 1.1054348 t² + 0.00007964 t³ − 0.000023857 t⁴ − 0.0000000383 t⁵`
arcseconds (t in Julian centuries of TT from J2000.0), is measured on the moving
ecliptic. The construction above is measured on the ecliptic of the epoch, the frame
every published ayanamsha table was computed in. A Lahiri carried by `p_A` stands
0.0009″ from the same value carried by the construction under Vondrák 2011 over 1800 to
2200 (worst −0.13127″ against −0.13036″ from the catalogued Lahiri). The construction
is kept because it is the definition the catalogue, Swiss and Teimeris share (C2160).

**The basis of the stated value** (C2161). A value at an epoch is quoted either with or
without the epoch's nutation in longitude. The Indian Astronomical Ephemeris's
23°15′00.658″ for 1956 March 21 is a true value. `basis` says which. A `TRUE` value is
carried as `value − Δψ(epoch)`, using IAU 2000B (`nut00b`), the series the SDK's true
basis takes. The result is then subtracted on `frame.ayanamsha_basis`, as every
ayanamsha is: the basis the number was stated on and the basis it is applied on are two
different questions. A per-ayanamsha output basis was considered and not taken. One
zodiac is read on one basis per settings value, and the SR-118 warning
(`ayanamsha-mean-basis`) already guards the pairing.

## 4. A star held at a longitude (C2162)

The star is read from the star table as the catalogue's own anchored members read it
(`astro-star-table.md` §4): its apparent longitude on the mean ecliptic of date
(aberration and light deflection applied, no nutation), placed by the context's
precession model. A fixed direction (`StarClass::DIRECTION`, the two galactic poles) is
read by its geometric longitude, because a construction of planes has no light to
aberrate. The value is `longitude − longitude_deg`.

The anchor needs no stated basis. The nutation in longitude moves the equinox along the
ecliptic and so shifts every ecliptic longitude by the same Δψ. Holding the star at 180°
of true longitude and holding it at 180° of mean longitude give the same mean
ayanamsha, and on the true basis Spica's sidereal true longitude is 180° exactly. A
definition by right ascension (Wilhelm's galactic centre) is not expressible as a held
longitude, so it stays a catalogue member.

## 5. Measured

`crates/astro/tests/custom_ayanamsha.rs`, every 10 days from 1800-01-01 to 2200-01-01
in TT, on both bases:

| definition | against | worst |
|---|---|---:|
| `PRECESSED` at 1956-03-21 0h TT, 23.245560954594648° (the catalogued Lahiri there under Vondrák 2011), `MEAN`, `IAU_2006` | `LAHIRI` under the root's Vondrák 2011 | **0.0012″** (SR-112's bound 0.01″) |
| the same carried by `VONDRAK_2011` | the same | 0 (to the bit) |
| the IAE's 23.250182778°, `TRUE`, `IAU_1976` | `LAHIRI` under IAU 1976 | −0.00779″, constant: IAU 2000B's Δψ at the epoch (16.7767″) against the 16.7689″ Swiss subtracts (its IAU 1980 series) |
| 23.250182778° − 0.004658035°, `MEAN`, `IAU_1976` (Swiss's own statement) | `LAHIRI` under IAU 1976 | 0 |
| the IAE's number, `TRUE`, `IAU_2006` | `LAHIRI` under Vondrák 2011 | −0.138″, constant: the catalogue's IAU 1976 fitted correction (0.130″) and the series (0.008″) |
| `PRECESSED` restating `USHASHASHI` under each model | `USHASHASHI` under that model | 0 (to the bit; the construction the Teimeris table holds to 1e-7″) |
| `ANCHORED` restating each of the 11 longitude-anchored members | the member | 0, or 1e-9″ where the author wrote two terms |

Through the façade (`crates/sdk/tests/custom_ayanamsha.rs`, `nepali-drik`, Kathmandu):
the precessed Lahiri's chart zodiac stands 0.000045″ from `LAHIRI`, and its Mesha
sankranti of April 2026 0.0006 s. Spica held at 180° gives True Chitra's chart and its
sankranti to the bit.

## 6. The solar months (C2164)

The solar model (`DrikSun`) asks the port for a sidereal frame, and the port's frame
names only catalogue members. Until SR-112 the façade's `drik_sun` put Lahiri in place
of any custom definition. The chart therefore took the custom zodiac while the
sankrantis, the solar months and the lunar month's adhika test kept Lahiri's. For Spica
at 180° that would have moved April 2026's Mesha sankranti by 913.6 s. `DrikSun` now
takes the choice (`with_ayanamsha_choice`): a catalogued member goes into the port's
frame as before, and a custom one asks for the tropical Sun and subtracts the SDK's
value at the instant in TT, on the model's basis and precession, as the completion's
own sidereal step does.

## 7. Errors

| situation | outcome |
|---|---|
| a non-finite epoch, value, rate or held longitude, in the frame or in a lagna table's zodiac | settings error `custom-ayanamsha-finite` on `frame.ayanamsha` or `panchanga.lagna_table`; the computation refuses it `INVALID_ARG` too |
| a star key the table does not hold | the settings document is refused on `frame.ayanamsha` |
| a precession model this build does not implement | `UNSUPPORTED` on `frame.ayanamsha` |
| an instant the star table cannot place | the star table's error |

## 8. Cost

A precessed value costs what a catalogued epoch member costs (0.58 µs under Vondrák
2011, `astro-ayanamsha-catalogue.md` §7). Its epoch's matrices are kept per thread as
the catalogue's are. A held star costs a star place, as True Chitra does. No new entry
point: the forms ride on every request that takes settings, so no cost record moves.

## 9. Open questions

1. **Ketaki** (C2165). The brief (§6) reports Ketkar's *Grahaganitam* ayanamsha as
   Revati-based, not verified. `ANCHORED` with `REVATI` (ζ Piscium) expresses it once
   a source gives the longitude. No rank-1 or rank-2 source was found on 2026-11-17.
2. **Through the port** (C2166). The port's `ayanamsha_deg` names catalogue members
   only, so a custom form is always the SDK's value, as every catalogued member is under
   `sdk-only`. Teimeris implements `TM_AYANAMSHA_USER` (an epoch and value under its
   model), which would answer `PRECESSED` natively. Carrying a definition through the
   port is a port change for the boundary lanes, not for this page.
