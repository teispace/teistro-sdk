# The panchanga's reach without the muhurta family

Status: `built`, 2026-11-17.

Three asks of Tithipatra's addendum 10 share one cause. A consumer that
builds teistro without the `muhurta` family could not read three things the
almanac already knew: an eclipse's vedha (SR-132), the season's blackouts
(SR-138), and the Sun's and the Moon's longitudes as the day's limbs read
them (SR-137). Each rule now lives once, in `teistro-panchanga`, which every
build links. The muhurta search reads it from there and answers bit for bit
as before.

| Ask | Entry points | Record | C boundary |
|---|---|---|---|
| SR-132, the vedha | `almanac.eclipses` in every build; `teistro_panchanga::vedha::{vedha_of, with_vedhas}` | `almanac.eclipses` (unchanged) | `ts_almanac_eclipses` (unchanged) |
| SR-138, the blackouts | `AlmanacArea::blackouts`, `AlmanacArea::blackouts_request`; `teistro_panchanga::season::sky_blackouts` | `almanac.blackouts` | `ts_almanac_blackouts` |
| SR-137, the angles | `AlmanacArea::limb_angles`, `AlmanacArea::limb_angles_request`; `Almanac::limb_angles`, `limb::angles_at` | `almanac.limbAngles` | `ts_almanac_limb_angles` |

Each binding has both calls: `limb_angles` and `blackouts` in Python,
`limbAngles` and `blackouts` in Node, Dart, Java and wasm. The MCP server
has the tools `almanac.limbAngles` and `almanac.blackouts`.

## 1. One rule, one crate

The season (`months`, `month_of`, `blackouts`, the elongation constants)
and the vedha (`SeenEclipse`, `vedha_praharas`, `eclipse_vedha`,
`vedha_over_days`) moved from `teistro-muhurta`'s `season` module to two
new modules of `teistro-panchanga`: `season` and `vedha`. The code moved as
it was. `teistro-muhurta::season` re-exports what it moved, so its users
keep the same paths. Two things stay in muhurta because only an election
reads them:

- **Asta** carries an `AstaCriterion`, a muhurta setting.
- **The eclipse's star** needs the search's own eclipse list.

Two helpers are new:

- **`vedha::vedha_of`** reads the vedha off the days around an eclipse's
  first moment seen. It takes a day reader, so the muhurta search
  (`Sources::day`) and the almanac (`Almanac::day`) feed it the same days.
- **`vedha::with_vedhas`** fills each seen eclipse's `vedha`.
  `AlmanacArea::eclipses` calls it in every build.

Before this change, a build without `muhurta` answered `vedha: null`. Now it
answers the window, under `panchanga.eclipse_vedha`, as the full build did.

**Held.** Every test of `teistro-muhurta` passes unchanged. That includes
its pinned windows and its season tests, which compare whole answers.
`every_seen_eclipse_carries_its_vedha_in_every_build` passes under default
features and under `--no-default-features --features builtin-compact`.

## 2. The blackouts over a range

`almanac.blackouts` takes:

- `first`, and an optional `last` (the first day when left out), as
  `{year, month, day}` in `calendar` (Gregorian when left out);
- `utcOffsetSeconds` or `zone`, one of the two;
- `kinds`, the kinds wanted.

It answers `{kind, at}` for each blackout, in order of start, with `kind`
written as a full key (`blackout_kind.KHARMAS`).

- **Kinds.** A blackout is a kind the Sun and the Moon make: `CHATURMAS`,
  `ADHIKA_MASA`, `SAMSARPA`, `KSHAYA_MASA`, `KHARMAS`, `PITRU_PAKSHA`,
  `SANKRANTI` and `HOLASHTAKA` (`season::SKY_KINDS`). A request that leaves
  `kinds` out gets all eight.
- **Refused kinds.** Asta, the eclipse's star and the vedha are refused on
  `blackouts.kinds`, with a hint naming where each one is answered. Asta
  and the eclipse's star come from `almanac.elect`, and the vedha from
  `almanac.eclipses`. Leaving them out without a word would read as "none
  this year".
- **The range.** It runs from the first day's local midnight to the
  midnight after the last day. Each blackout is clipped to it, as the
  muhurta search clips its own. A range given the wrong way round is
  refused.
- **The sky.** The rule is `season::blackouts`, read in the profile's chart
  zodiac taken at the range's middle (`ChartZodiac::searched`), geocentric.
  This is how `Sources` reads the season for an election, so a ceremony
  page and an election name the same days. The provenance says so. It
  lists the conventions `blackouts.window` (the range's bounds) and
  `blackouts.zodiacAt` (the instant of the zodiac).
- **Held.** The malmas the call answers runs over exactly the days the
  almanac marks adhika, under drik and under the committee, April to July 2026.
- **Held.** Kharmas is three stretches in 2026 (the Sun in Dhanu, in
  Meena, then in Dhanu again).
- **Held.** Chaturmas ends at the start of a Shukla Ekadashi span.

## 3. The angles the limbs read

`almanac.limbAngles` takes 1 to 4,096 instants (UTC Julian days) and a
place. For each instant, in order, it answers
`{at, sunDeg, moonDeg, elongationDeg, yogaDeg}`.

The angles are read along the same code path as the day's limbs:

- One private helper, `Almanac::limb_sky`, builds the sky the almanac's
  eleven limb readers use: the frame and the search zodiac from
  `at_instant`, the frame longitudes, and the observer when the panchanga
  is topocentric.
- `limb::angles_at` reads each angle through the same quantities the limb
  search reads:
  - the Moon through `Limb::Nakshatra.quantity()`;
  - the elongation through `Limb::Tithi.quantity()`;
  - the sum through `Limb::Yoga.quantity()`;
  - the Sun through `Quantity::Longitude(Sun)`.

  The shift is `Sidereal::over`, the ayanamsha at each instant, exactly as
  the search applies it.

Every knob the limbs honour, the angles honour too, with no list to keep
in step:

- the ayanamsha, its basis (mean or true), and a custom ayanamsha;
- apparent or true positions;
- the panchanga's centre;
- a provider that defines its own zodiac (the Surya Siddhanta).

The frame and the search zodiac do not depend on the instant, so one call
builds them once.

**Held.** At every limb end the month's search found, the angle sits on
the limb's grid. The tests cover November 2026 and March 1900, under
`nepali-drik` and under `nepali-committee`, Kathmandu. The ends are the
tithis' and karanas' on the elongation, the nakshatras' on the Moon, and
the yogas' on the sum.

- The bound is the search's tolerance (`1e-7` days) times the fastest rate
  of the three angles, which is 1.6e-6°.
- The worst measured was 4.0e-9°, about 0.000015″. The two roads the
  consumer compared disagreed by up to 20″.
- The same holds under each of the knob patches: a mean and a true basis,
  true positions, a topocentric centre, and a custom ayanamsha.

## 4. A fix found on the way

`panchanga.centre: TOPOCENTRIC` failed every day with "a topocentric frame
needs the place the observer stands at". The Moon's signs were read in the
day's own frame, which carried no observer. The signs are geocentric by
convention, so they are now read in `limb::geocentric(frame)`. This is the
frame the season reads too. A shipped profile sets no topocentric centre,
so no shipped answer moves. The angle tests above run a topocentric
context, and they are what found the bug.

## 5. The feature flags, measured

Measured from `crates/sdk/Cargo.toml`, `bindings/wasm/native/Cargo.toml`
and `docs/05-testing/sizes.json`:

| Build | Families | Links `teistro-muhurta` | Read by the size gate |
|---|---|---|---|
| native library, addon, wheel, jar, C bundle | `default` = `builtin-standard`, `full`, `signed-packs` | yes | yes, the `library`, `addon`, `wheel`, `jar` and `c-bundle` rows |
| wasm `full` module | `builtin-compact`, `full`, `signed-packs` | yes | yes, budget 3,980,000 bytes gzipped |
| wasm `panchanga` module | `builtin-compact`, `muhurta` | yes | yes, budget 2,310,000 bytes gzipped |
| a consumer's build without the muhurta family | `builtin-*` and whichever families it names, not `muhurta` | no | no |

- **What `teistro-panchanga` brings.** It is a plain dependency of
  `teistro`, so every build links it. The season and the vedha are now in
  every build.
- **What `muhurta` adds.** The feature adds `teistro-muhurta`, which keeps
  asta, the eclipse's star and the search itself, and re-exports the rest
  from `teistro-panchanga`.
- **What `chart` and the other families add.** Nothing in this change.
- **The modules the gate reads.** All three already linked
  `teistro-muhurta`, so the rule code they carry is the same code as
  before, compiled once in `teistro-panchanga` instead of in
  `teistro-muhurta`. What each gains is the two records:
  - the request readers;
  - `Almanac::limb_angles` and `limb::angles_at`;
  - `AlmanacArea::blackouts`;
  - their FFI entry points and their glue.
- **The build that grows by the moved rules.** Only a build without the
  muhurta family, which no size row reads. That is the consumer's build,
  and what it gains is the point of SR-132 and SR-138.
- **Not measured.** The modules were not rebuilt locally for a byte count.
  CI's sizes job measures them against the budgets above, which carry 2%
  of headroom.
