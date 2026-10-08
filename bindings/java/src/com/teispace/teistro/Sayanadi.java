package com.teispace.teistro;

import java.util.List;

/**
 * A graha's Sayanadi state, with its sub-state under a name of each anka (BPHS ch. 45 vv. 30 to
 * 37).
 *
 * @param avastha The state, Shayana to Nidra.
 * @param cheshtas The sub-state under a name whose first syllable's anka is 1 to 5, in that order.
 */
public record Sayanadi(
        AvasthaSayanadi avastha,
        List<AvasthaCheshta> cheshtas) {
    /** The value, its lists copied and unmodifiable. */
    public Sayanadi {
        cheshtas = List.copyOf(cheshtas);
    }

    /**
     * The sub-state under a name of this anka.
     *
     * <pre>{@code state.sayanadi().cheshta(3)}</pre>
     *
     * @param anka the anka of the name's first syllable, 1 to 5
     * @return the sub-state
     * @throws IllegalArgumentException outside 1 to 5
     */
    public AvasthaCheshta cheshta(int anka) {
        if (anka < 1 || anka > 5) {
            throw new IllegalArgumentException("anka: " + anka + " is not a syllable's anka; it is 1 to 5");
        }
        return cheshtas.get(anka - 1);
    }
}
