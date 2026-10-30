package com.teispace.teistro;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;

import com.teispace.teistro.messages.Messages;
import com.teispace.teistro.request.RequestJson;

/**
 * The binding's own pieces, each held on its own: the width checks and
 * array copies every call crosses through, the JSON reader and writer, the
 * check a found library must pass, every tagged request union's dispatch,
 * and the message enums against the library's keys.
 */
final class UnitsTest {
    private UnitsTest() {}

    private static void check(boolean holds, String what) {
        if (!holds) {
            throw new AssertionError(what);
        }
    }

    private static <T> void same(T expected, T found, String what) {
        if (!java.util.Objects.equals(expected, found)) {
            throw new AssertionError(what + ": expected " + expected + ", found " + found);
        }
    }

    /** The message a call is refused with as an {@link IllegalArgumentException}. */
    private static String refused(Runnable call) {
        try {
            call.run();
        } catch (IllegalArgumentException e) {
            return e.getMessage();
        }
        throw new AssertionError("the call was not refused");
    }

    /** Reads {@code text}, which must be refused as not JSON. */
    private static void notJson(String text) {
        String said = refused(() -> Json.read(text));
        check(said.startsWith("not JSON at character "), text + ": " + said);
    }

    /** {@code SNAKE_CASE} as {@code snakeCase}. */
    private static String camel(String constant) {
        StringBuilder out = new StringBuilder();
        boolean upper = false;
        for (char c : constant.toLowerCase(java.util.Locale.ROOT).toCharArray()) {
            if (c == '_') {
                upper = true;
            } else {
                out.append(upper ? Character.toUpperCase(c) : c);
                upper = false;
            }
        }
        return out.toString();
    }

    /** The one public string constant of a union's variant: its tag, by the field it is written under. */
    private static Field tagOf(Class<?> variant) {
        Field tag = null;
        for (Field field : variant.getDeclaredFields()) {
            int modifiers = field.getModifiers();
            if (field.getType() == String.class && Modifier.isPublic(modifiers) && Modifier.isStatic(modifiers)
                    && Modifier.isFinal(modifiers)) {
                if (tag != null) {
                    return null;
                }
                tag = field;
            }
        }
        return tag;
    }

    /**
     * Adds the tests to {@code tests}.
     *
     * @param tests the run's tests, by name
     * @param teistro the library
     */
    static void add(Map<String, BindingTest.Test> tests, Teistro teistro) {
        // No build of the library can be made to panic on request, so the
        // real library's own records are classified at the layer that
        // classifies them, given the INTERNAL status a panic answers with.
        tests.put("an INTERNAL refusal tells a host's hook, and no other does", () -> {
            List<InternalReport> heard = new ArrayList<>();
            Runnable broken = Teistro.onInternalError(report -> {
                throw new IllegalStateException("a broken reporter");
            });
            Runnable stop = Teistro.onInternalError(heard::add);
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment error = Boundary.errorRecord(arena);
                same(Status.INVALID_ARG, Boundary.refusal(Status.INVALID_ARG.id(), error).status(), "a refusal");
                same(Status.INVALID_ARG, Calls.refused(teistro.lib, Status.INVALID_ARG.id()).status(), "no record");
                same(List.of(), heard, "no other status is told");
                same(Status.INTERNAL, Boundary.refusal(Teistro.INTERNAL_CODE, error).status(), "the record's");
                same(Status.INTERNAL, Calls.refused(teistro.lib, -10).status(), "a status alone");
            }
            InternalReport told = new InternalReport("INTERNAL", -10, "boundary", teistro.sdkVersion());
            same(List.of(told, told), heard, "both heard, a thrower dropped");
            stop.run();
            stop.run();
            broken.run();
            Calls.refused(teistro.lib, -10);
            same(2, heard.size(), "stopped");
            check(Arrays.stream(InternalReport.class.getRecordComponents())
                    .noneMatch(part -> part.getName().contains("message")), "a report has no message");
        });

        tests.put("a whole number is held to the width it crosses as, naming its field", () -> {
            same((byte) -1, Values.u8("f", 255), "255 is a u8's top");
            same("`f` is 0 to 255, and 256 is not", refused(() -> Values.u8("f", 256)), "past a u8");
            check(refused(() -> Values.u8("f", -1)).contains("-1 is not"), "below a u8");
            same((short) -1, Values.u16("f", 65_535), "a u16's top");
            check(refused(() -> Values.u16("f", 65_536)).contains("0 to 65535"), "past a u16");
            check(refused(() -> Values.u16("f", -1)).contains("0 to 65535"), "below a u16");
            same(-1, Values.u32("f", 0xFFFF_FFFFL), "a u32's top");
            check(refused(() -> Values.u32("f", 1L << 32)).contains("0 to 4294967295"), "past a u32");
            check(refused(() -> Values.u32("f", -1)).contains("0 to 4294967295"), "below a u32");
            same((byte) -128, Values.i8("f", -128), "an i8's floor");
            same((byte) 127, Values.i8("f", 127), "an i8's top");
            check(refused(() -> Values.i8("f", 128)).contains("-128 to 127"), "past an i8");
            check(refused(() -> Values.i8("f", -129)).contains("-128 to 127"), "below an i8");
            same((short) -32_768, Values.i16("f", -32_768), "an i16's floor");
            check(refused(() -> Values.i16("f", 32_768)).contains("-32768 to 32767"), "past an i16");
            check(refused(() -> Values.i16("f", -32_769)).contains("-32768 to 32767"), "below an i16");
        });

        tests.put("a member crosses as its id, and a set of them as bits", () -> {
            same(Graha.MARS.id(), Values.id("f", Graha.MARS), "an id");
            check(refused(() -> Values.id("f", Deity.UNKNOWN)).contains("cannot carry UNKNOWN"),
                    "a member from a newer build has no id to send");
            long bits = Values.bits("f", Set.of(Graha.SUN, Graha.MARS));
            same((1L << Graha.SUN.id()) | (1L << Graha.MARS.id()), bits, "one bit a member");
            same(Set.of(Graha.SUN, Graha.MARS), Values.members(bits, Graha::of), "and read back");
            same(Set.of(), Values.members(0, Graha::of), "no bits, no members");
            check(refused(() -> Values.bits("f", Set.of(Deity.SHUKRA))).contains("ids below 32"),
                    "an id past the bits");
        });

        tests.put("arrays cross into native memory and back, a null or empty one empty", () -> {
            try (Arena arena = Arena.ofConfined()) {
                long[] wide = {0, 0xFFFF_FFFFL, 7};
                check(Arrays.equals(wide, Values.u32s(Values.u32s(arena, "f", wide), wide.length)), "u32s");
                check(refused(() -> Values.u32s(arena, "f", new long[] {-1})).contains("`f`"), "a u32 below zero");
                int[] halves = {0, 65_535, 9};
                check(Arrays.equals(halves, Values.u16s(Values.u16s(arena, "f", halves), halves.length)), "u16s");
                List<Graha> grahas = List.of(Graha.SATURN, Graha.SUN);
                same(grahas, Values.u16Members(Values.u16Members(arena, "f", grahas), grahas.size(), Graha::of),
                        "members");
                double[] doubles = {1.5, -2.25};
                check(Arrays.equals(doubles, Values.doubles(Values.doubles(arena, doubles), 2)), "doubles");
                int[] ints = {-3, 4};
                check(Arrays.equals(ints, Values.i32s(Values.i32s(arena, ints), 2)), "i32s");
                same(0, Values.doubles(MemorySegment.NULL, 3).length, "a null pointer reads empty");
                same(0, Values.i32s(Values.i32s(arena, ints), 0).length, "a zero count reads empty");
                same(0, Values.u32s(MemorySegment.NULL, 2).length, "no u32s");
                same(0, Values.u16s(MemorySegment.NULL, 2).length, "no u16s");
                same(MemorySegment.NULL, Values.cString(arena, null), "no text is a null pointer");
                same(null, Values.text(MemorySegment.NULL, true), "a nullable null");
                same("", Values.text(MemorySegment.NULL, false), "a required null reads empty");
                same("नमस्ते", Values.text(Values.cString(arena, "नमस्ते"), false), "UTF-8 both ways");
            }
        });

        tests.put("JSON is written escaped and refused where it cannot say a value", () -> {
            String awkward = "\"\\\n\r\t\u0001/é";
            same(awkward, Json.read(Json.write(awkward)), "every escape both ways");
            same("\"\\\"\\\\\\n\\r\\t\\u0001/é\"", Json.write(awkward), "as JSON writes them");
            same("[1,\"graha.SUN\",\"DIRECT\",null,true]", Json.write(new Object[] {1, Graha.SUN, Accident.DIRECT,
                    null, true}), "a catalogue member whole, another bare");
            Map<String, Object> nested = new LinkedHashMap<>();
            nested.put("a", List.of(Double.NaN));
            same("`$.a[0]` is NaN, which JSON cannot say", refused(() -> Json.write(nested)), "NaN, by its path");
            check(refused(() -> Json.write(Float.POSITIVE_INFINITY)).contains("Infinity"), "an infinite float");
            check(refused(() -> Json.write(Map.of(1, 2))).contains("not a string"), "a key that is not text");
            check(refused(() -> Json.write(new StringBuilder())).contains("java.lang.StringBuilder"),
                    "a type JSON has no word for");
        });

        tests.put("JSON is read whole, and refused at the character it breaks", () -> {
            same("\b\f/\u00e9", Json.read("\"\\b\\f\\/\\u00e9\""), "escapes");
            same(1000.0, Json.read("1e3"), "an exponent is a double");
            same(-0.5, Json.read("-5E-1"), "a signed exponent");
            same(0L, Json.read("-0"), "a whole number is a long");
            same(1.2345678901234568E29, Json.read("123456789012345678901234567890"), "too wide for a long");
            same(Map.of(), Json.read(" { } "), "an empty object");
            same(List.of(), Json.read("[ ]"), "an empty list");
            same(null, Json.read("null"), "null");
            same(Map.of("a", List.of(false, Map.of("b", "c"))), Json.read("{\"a\":[false,{\"b\":\"c\"}]}"), "nested");
            for (String bad : List.of("", "tru", "{1:2}", "{\"a\" 1}", "{\"a\":1", "[1 2]", "\"\\u12\"",
                    "\"\\uZZZZ\"", "\"\\x\"", "\"a\u0001\"", "\"abc", "\"\\", "1.", "1e", "1e+", "-", "nul", "[")) {
                notJson(bad);
            }
            check(refused(() -> Json.object("[1]")).contains("not an object"), "an object where one belongs");
            same(Map.of("a", 1L), Json.object("{\"a\":1}"), "an object");
        });

        tests.put("a found library is refused unless it is this build's, unsanitised and optimised", () -> {
            Map<String, Object> good = new LinkedHashMap<>();
            good.put("abi", (long) com.teispace.teistro.ffi.Native.GENERATED_ABI_VERSION);
            good.put("sdk", com.teispace.teistro.ffi.Native.GENERATED_SDK_VERSION);
            good.put("optimised", true);
            same(null, Teistro.refusal(good, false), "this build");
            Map<String, Object> other = new LinkedHashMap<>(good);
            other.put("abi", 999L);
            check(Teistro.refusal(other, true).contains("ABI 999"), "another ABI");
            other.put("abi", "1");
            check(Teistro.refusal(other, true).contains("ABI 1,"), "an ABI that is not a number");
            other = new LinkedHashMap<>(good);
            other.put("sdk", "9.9.9");
            check(Teistro.refusal(other, true).contains("SDK 9.9.9"), "another SDK");
            other = new LinkedHashMap<>(good);
            other.put("sanitizer", "address");
            check(Teistro.refusal(other, true).contains("an address sanitizer build"), "an address sanitizer");
            other.put("sanitizer", "thread");
            check(Teistro.refusal(other, true).contains("a thread sanitizer build"), "a thread sanitizer");
            other.put("sanitizer", "");
            same(null, Teistro.refusal(other, true), "no sanitizer named");
            other = new LinkedHashMap<>(good);
            other.put("optimised", false);
            check(Teistro.refusal(other, false).contains("unoptimised"), "a debug build found");
            same(null, Teistro.refusal(other, true), "a debug build named outright");

            String before = System.getProperty(Teistro.LIBRARY_PROPERTY);
            String missing = java.nio.file.Paths.get("no", "such", "libteistro").toAbsolutePath().toString();
            System.setProperty(Teistro.LIBRARY_PROPERTY, missing);
            try {
                Teistro.open();
                throw new AssertionError("a library that is not a file was opened");
            } catch (TeistroException e) {
                same(Status.UNSUPPORTED, e.status(), "refused as unsupported");
                check(e.getMessage().contains(missing), "naming the path: " + e.getMessage());
            } finally {
                if (before == null) {
                    System.clearProperty(Teistro.LIBRARY_PROPERTY);
                } else {
                    System.setProperty(Teistro.LIBRARY_PROPERTY, before);
                }
            }
        });

        tests.put("a library none names is found in the workspace build above the working directory", () -> {
            String before = System.getProperty(Teistro.LIBRARY_PROPERTY);
            // Empty is unset, and outranks the variable the run sets.
            System.setProperty(Teistro.LIBRARY_PROPERTY, "");
            try {
                Teistro found = Teistro.open();
                check(found.path().endsWith(java.nio.file.Path.of("target", "release", found.path().getFileName()
                        .toString())), "the release build: " + found.path());
                same(teistro.sdkVersion(), found.sdkVersion(), "this build's SDK");
                same(teistro.abiVersion(), found.abiVersion(), "this build's ABI");
            } finally {
                if (before == null) {
                    System.clearProperty(Teistro.LIBRARY_PROPERTY);
                } else {
                    System.setProperty(Teistro.LIBRARY_PROPERTY, before);
                }
            }
        });

        // The test provider ships a two-function manifest, so the engine's own
        // operations run with no real engine present and still take the whole route.
        tests.put("the engine names its own operations and is called by the names it gives them", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().profile("nepali-default").testProvider(true)
                    .build())) {
                Engine engine = sky.engine();
                check(engine.names().contains("tp_echo"), "names: " + engine.names());
                check(engine.offers("tp_sum") && !engine.offers("tm_no_such_thing"), "offers what it names");
                same("test-provider", ((Map<?, ?>) engine.manifest()).get("engine"), "the manifest's engine");
                check(engine.manifest() == engine.manifest(), "the manifest is read once");
                same(engine.manifest(), Json.read(engine.manifestJson()), "the manifest as written");
                check(engine.toString().startsWith("Engine(test-provider ")
                        && engine.toString().endsWith(", " + engine.names().size() + " operations)"),
                        "described: " + engine);

                Map<?, ?> echoed = (Map<?, ?>) engine.call("tp_echo", Map.of("value", 6.0));
                same(6.0, ((Number) echoed.get("value")).doubleValue(), "echoed");
                Map<?, ?> summed = (Map<?, ?>) engine.call("tp_sum", Map.of("values", List.of(1.0, 2.0, 3.5)));
                same(6.5, ((Number) summed.get("total")).doubleValue(), "summed");
                same(Map.of("value", 1.5), Json.read(engine.callJson("tp_echo", "{\"value\":1.5}")),
                        "called with JSON, answered in JSON");

                Map<?, ?> signature = engine.signature("tp_sum").orElseThrow();
                List<Object> roles = new ArrayList<>();
                for (Object param : (List<?>) signature.get("params")) {
                    roles.add(((Map<?, ?>) param).get("role"));
                }
                same(List.of("array_in", "array_len", "scalar_out"), roles, "each parameter's role");
                check(engine.signature("tm_no_such_thing").isEmpty(), "no signature for what it lacks");
                try {
                    engine.call("tm_eclipse_when", Map.of());
                    throw new AssertionError("an operation the engine lacks was called");
                } catch (TeistroException e) {
                    check(e.getMessage().contains("tm_eclipse_when"), "naming it: " + e.getMessage());
                }
            }
        });

        tests.put("a civil time resolves in a zone, a fixed offset and local mean time, and back", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().profile("nepali-default").build())) {
                CivilDateTime civil = new CivilDateTime(Civil.date(Calendar.GREGORIAN, 1986, 1, 1),
                        new CivilTime(0, 20, 0, true, 0));
                ZoneSpec kathmandu = new ZoneSpec(ZoneKind.IANA, 0, new Longitude(0), "Asia/Kathmandu");
                ZoneResolution resolved = sky.time().resolve(civil, kathmandu);
                check(Math.abs(resolved.instantJdUtc() - 2_446_431.2743056) < 1e-6, "the instant");
                same(20_700, resolved.offsetSeconds(), "+05:45, the offset that began that midnight");
                same(ZoneEra.CURRENT, resolved.era(), "today's rules");
                same(ZoneSource.IANA, resolved.source(), "from the database");
                check(resolved.timeKnown() && resolved.warnings().isEmpty(), "nothing guessed");
                check(resolved.tzdbVersion().matches("20\\d\\d[a-z]"), "the database's version");

                TimeCivilResult back = sky.time().civilOf(resolved.instantJdUtc(), kathmandu, Calendar.GREGORIAN);
                same(List.of(1986, 1, 1), List.of(back.civil().date().year(), back.civil().date().month(),
                        back.civil().date().day()), "the day back");
                same(List.of(0, 20, true), List.of(back.civil().time().hour(), back.civil().time().minute(),
                        back.civil().time().hasTime()), "the time back");
                same(resolved.offsetSeconds(), back.resolution().offsetSeconds(), "the same offset");

                // A fixed offset is the caller's own; local mean time is the longitude's, four minutes a degree.
                ZoneResolution manual = sky.time().resolve(civil, kathmandu.withKind(ZoneKind.FIXED)
                        .withOffsetSeconds(19_800).withZone(null));
                same(ZoneSource.MANUAL, manual.source(), "a fixed offset");
                same(19_800, manual.offsetSeconds(), "as given");
                check(Math.abs(manual.instantJdUtc() - resolved.instantJdUtc() - 900 / 86_400.0) < 1e-9,
                        "fifteen minutes later than +05:45");
                ZoneResolution mean = sky.time().resolve(civil, new ZoneSpec(ZoneKind.LOCAL_MEAN, 0,
                        new Longitude(85.324), null));
                same(ZoneSource.LOCAL_MEAN, mean.source(), "local mean time");
                check(Math.abs(mean.offsetSeconds() - 85.324 * 240) <= 1, "the longitude's: " + mean.offsetSeconds());

                // A time the spring-forward gap skipped is refused under the default policy, naming it.
                try {
                    sky.time().resolve(new CivilDateTime(Civil.date(Calendar.GREGORIAN, 2024, 3, 10),
                            new CivilTime(2, 30, 0, true, 0)), new ZoneSpec(ZoneKind.IANA, 0, new Longitude(0),
                            "America/New_York"));
                    throw new AssertionError("a time in the gap was resolved");
                } catch (TeistroException e) {
                    same(List.of(Status.INVALID_ARG, "DST_GAP", "time"), List.of(e.status(), e.detail(), e.field()),
                            "the gap refused");
                }

                try {
                    sky.time().resolve(civil, kathmandu.withZone("Asia/Kathmandou"));
                    throw new AssertionError("a zone the database lacks was resolved");
                } catch (TeistroException e) {
                    same(List.of(Status.INVALID_ARG, "zone"), List.of(e.status(), e.field()), "an unknown zone");
                    check(e.hint().contains("Asia/Kathmandu"), "the nearest name as the hint: " + e.hint());
                }
            }
        });

        tests.put("every tagged request union reads each variant by its tag, and refuses a stranger", () -> {
            int unions = 0;
            int variants = 0;
            for (Class<?> union : SweepTest.moduleClasses()) {
                if (!union.isSealed() || !union.isInterface() || !RequestJson.class.isAssignableFrom(union)) {
                    continue;
                }
                Method of = union.getDeclaredMethod("of", Object.class);
                of.setAccessible(true);
                String key = null;
                List<Class<?>> tagged = new ArrayList<>();
                for (Class<?> variant : union.getPermittedSubclasses()) {
                    Field tag = tagOf(variant);
                    if (tag == null) {
                        tagged.clear();
                        break;
                    }
                    key = camel(tag.getName());
                    tagged.add(variant);
                }
                if (tagged.isEmpty()) {
                    continue;
                }
                unions += 1;
                for (Class<?> variant : tagged) {
                    variants += 1;
                    String value = (String) tagOf(variant).get(null);
                    try {
                        Object read = of.invoke(null, Map.of(key, value));
                        check(variant.isInstance(read), union.getSimpleName() + " read " + value + " as " + read);
                        same(value, ((Map<?, ?>) ((RequestJson) read).toJson()).get(key), value + " written back");
                    } catch (InvocationTargetException e) {
                        String said = String.valueOf(e.getCause().getMessage());
                        check(said.contains(variant.getSimpleName() + "."),
                                union.getSimpleName() + "'s " + value + " is read by " + variant.getSimpleName()
                                        + ", which names what it lacks: " + said);
                    }
                }
                try {
                    of.invoke(null, Map.of(key, "NOT_A_VARIANT"));
                    throw new AssertionError(union.getSimpleName() + " read a stranger");
                } catch (InvocationTargetException e) {
                    check(e.getCause() instanceof IllegalArgumentException
                            && e.getCause().getMessage().contains(union.getSimpleName()),
                            union.getSimpleName() + " refuses a stranger by name: " + e.getCause());
                }
            }
            check(unions > 20, "the tagged unions were found: " + unions);
            check(variants > 150, "their variants: " + variants);
        });

        tests.put("every message enum's member is a key the library knows", () -> {
            int members = 0;
            try (Context sky = teistro.context(ContextOptions.builder().profile("nepali-default").testProvider(true)
                    .build())) {
                for (Class<?> type : Messages.class.getDeclaredClasses()) {
                    if (!type.isEnum() || !type.getSimpleName().endsWith("Key")) {
                        continue;
                    }
                    Method value = type.getMethod("value");
                    for (Object member : type.getEnumConstants()) {
                        String key = (String) value.invoke(member);
                        same(key, sky.keys().name(sky.keys().id(key)), type.getSimpleName() + "." + member);
                        members += 1;
                    }
                }
            }
            check(members > 500, "the members were found: " + members);
            same(List.of("m", "f", "n"), Arrays.stream(Messages.Gender.values()).map(Messages.Gender::value).toList(),
                    "a gender is the engine's word");
        });
    }
}
