# A chart founded on a classical astronomy

Status: **built**, 2026-09-26: every step of §7 the same day. Written from
`classical-chart-measured.md` before any code; the order of work (§7)
re-aims that pass at the built thing, and the building has already
corrected it four times (§3, the corrections; §4, the midnight; §5, no
vtable slot; §6, a call in place of a refusal).

Derives from `siddhanta.md` §5, which built the Surya Siddhanta as a
provider "so a classical chart runs through the same trait, completion
and solver as the engines", and from ADR-0013's override policy. STATUS
Next item 1 names what was left: the classical provider's Lagna and its
planetary hours, exposed through the chart layer.

## 1. What was measured

`classical-chart-measured.md` founds every recorded birth through the
SDK over `SiddhantaProvider::text()`, naming the text's ayanamsha, and
holds each part of the chart against the text's own answer. Every claim
that the chart is the text's is falsified:

- **The zodiac is the catalogue's, not the text's.** The chart subtracts
  the `SURYASIDDHANTA` catalogue member (zero in the year 499, carried by
  modern precession), about 1.6° from the text's own ayanamsha (III.9 to
  12), while the stamp says `ayanamsha:NATIVE` because the *positions*
  step did use the text's. Every graha moves with it.
- **The Lagna is the SDK's spherical ascendant**, up to 8° from the
  text's in the tropical frame alone and in another sign on 6 of 53
  births. The text's 24° obliquity does not close it: the text reckons
  the Lagna from the Sun at its own sunrise through the signs' rising
  times in proportion within each sign, on a clock with no equation of
  time (III.46 to 49).
- **The day is the SDK's solver run over the text's Sun**, up to 24
  minutes from the text's sunrise, in another hora on 6 of 53 births,
  although the provider declares its sunrise as an override.

So a chart over the text is a hybrid nobody asked for. The overrides the
provider declares reach the completion (positions, obliquity) and stop
there: the chart layer computes its zodiac, its angles and its day
itself. Nothing in the SDK outside `astro::visibility` and the kit calls
a provider's native horizon search, and the envelope's `deviation`
field, "a classical model's deviation, when one answered", has never
been filled.

## 2. The principle

**A classical provider's overrides are definitions, not approximations.**
`Astronomy::Classical` already says so at the port: its obliquity,
precession, daily motions and sunrise "are the text's own definitions",
which the kit measures against modern astronomy and publishes rather
than gates. A modern provider's overrides are the same sky computed
another way, gated to agree with the SDK's within a published bound; a
classical provider's are a different question with no SDK answer to
agree with. There is no SDK version of the text's sunrise.

So under the default `prefer-native` policy, a chart founded over a
classical provider takes each part the provider **defines** from the
provider: its zodiac, its day and its angles. `sdk-only` still gives
the SDK's parts (the hybrid, asked for by name and stamped), and
`native-only` refuses a part the provider does not define.

A **modern** provider's overrides at the chart layer are not decided
here (§8): for Teimeris they would move a sunrise by up to a minute at
height (`horizon-atmosphere.md` §6, the fixed 34′ column) and every
chart's zodiac by its native ayanamsha's distance from the SDK's, which
wants its own measurement before a default moves.

## 3. The zodiac and the places

`ChartZodiac::defined` asks the completion whether the provider defines
the chart's ayanamsha member (`Completion::defines_ayanamsha`: a
classical astronomy that lists it, under a policy that lets it answer)
and, where it does, takes the value and its rate from the provider;
`ChartZodiac::of` answers from the catalogue otherwise, as before. Over
the text naming `SURYASIDDHANTA`, the value is the text's; naming
Lahiri, which the text does not list, it is the SDK's Lahiri, as the
positions already are. The rate is the same central difference over the
provider's values, so a graha's speed is measured in the zodiac it is
placed in. The stamp gains a `zodiac` step, `NATIVE` or `SDK`, beside
the completion's own `zodiac-shift`, which is the shift out of the
provider's sidereal frame and stays the SDK's.

**Building it found a fourth hybrid part.** With the zodiac the text's,
the grahas still stood 0.008° (the Sun) and 0.078° (the Moon) from the
text's places: the completion corrected them to apparent places with
the SDK's Earth and the IAU's nutation, and took the light time from the
provider's distance, which for a classical text is a fraction of the
body's mean distance and was read as astronomical units — an 8.3-minute
light time for the Moon. A classical astronomy's place is its own
definition, so the corrections step now passes it through, stamped
`corrections:NATIVE`, and the step refuses any provider whose distances
are not in astronomical units rather than reading them as such. The
nine grahas are then the text's to the last bit.

## 4. The day

`DrikSun::day_light` asks the provider's native horizon search for the
Sun's rise and set when the provider defines them (§2) and the
convention is one it answers; the text answers the centre on the
geometric horizon, which is the root profile's convention. A
convention the text refuses falls back to the SDK's solver under
`prefer-native` and is stamped, as an unlisted ayanamsha does; under
`native-only` it is refused, naming the convention. Every part of the
day that reads the sunrise follows: the hours, the ishtakaal, the
day-or-night, the ghatis and the Lagna the arudhas measure from.

The text's own Sun, `sidereal_sun_deg`, is already the provider's
positions in the zodiac of §3.

Built as `DrikSun::defined_day`, over `Completion::defines(RISE_SET)`.
A day with no sunrise falls back to the solver as well, because the
port answers an event or its absence and not whether the Sun stayed up,
and the solver says which polar state it is. The model's description
says `by the provider` when the provider gave the day, which is what
the local day stamps.

**Building it found a half-second midnight.** With the day the
provider's, the sunrise still stood up to 0.48 s from the text's as the
classical solar model counts it. Both solar models found a civil day's
local mean midnight through `LocalMeanTime`'s offset, which a
`UtcOffset` holds in whole seconds, while the text's own search starts
from the exact midnight. `solar::local_mean_midnight` is now the one
exact midnight both models read; no calendar, lunisolar, panchanga or
almanac page moved.

## 5. The angles

**The port gains an `angles` method** under a new override,
`Overrides::ANGLES` (`1 << 12`): the ascendant and the midheaven at an
instant and a place, tropical degrees, with the obliquity they were
built on. `HOUSES` stays what its name says, a native set of cusps, for
a later engine that wants to declare one. The text answers `angles`
with its Lagna and its meridian point (III.46 to 49), tropical, and
its 24° obliquity; the chart subtracts the zodiac of §3, so the
sidereal Lagna is the text's exactly.

**House systems from the angles.** Nine systems are built from the
ascendant and the midheaven alone: whole sign, equal, Vehlow, equal
from the midheaven, equal from Aries, Porphyry, Sripati and the two
Pullen systems. When the angles are the provider's, the chart builds
those from them. The other thirteen need the sidereal time, the
latitude and the obliquity as a sphere, which a classical text does not
define, and are **refused**, naming the system and the nine that are
not: a Placidus chart over the text would be the hybrid again, one
level down. The chalit follows the same rule.

**Built without a vtable slot.** This page first said the C boundary
would append an `angles` slot (ABI 5). Building it asked who would fill
one: the text is a Rust provider inside the library, which every
binding reaches through the ephemeris selector (§7 step 5), not
through the vtable, and no foreign classical provider exists. So the
vtable stays at ABI 4; a plugin that declares `ANGLES` is refused at
load, because nothing could ask it for them, and a Rust provider handed
out through the vtable crosses without the bit. A foreign classical
astronomy is the consumer who would earn the slot.

The text's answer is its sidereal Lagna and meridian point, carried by
the ayanamsha **of the instant** rather than of the day's sunrise, which
its own tropical walk takes them back with: the port speaks tropical of
the instant and the chart measures in the zodiac of the instant, so the
chart's Lagna is the text's to the last bit. A day with no sunrise, on
which the text has no Lagna, falls back to the sphere under
`prefer-native` and is refused under `native-only`, as the day does.

## 6. What is reported

The steps stamp who gave each part (`zodiac`, `angles`, `day`, each
`NATIVE` or `SDK`), and the envelope's `deviation` is filled when a
classical provider defined any of them: its `model` the provider's
identity and its `detail` the parts it defined. A reader of a stored
chart can tell a classical chart from a modern one without the
provider to hand.

Built: the steps gain `zodiac`, `angles` and `day`
(`SolarModel::defines_sunrise`, which the drik model stops claiming once
the provider refuses its convention), and `deviation` reads, over the
text, `SURYA_SIDDHANTA`: "the zodiac, the places, the angles and the day
are the provider's own".

**A call in place of a refusal.** `angles_of`, which answers a stored
chart's angles with no ephemeris, refuses a chart whose angles were its
provider's (`ChartFoundation::angles_are_the_providers`): the sphere
would give it a modern midheaven. But the Tajika sahams read the
midheaven through `angles_of`, so refusing alone would have broken them
over the text. The façade gains `sdk.chart().angles(&chart)`: the
sphere's angles for a modern chart, the provider's for one whose angles
were its provider's, and a refusal naming the ephemeris option when the
context's provider does not define them — which the acceptance test
found answering the sphere's before it was made to refuse. The sahams
read it, and so does the pass's midheaven row.

## 7. The order of work

1. **The zodiac and the places** (§3): **done**. The pass's zodiac and
   graha rows read zero, all nine grahas.
2. **The day** (§4): **done**. The sunrise row reads zero and no hora
   differs.
3. **The angles** (§5): **done**. The Lagna and midheaven rows read
   zero and no sign differs; `crates/sdk/tests/classical_chart.rs`
   holds `sdk-only`'s hybrid, the refused Placidus and a stored chart's
   angles refused without its provider.
4. **The report** (§6): **done**.
5. **Reaching it**: **done**. `Ephemeris::SuryaSiddhanta` in the Rust
   façade and `TS_EPHEMERIS_SURYA_SIDDHANTA` (3) at the C selector, which
   the generators carry to every binding as `'SURYA_SIDDHANTA'`,
   `Ephemeris.SURYA_SIDDHANTA` and `Ephemeris.suryaSiddhanta`. Each
   binding's suite opens it by name and reads the deviation back, and
   the parity gate compares a classical chart — steps, Lagna, sunrise,
   the nine grahas and the deviation — across Rust, Node, Python and
   Dart. Node's error message listed the names by hand and now reads
   them from the generated enum.
6. **Re-aim the pass** at each step: the claims that read falsified
   today are written to fail both ways, so each step flips its rows and
   the page records it.

## 8. Not decided here

- ~~**A modern provider's overrides at the chart layer.**~~ **Decided**
  (2026-09-26, `QUESTIONS.md` Q40): the SDK's, because against the
  Swiss-based recording its sunrise (9.77 s at worst) and nutated zodiac
  (0.0086″) are closer than the engine's own (32.39 s, and a mean value
  18.46″ off); ADR-0013 is amended to say so.
- **The text's own Lagna for the day-lagna and the arudhas.** They read
  the Lagna at the day's sunrise through the same founder `angles`, so
  they follow §5 without a decision of their own; the pass does not yet
  hold them to the text.
- ~~**A `surya-siddhanta` profile.**~~ **Decided and built**
  (2026-09-26). `parashari-classical` with the text's astronomy
  (`frame.siddhanta: SURYA`) and its own ayanamsha; the profile **asks**
  and the chain supplies, so it never opens the text itself (ADR-0029),
  and a modern engine under it is refused at the context rather than
  answering a hybrid. Researching it found the `frame.siddhanta` knob
  read by nothing since it was added, and every settings warning said to
  nobody (`settings-and-profiles.md` §4); both are built with it.
