package com.teispace.teistro;

import java.lang.reflect.Constructor;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.lang.reflect.RecordComponent;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.TreeSet;

import com.teispace.teistro.blob.IntlRender;
import com.teispace.teistro.messages.Messages;
import com.teispace.teistro.request.Cancellation;
import com.teispace.teistro.request.Condition;
import com.teispace.teistro.request.RequestJson;
import com.teispace.teistro.request.Requests;
import com.teispace.teistro.request.RuleDefinition;

/**
 * The tests that walk a whole generated surface rather than one call: every
 * typed message accessor against the locale engine, every catalogue member
 * against the library's key table, every request record's builder and
 * short constructor against the record its example reads into, and the
 * shipped rules through the request types. A generator that writes one
 * accessor, member or setter wrong fails here, whichever it is.
 */
final class SweepTest {
    private SweepTest() {}

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

    /** Calls a method reflectively, rethrowing what the method threw. */
    private static Object call(Method method, Object on, Object... args) throws Exception {
        try {
            return method.invoke(on, args);
        } catch (InvocationTargetException e) {
            throw unwrapped(e);
        }
    }

    /** What a reflective call's target threw, an error rethrown as itself. */
    private static Exception unwrapped(InvocationTargetException e) {
        if (e.getCause() instanceof Error error) {
            throw error;
        }
        return e.getCause() instanceof Exception cause ? cause : e;
    }

    /** A renderer that answers each key with itself and keeps what it was asked. */
    private static final class Recording implements Messages.Renderer {
        String key;
        Map<String, Object> params;

        @Override
        public String render(String key, Map<String, Object> params) {
            this.key = key;
            this.params = params;
            return key;
        }

        @Override
        public Messages.EntityForms entity(String key) {
            this.key = key;
            this.params = null;
            return Messages.EntityForms.of(Map.of("name", key));
        }
    }

    /** A value of a parameter's type for an accessor to be called with. */
    private static Object argument(Class<?> type) {
        if (type == long.class) {
            return 1L;
        }
        if (type == double.class) {
            return 2_460_000.25;
        }
        if (type == String.class) {
            return "x";
        }
        if (type == List.class) {
            return List.of("x");
        }
        if (type.isEnum()) {
            return type.getEnumConstants()[0];
        }
        throw new AssertionError("no argument for an accessor's " + type);
    }

    /** A parameter the engine reads as something other than free text. */
    private static Object meaningful(String name, Object value) {
        return "zone".equals(name) ? "Asia/Kathmandu" : value;
    }

    /**
     * Whether the engine's {@code warning}, rendering {@code key} in any
     * locale with no pack loaded, is one the sweep expects. Text
     * parameters get the stand-in {@code x}, which a message reading the
     * parameter as an entity a pack brings (a rule, a house's reading)
     * refuses by name. A phala message reads a form of its entity
     * ({@code phala}, {@code dashaActivation}) that only an interpretation
     * pack brings, and the {@code state} kind has no entry in either shipped
     * locale at all.
     */
    private static boolean expected(String key, String warning) {
        return warning.contains("`x`")
                || key.startsWith("sdk.phala.") && (warning.startsWith("missing entity `state.")
                        || warning.matches("entity `[a-z_]+\\.[A-Z0-9_]+` has no `\\w+` form"));
    }

    /** What the sweep of the accessors met. */
    private static final class Swept {
        final Set<String> messages = new TreeSet<>();
        final Set<String> entities = new TreeSet<>();
        final Map<String, List<String>> unexpected = new java.util.TreeMap<>();
    }

    /**
     * Walks two accessor trees side by side, one over the recording renderer
     * and one over the context's: each message accessor must reach a key the
     * locale carries, with no warning, and answer what rendering that key
     * answers; each entity accessor the forms the locale gives that key.
     */
    private static void walk(Object recorded, Object real, Recording recording, IntlArea intl, Swept swept)
            throws Exception {
        for (Method method : recorded.getClass().getDeclaredMethods()) {
            int modifiers = method.getModifiers();
            if (!Modifier.isPublic(modifiers) || Modifier.isStatic(modifiers) || method.isSynthetic()) {
                continue;
            }
            Class<?> returned = method.getReturnType();
            String name = recorded.getClass().getSimpleName() + "." + method.getName();
            if (returned == String.class) {
                Object[] args = new Object[method.getParameterCount()];
                for (int i = 0; i < args.length; i += 1) {
                    args[i] = argument(method.getParameterTypes()[i]);
                }
                Object answered = call(method, recorded, args);
                String key = recording.key;
                same(key, answered, name + " answers what it rendered");
                check(swept.messages.add(key), key + " is reached by one accessor, not two");
                check(intl.has(key), name + " names " + key + ", which the locale carries");
                Map<String, Object> params = new LinkedHashMap<>(recording.params);
                same(params.size(), method.getParameterCount(), name + " sends every parameter");
                String text = (String) call(method, real, args);
                same(intl.render(key, params).text(), text, name + " renders through the context");
                check(!text.isEmpty() && !text.equals(key), name + " says something: " + text);
                params.replaceAll(SweepTest::meaningful);
                IntlRender rendered = intl.render(key, params);
                String locale = intl.locale();
                List<String> unexpected = ((List<?>) Json.read(rendered.warnings())).stream().map(String::valueOf)
                        .filter(warning -> !expected(key, warning)).toList();
                if (!unexpected.isEmpty()) {
                    swept.unexpected.put(key, unexpected);
                }
            } else if (returned == Messages.EntityForms.class) {
                call(method, recorded);
                String key = recording.key;
                check(swept.entities.add(key), key + " is reached by one accessor, not two");
                Messages.EntityForms forms = (Messages.EntityForms) call(method, real);
                same(intl.entity(key), forms, name + " reads the locale's forms");
                check(!forms.name().isEmpty(), key + " has a name");
            } else if (method.getParameterCount() == 0 && returned.getEnclosingClass() != null
                    && !returned.isEnum() && !returned.isRecord()) {
                walk(call(method, recorded), call(method, real), recording, intl, swept);
            }
        }
    }

    /** Every class of the binding's module, by name, loaded without being initialised. */
    static List<Class<?>> moduleClasses() throws Exception {
        Module module = Teistro.class.getModule();
        var reference = module.getLayer().configuration().findModule(module.getName()).orElseThrow().reference();
        List<Class<?>> found = new ArrayList<>();
        try (var reader = reference.open(); var names = reader.list()) {
            for (String name : names.filter(one -> one.endsWith(".class") && !one.equals("module-info.class"))
                    .toList()) {
                String binary = name.substring(0, name.length() - ".class".length()).replace('/', '.');
                found.add(Class.forName(binary, false, Teistro.class.getClassLoader()));
            }
        }
        return found;
    }

    /**
     * Rebuilds {@code record} through its builder, from its own fields, and
     * holds the builder, its required fields and the short constructor to
     * the record; then does the same for every record inside it.
     */
    private static void rebuild(Object value, Set<Class<?>> rebuilt) throws Exception {
        if (value instanceof List<?> items) {
            for (Object item : items) {
                rebuild(item, rebuilt);
            }
            return;
        }
        if (value instanceof Map<?, ?> entries) {
            for (Object item : entries.values()) {
                rebuild(item, rebuilt);
            }
            return;
        }
        if (value == null || !value.getClass().isRecord() || !(value instanceof RequestJson)) {
            return;
        }
        Class<?> type = value.getClass();
        RecordComponent[] fields = type.getRecordComponents();
        Object[] values = new Object[fields.length];
        for (int i = 0; i < fields.length; i += 1) {
            values[i] = call(fields[i].getAccessor(), value);
            rebuild(values[i], rebuilt);
        }
        Method builder;
        try {
            builder = type.getMethod("builder");
        } catch (NoSuchMethodException none) {
            return;
        }
        rebuilt.add(type);
        same(value, build(type, builder, fields, values, -1), type.getSimpleName() + " built from its own fields");

        // A field left unset is either refused by name or left out.
        Object[] required = new Object[fields.length];
        List<Object> shortArgs = new ArrayList<>();
        for (int i = 0; i < fields.length; i += 1) {
            if (values[i] == null) {
                continue;
            }
            String name = type.getSimpleName() + "." + fields[i].getName();
            try {
                Object without = build(type, builder, fields, values, i);
                check(!fields[i].getType().isPrimitive(), name + " is a number every request needs, yet unset built");
                same(null, call(fields[i].getAccessor(), without), name + " left unset is left out");
            } catch (NullPointerException refused) {
                // A field named for a Java keyword is refused by its wire name.
                String wire = name.endsWith("Value") ? name.substring(0, name.length() - "Value".length()) : name;
                String said = String.valueOf(refused.getMessage());
                check(said.contains("`" + name + "` is required") || said.contains("`" + wire + "` is required"),
                        name + " refused by name: " + said);
                required[i] = values[i];
                shortArgs.add(values[i]);
            }
        }
        if (shortArgs.size() == fields.length) {
            return;
        }
        for (Constructor<?> constructor : type.getConstructors()) {
            if (constructor.getParameterCount() != shortArgs.size()) {
                continue;
            }
            try {
                same(build(type, builder, fields, required, -1), constructor.newInstance(shortArgs.toArray()),
                        type.getSimpleName() + "'s required fields alone");
            } catch (InvocationTargetException e) {
                throw unwrapped(e);
            }
        }
    }

    /** A record built through its builder from {@code values}, the field at {@code skip} left unset. */
    private static Object build(Class<?> type, Method builder, RecordComponent[] fields, Object[] values, int skip)
            throws Exception {
        Object building = call(builder, null);
        for (int i = 0; i < fields.length; i += 1) {
            if (i == skip || values[i] == null) {
                continue;
            }
            Method setter = setter(building.getClass(), fields[i].getName(), values[i]);
            same(building, call(setter, building, values[i]), type.getSimpleName() + "." + fields[i].getName()
                    + " answers its builder");
        }
        return call(building.getClass().getMethod("build"), building);
    }

    private static final Map<Class<?>, Class<?>> BOXES = Map.of(int.class, Integer.class, long.class, Long.class,
            double.class, Double.class, boolean.class, Boolean.class, float.class, Float.class, short.class,
            Short.class, byte.class, Byte.class);

    /** The builder's setter named for a field that takes its value. */
    private static Method setter(Class<?> builder, String name, Object value) {
        for (Method method : builder.getMethods()) {
            if (method.getName().equals(name) && method.getParameterCount() == 1) {
                Class<?> takes = method.getParameterTypes()[0];
                if (BOXES.getOrDefault(takes, takes).isInstance(value)) {
                    return method;
                }
            }
        }
        throw new AssertionError(builder.getName() + " has no setter " + name + " for " + value.getClass());
    }

    /**
     * Adds the sweeps to {@code tests}.
     *
     * @param tests the run's tests, by name
     * @param teistro the library
     */
    static void add(Map<String, BindingTest.Test> tests, Teistro teistro) {
        tests.put("every message accessor reaches a key the locale renders, without a warning", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().profile("nepali-default").testProvider(true)
                    .build())) {
                for (String locale : List.of("en-Latn", "ne-Deva-NP")) {
                    sky.intl().setLocale(locale);
                    Recording recording = new Recording();
                    Swept swept = new Swept();
                    try {
                        walk(new Messages(recording), sky.intl().messages(), recording, sky.intl(), swept);
                    } catch (Exception | AssertionError e) {
                        throw new AssertionError(locale + ": " + e, e);
                    }
                    same(Map.of(), swept.unexpected, locale + ": every key takes what its accessor sends");
                    check(swept.messages.size() > 140, locale + ": the messages were found: " + swept.messages.size());
                    check(swept.entities.size() > 500, locale + ": the entities were found: " + swept.entities.size());
                }
            }
        });

        tests.put("an entity's forms read from the boundary's JSON, a gender only when the locale gives one", () -> {
            Messages.EntityForms forms = Messages.EntityForms.of(Map.of("name", "Sun", "iast", "Sūrya", "gender",
                    "m", "short", "Su", "glyph", "☉", "extra", "x", "count", 3));
            same("Sun", forms.name(), "name");
            same("Su", forms.short_(), "short");
            same(Messages.Gender.M, forms.gender(), "gender");
            same("x", forms.forms().get("extra"), "an open form is kept");
            check(!forms.forms().containsKey("count"), "a value that is not text is no form");
            Messages.EntityForms bare = Messages.EntityForms.of(Map.of("gender", "q"));
            same("", bare.prose(), "a guaranteed form the locale lacks is empty");
            same(null, bare.gender(), "a gender the build does not know is none");
            same(null, bare.glyph(), "no glyph");
            try {
                Messages.EntityForms.of(List.of());
                throw new AssertionError("a list was read as forms");
            } catch (IllegalArgumentException expected) {
                check(expected.getMessage().contains("object"), expected.getMessage());
            }
        });

        tests.put("every catalogue member reads back by its id and key, and is the library's packed key", () -> {
            int enums = 0;
            int members = 0;
            try (Context sky = teistro.context(ContextOptions.builder().profile("nepali-default").testProvider(true)
                    .build())) {
                for (Class<?> type : moduleClasses()) {
                    if (!type.isEnum() || !Member.class.isAssignableFrom(type)) {
                        continue;
                    }
                    enums += 1;
                    boolean open = Catalogued.class.isAssignableFrom(type);
                    Method of = type.getMethod("of", int.class);
                    Method byKey = type.getMethod("byKey", String.class);
                    String name = type.getSimpleName();
                    java.util.Set<Integer> ids = new java.util.HashSet<>();
                    for (Object constant : type.getEnumConstants()) {
                        Member member = (Member) constant;
                        if (open && member.key().equals("UNKNOWN")) {
                            same(-1, member.id(), name + ".UNKNOWN's id");
                            same(Optional.empty(), call(byKey, null, "UNKNOWN"), name + ": UNKNOWN is no key");
                            continue;
                        }
                        members += 1;
                        check(ids.add(member.id()), name + "." + member.key() + "'s id is its own");
                        same(member, call(of, null, member.id()), name + "." + member.key() + " by its id");
                        same(Optional.of(member), call(byKey, null, member.key()), name + "." + member.key()
                                + " by its key");
                        if (member instanceof Catalogued keyed) {
                            String full = keyed.kind().key() + "." + member.key();
                            same(full, keyed.fullKey(), name + "." + member.key() + "'s full key");
                            same(Optional.of(member), call(byKey, null, full), full + " by its full key");
                            long packed = ((long) keyed.kind().id() << 16) | member.id();
                            same(packed, sky.keys().id(full), full + " is the library's packed key");
                            same(full, sky.keys().name(packed), full + " is the library's name of its key");
                        }
                    }
                    same(Optional.empty(), call(byKey, null, "NOT_A_MEMBER_OF_ANY_KIND"), name + ": a stranger");
                    int stranger = 0x7FFE;
                    if (open) {
                        same("UNKNOWN", ((Member) call(of, null, stranger)).key(), name + ": an id from a newer build");
                    } else {
                        try {
                            call(of, null, stranger);
                            throw new AssertionError(name + " took an id it does not have");
                        } catch (IllegalArgumentException expected) {
                            check(expected.getMessage().contains(name), expected.getMessage());
                        }
                    }
                }
            }
            check(enums > 150, "the catalogue's enums were found: " + enums);
            check(members > 1000, "their members: " + members);
        });

        tests.put("every request record's builder and short constructor build what its example reads into", () -> {
            List<?> examples = (List<?>) Json.read(Files.readString(Path.of("crates/sdk/tests/deep-requests.json")));
            Set<Class<?>> rebuilt = new java.util.HashSet<>();
            for (Object example : examples) {
                Map<?, ?> one = (Map<?, ?>) example;
                String record = (String) one.get("record");
                try {
                    rebuild(Requests.read(record, one.get("request")), rebuilt);
                } catch (Exception | AssertionError e) {
                    throw new AssertionError(record + ": " + e, e);
                }
            }
            check(rebuilt.size() > 100, "the records were rebuilt: " + rebuilt.size());
        });

        tests.put("every request record read from an empty object writes nothing, or names what it requires", () -> {
            int empty = 0;
            int refused = 0;
            for (Class<?> type : moduleClasses()) {
                if (!type.isRecord() || !RequestJson.class.isAssignableFrom(type)
                        || !type.getPackageName().equals("com.teispace.teistro.request")) {
                    continue;
                }
                Method of;
                try {
                    of = type.getMethod("of", Object.class);
                } catch (NoSuchMethodException none) {
                    // A part the layer writes and never reads (`Keyed`).
                    continue;
                }
                String name = type.getSimpleName();
                Object read;
                try {
                    read = of.invoke(null, Map.of());
                } catch (InvocationTargetException e) {
                    // A required field is named by its record; a value of another
                    // shape (a list, a number, a wrapped record) is refused by its reader.
                    String said = String.valueOf(e.getCause().getMessage());
                    check(e.getCause() instanceof NullPointerException
                            ? said.matches("`" + name + "\\.\\w+` is required")
                            : e.getCause() instanceof IllegalArgumentException && !said.isBlank(),
                            name + " refused as " + e.getCause());
                    refused += 1;
                    continue;
                }
                // Every field left out: nothing is written but a variant's tag, and
                // what is written reads back as the same record.
                Map<?, ?> written = (Map<?, ?>) ((RequestJson) read).toJson();
                check(written.size() <= 1, name + " wrote " + written);
                same(read, of.invoke(null, written), name + " read back");
                try {
                    Object builder = type.getMethod("builder").invoke(null);
                    same(read, builder.getClass().getMethod("build").invoke(builder), name + "'s empty builder");
                } catch (NoSuchMethodException none) {
                    // A record with no field has no builder.
                }
                empty += 1;
            }
            check(empty > 100, "records read empty: " + empty);
            check(refused > 50, "records refused empty: " + refused);
        });

        tests.put("every shipped rule reads into the request types and writes back as itself", () -> {
            int rules = 0;
            int cancellations = 0;
            Map<String, Integer> types = new HashMap<>();
            try (var files = Files.list(Path.of("crates/rules/rules"))) {
                for (Path file : files.filter(one -> one.toString().endsWith(".json")).sorted().toList()) {
                    Map<?, ?> pack = (Map<?, ?>) Json.read(Files.readString(file));
                    if (!(pack.get("rules") instanceof List<?> shipped)) {
                        continue;
                    }
                    for (Object raw : shipped) {
                        RuleDefinition rule = RuleDefinition.of(raw);
                        same(((Map<?, ?>) raw).get("key"), rule.key(), file.getFileName() + ": its key");
                        same(rule, RuleDefinition.of(Json.read(Json.write(rule))), rule.key() + " written and read");
                        same(null, disagreement(Json.read(Json.write(rule)), raw, "$"),
                                rule.key() + " is written as " + file.getFileName() + " has it");
                        if (rule.conditions() != null) {
                            for (Condition condition : rule.conditions()) {
                                count(condition, types);
                            }
                        }
                        if (rule.cancellations() != null) {
                            for (Cancellation cancellation : rule.cancellations()) {
                                count(cancellation.condition(), types);
                                cancellations += 1;
                            }
                        }
                        rules += 1;
                    }
                }
            }
            check(rules > 1000, "the rules were read: " + rules);
            check(cancellations > 50, "the cancellations were read, each with its condition: " + cancellations);
            check(types.size() > 40, "the kinds of condition they use: " + types.keySet());
        });
    }

    /**
     * Where {@code written} stops saying what {@code source} says, or null
     * where it says it all: every field the source gives, with the same
     * value, a member written whole where the source wrote it bare. A field
     * the source leaves out may be written with its default.
     */
    private static String disagreement(Object written, Object source, String at) {
        if (written instanceof Map<?, ?> w && source instanceof Map<?, ?> s) {
            for (Map.Entry<?, ?> entry : s.entrySet()) {
                if (!w.containsKey(entry.getKey())) {
                    return at + "." + entry.getKey() + " is not written";
                }
                String found = disagreement(w.get(entry.getKey()), entry.getValue(), at + "." + entry.getKey());
                if (found != null) {
                    return found;
                }
            }
            return null;
        }
        if (written instanceof List<?> w && source instanceof List<?> s) {
            if (w.size() != s.size()) {
                return at + " has " + w.size() + ", not " + s.size();
            }
            for (int i = 0; i < w.size(); i += 1) {
                String found = disagreement(w.get(i), s.get(i), at + "[" + i + "]");
                if (found != null) {
                    return found;
                }
            }
            return null;
        }
        if (written instanceof String w && source instanceof String s && (w.equals(s) || w.endsWith("." + s))) {
            return null;
        }
        if (written instanceof Number w && source instanceof Number s && w.doubleValue() == s.doubleValue()) {
            return null;
        }
        return java.util.Objects.equals(written, source) ? null : at + " is " + written + ", not " + source;
    }

    /** Counts each kind of condition in a tree of them, by its record. */
    private static void count(Object value, Map<String, Integer> types) throws Exception {
        if (value instanceof List<?> items) {
            for (Object item : items) {
                count(item, types);
            }
        } else if (value instanceof Condition condition) {
            types.merge(condition.getClass().getSimpleName(), 1, Integer::sum);
            for (RecordComponent field : condition.getClass().getRecordComponents()) {
                count(call(field.getAccessor(), condition), types);
            }
        }
    }
}
