package com.teispace.teistro;

import java.util.List;

/**
 * One planet's dispositors, followed until one repeats.
 *
 * @param planet the planet the chain starts from
 * @param dispositors its dispositor, that planet's, and so on; empty for a planet in its own sign
 * @param end where it stops: one of the chart's {@link Dispositions#ends()}
 */
public record DispositorChain(Graha planet, List<Graha> dispositors, DispositorEnd end) {
    /** Keeps the list unmodifiable. */
    public DispositorChain {
        dispositors = List.copyOf(dispositors);
    }
}
