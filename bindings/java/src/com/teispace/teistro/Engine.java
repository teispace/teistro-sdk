package com.teispace.teistro;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.Supplier;

/**
 * The engine's own operations, reached by the names it gives them.
 *
 * <p>The SDK names eight operations. An engine names far more, and what it
 * names beyond them is reached through here rather than around the SDK.
 * <b>Nothing in this class is a list of an engine's operations</b>: it asks
 * the engine what it offers and calls what the answer names, so a function
 * the engine gains after this package ships is callable without a new
 * release of it.
 *
 * <p>The engine's own spelling is the name to use, because that is what its
 * manifest says and what its documentation calls it:
 *
 * <pre>{@code
 * Engine engine = sky.ephemeris();
 * Object answer = engine.call("tp_echo", Map.of("value", 6.0));
 * List<String> offered = engine.names();
 * }</pre>
 */
public final class Engine {
    private final Context context;
    private final Supplier<Object> manifest;

    /**
     * The engine a context computes on.
     *
     * @param context the context every call goes through
     */
    Engine(Context context) {
        this.context = context;
        this.manifest = Lazy.of(() -> Json.read(manifestJson()));
    }

    /**
     * The manifest as the engine wrote it.
     *
     * @return the manifest's JSON
     */
    public String manifestJson() {
        return context.locked(Calls::ephemerisManifest);
    }

    /**
     * The manifest, parsed and remembered: read once per engine, since it
     * changes when the engine does, and an engine does not change under a
     * live context. As {@link Json#read} reads it.
     *
     * @return the manifest
     */
    public Object manifest() {
        return manifest.get();
    }

    private List<Map<?, ?>> functions() {
        List<Map<?, ?>> out = new ArrayList<>();
        if (manifest() instanceof Map<?, ?> top && top.get("functions") instanceof List<?> functions) {
            for (Object function : functions) {
                if (function instanceof Map<?, ?> one) {
                    out.add(one);
                }
            }
        }
        return out;
    }

    /**
     * Every operation the engine offers, in its own order.
     *
     * @return the operations' names
     */
    public List<String> names() {
        List<String> out = new ArrayList<>();
        for (Map<?, ?> function : functions()) {
            Object name = function.get("name");
            out.add(name == null ? "" : String.valueOf(name));
        }
        return Collections.unmodifiableList(out);
    }

    /**
     * What the manifest says about one operation. The parameters carry the
     * role of each, which says which of them a caller supplies and which the
     * engine fills.
     *
     * @param function the operation's name
     * @return its entry in the manifest, as {@link Json#read} reads it, if the engine offers it
     */
    public Optional<Map<?, ?>> signature(String function) {
        for (Map<?, ?> candidate : functions()) {
            if (function.equals(candidate.get("name"))) {
                return Optional.of(candidate);
            }
        }
        return Optional.empty();
    }

    /**
     * Whether the engine offers an operation.
     *
     * @param function the operation's name
     * @return true when its manifest names it
     */
    public boolean offers(String function) {
        return names().contains(function);
    }

    /**
     * Calls an operation by name, with its parameters by the names the
     * manifest gives them. What comes back is the engine's own answer,
     * parsed as {@link Json#read} reads it.
     *
     * @param function the operation's name
     * @param arguments its parameters, as {@link Json#write} writes them
     * @return the engine's answer
     */
    public Object call(String function, Map<String, ?> arguments) {
        return Json.read(callJson(function, Json.write(arguments)));
    }

    /**
     * Calls an operation with arguments already written as JSON, and answers
     * with the engine's own JSON: the form to use when the answer is being
     * handed on rather than read.
     *
     * @param function the operation's name
     * @param argumentsJson its parameters as a JSON object
     * @return the engine's answer as JSON
     */
    public String callJson(String function, String argumentsJson) {
        return context.locked((lib, raw) -> Calls.ephemerisCall(lib, raw, function, argumentsJson));
    }

    /**
     * The engine as its manifest names it, with how many operations it offers.
     *
     * @return a description
     */
    @Override
    public String toString() {
        Object engine = "?";
        Object version = "?";
        if (manifest() instanceof Map<?, ?> top) {
            engine = top.containsKey("engine") ? top.get("engine") : "?";
            version = top.containsKey("version") ? top.get("version") : "?";
        }
        return "Engine(" + engine + " " + version + ", " + names().size() + " operations)";
    }
}
