package com.teispace.teistro;

import java.util.List;

/**
 * Where chains of dispositors stop.
 *
 * @param kind a planet in its own sign, or a cycle
 * @param planets the planet in its own sign, or the cycle's from its slowest member
 */
public record DispositorEnd(ChainEnd kind, List<Graha> planets) {
    /** Keeps the list unmodifiable. */
    public DispositorEnd {
        planets = List.copyOf(planets);
    }
}
