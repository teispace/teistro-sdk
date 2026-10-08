package com.teispace.teistro;

import java.lang.foreign.MemorySegment;
import java.lang.foreign.Arena;
import java.lang.foreign.ValueLayout;
import java.lang.ref.Cleaner;
import java.util.HashMap;
import java.util.HexFormat;
import java.util.List;
import java.util.Map;
import java.util.function.Supplier;
import java.util.concurrent.locks.ReentrantLock;

import com.teispace.teistro.blob.Positions;
import com.teispace.teistro.ffi.Native;

/**
 * A context: a profile, its settings and a locale, over an ephemeris. Every
 * computation is a method of one.
 *
 * <p>Close it, with {@code try}-with-resources: a {@link Cleaner} frees a
 * context nobody closed, but only when the collector happens to run.
 *
 * <p>A context is used by one thread at a time and may move between
 * threads; each call holds the context's lock, so two threads that share
 * one wait for each other rather than race on the library's state.
 */
public final class Context implements AutoCloseable {
    private static final Cleaner CLEANER = Cleaner.create();

    private final Native lib;
    private final State state;
    private final Cleaner.Cleanable cleanable;
    private final ReentrantLock lock = new ReentrantLock();
    private final CalendarArea calendar = new CalendarArea(this);
    private final TimeArea time = new TimeArea(this);
    private final IntlArea intl = new IntlArea(this);
    private final KeysArea keys = new KeysArea(this);
    private final ChartArea chart = new ChartArea(this);
    private final AlmanacArea almanac = new AlmanacArea(this);
    private final MatchingArea matching = new MatchingArea(this);
    private final NumerologyArea numerology = new NumerologyArea(this);
    private final Engine ephemeris = new Engine(this);
    private final FrameArea frame;

    /** The handle, apart from the context, so the cleaner can free it. */
    private static final class State implements Runnable {
        private final Native lib;
        private MemorySegment handle;
        /** The provider written in Java, released after the handle. */
        private final HostProvider host;

        State(Native lib, MemorySegment handle, HostProvider host) {
            this.lib = lib;
            this.handle = handle;
            this.host = host;
        }

        @Override
        public void run() {
            if (handle != null) {
                MemorySegment freed = handle;
                handle = null;
                try {
                    Boundary.call(() -> {
                        lib.ts_context_free.invokeExact(freed);
                        return null;
                    });
                } finally {
                    // Only once the library holds no pointer into it.
                    if (host != null) {
                        host.close();
                    }
                }
            }
        }
    }

    private final Supplier<Map<Integer, String>> dashaNames;

    private Context(Teistro teistro, MemorySegment handle, String dashasJson, HostProvider host) {
        this.lib = teistro.lib();
        this.dashaNames = Lazy.of(() -> registered(dashasJson, "dasha_system."));
        this.frame = new FrameArea(teistro);
        this.state = new State(this.lib, handle, host);
        this.cleanable = CLEANER.register(this, state);
    }

    /** The keys of the dasha systems this context registered, by the id a batch carries. */
    Map<Integer, String> dashaNames() {
        return dashaNames.get();
    }

    /**
     * What a context registered of one kind, turned round so a batch names a
     * registered id by its full key: each row's {@code key} under the kind's
     * prefix, resolved by this context.
     */
    private Map<Integer, String> registered(String json, String prefix) {
        if (json == null) {
            return Map.of();
        }
        Map<Integer, String> names = new HashMap<>();
        if (Json.read(json) instanceof List<?> rows) {
            for (Object row : rows) {
                if (row instanceof Map<?, ?> fields && fields.get("key") instanceof String key) {
                    String full = prefix + key;
                    names.put((int) (keys().id(full) & 0xFFFF), full);
                }
            }
        }
        return Map.copyOf(names);
    }

    static Context open(Teistro teistro, ContextOptions options) {
        Native lib = teistro.lib();
        HostProvider host = options.provider() == null ? null : new HostProvider(lib, options.provider());
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment raw = arena.allocate(Native.TsContextOptions.LAYOUT);
            Native.TsContextOptions.STRUCT_SIZE.set(raw, 0L, (int) Native.TsContextOptions.SIZE);
            Native.TsContextOptions.FLAGS.set(raw, 0L, options.testProvider() ? Native.TS_CONTEXT_TEST_PROVIDER : 0);
            Native.TsContextOptions.PROFILE.set(raw, 0L, Boundary.cString(arena, options.profile()));
            Native.TsContextOptions.SETTINGS_JSON.set(raw, 0L, Boundary.cString(arena, options.settingsJson()));
            Native.TsContextOptions.LOCALE.set(raw, 0L, Boundary.cString(arena, options.locale()));
            Native.TsContextOptions.LAYOUTS_JSON.set(raw, 0L, Boundary.cString(arena, options.layoutsJson()));
            Native.TsContextOptions.DASHAS_JSON.set(raw, 0L, Boundary.cString(arena, options.dashasJson()));
            Native.TsContextOptions.EPHEMERIS.set(raw, 0L, (byte) options.ephemeris().id());
            MemorySegment out = arena.allocate(ValueLayout.ADDRESS);
            MemorySegment error = Boundary.errorRecord(arena);
            // A typed local: `invokeExact` reads a conditional as `Object`.
            MemorySegment vtable = host == null ? MemorySegment.NULL : host.vtable();
            int status = Boundary.call(() -> (int) lib.ts_context_new.invokeExact(
                    raw, vtable, MemorySegment.NULL, out, error));
            if (status != Status.OK.id()) {
                if (host != null) {
                    host.close();
                }
                // A constructor's record crosses whole and owns its strings,
                // so it is read before it is freed (`owned-error-record`).
                TeistroException refusal = Boundary.refusal(status, error);
                Boundary.call(() -> {
                    lib.ts_error_free.invokeExact(error);
                    return null;
                });
                throw refusal;
            }
            return new Context(teistro, out.get(ValueLayout.ADDRESS, 0), options.dashasJson(), host);
        }
    }

    private MemorySegment handle() {
        if (state.handle == null) {
            throw new IllegalStateException("the context is closed");
        }
        return state.handle;
    }

    /** A call on the live handle, under the context's lock. */
    @FunctionalInterface
    interface Call<T> {
        T run(Native lib, MemorySegment context);
    }

    /**
     * Runs a call on the live handle while holding the lock, so a refusal is
     * read from the record of this call and not a later one.
     */
    <T> T locked(Call<T> call) {
        lock.lock();
        try {
            HostProvider host = state.host;
            if (host == null) {
                return call.run(lib, handle());
            }
            host.clear();
            try {
                return call.run(lib, handle());
            } catch (TeistroException refusal) {
                throw thrown(host.raised(), refusal);
            }
        } finally {
            lock.unlock();
        }
    }

    /**
     * What a provider written in Java threw, thrown on the caller's side,
     * with the library's refusal kept beside it; the refusal itself when the
     * provider threw nothing. Only a code crosses the C boundary, so without
     * this a provider's own message would be lost.
     */
    private static RuntimeException thrown(Throwable raised, TeistroException refusal) {
        if (raised == null) {
            return refusal;
        }
        raised.addSuppressed(refusal);
        if (raised instanceof RuntimeException unchecked) {
            return unchecked;
        }
        if (raised instanceof Error error) {
            throw error;
        }
        return new ProviderException((Exception) raised);
    }

    /**
     * The profile the context was made with.
     *
     * @return the profile's id
     */
    public String profile() {
        return locked(Calls::profile);
    }

    /**
     * The context's settings, parsed: every group and knob, as
     * {@link Json#read} reads them.
     *
     * @return the settings
     */
    public Object settings() {
        return Json.read(settingsJson());
    }

    /**
     * The context's settings as canonical JSON: every group and knob, the
     * profile's values with the patch applied.
     *
     * @return the settings
     */
    public String settingsJson() {
        return locked(Calls::settingsJson);
    }

    /**
     * The settings' hash: SHA-256 of the canonical settings, as lower-case
     * hexadecimal, the same in every binding.
     *
     * @return the hash
     */
    public String settingsHash() {
        return HexFormat.of().formatHex(locked(Calls::settingsHash).bytes());
    }

    /**
     * The calendars: dates by fixed day, conversion, month lengths, leap
     * years and weekdays.
     *
     * @return the calendar area of this context
     */
    public CalendarArea calendar() {
        return calendar;
    }

    /**
     * The time scales and zones.
     *
     * @return the time area of this context
     */
    public TimeArea time() {
        return time;
    }

    /**
     * The locale, its messages and the scripts they are in.
     *
     * @return the intl area of this context
     */
    public IntlArea intl() {
        return intl;
    }

    /**
     * Catalogue keys and their packed ids.
     *
     * @return the keys area of this context
     */
    public KeysArea keys() {
        return keys;
    }

    /**
     * Charts founded at an instant and a place.
     *
     * @return the chart area of this context
     */
    public ChartArea chart() {
        return chart;
    }

    /**
     * The almanac: a span of days at a place, with its muhurtas, festivals,
     * years and eclipses on request.
     *
     * @return the almanac area of this context
     */
    public AlmanacArea almanac() {
        return almanac;
    }

    /**
     * Two births compared: the naam milan from names alone.
     *
     * @return the matching area of this context
     */
    public MatchingArea matching() {
        return matching;
    }

    /**
     * A name and a civil date read under the numerology tables.
     *
     * @return the numerology area of this context
     */
    public NumerologyArea numerology() {
        return numerology;
    }

    /**
     * The plugged-in engine's own functions, reached through the SDK.
     *
     * @return the engine of this context
     */
    public Engine ephemeris() {
        return ephemeris;
    }

    /**
     * The coordinate conventions a request is expressed in.
     *
     * @return the frame area
     */
    public FrameArea frame() {
        return frame;
    }

    /**
     * The positions of a grid of bodies at a grid of instants. The cells run
     * instants outermost: cell {@code i * bodies + j} is instant {@code i},
     * body {@code j}.
     *
     * @param request the instants, bodies, scale, frame and observer
     * @return the grid, read cell by cell with {@link PositionGrid#at}
     */
    public PositionGrid positions(PositionRequest request) {
        return new PositionGrid(Positions.decode(locked((lib, raw) -> Calls.positions(lib, raw, request))));
    }

    /**
     * The positions of bodies at instants on UT1, in the canonical frame,
     * with speeds, seen from the Earth's centre.
     *
     * @param instants the instants as Julian days, at least one
     * @param bodies the bodies, at least one
     * @return the grid, read cell by cell with {@link PositionGrid#at}
     */
    public PositionGrid positions(double[] instants, java.util.List<Body> bodies) {
        if (instants.length == 0) {
            throw new IllegalArgumentException("a request needs at least one instant");
        }
        if (bodies.isEmpty()) {
            throw new IllegalArgumentException("a request needs at least one body");
        }
        long frame = Calls.framePack(lib, Calls.frameCanonical(lib));
        return positions(new PositionRequest(TimeScale.UT1, frame, true, null, instants, bodies));
    }

    /**
     * The provider written in Java this context asks, if it was made with one.
     *
     * @return the provider
     */
    public java.util.Optional<EphemerisProvider> provider() {
        return java.util.Optional.ofNullable(state.host).map(HostProvider::provider);
    }

    /** Frees the context. Closing twice does nothing. */
    @Override
    public void close() {
        lock.lock();
        try {
            cleanable.clean();
        } finally {
            lock.unlock();
        }
    }
}
