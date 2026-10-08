package com.teispace.teistro;

import java.util.List;
import java.util.Objects;

/**
 * The grid a provider is asked to fill: one call for the whole of it,
 * never a loop. Cell {@code i * bodies().size() + b} is body {@code b} at
 * instant {@code i}, which is the order the answer's columns are read in.
 *
 * @param scale the scale the instants are on
 * @param frameBits the frame the positions are wanted in, as {@link Teistro#packFrame} packs it
 * @param speeds whether speeds are wanted
 * @param observer the place a topocentric frame is seen from, or null
 * @param jds the instants, on {@code scale}: a copy, the provider's to keep
 * @param bodies the bodies, in the order the cells run
 */
public record PositionQuery(
        TimeScale scale, long frameBits, boolean speeds, Observer observer, double[] jds, List<Body> bodies) {
    /** The query, checked. */
    public PositionQuery {
        Objects.requireNonNull(scale, "scale");
        Objects.requireNonNull(jds, "jds");
        bodies = List.copyOf(bodies);
    }

    /**
     * How many cells the answer must hold.
     *
     * @return the instants times the bodies
     */
    public int cellCount() {
        return jds.length * bodies.size();
    }
}
