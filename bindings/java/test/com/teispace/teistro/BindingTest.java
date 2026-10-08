package com.teispace.teistro;

import java.lang.foreign.StructLayout;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;

import com.teispace.teistro.ffi.Native;

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
                int sun = sky.keyId("graha.SUN");
                same(Kind.GRAHA.id(), sun >>> 16, "the kind is the high half");
                same(Graha.SUN.id(), sun & 0xFFFF, "the member is the low half");
                same("graha.SUN", sky.keyName(sun), "the name");
                same(Graha.SUN, Graha.of(Graha.SUN.id()), "of");
                same(Graha.UNKNOWN, Graha.of(0xFFFE), "an unknown id from a newer library");
                same(Graha.SUN, Graha.byKey("graha.SUN").orElseThrow(), "byKey, full");
                same("graha.SUN", Graha.SUN.fullKey(), "fullKey");
            }
        });

        tests.put("a refusal carries its status, its field and its hint", () -> {
            try (Context sky = context(teistro)) {
                TeistroException typo = refusal(() -> sky.keyId("graha.SUNN"));
                same(Status.UNSUPPORTED, typo.status(), "status");
                same("UNKNOWN_KEY", typo.detail(), "detail");
                check(typo.hint().contains("SUN"), "the hint names the near key: " + typo.hint());
                // The second refusal carries the second record, not the first.
                TeistroException other = refusal(() -> sky.keyId("rashi.ARIESS"));
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
