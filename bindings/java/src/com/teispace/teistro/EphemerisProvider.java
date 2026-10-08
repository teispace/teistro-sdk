package com.teispace.teistro;

import java.util.List;
import java.util.Optional;

/**
 * An ephemeris the SDK can drive, written in Java.
 *
 * <p>Everything but {@link #name}, {@link #bodies} and {@link #positions}
 * has a default, because a provider that answers the canonical frame with
 * apparent geocentric positions is the common case and should not have to
 * say so. Hand one to {@link ContextOptions.Builder#provider}.
 *
 * <pre>{@code
 * final class StraightLine extends EphemerisProvider {
 *     public String name() { return "straight-line"; }
 *     public List<Body> bodies() { return List.of(Body.SUN); }
 *     public Optional<PositionAnswer> positions(PositionQuery query) {
 *         double[] zeros = new double[query.cellCount()];
 *         return Optional.of(PositionAnswer.of(zeros, zeros, zeros));
 *     }
 * }
 * }</pre>
 *
 * <p>The SDK calls back on the thread that made the call, inside it, while
 * that thread holds the context: one context, one thread at a time, which
 * is the boundary's contract.
 */
public abstract class EphemerisProvider {
    /** A provider; subclass it. */
    protected EphemerisProvider() {}

    /**
     * What the provider is, stamped in every result's provenance.
     *
     * @return its name, never empty
     */
    public abstract String name();

    /**
     * The bodies it answers. A request for another is refused by name
     * before the call reaches {@link #positions}.
     *
     * @return the bodies, at least one
     */
    public abstract List<Body> bodies();

    /**
     * The positions for a whole grid, or empty for "not in that frame", in
     * which case the SDK asks again in {@link #nativeFrame} and completes
     * the rest itself, stamping every step it applied.
     *
     * <p>Compare {@link PositionQuery#frameBits} with the frame you compute
     * in before answering: an answer says it is in the frame that was asked
     * for ({@link PositionAnswer}).
     *
     * <p>Anything thrown reaches the caller of the call that asked: an
     * unchecked exception as itself, a checked one as the cause of a
     * {@link ProviderException}. Only a code crosses the C boundary, so the
     * binding keeps the exception and throws it on the caller's side.
     *
     * @param query the grid: every instant and every body, in one call
     * @return the answer, or empty for a frame this provider does not compute
     * @throws Exception whatever the provider's own sources throw
     */
    public abstract Optional<PositionAnswer> positions(PositionQuery query) throws Exception;

    /**
     * Its version.
     *
     * @return the version; empty by default
     */
    public String version() {
        return "";
    }

    /**
     * What identifies its data, an ephemeris file's edition. A result's
     * provenance carries it, so two runs against different data are told
     * apart even when the code is the same.
     *
     * @return the data version; empty by default
     */
    public String dataVersion() {
        return "";
    }

    /**
     * The first Julian day it covers (UT1). An instant outside the span is
     * never asked for: its cells come back {@code OUT_OF_RANGE} and the
     * rest of the batch is answered.
     *
     * @return the first day; year 0 by default
     */
    public double jdMin() {
        return 1721057.5;
    }

    /**
     * The last Julian day it covers (UT1).
     *
     * @return the last day; year 3000 by default
     */
    public double jdMax() {
        return 2816787.5;
    }

    /**
     * The frame it returns natively. A request in another frame reaches
     * {@link #positions} first, and answering empty there has the SDK ask
     * again in this one.
     *
     * @return the frame; empty for the SDK's canonical frame, the default
     */
    public Optional<Frame> nativeFrame() {
        return Optional.empty();
    }

    /**
     * Whether it computes speeds.
     *
     * @return true by default
     */
    public boolean speeds() {
        return true;
    }

    /**
     * Whether identical requests give identical bits. A provider that is
     * not deterministic must say so, because the conformance contract
     * rests on it.
     *
     * @return true by default
     */
    public boolean deterministic() {
        return true;
    }

    /**
     * What its distances are measured in.
     *
     * @return {@code ASTRONOMICAL_UNITS} by default
     */
    public DistanceUnit distanceUnit() {
        return DistanceUnit.ASTRONOMICAL_UNITS;
    }

    /**
     * What its speeds are.
     *
     * @return {@code DERIVATIVE} by default
     */
    public SpeedModel speedModel() {
        return SpeedModel.DERIVATIVE;
    }

    /**
     * Whether it is modern astronomy or a classical text's model.
     *
     * @return {@code MODERN} by default
     */
    public Astronomy astronomy() {
        return Astronomy.MODERN;
    }
}
