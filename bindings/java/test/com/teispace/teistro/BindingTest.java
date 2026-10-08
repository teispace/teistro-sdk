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
                Positions grid = sky.positions(new double[] {2_451_545.0, 2_451_546.0},
                        List.of(Body.SUN, Body.MOON, Body.MARS));
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

        tests.put("a closed enum refuses an id that is no member", () -> {
            try {
                Status.of(12345);
                throw new AssertionError("an unknown status read");
            } catch (IllegalArgumentException expected) {
                check(expected.getMessage().contains("12345"), "names the id");
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
