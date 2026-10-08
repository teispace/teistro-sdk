package com.teispace.teistro;

import java.util.List;

/**
 * One graha's dasha phala (BPHS ch. 28 vv. 7 to 10, ch. 47 vv. 3 to 6).
 *
 * @param graha Which graha, Sun to Ketu.
 * @param subhankas Its Subhanka in the D1, D2, D3, D7, D9, D12 and D30: out of 60 in the first and
 *     30 in the rest.
 * @param subhanka The seven together, out of 240.
 * @param asubhanka Their complements together, out of 240.
 * @param nature Whether its rasi place is auspicious (benefic), neutral or inauspicious (malefic).
 * @param phase Where in its dasha its effects come.
 * @param favourable Whether its placement makes its dasha favourable.
 * @param unfavourable Whether its placement makes its dasha unfavourable; both can hold.
 */
public record GrahaDashaPhala(
        Graha graha,
        List<Double> subhankas,
        double subhanka,
        double asubhanka,
        Nature nature,
        DashaPhase phase,
        boolean favourable,
        boolean unfavourable) {
    /** The value, its lists copied and unmodifiable. */
    public GrahaDashaPhala {
        subhankas = List.copyOf(subhankas);
    }
}
