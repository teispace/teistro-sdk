package com.teispace.teistro;

import java.util.List;

/**
 * The lajjitadi a body holds, is ruled out of, and nothing decides.
 *
 * @param holding The states that hold.
 * @param ruledOut The states that certainly do not hold.
 * @param undecided The states nothing decides: the necessary condition holds and what narrows it
 *     further is not in the chart.
 */
public record Lajjitadi(
        List<AvasthaLajjitadi> holding,
        List<AvasthaLajjitadi> ruledOut,
        List<AvasthaLajjitadi> undecided) {
    /** The value, its lists copied and unmodifiable. */
    public Lajjitadi {
        holding = List.copyOf(holding);
        ruledOut = List.copyOf(ruledOut);
        undecided = List.copyOf(undecided);
    }
}
