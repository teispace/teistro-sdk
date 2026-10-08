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

        tests.put("a chart carries its essential dignities in the Chaldean order", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().profile("conformance-baseline")
                    .ephemeris(Ephemeris.BUILTIN).build())) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(0));
                Chart bare = sky.chart().found(2_460_676.5, kathmandu, 20_700, ChartOptions.builder().build());
                check(bare.dignities().isEmpty(), "not asked, not read");
                Dignities read = sky.chart().found(2_460_676.5, kathmandu, 20_700,
                        ChartOptions.builder().dignities(Map.of()).build()).dignities().orElseThrow();
                same(SectRule.HORIZON, read.sectRule(), "the horizon decides the sect");
                same(List.of(Graha.SATURN, Graha.JUPITER, Graha.MARS, Graha.SUN, Graha.VENUS, Graha.MERCURY,
                        Graha.MOON), read.planets().stream().map(PlanetDignity::planet).toList(), "the Chaldean order");
                same(5, read.scores().house(), "Lilly's house score");
            }
        });

        tests.put("every reading a chart was asked for reads back", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().profile("nepali-default")
                    .ephemeris(Ephemeris.BUILTIN).build())) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
                ChartOptions everything = ChartOptions.builder().state(true).aspects(true).points(true).houses(true)
                        .ashtakavarga(true).vimshopaka(true).vaiseshikamsa(true).dashaPhala(true).jaimini(true)
                        .avakahada(true).outerPlanets(true).shadbala(true).bhavaBala(true)
                        .vargas(Varga.D9).dashas(DashaSystem.VIMSHOTTARI).remedies(Map.of())
                        .fortitudes(Map.of()).lots(Map.of()).westernAspects(Map.of())
                        .westernHouses(Map.of()).antiscia(Map.of()).midpoints(Map.of()).parallels(Map.of())
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
                    check(chart.prashna().isEmpty() && chart.matching().isEmpty() && chart.synastry().isEmpty(),
                            "what was not asked is empty");
                    check(chart.hits().isEmpty() && chart.gochar().isEmpty() && chart.drawings().isEmpty(),
                            "and every list not asked for is empty");
                    same(chart.states(), chart.states(), "read twice, the same");
                    check(chart.timing() != null && chart.day() != null, "the timing and the day");
                }
                same(Graha.SUN, batch.get(0).grahas().get(0).graha(), "the Sun first");
            }
        });

        tests.put("a chart founded under KP's ayanamsha carries its KP reading", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().profile("kp-default")
                    .ephemeris(Ephemeris.BUILTIN).build())) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
                Chart chart = sky.chart().found(2_447_995.489_583_333_5, kathmandu, 20_700,
                        ChartOptions.builder().kp(Map.of()).build());
                check(chart.kp().isPresent(), "the KP reading");
            }
        });

        tests.put("the areas beside the chart answer", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().profile("nepali-default")
                    .ephemeris(Ephemeris.BUILTIN).build())) {
                Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
                NumerologyProfile profile = sky.numerology().profile("Ram Bahadur", java.time.LocalDate.of(1990, 4, 14));
                check(profile != null, "a numerology profile");
                check(sky.matching().naam("सीता", "राम") != null, "a naam milan");
                CalendarDate first = sky.chart().found(2_460_000.0, kathmandu, 20_700, ChartOptions.builder().build())
                        .day().date();
                Almanac week = sky.almanac().of(first,
                        sky.calendar().dateOf(first.calendar(), sky.calendar().fixedOf(first) + 6), kathmandu, 20_700);
                same(7, week.size(), "a day per date");
                check(!week.get(0).tithi().isEmpty(), "the tithis");
                RashifalAnswer read = sky.chart().rashifal(RashifalRequest.of(first, kathmandu, 20_700));
                check(read.period() != null, "a rashifal period");
                same(Status.UNSUPPORTED, refusal(() -> sky.ephemeris().names()).status(),
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
                same("teimeris", ((Map<?, ?>) sky.ephemeris().manifest()).get("engine"), "the engine's manifest");
                same("Sun", ((Map<?, ?>) sky.ephemeris().call("tm_body_name", Map.of("body", 0))).get("buf"),
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
                same(0L, left.count(), "no file left behind");
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
                same(1L, left.count(), "and no part files");
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
