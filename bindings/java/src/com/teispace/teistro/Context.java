package com.teispace.teistro;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;
import java.lang.ref.Cleaner;
import java.util.HexFormat;
import java.util.concurrent.locks.ReentrantLock;

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

    /** The handle, apart from the context, so the cleaner can free it. */
    private static final class State implements Runnable {
        private final Native lib;
        private MemorySegment handle;

        State(Native lib, MemorySegment handle) {
            this.lib = lib;
            this.handle = handle;
        }

        @Override
        public void run() {
            if (handle != null) {
                MemorySegment freed = handle;
                handle = null;
                Boundary.call(() -> {
                    lib.ts_context_free.invokeExact(freed);
                    return null;
                });
            }
        }
    }

    private Context(Native lib, MemorySegment handle) {
        this.lib = lib;
        this.state = new State(lib, handle);
        this.cleanable = CLEANER.register(this, state);
    }

    static Context open(Teistro teistro, ContextOptions options) {
        Native lib = teistro.lib();
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
            int status = Boundary.call(() -> (int) lib.ts_context_new.invokeExact(
                    raw, MemorySegment.NULL, MemorySegment.NULL, out, error));
            if (status != Status.OK.id()) {
                // A constructor's record crosses whole and owns its strings,
                // so it is read before it is freed (`owned-error-record`).
                TeistroException refusal = Boundary.refusal(status, error);
                Boundary.call(() -> {
                    lib.ts_error_free.invokeExact(error);
                    return null;
                });
                throw refusal;
            }
            return new Context(lib, out.get(ValueLayout.ADDRESS, 0));
        }
    }

    private MemorySegment handle() {
        if (state.handle == null) {
            throw new IllegalStateException("the context is closed");
        }
        return state.handle;
    }

    /**
     * The exception for a status a context method answered, from this
     * context's own record of it: read under the lock, before anything else
     * touches the context.
     */
    private TeistroException refused(int status) {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment error = Boundary.errorRecord(arena);
            MemorySegment context = handle();
            int read = Boundary.call(() -> (int) lib.ts_context_last_error.invokeExact(context, error));
            if (read != Status.OK.id()) {
                Status code = Status.of(status);
                return new TeistroException(code, code.key(), "", "", "", "");
            }
            return Boundary.refusal(status, error);
        }
    }

    private void check(int status) {
        if (status != Status.OK.id()) {
            throw refused(status);
        }
    }

    /**
     * The profile the context was made with.
     *
     * @return the profile's id
     */
    public String profile() {
        lock.lock();
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment out = arena.allocate(Native.TsStr.LAYOUT);
            MemorySegment context = handle();
            check(Boundary.call(() -> (int) lib.ts_context_profile.invokeExact(context, out)));
            return Boundary.borrowed(out);
        } finally {
            lock.unlock();
        }
    }

    /**
     * The context's settings as canonical JSON: every group and knob, the
     * profile's values with the patch applied.
     *
     * @return the settings
     */
    public String settingsJson() {
        lock.lock();
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment out = arena.allocate(Native.TsString.LAYOUT);
            MemorySegment context = handle();
            check(Boundary.call(() -> (int) lib.ts_context_settings_json.invokeExact(context, out)));
            try {
                return Boundary.owned(out);
            } finally {
                Boundary.call(() -> {
                    lib.ts_string_free.invokeExact(out);
                    return null;
                });
            }
        } finally {
            lock.unlock();
        }
    }

    /**
     * The settings' hash: SHA-256 of the canonical settings, as lower-case
     * hexadecimal, the same in every binding.
     *
     * @return the hash
     */
    public String settingsHash() {
        lock.lock();
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment out = arena.allocate(Native.TsHash.LAYOUT);
            MemorySegment context = handle();
            check(Boundary.call(() -> (int) lib.ts_context_settings_hash.invokeExact(context, out)));
            byte[] bytes = out.asSlice(Native.TsHash.BYTES_OFFSET, 32).toArray(ValueLayout.JAVA_BYTE);
            return HexFormat.of().formatHex(bytes);
        } finally {
            lock.unlock();
        }
    }

    /**
     * A catalogue key's packed id.
     *
     * @param key the key, full ({@code graha.SUN}) or bare where it is unambiguous
     * @return the packed id, the kind in the high half
     * @throws TeistroException with {@link Status#UNSUPPORTED} for an unknown key, with a suggestion
     */
    public int keyId(String key) {
        lock.lock();
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment text = Boundary.cString(arena, key);
            MemorySegment out = arena.allocate(ValueLayout.JAVA_INT);
            MemorySegment context = handle();
            check(Boundary.call(() -> (int) lib.ts_key_parse.invokeExact(context, text, out)));
            return out.get(ValueLayout.JAVA_INT, 0);
        } finally {
            lock.unlock();
        }
    }

    /**
     * A packed id's key.
     *
     * @param id the packed id
     * @return the full key
     */
    public String keyName(int id) {
        lock.lock();
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment out = arena.allocate(Native.TsStr.LAYOUT);
            MemorySegment context = handle();
            check(Boundary.call(() -> (int) lib.ts_key_name.invokeExact(context, id, out)));
            return Boundary.borrowed(out);
        } finally {
            lock.unlock();
        }
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
