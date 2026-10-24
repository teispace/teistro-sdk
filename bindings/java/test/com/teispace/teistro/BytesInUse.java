package com.teispace.teistro;

import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemoryLayout;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SegmentAllocator;
import java.lang.foreign.StructLayout;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
import java.util.Arrays;
import java.util.Locale;
import java.util.Optional;
import java.util.function.LongSupplier;

/**
 * The C allocator's own count of the bytes in use, which the library's
 * allocations go through, read as the Python binding's
 * {@code tests/test_leaks.py} reads it ({@code docs/05-testing/01-quality-bar.md},
 * "leak checks per binding"): {@code malloc_zone_statistics} on macOS and
 * glibc's {@code mallinfo2} on Linux. The resident set will not do, since
 * macOS compresses idle pages and a process keeping its contexts shrinks.
 *
 * <p>Each family runs its cycles once to settle, then {@link #RUNS} times
 * measured, and at least one measured run must leave the bytes in use within
 * {@link #BUDGET} of where it began: a leak grows them in every run, while
 * what the JVM allocates for itself moves them a little, and now and then
 * once. Unread on Windows, on musl, and on Intel macOS, whose zones'
 * {@code size_in_use} is not the bytes live (the Python test says why).
 */
final class BytesInUse {
    /** How far the bytes in use may move over a measured run. */
    static final long BUDGET = 64 * 1024;
    /** The cycles of one run. */
    static final int CYCLES = 500;
    /** The measured runs of each family. */
    static final int RUNS = 3;

    private BytesInUse() {}

    /** macOS's {@code malloc_statistics_t}. */
    private static final StructLayout ZONE_STATISTICS = MemoryLayout.structLayout(
            ValueLayout.JAVA_INT.withName("blocks_in_use"),
            MemoryLayout.paddingLayout(4),
            ValueLayout.JAVA_LONG.withName("size_in_use"),
            ValueLayout.JAVA_LONG.withName("max_size_in_use"),
            ValueLayout.JAVA_LONG.withName("size_allocated"));

    /** glibc's {@code struct mallinfo2}: ten {@code size_t}s. */
    private static final StructLayout MALLINFO2 = MemoryLayout.structLayout(
            Arrays.stream(new String[] {"arena", "ordblks", "smblks", "hblks", "hblkhd", "usmblks", "fsmblks",
                    "uordblks", "fordblks", "keepcost"}).map(ValueLayout.JAVA_LONG::withName)
                    .toArray(MemoryLayout[]::new));

    /** How to read the bytes in use, where the allocator says: two downcalls into libc. */
    @SuppressWarnings("restricted")
    static Optional<LongSupplier> reader() {
        String os = System.getProperty("os.name").toLowerCase(Locale.ROOT);
        String arch = System.getProperty("os.arch").toLowerCase(Locale.ROOT);
        Linker linker = Linker.nativeLinker();
        if (os.contains("mac")) {
            if (arch.equals("x86_64") || arch.equals("amd64")) {
                return Optional.empty();
            }
            return linker.defaultLookup().find("malloc_zone_statistics").map(symbol -> {
                MethodHandle statistics = linker.downcallHandle(symbol,
                        FunctionDescriptor.ofVoid(ValueLayout.ADDRESS, ValueLayout.ADDRESS));
                return () -> {
                    try (Arena arena = Arena.ofConfined()) {
                        MemorySegment stats = arena.allocate(ZONE_STATISTICS);
                        statistics.invokeExact(MemorySegment.NULL, stats);
                        return stats.get(ValueLayout.JAVA_LONG,
                                ZONE_STATISTICS.byteOffset(MemoryLayout.PathElement.groupElement("size_in_use")));
                    } catch (Throwable e) {
                        throw new AssertionError("malloc_zone_statistics", e);
                    }
                };
            });
        }
        if (os.contains("linux")) {
            return linker.defaultLookup().find("mallinfo2").map(symbol -> {
                MethodHandle mallinfo2 = linker.downcallHandle(symbol, FunctionDescriptor.of(MALLINFO2));
                long small = MALLINFO2.byteOffset(MemoryLayout.PathElement.groupElement("uordblks"));
                long mapped = MALLINFO2.byteOffset(MemoryLayout.PathElement.groupElement("hblkhd"));
                return () -> {
                    try (Arena arena = Arena.ofConfined()) {
                        MemorySegment info = (MemorySegment) mallinfo2.invokeExact((SegmentAllocator) arena);
                        // The small blocks in use and the large ones mapped on their own.
                        return info.get(ValueLayout.JAVA_LONG, small) + info.get(ValueLayout.JAVA_LONG, mapped);
                    } catch (Throwable e) {
                        throw new AssertionError("mallinfo2", e);
                    }
                };
            });
        }
        return Optional.empty();
    }

    /** Collects what is unreachable and lets the cleaner free what it held. */
    private static void settle() throws InterruptedException {
        for (int pass = 0; pass < 3; pass += 1) {
            System.gc();
            Thread.sleep(20);
        }
    }

    /**
     * At least one of {@link #RUNS} runs of {@code cycle} grows the bytes in
     * use within {@link #BUDGET}. A failure names every run's growth and one
     * run's over four times the cycles: a leak grows with the cycles, and what
     * the allocator keeps for itself does not.
     */
    static void givesBack(LongSupplier inUse, Runnable cycle) throws InterruptedException {
        long[] grown = new long[RUNS];
        java.util.function.IntToLongFunction run = count -> {
            try {
                settle();
                long before = inUse.getAsLong();
                for (int at = 0; at < count; at += 1) {
                    cycle.run();
                }
                settle();
                return inUse.getAsLong() - before;
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
                throw new AssertionError("interrupted", e);
            }
        };
        run.applyAsLong(CYCLES);
        long least = Long.MAX_VALUE;
        for (int at = 0; at < RUNS; at += 1) {
            grown[at] = run.applyAsLong(CYCLES);
            least = Math.min(least, grown[at]);
        }
        if (least <= BUDGET) {
            return;
        }
        long longer = run.applyAsLong(4 * CYCLES);
        throw new AssertionError("the bytes in use grew " + Arrays.toString(grown) + " over "
                + RUNS + " runs of " + CYCLES + " cycles (" + least / CYCLES + " a cycle at least), and " + longer
                + " over " + 4 * CYCLES + "; the budget is " + BUDGET);
    }
}
