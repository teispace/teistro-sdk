package com.teispace.teistro;

import java.lang.foreign.Arena;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
import java.lang.invoke.MethodHandles;
import java.lang.invoke.MethodType;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

import com.teispace.teistro.ffi.Native;

/**
 * An {@link EphemerisProvider} bound into the port's vtable
 * ({@code 02-architecture/07-binding-architecture.md}, "Ports across the
 * boundary"): two upcall stubs, the capabilities described once, and the
 * memory both point at, all in one arena the context that uses them owns.
 *
 * <p>Two things here are not optional politeness:
 *
 * <ul>
 *   <li>the arena outlives the context: the context is freed first and the
 *       arena closed after it, or the library would hold dangling function
 *       pointers;
 *   <li>every upcall catches everything, because an exception that escapes
 *       an FFM upcall ends the JVM. What a provider threw is kept in
 *       {@link #raised} and thrown on the caller's side; only a code
 *       crosses.
 * </ul>
 *
 * <p>Restricted calls are the host's whole work, so they are allowed for
 * the class: an upcall's pointers arrive with no size, and each is given
 * the size of the struct or column the port's contract says it is.
 */
@SuppressWarnings("restricted")
final class HostProvider {
    private final EphemerisProvider provider;
    private final Arena arena = Arena.ofShared();
    private final MemorySegment capabilities;
    private final MemorySegment vtable;
    /** What the provider threw during the call under way, if anything. */
    private Throwable raised;

    HostProvider(Native lib, EphemerisProvider provider) {
        if (provider.name() == null || provider.name().isEmpty()) {
            throw new IllegalArgumentException("a provider must have a name, which every result is stamped with");
        }
        if (provider.bodies() == null || provider.bodies().isEmpty()) {
            throw new IllegalArgumentException("a provider must answer at least one body");
        }
        this.provider = provider;
        this.capabilities = describe(lib);
        Linker linker = Linker.nativeLinker();
        this.vtable = arena.allocate(Native.ProviderVtable.LAYOUT);
        Native.ProviderVtable.STRUCT_SIZE.set(vtable, 0L, (int) Native.ProviderVtable.SIZE);
        Native.ProviderVtable.ABI_VERSION.set(vtable, 0L, Native.VTABLE_ABI_VERSION);
        Native.ProviderVtable.CAPABILITIES.set(vtable, 0L, linker.upcallStub(
                handle("fillCapabilities", MethodType.methodType(int.class, MemorySegment.class, MemorySegment.class)), Native.CAPABILITIES_FN, arena));
        Native.ProviderVtable.POSITIONS.set(vtable, 0L, linker.upcallStub(
                handle("fillPositions",
                        MethodType.methodType(int.class, MemorySegment.class, MemorySegment.class, MemorySegment.class)), Native.POSITIONS_FN, arena));
    }

    /** One of this host's upcalls, bound to it. */
    private MethodHandle handle(String name, MethodType type) {
        try {
            return MethodHandles.lookup().findVirtual(HostProvider.class, name, type).bindTo(this);
        } catch (NoSuchMethodException | IllegalAccessException e) {
            throw new IllegalStateException("the provider's upcall " + name + " is missing", e);
        }
    }

    /**
     * The capabilities, described once: the SDK reads them whenever it asks,
     * and nothing about a provider changes while it is bound.
     */
    private MemorySegment describe(Native lib) {
        List<Body> bodies = provider.bodies();
        MemorySegment ids = arena.allocate(ValueLayout.JAVA_SHORT, bodies.size());
        for (int index = 0; index < bodies.size(); index += 1) {
            ids.setAtIndex(ValueLayout.JAVA_SHORT, index, (short) bodies.get(index).id());
        }
        Frame frame = provider.nativeFrame().orElseGet(() -> Calls.frameCanonical(lib));
        MemorySegment out = arena.allocate(Native.CapabilitiesC.LAYOUT);
        Native.CapabilitiesC.STRUCT_SIZE.set(out, 0L, (int) Native.CapabilitiesC.SIZE);
        Native.CapabilitiesC.SPEEDS.set(out, 0L, (byte) (provider.speeds() ? 1 : 0));
        Native.CapabilitiesC.DETERMINISTIC.set(out, 0L, (byte) (provider.deterministic() ? 1 : 0));
        Native.CapabilitiesC.DISTANCE_UNIT.set(out, 0L, (byte) provider.distanceUnit().id());
        Native.CapabilitiesC.SPEED_MODEL.set(out, 0L, (byte) provider.speedModel().id());
        Native.CapabilitiesC.ASTRONOMY.set(out, 0L, (byte) provider.astronomy().id());
        Native.CapabilitiesC.NAME.set(out, 0L, arena.allocateFrom(provider.name()));
        Native.CapabilitiesC.VERSION.set(out, 0L, arena.allocateFrom(provider.version()));
        Native.CapabilitiesC.DATA_VERSION.set(out, 0L, arena.allocateFrom(provider.dataVersion()));
        Native.CapabilitiesC.JD_MIN.set(out, 0L, provider.jdMin());
        Native.CapabilitiesC.JD_MAX.set(out, 0L, provider.jdMax());
        Native.CapabilitiesC.BODIES.set(out, 0L, ids);
        Native.CapabilitiesC.BODY_COUNT.set(out, 0L, (long) bodies.size());
        Native.CapabilitiesC.NATIVE_FRAME_BITS.set(out, 0L, (int) Calls.framePack(lib, frame));
        return out;
    }

    /** The vtable the context drives the provider through. */
    MemorySegment vtable() {
        return vtable;
    }

    /** The provider bound. */
    EphemerisProvider provider() {
        return provider;
    }

    /** Forgets what the last call raised, before a call that may reach the provider. */
    void clear() {
        raised = null;
    }

    /** What the provider threw during the last call, or null. */
    Throwable raised() {
        return raised;
    }

    /** Releases the stubs and the memory they point at; the context is freed first. */
    void close() {
        arena.close();
    }

    @SuppressWarnings("unused") // reached through the upcall stub
    private int fillCapabilities(MemorySegment userData, MemorySegment out) {
        if (out.equals(MemorySegment.NULL)) {
            return ProviderCode.INVALID.id();
        }
        try {
            // The SDK reads what it asked for out of the struct described
            // once, so a struct of another size is still answered.
            int size = out.reinterpret(Integer.BYTES).get(ValueLayout.JAVA_INT, 0);
            long copied = Math.min(size, Native.CapabilitiesC.SIZE);
            MemorySegment target = out.reinterpret(copied);
            target.copyFrom(capabilities.asSlice(0, copied));
            target.set(ValueLayout.JAVA_INT, 0, size);
            return ProviderCode.OK.id();
        } catch (Throwable error) { // nothing may escape an upcall
            raised = error;
            return ProviderCode.REFUSED.id();
        }
    }

    @SuppressWarnings("unused") // reached through the upcall stub
    private int fillPositions(MemorySegment userData, MemorySegment request, MemorySegment out) {
        if (request.equals(MemorySegment.NULL) || out.equals(MemorySegment.NULL)) {
            return ProviderCode.INVALID.id();
        }
        try {
            return answer(request.reinterpret(Native.PositionRequestC.SIZE),
                    out.reinterpret(Native.PositionColumnsC.SIZE));
        } catch (Throwable error) { // nothing may escape an upcall
            raised = error;
            return ProviderCode.REFUSED.id();
        }
    }

    private int answer(MemorySegment request, MemorySegment out) throws Exception {
        // Nothing is checked before the provider is asked: coverage, the
        // observer, the bodies and the instants are all the port's, on the
        // SDK's side, so an instant outside the span never arrives here.
        PositionQuery query = read(request);
        int cells = query.cellCount();
        if ((long) Native.PositionColumnsC.CAPACITY.get(out, 0L) < cells) {
            return ProviderCode.INVALID.id();
        }
        Optional<PositionAnswer> answered = provider.positions(query);
        // Nothing means "not in that frame"; the SDK asks again in ours.
        if (answered.isEmpty()) {
            return ProviderCode.UNSUPPORTED.id();
        }
        PositionAnswer answer = answered.get();
        // Every column the provider supplied, not only the three it must: a
        // speed column of the wrong length silently padded with zeroes is a
        // wrong answer, and a wrong answer is worse than a refusal.
        String[] names = {"lon", "lat", "dist", "lonSpeed", "latSpeed", "distSpeed", "status", "source"};
        int[] lengths = {length(answer.lon()), length(answer.lat()), length(answer.dist()),
            length(answer.lonSpeed()), length(answer.latSpeed()), length(answer.distSpeed()),
            answer.status() == null ? -1 : answer.status().length,
            answer.source() == null ? -1 : answer.source().length};
        for (int index = 0; index < names.length; index += 1) {
            if (lengths[index] >= 0 && lengths[index] != cells) {
                raised = new IllegalStateException("the provider returned " + lengths[index] + " values in `"
                        + names[index] + "` for " + cells + " cells");
                return ProviderCode.REFUSED.id();
            }
        }
        Native.PositionColumnsC.FRAME_BITS.set(out, 0L,
                (int) (answer.frameBits() == null ? query.frameBits() : answer.frameBits().longValue()));
        write(out, Native.PositionColumnsC.LON_OFFSET, answer.lon(), cells);
        write(out, Native.PositionColumnsC.LAT_OFFSET, answer.lat(), cells);
        write(out, Native.PositionColumnsC.DIST_OFFSET, answer.dist(), cells);
        write(out, Native.PositionColumnsC.LON_SPEED_OFFSET, answer.lonSpeed(), cells);
        write(out, Native.PositionColumnsC.LAT_SPEED_OFFSET, answer.latSpeed(), cells);
        write(out, Native.PositionColumnsC.DIST_SPEED_OFFSET, answer.distSpeed(), cells);
        write(out, Native.PositionColumnsC.STATUS_OFFSET, answer.status(), cells);
        write(out, Native.PositionColumnsC.SOURCE_OFFSET, answer.source(), cells);
        return ProviderCode.OK.id();
    }

    private static int length(double[] column) {
        return column == null ? -1 : column.length;
    }

    /**
     * The request as a Java value. The instants and the ids are the SDK's
     * memory, valid for the length of the call, so they are copied.
     */
    private static PositionQuery read(MemorySegment request) {
        long count = (long) Native.PositionRequestC.JD_COUNT.get(request, 0L);
        long bodyCount = (long) Native.PositionRequestC.BODY_COUNT.get(request, 0L);
        double[] jds = ((MemorySegment) Native.PositionRequestC.JDS.get(request, 0L))
                .reinterpret(count * Double.BYTES).toArray(ValueLayout.JAVA_DOUBLE);
        MemorySegment ids = ((MemorySegment) Native.PositionRequestC.BODIES.get(request, 0L))
                .reinterpret(bodyCount * Short.BYTES);
        List<Body> bodies = new ArrayList<>((int) bodyCount);
        for (long index = 0; index < bodyCount; index += 1) {
            bodies.add(Body.of(Short.toUnsignedInt(ids.getAtIndex(ValueLayout.JAVA_SHORT, index))));
        }
        boolean placed = (byte) Native.PositionRequestC.HAS_OBSERVER.get(request, 0L) != 0;
        return new PositionQuery(
                TimeScale.of((int) Native.PositionRequestC.SCALE.get(request, 0L)),
                Integer.toUnsignedLong((int) Native.PositionRequestC.FRAME_BITS.get(request, 0L)),
                (byte) Native.PositionRequestC.SPEEDS.get(request, 0L) != 0,
                placed ? Observer.of(request.asSlice(Native.PositionRequestC.OBSERVER_OFFSET, Native.ObserverC.SIZE))
                        : null,
                jds,
                bodies);
    }

    /** The column pointer at {@code offset} in the SDK's struct, or null where it asked for none. */
    private static MemorySegment column(MemorySegment out, long offset, int cells, long width) {
        MemorySegment pointer = out.get(ValueLayout.ADDRESS, offset);
        return pointer.equals(MemorySegment.NULL) ? null : pointer.reinterpret(cells * width);
    }

    /** Fills one of the SDK's columns, with zeroes where the provider left it out. */
    private static void write(MemorySegment out, long offset, double[] values, int cells) {
        MemorySegment target = column(out, offset, cells, Double.BYTES);
        if (target == null) {
            return;
        }
        if (values == null) {
            target.fill((byte) 0);
        } else {
            MemorySegment.copy(values, 0, target, ValueLayout.JAVA_DOUBLE, 0, cells);
        }
    }

    private static void write(MemorySegment out, long offset, int[] values, int cells) {
        MemorySegment target = column(out, offset, cells, Integer.BYTES);
        if (target == null) {
            return;
        }
        if (values == null) {
            target.fill((byte) 0);
        } else {
            MemorySegment.copy(values, 0, target, ValueLayout.JAVA_INT, 0, cells);
        }
    }
}
