package com.teispace.teistro;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.nio.charset.StandardCharsets;

import com.teispace.teistro.ffi.Native;

/**
 * What every call across the boundary shares: reading the strings the
 * library lends and owns, the error record, and turning a method handle's
 * checked {@code Throwable} into what the API throws.
 */
final class Boundary {
    private Boundary() {}

    /** A call through a method handle. */
    @FunctionalInterface
    interface Call<T> {
        T run() throws Throwable;
    }

    /**
     * Runs a call, rethrowing what the boundary throws as itself and anything
     * else, which only a broken declaration could cause, as an error.
     */
    static <T> T call(Call<T> call) {
        try {
            return call.run();
        } catch (RuntimeException | Error e) {
            throw e;
        } catch (Throwable e) {
            throw new IllegalStateException("a call into the library failed: " + e, e);
        }
    }

    /** A NUL-terminated UTF-8 string the library lent, or an empty string for NULL. */
    // A pointer the library returned has no size until it is given one.
    @SuppressWarnings("restricted")
    static String text(MemorySegment pointer) {
        if (pointer.equals(MemorySegment.NULL)) {
            return "";
        }
        return pointer.reinterpret(Long.MAX_VALUE).getString(0, StandardCharsets.UTF_8);
    }

    /** A borrowed string, a {@code ts_str}: a pointer and a length, copied out now. */
    static String borrowed(MemorySegment str) {
        MemorySegment data = (MemorySegment) Native.TsStr.DATA.get(str, 0L);
        long len = (long) Native.TsStr.LEN.get(str, 0L);
        return bytes(data, len);
    }

    /** An owned string, a {@code ts_string}: copied out, the caller then frees it. */
    static String owned(MemorySegment string) {
        MemorySegment data = (MemorySegment) Native.TsString.DATA.get(string, 0L);
        long len = (long) Native.TsString.LEN.get(string, 0L);
        return bytes(data, len);
    }

    @SuppressWarnings("restricted")
    private static String bytes(MemorySegment data, long len) {
        if (len == 0 || data.equals(MemorySegment.NULL)) {
            return "";
        }
        byte[] copied = data.reinterpret(len).toArray(java.lang.foreign.ValueLayout.JAVA_BYTE);
        return new String(copied, StandardCharsets.UTF_8);
    }

    /** A string as the library takes one: NUL-terminated UTF-8, or NULL for null. */
    static MemorySegment cString(Arena arena, String text) {
        return text == null ? MemorySegment.NULL : arena.allocateFrom(text, StandardCharsets.UTF_8);
    }

    /** An error record, sized for the library to fill. */
    static MemorySegment errorRecord(Arena arena) {
        MemorySegment error = arena.allocate(Native.TsError.LAYOUT);
        Native.TsError.STRUCT_SIZE.set(error, 0L, (int) Native.TsError.SIZE);
        return error;
    }

    /** The exception a filled error record says, for the status the call answered. */
    static TeistroException refusal(int status, MemorySegment error) {
        Status code = Status.of(status);
        String message = text((MemorySegment) Native.TsError.MESSAGE.get(error, 0L));
        return new TeistroException(
                code,
                message.isEmpty() ? code.key() : message,
                text((MemorySegment) Native.TsError.DETAIL.get(error, 0L)),
                text((MemorySegment) Native.TsError.FIELD.get(error, 0L)),
                text((MemorySegment) Native.TsError.HINT.get(error, 0L)),
                text((MemorySegment) Native.TsError.KEY.get(error, 0L)));
    }
}
