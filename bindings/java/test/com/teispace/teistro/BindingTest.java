package com.teispace.teistro;

import java.lang.foreign.StructLayout;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;

import com.teispace.teistro.blob.BlobFormatException;
import com.teispace.teistro.blob.IntlRender;
import com.teispace.teistro.blob.Positions;
import com.teispace.teistro.ffi.Native;
import com.teispace.teistro.record.Provenance;
import com.teispace.teistro.record.Step;
import com.teispace.teistro.request.*;

/**
 * The binding's tests, with no test framework: each is a method, a failure
 * is an {@link AssertionError}, and the run prints one line per test and
 * exits non-zero when any failed. Run by {@code cargo xtask check-java}.
 */
public final class BindingTest {
    private BindingTest() {}

    @FunctionalInterface
    private interface Test {
        void run() throws Exception;
    }

    private static void check(boolean holds, String what) {
        if (!holds) {
            throw new AssertionError(what);
        }
    }

    private static <T> void same(T expected, T found, String what) {
        if (!expected.equals(found)) {
            throw new AssertionError(what + ": expected " + expected + ", found " + found);
        }
    }

    private static TeistroException refusal(Runnable call) {
        try {
            call.run();
        } catch (TeistroException e) {
            return e;
        }
        throw new AssertionError("the call was not refused");
    }

    private static IllegalArgumentException refused(java.util.function.Supplier<ChartOptions> options, Context sky,
            Observer place) {
        try {
            sky.chart().found(2_460_000.0, place, 0, options.get());
        } catch (IllegalArgumentException e) {
            return e;
        }
        throw new AssertionError("the options were not refused");
    }

    /** A straight-line Sun and Moon, counting its calls, throwing {@code fails} when it is set. */
    private static class Line extends EphemerisProvider {
        private final RuntimeException fails;
        private final List<Body> bodies;
        int calls;

        Line(RuntimeException fails) {
            this(fails, List.of(Body.SUN, Body.MOON));
        }

        Line(RuntimeException fails, List<Body> bodies) {
            this.fails = fails;
            this.bodies = bodies;
        }

        @Override
        public String name() {
            return "line";
        }

        @Override
        public List<Body> bodies() {
            return bodies;
        }

        @Override
        public java.util.Optional<PositionAnswer> positions(PositionQuery query) {
            calls += 1;
            if (fails != null) {
                throw fails;
            }
            double[] lon = new double[query.cellCount()];
            for (int cell = 0; cell < lon.length; cell += 1) {
                lon[cell] = (280.46 + 0.9856 * (query.jds()[cell / query.bodies().size()] - 2_451_545.0)) % 360.0;
            }
            double[] zeros = new double[lon.length];
            double[] ones = new double[lon.length];
            java.util.Arrays.fill(ones, 1.0);
            return java.util.Optional.of(PositionAnswer.of(lon, zeros, ones));
        }
    }

    private static IllegalArgumentException refusedProvider(java.util.function.Supplier<EphemerisProvider> provider) {
        try {
            Teistro.open().context(ContextOptions.builder().provider(provider.get()).build()).close();
        } catch (IllegalArgumentException e) {
            return e;
        }
        throw new AssertionError("the provider was not refused");
    }

    private static Context context(Teistro teistro) {
        return teistro.context(ContextOptions.builder().profile("nepali-default").testProvider(true).build());
    }

    /**
     * Runs every test.
     *
     * @param args unused
     */
    public static void main(String[] args) {
        Map<String, Test> tests = new java.util.LinkedHashMap<>();
        Teistro teistro = Teistro.open();

        tests.put("the library is the build these declarations describe", () -> {
            same(Native.GENERATED_ABI_VERSION, teistro.abiVersion(), "abi");
            same(Native.GENERATED_SDK_VERSION, teistro.sdkVersion(), "sdk");
            same(Native.GENERATED_SDK_VERSION, teistro.buildInfo().get("sdk"), "build info");
            check(!teistro.defaultProfile().isEmpty(), "a default profile");
        });

        tests.put("every layout FFM built is the size the description computes", () -> {
            for (Map.Entry<String, StructLayout> entry : Native.LAYOUTS.entrySet()) {
                same(Native.SIZES.get(entry.getKey()), entry.getValue().byteSize(), entry.getKey());
            }
            same(Native.SIZES.size(), Native.LAYOUTS.size(), "one size per layout");
        });

        tests.put("a context answers its profile, settings and hash", () -> {
            try (Context sky = context(teistro)) {
                same("nepali-default", sky.profile(), "profile");
                check(sky.settingsJson().startsWith("{"), "settings are a JSON object");
                Json.object(sky.settingsJson());
                same(64, sky.settingsHash().length(), "a SHA-256 in hexadecimal");
                same(sky.settingsHash(), sky.settingsHash(), "the hash is stable");
            }
        });

        tests.put("a key and its id read back", () -> {
            try (Context sky = context(teistro)) {
                long sun = sky.keys().id("graha.SUN");
                same((long) Kind.GRAHA.id(), sun >>> 16, "the kind is the high half");
                same((long) Graha.SUN.id(), sun & 0xFFFF, "the member is the low half");
                same("graha.SUN", sky.keys().name(sun), "the name");
                same(Graha.SUN, Graha.of(Graha.SUN.id()), "of");
                same(Graha.UNKNOWN, Graha.of(0xFFFE), "an unknown id from a newer library");
                same(Graha.SUN, Graha.byKey("graha.SUN").orElseThrow(), "byKey, full");
                same("graha.SUN", Graha.SUN.fullKey(), "fullKey");
            }
        });

        tests.put("a refusal carries its status, its field and its hint", () -> {
            try (Context sky = context(teistro)) {
                TeistroException typo = refusal(() -> sky.keys().id("graha.SUNN"));
                same(Status.UNSUPPORTED, typo.status(), "status");
                same("UNKNOWN_KEY", typo.detail(), "detail");
                check(typo.hint().contains("SUN"), "the hint names the near key: " + typo.hint());
                // The second refusal carries the second record, not the first.
                TeistroException other = refusal(() -> sky.keys().id("rashi.ARIESS"));
                check(other.hint().contains("ARIES"), "the second record: " + other.hint());
            }
        });

        tests.put("a context the library refuses carries the record it wrote", () -> {
            TeistroException refused = refusal(() -> teistro.context(
                    ContextOptions.builder().profile("no-such-profile").build()).close());
            check(refused.status() != Status.OK, "refused");
            check(!refused.getMessage().isEmpty(), "with a message");
            check(refused.toString().startsWith("TeistroException ["), "and says so");
        });

        tests.put("a closed context refuses, and closing twice does nothing", () -> {
            Context sky = context(teistro);
            sky.close();
            sky.close();
            try {
                sky.profile();
                throw new AssertionError("a closed context answered");
            } catch (IllegalStateException expected) {
                check(expected.getMessage().contains("closed"), "says why");
            }
        });

        tests.put("a date converts between calendars and back to the same fixed day", () -> {
            try (Context sky = context(teistro)) {
                CalendarArea calendar = sky.calendar();
                CalendarDate first = calendar.dateOf(Calendar.GREGORIAN, 1);
                same(1, first.year(), "fixed day 1 is the year 1");
                same(1, first.month(), "in January");
                same(1, first.day(), "on its first");
                same(1, calendar.weekdayOf(first), "a Monday");
                CalendarDate today = new CalendarDate(Calendar.GREGORIAN, null, 2026, 0, 10, 8,
                        Resolution.DEFINED, 0, 0);
                long fixed = calendar.fixedOf(today);
                CalendarDate bs = calendar.convert(today, Calendar.BIKRAM_SAMBAT);
                same(Calendar.BIKRAM_SAMBAT, bs.calendar(), "the calendar asked for");
                same(2083, bs.year(), "Bikram Sambat runs 56 or 57 years ahead");
                same(fixed, calendar.fixedOf(bs), "the same day");
                same(today.day(), calendar.convert(bs, Calendar.GREGORIAN).day(), "and back");
                CalendarDate independence = new CalendarDate(Calendar.GREGORIAN, null, 1947, 0, 8, 15,
                        Resolution.DEFINED, 0, 0);
                CalendarDate saka = calendar.convert(independence, Calendar.INDIAN_NATIONAL);
                same(1869, saka.year(), "Shravana 24, 1869 (Saka)");
                same(5, saka.month(), "Shravana");
                same(24, saka.day(), "the 24th");
                check(calendar.isLeap(Calendar.GREGORIAN, 2024), "2024 is a leap year");
                check(!calendar.isLeap(Calendar.GREGORIAN, 2100), "2100 is not");
                same(28, calendar.monthLength(Calendar.GREGORIAN, 2026, 2), "February 2026");
            }
        });

        tests.put("a value C cannot hold is refused by the name the caller wrote", () -> {
            try (Context sky = context(teistro)) {
                CalendarDate bad = new CalendarDate(Calendar.GREGORIAN, null, 2026, 0, 300, 8,
                        Resolution.DEFINED, 0, 0);
                try {
                    sky.calendar().fixedOf(bad);
                    throw new AssertionError("a month of 300 was sent");
                } catch (IllegalArgumentException expected) {
                    check(expected.getMessage().contains("`month`"), expected.getMessage());
                }
                CalendarDate unknown = new CalendarDate(Calendar.UNKNOWN, null, 2026, 0, 1, 1,
                        Resolution.DEFINED, 0, 0);
                try {
                    sky.calendar().fixedOf(unknown);
                    throw new AssertionError("an unknown calendar was sent");
                } catch (IllegalArgumentException expected) {
                    check(expected.getMessage().contains("UNKNOWN"), expected.getMessage());
                }
                try {
                    new Latitude(91);
                    throw new AssertionError("a latitude of 91 was made");
                } catch (IllegalArgumentException expected) {
                    check(expected.getMessage().contains("[-90,90]"), expected.getMessage());
                }
            }
        });

        tests.put("the library's own calls need no context", () -> {
            Frame canonical = teistro.canonicalFrame();
            same(canonical, teistro.unpackFrame(teistro.packFrame(canonical)), "a frame packs and unpacks");
            // A wither replaces one component and keeps the rest.
            Frame lahiri = canonical.withSidereal(true).withAyanamsha(Ayanamsha.LAHIRI);
            same(new Frame(Ayanamsha.LAHIRI, canonical.centre(), canonical.equinox(), canonical.coordinates(),
                    true, canonical.lightTime(), canonical.aberration(), canonical.deflection(),
                    canonical.nutation()), lahiri, "withSidereal and withAyanamsha");
            same(canonical, lahiri.withSidereal(false).withAyanamsha(null), "and back");
            double jd = teistro.julianDayOfFixed(739_000);
            same(739_000 + 1_721_424.5, jd, "fixed + 1721424.5");
            CalendarFixedOfJdResult back = teistro.fixedOfJulianDay(jd + 0.25);
            same(739_000L, back.value(), "the fixed day");
            same(0.25, back.fraction(), "a quarter of it elapsed");
        });

        tests.put("an instant converts between scales and back", () -> {
            try (Context sky = context(teistro)) {
                double jd = 2_461_322.5;
                TimeConversion tt = sky.time().convert(jd, Scale.UTC, Scale.TT);
                same(Scale.TT, tt.to(), "the scale asked for");
                check(tt.jd() > jd, "TT runs ahead of UTC");
                TimeConversion utc = sky.time().convert(tt.jd(), Scale.TT, Scale.UTC);
                check(Math.abs(utc.jd() - jd) < 1e-9, "and back: " + (utc.jd() - jd));
                DeltaT delta = sky.time().deltaT(jd);
                check(delta.seconds() > 60 && delta.seconds() < 80, "delta T near 69 s: " + delta.seconds());
            }
        });

        tests.put("the intl area reads and sets the locale", () -> {
            try (Context sky = context(teistro)) {
                check(!sky.intl().locale().isEmpty(), "a locale");
                sky.intl().setLocale("en-Latn");
                same("en-Latn", sky.intl().locale(), "the locale set");
                TeistroException bad = refusal(() -> sky.intl().setLocale("xx-NOPE"));
                check(bad.status() != Status.OK, "an unknown locale is refused");
            }
        });

        tests.put("a positions grid decodes instants outermost", () -> {
            try (Context sky = context(teistro)) {
                PositionGrid read = sky.positions(new double[] {2_451_545.0, 2_451_546.0},
                        List.of(Body.SUN, Body.MOON, Body.MARS));
                same(read.at(1, 1).longitude(), read.decoded().cells().lon(4), "a cell is read instants outermost");
                Positions grid = read.decoded();
                same(2L, grid.jdCount(), "two instants");
                same(3L, grid.bodyCount(), "three bodies");
                same((long) TimeScale.UT1.id(), grid.scale(), "the scale asked for");
                same(2_451_546.0, grid.instants().jd(1), "the instants in order");
                same(Body.MOON, Body.of(grid.bodies().body(1)), "the bodies in order");
                Positions.Cells cells = grid.cells();
                same(6, cells.length(), "a cell per instant and body");
                for (int row = 0; row < cells.length(); row += 1) {
                    same(0, cells.status(row), "cell " + row + " has a value");
                    check(0 <= cells.lon(row) && cells.lon(row) <= 360, "a longitude");
                }
                check(Math.abs(cells.lonSpeed(1)) > Math.abs(cells.lonSpeed(0)), "the Moon outruns the Sun");
                check(Json.read(grid.steps()) instanceof List<?>, "the steps are a JSON list");
                check(Json.object(grid.provenanceJson()).containsKey("settings_hash"), "provenance");
                Provenance provenance = Provenance.of(Json.read(grid.provenanceJson()));
                same(sky.settingsHash(), provenance.settingsHash(), "the provenance is the context's settings");
                List<Step> steps = ((List<?>) Json.read(grid.steps())).stream().map(Step::of).toList();
                check(!steps.isEmpty() && !steps.get(0).name().isEmpty(), "the steps decode as records");
                try {
                    cells.lon(6);
                    throw new AssertionError("a seventh cell was read");
                } catch (IndexOutOfBoundsException expected) {
                    // the section holds six
                }
            }
        });

        tests.put("bytes that are not a blob are refused, never misread", () -> {
            try (Context sky = context(teistro)) {
                PositionRequest request = new PositionRequest(TimeScale.UT1,
                        teistro.packFrame(teistro.canonicalFrame()), true, null,
                        new double[] {2_451_545.0}, List.of(Body.SUN));
                byte[] raw = sky.locked((lib, handle) -> Calls.positions(lib, handle, request));
                Positions.decode(raw);
                java.util.function.Function<byte[], String> refused = bytes -> {
                    try {
                        Positions.decode(bytes);
                    } catch (BlobFormatException e) {
                        return e.getMessage();
                    }
                    throw new AssertionError("decoded");
                };
                check(refused.apply(java.util.Arrays.copyOf(raw, 16)).contains("header"), "too short");
                byte[] magic = raw.clone();
                magic[0] = 'N';
                check(refused.apply(magic).contains("not a Teistro"), "the magic");
                byte[] version = raw.clone();
                version[4] = 99;
                check(refused.apply(version).contains("version 99"), "the version");
                check(refused.apply(java.util.Arrays.copyOf(raw, raw.length + 8)).contains("bytes"), "the length");
                try {
                    IntlRender.decode(raw);
                    throw new AssertionError("another schema decoded");
                } catch (BlobFormatException expected) {
                    check(expected.getMessage().contains("schema"), expected.getMessage());
                }
            }
        });

        tests.put("a message renders in the context's locale", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().profile("nepali-default")
                    .locale("en-Latn").testProvider(true).build())) {
                IntlRender said = sky.intl().render("sdk.reason.grahaInBhava",
                        Map.of("graha", Map.of("$entity", "graha.JUPITER"), "bhava", 7));
                check(!said.text().isEmpty(), "some text");
                same(0, said.isFallback(), "the locale carries it");
                same("{\"a\":[1,\"x\\n\",null,\"graha.SUN\"]}",
                        Json.write(new java.util.LinkedHashMap<>(Map.of("a",
                                java.util.Arrays.asList(1, "x\n", null, Graha.SUN)))), "JSON written");
            }
        });

        tests.put("a chart is founded at an instant and a place", () -> {
            try (Context sky = context(teistro)) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
                ChartBatch batch = sky.chart().foundMany(new double[] {2_460_000.0, 2_460_001.0}, kathmandu, 20_700,
                        ChartOptions.builder().state(true).dashas(DashaSystem.VIMSHOTTARI).build());
                same(2, batch.size(), "a chart per instant");
                same(ChartKind.NATAL, batch.kind(), "natal by default");
                same(27.7172, batch.place().latitudeDeg().value(), "the place");
                Chart chart = batch.get(1);
                same(2_460_001.0, chart.instant(), "the instant asked for");
                check(0 <= chart.lagnaDeg() && chart.lagnaDeg() < 360, "a lagna");
                check(chart.ayanamsha().isPresent(), "the Nepali profile reads a catalogued ayanamsha");
                same(64, chart.provenance().contentHash().length(), "the chart's own hash");
                check(!chart.provenance().contentHash().equals(batch.provenance().contentHash()),
                        "a chart's hash is its own, not the batch's");
                same(sky.settingsHash(), chart.provenance().settingsHash(), "under the context's settings");
                int seen = 0;
                for (Chart each : batch) {
                    same(seen, each.index(), "in order");
                    seen += 1;
                }
                IllegalArgumentException typo = refused(() -> ChartOptions.builder().reading("kpp", Map.of()).build(),
                        sky, kathmandu);
                check(typo.getMessage().contains("`kpp`"), typo.getMessage());
            }
        });

        tests.put("a hit list carries varga ingresses, lunar phases and eclipses", () -> {
            // The birth is an hour after the solar eclipse of 29 March 2025,
            // so 2025 holds an eclipse on its luminaries.
            try (Context sky = teistro.context(ContextOptions.builder().ephemeris(Ephemeris.BUILTIN).build())) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
                List<HitKind> kinds = List.of(HitKind.VARGA_INGRESS, HitKind.LUNAR_PHASE, HitKind.PHASE_RETURN,
                        HitKind.SOLAR_ECLIPSE, HitKind.LUNAR_ECLIPSE);
                HitRequest.Builder asked = HitRequest.builder().from(2_460_676.5).to(2_461_041.5)
                        .grahas(List.of(Graha.SUN, Graha.MARS)).kinds(kinds).vargas(List.of(Varga.D9, Varga.D30))
                        .eclipseOrbDeg(3);
                List<Hit> hits = sky.chart().found(2_460_764.0, kathmandu, 0,
                        ChartOptions.builder().hits(asked.build()).build()).hits();
                same(java.util.Set.copyOf(kinds),
                        hits.stream().map(hit -> hit.event().kind()).collect(java.util.stream.Collectors.toSet()),
                        "every kind asked for");
                int phases = 0;
                for (Hit hit : hits) {
                    switch (hit.event()) {
                        case VargaIngress ingress -> check(ingress.varga() == Varga.D9 || ingress.varga() == Varga.D30,
                                "a varga asked for: " + ingress.varga());
                        case LunarPhase phase -> {
                            same(Graha.MOON, hit.graha(), "a phase is the Moon's");
                            phases += 1;
                        }
                        case SolarEclipseHit eclipse -> same(Graha.SUN, hit.graha(), "a solar eclipse is the Sun's hit");
                        case LunarEclipseHit eclipse -> same(Graha.MOON, hit.graha(), "an eclipse is the Moon's hit");
                        case PhaseReturn back -> same(Graha.MOON, hit.graha(), "a phase return is the Moon's");
                        default -> throw new AssertionError("a kind not asked for: " + hit.event());
                    }
                }
                check(48 <= phases && phases <= 50, "a year's phases: " + phases);
                TeistroException d1 = refusal(() -> sky.chart().found(2_460_764.0, kathmandu, 0,
                        ChartOptions.builder().hits(asked.vargas(List.of(Varga.D1)).build()).build()));
                same("hits.vargas", d1.field(), "the rashi chart refused by name");
            }
        });

        tests.put("a chart's day is the almanac's, and its date converts", () -> {
            try (Context sky = context(teistro)) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
                LocalDay day = sky.chart().found(2_451_545.0, kathmandu, 20_700, ChartOptions.builder().build()).day();
                same(day, sky.almanac().day(day.date(), kathmandu, 20_700).day(), "one day, two areas");
                same(Calendar.BIKRAM_SAMBAT, day.date().calendar(), "the profile's calendar");
                same(List.of(2056, 9, 17), List.of(day.date().year(), day.date().month(), day.date().day()),
                        "the Bikram Sambat date");
                same(Vara.SHANIVARA, day.vara(), "a Saturday");
                check(day.sunrise() < day.sunset() && day.sunset() < 2_451_545.0 && 2_451_545.0 < day.nextSunrise(),
                        "the instant is in the day's night");
                same(Sunrise.CENTRE_NO_REFRACTION, day.convention(), "the profile's sunrise");
            }
        });

        tests.put("a chart carries its dispositors", () -> {
            // Each chain the sign lords followed from the planet's own sign until
            // one repeats, its end one of the chart's ends, and the final
            // dispositor exactly when every chain stops at one planet in its own
            // sign (03-design/essential-dignities.md §Dispositors).
            try (Context sky = teistro.context(ContextOptions.builder().profile("conformance-baseline")
                    .ephemeris(Ephemeris.BUILTIN).build())) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(0));
                // The traditional lords, Aries to Pisces.
                List<Graha> lords = List.of(Graha.MARS, Graha.VENUS, Graha.MERCURY, Graha.MOON, Graha.SUN,
                        Graha.MERCURY, Graha.VENUS, Graha.MARS, Graha.JUPITER, Graha.SATURN, Graha.SATURN,
                        Graha.JUPITER);
                java.util.Set<Boolean> finals = new java.util.HashSet<>();
                for (int step = 0; step < 30; step += 1) {
                    Dignities read = sky.chart().found(2_460_676.5 + step * 23.7, kathmandu, 20_700,
                            ChartOptions.builder().dignities(DignityRequest.of(Map.of())).build())
                            .dignities().orElseThrow();
                    Map<Graha, Graha> lordOf = new java.util.HashMap<>();
                    for (PlanetDignity p : read.planets()) {
                        lordOf.put(p.planet(), lords.get((int) Math.floor(p.longitudeDeg() / 30)));
                    }
                    Dispositions disposed = read.dispositions();
                    same("TRADITIONAL", disposed.rulers(), "the traditional rulers");
                    same(read.planets().stream().map(PlanetDignity::planet).toList(),
                            disposed.chains().stream().map(DispositorChain::planet).toList(), "the planets' order");
                    for (DispositorChain chain : disposed.chains()) {
                        List<Graha> walked = new ArrayList<>();
                        for (Graha at = lordOf.get(chain.planet()); at != chain.planet() && !walked.contains(at);
                                at = lordOf.get(at)) {
                            walked.add(at);
                        }
                        same(walked, chain.dispositors(), chain.planet() + "'s chain");
                        check(disposed.ends().contains(chain.end()), "one of the chart's ends");
                        Graha last = walked.isEmpty() ? chain.planet() : walked.get(walked.size() - 1);
                        if (chain.end().kind() == ChainEnd.OWN_SIGN) {
                            same(List.of(last), chain.end().planets(), "the planet in its own sign");
                            same(last, lordOf.get(last), last + " in its own sign");
                        } else {
                            same(ChainEnd.CYCLE, chain.end().kind(), "a cycle");
                            check(chain.end().planets().size() >= 2 && chain.end().planets().contains(last),
                                    "a cycle holding the chain's last");
                        }
                    }
                    boolean one = disposed.ends().size() == 1 && disposed.ends().get(0).kind() == ChainEnd.OWN_SIGN;
                    check(java.util.Objects.equals(one ? disposed.ends().get(0).planets().get(0) : null,
                            disposed.finalDispositor()), "the final dispositor");
                    finals.add(disposed.finalDispositor() == null);
                }
                same(java.util.Set.of(true, false), finals, "both a final dispositor and none");
            }
        });

        tests.put("a chart carries its essential dignities in the Chaldean order", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().profile("conformance-baseline")
                    .ephemeris(Ephemeris.BUILTIN).build())) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(0));
                Chart bare = sky.chart().found(2_460_676.5, kathmandu, 20_700, ChartOptions.builder().build());
                check(bare.dignities().isEmpty(), "not asked, not read");
                Dignities read = sky.chart().found(2_460_676.5, kathmandu, 20_700,
                        ChartOptions.builder().dignities(DignityRequest.of(Map.of())).build()).dignities().orElseThrow();
                same(SectRule.HORIZON, read.sectRule(), "the horizon decides the sect");
                same(List.of(Graha.SATURN, Graha.JUPITER, Graha.MARS, Graha.SUN, Graha.VENUS, Graha.MERCURY,
                        Graha.MOON), read.planets().stream().map(PlanetDignity::planet).toList(), "the Chaldean order");
                same(5, read.scores().house(), "Lilly's house score");
            }
        });

        tests.put("an almanac reads a native's bird over its days", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().ephemeris(Ephemeris.BUILTIN).build())) {
                Observer madras = new Observer(new Longitude(80.2707), new Latitude(13.0827), new Altitude(6));
                CalendarDate tuesday = new CalendarDate(Calendar.GREGORIAN, null, 1984, 0, 10, 30, Resolution.DEFINED, 0, 0);
                CalendarDate wednesday = new CalendarDate(Calendar.GREGORIAN, null, 1984, 0, 10, 31, Resolution.DEFINED, 0, 0);
                PakshiDays sealed = sky.almanac().pakshi(tuesday, wednesday, madras, 19_800,
                        PakshiNative.star(Nakshatra.UTTARA_ASHADHA, Paksha.SHUKLA));
                check(!sealed.provenance().inputHash().isEmpty(), "the request is sealed");
                List<PakshiDay> days = sealed.value();
                same(2, days.size(), "two days");
                PakshiDay.Reading first = days.get(0).reading();
                PakshiDay.Reading second = days.get(1).reading();
                same(first.day().nextSunrise(), second.day().sunrise(), "one day runs into the next");
                same(Vara.BUDHAVARA, second.day().vara(), "a Wednesday");
                same(Paksha.SHUKLA, second.day().paksha(), "in the bright half");
                same("COCK", second.bird(), "Uttara Ashadha's bird");
                // PUL p. vii: the cock sleeps in the day's second yama and dies from the third.
                same(List.of("SLEEPING", "DYING"),
                        List.of(second.yamas().get(1).activity(), second.yamas().get(2).activity()), "the activities");
                same("OWN", second.yamas().get(0).subs().get(0).ownerIs(), "the native's own sub-period");
                try {
                    PakshiRules.of(Map.of("clock", "SUNDIAL"));
                    throw new AssertionError("a clock the SDK does not read was read");
                } catch (IllegalArgumentException e) {
                    check(e.getMessage().contains("PakshiRules.clock"), e.getMessage());
                }
                // A map built in Java may hold the member itself, as a builder's would.
                same(SubLengths.PULIPPANI, PakshiRules.of(Map.of("subs", SubLengths.PULIPPANI)).subs(),
                        "a request member reads as itself");
                same(SubLengths.AGASTYA, SubLengths.of(SubLengths.AGASTYA), "and on its own");
                check(sky.almanac().pakshi(wednesday, null, madras, 19_800, PakshiNative.bird("OWL"),
                        PakshiRules.builder().subs(SubLengths.PULIPPANI).build()) != null,
                        "a typed rule reaches the SDK");
            }
            try (Context polar = teistro.context(ContextOptions.builder().ephemeris(Ephemeris.BUILTIN)
                    .settingsJson("{\"day\": {\"polar_day_policy\": \"NEAREST_EVENT\"}}").build())) {
                Observer tromso = new Observer(new Longitude(18.9553), new Latitude(69.6492), new Altitude(0));
                CalendarDate midsummer = new CalendarDate(Calendar.GREGORIAN, null, 2024, 0, 6, 21, Resolution.DEFINED, 0, 0);
                List<PakshiDay> days = polar.almanac().pakshi(midsummer, null, tromso, 7_200, PakshiNative.bird("OWL"))
                        .value();
                check(days.get(0).reading() == null, "no sunset, no yamas");
            }
        });

        tests.put("a study counts and tests its rules over a batch of births", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().ephemeris(Ephemeris.BUILTIN).build())) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(0));
                List<ResearchBirth> births = new java.util.ArrayList<>();
                for (int i = 0; i < 6; i += 1) {
                    births.add(new ResearchBirth(2447000.25 + 977.3 * i, kathmandu, 20_700));
                }
                com.teispace.teistro.request.RuleRequest yogas = com.teispace.teistro.request.RuleRequest.builder()
                        .shipped(List.of(com.teispace.teistro.request.ShippedRules.YOGAS)).build();
                com.teispace.teistro.request.Design design = com.teispace.teistro.request.Design.of(
                        Map.of("groups", List.of(0, 1, 0, 1, 0, 1)));
                ResearchCounts table = sky.research().counts(births, yogas, design, null);
                check(!table.rows().isEmpty(), "a row per rule");
                for (ResearchCounts.Row row : table.rows()) {
                    for (ResearchCounts.Group group : row.counts()) {
                        same(3, group.present() + group.absent() + group.unreadable() + group.unstable(),
                                row.predicate());
                    }
                }
                // A seed past Long.MAX_VALUE is a BigInteger, and the same seed in every binding.
                java.math.BigInteger widest = java.math.BigInteger.TWO.pow(64).subtract(java.math.BigInteger.ONE);
                ResearchTested tested = sky.research().compare(births, yogas, design,
                        ResearchGroupTest.builder(widest, 199, ResearchContrast.caseVsRest(1)).alpha(0.05).build(),
                        null);
                // Six births, three cases: 20 labellings, fewer than the 199 asked, each read once.
                check(tested.enumerated(), "a set this small is walked whole");
                same(20, tested.permutations(), "every labelling");
                same(1.0 / 20, tested.resolution(), "the resolution they give");
                for (ResearchTested.Row row : tested.rows()) {
                    check(row.p().value() >= tested.resolution(), "never under the resolution");
                    check(row.adjusted().maxT() >= row.p().value(), "an adjustment never lowers a p-value");
                    check(row.underAlpha() != null, "alpha was given");
                }
                check(!tested.provenance().inputHash().equals(table.provenance().inputHash()), "another study");
                TeistroException refused = refusal(() -> sky.research().timed(List.of(), yogas,
                        DashaSystem.VIMSHOTTARI, ResearchEventShuffle.AGES_AT_EVENT,
                        ResearchEventTest.builder(1, 19).build(), null));
                same("research.subjects", refused.field(), "named by its record");
                // Self-matched, each person is read against their own year.
                List<ResearchSubject> lives = new java.util.ArrayList<>();
                for (ResearchBirth birth : births) {
                    lives.add(new ResearchSubject(birth, birth.instant() + 9000.5));
                }
                ResearchTested matched = sky.research().timed(lives, yogas, DashaSystem.VIMSHOTTARI,
                        new ResearchReferents(365.25, 30), ResearchEventTest.builder(1, 19).build(), null);
                same(tested.rows().size(), matched.rows().size(), "a row per rule");
                TeistroException unreferred = refusal(() -> sky.research().timed(lives, yogas,
                        DashaSystem.VIMSHOTTARI, ResearchEventShuffle.SELF_MATCHED,
                        ResearchEventTest.builder(1, 19).build(), null));
                same("referents", unreferred.field(), "a self-matched study needs its referents");
                TeistroException tight = refusal(() -> sky.research().timed(lives, yogas, DashaSystem.VIMSHOTTARI,
                        new ResearchReferents(365.25, 0.25), ResearchEventTest.builder(1, 19).build(), null));
                same("referents.stepDays", tight.field(), "a window holds at most a thousand instants");
                // A transit rule is read at each pairing's instant, the sky's cost chosen with maxPairs:
                // six lives under the age shuffle need the sky at up to 36 instants.
                com.teispace.teistro.request.RuleRequest saturnReturn = com.teispace.teistro.request.RuleRequest
                        .builder()
                        .rules(List.of(com.teispace.teistro.request.RuleDefinition.builder()
                                .key("SATURN_RETURN")
                                .category("transit")
                                .source(com.teispace.teistro.request.RuleSource.builder().text("a test").build())
                                .conditions(List.of(new com.teispace.teistro.request.ConditionTransitInHouse(
                                        "SATURN", List.of(1), Graha.SATURN)))
                                .build()))
                        .build();
                ResearchTested transit = sky.research().timed(lives, saturnReturn, DashaSystem.VIMSHOTTARI,
                        ResearchEventShuffle.AGES_AT_EVENT, ResearchEventTest.builder(1, 19).build(), null);
                same("SATURN_RETURN", transit.rows().get(0).predicate(), "the transit rule's row");
                TeistroException capped = refusal(() -> sky.research().timed(lives, saturnReturn,
                        DashaSystem.VIMSHOTTARI, ResearchEventShuffle.AGES_AT_EVENT,
                        ResearchEventTest.builder(1, 19).maxPairs(2).build(), null));
                same("test.maxPairs", capped.field(), "the sky's cost is chosen");
                // A test the call needs is refused by name before anything crosses.
                TeistroException untested = refusal(() -> sky.research().compare(births, yogas, design, null, null));
                same("test", untested.field(), "no test, no study");
                // An uncertain birth's window is read every step, and the step is part of the study.
                List<ResearchBirth> uncertain = new java.util.ArrayList<>();
                for (ResearchBirth birth : births) {
                    uncertain.add(new ResearchBirth(birth.instant(), birth.place(), birth.utcOffsetSeconds(), 30));
                }
                String fine = sky.research().counts(uncertain, yogas, design,
                        ResearchOptions.builder().uncertaintyStepMinutes(10).build())
                        .provenance().inputHash();
                check(!fine.equals(sky.research().counts(uncertain, yogas, design, null).provenance().inputHash()),
                        "the step is sealed with the study");
                TeistroException still = refusal(() -> sky.research().counts(uncertain, yogas, design,
                        ResearchOptions.builder().uncertaintyStepMinutes(0).build()));
                // A field the study does not read is refused by name, not dropped.
                TeistroException stray = refusal(() -> sky.research().counts(births, yogas, design,
                        ResearchOptions.builder().depth(3).build()));
                same("research.depth", stray.field(), "a counts study reads no depth");
                same("uncertaintyStepMinutes", still.field(), "a step is above zero");
            }
        });

        tests.put("a chart carries its moment, the eclipse its place sees", () -> {
            // The rising sign's edges bound the instant, and the lunar eclipse of 2025 March 14,
            // seen from New York, is the instant's and not a day later's (C378).
            try (Context sky = teistro.context(ContextOptions.builder().ephemeris(Ephemeris.BUILTIN).build())) {
                Observer newYork = new Observer(new Longitude(-74.0), new Latitude(40.71), new Altitude(0));
                double eclipse = 2_460_748.790_8;
                check(sky.chart().found(eclipse, newYork, 0, ChartOptions.builder().build()).moment().isEmpty(),
                        "none unless asked");
                double[] instants = {eclipse, eclipse + 1};
                ChartBatch batch = sky.chart().foundMany(instants, newYork, 0,
                        ChartOptions.builder().moment(true).build());
                for (int k = 0; k < instants.length; k += 1) {
                    RisingSign rising = batch.get(k).moment().orElseThrow().rising();
                    check(rising.rose() <= instants[k] && instants[k] < rising.sets(),
                            "the rising sign's edges bound the instant");
                }
                Moment during = batch.get(0).moment().orElseThrow();
                Moment after = batch.get(1).moment().orElseThrow();
                same(BirthEclipse.LUNAR, during.eclipse().kind(), "the eclipse New York sees");
                check(Math.abs(during.eclipse().greatest() - eclipse) < 0.1, "its greatest moment");
                same(new MomentEclipse(BirthEclipse.NONE, null), after.eclipse(), "a day later");
            }
        });

        tests.put("a chart reads the Sarvatobhadra chakra", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().ephemeris(Ephemeris.BUILTIN).build())) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
                double instant = 2_447_995.489_583_333_5;
                List<Double> instants = List.of(2_460_676.5, 2_460_776.5, 2_460_876.5);
                check(sky.chart().found(instant, kathmandu, 20_700, ChartOptions.builder().build()).sarvatobhadra()
                        .isEmpty(), "none unless asked");
                Chart natal = sky.chart().found(instant, kathmandu, 20_700, ChartOptions.builder()
                        .sarvatobhadra(SarvatobhadraRequest.of(Map.of("instants", instants))).build());
                List<Sarvatobhadra> read = natal.sarvatobhadra().orElseThrow();
                same(instants.size(), read.size(), "a reading an instant");
                double moon = natal.grahas().stream().filter(g -> g.graha() == Graha.MOON).findFirst().orElseThrow()
                        .longitudeDeg();
                List<Graha> five = List.of(Graha.MARS, Graha.MERCURY, Graha.JUPITER, Graha.VENUS, Graha.SATURN);
                List<String> five28 = List.of("NAKSHATRA", "LETTER", "VOWEL", "TITHI", "RASHI");
                for (int k = 0; k < read.size(); k++) {
                    Sarvatobhadra one = read.get(k);
                    same(Nakshatra.of((int) (moon * 27 / 360)), one.nativePoints().nakshatra(), "the natal Moon's star");
                    List<PlacedGraha> placed = sky.chart().found(instants.get(k), kathmandu, 0,
                            ChartOptions.builder().build()).grahas();
                    for (int n = 0; n < one.grahas().size(); n++) {
                        Sarvatobhadra.GrahaVedha vedha = one.grahas().get(n);
                        same(placed.get(n).graha(), vedha.graha(), "the nine in order");
                        same(Nakshatra.of((int) (placed.get(n).longitudeDeg() * 27 / 360)), vedha.place().nakshatra(),
                                "each transit's star");
                        if (five.contains(vedha.graha())) {
                            same(placed.get(n).speedDegPerDay() < 0, "RETROGRADE".equals(vedha.motion()),
                                    "the sky's motion");
                        }
                        for (Sarvatobhadra.Reach reach : vedha.cells()) {
                            same("STANDS".equals(reach.by()), reach.cell() == null, "a cell unless standing");
                        }
                    }
                    same("FAST", one.grahas().get(0).motion(), "the Sun fast");
                    same("RETROGRADE", one.grahas().get(8).motion(), "Ketu retrograde");
                    same((int) one.points().stream()
                            .filter(p -> five28.contains(p.point()) && !p.malefics().isEmpty()).count(),
                            one.maleficPoints(), "v. 28's five a malefic strikes");
                }
                List<String> given = List.of("RETROGRADE", "MEAN", "FAST", "MEAN", "RETROGRADE");
                Sarvatobhadra asked = sky.chart().found(instant, kathmandu, 20_700, ChartOptions.builder()
                        .sarvatobhadra(SarvatobhadraRequest.of(Map.of("instants", instants,
                                "motion", Map.of("GIVEN", given),
                                "name", Map.of("letter", "र", "vowel", "A"),
                                "rules", Map.of("sight", "ALL_THREE"))))
                        .build()).sarvatobhadra().orElseThrow().get(0);
                List<String> motions = new ArrayList<>();
                for (Graha graha : five) {
                    motions.add(asked.grahas().stream().filter(g -> g.graha() == graha).findFirst().orElseThrow()
                            .motion());
                }
                same(given, motions, "the caller's five");
                check(asked.grahas().stream().allMatch(g -> g.lines().size() == 3), "all three lines");
                same(new Sarvatobhadra.Name("र", "A"), asked.nativePoints().name(), "the name");
                same("ALL_THREE", asked.rules().sight(), "the rules read back");
                TeistroException refused = refusal(() -> sky.chart().found(instant, kathmandu, 20_700,
                        ChartOptions.builder().sarvatobhadra(SarvatobhadraRequest.of(Map.of("instants", List.of())))
                                .build()));
                same("sarvatobhadra.instants", refused.field(), "a refusal names its field");
            }
        });

        tests.put("a chart reads its tithi praveshas", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().ephemeris(Ephemeris.BUILTIN).build())) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
                double instant = 2_447_995.489_583_333_5;
                check(sky.chart().found(instant, kathmandu, 20_700, ChartOptions.builder().build()).tithiPravesha()
                        .isEmpty(), "none unless asked");
                java.util.function.ToDoubleFunction<Chart> elongation = chart -> {
                    double sun = 0;
                    double moon = 0;
                    for (PlacedGraha g : chart.grahas()) {
                        if (g.graha() == Graha.SUN) {
                            sun = g.longitudeDeg();
                        } else if (g.graha() == Graha.MOON) {
                            moon = g.longitudeDeg();
                        }
                    }
                    return ((moon - sun) % 360 + 360) % 360;
                };
                Chart natal = sky.chart().found(instant, kathmandu, 20_700, ChartOptions.builder()
                        .tithiPravesha(TithiPraveshaRequest.builder().through(3).build()).build());
                List<TithiPravesha> years = natal.tithiPravesha().orElseThrow();
                same(List.of(1, 2, 3), years.stream().map(TithiPravesha::year).toList(), "a year a row, 1 up");
                double born = elongation.applyAsDouble(natal);
                for (TithiPravesha year : years) {
                    same(TithiRule.NATAL_SIGN, year.rule(), "the default rule");
                    same(year.rule() != year.picked(), year.fellBack(), "fell back when the rules differ");
                    check(Math.abs(year.elongationDeg() - born) < 1e-9, "the birth's elongation");
                    check(Math.abs(year.at() - year.solarReturn()) < 33, "inside the window");
                    check(year.inSign() >= 0 && year.inSign() <= 2, "at most two in the sign");
                    double then = elongation.applyAsDouble(
                            sky.chart().found(year.at(), kathmandu, 20_700, ChartOptions.builder().build()));
                    check(Math.abs(then - born) < 1e-3, "year " + year.year() + " reads back");
                }
                List<TithiPravesha> nearest = sky.chart().found(instant, kathmandu, 20_700, ChartOptions.builder()
                        .tithiPravesha(TithiPraveshaRequest.builder().through(3).rule(TithiRule.NEAREST_RETURN)
                                .build())
                        .build()).tithiPravesha().orElseThrow();
                for (TithiPravesha year : nearest) {
                    same(TithiRule.NEAREST_RETURN, year.picked(), "the rival picks itself");
                    check(!year.fellBack(), "the rival never falls back");
                    check(Math.abs(year.at() - year.solarReturn()) < 16, "within a half month");
                }
                TeistroException refused = refusal(() -> sky.chart().found(instant, kathmandu, 20_700,
                        ChartOptions.builder().tithiPravesha(TithiPraveshaRequest.builder().through(0).build())
                                .build()));
                same("tithiPravesha.through", refused.field(), "a refusal names its field");
            }
        });

        tests.put("a chart reads its month and day charts", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().ephemeris(Ephemeris.BUILTIN).build())) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
                double instant = 2_447_995.489_583_333_5;
                check(sky.chart().found(instant, kathmandu, 20_700, ChartOptions.builder().build()).masaPravesha()
                        .isEmpty(), "none unless asked");
                java.util.function.DoubleUnaryOperator sun = at -> sky.chart()
                        .found(at, kathmandu, 20_700, ChartOptions.builder().build()).grahas().stream()
                        .filter(g -> g.graha() == Graha.SUN).findFirst().orElseThrow().longitudeDeg();
                java.util.function.DoubleBinaryOperator apart = (a, b) -> {
                    double d = ((a - b) % 360 + 360) % 360;
                    return Math.min(d, 360 - d);
                };
                double born = sun.applyAsDouble(instant);
                List<MasaPravesha> months = sky.chart().found(instant, kathmandu, 20_700, ChartOptions.builder()
                        .masaPravesha(MasaPraveshaRequest.builder().year(3).build()).build()).masaPravesha()
                        .orElseThrow();
                same(12, months.size(), "twelve months");
                for (MasaPravesha month : months) {
                    same(1, month.day(), "a month as its first day");
                    check(apart.applyAsDouble(month.sunDeg(), born + 30 * (month.month() - 1)) < 1e-9,
                            "month " + month.month() + " a sign on");
                }
                MasaPravesha fifth = months.get(4);
                check(apart.applyAsDouble(sun.applyAsDouble(fifth.at()), fifth.sunDeg()) < 1e-4, "reads back");
                List<MasaPravesha> days = sky.chart().found(instant, kathmandu, 20_700, ChartOptions.builder()
                        .masaPravesha(MasaPraveshaRequest.builder().year(3).month(5).build()).build())
                        .masaPravesha().orElseThrow();
                same(30, days.size(), "thirty days");
                same(fifth.at(), days.get(0).at(), "the first day is the month");
                check(apart.applyAsDouble(sun.applyAsDouble(days.get(9).at()), fifth.sunDeg() + 9) < 1e-4,
                        "the tenth day nine degrees on");
                TeistroException refused = refusal(() -> sky.chart().found(instant, kathmandu, 20_700,
                        ChartOptions.builder().masaPravesha(MasaPraveshaRequest.builder().year(3).month(13).build())
                                .build()));
                same("masaPravesha.month", refused.field(), "a refusal names its field");

                // A place founds each month and reads its lord; the first month's is the year's.
                check(months.get(0).masesha() == null, "no lord without a place");
                YearLord third = sky.chart().found(instant, kathmandu, 20_700, ChartOptions.builder()
                        .varsha(VarshaRequest.builder().through(3).place(AnnualPlace.BIRTH).build()).build())
                        .praveshas().get(2).annual().yearLord();
                List<MasaPravesha> lords = sky.chart().found(instant, kathmandu, 20_700, ChartOptions.builder()
                        .masaPravesha(MasaPraveshaRequest.builder().year(3).place(AnnualPlace.BIRTH).build()).build())
                        .masaPravesha().orElseThrow();
                same(12, lords.size(), "twelve months");
                same(third.graha(), lords.get(0).masesha().graha(), "the first month's lord is the year's");
                for (MasaPravesha row : lords) {
                    Masesha lord = row.masesha();
                    same(MaseshaReading.CHAIN, lord.reading(), "the verse's own reading");
                    same(lord.chain().graha(), lord.graha(), "the chain's lord");
                    check(lord.chain().claims().stream().allMatch(c -> c.vishwa().total() > 0), "every claim weighed");
                }
                List<MasaPravesha> others = sky.chart().found(instant, kathmandu, 20_700, ChartOptions.builder()
                        .masaPravesha(MasaPraveshaRequest.builder().year(3).place(AnnualPlace.BIRTH)
                                .masesha(MaseshaRules.builder().reading(MaseshaReading.LAGNA_LORD).build()).build())
                        .build()).masaPravesha().orElseThrow();
                for (MasaPravesha row : others) {
                    same(row.masesha().bearers().masaLagna(), row.masesha().graha(), "the others' reading");
                }
                // Each month's dasha: the five Huddas of the stronger lord's sign, end to end from the
                // month's opening to the next's.
                check(months.get(0).dasha() == null, "no dasha without a place");
                for (int index = 0; index < lords.size(); index++) {
                    MasaPravesha row = lords.get(index);
                    MasaDasha dasha = row.dasha();
                    MonthBearers bearers = row.masesha().bearers();
                    same(dasha.office() == MonthOffice.MUNTHA ? bearers.muntha() : bearers.masaLagna(),
                            dasha.graha(), "the stronger lord's");
                    same(30, dasha.periods().stream().mapToInt(MasaDashaPeriod::degrees).sum(), "thirty degrees");
                    same(row.at(), dasha.periods().get(0).starts(), "from the month's opening");
                    if (index < 11) {
                        check(Math.abs(dasha.periods().get(4).ends() - lords.get(index + 1).at()) < 1e-6,
                                "ends as the next opens");
                    }
                }
                MasaDasha civil = sky.chart().found(instant, kathmandu, 20_700, ChartOptions.builder()
                        .masaPravesha(MasaPraveshaRequest.builder().year(3).place(AnnualPlace.BIRTH)
                                .dasha(MasaDashaRules.builder().clock(MonthClock.DAYS).build()).build())
                        .build()).masaPravesha().orElseThrow().get(0).dasha();
                same(MonthClock.DAYS, civil.rules().clock(), "the clock asked for");
                same(MasaDashaSign.OCCUPIED, civil.rules().sign(), "the default sign named");
                check(Math.abs(civil.periods().get(4).ends() - civil.periods().get(0).starts() - 30) < 1e-9,
                        "thirty civil days");
                // Beside a month, a place founds each day and reads its lord; the first day's is the month's.
                List<MasaPravesha> daily = sky.chart().found(instant, kathmandu, 20_700,
                        ChartOptions.builder().masaPravesha(MasaPraveshaRequest.builder().year(3).month(2)
                                .place(AnnualPlace.BIRTH).build()).build()).masaPravesha().orElseThrow();
                same(30, daily.size(), "thirty days");
                same(lords.get(1).masesha().graha(), daily.get(0).dinesha().graha(), "the first day's lord is the month's");
                for (MasaPravesha row : daily) {
                    check(row.masesha() == null, "a day carries no month's lord");
                    same(row.dinesha().chain().graha(), row.dinesha().graha(), "the chain's lord");
                    same(lords.get(1).masesha().bearers().masaLagna(), row.dinesha().bearers().masaLagna(),
                            "the month lagna's lord");
                    // Each day's results beside its lord: the four lords the day's, the year's,
                    // the month's and the Muntha's.
                    DinaPhala results = row.dayResults();
                    same(12, results.houses().size(), "twelve houses of the day");
                    same(List.of(row.dinesha().graha(), third.graha(), lords.get(1).masesha().graha(),
                            row.dinesha().bearers().muntha()),
                            results.lords().stream().map(DayLordPlace::graha).toList(), "the day's four lords");
                }
                check(daily.stream().map(row -> row.dayResults().moonState()).distinct().count() > 1,
                        "the Moon moves through her states");
                same(30, sky.chart().found(instant, kathmandu, 20_700,
                        ChartOptions.builder().masaPravesha(MasaPraveshaRequest.builder().year(3).month(2)
                                .place(AnnualPlace.BIRTH)
                                .results(MasaPhalaRules.builder().kshina(KshinaRule.DARK_ELEVENTH_TO_NEW_MOON).build())
                                .build()).build()).masaPravesha().orElseThrow().size(), "the waning Moon read otherwise");

                // Each month's results beside its lord: twelve houses, five lords.
                check(months.get(0).results() == null, "no results without a place");
                for (MasaPravesha row : lords) {
                    MasaPhala results = row.results();
                    same(12, results.houses().size(), "twelve houses");
                    same(List.of(MonthLord.values()), results.lords().stream().map(MonthLordPlace::office).toList(),
                            "the five lords in order");
                    same(row.masesha().graha(), results.lords().get(1).graha(), "the Masesha");
                    same(third.graha(), results.lords().get(2).graha(), "the year lord read on the year chart");
                    same(row.masesha().bearers().masaLagna(), results.lords().get(3).graha(), "the month lagna's lord");
                }
                TeistroException floors = refusal(() -> sky.chart().found(instant, kathmandu, 20_700,
                        ChartOptions.builder().masaPravesha(MasaPraveshaRequest.builder().year(3)
                                .place(AnnualPlace.BIRTH)
                                .results(MasaPhalaRules.builder().weakBelow(9 * 3600).strongFrom(3600).build())
                                .build()).build()));
                same("masaPravesha.results.strongFrom", floors.field(), "floors refused by their wire name");
            }
        });

        tests.put("a chart carries its aspect configurations", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().profile("western-tropical-default")
                    .ephemeris(Ephemeris.BUILTIN).build())) {
                Observer palace = new Observer(new Longitude(-0.142), new Latitude(51.501), new Altitude(0));
                double birth = 2_393_783.95;
                check(sky.chart().found(birth, palace, 0, ChartOptions.builder().build()).westernConfigurations()
                        .isEmpty(), "none unless asked");
                Chart chart = sky.chart().found(birth, palace, 0, ChartOptions.builder().outerPlanets(true)
                        .westernAspects(AspectRequest.of(Map.of()))
                        .westernConfigurations(ConfigurationRequest.builder().build()).build());
                List<WesternConfiguration> shapes = chart.westernConfigurations().orElseThrow();
                // Edward VII's two T-squares, as the SDK's own test pins them.
                same(List.of(
                        new WesternConfiguration(WesternConfigurationKind.T_SQUARE,
                                List.of(Graha.MOON, Graha.JUPITER, Graha.URANUS), Graha.JUPITER, null,
                                shapes.get(0).widestFromExactDeg(), List.of()),
                        new WesternConfiguration(WesternConfigurationKind.T_SQUARE,
                                List.of(Graha.MARS, Graha.VENUS, Graha.PLUTO), Graha.MARS, null,
                                shapes.get(1).widestFromExactDeg(), List.of())), shapes, "his two T-squares");
                List<WesternAspectRow> rows = chart.westernAspects().orElseThrow();
                java.util.function.BiPredicate<Graha, Graha> square = (a, b) -> rows.stream().anyMatch(row ->
                        row.aspect() == WesternAspect.SQUARE && java.util.Set.of(row.first(), row.second()).equals(java.util.Set.of(a, b)));
                for (WesternConfiguration shape : shapes) {
                    List<Graha> ends = shape.bodies().stream().filter(g -> g != shape.apex()).toList();
                    check(square.test(shape.apex(), ends.get(0)) && square.test(shape.apex(), ends.get(1)),
                            "the apex square to both ends");
                    check(shape.widestFromExactDeg() > 0 && shape.widestFromExactDeg() <= 10, "inside the orbs");
                }
                TeistroException refused = refusal(() -> sky.chart().found(birth, palace, 0, ChartOptions.builder()
                        .westernConfigurations(ConfigurationRequest.builder().stelliumLeast(2).build()).build()));
                same("westernConfigurations.stelliumLeast", refused.field(), "a refusal names its field");
            }
        });

        tests.put("a chart writes its dossier", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().ephemeris(Ephemeris.BUILTIN).build())) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
                double instant = 2_447_995.489_583_333_5;
                check(sky.chart().found(instant, kathmandu, 20_700, ChartOptions.builder().build()).dossier()
                        .isEmpty(), "none unless asked");
                DossierOptions asked = DossierOptions.builder().sections(List.of("CHART", "GRAHAS", "SHADBALA"))
                        .build();
                String text = sky.chart().found(instant, kathmandu, 20_700,
                        ChartOptions.builder().dossier(asked).build()).dossier().orElseThrow();
                check(text.matches("(?s)# teistro dossier \\d+\n== LEGEND ==\n.*"), "the format line, then the legend");
                List<String> banners = text.lines().filter(line -> line.startsWith("== "))
                        .map(line -> line.substring(3, line.length() - 3)).toList();
                same(List.of("LEGEND", "CHART", "GRAHAS", "SHADBALA"), banners, "the record widened the request");
                check(text.contains("+05:45"), "the request's clock");
                same(text, sky.chart().found(instant, kathmandu, 20_700,
                        ChartOptions.builder().dossier(asked).build()).dossier().orElseThrow(), "the same bytes");
                TeistroException refused = refusal(() -> sky.chart().found(instant, kathmandu, 20_700,
                        ChartOptions.builder().dossier(DossierOptions.builder().sections(List.of("DASHAS")).build())
                                .build()));
                same("dossier.sections[0]", refused.field(), "a refusal names its index");
            }
        });

        tests.put("a chart carries its Lal Kitab", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().ephemeris(Ephemeris.BUILTIN).build())) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
                double instant = 2_447_995.489_583_333_5;
                check(sky.chart().found(instant, kathmandu, 20_700, ChartOptions.builder().build()).lalkitab()
                        .isEmpty(), "none unless asked");
                LalKitab plain = sky.chart().found(instant, kathmandu, 20_700,
                        ChartOptions.builder().lalkitab(LalKitabRequest.of(Map.of())).build()).lalkitab().orElseThrow();
                same(new LalKitab.Cycle(Graha.SATURN, 1), plain.cycle(), "the book's general table");
                check(plain.year() == null, "no year unless asked");
                same(9, plain.reading().planets().size(), "nine planets");
                same(1, plain.periods().get(0).from(), "the cycle from the first year");
                same(120, plain.periods().get(plain.periods().size() - 1).to(), "to the 120th");
                List<List<Integer>> rows = new ArrayList<>();
                for (int y = 0; y < 120; y++) {
                    List<Integer> row = new ArrayList<>();
                    for (int h = 0; h < 12; h++) {
                        row.add((h + y) % 12 + 1);
                    }
                    rows.add(row);
                }
                LalKitab read = sky.chart().found(instant, kathmandu, 20_700, ChartOptions.builder()
                        .lalkitab(LalKitabRequest.of(Map.of("cycle", Map.of("planet", Graha.VENUS, "year", 17), "year", 43,
                                "varshphal", Map.of("rows", rows))))
                        .build()).lalkitab().orElseThrow();
                same(Graha.JUPITER, read.year().ruler(), "the year's ruler");
                same(List.of(Graha.KETU, Graha.JUPITER, Graha.SUN), read.year().thirds(), "its thirds");
                List<LalKitab.Planet> natal = read.reading().planets();
                List<LalKitab.Planet> moved = read.year().annual().planets();
                for (int n = 0; n < natal.size(); n++) {
                    same((natal.get(n).house() + 41) % 12 + 1, moved.get(n).house(), "the annual teva's house");
                }
                TeistroException refused = refusal(() -> sky.chart().found(instant, kathmandu, 20_700,
                        ChartOptions.builder().lalkitab(LalKitabRequest.of(Map.of("year", 121))).build()));
                same("lalkitab.year", refused.field(), "named by its record");
            }
        });

        tests.put("a chart carries its sthira karakas and karaka bhavas", () -> {
            // BPHS ch. 32 vv. 18 to 24 as 03-design/jaimini-significators.md §7
            // tabulates them: the table's order, the verse's candidates, a
            // pair's stronger chosen and saying how, each house from its graha.
            try (Context sky = teistro.context(ContextOptions.builder().profile("nepali-default")
                    .ephemeris(Ephemeris.BUILTIN).build())) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
                List<Signification> significations = List.of(Signification.FATHER, Signification.MOTHER,
                        Signification.SISTER, Signification.WIFES_BROTHER, Signification.YOUNGER_BROTHER,
                        Signification.MATERNAL_RELATIVES, Signification.MATERNAL_UNCLE,
                        Signification.PATERNAL_GRANDFATHER, Signification.HUSBAND, Signification.SON);
                List<List<Graha>> candidates = List.of(List.of(Graha.SUN, Graha.VENUS),
                        List.of(Graha.MOON, Graha.MARS), List.of(Graha.MARS), List.of(Graha.MARS),
                        List.of(Graha.MARS), List.of(Graha.MERCURY), List.of(Graha.MERCURY), List.of(Graha.JUPITER),
                        List.of(Graha.VENUS), List.of(Graha.SATURN));
                List<Graha> from = List.of(Graha.SUN, Graha.MOON, Graha.MARS, Graha.MERCURY, Graha.JUPITER,
                        Graha.VENUS, Graha.SATURN);
                List<Integer> houses = List.of(9, 4, 3, 6, 5, 7, 8);
                List<Signification> shown = List.of(Signification.FATHER, Signification.MOTHER, Signification.BROTHER,
                        Signification.MATERNAL_UNCLE, Signification.SON, Signification.WIFE, Signification.DEATH);
                java.util.Set<ChosenBy> chosen = new java.util.HashSet<>();
                for (int step = 0; step < 40; step += 1) {
                    Chart chart = sky.chart().found(2_451_545.0 + step * 2.9, kathmandu, 20700,
                            ChartOptions.builder().jaimini(true).build());
                    JaiminiReading reading = chart.jaimini().orElseThrow();
                    same(significations, reading.sthiraKarakas().stream().map(SthiraKaraka::signification).toList(),
                            "the table's order");
                    same(candidates, reading.sthiraKarakas().stream().map(SthiraKaraka::candidates).toList(),
                            "the verse's candidates");
                    for (SthiraKaraka k : reading.sthiraKarakas()) {
                        check(k.candidates().contains(k.graha()), k.signification() + ": one of its candidates");
                        check((k.chosenBy() == ChosenBy.ALONE) == (k.candidates().size() == 1),
                                k.signification() + ": alone exactly when the verse names one");
                        if (k.candidates().size() == 2) {
                            chosen.add(k.chosenBy());
                        }
                    }
                    same(from, reading.karakaBhavas().stream().map(KarakaBhava::graha).toList(), "counted from");
                    same(houses, reading.karakaBhavas().stream().map(KarakaBhava::house).toList(), "the houses");
                    same(shown, reading.karakaBhavas().stream().map(KarakaBhava::signification).toList(), "shown");
                    for (KarakaBhava b : reading.karakaBhavas()) {
                        int own = (int) Math.floor(chart.grahas().get(b.graha().id()).longitudeDeg() / 30);
                        same((own + b.house() - 1) % 12, b.sign().id(), b.graha() + "'s house " + b.house());
                    }
                }
                check(chosen.contains(ChosenBy.SIGN) && chosen.size() > 1, "a pair parted more ways than one: " + chosen);
            }
        });

        tests.put("every reading a chart was asked for reads back", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().profile("nepali-default")
                    .ephemeris(Ephemeris.BUILTIN).build())) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
                ChartOptions everything = ChartOptions.builder().state(true).aspects(true).points(true).houses(true)
                        .ashtakavarga(true).vimshopaka(true).vaiseshikamsa(true).dashaPhala(true).jaimini(true)
                        .avakahada(true).moment(true).outerPlanets(true).shadbala(true).bhavaBala(true)
                        .vargas(Varga.D9).dashas(DashaSystem.VIMSHOTTARI).remedies(RemedyRequest.of(Map.of()))
                        .fortitudes(FortitudeRequest.of(Map.of())).lots(LotRequest.of(Map.of())).westernAspects(AspectRequest.of(Map.of()))
                        .westernHouses(HouseRequest.of(Map.of())).antiscia(AntisciaRequest.of(Map.of())).midpoints(MidpointRequest.of(Map.of())).parallels(ParallelRequest.of(Map.of()))
                        .build();
                ChartBatch batch = sky.chart().foundMany(new double[] {2_447_995.489_583_333_5, 2_451_545.0},
                        kathmandu, 20_700, everything);
                for (Chart chart : batch) {
                    same(9, chart.grahas().size(), "nine grahas");
                    same(12, chart.houses().size(), "twelve bhavas");
                    same(12, chart.chalit().size(), "twelve chalit bhavas");
                    same(9, chart.states().size(), "a state per graha");
                    check(!chart.points().isEmpty(), "the derived points");
                    check(!chart.aspects().isEmpty(), "the drishti");
                    check(!chart.bhavas().isEmpty(), "the served houses");
                    check(!chart.outer().isEmpty(), "the outer planets");
                    same(1, chart.vargas().size(), "the navamsha asked for");
                    same(1, chart.dashas().size(), "the Vimshottari asked for");
                    check(chart.ashtakavarga().isPresent(), "the ashtakavarga");
                    check(chart.vimshopaka().isPresent(), "the vimshopaka");
                    check(chart.vaiseshikamsa().isPresent(), "the vaiseshikamsa");
                    check(chart.dashaPhala().isPresent(), "the dasha phala");
                    check(chart.jaimini().isPresent(), "the jaimini reading");
                    check(chart.avakahada().isPresent(), "the avakahada");
                    check(chart.moment().isPresent(), "the moment");
                    check(chart.shadbala().isPresent(), "the shadbala");
                    check(chart.bhavaBala().isPresent(), "the bhava bala");
                    check(chart.kp().isEmpty(), "KP needs its own ayanamsha, and was not asked");
                    check(chart.remedies().isPresent(), "the remedies");
                    check(chart.fortitudes().isPresent(), "the fortitudes");
                    check(chart.lots().isPresent(), "the lots");
                    check(chart.westernAspects().isPresent(), "the western aspects");
                    check(chart.westernHouses().isPresent(), "the western houses");
                    check(chart.antiscia().isPresent(), "the antiscia");
                    check(chart.midpoints().isPresent(), "the midpoints");
                    check(chart.parallels().isPresent(), "the parallels");
                    check(chart.prashna().isEmpty() && chart.matching().isEmpty() && chart.synastry().isEmpty()
                            && chart.rectification().isEmpty(), "what was not asked is empty");
                    check(chart.hits().isEmpty() && chart.gochar().isEmpty() && chart.drawings().isEmpty(),
                            "and every list not asked for is empty");
                    same(chart.states(), chart.states(), "read twice, the same");
                    check(chart.timing() != null && chart.day() != null, "the timing and the day");
                }
                same(Graha.SUN, batch.get(0).grahas().get(0).graha(), "the Sun first");
            }
        });

        tests.put("a chart places the Black Moon Lilith its request names", () -> {
            try (Context western = teistro.context(ContextOptions.builder().profile("western-tropical-default")
                    .ephemeris(Ephemeris.BUILTIN).build())) {
                Observer london = new Observer(new Longitude(0), new Latitude(51.5), new Altitude(0));
                Chart plain = western.chart().found(2_451_545.0, london, 0, ChartOptions.none());
                check(plain.lilith().isEmpty(), "none unless the request names one");
                Chart founded = western.chart().found(2_451_545.0, london, 0,
                        ChartOptions.builder().lilith(Lilith.MEAN).build());
                LilithPosition mean = founded.lilith().orElseThrow();
                // ELP2000-82B's mean perigee at J2000 is 83.3532°, so the apogee
                // stands half a circle on, in Sagittarius (Meeus, ch. 47).
                same(Lilith.MEAN, mean.apogee(), "the mean apogee");
                check(mean.longitudeDeg() > 263 && mean.longitudeDeg() < 264, "in Sagittarius: " + mean.longitudeDeg());
                // The latitude is the frame's rounding, a ten-billionth of a degree.
                check(Math.abs(mean.latitudeDeg()) < 1e-6 && mean.distanceAu() == 0, "on the ecliptic, no distance");
                check(!mean.retrograde(), "direct");
                check(mean.house().bhava() >= 1 && mean.house().bhava() <= 12, "in a bhava");
                same(plain.grahas(), founded.grahas(), "the nine as they were, to the bit");
                LilithPosition osculating = western.chart().found(2_451_545.0, london, 0,
                        ChartOptions.builder().lilith(Lilith.OSCULATING).build()).lilith().orElseThrow();
                same(Lilith.OSCULATING, osculating.apogee(), "the osculating apogee");
                check(osculating.distanceAu() > 0.0026, "the far end of the ellipse");
                // The two apogees part by no more than about thirty degrees.
                double apart = Math.abs((osculating.longitudeDeg() - mean.longitudeDeg() + 540) % 360 - 180);
                check(apart < 31, "within a month of the mean: " + apart);
                check(refused(() -> ChartOptions.builder().lilith(Lilith.NONE).build(), western, london).getMessage()
                        .contains("lilith"), "NONE is refused naming lilith");
            }
        });

        tests.put("a chart's progressions carry their parallels when the window asks them", () -> {
            try (Context leo = teistro.context(ContextOptions.builder().profile("western-tropical-default")
                    .ephemeris(Ephemeris.BUILTIN).build())) {
                Observer london = new Observer(new Longitude(0), new Latitude(51.5), new Altitude(0));
                double birth = 2_400_629.742_361_111;
                // Jupiter "has moved to the parallel of the radical Sun" (p. 33):
                // within Leo's degree as his year opens, and separating.
                Map<String, Object> window = new java.util.HashMap<>(Map.of("from", 2_417_523.5, "to", 2_417_888.5,
                        "grahas", List.of("JUPITER"), "points", List.of(Map.of("point", "GRAHA", "graha", "SUN"))));
                Progressions plain = leo.chart().found(birth, london, 0, ChartOptions.builder()
                        .progressions(ProgressionsRequest.of(Map.of("contacts", window))).build())
                        .progressions().orElseThrow();
                check(plain.parallels() == null, "no parallels unless the window asks them");
                window.put("parallels", Map.of());
                Progressions read = leo.chart().found(birth, london, 0, ChartOptions.builder()
                        .progressions(ProgressionsRequest.of(Map.of("contacts", window))).build())
                        .progressions().orElseThrow();
                same(1, read.parallels().size(), "one parallel");
                ProgressedParallel parallel = read.parallels().get(0);
                same(Graha.JUPITER, parallel.graha(), "the progressed planet");
                same(new NatalPoint("GRAHA", Graha.SUN), parallel.to(), "the radical Sun");
                check(parallel.enters() == null, "in force as the window opens");
                same(2_417_523.5, parallel.closest(), "closest as the window opens");
                check(!parallel.contrary() && parallel.closestDeg() < 1, "within the degree, the same side");
                check(read.mutual() == null, "no mutual contacts unless asked");
                // The progressed Moon quincunx the progressed Venus (p. 41,
                // "P. to P."), July 1907 by Leo's month for each degree.
                Progressions pp = leo.chart().found(birth, london, 0, ChartOptions.builder()
                        .progressions(ProgressionsRequest.of(Map.of("contacts", Map.of("from", 2_417_484.5,
                                "to", 2_417_941.5, "grahas", List.of("MOON", "VENUS"), "aspects", List.of(150),
                                "mutual", true)))).build())
                        .progressions().orElseThrow();
                same(1, pp.mutual().size(), "one mutual contact");
                MutualContact venus = pp.mutual().get(0);
                same(Graha.MOON, venus.first(), "the Moon first");
                same(Graha.VENUS, venus.second(), "Venus second");
                same(150, venus.angle(), "the quincunx");
                check(venus.life() >= 2_417_712.5 && venus.life() < 2_417_773.5, "in Leo's July");
            }
        });

        tests.put("a chart founded under KP's ayanamsha carries its KP reading", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().profile("kp-default")
                    .ephemeris(Ephemeris.BUILTIN).build())) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
                Chart chart = sky.chart().found(2_447_995.489_583_333_5, kathmandu, 20_700,
                        ChartOptions.builder().kp(KpRequest.of(Map.of())).build());
                check(chart.kp().isPresent(), "the KP reading");
            }
        });

        tests.put("a chart read as a birth time to rectify carries each reading asked, and only those", () -> {
            try (Context sky = context(teistro)) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
                Map<String, Object> asked = Map.of(
                        "purify", Map.of("minutes", 20),
                        "conception", Map.of(),
                        "circumstance", Map.of("facts", Map.of("fatherPresent", false)),
                        "baseline", Map.of("uncertaintyMinutes", 30, "sex", "MALE", "events", List.of(
                                Map.of("kind", "MARRIAGE", "on", 2_469_000.5),
                                Map.of("kind", "ACCIDENT", "on", 2_471_000.5, "heldOut", true))),
                        "svarodaya", Map.of("minutes", 20));
                Rectification read = sky.chart().found(2_460_482.5, kathmandu, 20_700,
                        ChartOptions.builder().rectification(RectificationRequest.of(asked)).build()).rectification().orElseThrow();
                Purified purified = read.purified().orElseThrow();
                check(!purified.intervals().isEmpty(), "the purifier leaves a run standing");
                Purified.Clause clause = purified.intervals().get(0).verdict().clauses().get(0);
                same("PRANAPADA", clause.purifier(), "the pranapada judged first");
                check(clause.sign().fullKey().startsWith("rashi."), "a clause's sign is a rashi");
                Conception conception = read.conception().orElseThrow();
                check(conception.moon().moonSign().fullKey().startsWith("rashi."), "the Moon's sign is a rashi");
                check(conception.moon().predicted().nakshatra().isPresent(),
                        "the default count predicts a nakshatra");
                Circumstance circumstance = read.circumstance().orElseThrow();
                same(7, circumstance.sky().grahasDeg().size(), "the seven grahas' longitudes");
                check(circumstance.presentation().lord().fullKey().startsWith("graha."), "the lagna's lord");
                same("FATHER", circumstance.weights().get(0).indication(), "the one fact given, weighed");
                BaselineRectification baseline = read.baseline().orElseThrow();
                check(baseline.candidates().get(0).lagnaNakshatra().fullKey().startsWith("nakshatra."),
                        "a candidate's nakshatra");
                same(1, baseline.holdOut().size(), "the held-out event, tested");
                List<String> kinds = new ArrayList<>();
                for (BaselineRectification.Stage stage : baseline.stages()) {
                    for (BaselineNote note : stage.notes()) {
                        switch (note) {
                            case BaselineNote.TattvaSex sex -> same("MALE", sex.sex(), "the sex asked");
                            case BaselineNote.ReportedTime time ->
                                    same(30.0, time.uncertaintyMinutes(), "the uncertainty asked");
                            case BaselineNote.EventFit fit -> {
                                check(fit.id().isEmpty(), "an event given no name has none");
                                check(fit.lords().stream().allMatch(g -> g.fullKey().startsWith("graha.")),
                                        "the period lords are grahas");
                            }
                        }
                        kinds.add(note.kind());
                    }
                }
                same(List.of("TATTVA_SEX", "REPORTED_TIME", "EVENT_FIT"), kinds, "a note of each kind");
                SvarodayaAround svarodaya = read.svarodaya().orElseThrow();
                List<SvarodayaAround.Run> runs = svarodaya.runs();
                check(!runs.isEmpty(), "the window's runs");
                check(Math.abs(runs.get(0).from() - (2_460_482.5 - 20.0 / 1440.0)) < 1e-9,
                        "the runs start at the window's start");
                check(Math.abs(runs.get(runs.size() - 1).to() - (2_460_482.5 + 20.0 / 1440.0)) < 1e-9,
                        "and end at its end");
                for (int k = 1; k < runs.size(); k += 1) {
                    same(runs.get(k - 1).to(), runs.get(k).from(), "each run starts where the last ended");
                }
                SvarodayaAround.Run now = svarodaya.at().run();
                check(now.from() <= 2_460_482.5 && 2_460_482.5 < now.to(), "the chart's instant falls in its run");
                check(svarodaya.at().tithi().fullKey().startsWith("tithi."), "the sunrise's tithi is a tithi");
                same(now.nadi().equals("MOON") ? Sex.FEMALE : Sex.MALE, now.sex(), "the nadi gives the sex");
                same(2, svarodaya.at().junctions().size(), "the turn's two junctions");

                Rectification alone = sky.chart().found(2_460_482.5, kathmandu, 20_700,
                        ChartOptions.builder().rectification(RectificationRequest.of(Map.of("purify", Map.of("minutes", 20)))).build())
                        .rectification().orElseThrow();
                check(alone.purified().isPresent(), "the purifier asked");
                check(alone.conception().isEmpty() && alone.circumstance().isEmpty() && alone.baseline().isEmpty()
                        && alone.svarodaya().isEmpty(),
                        "what was not asked is empty");
            }
        });

        tests.put("the areas beside the chart answer", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().profile("nepali-default")
                    .ephemeris(Ephemeris.BUILTIN).build())) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
                NumerologyProfile profile = sky.numerology().profile("Ram Bahadur", java.time.LocalDate.of(1990, 4, 14));
                check(profile != null, "a numerology profile");
                check(sky.matching().naam("सीता", "राम") != null, "a naam milan");
                // The typed rules reach the SDK: each turns a refusal into an answer.
                same("numerology.name", refusal(() -> sky.numerology().profile("Ram राम",
                        java.time.LocalDate.of(1990, 4, 14), null)).field(), "a non-Latin letter is refused");
                check(sky.numerology().profile("Ram राम", java.time.LocalDate.of(1990, 4, 14),
                        NumerologyRules.builder().nonLatin(NonLatin.SKIP).build()) != null,
                        "NumerologyRules.nonLatin reaches the SDK");
                check(refusal(() -> sky.matching().naam("Sita", "Rama", null)).field().startsWith("naam."),
                        "a Latin name is refused by default");
                check(sky.matching().naam("Sita", "Rama", NaamRules.builder()
                        .name(NameRules.builder().latin(LatinName.IAST).build()).build()) != null,
                        "NaamRules.name.latin reaches the SDK");
                CalendarDate first = sky.chart().found(2_460_000.0, kathmandu, 20_700, ChartOptions.builder().build())
                        .day().date();
                Almanac week = sky.almanac().of(first,
                        sky.calendar().dateOf(first.calendar(), sky.calendar().fixedOf(first) + 6), kathmandu, 20_700);
                same(7, week.size(), "a day per date");
                // A festival list without the days: founded as only what the
                // rules read, and the almanac's own answer to the bit, stamp
                // and all.
                CalendarDate autumn = Civil.date(Calendar.GREGORIAN, 2026, 10, 15);
                CalendarDate winter = Civil.date(Calendar.GREGORIAN, 2026, 11, 10);
                var pack = new com.teispace.teistro.request.FestivalRequest("DHARMASINDHU");
                FestivalAnswer withDays = sky.almanac().of(autumn, winter, kathmandu, 20_700, null, pack,
                        false, false, false).festivals().orElseThrow();
                FestivalAnswer lean = sky.almanac().festivals(autumn, winter, kathmandu, 20_700, pack);
                same(withDays, lean, "the festivals without the days are the ones with them");
                check(!lean.observances().isEmpty()
                        && lean.observances().stream().allMatch(o -> o.marginSeconds() >= 0),
                        "each observance says how far the place can move");
                check(!week.get(0).tithi().isEmpty(), "the tithis");
                int varjyam = 0;
                for (AlmanacDay day : week) {
                    for (Span<Nakshatra> window : day.varjyam()) {
                        Span<Nakshatra> star = day.nakshatra().stream()
                                .filter(span -> span.member() == window.member()).findFirst().orElseThrow();
                        double length = window.whole().toJd() - window.whole().fromJd();
                        check(Math.abs(length * 15 - (star.whole().toJd() - star.whole().fromJd())) < 1e-6,
                                "a varjyam is four sixtieths of its star");
                        varjyam += 1;
                    }
                }
                check(varjyam >= 5 && varjyam <= 9, "about a varjyam a day: " + varjyam);
                // Every vara censures one muhurta or two (crux C398); the Moon's
                // signs carry where she dwells and whose chandrashtama it is, the
                // eighth sign from the natal one (C403, C404); and a shool is a
                // nakshatra's own run (C402).
                int shools = 0;
                for (AlmanacDay day : week) {
                    long censured = day.muhurtas().stream().filter(Muhurta::durmuhurta).count();
                    check(censured >= 1 && censured <= 2, "a durmuhurta or two a day: " + censured);
                    List<Span<Rashi>> signs = day.moonSigns();
                    same(signs.size(), day.chandraVasa().size(), "a dwelling a sign");
                    same(signs.size(), day.chandrashtama().size(), "a chandrashtama a sign");
                    for (int k = 0; k < signs.size(); k += 1) {
                        same(signs.get(k).whole(), day.chandraVasa().get(k).whole(), "the vasa is the sign's run");
                        Span<Rashi> ashtama = day.chandrashtama().get(k);
                        same(signs.get(k).whole(), ashtama.whole(), "the chandrashtama is the sign's run");
                        same(signs.get(k).member().id(), (ashtama.member().id() + 7) % 12, "the eighth from the natal sign");
                    }
                    for (Span<Direction> shool : day.nakshatraShool()) {
                        check(day.nakshatra().stream().anyMatch(star -> star.whole().equals(shool.whole())),
                                "a shool is a nakshatra's own run");
                        shools += 1;
                    }
                }
                check(shools >= 1 && shools <= 3, "a shool or two a week: " + shools);
                RashifalSealed read = sky.chart().rashifal(RashifalRequest.of(first, kathmandu, 20_700));
                check(read.period() != null, "a rashifal period");
                check(sky.chart().rashifal(RashifalRequest.of(first, kathmandu, 20_700)
                        .withSnapshot(new SnapshotClock(6, 0))).period() != null, "a rashifal at six");
                check(!read.provenance().inputHash().isEmpty(), "the request is sealed");
                same(read.provenance().inputHash(),
                        sky.chart().rashifalMany(List.of(RashifalRequest.of(first, kathmandu, 20_700)), null)
                                .provenance().inputHash(),
                        "a batch of one seals as the one does");
                same(Status.UNSUPPORTED, refusal(() -> sky.engine().names()).status(),
                        "the built-in engine describes no operations of its own");
            }
        });

        tests.put("a consumer's dasha system reads back by its key, as the row it copies", () -> {
            String shashti = "[{\"kernel\":\"UDU\",\"key\":\"ACME_SHASHTI\",\"lords\":["
                    + "{\"graha\":\"JUPITER\",\"years\":10},{\"graha\":\"SUN\",\"years\":10},"
                    + "{\"graha\":\"MARS\",\"years\":10},{\"graha\":\"MOON\",\"years\":6},"
                    + "{\"graha\":\"MERCURY\",\"years\":6},{\"graha\":\"VENUS\",\"years\":6},"
                    + "{\"graha\":\"SATURN\",\"years\":6},{\"graha\":\"RAHU\",\"years\":6}],"
                    + "\"reference\":\"ASHWINI\",\"groups\":[3,4,3,4,3,4,3,4],"
                    + "\"wheel\":\"WITH_ABHIJIT\",\"repeats\":false}]";
            try (Context sky = teistro.context(ContextOptions.builder().testProvider(true).dashasJson(shashti).build())) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
                List<Dasha> read = sky.chart().found(2_451_545.0, kathmandu, 20_700, ChartOptions.builder()
                        .dashas("dasha_system.ACME_SHASHTI", DashaSystem.SHASHTIHAYANI).build()).dashas();
                same("dasha_system.ACME_SHASHTI", read.get(0).system(), "the consumer's key, not UNKNOWN");
                same(DashaSystem.SHASHTIHAYANI, read.get(1).system(), "the catalogue's member");
                same(read.get(1).periods(), read.get(0).periods(), "period for period as the text's row");
            }
        });

        tests.put("an event study reads a dasha system the context registered", () -> {
            String twin = "[{\"kernel\":\"UDU\",\"key\":\"ACME_VIMSHOTTARI\",\"lords\":["
                    + "{\"graha\":\"KETU\",\"years\":7},{\"graha\":\"VENUS\",\"years\":20},"
                    + "{\"graha\":\"SUN\",\"years\":6},{\"graha\":\"MOON\",\"years\":10},"
                    + "{\"graha\":\"MARS\",\"years\":7},{\"graha\":\"RAHU\",\"years\":18},"
                    + "{\"graha\":\"JUPITER\",\"years\":16},{\"graha\":\"SATURN\",\"years\":19},"
                    + "{\"graha\":\"MERCURY\",\"years\":17}],\"reference\":\"ASHWINI\"}]";
            try (Context sky = teistro.context(
                    ContextOptions.builder().ephemeris(Ephemeris.BUILTIN).dashasJson(twin).build())) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(0));
                List<ResearchSubject> lives = new java.util.ArrayList<>();
                for (int i = 0; i < 6; i += 1) {
                    double born = 2447000.25 + 977.3 * i;
                    lives.add(new ResearchSubject(new ResearchBirth(born, kathmandu, 20_700), born + 9000.5));
                }
                com.teispace.teistro.request.RuleRequest yogas = com.teispace.teistro.request.RuleRequest.builder()
                        .shipped(List.of(com.teispace.teistro.request.ShippedRules.YOGAS)).build();
                ResearchTested own = sky.research().timed(lives, yogas, "dasha_system.ACME_VIMSHOTTARI",
                        ResearchEventShuffle.AGES_AT_EVENT, ResearchEventTest.builder(1, 19).build(), null);
                ResearchTested shipped = sky.research().timed(lives, yogas, DashaSystem.VIMSHOTTARI,
                        ResearchEventShuffle.AGES_AT_EVENT, ResearchEventTest.builder(1, 19).build(), null);
                same(shipped.rows(), own.rows(), "delivered as the twin's periods deliver");
                check(!own.provenance().inputHash().equals(shipped.provenance().inputHash()),
                        "another registration sealed");
                TeistroException unknown = refusal(() -> sky.research().timed(lives, yogas,
                        "dasha_system.ACME_OTHER", ResearchEventShuffle.AGES_AT_EVENT,
                        ResearchEventTest.builder(1, 19).build(), null));
                same("dasha", unknown.field(), "a key the context did not register");
            }
        });

        tests.put("a closed enum refuses an id that is no member", () -> {
            try {
                Status.of(12345);
                throw new AssertionError("an unknown status read");
            } catch (IllegalArgumentException expected) {
                check(expected.getMessage().contains("12345"), "names the id");
            }
        });

        tests.put("a provider written in Java answers a chart, and a throw reaches the caller as itself", () -> {
            Line line = new Line(null, List.of(Body.values()));
            Observer place = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
            try (Context sky = teistro.context(ContextOptions.builder().profile("parashari-classical")
                    .provider(line).build())) {
                same(java.util.Optional.of(line), sky.provider().map(p -> (Line) p), "the context names its provider");
                Chart chart = sky.chart().found(2_451_545.0, place, 20700, ChartOptions.none());
                check(line.calls > 0, "founding a chart asked the provider");
                check(chart.lagnaDeg() >= 0 && chart.lagnaDeg() < 360, "a lagna");
            }
            IllegalStateException thrown = new IllegalStateException("the index is corrupt");
            try (Context sky = teistro.context(ContextOptions.builder().profile("parashari-classical")
                    .provider(new Line(thrown)).build())) {
                try {
                    sky.positions(new double[] {2_451_545.0}, List.of(Body.SUN));
                    throw new AssertionError("the provider's throw was swallowed");
                } catch (IllegalStateException found) {
                    check(found == thrown, "the provider's own exception, not a copy");
                    check(found.getSuppressed().length == 1 && found.getSuppressed()[0] instanceof TeistroException,
                            "the library's refusal is kept beside it");
                }
                // The next call starts clean: a throw is never read from an
                // earlier call (`error-carries-its-record`).
                TeistroException refusal = refusal(() -> sky.positions(new double[] {2_451_545.0}, List.of(Body.SATURN)));
                check(refusal.getMessage().contains("SATURN"), "a later refusal is the library's own");
            }
        });

        tests.put("a provider's column of the wrong length is refused, never padded", () -> {
            EphemerisProvider short_ = new Line(null) {
                @Override
                public java.util.Optional<PositionAnswer> positions(PositionQuery query) {
                    double[] one = new double[1];
                    return java.util.Optional.of(PositionAnswer.of(one, one, one));
                }
            };
            try (Context sky = teistro.context(ContextOptions.builder().profile("parashari-classical")
                    .provider(short_).build())) {
                IllegalStateException found = null;
                try {
                    sky.positions(new double[] {2_451_545.0, 2_451_546.0}, List.of(Body.SUN));
                } catch (IllegalStateException e) {
                    found = e;
                }
                check(found != null && found.getMessage().contains("1 values in `lon` for 2 cells"),
                        "the length named: " + found);
            }
        });

        tests.put("a provider's binding is released with its context, every time", () -> {
            for (int round = 0; round < 200; round += 1) {
                Line line = new Line(null);
                Context sky = teistro.context(ContextOptions.builder().profile("parashari-classical")
                        .provider(line).build());
                sky.positions(new double[] {2_451_545.0 + round}, List.of(Body.SUN, Body.MOON));
                sky.close();
                sky.close();
                same(1, line.calls, "round " + round + " asked once");
            }
            check(refusedProvider(() -> new Line(null) {
                @Override
                public String name() {
                    return "";
                }
            }).getMessage().contains("must have a name"), "a nameless provider is refused before it is bound");
        });

        tests.put("an ephemeris chain is tried in order and refuses naming each", () -> {
            // An adapter that is not there, then the built-in: the fallback
            // the caller wrote down.
            try (Context fell = teistro.context(ContextOptions.builder().profile("parashari-classical")
                    .ephemeris(Plugin.of("/nowhere/adapter.so"), EphemerisChoice.of(Ephemeris.BUILTIN)).build())) {
                double sun = fell.positions(new double[] {2_451_545.0}, List.of(Body.SUN)).at(0, 0).longitude();
                check(Math.abs(sun - 280.37) < 0.5, "the built-in answered: " + sun);
            }
            IllegalArgumentException none = null;
            try {
                teistro.context(ContextOptions.builder().ephemeris(Plugin.of("/a.so"), Plugin.of("/b.so")).build());
            } catch (IllegalArgumentException e) {
                none = e;
            }
            check(none != null && none.getMessage().contains("/a.so") && none.getMessage().contains("/b.so"),
                    "one refusal names each entry: " + none);
            IllegalArgumentException empty = null;
            try {
                ContextOptions.builder().ephemeris(new EphemerisChoice[0]);
            } catch (IllegalArgumentException e) {
                empty = e;
            }
            check(empty != null && empty.getMessage().contains("names nothing"), "a chain of none is a mistake");
            IllegalArgumentException both = null;
            try {
                ContextOptions.builder().ephemeris(Ephemeris.BUILTIN).provider(new Line(null)).build();
            } catch (IllegalArgumentException e) {
                both = e;
            }
            check(both != null && both.getMessage().contains("give one of them"), "a provider and an ephemeris");
        });

        tests.put("the Surya Siddhanta takes its sunrise from the adapter after it", () -> {
            // Nepal's official days: the text takes the adapters after it, as
            // the façade's chain does. Before this, each entry opened alone
            // and the adapter after the text was never asked.
            java.util.function.Function<ContextOptions, String> stamp = options -> {
                try (Context sky = teistro.context(options)) {
                    return sky.positions(new double[] {2_451_545.0}, List.of(Body.SUN)).provenance().provider()
                            .dataVersion();
                }
            };
            // An adapter that is not there is passed over, as the façade
            // passes over it, and the text keeps the built-in's sunrise.
            String fellBack = stamp.apply(
                    ContextOptions.builder().nepalOfficial(Plugin.of("/nowhere/adapter.so")).build());
            check(fellBack.contains("sunrise from teistro-builtin"), "the built-in's sunrise: " + fellBack);
            String adapter = System.getenv("TEISTRO_TEIMERIS_ADAPTER");
            if (adapter == null || adapter.isEmpty()) {
                System.out.println("      (no TEISTRO_TEIMERIS_ADAPTER: the plugin itself was not loaded)");
                return;
            }
            String official = stamp.apply(
                    ContextOptions.builder().nepalOfficial(Plugin.of(adapter)).locale("en-Latn").build());
            check(official.contains("sunrise from teimeris"), "the engine's sunrise: " + official);
            same(ContextOptions.NEPAL_OFFICIAL_PROFILE,
                    ContextOptions.builder().nepalOfficial().build().profile(), "the committee's profile");
        });

        tests.put("two reckonings say where they disagree and why", () -> {
            // November 2025 at Kathmandu, the committee's reckoning over the
            // Surya Siddhanta against the modern one: each tithi difference
            // names both members and the instant they meet under each sky,
            // on opposite sides of its sunrise, and Haribodhini Ekadashi is
            // fasted on the 1st by the text and the 2nd by the modern sky.
            Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
            CalendarDate first = Civil.date(Calendar.GREGORIAN, 2025, 11, 1);
            CalendarDate last = Civil.date(Calendar.GREGORIAN, 2025, 11, 30);
            var nepal = new com.teispace.teistro.request.FestivalRequest("NEPAL");
            try (Context official = teistro.context(ContextOptions.builder().nepalOfficial().build());
                    Context modern = teistro.context(ContextOptions.builder().profile("nepali-default")
                            .ephemeris(Ephemeris.BUILTIN).build())) {
                Disagreements found = official.almanac().disagreements(modern, first, last, kathmandu, 20_700,
                        true, nepal);
                Disagreements.Days days = found.days();
                same(30, days.compared(), "every day compared");
                same("nepali-default", days.otherProfile(), "the other's profile");
                same(modern.settingsHash(), days.otherSettingsHash(), "the other's settings hash");
                same(List.of(1, 2, 9, 10, 11),
                        days.days().stream().filter(d -> d.tithi() != null).map(d -> d.date().day()).toList(),
                        "the tithi disagreements");
                Disagreements.Limb<Tithi> tithi = days.days().get(0).tithi();
                same(Tithi.SHUKLA_EKADASHI, tithi.official().member(), "the text's 11th");
                same(Tithi.SHUKLA_DASHAMI, tithi.other().member(), "the modern 10th");
                check(tithi.official().change().minutesFromSunrise() < 0
                        && tithi.other().change().minutesFromSunrise() > 0, "opposite sides of sunrise");
                Disagreements.Ekadashi haribodhini = found.festivals().ekadashis().stream()
                        .filter(f -> f.tithi() == Tithi.SHUKLA_EKADASHI).findFirst().orElseThrow();
                same(List.of(1, 2), List.of(haribodhini.official().day().day(), haribodhini.other().day().day()),
                        "Haribodhini Ekadashi moves a day");

                Disagreements lean = official.almanac().disagreements(modern, first, last, kathmandu, 20_700,
                        false, nepal);
                check(lean.days() == null, "the days not asked");
                same(found.festivals().ekadashis(), lean.festivals().ekadashis(), "the festivals alike");
                same("disagreements.days", refusal(() -> official.almanac().disagreements(modern, first, last,
                        kathmandu, 20_700, false, null)).field(), "a comparison of nothing is refused by name");
                // The other way round, each holding the other's lock: the same
                // days differ, which also proves the two locks are taken whatever
                // the order the caller names them in.
                same(found.days().days().size(), modern.almanac().disagreements(official, first, last, kathmandu,
                        20_700, true, null).days().days().size(), "the other way round, the same days");
            }
        });

        tests.put("an almanac writes its festivals as an iCalendar feed", () -> {
            // Dashain 2025 at Kathmandu: one event for each observance and fast
            // the festival answer gives, Vijaya Dashami on 2 October, every
            // line CRLF and at most 75 octets, the same request the same bytes.
            Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
            CalendarDate first = Civil.date(Calendar.GREGORIAN, 2025, 9, 20);
            CalendarDate last = Civil.date(Calendar.GREGORIAN, 2025, 11, 10);
            var panchanga = new com.teispace.teistro.request.FestivalRequest("NEPAL_PANCHANGA");
            try (Context sky = teistro.context(ContextOptions.builder().nepalOfficial().build())) {
                FeedOptions options = FeedOptions.defaults().withName("Dashain").withStampUnixSeconds(1_767_225_600L);
                String ics = sky.almanac().festivalCalendar(first, last, kathmandu, 20_700, panchanga, options);
                check(ics.startsWith("BEGIN:VCALENDAR\r\nVERSION:2.0\r\n"), "a calendar");
                check(ics.contains("X-WR-CALNAME:Dashain\r\n"), "its name");
                FestivalAnswer answer = sky.almanac().festivals(first, last, kathmandu, 20_700, panchanga);
                same(answer.observances().size() + answer.ekadashis().size() + answer.dated().size(),
                        ics.split("BEGIN:VEVENT", -1).length - 1, "one event an occurrence");
                DatedObservance tula = answer.dated().get(0);
                same("TULA_SANKRANTI", tula.rule(), "Kartik's sankranti");
                same(java.util.List.of(10, 18), java.util.List.of(tula.day().month(), tula.day().day()), "on 18 October");
                same(java.util.List.of(Calendar.BIKRAM_SAMBAT, 7, 1),
                        java.util.List.of(tula.on().calendar(), tula.on().month(), tula.on().day()), "Kartik 1");
                check(ics.replace("\r\n ", "").contains(
                        "DTSTART;VALUE=DATE:20251002\r\nDTEND;VALUE=DATE:20251003\r\nSUMMARY:Vijaya Dashami\r\n"),
                        "Vijaya Dashami on 2 October");
                for (String line : ics.split("\r\n")) {
                    check(line.getBytes(java.nio.charset.StandardCharsets.UTF_8).length <= 75, "folded: " + line);
                }
                same(ics, sky.almanac().festivalCalendar(first, last, kathmandu, 20_700, panchanga, options),
                        "the same bytes");
                check(!sky.almanac().festivalCalendar(first, last, kathmandu, 20_700, panchanga,
                        options.withoutRefresh()).contains("REFRESH-INTERVAL"), "no refresh when asked for none");
                check(sky.almanac().festivalCalendar(first, last, kathmandu, 20_700, panchanga).contains("DTSTAMP:"),
                        "stamped now when left out");
            }
        });

        tests.put("a provider written in Java stands behind the Surya Siddhanta", () -> {
            // A chain entry too, so the text takes its modern sunrise from
            // it under the committee's profile, and what it throws there
            // reaches the caller as itself.
            Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
            Line line = new Line(null);
            try (Context sky = teistro.context(ContextOptions.builder()
                    .nepalOfficial(EphemerisChoice.of(line)).build())) {
                String stamp = sky.positions(new double[] {2_451_545.0}, List.of(Body.SUN)).provenance().provider()
                        .dataVersion();
                check(stamp.contains("sunrise from line"), "the provider's sunrise: " + stamp);
                line.calls = 0;
                try {
                    sky.chart().found(2_451_545.0, kathmandu, 20_700, ChartOptions.none());
                } catch (TeistroException noSky) {
                    // A straight line is no sky; what is held is that it was asked.
                }
                check(line.calls > 0, "the provider behind the text was asked");
            }
            IllegalStateException thrown = new IllegalStateException("the engine is not ready");
            try (Context failing = teistro.context(ContextOptions.builder()
                    .nepalOfficial(EphemerisChoice.of(new Line(thrown))).build())) {
                try {
                    failing.chart().found(2_451_545.0, kathmandu, 20_700, ChartOptions.none());
                    throw new AssertionError("the provider's exception did not reach the caller");
                } catch (IllegalStateException found) {
                    check(found == thrown, "the provider's own exception, not a copy");
                }
            }
        });

        tests.put("an ephemeris is plugged in by naming its platform binary", () -> {
            // Runs only where the adapter is built, as every binding's plugin
            // test does: a checkout has neither the adapter nor its data.
            String adapter = System.getenv("TEISTRO_TEIMERIS_ADAPTER");
            if (adapter == null || adapter.isEmpty()) {
                System.out.println("      (no TEISTRO_TEIMERIS_ADAPTER: the plugin itself was not loaded)");
                return;
            }
            try (Context sky = teistro.context(ContextOptions.builder().profile("parashari-classical")
                    .ephemeris(Plugin.of(adapter)).build())) {
                double sun = sky.positions(new double[] {2_451_545.0}, List.of(Body.SUN)).at(0, 0).longitude();
                check(Math.abs(sun - 280.37) < 0.5, "a real engine answered: " + sun);
                // And its own functions came with it, which no SDK operation
                // offers.
                same("teimeris", ((Map<?, ?>) sky.engine().manifest()).get("engine"), "the engine's manifest");
                same("Sun", ((Map<?, ?>) sky.engine().call("tm_body_name", Map.of("body", 0))).get("buf"),
                        "the engine's own function");
            }
        });

        tests.put("JSON is read strictly", () -> {
            same(List.of(1L, 2.5, "x", true), Json.read("[1, 2.5, \"x\", true]"), "values");
            for (String bad : List.of("{\"a\": 1, \"a\": 2}", "[1,]", "01", "[1] [2]", "\"\\q\"")) {
                try {
                    Json.read(bad);
                    throw new AssertionError("read " + bad);
                } catch (IllegalArgumentException expected) {
                    // refused, as it should be
                }
            }
        });

        tests.put("every record's example request reads into its type and writes back as itself", () -> {
            // The fixture the agent server's tests send too: one request a
            // record, so a request type that reads it wrong fails here.
            List<?> examples = (List<?>) Json.read(java.nio.file.Files.readString(
                    java.nio.file.Path.of("crates/sdk/tests/requests.json")));
            java.util.Set<String> untyped = java.util.Set.of("engine.call", "engine.manifest");
            java.util.Set<String> read = new java.util.TreeSet<>();
            for (Object example : examples) {
                Map<?, ?> one = (Map<?, ?>) example;
                String record = (String) one.get("record");
                if (untyped.contains(record)) {
                    continue;
                }
                read.add(record);
                RequestJson request = Requests.read(record, one.get("request"));
                same(request, Requests.read(record, Json.read(Json.write(request))), record + " written and read again");
            }
            same(new java.util.TreeSet<>(Requests.records()), read, "the records with a typed request");
        });

        tests.put("every reading a chart's options name takes the request type the SDK reads", () -> {
            // `ChartOptions` is written by hand and `ChartFoundRequest` is
            // generated from the SDK's schema of the same request, so each
            // setter named after a field of the one takes the other's type:
            // a setter left on `Object`, or on the wrong type, fails here.
            Map<String, java.lang.reflect.Type> fields = new java.util.HashMap<>();
            for (java.lang.reflect.RecordComponent field : ChartFoundRequest.class.getRecordComponents()) {
                fields.put(field.getName(), field.getGenericType());
            }
            int checked = 0;
            for (java.lang.reflect.Method setter : ChartOptions.Builder.class.getMethods()) {
                java.lang.reflect.Type field = fields.get(setter.getName());
                if (setter.getParameterCount() != 1 || field == null
                        || !(field instanceof Class<?> type && RequestJson.class.isAssignableFrom(type))) {
                    continue;
                }
                same(field, setter.getGenericParameterTypes()[0], "ChartOptions.Builder." + setter.getName());
                checked += 1;
            }
            check(checked >= 20, "the readings were found: " + checked);
        });

        tests.put("a request type refuses what the SDK would, naming the field", () -> {
            try {
                Requests.read("time.convert", Json.read("{\"jd\": 2451545.0, \"from\": \"UTX\", \"to\": \"TT\"}"));
                throw new AssertionError("a scale the SDK does not read was read");
            } catch (IllegalArgumentException e) {
                check(e.getMessage().contains("ScaleRequest.from"), e.getMessage());
            }
            try {
                VarshaRequest.builder().through(70_000).build();
                throw new AssertionError("a year past a u16 was taken");
            } catch (IllegalArgumentException e) {
                check(e.getMessage().contains("VarshaRequest.through"), e.getMessage());
            }
            try {
                VarshaRequest.builder().build();
                throw new AssertionError("a request without its required year was built");
            } catch (NullPointerException e) {
                check(e.getMessage().contains("VarshaRequest.through"), e.getMessage());
            }
        });

        List<String> failed = new ArrayList<>();
        tests.put("every release platform is named from what its JVM reports", () -> {
            // One row per platform the release builds; `check-lints` holds
            // this list to `xtask/src/platform.rs`.
            String[][] hosts = {
                {"Linux", "amd64", "false", "linux-x64"},
                {"Linux", "aarch64", "false", "linux-arm64"},
                {"Linux", "amd64", "true", "linux-x64-musl"},
                {"Linux", "aarch64", "true", "linux-arm64-musl"},
                {"Mac OS X", "aarch64", "false", "darwin-arm64"},
                {"Mac OS X", "x86_64", "false", "darwin-x64"},
                {"Windows 11", "amd64", "false", "win32-x64"},
                {"Windows Server 2025", "aarch64", "false", "win32-arm64"},
            };
            for (String[] host : hosts) {
                same(host[3], Host.platform(host[0], host[1], Boolean.parseBoolean(host[2])), host[0] + " " + host[1]);
            }
            // musl is a Linux question only, and an architecture no release
            // builds is named as itself, so the refusal can say it.
            same("darwin-arm64", Host.platform("Mac OS X", "aarch64", true), "no musl off Linux");
            same("linux-riscv64", Host.platform("Linux", "riscv64", false), "an unbuilt architecture");
            same("teistro_ffi.dll", Host.fileName("Windows 11"), "Windows");
            same("libteistro_ffi.dylib", Host.fileName("Mac OS X"), "macOS");
            same("libteistro_ffi.so", Host.fileName("Linux"), "Linux");
        });

        tests.put("a packaged library is written once, hashed, and refused when it is not the one staged", () -> {
            java.nio.file.Path root = java.nio.file.Files.createTempDirectory("teistro-cache-test");
            byte[] bytes = "a library's bytes".getBytes(java.nio.charset.StandardCharsets.UTF_8);
            String digest = java.util.HexFormat.of().formatHex(
                    java.security.MessageDigest.getInstance("SHA-256").digest(bytes));
            java.util.function.Supplier<java.io.InputStream> stream = () -> new java.io.ByteArrayInputStream(bytes);
            java.nio.file.Path written = NativeCache.extract(stream.get(), digest, root, "9.9.9", "lib.so");
            same(digest, NativeCache.sha256(written), "written whole");
            same(written, NativeCache.extract(stream.get(), digest, root, "9.9.9", "lib.so"), "found again");
            // Bytes that do not hash to the staged digest leave nothing behind.
            String other = "0".repeat(64);
            try {
                NativeCache.extract(stream.get(), other, root, "9.9.9", "lib.so");
                throw new AssertionError("a mismatched library was written");
            } catch (java.io.IOException expected) {
                check(expected.getMessage().contains(other), "names the digest: " + expected.getMessage());
            }
            try (var left = java.nio.file.Files.list(root.resolve("teistro-9.9.9-" + other))) {
                same(0L, left.filter(path -> !path.getFileName().toString().equals(NativeCache.LOCK_FILE)).count(),
                        "no library or part file left behind");
            }
            // A cached file someone changed is refused, not replaced.
            if (root.getFileSystem().supportedFileAttributeViews().contains("posix")) {
                java.nio.file.Files.setPosixFilePermissions(written,
                        java.nio.file.attribute.PosixFilePermissions.fromString("rw-------"));
            }
            java.nio.file.Files.write(written, "tampered".getBytes(java.nio.charset.StandardCharsets.UTF_8));
            try {
                NativeCache.extract(stream.get(), digest, root, "9.9.9", "lib.so");
                throw new AssertionError("a tampered library was used");
            } catch (java.io.IOException expected) {
                check(expected.getMessage().contains("delete it"), "says what to do: " + expected.getMessage());
            }
        });

        tests.put("threads extracting at once converge on one file", () -> {
            java.nio.file.Path root = java.nio.file.Files.createTempDirectory("teistro-cache-race");
            byte[] bytes = new byte[1 << 20];
            new java.util.Random(7).nextBytes(bytes);
            String digest = java.util.HexFormat.of().formatHex(
                    java.security.MessageDigest.getInstance("SHA-256").digest(bytes));
            List<java.util.concurrent.Future<java.nio.file.Path>> found = new ArrayList<>();
            try (var pool = java.util.concurrent.Executors.newFixedThreadPool(8)) {
                for (int i = 0; i < 8; i++) {
                    found.add(pool.submit(() -> NativeCache.extract(new java.io.ByteArrayInputStream(bytes), digest,
                            root, "9.9.9", "lib.so")));
                }
            }
            java.nio.file.Path first = found.get(0).get();
            for (var one : found) {
                same(first, one.get(), "one file");
            }
            try (var left = java.nio.file.Files.list(first.getParent())) {
                same(List.of("lib.so"), left.map(path -> path.getFileName().toString())
                        .filter(name -> !name.equals(NativeCache.LOCK_FILE)).sorted().toList(),
                        "and no part files");
            }
        });

        tests.put("a cache directory another may write to is refused", () -> {
            java.nio.file.Path root = java.nio.file.Files.createTempDirectory("teistro-cache-shared");
            if (!root.getFileSystem().supportedFileAttributeViews().contains("posix")) {
                return;
            }
            java.nio.file.Path dir = java.nio.file.Files.createDirectory(root.resolve("teistro-9.9.9-" + "1".repeat(64)));
            java.nio.file.Files.setPosixFilePermissions(dir,
                    java.nio.file.attribute.PosixFilePermissions.fromString("rwxrwxrwx"));
            try {
                NativeCache.extract(new java.io.ByteArrayInputStream(new byte[0]), "1".repeat(64), root, "9.9.9",
                        "lib.so");
                throw new AssertionError("a shared directory was used");
            } catch (java.io.IOException expected) {
                check(expected.getMessage().contains(NativeCache.CACHE_PROPERTY), "says how: " + expected.getMessage());
            }
            java.nio.file.Path fresh = NativeCache.extract(new java.io.ByteArrayInputStream(new byte[0]),
                    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", root, "9.9.9", "lib.so");
            same("rwx------", java.nio.file.attribute.PosixFilePermissions.toString(
                    java.nio.file.Files.getPosixFilePermissions(fresh.getParent())), "a new directory is private");
        });

        for (Map.Entry<String, Test> test : tests.entrySet()) {
            try {
                test.getValue().run();
                System.out.println("ok    " + test.getKey());
            } catch (Exception | AssertionError e) {
                failed.add(test.getKey());
                System.out.println("FAIL  " + test.getKey() + ": " + e);
            }
        }
        System.out.println(tests.size() - failed.size() + " passed, " + failed.size() + " failed, against "
                + teistro.path());
        if (!failed.isEmpty()) {
            System.exit(1);
        }
    }
}
